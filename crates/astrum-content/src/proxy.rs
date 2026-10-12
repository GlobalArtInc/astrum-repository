use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use astrum_blob::ContentStream;
use astrum_db::setting;
use astrum_repo::config::{AuthenticationType, Repository};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::http::{HeaderMap, StatusCode, header};
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::TryStreamExt;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use crate::http::{self, Guard};
use crate::store::{self, Asset, PutAsset};
use crate::{ContentError, Ctx};

const DEFAULT_MAX_AGE_MINUTES: i64 = 1440;
const DEFAULT_NEGATIVE_MINUTES: i64 = 1440;
const SSRF_REFRESH: Duration = Duration::from_secs(10);
const FIRST_BLOCK: Duration = Duration::from_secs(40);
const LONGEST_BLOCK: Duration = Duration::from_secs(20 * 60);

const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'[')
    .add(b']');

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Content,
    Metadata,
}

#[derive(Clone, Copy)]
struct Block {
    until: Instant,
    next: Duration,
}

type Key = (String, String);

#[derive(Default)]
pub(crate) struct ProxyState {
    negative: Mutex<HashMap<Key, Instant>>,
    flights: Mutex<HashMap<Key, Weak<tokio::sync::Mutex<()>>>>,
    clients: Mutex<HashMap<String, (String, reqwest::Client)>>,
    blocks: Mutex<HashMap<String, Block>>,
    ssrf: Mutex<Option<(Instant, SsrfProtection)>>,
}

impl ProxyState {
    pub(crate) fn forget(&self, repository: &str) {
        self.negative
            .lock()
            .expect("the negative cache lock is never poisoned")
            .retain(|(name, _), _| name != repository);
        self.clients
            .lock()
            .expect("the client cache lock is never poisoned")
            .remove(repository);
        self.blocks
            .lock()
            .expect("the block lock is never poisoned")
            .remove(repository);
        *self
            .ssrf
            .lock()
            .expect("the SSRF cache lock is never poisoned") = None;
    }

    fn negative_hit(&self, key: &Key) -> bool {
        let mut negative = self
            .negative
            .lock()
            .expect("the negative cache lock is never poisoned");
        match negative.get(key) {
            Some(until) if *until > Instant::now() => true,
            Some(_) => {
                negative.remove(key);
                false
            }
            None => false,
        }
    }

    fn remember_missing(&self, key: Key, ttl: Duration) {
        let mut negative = self
            .negative
            .lock()
            .expect("the negative cache lock is never poisoned");
        if negative.len() > 100_000 {
            let now = Instant::now();
            negative.retain(|_, until| *until > now);
        }
        negative.insert(key, Instant::now() + ttl);
    }

    fn flight(&self, key: Key) -> Arc<tokio::sync::Mutex<()>> {
        let mut flights = self
            .flights
            .lock()
            .expect("the flight lock is never poisoned");
        if let Some(existing) = flights.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        flights.retain(|_, weak| weak.strong_count() > 0);
        let fresh = Arc::new(tokio::sync::Mutex::new(()));
        flights.insert(key, Arc::downgrade(&fresh));
        fresh
    }

    fn blocked(&self, repository: &str) -> bool {
        self.blocks
            .lock()
            .expect("the block lock is never poisoned")
            .get(repository)
            .is_some_and(|block| block.until > Instant::now())
    }

    fn failed(&self, repository: &str) {
        let mut blocks = self
            .blocks
            .lock()
            .expect("the block lock is never poisoned");
        let next = blocks
            .get(repository)
            .map(|block| block.next)
            .unwrap_or(FIRST_BLOCK);
        blocks.insert(
            repository.to_owned(),
            Block {
                until: Instant::now() + next,
                next: (next * 2).min(LONGEST_BLOCK),
            },
        );
        tracing::warn!(
            repository,
            seconds = next.as_secs(),
            "the remote failed; the proxy stops asking it for a while"
        );
    }

    fn succeeded(&self, repository: &str) {
        self.blocks
            .lock()
            .expect("the block lock is never poisoned")
            .remove(repository);
    }
}

async fn guard(ctx: &Ctx) -> Result<SsrfProtection, ContentError> {
    let state = ctx.content.proxy_state();
    if let Some((at, settings)) = state
        .ssrf
        .lock()
        .expect("the SSRF cache lock is never poisoned")
        .clone()
        && at.elapsed() < SSRF_REFRESH
    {
        return Ok(settings);
    }
    let settings: SsrfProtection = setting::get(ctx.db(), SSRF_KEY).await?.unwrap_or_default();
    *state
        .ssrf
        .lock()
        .expect("the SSRF cache lock is never poisoned") = Some((Instant::now(), settings.clone()));
    Ok(settings)
}

async fn client(ctx: &Ctx) -> Result<(reqwest::Client, Arc<Guard>), ContentError> {
    let settings = guard(ctx).await?;
    let guard = Arc::new(Guard::new(&settings));
    let fingerprint = format!(
        "{}|{}",
        serde_json::to_string(&ctx.repository.attributes.http_client).unwrap_or_default(),
        serde_json::to_string(&settings).unwrap_or_default()
    );
    let state = ctx.content.proxy_state();
    if let Some((stored, client)) = state
        .clients
        .lock()
        .expect("the client cache lock is never poisoned")
        .get(ctx.name())
        && *stored == fingerprint
    {
        return Ok((client.clone(), guard));
    }
    let client = http::build(&ctx.repository, guard.clone()).map_err(ContentError::internal)?;
    state
        .clients
        .lock()
        .expect("the client cache lock is never poisoned")
        .insert(ctx.name().to_owned(), (fingerprint, client.clone()));
    Ok((client, guard))
}

pub fn remote_url(
    repository: &Repository,
    path: &str,
    query: Option<&str>,
) -> Result<url::Url, ContentError> {
    let base = repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .ok_or_else(|| {
            ContentError::internal(anyhow::anyhow!(
                "the proxy {} has no remote URL",
                repository.name
            ))
        })?;
    let encoded: String = path
        .trim_start_matches('/')
        .split('/')
        .map(|segment| utf8_percent_encode(segment, PATH_SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/");
    let mut joined = format!("{}/{encoded}", base.trim_end_matches('/'));
    if let Some(query) = query.filter(|query| !query.is_empty()) {
        joined.push('?');
        joined.push_str(query);
    }
    url::Url::parse(&joined)
        .map_err(|error| ContentError::BadGateway(format!("the remote URL is not valid: {error}")))
}

pub struct Remote {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: ContentStream,
}

impl Remote {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    pub async fn bytes(self, limit: usize) -> Result<Vec<u8>, ContentError> {
        let mut body = self.body;
        let mut collected = Vec::new();
        while let Some(chunk) = body
            .try_next()
            .await
            .map_err(|error| ContentError::BadGateway(error.to_string()))?
        {
            if collected.len() + chunk.len() > limit {
                return Err(ContentError::BadGateway(
                    "the remote answer is too large".to_owned(),
                ));
            }
            collected.extend_from_slice(&chunk);
        }
        Ok(collected)
    }
}

pub fn remote_blocked(ctx: &Ctx) -> bool {
    let manual = ctx
        .repository
        .attributes
        .http_client
        .as_ref()
        .and_then(|client| client.blocked)
        .unwrap_or(false);
    manual || ctx.content.proxy_state().blocked(ctx.name())
}

pub async fn request(
    ctx: &Ctx,
    url: url::Url,
    extra: &[(header::HeaderName, String)],
) -> Result<Remote, ContentError> {
    if remote_blocked(ctx) {
        return Err(ContentError::BadGateway(format!(
            "the remote of {} is blocked",
            ctx.name()
        )));
    }
    let (client, guard) = client(ctx).await?;
    if !http::literal_allowed(&guard, &url) {
        return Err(ContentError::BadGateway(
            "the remote address is blocked by the SSRF protection".to_owned(),
        ));
    }
    let http_client = ctx.repository.attributes.http_client.as_ref();
    let retries = http_client
        .and_then(|client| client.connection.as_ref())
        .and_then(|connection| connection.retries)
        .unwrap_or(2)
        .clamp(0, 10) as usize;
    let mut attempt = 0;
    loop {
        let mut builder = client.get(url.clone());
        for (name, value) in extra {
            builder = builder.header(name, value);
        }
        if let Some(authentication) = http_client.and_then(|client| client.authentication.as_ref())
        {
            let open = |sealed: &Option<String>| {
                sealed
                    .as_deref()
                    .and_then(|sealed| ctx.content.secrets().open(sealed).ok())
            };
            builder = match authentication.kind {
                Some(AuthenticationType::BearerToken) => match open(&authentication.bearer_token) {
                    Some(token) => builder.bearer_auth(token),
                    None => builder,
                },
                Some(AuthenticationType::Username) => builder.basic_auth(
                    authentication.username.clone().unwrap_or_default(),
                    open(&authentication.password),
                ),
                _ => builder,
            };
        }
        match builder.send().await {
            Ok(response) => {
                let status = response.status();
                let auto_block = http_client
                    .and_then(|client| client.auto_block)
                    .unwrap_or(true);
                if (status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS)
                    && auto_block
                {
                    ctx.content.proxy_state().failed(ctx.name());
                } else {
                    ctx.content.proxy_state().succeeded(ctx.name());
                }
                let headers = response.headers().clone();
                let body: ContentStream =
                    Box::pin(response.bytes_stream().map_err(io::Error::other));
                return Ok(Remote {
                    status,
                    headers,
                    body,
                });
            }
            Err(error) if attempt < retries && (error.is_connect() || error.is_timeout()) => {
                attempt += 1;
                tracing::debug!(%error, attempt, "retrying the remote");
            }
            Err(error) => {
                if http_client
                    .and_then(|client| client.auto_block)
                    .unwrap_or(true)
                {
                    ctx.content.proxy_state().failed(ctx.name());
                }
                return Err(ContentError::BadGateway(format!(
                    "the remote did not answer: {error}"
                )));
            }
        }
    }
}

fn max_age(ctx: &Ctx, freshness: Freshness) -> Option<chrono::Duration> {
    let proxy = ctx.repository.attributes.proxy.as_ref();
    let minutes = match freshness {
        Freshness::Content => proxy.and_then(|proxy| proxy.content_max_age),
        Freshness::Metadata => proxy.and_then(|proxy| proxy.metadata_max_age),
    }
    .map(i64::from)
    .unwrap_or(DEFAULT_MAX_AGE_MINUTES);
    (minutes >= 0).then(|| chrono::Duration::minutes(minutes))
}

pub fn is_fresh(ctx: &Ctx, asset: &Asset, freshness: Freshness) -> bool {
    let Some(verified) = asset.last_verified_at else {
        return false;
    };
    match max_age(ctx, freshness) {
        None => true,
        Some(age) => Utc::now().fixed_offset() - verified < age,
    }
}

fn negative_ttl(ctx: &Ctx) -> Option<Duration> {
    let negative = ctx.repository.attributes.negative_cache.as_ref();
    if !negative
        .and_then(|negative| negative.enabled)
        .unwrap_or(true)
    {
        return None;
    }
    let minutes = negative
        .and_then(|negative| negative.time_to_live)
        .map(i64::from)
        .unwrap_or(DEFAULT_NEGATIVE_MINUTES);
    (minutes > 0).then(|| Duration::from_secs(minutes as u64 * 60))
}

fn parse_date(value: Option<&str>) -> Option<DateTime<FixedOffset>> {
    let parsed = httpdate::parse_http_date(value?).ok()?;
    Some(DateTime::<Utc>::from(parsed).fixed_offset())
}

pub async fn fetch(
    ctx: &Ctx,
    path: &str,
    query: Option<&str>,
    freshness: Freshness,
    describe: impl FnOnce() -> PutAsset,
) -> Result<Option<Asset>, ContentError> {
    let existing = store::find(ctx.db(), ctx.name(), path).await?;
    if let Some(asset) = &existing
        && is_fresh(ctx, asset, freshness)
    {
        return Ok(existing);
    }
    let key = (ctx.name().to_owned(), path.to_owned());
    let state = ctx.content.proxy_state();
    if existing.is_none() && state.negative_hit(&key) {
        return Ok(None);
    }
    let flight = state.flight(key.clone());
    let _held = flight.lock().await;
    let existing = store::find(ctx.db(), ctx.name(), path).await?;
    if let Some(asset) = &existing
        && is_fresh(ctx, asset, freshness)
    {
        return Ok(existing);
    }
    let url = remote_url(&ctx.repository, path, query)?;
    let mut conditional = Vec::new();
    if let Some(asset) = &existing {
        if let Some(etag) = &asset.remote_etag {
            conditional.push((header::IF_NONE_MATCH, etag.clone()));
        }
        if let Some(modified) = asset.remote_last_modified {
            conditional.push((
                header::IF_MODIFIED_SINCE,
                httpdate::fmt_http_date(modified.with_timezone(&Utc).into()),
            ));
        }
    }
    let remote = match request(ctx, url, &conditional).await {
        Ok(remote) => remote,
        Err(error) => {
            return match existing {
                Some(stale) => {
                    tracing::info!(repository = ctx.name(), path, %error, "serving a stale copy");
                    Ok(Some(stale))
                }
                None => Err(error),
            };
        }
    };
    match remote.status {
        StatusCode::OK => {
            let etag = remote.header("etag").map(str::to_owned);
            let last_modified = parse_date(remote.header("last-modified"));
            let content_type = remote
                .header("content-type")
                .and_then(|value| value.split(';').next())
                .map(|value| value.trim().to_owned());
            let mut put = describe();
            put.path = path.to_owned();
            put.verified = true;
            put.write_policy = None;
            put.check_content_type = false;
            put.remote_etag = etag;
            put.remote_last_modified = last_modified;
            if put.content_type.is_none() {
                put.content_type = content_type;
            }
            let stored = store::put(ctx, put, remote.body).await?;
            Ok(Some(stored))
        }
        StatusCode::NOT_MODIFIED => match existing {
            Some(asset) => {
                store::touch(ctx, asset.id, None, None).await?;
                Ok(store::find(ctx.db(), ctx.name(), path).await?)
            }
            None => Err(ContentError::BadGateway(
                "the remote answered 304 to a plain request".to_owned(),
            )),
        },
        StatusCode::NOT_FOUND | StatusCode::GONE => {
            if let Some(ttl) = negative_ttl(ctx) {
                state.remember_missing(key, ttl);
            }
            Ok(existing)
        }
        status => match existing {
            Some(stale) => Ok(Some(stale)),
            None if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN => {
                Err(ContentError::BadGateway(format!(
                    "the remote refused the proxy's credentials ({status})"
                )))
            }
            None => Err(ContentError::BadGateway(format!(
                "the remote answered {status}"
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use astrum_repo::config::{Attributes, Proxy};
    use astrum_repo::{RepositoryType, format};

    use super::*;

    fn proxy(remote: &str) -> Repository {
        Repository {
            name: "remote".to_owned(),
            format: format("raw").unwrap(),
            kind: RepositoryType::Proxy,
            online: true,
            attributes: Attributes {
                proxy: Some(Proxy {
                    remote_url: Some(remote.to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        }
    }

    #[test]
    fn remote_urls_join_and_encode() {
        let repository = proxy("https://repo.example/maven2/");
        assert_eq!(
            remote_url(&repository, "/org/a b/x+y.jar", None)
                .unwrap()
                .as_str(),
            "https://repo.example/maven2/org/a%20b/x+y.jar"
        );
        assert_eq!(
            remote_url(&proxy("https://repo.example"), "/simple/", Some("q=1"))
                .unwrap()
                .as_str(),
            "https://repo.example/simple/?q=1"
        );
    }

    #[test]
    fn blocks_grow_and_end() {
        let state = ProxyState::default();
        assert!(!state.blocked("r"));
        state.failed("r");
        assert!(state.blocked("r"));
        state.succeeded("r");
        assert!(!state.blocked("r"));
    }

    #[test]
    fn negative_entries_expire() {
        let state = ProxyState::default();
        let key = ("r".to_owned(), "/a".to_owned());
        state.remember_missing(key.clone(), Duration::from_secs(60));
        assert!(state.negative_hit(&key));
        state.forget("r");
        assert!(!state.negative_hit(&key));
        state.remember_missing(key.clone(), Duration::from_millis(0));
        assert!(!state.negative_hit(&key));
    }
}
