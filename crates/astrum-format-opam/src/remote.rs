use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use astrum_content::proxy;
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::StatusCode;

use crate::names::{Algorithm, CacheKey, INDEX_PATH, REPO_PATH};
use crate::{CACHE_KIND, INDEX_KIND, REPO_KIND};

#[derive(Default)]
pub struct Flights {
    held: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
}

impl Flights {
    fn flight(&self, key: String) -> Arc<tokio::sync::Mutex<()>> {
        let mut held = self.held.lock().expect("the flight lock is never poisoned");
        if let Some(existing) = held.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        held.retain(|_, weak| weak.strong_count() > 0);
        let fresh = Arc::new(tokio::sync::Mutex::new(()));
        held.insert(key, Arc::downgrade(&fresh));
        fresh
    }
}

pub async fn metadata(
    ctx: &Ctx,
    path: &'static str,
    kind: &'static str,
) -> Result<Option<Asset>, ContentError> {
    proxy::fetch(ctx, path, None, Freshness::Metadata, || {
        PutAsset::cached(path).kind(kind)
    })
    .await
}

pub async fn index(ctx: &Ctx) -> Result<Option<Asset>, ContentError> {
    metadata(ctx, INDEX_PATH, INDEX_KIND).await
}

pub async fn repo(ctx: &Ctx) -> Result<Option<Asset>, ContentError> {
    metadata(ctx, REPO_PATH, REPO_KIND).await
}

pub async fn cache(
    ctx: &Ctx,
    flights: &Flights,
    key: &CacheKey,
) -> Result<Option<Asset>, ContentError> {
    let path = key.path();
    if let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? {
        return Ok(Some(found));
    }
    let flight = flights.flight(format!("{}|{path}", ctx.name()));
    let _held = flight.lock().await;
    if let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? {
        return Ok(Some(found));
    }
    let url = proxy::remote_url(&ctx.repository, &path, None)?;
    let remote = proxy::request(ctx, url, &[]).await?;
    match remote.status {
        StatusCode::OK => {}
        StatusCode::NOT_FOUND | StatusCode::GONE => return Ok(None),
        status => {
            return Err(ContentError::BadGateway(format!(
                "the remote answered {status}"
            )));
        }
    }
    let content_type = remote
        .header("content-type")
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_owned());
    let written = store::write_blob(ctx, remote.body).await?;
    let actual = match key.algorithm {
        Algorithm::Sha256 => &written.digests.sha256,
        Algorithm::Sha512 => &written.digests.sha512,
    };
    if *actual != key.hex {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        tracing::warn!(
            repository = ctx.name(),
            path,
            "the remote sent a file that does not match its digest"
        );
        return Err(ContentError::BadGateway(format!(
            "the remote sent a file that does not match {}:{}",
            key.algorithm.as_str(),
            key.hex
        )));
    }
    let mut put = PutAsset::cached(&path).kind(CACHE_KIND);
    put.content_type = content_type;
    store::adopt(ctx, put, &written).await.map(Some)
}
