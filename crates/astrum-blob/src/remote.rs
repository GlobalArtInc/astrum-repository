use std::error::Error as _;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::digest::Hasher;
use crate::{
    BlobId, BlobStore, Content, ContentStream, Error, Metrics, StoreKind, StoredBlob, Totals,
    Written, clamp_range,
};

pub const BLOB_SIZE_HEADER: &str = "x-astrum-blob-size";
pub const BLOBS_PATH: &str = "/v1/blobs";
pub const STATUS_PATH: &str = "/v1/status";
pub const MIN_TOKEN_LENGTH: usize = 32;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(120);
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSettings {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredReply {
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedReply {
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedBlob {
    pub id: BlobId,
    pub size: u64,
    pub modified_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub version: String,
    pub available_space: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentError {
    pub message: String,
}

pub fn millis_of(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|age| age.as_millis() as u64)
        .unwrap_or_default()
}

#[derive(Debug)]
pub struct RemoteBlobStore {
    name: String,
    base: String,
    token: String,
    client: reqwest::Client,
    metrics: Metrics,
}

impl RemoteBlobStore {
    pub fn open(
        name: impl Into<String>,
        settings: &RemoteSettings,
        token: String,
        metrics: Metrics,
    ) -> Result<Self, Error> {
        let name = name.into();
        let broken = |message: String| Error::Config {
            store: name.clone(),
            message,
        };
        let base = settings.url.trim().trim_end_matches('/').to_owned();
        if !(base.starts_with("http://") || base.starts_with("https://")) {
            return Err(broken(
                "the agent address must start with http:// or https://".to_owned(),
            ));
        }
        if token.is_empty() {
            return Err(broken("the agent token is missing".to_owned()));
        }
        let mut builder = reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            .pool_idle_timeout(POOL_IDLE_TIMEOUT);
        if let Some(pem) = settings
            .certificate
            .as_deref()
            .filter(|pem| !pem.trim().is_empty())
        {
            let certificates = reqwest::Certificate::from_pem_bundle(pem.as_bytes())
                .map_err(|error| broken(format!("the certificate cannot be read: {error}")))?;
            if certificates.is_empty() {
                return Err(broken(
                    "the certificate holds no PEM certificate".to_owned(),
                ));
            }
            for certificate in certificates {
                builder = builder.add_root_certificate(certificate);
            }
        }
        let client = builder.build().map_err(|error| broken(describe(&error)))?;
        Ok(Self {
            name,
            base,
            token,
            client,
            metrics,
        })
    }

    pub fn url(&self) -> &str {
        &self.base
    }

    pub async fn status(&self, check: bool) -> Result<AgentStatus, Error> {
        let url = match check {
            true => format!("{}{STATUS_PATH}?check=true", self.base),
            false => format!("{}{STATUS_PATH}", self.base),
        };
        let response = self
            .send("report its status", self.request(Method::GET, url))
            .await?;
        if response.status() != StatusCode::OK {
            return Err(self.refused("report its status", response).await);
        }
        self.json("report its status", response).await
    }

    fn blob_url(&self, id: BlobId) -> String {
        format!("{}{BLOBS_PATH}/{id}", self.base)
    }

    fn request(&self, method: Method, url: String) -> RequestBuilder {
        self.client.request(method, url).bearer_auth(&self.token)
    }

    async fn send(
        &self,
        operation: &'static str,
        request: RequestBuilder,
    ) -> Result<Response, Error> {
        request
            .send()
            .await
            .map_err(|error| self.failed(operation, describe(&error)))
    }

    fn failed(&self, operation: &'static str, message: impl Into<String>) -> Error {
        Error::Remote {
            store: self.name.clone(),
            operation,
            message: message.into(),
        }
    }

    async fn refused(&self, operation: &'static str, response: Response) -> Error {
        let status = response.status();
        let body = response.bytes().await.unwrap_or_default();
        let message = serde_json::from_slice::<AgentError>(&body)
            .map(|error| error.message)
            .unwrap_or_else(|_| String::from_utf8_lossy(&body).trim().to_owned());
        let message = match message.is_empty() {
            true => format!("the agent answered {status}"),
            false => format!("the agent answered {status}: {message}"),
        };
        self.failed(operation, message)
    }

    async fn json<T: DeserializeOwned>(
        &self,
        operation: &'static str,
        response: Response,
    ) -> Result<T, Error> {
        let body = response
            .bytes()
            .await
            .map_err(|error| self.failed(operation, describe(&error)))?;
        serde_json::from_slice(&body).map_err(|error| {
            self.failed(
                operation,
                format!("the agent answered something unexpected: {error}"),
            )
        })
    }

    fn blob_size(&self, operation: &'static str, response: &Response) -> Result<u64, Error> {
        response
            .headers()
            .get(BLOB_SIZE_HEADER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| self.failed(operation, format!("the agent sent no {BLOB_SIZE_HEADER}")))
    }

    async fn fetch(&self, id: BlobId, range: Option<(u64, u64)>) -> Result<Option<Content>, Error> {
        let url = match range {
            Some((offset, length)) => {
                format!("{}?offset={offset}&length={length}", self.blob_url(id))
            }
            None => self.blob_url(id),
        };
        let response = self
            .send("read a blob", self.request(Method::GET, url))
            .await?;
        match response.status() {
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::RANGE_NOT_SATISFIABLE => {
                let size = self.blob_size("read a blob", &response)?;
                Err(Error::Range {
                    offset: range.map(|(offset, _)| offset).unwrap_or_default(),
                    size,
                })
            }
            StatusCode::OK => {
                let size = self.blob_size("read a blob", &response)?;
                let (offset, length) = match range {
                    Some((offset, length)) => (offset, clamp_range(size, offset, length)?),
                    None => (0, size),
                };
                Ok(Some(Content {
                    size,
                    offset,
                    length,
                    stream: response.bytes_stream().map_err(io::Error::other).boxed(),
                }))
            }
            _ => Err(self.refused("read a blob", response).await),
        }
    }

    async fn remove(&self, id: BlobId) -> Result<Option<u64>, Error> {
        let response = self
            .send(
                "delete a blob",
                self.request(Method::DELETE, self.blob_url(id)),
            )
            .await?;
        match response.status() {
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::OK => Ok(Some(
                self.json::<DeletedReply>("delete a blob", response)
                    .await?
                    .size,
            )),
            _ => Err(self.refused("delete a blob", response).await),
        }
    }
}

#[async_trait]
impl BlobStore for RemoteBlobStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StoreKind {
        StoreKind::Remote
    }

    async fn put(&self, content: ContentStream) -> Result<Written, Error> {
        let id = BlobId::new();
        let hasher = Arc::new(Mutex::new(Hasher::default()));
        let failure: Arc<Mutex<Option<io::Error>>> = Arc::new(Mutex::new(None));
        let (tap, broken) = (hasher.clone(), failure.clone());
        let body = content.map(move |chunk| match chunk {
            Ok(bytes) => {
                tap.lock()
                    .expect("the upload hasher lock is never poisoned")
                    .update(&bytes);
                Ok(bytes)
            }
            Err(error) => {
                let message = error.to_string();
                *broken
                    .lock()
                    .expect("the upload failure lock is never poisoned") = Some(error);
                Err(io::Error::other(message))
            }
        });
        let sent = self
            .request(Method::PUT, self.blob_url(id))
            .body(reqwest::Body::wrap_stream(body))
            .send()
            .await;
        if let Some(error) = failure
            .lock()
            .expect("the upload failure lock is never poisoned")
            .take()
        {
            return Err(Error::Content(error));
        }
        let response = sent.map_err(|error| self.failed("write a blob", describe(&error)))?;
        if response.status() != StatusCode::CREATED {
            return Err(self.refused("write a blob", response).await);
        }
        let stored: StoredReply = self.json("write a blob", response).await?;
        let (size, digests) = std::mem::take(
            &mut *hasher
                .lock()
                .expect("the upload hasher lock is never poisoned"),
        )
        .finish();
        if stored.size != size || stored.sha256 != digests.sha256 {
            if let Err(error) = self.remove(id).await {
                tracing::warn!(store = %self.name, blob = %id, %error, "a damaged remote blob was kept");
            }
            return Err(self.failed(
                "write a blob",
                format!(
                    "the agent stored {} bytes with SHA-256 {} for {size} bytes with SHA-256 {}",
                    stored.size, stored.sha256, digests.sha256
                ),
            ));
        }
        self.metrics.added(size).await;
        Ok(Written {
            id,
            store: self.name.clone(),
            size,
            digests,
        })
    }

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error> {
        self.fetch(id, None).await
    }

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error> {
        self.fetch(id, Some((offset, length))).await
    }

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error> {
        let response = self
            .send(
                "measure a blob",
                self.request(Method::HEAD, self.blob_url(id)),
            )
            .await?;
        match response.status() {
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::OK => Ok(Some(self.blob_size("measure a blob", &response)?)),
            _ => Err(self.refused("measure a blob", response).await),
        }
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error> {
        match self.remove(id).await? {
            Some(size) => {
                self.metrics.removed(size).await;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn list(&self, after: Option<BlobId>, limit: usize) -> Result<Vec<StoredBlob>, Error> {
        let url = match after {
            Some(after) => format!("{}{BLOBS_PATH}?limit={limit}&after={after}", self.base),
            None => format!("{}{BLOBS_PATH}?limit={limit}", self.base),
        };
        let response = self
            .send("list blobs", self.request(Method::GET, url))
            .await?;
        if response.status() != StatusCode::OK {
            return Err(self.refused("list blobs", response).await);
        }
        let listed: Vec<ListedBlob> = self.json("list blobs", response).await?;
        Ok(listed
            .into_iter()
            .map(|blob| StoredBlob {
                id: blob.id,
                size: blob.size,
                modified: UNIX_EPOCH + Duration::from_millis(blob.modified_millis),
            })
            .collect())
    }

    async fn totals(&self) -> Result<Totals, Error> {
        self.metrics.totals().await
    }

    async fn available_space(&self) -> Result<Option<u64>, Error> {
        Ok(self.status(false).await?.available_space)
    }

    async fn check(&self) -> Result<(), Error> {
        self.status(true).await.map(|_| ())
    }
}

fn describe(error: &reqwest::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let text = cause.to_string();
        if !message.contains(&text) {
            message.push_str(": ");
            message.push_str(&text);
        }
        source = cause.source();
    }
    message
}
