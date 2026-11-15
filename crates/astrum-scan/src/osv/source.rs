use std::fmt::Display;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_content::Content;
use astrum_outbound::certificate::{FetchError, resolve_guarded};
use astrum_outbound::{ClientOptions, Snapshot};
use astrum_security::ssrf::{Guard, GuardedResolver, SSRF_KEY, SsrfProtection, literal_allowed};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use tokio::io::AsyncWriteExt;
use url::Url;
use uuid::Uuid;

use super::{MAX_ARCHIVE_BYTES, valid_ecosystem};

const DEADLINE: Duration = Duration::from_secs(2 * 60 * 60);
const MAX_REDIRECTS: usize = 5;
const MAX_ETAG_CHARS: usize = 256;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("{0}")]
    Refused(String),
    #[error("the advisory server answered {0}")]
    Status(u16),
    #[error("the advisory download failed: {0}")]
    Network(String),
    #[error("the archive is larger than the limit of {} bytes", MAX_ARCHIVE_BYTES)]
    TooLarge,
    #[error("the archive cannot be stored: {0}")]
    Io(#[from] io::Error),
    #[error("the upload was used already")]
    Used,
}

#[derive(Debug)]
pub struct TempFile {
    path: PathBuf,
    size: u64,
}

impl TempFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub async fn from_stream<S, B, E>(
        dir: &Path,
        mut stream: S,
        limit: u64,
    ) -> Result<TempFile, SourceError>
    where
        S: Stream<Item = Result<B, E>> + Unpin,
        B: AsRef<[u8]>,
        E: Display,
    {
        tokio::fs::create_dir_all(dir).await?;
        let path = dir.join(format!("advisories-{}.zip", Uuid::new_v4()));
        let mut file = tokio::fs::File::create(&path).await?;
        let mut staged = TempFile { path, size: 0 };
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| SourceError::Network(error.to_string()))?;
            let chunk = chunk.as_ref();
            staged.size += chunk.len() as u64;
            if staged.size > limit {
                return Err(SourceError::TooLarge);
            }
            file.write_all(chunk).await?;
        }
        file.flush().await?;
        Ok(staged)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != io::ErrorKind::NotFound
        {
            tracing::warn!(%error, path = %self.path.display(), "an advisory archive was not removed");
        }
    }
}

#[derive(Debug)]
pub enum Fetched {
    Data {
        file: TempFile,
        etag: Option<String>,
    },
    NotModified,
}

#[async_trait]
pub trait AdvisorySource: Send + Sync {
    async fn fetch(&self, ecosystem: &str, etag: Option<&str>) -> Result<Fetched, SourceError>;
}

pub struct Upload {
    file: Mutex<Option<TempFile>>,
}

impl Upload {
    pub fn new(file: TempFile) -> Self {
        Self {
            file: Mutex::new(Some(file)),
        }
    }
}

#[async_trait]
impl AdvisorySource for Upload {
    async fn fetch(&self, _ecosystem: &str, _etag: Option<&str>) -> Result<Fetched, SourceError> {
        let file = self
            .file
            .lock()
            .expect("the upload lock is never poisoned")
            .take()
            .ok_or(SourceError::Used)?;
        Ok(Fetched::Data { file, etag: None })
    }
}

pub struct HttpDump {
    content: Content,
    base: Url,
    dir: PathBuf,
}

pub fn parse_base_url(text: &str) -> Result<Url, String> {
    let url = Url::parse(text.trim()).map_err(|error| format!("the URL is not valid: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("the URL must use http or https".to_owned());
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err("the URL has no host".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("the URL must not carry credentials".to_owned());
    }
    if url.cannot_be_a_base() {
        return Err("the URL cannot be a base for paths".to_owned());
    }
    Ok(url)
}

impl HttpDump {
    pub fn new(content: Content, base_url: &str, dir: PathBuf) -> Result<Self, SourceError> {
        let base = parse_base_url(base_url).map_err(SourceError::Refused)?;
        Ok(Self { content, base, dir })
    }

    fn url_for(&self, ecosystem: &str) -> Result<Url, SourceError> {
        if !valid_ecosystem(ecosystem) {
            return Err(SourceError::Refused(format!(
                "{ecosystem:?} is not an ecosystem name"
            )));
        }
        let mut url = self.base.clone();
        url.set_query(None);
        url.set_fragment(None);
        url.path_segments_mut()
            .map_err(|()| SourceError::Refused("the URL cannot be a base for paths".to_owned()))?
            .pop_if_empty()
            .push(ecosystem)
            .push("all.zip");
        Ok(url)
    }

    async fn client(&self) -> Result<(reqwest::Client, Arc<Guard>, Arc<Snapshot>), SourceError> {
        let outbound = self
            .content
            .outbound()
            .snapshot()
            .await
            .map_err(|error| SourceError::Network(error.to_string()))?;
        let ssrf: SsrfProtection = astrum_db::setting::get(self.content.db(), SSRF_KEY)
            .await
            .map_err(|error| SourceError::Network(error.to_string()))?
            .unwrap_or_default();
        let guard = Arc::new(Guard::new(&outbound.guard_settings(&ssrf)));
        let options = ClientOptions {
            user_agent_suffix: None,
            timeout_seconds: None,
            use_trust_store: true,
        };
        let client = outbound
            .configure(reqwest::Client::builder(), self.content.secrets(), &options)
            .redirect(reqwest::redirect::Policy::none())
            .dns_resolver(Arc::new(GuardedResolver::new(guard.clone())))
            .build()
            .map_err(|error| SourceError::Network(error.to_string()))?;
        Ok((client, guard, outbound))
    }

    async fn download(&self, ecosystem: &str, etag: Option<&str>) -> Result<Fetched, SourceError> {
        let mut url = self.url_for(ecosystem)?;
        let (client, guard, outbound) = self.client().await?;
        let mut redirects = 0;
        let response = loop {
            check_target(&guard, &outbound, &url).await?;
            let mut request = client.get(url.clone());
            if let Some(etag) = etag {
                request = request.header("if-none-match", etag);
            }
            let response = request
                .send()
                .await
                .map_err(|error| SourceError::Network(error_chain(&error)))?;
            if !matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308) {
                break response;
            }
            let Some(next) = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|location| url.join(location).ok())
                .filter(|next| matches!(next.scheme(), "http" | "https"))
            else {
                break response;
            };
            if redirects >= MAX_REDIRECTS {
                return Err(SourceError::Network("too many redirects".to_owned()));
            }
            redirects += 1;
            url = next;
        };
        let status = response.status();
        if status == reqwest::StatusCode::NOT_MODIFIED && etag.is_some() {
            return Ok(Fetched::NotModified);
        }
        if !status.is_success() {
            return Err(SourceError::Status(status.as_u16()));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_ARCHIVE_BYTES)
        {
            return Err(SourceError::TooLarge);
        }
        let etag = response
            .headers()
            .get("etag")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty() && value.chars().count() <= MAX_ETAG_CHARS)
            .map(str::to_owned);
        let file = TempFile::from_stream(
            &self.dir,
            Box::pin(response.bytes_stream()),
            MAX_ARCHIVE_BYTES,
        )
        .await?;
        Ok(Fetched::Data { file, etag })
    }
}

async fn check_target(guard: &Guard, outbound: &Snapshot, url: &Url) -> Result<(), SourceError> {
    let blocked = || {
        SourceError::Refused(format!(
            "the SSRF protection blocks the address of {}",
            url.host_str().unwrap_or_default()
        ))
    };
    if !literal_allowed(guard, url) {
        return Err(blocked());
    }
    if outbound.proxied(url)
        && let Some(host) = url.host_str()
        && let Err(FetchError::Blocked(_)) =
            resolve_guarded(guard, host, url.port_or_known_default().unwrap_or(443)).await
    {
        return Err(blocked());
    }
    Ok(())
}

fn error_chain(error: &reqwest::Error) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(next) = source {
        text.push_str(": ");
        text.push_str(&next.to_string());
        source = next.source();
    }
    text
}

#[async_trait]
impl AdvisorySource for HttpDump {
    async fn fetch(&self, ecosystem: &str, etag: Option<&str>) -> Result<Fetched, SourceError> {
        tokio::time::timeout(DEADLINE, self.download(ecosystem, etag))
            .await
            .map_err(|_| SourceError::Network("the download took too long".to_owned()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astrum_outbound::{HttpSettings, ProxySettings};

    fn snapshot(proxy: bool) -> Snapshot {
        Snapshot {
            http: HttpSettings {
                http_proxy: ProxySettings {
                    enabled: proxy,
                    host: Some("proxy.example".to_owned()),
                    port: Some("3128".to_owned()),
                    ..ProxySettings::default()
                },
                ..HttpSettings::default()
            },
            ..Snapshot::default()
        }
    }

    #[tokio::test]
    async fn a_proxied_target_that_resolves_to_a_private_address_is_refused() {
        let guard = Guard::new(&SsrfProtection::default());
        let url = Url::parse("http://localhost:9/all.zip").unwrap();
        let refused = check_target(&guard, &snapshot(true), &url).await;
        assert!(
            matches!(refused, Err(SourceError::Refused(_))),
            "{refused:?}"
        );
        assert!(check_target(&guard, &snapshot(false), &url).await.is_ok());
        let literal = Url::parse("http://10.1.2.3/all.zip").unwrap();
        assert!(
            check_target(&guard, &snapshot(true), &literal)
                .await
                .is_err()
        );
    }
}
