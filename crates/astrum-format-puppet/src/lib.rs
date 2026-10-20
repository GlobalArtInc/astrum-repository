pub mod api;
pub mod archive;
mod hosted;
pub mod names;
mod remote;

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Grant, Serve, UploadField, UploadForm,
    UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures_util::TryStreamExt;
use serde_json::{Value, json};

use crate::api::Params;
use crate::names::Module;

pub const ID: &str = "puppet";
pub const ARTIFACT: &str = "module";
pub const METADATA: &str = "metadata";
pub const GZIP_TYPE: &str = "application/gzip";
const MODULES: &str = "/v3/modules";
const RELEASES: &str = "/v3/releases";
const MAX_FIELD: u64 = 64 * 1024;
const MAX_JSON_UPLOAD: usize = 256 * 1024 * 1024;
const MAX_GROUP_LISTING: usize = 1000;

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "puppet.tag",
        target: SearchTarget::Component,
        pointer: &["puppet", "tags"],
    },
    SearchKey {
        param: "puppet.license",
        target: SearchTarget::Component,
        pointer: &["puppet", "license"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["puppet", "summary"], &["puppet", "tags"]];

#[derive(Default)]
pub struct Puppet {
    remote: remote::Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Modules,
    Module(Module),
    Releases,
    Release(Module, String),
    File(Module, String),
    Artifact(Module, String),
    Other,
}

impl Route {
    fn is_api(&self) -> bool {
        !matches!(self, Route::Root | Route::Artifact(..) | Route::Other)
    }
}

fn route(path: &str) -> Route {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    let directory = trimmed.ends_with('/');
    let parts: Vec<&str> = trimmed.trim_end_matches('/').split('/').collect();
    match parts.as_slice() {
        ["v3", "modules"] => Route::Modules,
        ["v3", "modules", slug] => Module::parse(slug)
            .map(Route::Module)
            .unwrap_or(Route::Other),
        ["v3", "modules", owner, name] => Module::new(owner, name)
            .map(Route::Module)
            .unwrap_or(Route::Other),
        ["v3", "releases"] => Route::Releases,
        ["v3", "releases", slug] => names::parse_release(slug)
            .map(|(module, version)| Route::Release(module, version))
            .unwrap_or(Route::Other),
        ["v3", "files", file] if !directory => names::parse_file_name(file)
            .map(|(module, version)| Route::File(module, version))
            .unwrap_or(Route::Other),
        [owner, name, version, file] if !directory && names::valid_version(version) => {
            match Module::new(owner, name) {
                Some(module) if module.owner == *owner && *file == module.file_name(version) => {
                    Route::Artifact(module, (*version).to_owned())
                }
                _ => Route::Other,
            }
        }
        _ => Route::Other,
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Puppet Forge repository of modules. Install from it with <code>puppet module install --module_repository {url} &lt;owner&gt;-&lt;name&gt;</code>, or set <code>{url}</code> as the Forge <code>baseurl</code> of r10k or Bolt; the API is at <code>{url}/v3/</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn denied(ctx: &Ctx) -> ContentError {
    match &ctx.subject {
        Some(subject) if !subject.anonymous => ContentError::Forbidden,
        _ => ContentError::Unauthorized,
    }
}

fn require_reader(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Read).any() {
        true => Ok(()),
        false => Err(denied(ctx)),
    }
}

fn log_member_failure(ctx: &Ctx, member: &Ctx, error: &ContentError) {
    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
}

fn visible_release(ctx: &Ctx, release: &Value) -> bool {
    api::release_slug_of(release)
        .is_some_and(|(module, version)| ctx.allowed(View::Read, &module.artifact_path(&version)))
}

fn visible_module(ctx: &Ctx, document: &Value) -> bool {
    match ctx.grant(View::Read) {
        Grant::Whole => true,
        Grant::Nothing => false,
        Grant::Selected(_) => document
            .get("current_release")
            .into_iter()
            .chain(
                document
                    .get("releases")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten(),
            )
            .any(|release| visible_release(ctx, release)),
    }
}

fn merge_module_lists(lists: Vec<Vec<Value>>) -> Vec<Value> {
    let mut by_slug: Vec<(String, Vec<Value>)> = Vec::new();
    for list in lists {
        for document in list {
            let slug = document
                .get("slug")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            match by_slug.iter_mut().find(|(known, _)| *known == slug) {
                Some((_, documents)) => documents.push(document),
                None => by_slug.push((slug, vec![document])),
            }
        }
    }
    by_slug
        .into_iter()
        .filter_map(|(_, documents)| api::merge_modules(documents))
        .collect()
}

impl Puppet {
    async fn module_of(&self, ctx: &Ctx, module: &Module) -> Result<Option<Value>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let releases: Vec<Value> = hosted::stored(ctx, Some(module), None)
                    .await?
                    .iter()
                    .map(hosted::Stored::document)
                    .collect();
                Ok(api::module_document(module, &releases))
            }
            RepositoryType::Proxy => self.remote.module(ctx, module).await,
            RepositoryType::Group => {
                let mut found = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.module_of(&member, module)).await {
                        Ok(Some(document)) => found.push(document),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(api::merge_modules(found))
            }
        }
    }

    async fn module_document(
        &self,
        ctx: &Ctx,
        module: &Module,
    ) -> Result<Option<Value>, ContentError> {
        let Some(mut document) = self.module_of(ctx, module).await? else {
            return Ok(None);
        };
        if matches!(ctx.grant(View::Read), Grant::Whole) {
            return Ok(Some(document));
        }
        let allowed = |version: &str| ctx.allowed(View::Read, &module.artifact_path(version));
        let mut releases: Vec<Value> = document
            .get("releases")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        releases.retain(|release| allowed(api::version_of(release)));
        let Some(highest) = names::highest(releases.iter().map(api::version_of)).map(str::to_owned)
        else {
            return Ok(None);
        };
        let current = match document.get("current_release") {
            Some(current) if api::version_of(current) == highest => current.clone(),
            _ => self
                .release_of(ctx, module, &highest)
                .await?
                .unwrap_or(Value::Null),
        };
        if let Some(object) = document.as_object_mut() {
            object.insert("releases".to_owned(), Value::Array(releases));
            object.insert("current_release".to_owned(), current);
        }
        Ok(Some(document))
    }

    async fn release_of(
        &self,
        ctx: &Ctx,
        module: &Module,
        version: &str,
    ) -> Result<Option<Value>, ContentError> {
        if !ctx.allowed(View::Read, &module.artifact_path(version)) {
            return Ok(None);
        }
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::stored(ctx, Some(module), None)
                .await?
                .into_iter()
                .find(|stored| stored.component.version == version)
                .map(|stored| stored.document())),
            RepositoryType::Proxy => self.remote.release(ctx, module, version).await,
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.release_of(&member, module, version)).await {
                        Ok(Some(document)) => return Ok(Some(document)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn releases_of(
        &self,
        ctx: &Ctx,
        module: &Module,
    ) -> Result<Option<Vec<Value>>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(Some(
                hosted::stored(ctx, Some(module), None)
                    .await?
                    .iter()
                    .map(hosted::Stored::document)
                    .collect(),
            )),
            RepositoryType::Proxy => self.remote.releases(ctx, module).await,
            RepositoryType::Group => {
                let mut lists = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.releases_of(&member, module)).await {
                        Ok(Some(releases)) => lists.push(releases),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(Some(api::merge_releases(lists)))
            }
        }
    }

    async fn gather(
        &self,
        ctx: &Ctx,
        base: &str,
        params: &Params,
        want: usize,
    ) -> Result<(Vec<Value>, usize), ContentError> {
        let modules = base == MODULES;
        let mut items: Vec<Value> = match ctx.kind() {
            RepositoryType::Hosted => {
                let stored = hosted::stored(ctx, None, params.get("owner")).await?;
                match modules {
                    true => hosted::modules(&stored)
                        .into_iter()
                        .filter(|document| api::module_matches(document, params))
                        .collect(),
                    false => stored
                        .iter()
                        .map(hosted::Stored::document)
                        .filter(|release| api::release_matches(release, params, None))
                        .collect(),
                }
            }
            RepositoryType::Proxy => return self.remote.gather(ctx, base, params, want).await,
            RepositoryType::Group => {
                let mut lists = Vec::new();
                let mut total = 0;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.gather(&member, base, params, want)).await {
                        Ok((items, count)) => {
                            total += count;
                            lists.push(items);
                        }
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                let before: usize = lists.iter().map(Vec::len).sum();
                let mut merged = match modules {
                    true => merge_module_lists(lists),
                    false => api::merge_releases(lists),
                };
                match modules {
                    true => {
                        api::sort_modules(&mut merged, params.get("sort_by"), params.get("query"))
                    }
                    false => api::sort_releases(&mut merged, params.get("sort_by")),
                }
                let total = total.saturating_sub(before - merged.len());
                return Ok((merged, total));
            }
        };
        match modules {
            true => api::sort_modules(&mut items, params.get("sort_by"), params.get("query")),
            false => api::sort_releases(&mut items, params.get("sort_by")),
        }
        let total = items.len();
        Ok((items, total))
    }

    async fn listing(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        base: &str,
        params: &Params,
    ) -> Result<Value, ContentError> {
        let visible = |item: &Value| match base {
            MODULES => visible_module(ctx, item),
            _ => visible_release(ctx, item),
        };
        if base == RELEASES
            && let Some(module) = params.module()?
        {
            let mut releases = self.releases_of(ctx, &module).await?.unwrap_or_default();
            releases.retain(|release| {
                visible(release) && api::release_matches(release, params, Some(&module))
            });
            api::sort_releases(&mut releases, params.get("sort_by"));
            let total = releases.len();
            return Ok(params.page(base, releases, total));
        }
        if ctx.kind() == RepositoryType::Proxy {
            let mut document = self
                .remote
                .listing(ctx, base, request.query.as_deref().unwrap_or_default())
                .await?;
            if !matches!(ctx.grant(View::Read), Grant::Whole)
                && let Some(Value::Array(results)) = document.get_mut("results")
            {
                results.retain(visible);
            }
            return Ok(document);
        }
        let want = (params.offset + params.limit).min(MAX_GROUP_LISTING);
        let (mut items, mut total) = self.gather(ctx, base, params, want).await?;
        if !matches!(ctx.grant(View::Read), Grant::Whole) {
            let before = items.len();
            items.retain(visible);
            total = total.saturating_sub(before - items.len());
        }
        Ok(params.page(base, items, total))
    }

    async fn api_read(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let params = Params::of(request.query.as_deref());
        let ok = |value: &Value| api::json(StatusCode::OK, Some(request), value);
        let missing = || ContentError::not_found(format!("{} cannot be found", request.path));
        match route {
            Route::Modules => Ok(ok(&self.listing(ctx, request, MODULES, &params).await?)),
            Route::Releases => Ok(ok(&self.listing(ctx, request, RELEASES, &params).await?)),
            Route::Module(module) => {
                let document = self
                    .module_document(ctx, &module)
                    .await?
                    .ok_or_else(missing)?;
                Ok(ok(&params.exclude(document)))
            }
            Route::Release(module, version) => {
                let document = self
                    .release_of(ctx, &module, &version)
                    .await?
                    .ok_or_else(missing)?;
                Ok(ok(&params.exclude(document)))
            }
            _ => Err(missing()),
        }
    }

    async fn artifact(
        &self,
        ctx: &Ctx,
        module: &Module,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                Ok(
                    store::find(ctx.db(), ctx.name(), &module.artifact_path(version))
                        .await?
                        .map(|found| (ctx.clone(), found)),
                )
            }
            RepositoryType::Proxy => Ok(self
                .remote
                .artifact(ctx, module, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.artifact(&member, module, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        module: &Module,
        version: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &module.artifact_path(version))?;
        let (owner, found) = self
            .artifact(ctx, module, version)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("{} cannot be found", request.path)))?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::download().disposition("attachment"),
        )
        .await
    }

    async fn received(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
    ) -> Result<astrum_blob::Written, ContentError> {
        let content_type = request
            .header("content-type")
            .unwrap_or_default()
            .to_owned();
        if content_type
            .to_ascii_lowercase()
            .starts_with("application/json")
        {
            let body = request.bytes(MAX_JSON_UPLOAD).await?;
            let document: Value = serde_json::from_slice(&body).map_err(|error| {
                ContentError::BadRequest(format!("Cannot parse request body as JSON: {error}"))
            })?;
            let encoded: String = document
                .get("file")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ContentError::BadRequest(
                        "the request body has no file with the base64-encoded module archive"
                            .to_owned(),
                    )
                })?
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            let archive = STANDARD.decode(encoded).map_err(|error| {
                ContentError::BadRequest(format!(
                    "the file of the request body is not base64: {error}"
                ))
            })?;
            return store::write_blob(ctx, astrum_blob::content_of(archive)).await;
        }
        let boundary = multer::parse_boundary(&content_type).map_err(|_| {
            ContentError::BadRequest(
                "publish a module as a multipart/form-data request with a file part, or as JSON with the base64-encoded archive in file".to_owned(),
            )
        })?;
        let limits = multer::Constraints::new().size_limit(
            multer::SizeLimit::new()
                .per_field(MAX_FIELD)
                .for_field("file", u64::MAX),
        );
        let mut multipart = multer::Multipart::with_constraints(
            request.take_body().into_data_stream(),
            boundary,
            limits,
        );
        let bad = |error: multer::Error| {
            ContentError::BadRequest(format!("the upload form cannot be read: {error}"))
        };
        let mut written = None;
        while let Some(mut part) = multipart.next_field().await.map_err(bad)? {
            match part.name() {
                Some("file") if written.is_none() => {
                    let body: astrum_blob::ContentStream =
                        Box::pin(part.map_err(std::io::Error::other));
                    written = Some(store::write_blob(ctx, body).await?);
                }
                _ => while part.chunk().await.map_err(bad)?.is_some() {},
            }
        }
        written.ok_or_else(|| ContentError::BadRequest("the upload has no file part".to_owned()))
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(format!(
                "{} is a {} repository; publish modules to a hosted repository",
                ctx.name(),
                ctx.kind()
            )));
        }
        if !hosted::may_write(ctx) {
            return Err(denied(ctx));
        }
        let written = self.received(ctx, request).await?;
        let (release, _) = hosted::store_release(ctx, written)
            .await
            .map_err(hosted::Refusal::into_error)?;
        Ok(api::json(
            StatusCode::CREATED,
            None,
            &json!({
                "uri": release.module.release_uri(&release.version),
                "slug": release.module.release_slug(&release.version),
                "file_uri": release.module.file_uri(&release.version),
            }),
        ))
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(format!("{path} cannot be found")));
        }
        no_content()
    }

    async fn delete_module(&self, ctx: &Ctx, module: &Module) -> Result<Response, ContentError> {
        let releases = hosted::stored(ctx, Some(module), None).await?;
        if releases.is_empty() {
            return Err(ContentError::not_found(format!(
                "{} cannot be found",
                module.uri()
            )));
        }
        for release in &releases {
            ctx.require(View::Delete, &release.asset.path)?;
        }
        for release in releases {
            store::delete_asset(&ctx.content, release.asset).await?;
        }
        no_content()
    }

    async fn read_other(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if request.is_directory() {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        let found = match ctx.kind() {
            RepositoryType::Group => {
                group::first(ctx, |member| async move {
                    store::find(member.db(), member.name(), path).await
                })
                .await?
            }
            _ => store::find(ctx.db(), ctx.name(), path)
                .await?
                .map(|found| (ctx.clone(), found)),
        };
        let (owner, found) = found.ok_or_else(|| ContentError::not_found(path.to_owned()))?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::default(),
        )
        .await
    }

    fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
        ContentError::MethodNotAllowed {
            method: method.to_string(),
            allow: match kind {
                RepositoryType::Hosted => "GET,HEAD,POST,DELETE",
                RepositoryType::Proxy => "GET,HEAD,DELETE",
                RepositoryType::Group => "GET,HEAD",
            },
        }
    }
}

fn no_content() -> Result<Response, ContentError> {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .map_err(ContentError::internal)
}

#[async_trait]
impl Format for Puppet {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let api_route = route.is_api();
        let method = request.method.clone();
        let kind = ctx.kind();
        let result = match (&method, route) {
            (&Method::GET | &Method::HEAD, Route::Root) => directory_page(&ctx),
            (
                &Method::GET | &Method::HEAD,
                Route::File(module, version) | Route::Artifact(module, version),
            ) => self.download(&ctx, &request, &module, &version).await,
            (&Method::GET | &Method::HEAD, Route::Other) => self.read_other(&ctx, &request).await,
            (&Method::GET | &Method::HEAD, route) => self.api_read(&ctx, &request, route).await,
            (&Method::POST, Route::Releases) => self.publish(&ctx, request).await,
            (&Method::DELETE, Route::Release(module, version))
                if kind == RepositoryType::Hosted =>
            {
                self.delete(&ctx, &module.artifact_path(&version)).await
            }
            (&Method::DELETE, Route::Module(module)) if kind == RepositoryType::Hosted => {
                self.delete_module(&ctx, &module).await
            }
            (&Method::DELETE, Route::File(module, version)) if kind != RepositoryType::Group => {
                self.delete(&ctx, &module.artifact_path(&version)).await
            }
            (&Method::DELETE, Route::Artifact(..) | Route::Other)
                if kind != RepositoryType::Group =>
            {
                self.delete(&ctx, &request.path).await
            }
            (method, _) => Err(Self::not_allowed(method, kind)),
        };
        match result {
            Ok(response) => Ok(response),
            Err(error) if api_route => Ok(api::error(error)),
            Err(error) => Ok(error.into_response()),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Artifact(module, version) | Route::File(module, version) => Ok(self
                .artifact(ctx, &module, &version)
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
            asset_fields: vec![UploadField::file("asset")],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "only hosted repositories take uploads".to_owned(),
            ));
        }
        let file = form
            .files
            .get(&format!("{ID}.asset"))
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest(format!("{ID}.asset is required")))?;
        if !hosted::may_write(&ctx) {
            return Err(denied(&ctx));
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::store_release(&ctx, written)
            .await
            .map(|_| ())
            .map_err(hosted::Refusal::into_error)
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        KEYWORDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(slug: &str) -> Module {
        Module::parse(slug).unwrap()
    }

    #[test]
    fn paths_route_to_the_v3_api_and_archives() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/v3/modules"), Route::Modules);
        assert_eq!(route("/v3/modules/"), Route::Modules);
        assert_eq!(
            route("/v3/modules/puppetlabs-stdlib"),
            Route::Module(module("puppetlabs-stdlib"))
        );
        assert_eq!(
            route("/v3/modules/PuppetLabs/stdlib"),
            Route::Module(module("puppetlabs-stdlib"))
        );
        assert_eq!(route("/v3/releases"), Route::Releases);
        assert_eq!(
            route("/v3/releases/puppetlabs-stdlib-9.0.0-rc.1"),
            Route::Release(module("puppetlabs-stdlib"), "9.0.0-rc.1".to_owned())
        );
        assert_eq!(
            route("/v3/files/puppetlabs-stdlib-9.0.0.tar.gz"),
            Route::File(module("puppetlabs-stdlib"), "9.0.0".to_owned())
        );
        assert_eq!(
            route("/puppetlabs/stdlib/9.0.0/puppetlabs-stdlib-9.0.0.tar.gz"),
            Route::Artifact(module("puppetlabs-stdlib"), "9.0.0".to_owned())
        );
        for other in [
            "/v3/modules/puppetlabs-stdlib.json",
            "/v3/modules/puppetlabs-stdlib/releases.json",
            "/v3/releases/puppetlabs-stdlib-9.0.0.json",
            "/v3/releases/puppetlabs-stdlib",
            "/v3/files/puppetlabs-stdlib-9.0.tar.gz",
            "/v3/files/stdlib.tar.gz",
            "/PuppetLabs/stdlib/9.0.0/puppetlabs-stdlib-9.0.0.tar.gz",
            "/puppetlabs/stdlib/9.0.0/other-9.0.0.tar.gz",
            "/v3/users/puppetlabs",
            "/v2/modules",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        assert!(route("/v3/files/a-b-1.0.0.tar.gz").is_api());
        assert!(!route("/a/b/1.0.0/a-b-1.0.0.tar.gz").is_api());
    }

    #[test]
    fn group_listings_merge_modules_of_the_same_slug() {
        let first = json!({"slug": "acme-tools", "downloads": 1, "releases": [{"version": "1.0.0", "slug": "acme-tools-1.0.0"}], "current_release": {"version": "1.0.0"}});
        let second = json!({"slug": "acme-tools", "downloads": 2, "releases": [{"version": "2.0.0", "slug": "acme-tools-2.0.0"}], "current_release": {"version": "2.0.0"}});
        let other =
            json!({"slug": "acme-other", "downloads": 5, "releases": [], "current_release": null});
        let merged = merge_module_lists(vec![vec![first], vec![second, other]]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0]["current_release"]["version"], "2.0.0");
        assert_eq!(merged[0]["downloads"], 3);
        assert_eq!(merged[1]["slug"], "acme-other");
    }
}
