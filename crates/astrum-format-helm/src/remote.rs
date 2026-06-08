use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::header;
use chrono::Utc;
use futures_util::TryStreamExt;
use serde_json::json;
use uuid::Uuid;

use crate::chart::Chart;
use crate::hosted;
use crate::index::{self, Upstream};
use crate::{CHART, CHART_TYPE, INDEX, INDEX_TYPE, PROVENANCE, PROVENANCE_SUFFIX, PROVENANCE_TYPE};

const MAX_INDEX: u64 = 256 * 1024 * 1024;
const ACCEPT: &str = "application/yaml, text/yaml;q=0.9, */*;q=0.8";

pub struct Parsed {
    pub blob: Uuid,
    pub upstream: Upstream,
}

#[derive(Default)]
pub struct Cache {
    parsed: Mutex<HashMap<String, Arc<Parsed>>>,
}

impl Cache {
    fn get(&self, repository: &str, blob: Uuid) -> Option<Arc<Parsed>> {
        self.parsed
            .lock()
            .expect("the index cache lock is never poisoned")
            .get(repository)
            .filter(|parsed| parsed.blob == blob)
            .cloned()
    }

    fn put(&self, repository: &str, parsed: Arc<Parsed>) {
        self.parsed
            .lock()
            .expect("the index cache lock is never poisoned")
            .insert(repository.to_owned(), parsed);
    }

    pub fn forget(&self, repository: &str) {
        self.parsed
            .lock()
            .expect("the index cache lock is never poisoned")
            .remove(repository);
    }
}

async fn read(ctx: &Ctx, asset: &Asset) -> Result<Vec<u8>, ContentError> {
    if asset.size.max(0) as u64 > MAX_INDEX {
        return Err(ContentError::BadGateway(
            "the remote index is too large".to_owned(),
        ));
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = store.read(asset.blob_id.into()).await? else {
        return Err(ContentError::not_found(asset.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn index(ctx: &Ctx, cache: &Cache) -> Result<Option<Arc<Parsed>>, ContentError> {
    let url = proxy::remote_url(&ctx.repository, INDEX, None)?;
    let origin = Origin::new(url.clone()).header(header::ACCEPT, ACCEPT);
    let found = proxy::fetch_from(
        ctx,
        INDEX,
        Freshness::Metadata,
        || async { Ok(Some(origin)) },
        || {
            PutAsset::cached(INDEX)
                .kind(crate::INDEX_KIND)
                .content_type(INDEX_TYPE)
        },
    )
    .await?;
    let Some(asset) = found else {
        return Ok(None);
    };
    if let Some(parsed) = cache.get(ctx.name(), asset.blob_id) {
        return Ok(Some(parsed));
    }
    let body = read(ctx, &asset).await?;
    let fallback = asset.blob_updated_at.with_timezone(&Utc);
    let mut upstream = tokio::task::spawn_blocking(move || index::parse_upstream(&body, &url))
        .await
        .map_err(ContentError::internal)?
        .map_err(|error| {
            ContentError::BadGateway(format!("the remote index cannot be read: {error}"))
        })?;
    if upstream.skipped > 0 {
        tracing::info!(
            repository = ctx.name(),
            skipped = upstream.skipped,
            "the remote index lists charts the proxy cannot serve"
        );
    }
    upstream.listing.generated = upstream.listing.generated.or(Some(fallback));
    let parsed = Arc::new(Parsed {
        blob: asset.blob_id,
        upstream,
    });
    cache.put(ctx.name(), parsed.clone());
    Ok(Some(parsed))
}

pub async fn file(ctx: &Ctx, cache: &Cache, path: &str) -> Result<Option<Asset>, ContentError> {
    let name = path.trim_start_matches('/');
    let (chart_file, provenance) = match name.strip_suffix(PROVENANCE_SUFFIX) {
        Some(chart_file) => (chart_file, true),
        None => (name, false),
    };
    if name.contains('/') || !chart_file.ends_with(".tgz") {
        return store::find(ctx.db(), ctx.name(), path).await;
    }
    let listed: Mutex<Option<Chart>> = Mutex::new(None);
    proxy::fetch_from(
        ctx,
        path,
        Freshness::Content,
        || async {
            let Some(parsed) = index(ctx, cache).await? else {
                return Ok(None);
            };
            let Some(located) = parsed.upstream.files.get(chart_file) else {
                return Ok(None);
            };
            let mut url = located.urls[0].clone();
            if provenance {
                let path = format!("{}{PROVENANCE_SUFFIX}", url.path());
                url.set_path(&path);
            }
            *listed
                .lock()
                .expect("the listed chart lock is never poisoned") = Some(located.chart.clone());
            Ok(Some(Origin::new(url)))
        },
        || {
            let chart = listed
                .lock()
                .expect("the listed chart lock is never poisoned")
                .take()
                .unwrap_or_default();
            let put = PutAsset::cached(path).component(hosted::key(&chart));
            match provenance {
                true => put
                    .kind(PROVENANCE)
                    .content_type(PROVENANCE_TYPE)
                    .attributes(json!({})),
                false => put
                    .kind(CHART)
                    .content_type(CHART_TYPE)
                    .attributes(chart.attributes()),
            }
        },
    )
    .await
}
