use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use astrum_db::entity::asset;
use axum::http::{StatusCode, header};
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{Value, json};

use crate::api::{JSON_TYPE, Summary};
use crate::names::{self, Coll};
use crate::{ARTIFACT, GZIP_TYPE, ID, METADATA};

const DISCOVERY_TTL: Duration = Duration::from_secs(600);
const MAX_DOCUMENT: usize = 64 * 1024 * 1024;
const MAX_ROOT: usize = 1024 * 1024;
const MAX_PAGES: usize = 200;
const PAGE_SIZE: usize = 100;
const MAX_CACHED_COLLECTIONS: u64 = 10_000;

fn accept_json() -> (header::HeaderName, String) {
    (header::ACCEPT, JSON_TYPE.to_owned())
}

fn bad_gateway(what: &str, error: impl std::fmt::Display) -> ContentError {
    ContentError::BadGateway(format!("{what}: {error}"))
}

pub async fn read_json(ctx: &Ctx, found: &Asset) -> Result<Value, ContentError> {
    if found.size.max(0) as usize > MAX_DOCUMENT {
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
    serde_json::from_slice(&body)
        .map_err(|error| bad_gateway("a cached Galaxy document is not JSON", error))
}

/// Finds the v3 API of the remote the way `ansible-galaxy` finds it: the configured address
/// itself, else `api/` below it, must answer `available_versions` with a `v3` entry.
#[derive(Default)]
pub struct Remote {
    roots: Mutex<HashMap<String, (String, url::Url, Instant)>>,
}

impl Remote {
    fn remote_url(ctx: &Ctx) -> Result<String, ContentError> {
        ctx.repository
            .attributes
            .proxy
            .as_ref()
            .and_then(|proxy| proxy.remote_url.clone())
            .ok_or_else(|| {
                ContentError::internal(std::io::Error::other(format!(
                    "the proxy {} has no remote URL",
                    ctx.name()
                )))
            })
    }

    pub async fn v3(&self, ctx: &Ctx) -> Result<url::Url, ContentError> {
        let remote = Self::remote_url(ctx)?;
        if let Some((known, root, at)) = self
            .roots
            .lock()
            .expect("the remote root lock is never poisoned")
            .get(ctx.name())
            && *known == remote
            && at.elapsed() < DISCOVERY_TTL
        {
            return Ok(root.clone());
        }
        let base = url::Url::parse(&format!("{}/", remote.trim_end_matches('/')))
            .map_err(|error| bad_gateway("the remote URL is not valid", error))?;
        let mut candidates = vec![base.clone()];
        if !base.path().ends_with("/api/") {
            candidates.push(
                base.join("api/")
                    .map_err(|error| bad_gateway("the remote URL is not valid", error))?,
            );
        }
        let mut failure = None;
        for candidate in candidates {
            match self.available_v3(ctx, candidate).await {
                Ok(Some(root)) => {
                    self.roots
                        .lock()
                        .expect("the remote root lock is never poisoned")
                        .insert(
                            ctx.name().to_owned(),
                            (remote, root.clone(), Instant::now()),
                        );
                    return Ok(root);
                }
                Ok(None) => {}
                Err(error) => failure = Some(error),
            }
        }
        Err(failure.unwrap_or_else(|| {
            ContentError::BadGateway(format!(
                "the remote {remote} answers no Galaxy API with version v3"
            ))
        }))
    }

    async fn available_v3(
        &self,
        ctx: &Ctx,
        candidate: url::Url,
    ) -> Result<Option<url::Url>, ContentError> {
        let answer = proxy::request(ctx, candidate.clone(), &[accept_json()]).await?;
        if answer.status != StatusCode::OK {
            return Ok(None);
        }
        let Ok(body) = answer.bytes(MAX_ROOT).await else {
            return Ok(None);
        };
        let Ok(document) = serde_json::from_slice::<Value>(&body) else {
            return Ok(None);
        };
        Ok(document
            .get("available_versions")
            .and_then(|versions| versions.get("v3"))
            .and_then(Value::as_str)
            .and_then(|v3| candidate.join(v3).ok()))
    }

    async fn api_url(&self, ctx: &Ctx, relative: &str) -> Result<url::Url, ContentError> {
        self.v3(ctx)
            .await?
            .join(relative)
            .map_err(|error| bad_gateway("the remote address is not valid", error))
    }

    async fn document(
        &self,
        ctx: &Ctx,
        path: String,
        relative: String,
    ) -> Result<Option<(Asset, Value)>, ContentError> {
        let found = proxy::fetch_from(
            ctx,
            &path,
            Freshness::Metadata,
            || async {
                self.api_url(ctx, &relative)
                    .await
                    .map(|url| Some(Origin::new(url).header(header::ACCEPT, JSON_TYPE)))
            },
            || {
                PutAsset::cached(path.clone())
                    .kind(METADATA)
                    .content_type(JSON_TYPE)
            },
        )
        .await?;
        match found {
            Some(found) => {
                let document = read_json(ctx, &found).await?;
                Ok(Some((found, document)))
            }
            None => Ok(None),
        }
    }

    pub async fn collection(
        &self,
        ctx: &Ctx,
        coll: &Coll,
    ) -> Result<Option<Summary>, ContentError> {
        let Some((found, document)) = self
            .document(
                ctx,
                coll.cached_collection(),
                format!("collections/{}/{}/", coll.namespace, coll.name),
            )
            .await?
        else {
            return Ok(None);
        };
        let Some(summary) = Summary::from_remote(coll.clone(), &document) else {
            return Ok(None);
        };
        let kept = json!({ID: {"highest": summary.highest, "deprecated": summary.deprecated, "created_at": summary.created_at, "updated_at": summary.updated_at, "download_count": summary.download_count}});
        if found.attributes != kept {
            store::set_attributes(ctx.db(), found.id, kept).await?;
        }
        Ok(Some(summary))
    }

    /// The collections whose documents this proxy has cached, read from the asset attributes.
    pub async fn cached_collections(&self, ctx: &Ctx) -> Result<Vec<Summary>, ContentError> {
        let rows = asset::Entity::find()
            .filter(asset::Column::Repository.eq(ctx.name()))
            .filter(asset::Column::Path.like("/api/v3/collections/%/collection.json"))
            .order_by_asc(asset::Column::Path)
            .limit(MAX_CACHED_COLLECTIONS)
            .all(ctx.db())
            .await?;
        let mut summaries = Vec::new();
        for row in rows {
            let parts: Vec<&str> = row.path.trim_start_matches('/').split('/').collect();
            let [
                "api",
                "v3",
                "collections",
                namespace,
                name,
                "collection.json",
            ] = parts.as_slice()
            else {
                continue;
            };
            let Some(coll) = Coll::new(namespace, name) else {
                continue;
            };
            if !ctx.allowed(astrum_content::View::Read, &row.path) {
                continue;
            }
            let kept = row.attributes.get(ID).cloned().unwrap_or(Value::Null);
            let mut remote = kept.clone();
            if let Some(object) = remote.as_object_mut() {
                object.insert(
                    "highest_version".to_owned(),
                    json!({"version": kept.get("highest").cloned().unwrap_or(Value::Null)}),
                );
            }
            if let Some(summary) = Summary::from_remote(coll, &remote) {
                summaries.push(summary);
            }
        }
        Ok(summaries)
    }

    pub async fn versions(
        &self,
        ctx: &Ctx,
        coll: &Coll,
    ) -> Result<Option<Vec<Value>>, ContentError> {
        let path = coll.cached_versions();
        let existing = store::find(ctx.db(), ctx.name(), &path).await?;
        if let Some(found) = &existing
            && proxy::is_fresh(ctx, found, Freshness::Metadata)
            && let Ok(document) = read_json(ctx, found).await
        {
            return Ok(Some(data_of(&document)));
        }
        match self.all_versions(ctx, coll).await {
            Ok(Some(data)) => {
                let document = json!({"meta": {"count": data.len()}, "data": data});
                let put = PutAsset::cached(path.clone())
                    .kind(METADATA)
                    .content_type(JSON_TYPE);
                store::put(
                    ctx,
                    put,
                    astrum_blob::content_of(document.to_string().into_bytes()),
                )
                .await?;
                Ok(Some(data))
            }
            Ok(None) => match &existing {
                Some(found) => Ok(Some(data_of(&read_json(ctx, found).await?))),
                None => Ok(None),
            },
            Err(error) => match &existing {
                Some(found) => {
                    tracing::info!(repository = ctx.name(), path, %error, "serving a stale copy");
                    Ok(Some(data_of(&read_json(ctx, found).await?)))
                }
                None => Err(error),
            },
        }
    }

    async fn all_versions(
        &self,
        ctx: &Ctx,
        coll: &Coll,
    ) -> Result<Option<Vec<Value>>, ContentError> {
        let mut url = self
            .api_url(
                ctx,
                &format!(
                    "collections/{}/{}/versions/?limit={PAGE_SIZE}",
                    coll.namespace, coll.name
                ),
            )
            .await?;
        let mut data = Vec::new();
        for page in 0..MAX_PAGES {
            let answer = proxy::request(ctx, url.clone(), &[accept_json()]).await?;
            match answer.status {
                StatusCode::OK => {}
                StatusCode::NOT_FOUND | StatusCode::GONE if page == 0 => return Ok(None),
                StatusCode::NOT_FOUND | StatusCode::GONE => break,
                status => {
                    return Err(ContentError::BadGateway(format!(
                        "the remote answered {status} to {url}"
                    )));
                }
            }
            let body = answer.bytes(MAX_DOCUMENT).await?;
            let document: Value = serde_json::from_slice(&body)
                .map_err(|error| bad_gateway("the remote version list is not JSON", error))?;
            data.extend(data_of(&document));
            let next = document
                .pointer("/links/next")
                .or_else(|| document.get("next"))
                .and_then(Value::as_str)
                .filter(|next| !next.is_empty());
            match next {
                Some(next) => {
                    url = url
                        .join(next)
                        .map_err(|error| bad_gateway("the next page address is not valid", error))?
                }
                None => break,
            }
        }
        Ok(Some(data))
    }

    pub async fn version(
        &self,
        ctx: &Ctx,
        coll: &Coll,
        version: &str,
    ) -> Result<Option<Value>, ContentError> {
        Ok(self
            .document(
                ctx,
                coll.cached_version(version),
                format!(
                    "collections/{}/{}/versions/{version}/",
                    coll.namespace, coll.name
                ),
            )
            .await?
            .map(|(_, document)| document))
    }

    pub async fn artifact(
        &self,
        ctx: &Ctx,
        coll: &Coll,
        version: &str,
    ) -> Result<Option<Asset>, ContentError> {
        let path = coll.artifact_path(version);
        let described: Mutex<(Option<String>, Value)> = Mutex::new((None, Value::Null));
        let found = proxy::fetch_from(
            ctx,
            &path,
            Freshness::Content,
            || async {
                let Some(document) = self.version(ctx, coll, version).await? else {
                    return Ok(None);
                };
                *described
                    .lock()
                    .expect("the description lock is never poisoned") = (
                    document
                        .pointer("/artifact/sha256")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    attributes_of(&document),
                );
                let Some(address) = document.get("download_url").and_then(Value::as_str) else {
                    return Ok(None);
                };
                let url = self
                    .v3(ctx)
                    .await?
                    .join(address)
                    .map_err(|error| bad_gateway("the download address is not valid", error))?;
                Ok(Some(Origin::new(url)))
            },
            || {
                PutAsset::cached(path.clone())
                    .kind(ARTIFACT)
                    .content_type(GZIP_TYPE)
                    .component(ComponentKey {
                        namespace: coll.namespace.clone(),
                        name: coll.name.clone(),
                        version: version.to_owned(),
                        kind: String::new(),
                        attributes: json!({ ID: described
                            .lock()
                            .expect("the description lock is never poisoned")
                            .1
                            .clone() }),
                    })
            },
        )
        .await?;
        let expected = described
            .lock()
            .expect("the description lock is never poisoned")
            .0
            .clone();
        if let (Some(found), Some(expected)) = (&found, &expected)
            && !found.sha256.eq_ignore_ascii_case(expected)
        {
            store::delete_asset(&ctx.content, found.clone()).await?;
            return Err(ContentError::BadGateway(format!(
                "the remote archive of {coll} {version} does not match the SHA-256 its version document gives"
            )));
        }
        Ok(found)
    }
}

pub fn data_of(document: &Value) -> Vec<Value> {
    document
        .get("data")
        .or_else(|| document.get("results"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// The component attributes of a proxied version, in the shape hosted versions keep.
pub fn attributes_of(document: &Value) -> Value {
    let metadata = document.get("metadata").cloned().unwrap_or(Value::Null);
    let pick = |key: &str| metadata.get(key).cloned().unwrap_or(Value::Null);
    let list = |key: &str| match pick(key) {
        Value::Array(items) => Value::Array(items),
        _ => json!([]),
    };
    json!({
        "authors": list("authors"),
        "description": pick("description"),
        "license": list("license"),
        "tags": list("tags"),
        "dependencies": match pick("dependencies") {
            Value::Object(dependencies) => Value::Object(dependencies),
            _ => json!({}),
        },
        "repository": pick("repository"),
        "documentation": pick("documentation"),
        "homepage": pick("homepage"),
        "issues": pick("issues"),
        "requires_ansible": document.get("requires_ansible").cloned().unwrap_or(Value::Null),
        "contents": list("contents"),
    })
}

pub fn valid_entry(value: &Value) -> bool {
    value
        .get("version")
        .and_then(Value::as_str)
        .is_some_and(names::valid_version)
}
