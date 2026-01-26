pub mod certificate;
pub mod settings;
pub mod tls;
pub mod truststore;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use astrum_db::setting;
use astrum_security::SecretBox;
use astrum_security::ssrf::SsrfProtection;
use rustls::pki_types::CertificateDer;
use sea_orm::DatabaseConnection;

pub use settings::{HttpSettings, PLACEHOLDER, ProxySettings};

const REFRESH: Duration = Duration::from_secs(10);
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
pub const DEFAULT_RETRIES: i32 = 2;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Settings(#[from] astrum_db::Error),
    #[error("the database refused a query: {0}")]
    Database(#[from] sea_orm::DbErr),
}

#[derive(Debug, Default)]
pub struct Snapshot {
    pub http: HttpSettings,
    pub trust: Vec<CertificateDer<'static>>,
    pub fingerprint: String,
}

#[derive(Clone)]
pub struct Outbound {
    inner: Arc<Inner>,
}

struct Inner {
    db: DatabaseConnection,
    secrets: SecretBox,
    cache: Mutex<Option<(Instant, Arc<Snapshot>)>>,
}

impl std::fmt::Debug for Outbound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Outbound")
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClientOptions {
    pub user_agent_suffix: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub use_trust_store: bool,
}

impl Outbound {
    pub fn new(db: DatabaseConnection, secrets: SecretBox) -> Self {
        Self {
            inner: Arc::new(Inner {
                db,
                secrets,
                cache: Mutex::new(None),
            }),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn secrets(&self) -> &SecretBox {
        &self.inner.secrets
    }

    pub fn forget(&self) {
        *self
            .inner
            .cache
            .lock()
            .expect("the outbound cache lock is never poisoned") = None;
    }

    pub async fn http_settings(&self) -> Result<HttpSettings, Error> {
        Ok(setting::get(self.db(), settings::HTTP_KEY)
            .await?
            .unwrap_or_default())
    }

    pub async fn save_http_settings(&self, http: &HttpSettings) -> Result<(), Error> {
        setting::put(self.db(), settings::HTTP_KEY, http).await?;
        self.forget();
        Ok(())
    }

    pub async fn snapshot(&self) -> Result<Arc<Snapshot>, Error> {
        if let Some((at, snapshot)) = self
            .inner
            .cache
            .lock()
            .expect("the outbound cache lock is never poisoned")
            .clone()
            && at.elapsed() < REFRESH
        {
            return Ok(snapshot);
        }
        let http = self.http_settings().await?;
        let stored = truststore::list(self.db()).await?;
        let fingerprint = format!(
            "{}|{}",
            serde_json::to_string(&http).unwrap_or_default(),
            stored
                .iter()
                .map(|stored| stored.certificate.id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        let snapshot = Arc::new(Snapshot {
            http,
            trust: stored
                .into_iter()
                .map(|stored| CertificateDer::from(stored.der))
                .collect(),
            fingerprint,
        });
        *self
            .inner
            .cache
            .lock()
            .expect("the outbound cache lock is never poisoned") =
            Some((Instant::now(), snapshot.clone()));
        Ok(snapshot)
    }

    pub async fn trust_roots(&self) -> Result<Vec<CertificateDer<'static>>, Error> {
        Ok(self.snapshot().await?.trust.clone())
    }
}

impl Snapshot {
    pub fn retries(&self, own: Option<i32>) -> usize {
        own.or(self.http.retries)
            .unwrap_or(DEFAULT_RETRIES)
            .clamp(0, settings::MAX_RETRIES) as usize
    }

    pub fn timeout(&self, own: Option<i32>) -> Duration {
        let seconds = own
            .filter(|seconds| *seconds > 0)
            .or(self.http.timeout.filter(|seconds| *seconds > 0))
            .map(|seconds| seconds as u64)
            .unwrap_or(DEFAULT_TIMEOUT_SECONDS);
        Duration::from_secs(seconds)
    }

    pub fn user_agent(&self, own: Option<&str>) -> String {
        let mut agent = format!("Astrum/{}", env!("CARGO_PKG_VERSION"));
        for part in [self.http.user_agent.as_deref(), own]
            .into_iter()
            .flatten()
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            agent.push(' ');
            agent.push_str(part);
        }
        agent
    }

    fn proxy_for(&self, url: &url::Url) -> Option<&ProxySettings> {
        let host = url.host_str()?;
        if settings::bypasses(&self.http.non_proxy_hosts, host) {
            return None;
        }
        let http = Some(&self.http.http_proxy).filter(|proxy| proxy.enabled);
        let https = Some(&self.http.https_proxy).filter(|proxy| proxy.enabled);
        match url.scheme() {
            "http" => http,
            "https" => https.or(http),
            _ => None,
        }
    }

    pub fn proxied(&self, url: &url::Url) -> bool {
        self.proxy_for(url).is_some()
    }

    pub fn guard_settings(&self, ssrf: &SsrfProtection) -> SsrfProtection {
        let mut adjusted = ssrf.clone();
        for proxy in [&self.http.http_proxy, &self.http.https_proxy] {
            if !proxy.enabled {
                continue;
            }
            let Some(host) = proxy.host.as_deref() else {
                continue;
            };
            match host.parse::<std::net::IpAddr>() {
                Ok(ip) => adjusted.allowed_ips.push(ip.to_string()),
                Err(_) => adjusted.allowed_domains.push(host.to_ascii_lowercase()),
            }
        }
        adjusted
    }

    fn proxies(&self, secrets: &SecretBox) -> Vec<reqwest::Proxy> {
        let mut proxies = Vec::new();
        let https_enabled = self.http.https_proxy.enabled;
        for (proxy, schemes) in [
            (&self.http.https_proxy, &["https"][..]),
            (
                &self.http.http_proxy,
                if https_enabled {
                    &["http"][..]
                } else {
                    &["http", "https"][..]
                },
            ),
        ] {
            let Some(address) = proxy.url() else {
                continue;
            };
            let Ok(target) = url::Url::parse(&address) else {
                continue;
            };
            let schemes: Vec<&'static str> = schemes.to_vec();
            let bypass = self.http.non_proxy_hosts.clone();
            let mut built = reqwest::Proxy::custom(move |url| {
                let host = url.host_str()?;
                if !schemes.contains(&url.scheme()) || settings::bypasses(&bypass, host) {
                    return None;
                }
                Some(target.clone())
            });
            let auth = &proxy.auth_info;
            if auth.enabled
                && let Some(username) = auth.username.as_deref()
            {
                let password = auth
                    .password
                    .as_deref()
                    .and_then(|sealed| secrets.open(sealed).ok())
                    .unwrap_or_default();
                built = built.basic_auth(username, &password);
            }
            proxies.push(built);
        }
        proxies
    }

    pub fn configure(
        &self,
        builder: reqwest::ClientBuilder,
        secrets: &SecretBox,
        options: &ClientOptions,
    ) -> reqwest::ClientBuilder {
        let timeout = self.timeout(options.timeout_seconds);
        let mut builder = builder
            .user_agent(self.user_agent(options.user_agent_suffix.as_deref()))
            .connect_timeout(timeout)
            .read_timeout(timeout)
            .no_proxy();
        for proxy in self.proxies(secrets) {
            builder = builder.proxy(proxy);
        }
        if options.use_trust_store {
            match tls::client_config(&self.trust, true) {
                Ok(mut config) => {
                    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                    builder = builder.use_preconfigured_tls(config);
                }
                Err(error) => {
                    tracing::warn!(%error, "the truststore cannot be used; the system roots apply")
                }
            }
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn snapshot(http: serde_json::Value) -> Snapshot {
        Snapshot {
            http: serde_json::from_value(http).unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn repository_values_override_the_global_ones() {
        let global = snapshot(json!({"userAgent": "Corp/1", "timeout": 5, "retries": 4}));
        assert_eq!(global.timeout(None), Duration::from_secs(5));
        assert_eq!(global.timeout(Some(9)), Duration::from_secs(9));
        assert_eq!(global.retries(None), 4);
        assert_eq!(global.retries(Some(0)), 0);
        assert!(
            global
                .user_agent(Some("Mirror"))
                .ends_with(" Corp/1 Mirror")
        );
        let plain = Snapshot::default();
        assert_eq!(
            plain.timeout(None),
            Duration::from_secs(DEFAULT_TIMEOUT_SECONDS)
        );
        assert_eq!(plain.retries(None), 2);
        assert_eq!(
            plain.user_agent(None),
            format!("Astrum/{}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn https_falls_back_to_the_http_proxy_and_bypass_hosts_go_direct() {
        let only_http = snapshot(json!({
            "httpProxy": {"enabled": true, "host": "10.0.0.5", "port": "3128"},
            "nonProxyHosts": ["*.internal"]
        }));
        let url = |text: &str| url::Url::parse(text).unwrap();
        assert!(only_http.proxied(&url("http://repo.example/a")));
        assert!(only_http.proxied(&url("https://repo.example/a")));
        assert!(!only_http.proxied(&url("https://repo.internal/a")));
        let guard = only_http.guard_settings(&SsrfProtection::default());
        assert!(guard.allowed_ips.contains(&"10.0.0.5".to_owned()));
        let both = snapshot(json!({
            "httpProxy": {"enabled": true, "host": "proxy.example", "port": "3128"},
            "httpsProxy": {"enabled": false}
        }));
        assert!(
            both.guard_settings(&SsrfProtection::default())
                .allowed_domains
                .contains(&"proxy.example".to_owned())
        );
        assert!(!Snapshot::default().proxied(&url("http://repo.example/a")));
    }
}
