pub mod api;
pub mod archive;
mod hosted;
mod imports;
pub mod names;
mod remote;
mod transfer;

use std::collections::BTreeMap;

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
use futures_util::TryStreamExt;
use serde_json::{Value, json};

use crate::api::{Entry, Page, Summary, Urls};
use crate::hosted::Refusal;
use crate::names::Coll;

pub const ID: &str = "ansible";
pub const ARTIFACT: &str = "collection";
pub const METADATA: &str = "metadata";
pub const GZIP_TYPE: &str = "application/gzip";
const MAX_FIELD: u64 = 64 * 1024;

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "ansible.tag",
        target: SearchTarget::Component,
        pointer: &["ansible", "tags"],
    },
    SearchKey {
        param: "ansible.license",
        target: SearchTarget::Component,
        pointer: &["ansible", "license"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["ansible", "description"], &["ansible", "tags"]];

#[derive(Default)]
pub struct Ansible {
    remote: remote::Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Api,
    Collections,
    Collection(Coll),
    Versions(Coll),
    Version(Coll, String),
    Publish,
    Import(String),
    Artifact(Coll, String),
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
    let coll = |namespace: &str, name: &str| Coll::new(namespace, name);
    match parts.as_slice() {
        ["api"] => Route::Api,
        ["api", "v3", "collections"] => Route::Collections,
        ["api", "v3", "collections", namespace, name] => coll(namespace, name)
            .map(Route::Collection)
            .unwrap_or(Route::Other),
        ["api", "v3", "collections", namespace, name, "versions"] => coll(namespace, name)
            .map(Route::Versions)
            .unwrap_or(Route::Other),
        [
            "api",
            "v3",
            "collections",
            namespace,
            name,
            "versions",
            version,
        ] if names::valid_version(version) => coll(namespace, name)
            .map(|coll| Route::Version(coll, (*version).to_owned()))
            .unwrap_or(Route::Other),
        ["api", "v3", "artifacts", "collections"] => Route::Publish,
        ["api", "v3", "imports", "collections", id] => Route::Import((*id).to_owned()),
        [namespace, name, version, file] if !directory && names::valid_version(version) => {
            match coll(namespace, name) {
                Some(coll) if *file == coll.file_name(version) => {
                    Route::Artifact(coll, (*version).to_owned())
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is an Ansible Galaxy repository of collections. Install from it with <code>ansible-galaxy collection install -s {url}/ &lt;namespace&gt;.&lt;name&gt;</code>; the API is at <code>{url}/api/</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
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

fn file_name_of(name: Option<&str>) -> Option<&str> {
    name.and_then(|name| name.rsplit(['/', '\\']).next())
        .filter(|name| !name.is_empty())
}

fn log_member_failure(ctx: &Ctx, member: &Ctx, error: &ContentError) {
    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
}

impl Ansible {
    async fn summaries(&self, ctx: &Ctx) -> Result<Vec<Summary>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::summaries(&hosted::stored(ctx, None).await?)),
            RepositoryType::Proxy => self.remote.cached_collections(ctx).await,
            RepositoryType::Group => {
                let mut merged: BTreeMap<Coll, Summary> = BTreeMap::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.summaries(&member)).await {
                        Ok(summaries) => {
                            for summary in summaries {
                                match merged.get_mut(&summary.coll) {
                                    Some(known) => known.merge(&summary),
                                    None => {
                                        merged.insert(summary.coll.clone(), summary);
                                    }
                                }
                            }
                        }
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(merged.into_values().collect())
            }
        }
    }

    async fn summary(&self, ctx: &Ctx, coll: &Coll) -> Result<Option<Summary>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                Ok(hosted::summaries(&hosted::stored(ctx, Some(coll)).await?)
                    .into_iter()
                    .next())
            }
            RepositoryType::Proxy => self.remote.collection(ctx, coll).await,
            RepositoryType::Group => {
                let mut merged: Option<Summary> = None;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.summary(&member, coll)).await {
                        Ok(Some(summary)) => match &mut merged {
                            Some(known) => known.merge(&summary),
                            None => merged = Some(summary),
                        },
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(merged)
            }
        }
    }

    async fn entries(&self, ctx: &Ctx, coll: &Coll) -> Result<Option<Vec<Entry>>, ContentError> {
        let mut entries = match ctx.kind() {
            RepositoryType::Hosted => hosted::stored(ctx, Some(coll))
                .await?
                .iter()
                .map(hosted::Stored::entry)
                .collect(),
            RepositoryType::Proxy => match self.remote.versions(ctx, coll).await? {
                Some(data) => data
                    .iter()
                    .filter(|value| remote::valid_entry(value))
                    .filter_map(Entry::from_remote)
                    .collect(),
                None => return Ok(None),
            },
            RepositoryType::Group => {
                let mut merged: Vec<Entry> = Vec::new();
                let mut any = false;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.entries(&member, coll)).await {
                        Ok(Some(entries)) => {
                            any = true;
                            for entry in entries {
                                if !merged.iter().any(|known| known.version == entry.version) {
                                    merged.push(entry);
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                if !any {
                    return Ok(None);
                }
                merged
            }
        };
        entries.retain(|entry| ctx.allowed(View::Read, &coll.artifact_path(&entry.version)));
        api::sort_entries(&mut entries);
        Ok((!entries.is_empty() || ctx.kind() != RepositoryType::Hosted).then_some(entries))
    }

    async fn version(
        &self,
        ctx: &Ctx,
        urls: &Urls,
        coll: &Coll,
        version: &str,
    ) -> Result<Option<Value>, ContentError> {
        if !ctx.allowed(View::Read, &coll.artifact_path(version)) {
            return Ok(None);
        }
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::stored(ctx, Some(coll))
                .await?
                .into_iter()
                .find(|stored| stored.component.version == version)
                .map(|stored| stored.document(urls))),
            RepositoryType::Proxy => self.remote.version(ctx, coll, version).await,
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.version(&member, urls, coll, version)).await {
                        Ok(Some(document)) => return Ok(Some(document)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn artifact(
        &self,
        ctx: &Ctx,
        coll: &Coll,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                Ok(
                    store::find(ctx.db(), ctx.name(), &coll.artifact_path(version))
                        .await?
                        .map(|found| (ctx.clone(), found)),
                )
            }
            RepositoryType::Proxy => Ok(self
                .remote
                .artifact(ctx, coll, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.artifact(&member, coll, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    fn visible(ctx: &Ctx, summary: &Summary) -> bool {
        match ctx.grant(View::Read) {
            Grant::Whole => true,
            Grant::Selected(_) => {
                ctx.allowed(View::Read, &summary.coll.artifact_path(&summary.highest))
            }
            Grant::Nothing => false,
        }
    }

    async fn api_read(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        let urls = Urls::of(ctx);
        let ok = |value: &Value| api::json(StatusCode::OK, Some(request), value);
        if route == Route::Api {
            return Ok(ok(&json!({"available_versions": {"v3": "v3/"}})));
        }
        if let Route::Import(id) = &route {
            if !ctx.grant(View::Read).any() && !ctx.grant(View::Add).any() {
                return Err(denied(ctx));
            }
            let document = imports::document(ctx, id)
                .await?
                .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
            return Ok(ok(&document));
        }
        require_reader(ctx)?;
        let missing = || ContentError::not_found(request.path.clone());
        match route {
            Route::Collections => {
                let summaries: Vec<Value> = self
                    .summaries(ctx)
                    .await?
                    .iter()
                    .filter(|summary| Self::visible(ctx, summary))
                    .map(|summary| summary.document(&urls))
                    .collect();
                Ok(ok(
                    &Page::of(request.query.as_deref()).list(&urls.collections(), summaries)
                ))
            }
            Route::Collection(coll) => {
                let summary = match ctx.grant(View::Read) {
                    Grant::Whole => self.summary(ctx, &coll).await?,
                    _ => {
                        let entries = self.entries(ctx, &coll).await?.unwrap_or_default();
                        Summary::from_entries(coll.clone(), &entries, 0)
                    }
                };
                let summary = summary.ok_or_else(missing)?;
                Ok(ok(&summary.document(&urls)))
            }
            Route::Versions(coll) => {
                let entries = self.entries(ctx, &coll).await?.ok_or_else(missing)?;
                let documents: Vec<Value> = entries
                    .iter()
                    .map(|entry| entry.document(&urls, &coll))
                    .collect();
                Ok(ok(
                    &Page::of(request.query.as_deref()).list(&urls.versions(&coll), documents)
                ))
            }
            Route::Version(coll, version) => {
                let document = self
                    .version(ctx, &urls, &coll, &version)
                    .await?
                    .ok_or_else(missing)?;
                Ok(ok(&api::rewrite_version(document, &urls, &coll, &version)))
            }
            _ => Err(missing()),
        }
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(format!(
                "{} is a {} repository; publish collections to a hosted repository",
                ctx.name(),
                ctx.kind()
            )));
        }
        if !ctx.grant(View::Add).any() && !ctx.grant(View::Edit).any() {
            return Err(denied(ctx));
        }
        let boundary = request
            .header("content-type")
            .and_then(|value| multer::parse_boundary(value).ok())
            .ok_or_else(|| {
                ContentError::BadRequest(
                    "publish a collection as a multipart/form-data request with a file part"
                        .to_owned(),
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
        let mut sha256: Option<String> = None;
        let mut uploaded = None;
        while let Some(part) = multipart.next_field().await.map_err(bad)? {
            match part.name().unwrap_or_default() {
                "file" if uploaded.is_none() => {
                    let file = file_name_of(part.file_name())
                        .unwrap_or_default()
                        .to_owned();
                    let named = names::parse_file_name(&file).ok_or_else(|| {
                        ContentError::BadRequest(format!(
                            "the file name {file} is not <namespace>-<name>-<version>.tar.gz"
                        ))
                    })?;
                    hosted::admit(ctx, &named.0, &named.1).await?;
                    let base64 = part
                        .headers()
                        .get("content-transfer-encoding")
                        .and_then(|value| value.to_str().ok())
                        .is_some_and(|value| value.trim().eq_ignore_ascii_case("base64"));
                    let mut body: astrum_blob::ContentStream =
                        Box::pin(part.map_err(std::io::Error::other));
                    if base64 {
                        body = transfer::decode_base64(body);
                    }
                    let written = store::write_blob(ctx, body).await?;
                    uploaded = Some((named, written));
                }
                "sha256" => sha256 = Some(part.text().await.map_err(bad)?.trim().to_owned()),
                _ => {
                    let mut part = part;
                    while part.chunk().await.map_err(bad)?.is_some() {}
                }
            }
        }
        let Some((named, written)) = uploaded else {
            return Err(ContentError::BadRequest(
                "the upload has no file part".to_owned(),
            ));
        };
        if let Some(expected) = sha256.filter(|expected| !expected.is_empty())
            && !expected.eq_ignore_ascii_case(&written.digests.sha256)
        {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(ContentError::BadRequest(format!(
                "the file does not match the sha256 of the form ({expected})"
            )));
        }
        let (coll, version) = named.clone();
        let outcome = match hosted::store_collection(ctx, written, Some(named)).await {
            Ok((collection, _)) => imports::Outcome {
                coll: Some((coll.clone(), version.clone())),
                messages: vec![
                    ("INFO", format!("Importing collection {coll} {version}")),
                    (
                        "INFO",
                        format!(
                            "Read MANIFEST.json with {} content items",
                            collection.contents.len()
                        ),
                    ),
                    (
                        "INFO",
                        format!("Collection {coll} {version} is published in {}", ctx.name()),
                    ),
                ],
                error: None,
            },
            Err(Refusal::Content(message)) => imports::Outcome {
                coll: Some((coll.clone(), version.clone())),
                messages: vec![
                    ("INFO", format!("Importing collection {coll} {version}")),
                    ("ERROR", message.clone()),
                ],
                error: Some(("invalid", message)),
            },
            Err(Refusal::Request(error)) => return Err(error),
        };
        let id = imports::record(ctx, outcome).await?;
        Ok(api::json(
            StatusCode::ACCEPTED,
            None,
            &json!({"task": Urls::of(ctx).import(&id.to_string())}),
        ))
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
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

#[async_trait]
impl Format for Ansible {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let api_route = route.is_api();
        let method = request.method.clone();
        let result = match (&method, route) {
            (&Method::GET | &Method::HEAD, Route::Root) => directory_page(&ctx),
            (&Method::GET | &Method::HEAD, Route::Artifact(coll, version)) => {
                ctx.require(View::Read, &coll.artifact_path(&version))?;
                let (owner, found) = self
                    .artifact(&ctx, &coll, &version)
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                serve(
                    &owner,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            (&Method::GET | &Method::HEAD, Route::Other) => self.read_other(&ctx, &request).await,
            (&Method::GET | &Method::HEAD, Route::Publish) => {
                Err(Self::not_allowed(&method, ctx.kind()))
            }
            (&Method::GET | &Method::HEAD, route) => self.api_read(&ctx, &request, route).await,
            (&Method::POST, Route::Publish) => self.publish(&ctx, request).await,
            (&Method::DELETE, Route::Version(coll, version))
                if ctx.kind() == RepositoryType::Hosted =>
            {
                self.delete(&ctx, &coll.artifact_path(&version)).await
            }
            (&Method::DELETE, Route::Artifact(..) | Route::Other)
                if ctx.kind() != RepositoryType::Group =>
            {
                self.delete(&ctx, &request.path).await
            }
            (method, _) => Err(Self::not_allowed(method, ctx.kind())),
        };
        match result {
            Ok(response) => Ok(response),
            Err(error) if api_route => Ok(api::error(error)),
            Err(error) => Ok(error.into_response()),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Artifact(coll, version) => Ok(self
                .artifact(ctx, &coll, &version)
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
        let named = file_name_of(file.filename.as_deref()).and_then(names::parse_file_name);
        match &named {
            Some((coll, version)) => hosted::admit(&ctx, coll, version).await?,
            None if !ctx.grant(View::Add).any() && !ctx.grant(View::Edit).any() => {
                return Err(denied(&ctx));
            }
            None => {}
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::store_collection(&ctx, written, named)
            .await
            .map(|_| ())
            .map_err(Refusal::into_error)
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

    fn coll(namespace: &str, name: &str) -> Coll {
        Coll::new(namespace, name).unwrap()
    }

    #[test]
    fn paths_route_to_the_v3_api_and_archives() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/api/"), Route::Api);
        assert_eq!(route("/api"), Route::Api);
        assert_eq!(route("/api/v3/collections/"), Route::Collections);
        assert_eq!(
            route("/api/v3/collections/community/general/"),
            Route::Collection(coll("community", "general"))
        );
        assert_eq!(
            route("/api/v3/collections/community/general/versions/"),
            Route::Versions(coll("community", "general"))
        );
        assert_eq!(
            route("/api/v3/collections/community/general/versions/8.0.0-rc.1/"),
            Route::Version(coll("community", "general"), "8.0.0-rc.1".to_owned())
        );
        assert_eq!(route("/api/v3/artifacts/collections/"), Route::Publish);
        assert_eq!(
            route("/api/v3/imports/collections/0b1c/"),
            Route::Import("0b1c".to_owned())
        );
        assert_eq!(
            route("/community/general/8.0.0/community-general-8.0.0.tar.gz"),
            Route::Artifact(coll("community", "general"), "8.0.0".to_owned())
        );
        for other in [
            "/community/general/8.0.0/other-8.0.0.tar.gz",
            "/community/general/8.0/community-general-8.0.tar.gz",
            "/api/v3/collections/Community/general/",
            "/api/v3/collections/community/general/versions/latest/",
            "/api/v3/collections/community/general/versions.json",
            "/api/v2/collections/",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        assert!(route("/api/v3/collections/").is_api());
        assert!(!route("/a/b/1.0.0/a-b-1.0.0.tar.gz").is_api());
    }

    #[test]
    fn uploaded_file_names_lose_their_folders() {
        assert_eq!(
            file_name_of(Some("/tmp/build/a-b-1.0.0.tar.gz")),
            Some("a-b-1.0.0.tar.gz")
        );
        assert_eq!(
            file_name_of(Some("C:\\x\\a-b-1.0.0.tar.gz")),
            Some("a-b-1.0.0.tar.gz")
        );
        assert_eq!(file_name_of(Some("")), None);
        assert_eq!(file_name_of(None), None);
    }
}
