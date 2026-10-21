use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use futures_util::TryStreamExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::model::{Meta, Stored, rewrite};
use crate::names::{self, FileRef};
use crate::{FILE, ID, METADATA, VSIX, VSIX_TYPE};

const MAX_DOCUMENT: u64 = 64 * 1024 * 1024;
const MAX_CHECKSUM: usize = 4096;

pub fn base(ctx: &Ctx) -> Option<String> {
    ctx.repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .map(|remote| remote.trim_end_matches('/').to_owned())
}

pub fn hashed(path: &str, key: &str) -> String {
    let digest = hex::encode(Sha256::digest(key.as_bytes()));
    format!("{}/-/{}.json", path.trim_end_matches('/'), &digest[..32])
}

pub struct Wanted<'a> {
    pub cache: String,
    pub path: &'a str,
    pub query: Option<String>,
    pub accept: &'static str,
}

impl<'a> Wanted<'a> {
    pub fn plain(path: &'a str) -> Wanted<'a> {
        Wanted {
            cache: path.to_owned(),
            path,
            query: None,
            accept: "application/json",
        }
    }
}

async fn read_blob(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
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

pub async fn document(ctx: &Ctx, wanted: Wanted<'_>) -> Result<Option<Value>, ContentError> {
    let cache = wanted.cache.clone();
    let found = proxy::fetch_from(
        ctx,
        &wanted.cache,
        Freshness::Metadata,
        || async {
            let url = proxy::remote_url(&ctx.repository, wanted.path, wanted.query.as_deref())?;
            Ok(Some(Origin::new(url).header(header::ACCEPT, wanted.accept)))
        },
        || {
            PutAsset::cached(cache.clone())
                .kind(METADATA)
                .content_type("application/json")
        },
    )
    .await?;
    let Some(found) = found else {
        return Ok(None);
    };
    let body = read_blob(ctx, &found, MAX_DOCUMENT).await?;
    let mut value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(error) => {
            tracing::info!(repository = ctx.name(), path = wanted.cache, %error, "the remote answered a document that is not JSON");
            store::delete(ctx, &wanted.cache).await?;
            return Err(ContentError::BadGateway(format!(
                "the remote answer for {} is not JSON",
                wanted.path
            )));
        }
    };
    if let Some(base) = base(ctx) {
        rewrite(&mut value, &base, &ctx.repository_url());
    }
    Ok(Some(value))
}

pub fn version_path(file: &FileRef) -> String {
    format!(
        "/api/{}/{}/{}/{}",
        file.namespace, file.name, file.platform, file.version
    )
}

async fn checksum(ctx: &Ctx, file: &FileRef) -> Result<Option<String>, ContentError> {
    let sibling = FileRef {
        file: names::sha256_name(&file.namespace, &file.name, &file.version, &file.platform),
        ..file.clone()
    };
    let url = proxy::remote_url(&ctx.repository, &sibling.path(), None)?;
    let answer = proxy::request(ctx, url, &[]).await?;
    if answer.status != StatusCode::OK {
        return Ok(None);
    }
    let body = answer.bytes(MAX_CHECKSUM).await?;
    let text = String::from_utf8_lossy(&body);
    Ok(text
        .split_whitespace()
        .next()
        .filter(|sum| sum.len() == 64 && sum.chars().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase))
}

pub async fn file(ctx: &Ctx, file: &FileRef) -> Result<Option<Asset>, ContentError> {
    let path = file.path();
    let vsix = file.is_vsix();
    let asked = AtomicBool::new(false);
    let described: Mutex<Option<Value>> = Mutex::new(None);
    let found = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async {
            asked.store(true, Ordering::Relaxed);
            if vsix {
                let version = version_path(file);
                match document(ctx, Wanted::plain(&version)).await {
                    Ok(document) => {
                        *described.lock().expect("the description lock is never poisoned") = document;
                    }
                    Err(error) => {
                        tracing::info!(repository = ctx.name(), path = version, %error, "the version document of a package is not available");
                    }
                }
            }
            Ok(Some(Origin::new(proxy::remote_url(
                &ctx.repository,
                &path,
                None,
            )?)))
        },
        || {
            let put = PutAsset::cached(path.clone()).kind(match vsix {
                true => VSIX,
                false => FILE,
            });
            if !vsix {
                return put;
            }
            let meta = described
                .lock()
                .expect("the description lock is never poisoned")
                .as_ref()
                .map(Meta::from_document)
                .unwrap_or_default();
            let stored = Stored {
                target_platform: file.platform.clone(),
                files: Default::default(),
                meta: meta.clone(),
            };
            put.content_type(VSIX_TYPE)
                .attributes(json!({ ID: stored }))
                .component(ComponentKey {
                    namespace: file.namespace.clone(),
                    name: file.name.clone(),
                    version: file.version.clone(),
                    kind: String::new(),
                    attributes: json!({ ID: meta }),
                })
        },
    )
    .await?;
    let Some(found) = found else {
        return Ok(None);
    };
    if vsix
        && asked.load(Ordering::Relaxed)
        && let Some(expected) = checksum(ctx, file).await?
        && !found.sha256.eq_ignore_ascii_case(&expected)
    {
        tracing::warn!(
            repository = ctx.name(),
            path,
            expected,
            found = found.sha256,
            "a fetched extension package does not match the checksum of the remote"
        );
        store::delete_asset(&ctx.content, found).await?;
        return Err(ContentError::BadGateway(format!(
            "the package {} from the remote does not match its SHA-256 checksum",
            file.file
        )));
    }
    Ok(Some(found))
}

pub async fn passthrough(
    ctx: &Ctx,
    path: &str,
    query: Option<&str>,
    cache: &str,
) -> Result<Option<Asset>, ContentError> {
    let cached = cache.to_owned();
    proxy::fetch_from(
        ctx,
        cache,
        Freshness::Content,
        || async {
            Ok(Some(Origin::new(proxy::remote_url(
                &ctx.repository,
                path,
                query,
            )?)))
        },
        || PutAsset::cached(cached).kind(FILE),
    )
    .await
}

pub fn item(ctx: &Ctx, item: &str) -> Option<String> {
    let base = base(ctx)?;
    let query: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("itemName", item)
        .finish();
    Some(format!("{base}/vscode/item?{query}"))
}
