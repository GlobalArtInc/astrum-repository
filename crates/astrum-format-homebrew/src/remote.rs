use std::collections::HashMap;
use std::sync::{Mutex, Weak};

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use serde_json::Value;

use crate::names::Item;

pub const REGISTRY: &str = "https://ghcr.io";
pub const API_KIND: &str = "api";
pub const MANIFEST_KIND: &str = "manifest";
pub const BLOB_KIND: &str = "blob";

const ANONYMOUS_PULL: &str = "Bearer QQ==";
const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const CORE: &str = "homebrew/core";
const API_NAMESPACE: &str = "api";

#[derive(Default)]
pub struct Flights {
    held: Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>,
}

impl Flights {
    fn flight(&self, key: String) -> std::sync::Arc<tokio::sync::Mutex<()>> {
        let mut held = self.held.lock().expect("the flight lock is never poisoned");
        if let Some(existing) = held.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        held.retain(|_, weak| weak.strong_count() > 0);
        let fresh = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        held.insert(key, std::sync::Arc::downgrade(&fresh));
        fresh
    }
}

fn component(namespace: &str, name: &str, version: &str) -> ComponentKey {
    ComponentKey {
        namespace: namespace.to_owned(),
        name: name.to_owned(),
        version: version.to_owned(),
        kind: String::new(),
        attributes: Value::Object(Default::default()),
    }
}

fn registry_url(registry: &str, image: &str, rest: &str) -> Result<url::Url, ContentError> {
    url::Url::parse(&format!(
        "{}/v2/{CORE}/{image}/{rest}",
        registry.trim_end_matches('/')
    ))
    .map_err(|error| ContentError::BadGateway(format!("the registry URL is not valid: {error}")))
}

pub async fn fetch(
    ctx: &Ctx,
    flights: &Flights,
    registry: &str,
    path: &str,
    item: &Item,
) -> Result<Option<Asset>, ContentError> {
    match item {
        Item::Api { endpoint } => {
            let name = endpoint.rsplit('/').next().unwrap_or(endpoint);
            proxy::fetch_from(
                ctx,
                path,
                Freshness::Metadata,
                || async {
                    proxy::remote_url(&ctx.repository, endpoint, None)
                        .map(|url| Some(Origin::new(url)))
                },
                || {
                    PutAsset::cached(path).kind(API_KIND).component(component(
                        API_NAMESPACE,
                        name,
                        "",
                    ))
                },
            )
            .await
        }
        Item::Manifest { image, tag } => {
            proxy::fetch_from(
                ctx,
                path,
                Freshness::Metadata,
                || async {
                    registry_url(registry, image, &format!("manifests/{tag}")).map(|url| {
                        Some(
                            Origin::new(url)
                                .header(header::AUTHORIZATION, ANONYMOUS_PULL)
                                .header(header::ACCEPT, OCI_INDEX),
                        )
                    })
                },
                || {
                    PutAsset::cached(path)
                        .kind(MANIFEST_KIND)
                        .component(component(CORE, image, tag))
                },
            )
            .await
        }
        Item::Blob { image, hex } => blob(ctx, flights, registry, path, image, hex).await,
    }
}

async fn blob(
    ctx: &Ctx,
    flights: &Flights,
    registry: &str,
    path: &str,
    image: &str,
    hex: &str,
) -> Result<Option<Asset>, ContentError> {
    if let Some(found) = store::find(ctx.db(), ctx.name(), path).await? {
        return Ok(Some(found));
    }
    let flight = flights.flight(format!("{}|{path}", ctx.name()));
    let _held = flight.lock().await;
    if let Some(found) = store::find(ctx.db(), ctx.name(), path).await? {
        return Ok(Some(found));
    }
    let url = registry_url(registry, image, &format!("blobs/sha256:{hex}"))?;
    let remote = proxy::request(
        ctx,
        url,
        &[(header::AUTHORIZATION, ANONYMOUS_PULL.to_owned())],
    )
    .await?;
    match remote.status {
        StatusCode::OK => {}
        StatusCode::NOT_FOUND | StatusCode::GONE => return Ok(None),
        status => {
            return Err(ContentError::BadGateway(format!(
                "the registry answered {status}"
            )));
        }
    }
    let content_type = remote
        .header("content-type")
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_owned());
    let written = store::write_blob(ctx, remote.body).await?;
    if written.digests.sha256 != hex {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        tracing::warn!(
            repository = ctx.name(),
            path,
            "the registry sent a blob that does not match its digest"
        );
        return Err(ContentError::BadGateway(format!(
            "the registry sent a blob that does not match sha256:{hex}"
        )));
    }
    let mut put = PutAsset::cached(path).kind(BLOB_KIND).component(component(
        CORE,
        image,
        &format!("sha256:{hex}"),
    ));
    put.content_type = content_type;
    store::adopt(ctx, put, &written).await.map(Some)
}
