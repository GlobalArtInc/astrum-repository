use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use astrum_content::proxy::{self, Remote};
use astrum_content::store::{self, Asset};
use astrum_content::{ContentError, Ctx, Freshness};
use astrum_repo::config::IndexType;
use axum::http::{HeaderName, Method, StatusCode, header};
use serde_json::Value;

use crate::manifest::{self, ACCEPTED};
use crate::reference::{Digest, Reference};
use crate::storage;

const MANIFEST_LIMIT: usize = 16 * 1024 * 1024;
const LIST_LIMIT: usize = 16 * 1024 * 1024;
const TOKEN_MARGIN: Duration = Duration::from_secs(10);
const TOKEN_LONGEST: Duration = Duration::from_secs(300);

#[derive(Default)]
pub struct Upstreams {
    tokens: Mutex<HashMap<String, (String, Instant)>>,
    flights: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
}

pub struct Challenge {
    realm: String,
    service: Option<String>,
    scope: Option<String>,
}

pub fn parse_challenge(value: &str) -> Option<Challenge> {
    let (scheme, rest) = value.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let mut params = HashMap::new();
    let mut remaining = rest.trim();
    while !remaining.is_empty() {
        let (key, after) = remaining.split_once('=')?;
        let key = key
            .trim()
            .trim_start_matches(',')
            .trim()
            .to_ascii_lowercase();
        let after = after.trim_start();
        let (value, next) = match after.strip_prefix('"') {
            Some(quoted) => {
                let end = quoted.find('"')?;
                (&quoted[..end], &quoted[end + 1..])
            }
            None => match after.find(',') {
                Some(end) => (&after[..end], &after[end..]),
                None => (after, ""),
            },
        };
        params.insert(key, value.to_owned());
        remaining = next.trim_start().trim_start_matches(',').trim_start();
    }
    Some(Challenge {
        realm: params.remove("realm")?,
        service: params.remove("service"),
        scope: params.remove("scope"),
    })
}

pub fn remote_name(ctx: &Ctx, image: &str) -> String {
    let hub = ctx
        .repository
        .attributes
        .docker_proxy
        .as_ref()
        .and_then(|proxy| proxy.index_type)
        == Some(IndexType::Hub);
    match hub && !image.contains('/') {
        true => format!("library/{image}"),
        false => image.to_owned(),
    }
}

fn remote_base(ctx: &Ctx) -> String {
    ctx.repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.clone())
        .unwrap_or_default()
}

impl Upstreams {
    fn flight(&self, key: String) -> Arc<tokio::sync::Mutex<()>> {
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

    fn cached_token(&self, key: &str) -> Option<String> {
        let mut tokens = self
            .tokens
            .lock()
            .expect("the token lock is never poisoned");
        match tokens.get(key) {
            Some((token, until)) if *until > Instant::now() => Some(token.clone()),
            Some(_) => {
                tokens.remove(key);
                None
            }
            None => None,
        }
    }

    fn keep_token(&self, key: String, token: String, lifetime: Duration) {
        let mut tokens = self
            .tokens
            .lock()
            .expect("the token lock is never poisoned");
        if tokens.len() > 10_000 {
            let now = Instant::now();
            tokens.retain(|_, (_, until)| *until > now);
        }
        tokens.insert(key, (token, Instant::now() + lifetime));
    }

    async fn fetch_token(
        &self,
        ctx: &Ctx,
        challenge: &Challenge,
        scope: &str,
    ) -> Result<(String, Duration), ContentError> {
        let mut url = url::Url::parse(&challenge.realm).map_err(|error| {
            ContentError::BadGateway(format!("the remote named a bad token realm: {error}"))
        })?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(service) = &challenge.service {
                query.append_pair("service", service);
            }
            query.append_pair("scope", challenge.scope.as_deref().unwrap_or(scope));
        }
        let remote =
            proxy::request(ctx, url, &[(header::ACCEPT, "application/json".to_owned())]).await?;
        if remote.status != StatusCode::OK {
            return Err(ContentError::BadGateway(format!(
                "the remote token service answered {}",
                remote.status
            )));
        }
        let body = remote.bytes(1024 * 1024).await?;
        let parsed: Value = serde_json::from_slice(&body).map_err(|_| {
            ContentError::BadGateway("the remote token service sent no JSON".to_owned())
        })?;
        let token = parsed
            .get("token")
            .or_else(|| parsed.get("access_token"))
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                ContentError::BadGateway("the remote token service sent no token".to_owned())
            })?
            .to_owned();
        let lifetime = parsed
            .get("expires_in")
            .and_then(Value::as_u64)
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(60))
            .min(TOKEN_LONGEST)
            .saturating_sub(TOKEN_MARGIN);
        Ok((token, lifetime))
    }

    pub async fn send(
        &self,
        ctx: &Ctx,
        method: Method,
        remote_image: &str,
        rest: &str,
        query: Option<&str>,
        accept: Option<&str>,
    ) -> Result<Remote, ContentError> {
        let path = format!("/v2/{remote_image}/{rest}");
        let url = proxy::remote_url(&ctx.repository, &path, query)?;
        let scope = format!("repository:{remote_image}:pull");
        let key = format!("{}|{}|{scope}", ctx.name(), remote_base(ctx));
        let mut extra: Vec<(HeaderName, String)> = Vec::new();
        if let Some(accept) = accept {
            extra.push((header::ACCEPT, accept.to_owned()));
        }
        let cached = self.cached_token(&key);
        if let Some(token) = &cached {
            extra.push((header::AUTHORIZATION, format!("Bearer {token}")));
        }
        let remote = proxy::send(ctx, method.clone(), url.clone(), &extra).await?;
        if remote.status != StatusCode::UNAUTHORIZED {
            return Ok(remote);
        }
        let Some(challenge) = remote.header("www-authenticate").and_then(parse_challenge) else {
            return Ok(remote);
        };
        let (token, lifetime) = self.fetch_token(ctx, &challenge, &scope).await?;
        self.keep_token(key, token.clone(), lifetime);
        extra.retain(|(name, _)| name != header::AUTHORIZATION);
        extra.push((header::AUTHORIZATION, format!("Bearer {token}")));
        proxy::send(ctx, method, url, &extra).await
    }

    async fn store_manifest(
        &self,
        ctx: &Ctx,
        image: &str,
        bytes: &[u8],
        content_type: Option<&str>,
        digest: &Digest,
        tag: Option<&str>,
    ) -> Result<Asset, ContentError> {
        let parsed = manifest::parse(bytes, content_type).map_err(|message| {
            ContentError::BadGateway(format!(
                "the remote sent a manifest this registry cannot read: {message}"
            ))
        })?;
        let by_digest = storage::manifest_path(image, &digest.to_string());
        let stored = match storage::find(ctx, &by_digest).await? {
            Some(existing) => existing,
            None => {
                let mut put =
                    storage::manifest_put(image, &digest.to_string(), digest, &parsed, None);
                put.verified = true;
                store::put(ctx, put, astrum_blob::content_of(bytes.to_vec())).await?
            }
        };
        match tag {
            Some(tag) => {
                let mut put = storage::manifest_put(image, tag, digest, &parsed, Some(tag));
                put.verified = true;
                store::put(ctx, put, astrum_blob::content_of(bytes.to_vec())).await
            }
            None => Ok(stored),
        }
    }

    pub async fn manifest(
        &self,
        ctx: &Ctx,
        image: &str,
        reference: &Reference,
    ) -> Result<Option<Asset>, ContentError> {
        let path = storage::manifest_path(image, &reference.as_str());
        let existing = storage::find(ctx, &path).await?;
        if let (Reference::Digest(_), Some(found)) = (reference, &existing) {
            return Ok(Some(found.clone()));
        }
        if let Some(found) = &existing
            && proxy::is_fresh(ctx, found, Freshness::Metadata)
        {
            return Ok(existing);
        }
        let flight = self.flight(format!("{}|{path}", ctx.name()));
        let _held = flight.lock().await;
        let existing = storage::find(ctx, &path).await?;
        if let Some(found) = &existing
            && (matches!(reference, Reference::Digest(_))
                || proxy::is_fresh(ctx, found, Freshness::Metadata))
        {
            return Ok(existing);
        }
        match self.refresh(ctx, image, reference, existing.as_ref()).await {
            Ok(found) => Ok(found.or(existing)),
            Err(error) => match existing {
                Some(stale) => {
                    tracing::info!(repository = ctx.name(), path, %error, "serving a stale manifest");
                    Ok(Some(stale))
                }
                None => Err(error),
            },
        }
    }

    async fn refresh(
        &self,
        ctx: &Ctx,
        image: &str,
        reference: &Reference,
        existing: Option<&Asset>,
    ) -> Result<Option<Asset>, ContentError> {
        let remote_image = remote_name(ctx, image);
        let rest = format!("manifests/{}", reference.as_str());
        if let Some(found) = existing
            && let Some(known) = storage::digest_of(found)
        {
            let head = self
                .send(
                    ctx,
                    Method::HEAD,
                    &remote_image,
                    &rest,
                    None,
                    Some(ACCEPTED),
                )
                .await?;
            let same = head.status == StatusCode::OK
                && head.header("docker-content-digest") == Some(known.to_string().as_str());
            if same {
                store::touch(ctx, found.id, None, None).await?;
                return storage::find(ctx, &found.path).await;
            }
            if head.status == StatusCode::NOT_FOUND {
                return Ok(None);
            }
        }
        let remote = self
            .send(ctx, Method::GET, &remote_image, &rest, None, Some(ACCEPTED))
            .await?;
        match remote.status {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Ok(None),
            status => {
                return Err(ContentError::BadGateway(format!(
                    "the remote answered {status}"
                )));
            }
        }
        let content_type = remote.header("content-type").map(manifest::bare);
        let bytes = remote.bytes(MANIFEST_LIMIT).await?;
        let digest = match reference {
            Reference::Digest(wanted) => {
                if !wanted.matches_bytes(&bytes) {
                    return Err(ContentError::BadGateway(format!(
                        "the remote sent a manifest that does not match {wanted}"
                    )));
                }
                wanted.clone()
            }
            Reference::Tag(_) => Digest::sha256(&bytes),
        };
        let tag = match reference {
            Reference::Tag(tag) => Some(tag.as_str()),
            Reference::Digest(_) => None,
        };
        self.store_manifest(ctx, image, &bytes, content_type.as_deref(), &digest, tag)
            .await
            .map(Some)
    }

    pub async fn blob(
        &self,
        ctx: &Ctx,
        image: &str,
        digest: &Digest,
    ) -> Result<Option<Asset>, ContentError> {
        let path = storage::blob_path(digest);
        if let Some(found) = storage::find(ctx, &path).await? {
            return Ok(Some(found));
        }
        let flight = self.flight(format!("{}|{path}", ctx.name()));
        let _held = flight.lock().await;
        if let Some(found) = storage::find(ctx, &path).await? {
            return Ok(Some(found));
        }
        let remote = self
            .send(
                ctx,
                Method::GET,
                &remote_name(ctx, image),
                &format!("blobs/{digest}"),
                None,
                None,
            )
            .await?;
        match remote.status {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Ok(None),
            status => {
                return Err(ContentError::BadGateway(format!(
                    "the remote answered {status}"
                )));
            }
        }
        let written = store::write_blob(ctx, remote.body).await?;
        if !digest.matches(&written.digests) {
            storage::discard(ctx, &written);
            return Err(ContentError::BadGateway(format!(
                "the remote sent a blob that does not match {digest}"
            )));
        }
        Ok(Some(storage::adopt_blob(ctx, digest, written).await?))
    }

    pub async fn blob_head(
        &self,
        ctx: &Ctx,
        image: &str,
        digest: &Digest,
    ) -> Result<Option<u64>, ContentError> {
        let remote = self
            .send(
                ctx,
                Method::HEAD,
                &remote_name(ctx, image),
                &format!("blobs/{digest}"),
                None,
                None,
            )
            .await?;
        match remote.status {
            StatusCode::OK => Ok(Some(
                remote
                    .header("content-length")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_default(),
            )),
            StatusCode::NOT_FOUND => Ok(None),
            status => Err(ContentError::BadGateway(format!(
                "the remote answered {status}"
            ))),
        }
    }

    pub async fn tags(
        &self,
        ctx: &Ctx,
        image: &str,
        query: Option<&str>,
    ) -> Result<Option<Vec<String>>, ContentError> {
        let remote = self
            .send(
                ctx,
                Method::GET,
                &remote_name(ctx, image),
                "tags/list",
                query,
                None,
            )
            .await?;
        match remote.status {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => return Ok(None),
            status => {
                return Err(ContentError::BadGateway(format!(
                    "the remote answered {status}"
                )));
            }
        }
        let body = remote.bytes(LIST_LIMIT).await?;
        let parsed: Value = serde_json::from_slice(&body)
            .map_err(|_| ContentError::BadGateway("the remote sent no tag list".to_owned()))?;
        Ok(Some(
            parsed
                .get("tags")
                .and_then(Value::as_array)
                .map(|tags| {
                    tags.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        ))
    }

    pub async fn referrers(
        &self,
        ctx: &Ctx,
        image: &str,
        subject: &Digest,
    ) -> Result<Vec<Value>, ContentError> {
        let remote_image = remote_name(ctx, image);
        let remote = self
            .send(
                ctx,
                Method::GET,
                &remote_image,
                &format!("referrers/{subject}"),
                None,
                Some(manifest::OCI_INDEX),
            )
            .await?;
        let index = match remote.status {
            StatusCode::OK => remote.bytes(LIST_LIMIT).await?,
            StatusCode::NOT_FOUND => {
                let fallback = self
                    .send(
                        ctx,
                        Method::GET,
                        &remote_image,
                        &format!("manifests/{}", subject.referrers_tag()),
                        None,
                        Some(manifest::OCI_INDEX),
                    )
                    .await?;
                match fallback.status {
                    StatusCode::OK => fallback.bytes(LIST_LIMIT).await?,
                    StatusCode::NOT_FOUND => return Ok(Vec::new()),
                    status => {
                        return Err(ContentError::BadGateway(format!(
                            "the remote answered {status}"
                        )));
                    }
                }
            }
            status => {
                return Err(ContentError::BadGateway(format!(
                    "the remote answered {status}"
                )));
            }
        };
        Ok(index_entries(&index))
    }
}

pub fn index_entries(bytes: &[u8]) -> Vec<Value> {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|index| index.get("manifests").and_then(Value::as_array).cloned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_challenges_are_parsed() {
        let challenge = parse_challenge(
            r#"Bearer realm="https://auth.docker.io/token",service="registry.docker.io",scope="repository:library/alpine:pull""#,
        )
        .unwrap();
        assert_eq!(challenge.realm, "https://auth.docker.io/token");
        assert_eq!(challenge.service.as_deref(), Some("registry.docker.io"));
        assert_eq!(
            challenge.scope.as_deref(),
            Some("repository:library/alpine:pull")
        );
        let bare = parse_challenge("bearer realm=http://h/t, service=h").unwrap();
        assert_eq!(bare.realm, "http://h/t");
        assert_eq!(bare.service.as_deref(), Some("h"));
        assert!(parse_challenge(r#"Basic realm="x""#).is_none());
        assert!(parse_challenge(r#"Bearer service="x""#).is_none());
    }

    #[test]
    fn index_entries_are_read() {
        let entries = index_entries(br#"{"schemaVersion":2,"manifests":[{"digest":"a"}]}"#);
        assert_eq!(entries.len(), 1);
        assert!(index_entries(b"nope").is_empty());
    }
}
