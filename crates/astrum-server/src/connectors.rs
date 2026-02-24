use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use astrum_api::AppState;
use astrum_format_docker::Connector;
use axum::Router;
use axum::extract::ConnectInfo;
use axum::http::{HeaderValue, Request};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower::ServiceExt;

use crate::config::ConnectorConfig;

const POLL: Duration = Duration::from_secs(10);
const HANDSHAKE: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    fn as_str(self) -> &'static str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
        }
    }
}

pub fn tls(config: &ConnectorConfig) -> anyhow::Result<Option<Arc<rustls::ServerConfig>>> {
    let (Some(certificate), Some(key)) = (&config.tls_certificate, &config.tls_key) else {
        return Ok(None);
    };
    let chain = read_certificates(certificate)?;
    let key = PrivateKeyDer::from_pem_slice(&read(key)?)
        .with_context(|| format!("{} holds no private key", key.display()))?;
    let mut server = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(chain, key)
    .context("the connector certificate does not match its key")?;
    server.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Some(Arc::new(server)))
}

fn read(path: &Path) -> anyhow::Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))
}

fn read_certificates(path: &Path) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let chain = CertificateDer::pem_slice_iter(&read(path)?)
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("{} is not a PEM certificate chain", path.display()))?;
    anyhow::ensure!(!chain.is_empty(), "{} holds no certificate", path.display());
    Ok(chain)
}

struct Open {
    scheme: Scheme,
    task: JoinHandle<()>,
}

pub struct Connectors {
    state: AppState,
    app: Router,
    host: IpAddr,
    tls: Option<Arc<rustls::ServerConfig>>,
    open: Mutex<BTreeMap<u16, Open>>,
    reported: Mutex<BTreeSet<(u16, Scheme)>>,
}

impl Connectors {
    pub fn new(
        state: AppState,
        app: Router,
        host: IpAddr,
        tls: Option<Arc<rustls::ServerConfig>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            state,
            app,
            host,
            tls,
            open: Mutex::new(BTreeMap::new()),
            reported: Mutex::new(BTreeSet::new()),
        })
    }

    pub fn spawn(self: &Arc<Self>) -> JoinHandle<()> {
        let connectors = self.clone();
        tokio::spawn(async move {
            loop {
                connectors.reconcile().await;
                tokio::select! {
                    () = connectors.state.repository_changes().notified() => {}
                    () = tokio::time::sleep(POLL) => {}
                }
            }
        })
    }

    pub async fn ports(&self) -> BTreeMap<u16, Scheme> {
        self.open
            .lock()
            .await
            .iter()
            .map(|(port, open)| (*port, open.scheme))
            .collect()
    }

    async fn wanted(&self) -> anyhow::Result<BTreeMap<u16, Scheme>> {
        let application = self.state.application_port();
        let mut wanted = BTreeMap::new();
        for repository in astrum_repo::store::list(self.state.db().conn()).await? {
            let Some(docker) = &repository.attributes.docker else {
                continue;
            };
            for (field, port) in docker.ports() {
                let Ok(port) = u16::try_from(port) else {
                    continue;
                };
                if Some(port) == application {
                    continue;
                }
                let scheme = match field {
                    "httpsPort" => Scheme::Https,
                    _ => Scheme::Http,
                };
                wanted.insert(port, scheme);
            }
        }
        Ok(wanted)
    }

    pub async fn reconcile(&self) {
        let wanted = match self.wanted().await {
            Ok(wanted) => wanted,
            Err(error) => {
                tracing::warn!(
                    error = format!("{error:#}"),
                    "the Docker connectors were not checked"
                );
                return;
            }
        };
        let mut open = self.open.lock().await;
        let closing: Vec<u16> = open
            .iter()
            .filter(|(port, connector)| {
                wanted.get(port) != Some(&connector.scheme) || connector.task.is_finished()
            })
            .map(|(port, _)| *port)
            .collect();
        for port in closing {
            if let Some(connector) = open.remove(&port) {
                connector.task.abort();
                let _ = connector.task.await;
                tracing::info!(
                    port,
                    scheme = connector.scheme.as_str(),
                    "closed a Docker connector"
                );
            }
        }
        let mut reported = self.reported.lock().await;
        reported.retain(|(port, scheme)| wanted.get(port) == Some(scheme));
        for (port, scheme) in wanted {
            if open.contains_key(&port) {
                continue;
            }
            let tls = match (scheme, &self.tls) {
                (Scheme::Https, None) => {
                    if reported.insert((port, scheme)) {
                        tracing::warn!(
                            port,
                            "an HTTPS Docker connector needs a certificate: set {} and {}",
                            crate::config::ENV_CONNECTOR_TLS_CERTIFICATE,
                            crate::config::ENV_CONNECTOR_TLS_KEY
                        );
                    }
                    continue;
                }
                (Scheme::Https, Some(tls)) => Some(tls.clone()),
                (Scheme::Http, _) => None,
            };
            let listener = match TcpListener::bind((self.host, port)).await {
                Ok(listener) => listener,
                Err(error) => {
                    if reported.insert((port, scheme)) {
                        tracing::warn!(port, %error, "a Docker connector cannot listen");
                    }
                    continue;
                }
            };
            reported.remove(&(port, scheme));
            let app = self.app.clone().layer(axum::Extension(Connector(port)));
            let task = match tls {
                None => tokio::spawn(serve_http(listener, app)),
                Some(tls) => tokio::spawn(serve_https(listener, app, tls)),
            };
            tracing::info!(port, scheme = scheme.as_str(), "opened a Docker connector");
            open.insert(port, Open { scheme, task });
        }
    }

    pub async fn close(&self) {
        let mut open = self.open.lock().await;
        for (_, connector) in std::mem::take(&mut *open) {
            connector.task.abort();
            let _ = connector.task.await;
        }
    }
}

async fn serve_http(listener: TcpListener, app: Router) {
    let port = listener.local_addr().map(|address| address.port()).ok();
    if let Err(error) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        tracing::warn!(?port, %error, "a Docker connector stopped");
    }
}

async fn serve_https(listener: TcpListener, app: Router, tls: Arc<rustls::ServerConfig>) {
    let acceptor = tokio_rustls::TlsAcceptor::from(tls);
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(error) => {
                tracing::debug!(%error, "a Docker connector did not accept a connection");
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let app = app.clone();
        tokio::spawn(async move {
            let stream = match tokio::time::timeout(HANDSHAKE, acceptor.accept(stream)).await {
                Ok(Ok(stream)) => stream,
                Ok(Err(error)) => {
                    tracing::debug!(%peer, %error, "a TLS handshake failed");
                    return;
                }
                Err(_) => return,
            };
            let service = app.map_request(move |mut request: Request<_>| {
                request.extensions_mut().insert(ConnectInfo(peer));
                request
                    .headers_mut()
                    .insert("x-forwarded-proto", HeaderValue::from_static("https"));
                request
            });
            if let Err(error) = auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(
                    TokioIo::new(stream),
                    TowerToHyperService::new(service),
                )
                .await
            {
                tracing::debug!(%peer, %error, "a connection ended with an error");
            }
        });
    }
}
