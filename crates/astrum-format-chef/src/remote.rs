use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use futures_util::TryStreamExt;
use serde_json::{Map, Value, json};

use crate::docs::{Cookbook, Item, JSON_TYPE, MAX_ITEMS, Query, Universe, parse_universe};
use crate::{COOKBOOK, GZIP_TYPE, ID, METADATA, hosted, names};

pub const UNIVERSE_PATH: &str = "/universe";
const MAX_DOCUMENT: u64 = 512 * 1024 * 1024;
const MAX_ANSWER: usize = 16 * 1024 * 1024;
const MAX_PAGES: usize = 10;

pub fn cookbook_path(name: &str) -> String {
    format!("/api/v1/cookbooks/{name}")
}

pub fn version_path(name: &str, version: &str) -> String {
    format!("{}/versions/{version}", cookbook_path(name))
}

fn accept_json() -> (header::HeaderName, String) {
    (header::ACCEPT, JSON_TYPE.to_owned())
}

async fn document(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let owned = path.to_owned();
    proxy::fetch_from(
        ctx,
        path,
        Freshness::Metadata,
        || async {
            let url = proxy::remote_url(&ctx.repository, path, None)?;
            Ok(Some(
                Origin::new(url).header(header::ACCEPT, JSON_TYPE.to_owned()),
            ))
        },
        || {
            PutAsset::cached(owned)
                .kind(METADATA)
                .content_type(JSON_TYPE)
        },
    )
    .await
}

async fn read_bytes(ctx: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > MAX_DOCUMENT {
        return Err(ContentError::BadGateway(format!(
            "{} is too large",
            found.path
        )));
    }
    let blob_store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = blob_store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn not_json(ctx: &Ctx, found: &Asset, error: serde_json::Error) -> ContentError {
    tracing::info!(repository = ctx.name(), path = found.path, %error, "the remote answered a document that is not JSON");
    if let Err(error) = store::delete_asset(&ctx.content, found.clone()).await {
        tracing::warn!(repository = ctx.name(), path = found.path, %error, "a cached document that is not JSON stays");
    }
    ContentError::BadGateway(format!("the remote answer for {} is not JSON", found.path))
}

async fn read_json(ctx: &Ctx, found: &Asset) -> Result<Value, ContentError> {
    let body = read_bytes(ctx, found).await?;
    match serde_json::from_slice::<Value>(&body) {
        Ok(value) => Ok(value),
        Err(error) => Err(not_json(ctx, found, error).await),
    }
}

pub async fn universe_asset(ctx: &Ctx) -> Result<Option<Asset>, ContentError> {
    document(ctx, UNIVERSE_PATH).await
}

pub async fn universe_of(ctx: &Ctx, found: &Asset) -> Result<Universe, ContentError> {
    let body = read_bytes(ctx, found).await?;
    let parsed = tokio::task::spawn_blocking(move || {
        serde_json::from_slice::<Value>(&body).map(|value| parse_universe(&value))
    })
    .await
    .map_err(ContentError::internal)?;
    match parsed {
        Ok(universe) => Ok(universe),
        Err(error) => Err(not_json(ctx, found, error).await),
    }
}

pub async fn cookbook(ctx: &Ctx, name: &str) -> Result<Option<Cookbook>, ContentError> {
    let Some(found) = document(ctx, &cookbook_path(name)).await? else {
        return Ok(None);
    };
    Ok(Cookbook::from_remote(name, &read_json(ctx, &found).await?)
        .filter(|cookbook| !cookbook.versions.is_empty()))
}

pub async fn version(
    ctx: &Ctx,
    name: &str,
    version: &str,
) -> Result<Option<Map<String, Value>>, ContentError> {
    let Some(found) = document(ctx, &version_path(name, version)).await? else {
        return Ok(None);
    };
    Ok(read_json(ctx, &found).await?.as_object().cloned())
}

fn origin_of(
    ctx: &Ctx,
    document: Option<&Map<String, Value>>,
    name: &str,
    version: &str,
) -> Result<url::Url, ContentError> {
    let base = ctx
        .repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .and_then(|remote| url::Url::parse(&format!("{}/", remote.trim_end_matches('/'))).ok());
    let listed = document
        .and_then(|document| document.get("file"))
        .and_then(Value::as_str)
        .zip(base)
        .and_then(|(file, base)| base.join(file).ok())
        .filter(|address| matches!(address.scheme(), "http" | "https"));
    match listed {
        Some(address) => Ok(address),
        None => proxy::remote_url(
            &ctx.repository,
            &format!("{}/download", version_path(name, version)),
            None,
        ),
    }
}

async fn describe(ctx: &Ctx, found: &Asset, name: &str, version: &str) {
    let Some(component_id) = found.component_id else {
        return;
    };
    let component = match store::component_by_id(ctx.db(), component_id).await {
        Ok(Some(component)) => component,
        Ok(None) => return,
        Err(error) => {
            tracing::info!(repository = ctx.name(), %error, "the component of a cached cookbook cannot be read");
            return;
        }
    };
    if component.attributes.get(ID).is_some_and(Value::is_object) {
        return;
    }
    let attributes = match hosted::read_metadata(ctx, &found.blob_store, found.blob_id.into()).await
    {
        Ok(Ok(metadata)) if metadata.name == name && metadata.version == version => {
            metadata.attributes(hosted::DEFAULT_CATEGORY)
        }
        Ok(Ok(metadata)) => {
            tracing::info!(
                repository = ctx.name(),
                name,
                version,
                found = metadata.name,
                found_version = metadata.version,
                "a cached cookbook archive names another cookbook"
            );
            json!({})
        }
        Ok(Err(message)) => {
            tracing::info!(
                repository = ctx.name(),
                name,
                version,
                message,
                "a cached cookbook archive has no usable metadata.json"
            );
            json!({})
        }
        Err(error) => {
            tracing::info!(repository = ctx.name(), %error, "a cached cookbook archive cannot be read");
            return;
        }
    };
    if let Err(error) =
        store::set_component_attributes(ctx.db(), component_id, json!({ ID: attributes })).await
    {
        tracing::info!(repository = ctx.name(), %error, "the metadata of a cached cookbook was not kept");
    }
}

pub async fn tarball(ctx: &Ctx, name: &str, version: &str) -> Result<Option<Asset>, ContentError> {
    let path = names::tarball_path(name, version);
    let found = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async {
            let document = self::version(ctx, name, version).await?;
            Ok(Some(Origin::new(origin_of(
                ctx,
                document.as_ref(),
                name,
                version,
            )?)))
        },
        || {
            PutAsset::cached(path.clone())
                .kind(COOKBOOK)
                .content_type(GZIP_TYPE)
                .component(ComponentKey {
                    namespace: String::new(),
                    name: name.to_owned(),
                    version: version.to_owned(),
                    kind: String::new(),
                    attributes: json!({}),
                })
        },
    )
    .await?;
    if let Some(found) = &found {
        describe(ctx, found, name, version).await;
    }
    Ok(found)
}

pub async fn listing(
    ctx: &Ctx,
    path: &str,
    query: &Query,
    start: usize,
    items: usize,
) -> Result<(usize, Vec<Item>), ContentError> {
    let wanted = {
        let mut pairs = url::form_urlencoded::Serializer::new(String::new());
        if !query.q.is_empty() {
            pairs.append_pair("q", &query.q);
        }
        pairs.append_pair("start", &start.to_string());
        pairs.append_pair("items", &items.to_string());
        if let Some(order) = &query.order {
            pairs.append_pair("order", order);
        }
        if let Some(user) = &query.user {
            pairs.append_pair("user", user);
        }
        pairs.finish()
    };
    let url = proxy::remote_url(&ctx.repository, path, Some(&wanted))?;
    let answer = proxy::request(ctx, url.clone(), &[accept_json()]).await?;
    match answer.status {
        StatusCode::OK => {}
        StatusCode::NOT_FOUND | StatusCode::GONE => return Ok((0, Vec::new())),
        status => {
            return Err(ContentError::BadGateway(format!(
                "the remote answered {status} to {url}"
            )));
        }
    }
    let body = answer.bytes(MAX_ANSWER).await?;
    let value: Value = serde_json::from_slice(&body).map_err(|error| {
        ContentError::BadGateway(format!("the remote answer to {url} is not JSON: {error}"))
    })?;
    let found: Vec<Item> = value
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Item::from_remote)
        .collect();
    let total = value
        .get("total")
        .and_then(Value::as_u64)
        .map(|total| total as usize)
        .unwrap_or(found.len());
    Ok((total, found))
}

pub async fn gather(
    ctx: &Ctx,
    path: &str,
    query: &Query,
    wanted: usize,
) -> Result<(usize, Vec<Item>), ContentError> {
    let mut items: Vec<Item> = Vec::new();
    let mut total = 0;
    for page in 0..MAX_PAGES {
        let start = page * MAX_ITEMS;
        if start >= wanted {
            break;
        }
        let (count, found) = listing(ctx, path, query, start, MAX_ITEMS).await?;
        total = count;
        let received = found.len();
        items.extend(found);
        if received < MAX_ITEMS || start + received >= count {
            break;
        }
    }
    Ok((total.max(items.len()), items))
}
