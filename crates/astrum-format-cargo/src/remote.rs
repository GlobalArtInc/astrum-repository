use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use futures_util::TryStreamExt;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::index;
use crate::{CRATE, CRATE_TYPE, ID, INDEX, INDEX_TYPE, crate_path};

const CONFIG_TTL: Duration = Duration::from_secs(3600);
const MAX_CONFIG: usize = 64 * 1024;
const MAX_INDEX: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize)]
pub struct Upstream {
    pub dl: String,
}

#[derive(Default)]
pub struct Cache {
    configs: Mutex<HashMap<String, (Instant, String, Upstream)>>,
}

impl Cache {
    fn cached(&self, repository: &str, remote: &str) -> Option<(bool, Upstream)> {
        self.configs
            .lock()
            .expect("the upstream cache lock is never poisoned")
            .get(repository)
            .filter(|(_, known, _)| known == remote)
            .map(|(at, _, upstream)| (at.elapsed() < CONFIG_TTL, upstream.clone()))
    }

    fn remember(&self, repository: &str, remote: &str, upstream: Upstream) {
        self.configs
            .lock()
            .expect("the upstream cache lock is never poisoned")
            .insert(
                repository.to_owned(),
                (Instant::now(), remote.to_owned(), upstream),
            );
    }
}

fn remote(ctx: &Ctx) -> String {
    ctx.repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.clone())
        .unwrap_or_default()
}

pub async fn upstream(ctx: &Ctx, cache: &Cache) -> Result<Upstream, ContentError> {
    let remote = remote(ctx);
    let stale = match cache.cached(ctx.name(), &remote) {
        Some((true, upstream)) => return Ok(upstream),
        Some((false, upstream)) => Some(upstream),
        None => None,
    };
    let fetched = async {
        let url = proxy::remote_url(&ctx.repository, "/config.json", None)?;
        let answer =
            proxy::request(ctx, url, &[(header::ACCEPT, "application/json".to_owned())]).await?;
        if answer.status != StatusCode::OK {
            return Err(ContentError::BadGateway(format!(
                "the remote answered {} for config.json",
                answer.status
            )));
        }
        let body = answer.bytes(MAX_CONFIG).await?;
        serde_json::from_slice::<Upstream>(&body).map_err(|error| {
            ContentError::BadGateway(format!("the remote config.json is not valid: {error}"))
        })
    }
    .await;
    match (fetched, stale) {
        (Ok(upstream), _) => {
            cache.remember(ctx.name(), &remote, upstream.clone());
            Ok(upstream)
        }
        (Err(error), Some(stale)) => {
            tracing::info!(repository = ctx.name(), %error, "using the remembered config.json of the remote");
            Ok(stale)
        }
        (Err(error), None) => Err(error),
    }
}

pub async fn index_file(ctx: &Ctx, name: &str) -> Result<Option<Asset>, ContentError> {
    let path = index::index_path(name);
    proxy::fetch(ctx, &path, None, Freshness::Metadata, || {
        PutAsset::cached(path.clone())
            .kind(INDEX)
            .content_type(INDEX_TYPE)
    })
    .await
}

pub async fn read(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
        return Err(ContentError::BadGateway(format!(
            "{} is too large",
            found.path
        )));
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn entries(ctx: &Ctx, name: &str) -> Result<Option<Vec<Value>>, ContentError> {
    match index_file(ctx, name).await? {
        Some(found) => {
            let body = read(ctx, &found, MAX_INDEX).await?;
            Ok(Some(index::parse(&String::from_utf8_lossy(&body))))
        }
        None => Ok(None),
    }
}

pub async fn crate_file(
    ctx: &Ctx,
    cache: &Cache,
    name: &str,
    version: &str,
) -> Result<Option<Asset>, ContentError> {
    let path = crate_path(name, version);
    let entry = std::sync::Arc::new(Mutex::new(None::<Value>));
    let remembered = entry.clone();
    proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async move {
            let upstream = upstream(ctx, cache).await?;
            let listed = entries(ctx, name).await.ok().flatten().unwrap_or_default();
            let found = listed
                .into_iter()
                .find(|line| line.get("vers").and_then(Value::as_str) == Some(version));
            let Some(found) = found else {
                return Ok(None);
            };
            let checksum = found
                .get("cksum")
                .and_then(Value::as_str)
                .map(str::to_owned);
            *remembered
                .lock()
                .expect("the index entry lock is never poisoned") = Some(found);
            let url = index::download_url(&upstream.dl, name, version, checksum.as_deref());
            let url = url::Url::parse(&url).map_err(|error| {
                ContentError::BadGateway(format!(
                    "the remote download address is not valid: {error}"
                ))
            })?;
            Ok(Some(Origin::new(url)))
        },
        || {
            let found = entry
                .lock()
                .expect("the index entry lock is never poisoned")
                .take();
            PutAsset::cached(path.clone())
                .kind(CRATE)
                .content_type(CRATE_TYPE)
                .component(ComponentKey {
                    namespace: String::new(),
                    name: name.to_owned(),
                    version: version.to_owned(),
                    kind: String::new(),
                    attributes: json!({ ID: { "index": found } }),
                })
        },
    )
    .await
}
