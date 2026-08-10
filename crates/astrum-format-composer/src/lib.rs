pub mod metadata;

use std::collections::HashSet;

use astrum_blob::Written;
use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip::{self, ZipError};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, UploadField, UploadForm,
    UploadSpec, View, group, serve,
};
use astrum_db::entity::component;
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use chrono::{SecondsFormat, Utc};
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::metadata::MINIFIED;

pub const ID: &str = "composer";
pub const DIST: &str = "dist";
pub const METADATA: &str = "metadata";
const ZIP_TYPE: &str = "application/zip";
const JSON_TYPE: &str = "application/json";
const MAX_MANIFEST: u64 = 4 * 1024 * 1024;
const MAX_METADATA: u64 = 128 * 1024 * 1024;

#[derive(Default)]
pub struct Composer;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Packages,
    List,
    Metadata {
        package: String,
        dev: bool,
    },
    Provider(String),
    Dist {
        vendor: String,
        project: String,
        version: String,
    },
    Upload {
        vendor: String,
        project: String,
        version: String,
    },
    Directory,
    Other,
}

fn package_name(vendor: &str, project: &str) -> Option<String> {
    (metadata::valid_part(vendor) && metadata::valid_part(project))
        .then(|| format!("{vendor}/{project}"))
}

fn route(path: &str) -> Route {
    match path {
        "/packages.json" => return Route::Packages,
        "/packages/list.json" => return Route::List,
        _ => {}
    }
    if path.ends_with('/') {
        return Route::Directory;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["p2", vendor, file] => {
            let (project, dev) = match file.strip_suffix("~dev.json") {
                Some(project) => (project, true),
                None => match file.strip_suffix(".json") {
                    Some(project) => (project, false),
                    None => return Route::Other,
                },
            };
            package_name(vendor, project)
                .map(|package| Route::Metadata { package, dev })
                .unwrap_or(Route::Other)
        }
        ["p", vendor, file] => file
            .strip_suffix(".json")
            .and_then(|project| package_name(vendor, project))
            .map(Route::Provider)
            .unwrap_or(Route::Other),
        ["packages", "upload", vendor, project, version]
            if package_name(vendor, project).is_some() && metadata::valid_version(version) =>
        {
            Route::Upload {
                vendor: (*vendor).to_owned(),
                project: (*project).to_owned(),
                version: (*version).to_owned(),
            }
        }
        [vendor, project, version, file]
            if package_name(vendor, project).is_some()
                && metadata::valid_version(version)
                && *file == format!("{vendor}-{project}-{version}.zip") =>
        {
            Route::Dist {
                vendor: (*vendor).to_owned(),
                project: (*project).to_owned(),
                version: (*version).to_owned(),
            }
        }
        _ => Route::Other,
    }
}

fn json_response(request: &ContentRequest, value: &Value) -> Response {
    let body = value.to_string();
    let etag = format!(
        "\"{}\"",
        hex::encode(&Sha256::digest(body.as_bytes())[..16])
    );
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    if unchanged {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .expect("an empty response is valid");
    }
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .expect("a JSON response is valid")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Composer repository. Add it with <code>composer config repositories.{name} composer {url}/</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

async fn read(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
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

fn uid(text: &str) -> u32 {
    let mut crc = flate2::Crc::new();
    crc.update(text.as_bytes());
    crc.sum()
}

fn zip_error(error: ZipError) -> ContentError {
    ContentError::BadRequest(error.to_string())
}

pub async fn manifest(source: &dyn zip::Source) -> Result<Map<String, Value>, ContentError> {
    let entries = zip::entries(source).await.map_err(zip_error)?;
    let found = entries
        .iter()
        .filter(|entry| {
            entry.name == "composer.json"
                || entry
                    .name
                    .strip_suffix("/composer.json")
                    .is_some_and(|folder| !folder.contains('/'))
        })
        .min_by_key(|entry| entry.name.len())
        .ok_or_else(|| {
            ContentError::BadRequest(
                "the archive has no composer.json at its top or in its only folder".to_owned(),
            )
        })?;
    let bytes = zip::read(source, found, MAX_MANIFEST)
        .await
        .map_err(zip_error)?;
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(object)) => Ok(object),
        Ok(_) => Err(ContentError::BadRequest(
            "composer.json is not a JSON object".to_owned(),
        )),
        Err(error) => Err(ContentError::BadRequest(format!(
            "composer.json is not valid JSON: {error}"
        ))),
    }
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

impl Composer {
    async fn store_dist(
        &self,
        ctx: &Ctx,
        written: Written,
        vendor: &str,
        project: &str,
        version: &str,
    ) -> Result<Asset, ContentError> {
        let path = metadata::dist_path(vendor, project, version);
        let refuse = |error: ContentError| {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            error
        };
        let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
        let allowed = if exists && write_policy(ctx) == WritePolicy::Allow {
            ctx.require(View::Edit, &path)
        } else {
            ctx.require(View::Add, &path)
        };
        if let Err(error) = allowed {
            return Err(refuse(error));
        }
        let blob_store = ctx.content.blob_stores().get(&written.store).await?;
        let source = zip::Blob {
            store: blob_store,
            id: written.id,
            size: written.size,
        };
        let manifest = match manifest(&source).await {
            Ok(manifest) => manifest,
            Err(error) => return Err(refuse(error)),
        };
        let put = PutAsset::hosted(ctx, path)
            .component(ComponentKey {
                namespace: vendor.to_owned(),
                name: project.to_owned(),
                version: version.to_owned(),
                kind: String::new(),
                attributes: json!({ ID: { "manifest": manifest } }),
            })
            .kind(DIST)
            .content_type(ZIP_TYPE);
        let stored = store::adopt(ctx, put, &written).await?;
        self.sync_stored(ctx, &format!("{vendor}/{project}"))
            .await?;
        Ok(stored)
    }

    async fn hosted_entries(&self, ctx: &Ctx, package: &str) -> Result<Vec<Value>, ContentError> {
        let Some((vendor, project)) = package.split_once('/') else {
            return Ok(Vec::new());
        };
        let rows = component::Entity::find()
            .filter(component::Column::Repository.eq(ctx.name()))
            .filter(component::Column::Format.eq(ID))
            .filter(component::Column::Namespace.eq(vendor))
            .filter(component::Column::Name.eq(project))
            .order_by_asc(component::Column::Id)
            .all(ctx.db())
            .await?;
        let mut entries = Vec::new();
        for row in rows {
            let path = metadata::dist_path(vendor, project, &row.version);
            if !ctx.allowed(View::Read, &path) {
                continue;
            }
            let Some(dist) = store::find(ctx.db(), ctx.name(), &path).await? else {
                continue;
            };
            let mut entry = Map::new();
            entry.insert("name".to_owned(), json!(package));
            entry.insert("version".to_owned(), json!(row.version));
            entry.insert(
                "dist".to_owned(),
                json!({
                    "url": format!("{}{path}", ctx.repository_url()),
                    "type": "zip",
                    "reference": dist.sha1,
                    "shasum": dist.sha1,
                }),
            );
            entry.insert(
                "time".to_owned(),
                json!(
                    dist.created_at
                        .with_timezone(&Utc)
                        .to_rfc3339_opts(SecondsFormat::Secs, false)
                ),
            );
            entry.insert(
                "uid".to_owned(),
                json!(uid(&format!("{package}:{}", row.version))),
            );
            if let Some(manifest) = row
                .attributes
                .get(ID)
                .and_then(|attributes| attributes.get("manifest"))
                .and_then(Value::as_object)
            {
                for (key, value) in manifest {
                    if !matches!(key.as_str(), "name" | "version" | "dist" | "time" | "uid") {
                        entry.insert(key.clone(), value.clone());
                    }
                }
            }
            entries.push(Value::Object(entry));
        }
        metadata::newest_first(&mut entries);
        Ok(entries)
    }

    async fn sync_stored(&self, ctx: &Ctx, package: &str) -> Result<(), ContentError> {
        let entries = self
            .hosted_entries(&ctx.member(ctx.repository.clone()), package)
            .await?;
        let p2_path = format!("/p2/{package}.json");
        let provider_path = format!("/p/{package}.json");
        if entries.is_empty() {
            store::delete(ctx, &p2_path).await?;
            store::delete(ctx, &provider_path).await?;
            return Ok(());
        }
        let documents = [
            (p2_path, p2_document(package, entries.clone())),
            (provider_path, provider_document(package, &entries)),
        ];
        for (path, document) in documents {
            let mut put = PutAsset::hosted(ctx, path)
                .kind(METADATA)
                .content_type(JSON_TYPE);
            put.check_content_type = false;
            put.write_policy = Some(WritePolicy::Allow);
            store::put(
                ctx,
                put,
                astrum_blob::content_of(document.to_string().into_bytes()),
            )
            .await?;
        }
        Ok(())
    }

    async fn upstream_metadata_url(
        &self,
        ctx: &Ctx,
        package: &str,
    ) -> Result<url::Url, ContentError> {
        let base = proxy::remote_url(&ctx.repository, "/", None)?;
        let template = match proxy::fetch(ctx, "/packages.json", None, Freshness::Metadata, || {
            PutAsset::cached("/packages.json").kind(METADATA).content_type(JSON_TYPE)
        })
        .await
        {
            Ok(Some(found)) => serde_json::from_slice::<Value>(&read(ctx, &found, MAX_METADATA).await?)
                .ok()
                .and_then(|root| root.get("metadata-url").and_then(Value::as_str).map(str::to_owned)),
            Ok(None) => None,
            Err(error) => {
                tracing::info!(repository = ctx.name(), %error, "the remote packages.json cannot be read");
                None
            }
        }
        .unwrap_or_else(|| "/p2/%package%.json".to_owned());
        base.join(&template.replace("%package%", package))
            .map_err(|error| {
                ContentError::BadGateway(format!(
                    "the remote metadata address is not valid: {error}"
                ))
            })
    }

    async fn proxied_metadata(
        &self,
        ctx: &Ctx,
        package: &str,
        dev: bool,
    ) -> Result<Option<Value>, ContentError> {
        let name = if dev {
            format!("{package}~dev")
        } else {
            package.to_owned()
        };
        let path = format!("/p2/{name}.json");
        let found = proxy::fetch_from(
            ctx,
            &path,
            Freshness::Metadata,
            || async {
                self.upstream_metadata_url(ctx, &name)
                    .await
                    .map(|url| Some(Origin::new(url)))
            },
            || {
                PutAsset::cached(path.clone())
                    .kind(METADATA)
                    .content_type(JSON_TYPE)
            },
        )
        .await?;
        match found {
            Some(found) => Ok(serde_json::from_slice(&read(ctx, &found, MAX_METADATA).await?).ok()),
            None => Ok(None),
        }
    }

    async fn entries(
        &self,
        ctx: &Ctx,
        package: &str,
        dev: bool,
    ) -> Result<Option<Vec<Value>>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let entries: Vec<Value> = self
                    .hosted_entries(ctx, package)
                    .await?
                    .into_iter()
                    .filter(|entry| {
                        metadata::is_dev(
                            entry
                                .get("version")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        ) == dev
                    })
                    .collect();
                Ok((!entries.is_empty()).then_some(entries))
            }
            RepositoryType::Proxy => Ok(self
                .proxied_metadata(ctx, package, dev)
                .await?
                .map(|document| metadata::entries_of(&document, package))
                .filter(|entries| !entries.is_empty())),
            RepositoryType::Group => {
                let mut seen = HashSet::new();
                let mut merged = Vec::new();
                let mut any = false;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.entries(&member, package, dev)).await {
                        Ok(Some(entries)) => {
                            any = true;
                            for entry in entries {
                                let version = entry
                                    .get("version")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_owned();
                                if seen.insert(version) {
                                    merged.push(entry);
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                metadata::newest_first(&mut merged);
                Ok(any.then_some(merged))
            }
        }
    }

    fn rewrite(&self, ctx: &Ctx, package: &str, entries: &mut [Value]) {
        let Some((vendor, project)) = package.split_once('/') else {
            return;
        };
        for entry in entries.iter_mut() {
            let version = entry
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if !metadata::valid_version(&version) {
                continue;
            }
            if let Some(dist) = entry.get_mut("dist").and_then(Value::as_object_mut) {
                dist.insert(
                    "url".to_owned(),
                    json!(format!(
                        "{}{}",
                        ctx.repository_url(),
                        metadata::dist_path(vendor, project, &version)
                    )),
                );
            }
        }
    }

    async fn dist(
        &self,
        ctx: &Ctx,
        vendor: &str,
        project: &str,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        let path = metadata::dist_path(vendor, project, version);
        match ctx.kind() {
            RepositoryType::Hosted => Ok(store::find(ctx.db(), ctx.name(), &path)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => {
                let package = format!("{vendor}/{project}");
                let found = proxy::fetch_from(
                    ctx,
                    &path,
                    Freshness::Content,
                    || async {
                        let dev = metadata::is_dev(version);
                        let Some(document) = self.proxied_metadata(ctx, &package, dev).await?
                        else {
                            return Ok(None);
                        };
                        let Some((url, _)) = metadata::dist_of(&document, &package, version) else {
                            return Ok(None);
                        };
                        url::Url::parse(&url)
                            .map(|url| Some(Origin::new(url)))
                            .map_err(|error| {
                                ContentError::BadGateway(format!(
                                    "the dist address is not valid: {error}"
                                ))
                            })
                    },
                    || {
                        PutAsset::cached(path.clone())
                            .kind(DIST)
                            .content_type(ZIP_TYPE)
                            .component(ComponentKey {
                                namespace: vendor.to_owned(),
                                name: project.to_owned(),
                                version: version.to_owned(),
                                kind: String::new(),
                                attributes: json!({}),
                            })
                    },
                )
                .await?;
                Ok(found.map(|found| (ctx.clone(), found)))
            }
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.dist(&member, vendor, project, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    async fn package_names(&self, ctx: &Ctx) -> Result<Vec<String>, ContentError> {
        let rows = component::Entity::find()
            .filter(component::Column::Repository.eq(ctx.name()))
            .filter(component::Column::Format.eq(ID))
            .order_by_asc(component::Column::Namespace)
            .order_by_asc(component::Column::Name)
            .all(ctx.db())
            .await?;
        let mut names: Vec<String> = Vec::new();
        for row in rows {
            let name = format!("{}/{}", row.namespace, row.name);
            if !names.contains(&name)
                && ctx.allowed(
                    View::Read,
                    &metadata::dist_path(&row.namespace, &row.name, &row.version),
                )
            {
                names.push(name);
            }
        }
        Ok(names)
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = route(path);
        if route == Route::Directory {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        let url = ctx.repository_url();
        match route {
            Route::Packages => {
                let mut document = json!({
                    "providers-url": format!("{url}/p/%package%.json"),
                    "metadata-url": format!("{url}/p2/%package%.json"),
                });
                match ctx.kind() {
                    RepositoryType::Hosted => {
                        document["available-packages"] = json!(self.package_names(ctx).await?);
                    }
                    RepositoryType::Proxy => {
                        document["list"] = json!(format!("{url}/packages/list.json"));
                    }
                    RepositoryType::Group => {}
                }
                Ok(json_response(request, &document))
            }
            Route::List => match ctx.kind() {
                RepositoryType::Hosted => Ok(json_response(
                    request,
                    &json!({"packageNames": self.package_names(ctx).await?}),
                )),
                RepositoryType::Proxy => {
                    let found = proxy::fetch(
                        ctx,
                        "/packages/list.json",
                        request.query.as_deref(),
                        Freshness::Metadata,
                        || {
                            PutAsset::cached("/packages/list.json")
                                .kind(METADATA)
                                .content_type(JSON_TYPE)
                        },
                    )
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                    serve(
                        ctx,
                        &found,
                        &request.method,
                        &request.headers,
                        Serve::default(),
                    )
                    .await
                }
                RepositoryType::Group => {
                    let mut names: Vec<String> = Vec::new();
                    for member in group::members(ctx).await? {
                        let member = ctx.member(member);
                        if member.kind() == RepositoryType::Hosted {
                            for name in self.package_names(&member).await? {
                                if !names.contains(&name) {
                                    names.push(name);
                                }
                            }
                        }
                    }
                    Ok(json_response(request, &json!({"packageNames": names})))
                }
            },
            Route::Metadata { package, dev } => {
                if ctx.kind() == RepositoryType::Hosted {
                    self.sync_if_stale(ctx, &package).await;
                }
                let mut entries = self
                    .entries(ctx, &package, dev)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                self.rewrite(ctx, &package, &mut entries);
                Ok(json_response(request, &p2_document(&package, entries)))
            }
            Route::Provider(package) => {
                if ctx.kind() == RepositoryType::Proxy {
                    return Err(ContentError::not_found(path.to_owned()));
                }
                let mut entries = self
                    .entries(ctx, &package, false)
                    .await?
                    .unwrap_or_default();
                entries.extend(self.entries(ctx, &package, true).await?.unwrap_or_default());
                if entries.is_empty() {
                    return Err(ContentError::not_found(path.to_owned()));
                }
                self.rewrite(ctx, &package, &mut entries);
                Ok(json_response(
                    request,
                    &provider_document(&package, &entries),
                ))
            }
            Route::Dist {
                vendor,
                project,
                version,
            } => {
                let (owner, found) = self
                    .dist(ctx, &vendor, &project, &version)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                serve(
                    &owner,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            _ => match ctx.kind() {
                RepositoryType::Hosted => {
                    let found = store::find(ctx.db(), ctx.name(), path)
                        .await?
                        .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                    serve(
                        ctx,
                        &found,
                        &request.method,
                        &request.headers,
                        Serve::default(),
                    )
                    .await
                }
                _ => Err(ContentError::not_found(path.to_owned())),
            },
        }
    }

    async fn sync_if_stale(&self, ctx: &Ctx, package: &str) {
        let trusted = ctx.member(ctx.repository.clone());
        let fresh = match self.hosted_entries(&trusted, package).await {
            Ok(entries) => fingerprint(&entries),
            Err(_) => return,
        };
        let stored = match store::find(ctx.db(), ctx.name(), &format!("/p2/{package}.json")).await {
            Ok(Some(found)) => read(ctx, &found, MAX_METADATA)
                .await
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .map(|document| fingerprint(&metadata::entries_of(&document, package))),
            _ => None,
        };
        let stale = match &stored {
            Some(stored) => *stored != fresh,
            None => !fresh.is_empty(),
        };
        if stale && let Err(error) = self.sync_stored(ctx, package).await {
            tracing::warn!(repository = ctx.name(), package, %error, "the stored Composer metadata was not refreshed");
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        if let Route::Dist {
            vendor, project, ..
        } = route(path)
            && ctx.kind() == RepositoryType::Hosted
        {
            self.sync_stored(ctx, &format!("{vendor}/{project}"))
                .await?;
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }
}

fn fingerprint(entries: &[Value]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = entries
        .iter()
        .map(|entry| {
            (
                entry
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                entry
                    .get("dist")
                    .and_then(|dist| dist.get("shasum"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect();
    pairs.sort();
    pairs
}

fn p2_document(package: &str, entries: Vec<Value>) -> Value {
    json!({"packages": {package: metadata::minify(&entries)}, "minified": MINIFIED})
}

fn provider_document(package: &str, entries: &[Value]) -> Value {
    let mut versions = Map::new();
    for entry in entries {
        if let Some(version) = entry.get("version").and_then(Value::as_str) {
            versions.insert(version.to_owned(), entry.clone());
        }
    }
    json!({"packages": {package: versions}})
}

#[async_trait]
impl Format for Composer {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, route(&request.path), ctx.kind()) {
            (&Method::GET | &Method::HEAD, _, _) => self.read(&ctx, &request).await,
            (
                &Method::PUT,
                Route::Upload {
                    vendor,
                    project,
                    version,
                },
                RepositoryType::Hosted,
            ) => {
                let path = metadata::dist_path(&vendor, &project, &version);
                if !ctx.allowed(View::Add, &path) && !ctx.allowed(View::Edit, &path) {
                    ctx.require(View::Add, &path)?;
                }
                let written = store::write_blob(&ctx, request.into_stream()).await?;
                self.store_dist(&ctx, written, &vendor, &project, &version)
                    .await?;
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::empty())
                    .map_err(ContentError::internal)
            }
            (&Method::DELETE, _, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (method, _, kind) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: match kind {
                    RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
                    RepositoryType::Proxy => "GET,HEAD,DELETE",
                    RepositoryType::Group => "GET,HEAD",
                },
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Dist {
                vendor,
                project,
                version,
            } => Ok(self
                .dist(ctx, &vendor, &project, &version)
                .await?
                .map(|(_, found)| found)),
            _ => match ctx.kind() {
                RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                    store::find(member.db(), member.name(), path).await
                })
                .await?
                .map(|(_, found)| found)),
                _ => store::find(ctx.db(), ctx.name(), path).await,
            },
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![],
            asset_fields: vec![
                UploadField::text("vendor", false),
                UploadField::text("project", false),
                UploadField::text("version", false),
                UploadField::file("asset"),
            ],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "only hosted repositories take uploads".to_owned(),
            ));
        }
        let field = |name: &str| {
            form.field(&format!("{ID}.asset.{name}"))
                .or_else(|| form.field(&format!("{ID}.{name}")))
                .map(|value| value.trim().to_owned())
        };
        let (Some(vendor), Some(project), Some(version)) =
            (field("vendor"), field("project"), field("version"))
        else {
            return Err(ContentError::BadRequest(
                "vendor, project and version are required".to_owned(),
            ));
        };
        if package_name(&vendor, &project).is_none() {
            return Err(ContentError::BadRequest(format!(
                "{vendor}/{project} is not a valid package name: use lower-case letters, digits, '-', '_' and '.'"
            )));
        }
        if !metadata::valid_version(&version) {
            return Err(ContentError::BadRequest(format!(
                "{version} is not a valid version"
            )));
        }
        let file = form
            .files
            .get(&format!("{ID}.asset"))
            .ok_or_else(|| ContentError::BadRequest("composer.asset is required".to_owned()))?;
        let path = metadata::dist_path(&vendor, &project, &version);
        if !ctx.allowed(View::Add, &path) && !ctx.allowed(View::Edit, &path) {
            ctx.require(View::Add, &path)?;
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        self.store_dist(&ctx, written, &vendor, &project, &version)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_metadata_dists_and_uploads() {
        assert_eq!(route("/packages.json"), Route::Packages);
        assert_eq!(route("/packages/list.json"), Route::List);
        assert_eq!(
            route("/p2/psr/log.json"),
            Route::Metadata {
                package: "psr/log".to_owned(),
                dev: false
            }
        );
        assert_eq!(
            route("/p2/psr/log~dev.json"),
            Route::Metadata {
                package: "psr/log".to_owned(),
                dev: true
            }
        );
        assert_eq!(
            route("/p/psr/log.json"),
            Route::Provider("psr/log".to_owned())
        );
        assert_eq!(
            route("/psr/log/3.0.2/psr-log-3.0.2.zip"),
            Route::Dist {
                vendor: "psr".to_owned(),
                project: "log".to_owned(),
                version: "3.0.2".to_owned()
            }
        );
        assert_eq!(
            route("/packages/upload/psr/log/3.0.2"),
            Route::Upload {
                vendor: "psr".to_owned(),
                project: "log".to_owned(),
                version: "3.0.2".to_owned()
            }
        );
        for other in [
            "/psr/log/3.0.2/other.zip",
            "/p2/PSR/log.json",
            "/p2/psr/log.txt",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }

    #[tokio::test]
    async fn the_manifest_is_found_at_the_top_or_in_one_folder() {
        let archive = zip::write(&[(
            "hello-1.0.0/composer.json",
            br#"{"name":"m4c/hello","require":{"php":">=8"}}"#,
        )]);
        let found = manifest(&zip::Memory(archive)).await.unwrap();
        assert_eq!(found["name"], "m4c/hello");
        let deep = zip::write(&[("a/b/composer.json", b"{}")]);
        assert!(manifest(&zip::Memory(deep)).await.is_err());
    }
}
