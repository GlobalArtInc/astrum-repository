use std::sync::Mutex;

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use futures_util::TryStreamExt;
use serde_json::{Value, json};

use crate::api::{self, JSON_TYPE, Params};
use crate::archive;
use crate::names::Module;
use crate::{ARTIFACT, GZIP_TYPE, ID, METADATA};

const MAX_DOCUMENT: usize = 64 * 1024 * 1024;
const MAX_PAGES: usize = 100;
const MAX_LISTING_PAGES: usize = 10;
const PAGE_SIZE: usize = 100;
const LIST_FIELDS: &str = "exclude_fields=readme+changelog+license+reference+docs";

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
        .map_err(|error| bad_gateway("a cached Forge document is not JSON", error))
}

pub fn upstream_url(ctx: &Ctx, link: &str) -> Result<url::Url, ContentError> {
    if let Ok(absolute) = url::Url::parse(link)
        && matches!(absolute.scheme(), "http" | "https")
    {
        return Ok(absolute);
    }
    let (path, query) = match link.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (link, None),
    };
    let path = path.trim_start_matches('/');
    proxy::remote_url(&ctx.repository, &format!("/{path}"), query)
}

fn results_of(document: &Value) -> Vec<Value> {
    document
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn next_of(document: &Value) -> Option<&str> {
    document
        .pointer("/pagination/next")
        .and_then(Value::as_str)
        .filter(|next| !next.is_empty())
}

fn rejected(status: StatusCode, url: &url::Url, body: &[u8]) -> ContentError {
    let detail = serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|document| {
            document
                .get("errors")
                .and_then(Value::as_array)
                .map(|errors| {
                    errors
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("; ")
                })
        })
        .filter(|detail| !detail.is_empty());
    match (status, detail) {
        (StatusCode::BAD_REQUEST, Some(detail)) => ContentError::BadRequest(detail),
        (StatusCode::NOT_FOUND, _) => ContentError::not_found(url.path().to_owned()),
        (status, _) => ContentError::BadGateway(format!("the remote answered {status} to {url}")),
    }
}

#[derive(Default)]
pub struct Remote;

impl Remote {
    async fn document(
        &self,
        ctx: &Ctx,
        path: String,
        remote: String,
    ) -> Result<Option<Value>, ContentError> {
        let found = proxy::fetch_from(
            ctx,
            &path,
            Freshness::Metadata,
            || async {
                upstream_url(ctx, &remote)
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
            Some(found) => Ok(Some(read_json(ctx, &found).await?)),
            None => Ok(None),
        }
    }

    pub async fn module(&self, ctx: &Ctx, module: &Module) -> Result<Option<Value>, ContentError> {
        Ok(self
            .document(ctx, module.cached_module(), module.uri())
            .await?
            .map(|document| api::rewrite_module(document, module)))
    }

    async fn raw_release(
        &self,
        ctx: &Ctx,
        module: &Module,
        version: &str,
    ) -> Result<Option<Value>, ContentError> {
        self.document(
            ctx,
            module.cached_release(version),
            module.release_uri(version),
        )
        .await
    }

    pub async fn release(
        &self,
        ctx: &Ctx,
        module: &Module,
        version: &str,
    ) -> Result<Option<Value>, ContentError> {
        Ok(self
            .raw_release(ctx, module, version)
            .await?
            .map(|document| api::rewrite_release(document, module, version)))
    }

    pub async fn releases(
        &self,
        ctx: &Ctx,
        module: &Module,
    ) -> Result<Option<Vec<Value>>, ContentError> {
        let path = module.cached_releases();
        let existing = store::find(ctx.db(), ctx.name(), &path).await?;
        let raw = match &existing {
            Some(found) if proxy::is_fresh(ctx, found, Freshness::Metadata) => {
                match read_json(ctx, found).await {
                    Ok(document) => Some(results_of(&document)),
                    Err(_) => None,
                }
            }
            _ => None,
        };
        let raw = match raw {
            Some(raw) => raw,
            None => match self.all_releases(ctx, module).await {
                Ok(Some(data)) => {
                    let document = json!({"results": data});
                    let put = PutAsset::cached(path.clone())
                        .kind(METADATA)
                        .content_type(JSON_TYPE);
                    store::put(
                        ctx,
                        put,
                        astrum_blob::content_of(document.to_string().into_bytes()),
                    )
                    .await?;
                    data
                }
                Ok(None) => match &existing {
                    Some(found) => results_of(&read_json(ctx, found).await?),
                    None => return Ok(None),
                },
                Err(error) => match &existing {
                    Some(found) => {
                        tracing::info!(repository = ctx.name(), path, %error, "serving a stale copy");
                        results_of(&read_json(ctx, found).await?)
                    }
                    None => return Err(error),
                },
            },
        };
        Ok(Some(
            raw.into_iter()
                .filter_map(api::rewrite_remote_release)
                .filter(|release| {
                    api::release_slug_of(release).is_some_and(|(found, _)| found == *module)
                })
                .collect(),
        ))
    }

    async fn all_releases(
        &self,
        ctx: &Ctx,
        module: &Module,
    ) -> Result<Option<Vec<Value>>, ContentError> {
        let mut url = upstream_url(
            ctx,
            &format!(
                "/v3/releases?module={}&limit={PAGE_SIZE}&offset=0&{LIST_FIELDS}",
                module.slug()
            ),
        )?;
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
                .map_err(|error| bad_gateway("the remote release list is not JSON", error))?;
            data.extend(results_of(&document));
            match next_of(&document) {
                Some(next) => url = upstream_url(ctx, next)?,
                None => break,
            }
        }
        Ok(Some(data))
    }

    pub async fn listing(&self, ctx: &Ctx, base: &str, query: &str) -> Result<Value, ContentError> {
        let url = upstream_url(ctx, &format!("{base}?{query}"))?;
        let answer = proxy::request(ctx, url.clone(), &[accept_json()]).await?;
        let status = answer.status;
        let body = answer.bytes(MAX_DOCUMENT).await?;
        if status != StatusCode::OK {
            return Err(rejected(status, &url, &body));
        }
        let mut document: Value = serde_json::from_slice(&body)
            .map_err(|error| bad_gateway("the remote list is not JSON", error))?;
        let rewrite: fn(Value) -> Option<Value> = match base {
            "/v3/modules" => api::rewrite_remote_module,
            _ => api::rewrite_remote_release,
        };
        if let Some(object) = document.as_object_mut() {
            let results: Vec<Value> = results_of(&Value::Object(object.clone()))
                .into_iter()
                .filter_map(rewrite)
                .collect();
            object.insert("results".to_owned(), Value::Array(results));
            if let Some(Value::Object(pagination)) = object.get_mut("pagination") {
                for key in ["first", "previous", "current", "next"] {
                    if let Some(Value::String(link)) = pagination.get_mut(key) {
                        *link = api::relative(link);
                    }
                }
            }
        }
        Ok(document)
    }

    pub async fn gather(
        &self,
        ctx: &Ctx,
        base: &str,
        params: &Params,
        want: usize,
    ) -> Result<(Vec<Value>, usize), ContentError> {
        let filters = params.query_without_paging();
        let mut items = Vec::new();
        let mut total = 0;
        let mut offset = 0;
        for _ in 0..MAX_LISTING_PAGES {
            let limit = want.saturating_sub(items.len()).clamp(1, PAGE_SIZE);
            let mut query = format!("limit={limit}&offset={offset}");
            if !filters.is_empty() {
                query = format!("{filters}&{query}");
            }
            let document = self.listing(ctx, base, &query).await?;
            let results = results_of(&document);
            total = document
                .pointer("/pagination/total")
                .and_then(Value::as_u64)
                .map(|total| total as usize)
                .unwrap_or(total.max(items.len() + results.len()));
            let found = results.len();
            items.extend(results);
            offset += found;
            if found == 0 || items.len() >= want || next_of(&document).is_none() {
                break;
            }
        }
        let total = total.max(items.len());
        Ok((items, total))
    }

    pub async fn artifact(
        &self,
        ctx: &Ctx,
        module: &Module,
        version: &str,
    ) -> Result<Option<Asset>, ContentError> {
        let path = module.artifact_path(version);
        let described: Mutex<(Option<(&'static str, String)>, Value)> =
            Mutex::new((None, Value::Null));
        let found = proxy::fetch_from(
            ctx,
            &path,
            Freshness::Content,
            || async {
                let Some(document) = self.raw_release(ctx, module, version).await? else {
                    return Ok(None);
                };
                let digest = |key: &str| {
                    document
                        .get(key)
                        .and_then(Value::as_str)
                        .filter(|digest| !digest.is_empty())
                        .map(str::to_owned)
                };
                let expected = digest("file_sha256")
                    .map(|digest| ("SHA-256", digest))
                    .or_else(|| digest("file_md5").map(|digest| ("MD5", digest)));
                *described
                    .lock()
                    .expect("the description lock is never poisoned") = (
                    expected,
                    archive::attributes_of(
                        &document
                            .get("metadata")
                            .cloned()
                            .unwrap_or_else(|| json!({})),
                    ),
                );
                let address = document
                    .get("file_uri")
                    .and_then(Value::as_str)
                    .filter(|address| !address.is_empty())
                    .map(str::to_owned)
                    .unwrap_or_else(|| module.file_uri(version));
                Ok(Some(Origin::new(upstream_url(ctx, &address)?)))
            },
            || {
                PutAsset::cached(path.clone())
                    .kind(ARTIFACT)
                    .content_type(GZIP_TYPE)
                    .component(ComponentKey {
                        namespace: module.owner.clone(),
                        name: module.name.clone(),
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
        if let (Some(found), Some((kind, expected))) = (&found, &expected) {
            let actual = match *kind {
                "MD5" => &found.md5,
                _ => &found.sha256,
            };
            if !actual.eq_ignore_ascii_case(expected) {
                store::delete_asset(&ctx.content, found.clone()).await?;
                return Err(ContentError::BadGateway(format!(
                    "the remote archive of {module} {version} does not match the {kind} its release document gives"
                )));
            }
        }
        Ok(found)
    }
}
