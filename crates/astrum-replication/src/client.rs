use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use astrum_outbound::{ClientOptions, Snapshot};
use astrum_security::SecretBox;
use astrum_security::ssrf::{Guard, GuardedResolver, remote_literal_allowed};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HeaderValue, RETRY_AFTER};
use reqwest::redirect::Policy;
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::time::Instant;
use url::Url;
use uuid::Uuid;

use crate::error::{self, ERROR_BODY_LIMIT, Error, parse_retry_after};
use crate::limit::Limiter;
use crate::protocol::{
    BLOB, CAPABILITIES, COMPONENTS, Capabilities, INVENTORY, INVENTORY_COMPARE, InventoryCompare,
    InventoryKey, InventoryPage, MAX_MISSING, MISSING, MissingReply, MissingRequest, PROTOCOL,
    Tombstone, valid_sha256,
};
use astrum_content::transfer::{Bundle, Outcome};

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const CONTROL_TIMEOUT: Duration = Duration::from_secs(60);
pub const UPLOAD_IDLE: Duration = Duration::from_secs(120);
pub const MAX_REPLY_BYTES: usize = 16 * 1024 * 1024;
const READ_BACKSTOP: Duration = Duration::from_secs(24 * 3600);
const KEEPALIVE: Duration = Duration::from_secs(60);
const MAX_VERSION: usize = 100;
const MAX_FORMATS: usize = 200;
const MAX_FORMAT: usize = 64;
const MAX_REPOSITORY: usize = 255;
const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

#[derive(Clone, Debug)]
pub struct Remote {
    pub url: String,
    pub use_trust_store: bool,
    pub allow_private_network: bool,
    pub max_concurrency: usize,
    pub max_bytes_per_second: Option<u64>,
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base: Url,
    authorization: HeaderValue,
    limiter: Option<Limiter>,
    max_concurrency: usize,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("url", &self.base.as_str())
            .field("max_concurrency", &self.max_concurrency)
            .finish_non_exhaustive()
    }
}

struct Watch {
    started: Instant,
    touched: AtomicU64,
    throttled: AtomicBool,
}

impl Watch {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            touched: AtomicU64::new(0),
            throttled: AtomicBool::new(false),
        }
    }

    fn touch(&self) {
        let elapsed = self.started.elapsed().as_millis() as u64;
        self.touched.store(elapsed, Ordering::Relaxed);
    }

    fn throttling(&self, on: bool) {
        self.throttled.store(on, Ordering::Relaxed);
    }

    fn quiet(&self) -> Duration {
        if self.throttled.load(Ordering::Relaxed) {
            return Duration::ZERO;
        }
        let touched = Duration::from_millis(self.touched.load(Ordering::Relaxed));
        self.started.elapsed().saturating_sub(touched)
    }

    async fn stalled(&self, idle: Duration) {
        loop {
            let quiet = self.quiet();
            if quiet >= idle {
                return;
            }
            tokio::time::sleep(idle - quiet).await;
        }
    }
}

fn refused(text: &str) -> Error {
    Error::Refused(text.to_owned())
}

fn short(text: &str, limit: usize) -> String {
    error::clean(text).chars().take(limit).collect()
}

fn segment(value: &str) -> String {
    utf8_percent_encode(value, SEGMENT).to_string()
}

fn repository_path(template: &str, repository: &str) -> Result<String, Error> {
    if repository.is_empty()
        || repository.len() > MAX_REPOSITORY
        || repository == "."
        || repository == ".."
        || repository.chars().any(char::is_control)
    {
        return Err(refused("the name of the remote repository cannot be used"));
    }
    Ok(template.replace("{repo}", &segment(repository)))
}

fn json_request<T: Serialize>(request: RequestBuilder, body: &T) -> Result<RequestBuilder, Error> {
    let bytes =
        serde_json::to_vec(body).map_err(|_| refused("the request could not be encoded"))?;
    Ok(request.header(CONTENT_TYPE, "application/json").body(bytes))
}

async fn read_limited(
    mut response: Response,
    limit: usize,
) -> Result<(Vec<u8>, bool), reqwest::Error> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        let room = limit - body.len();
        if chunk.len() > room {
            body.extend_from_slice(&chunk[..room]);
            return Ok((body, true));
        }
        body.extend_from_slice(&chunk);
    }
    Ok((body, false))
}

impl Client {
    pub fn new(
        remote: &Remote,
        token: &str,
        outbound: &Snapshot,
        secrets: &SecretBox,
        guard: &Guard,
    ) -> Result<Self, Error> {
        let base = Url::parse(&remote.url)
            .map_err(|_| refused("the address of the remote is not a valid URL"))?;
        match base.scheme() {
            "https" => {}
            "http" if remote.allow_private_network => {}
            "http" => {
                return Err(refused(
                    "an http:// address needs private networks to be allowed for the remote",
                ));
            }
            _ => {
                return Err(refused(
                    "the address of the remote must be an https:// address",
                ));
            }
        }
        if base.host().is_none()
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
            || base.path() != "/"
        {
            return Err(refused(
                "the address of the remote must hold only a scheme, a host and a port",
            ));
        }
        if !remote_literal_allowed(guard, &base, remote.allow_private_network) {
            return Err(refused(
                "the address of the remote is blocked by the SSRF protection",
            ));
        }
        if token.is_empty() {
            return Err(refused("the token of the remote is empty"));
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| refused("the token of the remote cannot be sent in a header"))?;
        authorization.set_sensitive(true);
        let guard = Arc::new(guard.clone());
        let resolver = if remote.allow_private_network {
            GuardedResolver::with_private_ranges(guard)
        } else {
            GuardedResolver::new(guard)
        };
        let options = ClientOptions {
            user_agent_suffix: Some("replication".to_owned()),
            timeout_seconds: None,
            use_trust_store: remote.use_trust_store,
        };
        let http = outbound
            .configure(reqwest::Client::builder(), secrets, &options)
            .redirect(Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_BACKSTOP)
            .tcp_keepalive(KEEPALIVE)
            .dns_resolver(Arc::new(resolver))
            .build()
            .map_err(|error| Error::transport(&error))?;
        Ok(Self {
            http,
            base,
            authorization,
            limiter: remote.max_bytes_per_second.and_then(Limiter::new),
            max_concurrency: remote.max_concurrency,
        })
    }

    pub fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }

    fn secret(&self) -> &str {
        self.authorization
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
            .unwrap_or_default()
    }

    fn url(&self, path: &str) -> Url {
        let mut url = self.base.clone();
        url.set_path(path);
        url
    }

    async fn send(&self, request: RequestBuilder) -> Result<Response, Error> {
        let response = request
            .header(AUTHORIZATION, self.authorization.clone())
            .send()
            .await
            .map_err(|error| Error::transport(&error).redacted(self.secret()))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let retry_after = parse_retry_after(
            response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
        );
        let body = read_limited(response, ERROR_BODY_LIMIT)
            .await
            .map(|(body, _)| body)
            .unwrap_or_default();
        Err(Error::from_status(status, retry_after, &body).redacted(self.secret()))
    }

    async fn reply<T: DeserializeOwned>(&self, response: Response) -> Result<T, Error> {
        if response
            .content_length()
            .is_some_and(|length| length > MAX_REPLY_BYTES as u64)
        {
            return Err(refused("the reply of the remote is too large"));
        }
        let (body, truncated) = read_limited(response, MAX_REPLY_BYTES)
            .await
            .map_err(|error| Error::transport(&error).redacted(self.secret()))?;
        if truncated {
            return Err(refused("the reply of the remote is too large"));
        }
        serde_json::from_slice(&body).map_err(|_| {
            refused("the remote sent a reply that does not follow the replication protocol")
        })
    }

    async fn control<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, Error> {
        let response = self.send(request.timeout(CONTROL_TIMEOUT)).await?;
        self.reply(response).await
    }

    pub async fn capabilities(&self) -> Result<Capabilities, Error> {
        self.control(self.http.get(self.url(CAPABILITIES))).await
    }

    pub async fn check(&self) -> Result<Capabilities, Error> {
        let mut capabilities = self.capabilities().await?;
        if capabilities.protocol != PROTOCOL {
            return Err(Error::Refused(format!(
                "the remote speaks protocol {}",
                capabilities.protocol
            )));
        }
        let id = Uuid::parse_str(&capabilities.instance_id)
            .map_err(|_| refused("the remote sent an instance id that is not a UUID"))?;
        capabilities.instance_id = id.to_string();
        capabilities.version = short(&capabilities.version, MAX_VERSION);
        capabilities.formats = capabilities
            .formats
            .iter()
            .take(MAX_FORMATS)
            .map(|format| short(format, MAX_FORMAT))
            .filter(|format| !format.is_empty())
            .collect();
        Ok(capabilities)
    }

    pub async fn missing(&self, repository: &str, sha256: &[String]) -> Result<Vec<String>, Error> {
        if sha256.len() > MAX_MISSING {
            return Err(Error::Refused(format!(
                "at most {MAX_MISSING} digests can be asked about at once"
            )));
        }
        if !sha256.iter().all(|digest| valid_sha256(digest)) {
            return Err(refused("a digest is not a lowercase SHA-256 in hex"));
        }
        let url = self.url(&repository_path(MISSING, repository)?);
        let request = json_request(
            self.http.post(url),
            &MissingRequest {
                sha256: sha256.to_vec(),
            },
        )?;
        let reply: MissingReply = self.control(request).await?;
        Ok(reply.missing)
    }

    pub async fn put_blob<S>(
        &self,
        repository: &str,
        sha256: &str,
        size: u64,
        stream: S,
    ) -> Result<(), Error>
    where
        S: Stream<Item = io::Result<Bytes>> + Send + 'static,
    {
        if !valid_sha256(sha256) {
            return Err(refused("the digest is not a lowercase SHA-256 in hex"));
        }
        let path = repository_path(BLOB, repository)?.replace("{sha256}", sha256);
        let watch = Arc::new(Watch::new());
        let limiter = self.limiter.clone();
        let progress = watch.clone();
        let body = stream.then(move |item| {
            let progress = progress.clone();
            let limiter = limiter.clone();
            async move {
                progress.touch();
                if let (Ok(bytes), Some(limiter)) = (&item, limiter) {
                    progress.throttling(true);
                    limiter.acquire(bytes.len()).await;
                    progress.throttling(false);
                    progress.touch();
                }
                item
            }
        });
        let request = self
            .http
            .put(self.url(&path))
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size)
            .body(reqwest::Body::wrap_stream(body));
        let response = tokio::select! {
            sent = self.send(request) => sent?,
            () = watch.stalled(UPLOAD_IDLE) => {
                return Err(Error::Retryable {
                    reason: format!(
                        "the upload made no progress for {} seconds",
                        UPLOAD_IDLE.as_secs()
                    ),
                    retry_after: None,
                });
            }
        };
        match response.status() {
            StatusCode::CREATED => Ok(()),
            other => Err(Error::Refused(format!(
                "the remote answered {} where 201 was expected",
                other.as_u16()
            ))),
        }
    }

    pub async fn commit(&self, repository: &str, bundle: &Bundle) -> Result<Outcome, Error> {
        let url = self.url(&repository_path(COMPONENTS, repository)?);
        let request = json_request(self.http.post(url), bundle)?;
        self.control(request).await
    }

    pub async fn tombstone(&self, repository: &str, tombstone: &Tombstone) -> Result<(), Error> {
        let url = self.url(&repository_path(COMPONENTS, repository)?);
        let request = json_request(self.http.delete(url), tombstone)?;
        self.send(request.timeout(CONTROL_TIMEOUT)).await?;
        Ok(())
    }

    pub async fn inventory_compare(
        &self,
        repository: &str,
        items: &[InventoryKey],
    ) -> Result<InventoryCompare, Error> {
        let url = self.url(&repository_path(INVENTORY_COMPARE, repository)?);
        let request = json_request(
            self.http.post(url),
            &InventoryCompare {
                items: items.to_vec(),
            },
        )?;
        self.control(request).await
    }

    pub async fn inventory(
        &self,
        repository: &str,
        after: Option<&str>,
        limit: u32,
    ) -> Result<InventoryPage, Error> {
        let mut url = self.url(&repository_path(INVENTORY, repository)?);
        {
            let mut query = url.query_pairs_mut();
            if let Some(after) = after {
                query.append_pair("after", after);
            }
            query.append_pair("limit", &limit.to_string());
        }
        self.control(self.http.get(url)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_one_path_segment() {
        let path = repository_path(COMPONENTS, "a b/c?d#e%f").unwrap();
        assert_eq!(
            path,
            "/service/rest/v1/replication/a%20b%2Fc%3Fd%23e%25f/components"
        );
        for refused in ["", ".", "..", "a\nb"] {
            assert!(repository_path(COMPONENTS, refused).is_err(), "{refused:?}");
        }
        assert_eq!(
            repository_path(MISSING, "docker-prod_1.x").unwrap(),
            "/service/rest/v1/replication/docker-prod_1.x/blobs/missing"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_watch_fires_only_after_a_quiet_stretch_without_throttling() {
        let watch = Watch::new();
        let idle = Duration::from_secs(120);
        tokio::time::sleep(Duration::from_secs(100)).await;
        watch.touch();
        let started = Instant::now();
        watch.stalled(idle).await;
        assert_eq!(started.elapsed(), idle);

        let throttled = Arc::new(Watch::new());
        throttled.throttling(true);
        let waiter = throttled.clone();
        let handle = tokio::spawn(async move { waiter.stalled(idle).await });
        tokio::time::sleep(Duration::from_secs(500)).await;
        assert!(!handle.is_finished());
        throttled.touch();
        throttled.throttling(false);
        tokio::time::sleep(idle + Duration::from_secs(1)).await;
        assert!(handle.is_finished());
    }
}
