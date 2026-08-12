pub mod reference;
mod routes;

pub use routes::routes;

use std::collections::BTreeMap;

use astrum_content::format::SearchTarget;
use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Freshness, SearchKey, Serve,
    View, group, serve,
};
use astrum_db::entity::component;
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Map, Value, json};

use crate::reference::{Recipe, Route};

pub const ID: &str = "conan";
pub const REALM: &str = "ConanToken";
pub const FILE: &str = "file";
pub const METADATA: &str = "metadata";
pub(crate) const TOKEN_DAYS: i64 = 30;
const MAX_LISTING: u64 = 100_000;
const MAX_INFO: u64 = 1024 * 1024;
const MAX_METADATA: u64 = 32 * 1024 * 1024;

#[derive(Default)]
pub struct Conan;

fn text(status: StatusCode, body: impl Into<String>) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(body.into()))
        .expect("a text response is valid")
}

fn json_response(request: &ContentRequest, value: &Value) -> Response {
    let body = value.to_string();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .expect("a JSON response is valid")
}

fn time(at: DateTime<FixedOffset>) -> String {
    at.with_timezone(&Utc)
        .format("%Y-%m-%dT%H:%M:%S%.3f+0000")
        .to_string()
}

fn content_type(file: &str) -> &'static str {
    if file.ends_with(".tgz") || file.ends_with(".gz") {
        "application/gzip"
    } else if file.ends_with(".py") {
        "text/x-python"
    } else if file.ends_with(".txt") {
        "text/plain"
    } else {
        "application/octet-stream"
    }
}

fn anonymous(ctx: &Ctx) -> bool {
    ctx.subject.as_ref().is_none_or(|subject| subject.anonymous)
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

fn conan_version(ctx: &Ctx) -> &'static str {
    match ctx
        .repository
        .attributes
        .conan_proxy
        .as_ref()
        .and_then(|proxy| proxy.conan_version)
    {
        Some(astrum_repo::config::ConanVersion::V1) => "V1",
        _ => "V2",
    }
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

pub fn parse_info(text: &str) -> Value {
    let mut sections: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(section) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            current = section.to_owned();
            sections.entry(current.clone()).or_default();
            continue;
        }
        sections
            .entry(current.clone())
            .or_default()
            .push(line.to_owned());
    }
    let pairs = |name: &str| -> Value {
        let mut map = Map::new();
        for line in sections.get(name).into_iter().flatten() {
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_owned(), json!(value.trim()));
            }
        }
        Value::Object(map)
    };
    let requires: Vec<Value> = sections
        .get("requires")
        .into_iter()
        .flatten()
        .map(|line| json!(line))
        .collect();
    json!({
        "settings": pairs("settings"),
        "options": pairs("options"),
        "requires": requires,
        "content": text,
    })
}

#[derive(Default)]
struct Revision {
    first: Option<DateTime<FixedOffset>>,
    files: Vec<String>,
}

fn revisions_document(
    display: String,
    mut revisions: Vec<(String, DateTime<FixedOffset>)>,
) -> Value {
    revisions.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    json!({
        "reference": display,
        "revisions": revisions
            .iter()
            .map(|(revision, at)| json!({"revision": revision, "time": time(*at)}))
            .collect::<Vec<_>>(),
    })
}

fn files_document(files: &[String]) -> Value {
    let map: Map<String, Value> = files.iter().map(|file| (file.clone(), json!({}))).collect();
    json!({ "files": map })
}

fn parse_time(text: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.3f%z")
        .or_else(|_| DateTime::parse_from_rfc3339(text))
        .ok()
}

impl Conan {
    async fn stored_under(&self, ctx: &Ctx, prefix: &str) -> Result<Vec<Asset>, ContentError> {
        Ok(store::under(ctx.db(), ctx.name(), prefix, MAX_LISTING)
            .await?
            .into_iter()
            .filter(|found| found.kind == FILE && ctx.allowed(View::Read, &found.path))
            .collect())
    }

    async fn hosted_recipe_revisions(
        &self,
        ctx: &Ctx,
        recipe: &Recipe,
    ) -> Result<BTreeMap<String, Revision>, ContentError> {
        let mut revisions: BTreeMap<String, Revision> = BTreeMap::new();
        for found in self
            .stored_under(ctx, &format!("{}/", recipe.base()))
            .await?
        {
            let Some(stored) = reference::parse_stored(&found.path) else {
                continue;
            };
            if stored.recipe != *recipe || stored.package.is_some() {
                continue;
            }
            let entry = revisions.entry(stored.revision).or_default();
            entry.first = Some(
                entry
                    .first
                    .map_or(found.created_at, |first| first.min(found.created_at)),
            );
            entry.files.push(stored.file);
        }
        Ok(revisions)
    }

    async fn hosted_package_revisions(
        &self,
        ctx: &Ctx,
        recipe: &Recipe,
        revision: &str,
        package: Option<&str>,
    ) -> Result<BTreeMap<(String, String), Revision>, ContentError> {
        let prefix = match package {
            Some(package) => format!("{}/{revision}/package/{package}/", recipe.base()),
            None => format!("{}/{revision}/package/", recipe.base()),
        };
        let mut revisions: BTreeMap<(String, String), Revision> = BTreeMap::new();
        for found in self.stored_under(ctx, &prefix).await? {
            let Some(stored) = reference::parse_stored(&found.path) else {
                continue;
            };
            let Some((package_id, package_revision)) = stored.package else {
                continue;
            };
            let entry = revisions.entry((package_id, package_revision)).or_default();
            entry.first = Some(
                entry
                    .first
                    .map_or(found.created_at, |first| first.min(found.created_at)),
            );
            entry.files.push(stored.file);
        }
        Ok(revisions)
    }

    async fn hosted_document(
        &self,
        ctx: &Ctx,
        route: &Route,
    ) -> Result<Option<Value>, ContentError> {
        match route {
            Route::Revisions(recipe) => {
                let revisions = self.hosted_recipe_revisions(ctx, recipe).await?;
                if revisions.is_empty() {
                    return Ok(None);
                }
                Ok(Some(revisions_document(
                    recipe.display(),
                    revisions
                        .into_iter()
                        .filter_map(|(revision, entry)| Some((revision, entry.first?)))
                        .collect(),
                )))
            }
            Route::Latest(recipe) => Ok(self
                .hosted_recipe_revisions(ctx, recipe)
                .await?
                .into_iter()
                .filter_map(|(revision, entry)| Some((revision, entry.first?)))
                .max_by(|left, right| left.1.cmp(&right.1))
                .map(|(revision, at)| json!({"revision": revision, "time": time(at)}))),
            Route::RecipeFiles(recipe, revision) => Ok(self
                .hosted_recipe_revisions(ctx, recipe)
                .await?
                .remove(revision)
                .map(|entry| files_document(&entry.files))),
            Route::PackageRevisions(recipe, revision, package) => {
                let revisions = self
                    .hosted_package_revisions(ctx, recipe, revision, Some(package))
                    .await?;
                if revisions.is_empty() {
                    return Ok(None);
                }
                Ok(Some(revisions_document(
                    format!("{}#{revision}:{package}", recipe.display()),
                    revisions
                        .into_iter()
                        .filter_map(|((_, package_revision), entry)| Some((package_revision, entry.first?)))
                        .collect(),
                )))
            }
            Route::PackageLatest(recipe, revision, package) => Ok(self
                .hosted_package_revisions(ctx, recipe, revision, Some(package))
                .await?
                .into_iter()
                .filter_map(|((_, package_revision), entry)| Some((package_revision, entry.first?)))
                .max_by(|left, right| left.1.cmp(&right.1))
                .map(|(package_revision, at)| json!({"revision": package_revision, "time": time(at)}))),
            Route::PackageFiles(recipe, revision, package, package_revision) => Ok(self
                .hosted_package_revisions(ctx, recipe, revision, Some(package))
                .await?
                .remove(&(package.clone(), package_revision.clone()))
                .map(|entry| files_document(&entry.files))),
            Route::PackageSearch(recipe, revision) => {
                let revision = match revision {
                    Some(revision) => revision.clone(),
                    None => match Box::pin(self.hosted_document(ctx, &Route::Latest(recipe.clone()))).await? {
                        Some(latest) => latest["revision"].as_str().unwrap_or_default().to_owned(),
                        None => return Ok(None),
                    },
                };
                if !self.hosted_recipe_revisions(ctx, recipe).await?.contains_key(&revision) {
                    return Ok(None);
                }
                let mut newest: BTreeMap<String, (DateTime<FixedOffset>, String)> = BTreeMap::new();
                for ((package, package_revision), entry) in
                    self.hosted_package_revisions(ctx, recipe, &revision, None).await?
                {
                    let Some(first) = entry.first else {
                        continue;
                    };
                    if !entry.files.iter().any(|file| file == "conaninfo.txt") {
                        continue;
                    }
                    if newest.get(&package).is_none_or(|(known, _)| *known < first) {
                        newest.insert(package, (first, package_revision));
                    }
                }
                let mut packages = Map::new();
                for (package, (_, package_revision)) in newest {
                    let path = recipe.package_path(&revision, &package, &package_revision, "conaninfo.txt");
                    if let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? {
                        let info = read(ctx, &found, MAX_INFO).await?;
                        packages.insert(package, parse_info(&String::from_utf8_lossy(&info)));
                    }
                }
                Ok(Some(Value::Object(packages)))
            }
            _ => Ok(None),
        }
    }

    async fn hosted_search(&self, ctx: &Ctx, pattern: &str) -> Result<Vec<String>, ContentError> {
        let rows = component::Entity::find()
            .filter(component::Column::Repository.eq(ctx.name()))
            .filter(component::Column::Format.eq(ID))
            .all(ctx.db())
            .await?;
        let mut results: Vec<String> = Vec::new();
        for row in rows {
            let (user, channel) = row.namespace.split_once('/').unwrap_or(("_", "_"));
            let recipe = Recipe {
                name: row.name.clone(),
                version: row.version.clone(),
                user: user.to_owned(),
                channel: channel.to_owned(),
            };
            let display = recipe.display();
            let short = format!("{}/{}", recipe.name, recipe.version);
            if (reference::glob(pattern, &display) || reference::glob(pattern, &short))
                && ctx.allowed(View::Read, &format!("{}/", recipe.base()))
                && !results.contains(&display)
            {
                results.push(display);
            }
        }
        results.sort();
        Ok(results)
    }

    async fn proxied_document(
        &self,
        ctx: &Ctx,
        request_path: &str,
    ) -> Result<Option<Value>, ContentError> {
        let found = proxy::fetch(ctx, request_path, None, Freshness::Metadata, || {
            PutAsset::cached(request_path.to_owned())
                .kind(METADATA)
                .content_type("application/json")
        })
        .await?;
        match found {
            Some(found) => Ok(serde_json::from_slice(&read(ctx, &found, MAX_METADATA).await?).ok()),
            None => Ok(None),
        }
    }

    async fn document(
        &self,
        ctx: &Ctx,
        route: &Route,
        request_path: &str,
    ) -> Result<Option<Value>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => self.hosted_document(ctx, route).await,
            RepositoryType::Proxy => {
                if conan_version(ctx) == "V1" {
                    return Ok(None);
                }
                self.proxied_document(ctx, request_path).await
            }
            RepositoryType::Group => {
                let mut documents = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.document(&member, route, request_path)).await {
                        Ok(Some(document)) => documents.push(document),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                Ok(merge(route, documents))
            }
        }
    }

    async fn file(
        &self,
        ctx: &Ctx,
        stored_path: &str,
        request_path: &str,
        recipe: &Recipe,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(store::find(ctx.db(), ctx.name(), stored_path)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => {
                if conan_version(ctx) == "V1" {
                    return Ok(None);
                }
                let file = stored_path
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .to_owned();
                let found = proxy::fetch_from(
                    ctx,
                    stored_path,
                    Freshness::Content,
                    || async {
                        proxy::remote_url(&ctx.repository, request_path, None)
                            .map(|url| Some(Origin::new(url)))
                    },
                    || {
                        PutAsset::cached(stored_path.to_owned())
                            .kind(FILE)
                            .content_type(content_type(&file))
                            .attributes(attributes(stored_path))
                            .component(component(recipe))
                    },
                )
                .await?;
                Ok(found.map(|found| (ctx.clone(), found)))
            }
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.file(&member, stored_path, request_path, recipe)).await {
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

    async fn search(
        &self,
        ctx: &Ctx,
        pattern: &str,
        request: &ContentRequest,
    ) -> Result<Vec<String>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => self.hosted_search(ctx, pattern).await,
            RepositoryType::Proxy => {
                if conan_version(ctx) == "V1" {
                    return Ok(Vec::new());
                }
                let url = proxy::remote_url(
                    &ctx.repository,
                    "/v2/conans/search",
                    request.query.as_deref(),
                )?;
                let answer = proxy::request(ctx, url, &[]).await?;
                if answer.status != StatusCode::OK {
                    return Ok(Vec::new());
                }
                let body = answer.bytes(MAX_METADATA as usize).await?;
                Ok(serde_json::from_slice::<Value>(&body)
                    .ok()
                    .and_then(|document| document.get("results").cloned())
                    .and_then(|results| serde_json::from_value::<Vec<String>>(results).ok())
                    .unwrap_or_default())
            }
            RepositoryType::Group => {
                let mut results: Vec<String> = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.search(&member, pattern, request)).await {
                        Ok(found) => {
                            for result in found {
                                if !results.contains(&result) {
                                    results.push(result);
                                }
                            }
                        }
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                results.sort();
                Ok(results)
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = reference::route(path);
        match &route {
            Route::Ping => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("X-Conan-Server-Capabilities", "revisions")
                    .body(Body::empty())
                    .map_err(ContentError::internal);
            }
            Route::Authenticate => {
                return match anonymous(ctx) {
                    true => Err(ContentError::Unauthorized),
                    false => Err(ContentError::not_found(path.to_owned())),
                };
            }
            Route::CheckCredentials => {
                return match ctx.subject.as_ref().filter(|subject| !subject.anonymous) {
                    Some(subject) => Ok(text(StatusCode::OK, subject.user_id.clone())),
                    None => Err(ContentError::Unauthorized),
                };
            }
            Route::Directory => return directory_page(ctx),
            Route::Other => return Err(ContentError::not_found(path.to_owned())),
            _ => {}
        }
        match &route {
            Route::Search => {
                ctx.require(View::Read, "/")?;
                let pattern =
                    url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
                        .find(|(key, _)| key == "q")
                        .map(|(_, value)| value.into_owned())
                        .unwrap_or_else(|| "*".to_owned());
                let results = self.search(ctx, &pattern, request).await?;
                Ok(json_response(request, &json!({ "results": results })))
            }
            Route::RecipeFile(recipe, revision, file) => {
                let stored = recipe.export_path(revision, file);
                ctx.require(View::Read, &stored)?;
                self.serve_file(ctx, request, &stored, recipe).await
            }
            Route::PackageFile(recipe, revision, package, package_revision, file) => {
                let stored = recipe.package_path(revision, package, package_revision, file);
                ctx.require(View::Read, &stored)?;
                self.serve_file(ctx, request, &stored, recipe).await
            }
            Route::Revisions(recipe)
            | Route::Latest(recipe)
            | Route::RecipeFiles(recipe, _)
            | Route::PackageSearch(recipe, _)
            | Route::PackageRevisions(recipe, _, _)
            | Route::PackageLatest(recipe, _, _)
            | Route::PackageFiles(recipe, _, _, _) => {
                ctx.require(View::Read, &format!("{}/", recipe.base()))?;
                match self.document(ctx, &route, path).await? {
                    Some(document) => Ok(json_response(request, &document)),
                    None => Err(ContentError::not_found(format!(
                        "{} not found",
                        recipe.display()
                    ))),
                }
            }
            _ => Err(ContentError::not_found(path.to_owned())),
        }
    }

    async fn serve_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        stored: &str,
        recipe: &Recipe,
    ) -> Result<Response, ContentError> {
        let (owner, found) = self
            .file(ctx, stored, &request.path, recipe)
            .await?
            .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
        let counted =
            found.path.ends_with("conan_package.tgz") || found.path.ends_with("conan_export.tgz");
        let options = Serve {
            count_download: counted,
            content_type: Some(content_type(&found.path).to_owned()),
            ..Serve::default()
        };
        serve(&owner, &found, &request.method, &request.headers, options).await
    }

    async fn upload(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        recipe: Recipe,
        stored: String,
    ) -> Result<Response, ContentError> {
        if request
            .header("x-checksum-deploy")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
        {
            return Ok(text(
                StatusCode::NOT_FOUND,
                "checksum deploy is not supported; send the file",
            ));
        }
        let exists = store::find(ctx.db(), ctx.name(), &stored).await?.is_some();
        if exists && write_policy(ctx) == WritePolicy::Allow {
            ctx.require(View::Edit, &stored)?;
        } else {
            ctx.require(View::Add, &stored)?;
        }
        let file = stored.rsplit('/').next().unwrap_or_default().to_owned();
        let mut put = PutAsset::hosted(ctx, stored.clone())
            .kind(FILE)
            .content_type(content_type(&file))
            .attributes(attributes(&stored))
            .component(component(&recipe));
        put.check_content_type = false;
        store::put(ctx, put, request.into_stream()).await?;
        Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    async fn remove(&self, ctx: &Ctx, prefix: &str) -> Result<Response, ContentError> {
        let found = store::under(ctx.db(), ctx.name(), prefix, MAX_LISTING).await?;
        if found.is_empty() {
            return Err(ContentError::not_found(prefix.to_owned()));
        }
        for asset in &found {
            ctx.require(View::Delete, &asset.path)?;
        }
        for asset in found {
            store::delete_asset(&ctx.content, asset).await?;
        }
        Ok(text(StatusCode::OK, ""))
    }
}

pub const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "conan.baseVersion",
        target: SearchTarget::Asset,
        pointer: &["conan", "baseVersion"],
    },
    SearchKey {
        param: "conan.channel",
        target: SearchTarget::Asset,
        pointer: &["conan", "channel"],
    },
    SearchKey {
        param: "conan.revision",
        target: SearchTarget::Asset,
        pointer: &["conan", "revision"],
    },
    SearchKey {
        param: "conan.packageId",
        target: SearchTarget::Asset,
        pointer: &["conan", "packageId"],
    },
    SearchKey {
        param: "conan.packageRevision",
        target: SearchTarget::Asset,
        pointer: &["conan", "packageRevision"],
    },
];

fn attributes(stored: &str) -> Value {
    let Some(parsed) = reference::parse_stored(stored) else {
        return json!({});
    };
    let mut conan = Map::new();
    conan.insert("baseVersion".to_owned(), json!(parsed.recipe.version));
    conan.insert("channel".to_owned(), json!(parsed.recipe.channel));
    conan.insert("revision".to_owned(), json!(parsed.revision));
    if let Some((package, package_revision)) = parsed.package {
        conan.insert("packageId".to_owned(), json!(package));
        conan.insert("packageRevision".to_owned(), json!(package_revision));
    }
    json!({ ID: conan })
}

fn component(recipe: &Recipe) -> ComponentKey {
    ComponentKey {
        namespace: recipe.namespace(),
        name: recipe.name.clone(),
        version: recipe.version.clone(),
        kind: String::new(),
        attributes: json!({}),
    }
}

fn merge(route: &Route, documents: Vec<Value>) -> Option<Value> {
    if documents.is_empty() {
        return None;
    }
    match route {
        Route::Revisions(_) | Route::PackageRevisions(..) => {
            let reference = documents[0]
                .get("reference")
                .cloned()
                .unwrap_or(Value::Null);
            let mut seen: BTreeMap<String, (DateTime<FixedOffset>, Value)> = BTreeMap::new();
            for document in &documents {
                for entry in document
                    .get("revisions")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    let Some(revision) = entry.get("revision").and_then(Value::as_str) else {
                        continue;
                    };
                    let at = entry
                        .get("time")
                        .and_then(Value::as_str)
                        .and_then(parse_time)
                        .unwrap_or_else(|| Utc::now().fixed_offset());
                    seen.entry(revision.to_owned())
                        .or_insert((at, entry.clone()));
                }
            }
            let mut entries: Vec<(DateTime<FixedOffset>, Value)> = seen.into_values().collect();
            entries.sort_by_key(|entry| std::cmp::Reverse(entry.0));
            Some(json!({
                "reference": reference,
                "revisions": entries.into_iter().map(|(_, entry)| entry).collect::<Vec<_>>(),
            }))
        }
        Route::Latest(_) | Route::PackageLatest(..) => {
            documents.into_iter().max_by_key(|document| {
                document
                    .get("time")
                    .and_then(Value::as_str)
                    .and_then(parse_time)
            })
        }
        _ => documents.into_iter().next(),
    }
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Conan 2 repository. Add it with <code>conan remote add {name} {url}/</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

#[async_trait]
impl Format for Conan {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = reference::route(&request.path);
        match (&request.method, route, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _, _) => self.read(&ctx, &request).await,
            (&Method::PUT, Route::RecipeFile(recipe, revision, file), RepositoryType::Hosted) => {
                let stored = recipe.export_path(&revision, &file);
                self.upload(&ctx, request, recipe, stored).await
            }
            (
                &Method::PUT,
                Route::PackageFile(recipe, revision, package, package_revision, file),
                RepositoryType::Hosted,
            ) => {
                let stored = recipe.package_path(&revision, &package, &package_revision, &file);
                self.upload(&ctx, request, recipe, stored).await
            }
            (&Method::DELETE, Route::RecipeRevision(recipe, revision), RepositoryType::Hosted) => {
                self.remove(&ctx, &format!("{}/{revision}/", recipe.base()))
                    .await
            }
            (&Method::DELETE, Route::Packages(recipe, revision), RepositoryType::Hosted) => {
                self.remove(&ctx, &format!("{}/{revision}/package/", recipe.base()))
                    .await
            }
            (
                &Method::DELETE,
                Route::PackageRevision(recipe, revision, package, package_revision),
                RepositoryType::Hosted,
            ) => {
                self.remove(
                    &ctx,
                    &format!(
                        "{}/{revision}/package/{package}/{package_revision}/",
                        recipe.base()
                    ),
                )
                .await
            }
            (
                &Method::DELETE,
                Route::Other | Route::Directory,
                RepositoryType::Hosted | RepositoryType::Proxy,
            ) => {
                ctx.require(View::Delete, &request.path)?;
                if anonymous(&ctx) {
                    return Err(ContentError::Unauthorized);
                }
                match store::delete(&ctx, &request.path).await? {
                    true => Ok(text(StatusCode::NO_CONTENT, "")),
                    false => Err(ContentError::not_found(request.path.clone())),
                }
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
        match ctx.kind() {
            RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                store::find(member.db(), member.name(), path).await
            })
            .await?
            .map(|(_, found)| found)),
            _ => store::find(ctx.db(), ctx.name(), path).await,
        }
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some("replace(a.path, '/_/_/', '/')"),
            generated: &[],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_information_is_read_from_conaninfo() {
        let info = parse_info(
            "[settings]\narch=x86_64\nos=Linux\n[options]\nshared=False\n[requires]\nzlib/1.3.Z\n",
        );
        assert_eq!(info["settings"], json!({"arch": "x86_64", "os": "Linux"}));
        assert_eq!(info["options"], json!({"shared": "False"}));
        assert_eq!(info["requires"], json!(["zlib/1.3.Z"]));
        assert!(info["content"].as_str().unwrap().starts_with("[settings]"));
    }

    #[test]
    fn group_revisions_merge_newest_first() {
        let route = Route::Revisions(Recipe {
            name: "a".to_owned(),
            version: "1".to_owned(),
            user: "_".to_owned(),
            channel: "_".to_owned(),
        });
        let merged = merge(
            &route,
            vec![
                json!({"reference": "a/1@_/_", "revisions": [{"revision": "r1", "time": "2026-01-01T00:00:00.000+0000"}]}),
                json!({"reference": "a/1@_/_", "revisions": [{"revision": "r2", "time": "2026-02-01T00:00:00.000+0000"}, {"revision": "r1", "time": "2026-01-01T00:00:00.000+0000"}]}),
            ],
        )
        .unwrap();
        let order: Vec<&str> = merged["revisions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["revision"].as_str().unwrap())
            .collect();
        assert_eq!(order, ["r2", "r1"]);
        assert_eq!(content_type("conan_package.tgz"), "application/gzip");
    }
}
