use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
pub use astrum_blob::remote::MIN_TOKEN_LENGTH;
use astrum_blob::remote::{
    AgentError, AgentStatus, BLOB_SIZE_HEADER, BLOBS_PATH, DeletedReply, ListedBlob, STATUS_PATH,
    StoredReply, millis_of,
};
use astrum_blob::{BlobId, BlobStore, Content, FileBlobStore, Metrics};
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Path as UrlPath, Query, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::{StreamExt, TryStreamExt};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::service::TowerToHyperService;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use subtle::ConstantTimeEq;
use tokio::net::TcpListener;
use tokio::task::JoinSet;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const STORE_NAME: &str = "agent";
const LIST_LIMIT: usize = 1000;
const HANDSHAKE: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct Agent {
    store: Arc<FileBlobStore>,
    token: Arc<str>,
}

impl Agent {
    pub async fn open(dir: impl Into<PathBuf>, token: impl Into<String>) -> anyhow::Result<Self> {
        let token = token.into();
        anyhow::ensure!(
            token.chars().count() >= MIN_TOKEN_LENGTH,
            "the token must be at least {MIN_TOKEN_LENGTH} characters long"
        );
        let store = FileBlobStore::open(STORE_NAME, dir, Metrics::detached(STORE_NAME))
            .await
            .context("the blob directory cannot be used")?;
        Ok(Self {
            store: Arc::new(store),
            token: token.into(),
        })
    }

    pub fn root(&self) -> &Path {
        self.store.root()
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(STATUS_PATH, get(status))
            .route(BLOBS_PATH, get(list))
            .route(
                &format!("{BLOBS_PATH}/{{id}}"),
                get(read).head(size).put(write).delete(remove),
            )
            .layer(DefaultBodyLimit::disable())
            .layer(middleware::from_fn_with_state(self.clone(), authorize))
            .with_state(self)
    }
}

pub fn tls(certificate: &Path, key: &Path) -> anyhow::Result<Arc<rustls::ServerConfig>> {
    let read = |path: &Path| {
        std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))
    };
    let chain = CertificateDer::pem_slice_iter(&read(certificate)?)
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("{} is not a PEM certificate chain", certificate.display()))?;
    anyhow::ensure!(
        !chain.is_empty(),
        "{} holds no certificate",
        certificate.display()
    );
    let key = PrivateKeyDer::from_pem_slice(&read(key)?)
        .with_context(|| format!("{} holds no private key", key.display()))?;
    let mut config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(chain, key)
    .context("the certificate does not match its key")?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

pub async fn serve(
    listener: TcpListener,
    app: Router,
    tls: Option<Arc<rustls::ServerConfig>>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let Some(tls) = tls else {
        return axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await;
    };
    let acceptor = tokio_rustls::TlsAcceptor::from(tls);
    let mut connections = JoinSet::new();
    tokio::pin!(shutdown);
    loop {
        let (stream, peer) = tokio::select! {
            () = &mut shutdown => break,
            accepted = listener.accept() => match accepted {
                Ok(accepted) => accepted,
                Err(error) => {
                    tracing::debug!(%error, "a connection was not accepted");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            },
        };
        let acceptor = acceptor.clone();
        let app = app.clone();
        connections.spawn(async move {
            let stream = match tokio::time::timeout(HANDSHAKE, acceptor.accept(stream)).await {
                Ok(Ok(stream)) => stream,
                Ok(Err(error)) => {
                    tracing::debug!(%peer, %error, "a TLS handshake failed");
                    return;
                }
                Err(_) => return,
            };
            if let Err(error) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(stream), TowerToHyperService::new(app))
                .await
            {
                tracing::debug!(%peer, %error, "a connection ended with an error");
            }
        });
        while connections.try_join_next().is_some() {}
    }
    while connections.join_next().await.is_some() {}
    Ok(())
}

async fn authorize(State(agent): State<Agent>, request: Request, next: Next) -> Response {
    let presented = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    if bool::from(presented.as_bytes().ct_eq(agent.token.as_bytes())) {
        return next.run(request).await;
    }
    refuse(
        StatusCode::UNAUTHORIZED,
        "a valid bearer token is required".to_owned(),
    )
}

#[derive(Default, Deserialize)]
struct StatusQuery {
    #[serde(default)]
    check: bool,
}

async fn status(State(agent): State<Agent>, Query(query): Query<StatusQuery>) -> Response {
    if query.check
        && let Err(error) = agent.store.check().await
    {
        return failed("check", error);
    }
    match agent.store.available_space().await {
        Ok(available_space) => Json(AgentStatus {
            version: VERSION.to_owned(),
            available_space,
        })
        .into_response(),
        Err(error) => failed("measure the free space", error),
    }
}

#[derive(Default, Deserialize)]
struct ListQuery {
    after: Option<String>,
    limit: Option<usize>,
}

async fn list(State(agent): State<Agent>, Query(query): Query<ListQuery>) -> Response {
    let after = match query.after.as_deref().map(str::parse::<BlobId>).transpose() {
        Ok(after) => after,
        Err(_) => return refuse(StatusCode::BAD_REQUEST, "after is not a blob id".to_owned()),
    };
    let limit = query.limit.unwrap_or(LIST_LIMIT).clamp(1, LIST_LIMIT);
    match agent.store.list(after, limit).await {
        Ok(blobs) => Json(
            blobs
                .into_iter()
                .map(|blob| ListedBlob {
                    id: blob.id,
                    size: blob.size,
                    modified_millis: millis_of(blob.modified),
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(error) => failed("list", error),
    }
}

#[derive(Default, Deserialize)]
struct ReadQuery {
    offset: Option<u64>,
    length: Option<u64>,
}

async fn read(
    State(agent): State<Agent>,
    UrlPath(id): UrlPath<String>,
    Query(query): Query<ReadQuery>,
) -> Response {
    let Some(id) = blob_id(&id) else {
        return not_a_blob_id();
    };
    let found = match (query.offset, query.length) {
        (None, None) => agent.store.read(id).await,
        (offset, length) => {
            agent
                .store
                .read_range(id, offset.unwrap_or(0), length.unwrap_or(u64::MAX))
                .await
        }
    };
    match found {
        Ok(Some(content)) => streamed(content),
        Ok(None) => missing(),
        Err(astrum_blob::Error::Range { size, .. }) => {
            let mut response = refuse(
                StatusCode::RANGE_NOT_SATISFIABLE,
                "the range starts past the end of the blob".to_owned(),
            );
            response
                .headers_mut()
                .insert(BLOB_SIZE_HEADER, HeaderValue::from(size));
            response
        }
        Err(error) => failed("read", error),
    }
}

async fn size(State(agent): State<Agent>, UrlPath(id): UrlPath<String>) -> Response {
    let Some(id) = blob_id(&id) else {
        return not_a_blob_id();
    };
    match agent.store.size(id).await {
        Ok(Some(size)) => (
            [
                (BLOB_SIZE_HEADER, HeaderValue::from(size)),
                (header::CONTENT_LENGTH.as_str(), HeaderValue::from(size)),
            ],
            StatusCode::OK,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => failed("measure", error),
    }
}

async fn write(
    State(agent): State<Agent>,
    UrlPath(id): UrlPath<String>,
    request: Request,
) -> Response {
    let Some(id) = blob_id(&id) else {
        return not_a_blob_id();
    };
    let content = request
        .into_body()
        .into_data_stream()
        .map_err(io::Error::other)
        .boxed();
    match agent.store.put_as(id, content).await {
        Ok(written) => (
            StatusCode::CREATED,
            Json(StoredReply {
                size: written.size,
                sha256: written.digests.sha256,
            }),
        )
            .into_response(),
        Err(astrum_blob::Error::Content(error)) => {
            tracing::info!(blob = %id, %error, "an upload ended early");
            refuse(
                StatusCode::BAD_REQUEST,
                format!("the upload ended early: {error}"),
            )
        }
        Err(error) => failed("write", error),
    }
}

async fn remove(State(agent): State<Agent>, UrlPath(id): UrlPath<String>) -> Response {
    let Some(id) = blob_id(&id) else {
        return not_a_blob_id();
    };
    let size = match agent.store.size(id).await {
        Ok(Some(size)) => size,
        Ok(None) => return missing(),
        Err(error) => return failed("measure", error),
    };
    match agent.store.delete(id).await {
        Ok(true) => Json(DeletedReply { size }).into_response(),
        Ok(false) => missing(),
        Err(error) => failed("delete", error),
    }
}

fn streamed(content: Content) -> Response {
    let mut response = Response::new(Body::from_stream(content.stream));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(content.length));
    headers.insert(BLOB_SIZE_HEADER, HeaderValue::from(content.size));
    response
}

fn blob_id(text: &str) -> Option<BlobId> {
    text.parse().ok()
}

fn not_a_blob_id() -> Response {
    refuse(StatusCode::BAD_REQUEST, "not a blob id".to_owned())
}

fn missing() -> Response {
    refuse(StatusCode::NOT_FOUND, "there is no such blob".to_owned())
}

fn failed(operation: &str, error: astrum_blob::Error) -> Response {
    tracing::warn!(operation, %error, "a request failed");
    refuse(StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn refuse(status: StatusCode, message: String) -> Response {
    (status, Json(AgentError { message })).into_response()
}
