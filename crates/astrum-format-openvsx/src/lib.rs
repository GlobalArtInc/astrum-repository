pub mod api;
pub mod gallery;
mod hosted;
pub mod model;
pub mod names;
mod remote;
mod routes;
pub mod vsix;

use std::io;

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, CHALLENGE, ContentError, ContentRequest, Ctx, Format, Serve, UploadField,
    UploadForm, UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::TryStreamExt;
use serde_json::{Map, Value, json};

use crate::api::{Lookup, Search};
use crate::hosted::Scope;
use crate::model::{Urls, extension_document, extensions, rewrite};
use crate::names::FileRef;

pub use routes::routes;

pub const ID: &str = "openvsx";
pub const VSIX: &str = "vsix";
pub const FILE: &str = "file";
pub const METADATA: &str = "metadata";
pub const VSIX_TYPE: &str = "application/vsix";
pub const MAX_VSIX: u64 = 512 * 1024 * 1024;
const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_BODY: usize = 1024 * 1024;
const UPLOAD_FIELD: &str = "openvsx.asset";
const JSON_TYPE: &str = "application/json";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "openvsx.category",
        target: SearchTarget::Component,
        pointer: &["openvsx", "categories"],
    },
    SearchKey {
        param: "openvsx.targetPlatform",
        target: SearchTarget::Asset,
        pointer: &["openvsx", "targetPlatform"],
    },
];

const KEYWORDS: &[&[&str]] = &[
    &["openvsx", "displayName"],
    &["openvsx", "description"],
    &["openvsx", "tags"],
];

#[derive(Default)]
pub struct OpenVsx;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Doc {
    Namespace(String),
    Extension {
        namespace: String,
        name: String,
        platform: Option<String>,
        version: Option<String>,
    },
    Versions {
        namespace: String,
        name: String,
        platform: Option<String>,
        references: bool,
        offset: usize,
        size: usize,
    },
    Reviews {
        namespace: String,
        name: String,
    },
    Search(Search),
    Lookup(Lookup, bool),
    Gallery(gallery::Query),
    GalleryLatest {
        namespace: String,
        name: String,
    },
}

impl Doc {
    fn path(&self) -> String {
        match self {
            Doc::Namespace(namespace) => format!("/api/{namespace}"),
            Doc::Extension {
                namespace,
                name,
                platform,
                version,
            } => {
                let mut path = format!("/api/{namespace}/{name}");
                for part in [platform, version].into_iter().flatten() {
                    path.push('/');
                    path.push_str(part);
                }
                path
            }
            Doc::Versions {
                namespace,
                name,
                platform,
                references,
                ..
            } => {
                let mut path = format!("/api/{namespace}/{name}");
                if let Some(platform) = platform {
                    path.push('/');
                    path.push_str(platform);
                }
                path.push_str(match references {
                    true => "/version-references",
                    false => "/versions",
                });
                path
            }
            Doc::Reviews { namespace, name } => format!("/api/{namespace}/{name}/reviews"),
            Doc::Search(_) => "/api/-/search".to_owned(),
            Doc::Lookup(_, true) => "/api/v2/-/query".to_owned(),
            Doc::Lookup(_, false) => "/api/-/query".to_owned(),
            Doc::Gallery(_) => "/vscode/gallery/extensionquery".to_owned(),
            Doc::GalleryLatest { namespace, name } => {
                format!("/vscode/gallery/{namespace}/{name}/latest")
            }
        }
    }

    fn query(&self) -> Option<String> {
        match self {
            Doc::Versions { offset, size, .. } => Some(api::paging_query(*offset, *size)),
            Doc::Search(search) => Some(search.to_query()),
            Doc::Lookup(lookup, _) => Some(lookup.to_query()),
            Doc::Gallery(query) => Some(
                url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("q", &query.to_value().to_string())
                    .finish(),
            ),
            _ => None,
        }
    }

    fn widened(&self) -> Doc {
        match self {
            Doc::Versions {
                namespace,
                name,
                platform,
                references,
                offset,
                size,
            } => Doc::Versions {
                namespace: namespace.clone(),
                name: name.clone(),
                platform: platform.clone(),
                references: *references,
                offset: 0,
                size: (offset + size).min(100),
            },
            Doc::Search(search) => Doc::Search(search.widened()),
            Doc::Lookup(lookup, v2) => Doc::Lookup(lookup.widened(), *v2),
            Doc::Gallery(query) => Doc::Gallery(query.widened()),
            other => other.clone(),
        }
    }

    fn is_gallery(&self) -> bool {
        matches!(self, Doc::Gallery(_) | Doc::GalleryLatest { .. })
    }

    fn missing(&self) -> String {
        match self {
            Doc::Namespace(namespace) => format!("Namespace not found: {namespace}"),
            Doc::Extension {
                namespace,
                name,
                version: Some(version),
                ..
            } if version != "latest" && version != "pre-release" => {
                format!("Extension not found: {namespace}.{name} {version}")
            }
            Doc::Extension {
                namespace, name, ..
            }
            | Doc::Versions {
                namespace, name, ..
            }
            | Doc::Reviews { namespace, name }
            | Doc::GalleryLatest { namespace, name } => {
                format!("Extension not found: {namespace}.{name}")
            }
            Doc::Search(_) | Doc::Lookup(..) | Doc::Gallery(_) => "nothing matches".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Registry,
    Doc(Doc),
    BadQuery(String),
    File(FileRef),
    PublicKey(String),
    VerifyPat(String),
    Delete(String, String),
    Publish,
    CreateNamespace,
    Lookup(bool),
    GalleryQuery,
    Package {
        namespace: String,
        name: String,
        version: String,
    },
    Asset {
        namespace: String,
        name: String,
        version: String,
        asset_type: String,
        rest: String,
    },
    Unpkg {
        namespace: String,
        name: String,
        version: String,
        path: String,
    },
    Item,
    Other,
}

impl Route {
    fn is_api(&self) -> bool {
        !matches!(
            self,
            Route::Root
                | Route::File(_)
                | Route::Package { .. }
                | Route::Asset { .. }
                | Route::Unpkg { .. }
                | Route::Item
                | Route::Other
        )
    }
}

fn version_like(version: &str) -> bool {
    version == "latest"
        || version == "pre-release"
        || (version.len() <= 128
            && version.starts_with(|c: char| c.is_ascii_digit())
            && version
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || ".-+_".contains(c)))
}

fn route(path: &str, query: Option<&str>) -> Route {
    if path == "/" || path.is_empty() {
        return Route::Root;
    }
    if let Some(file) = FileRef::parse(path) {
        return Route::File(file);
    }
    let directory = path.ends_with('/');
    let parts: Vec<&str> = path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .collect();
    let valid = |names: &[&str]| names.iter().all(|name| names::valid_name(name));
    let owned = |value: &str| value.to_owned();
    let extension = |namespace: &str, name: &str, platform: Option<&str>, version: Option<&str>| {
        Route::Doc(Doc::Extension {
            namespace: owned(namespace),
            name: owned(name),
            platform: platform.map(owned),
            version: version.map(owned),
        })
    };
    let versions =
        |namespace: &str, name: &str, platform: Option<&str>, kind: &str| match api::paging(
            query, 18,
        ) {
            Ok((offset, size)) => Route::Doc(Doc::Versions {
                namespace: owned(namespace),
                name: owned(name),
                platform: platform.map(owned),
                references: kind == "version-references",
                offset,
                size,
            }),
            Err(message) => Route::BadQuery(message),
        };
    match parts.as_slice() {
        ["api", "version"] => Route::Registry,
        ["api", "-", "search"] => match Search::parse(query) {
            Ok(search) => Route::Doc(Doc::Search(search)),
            Err(message) => Route::BadQuery(message),
        },
        ["api", "-", "query"] => Route::Lookup(false),
        ["api", "v2", "-", "query"] => Route::Lookup(true),
        ["api", "-", "publish"] => Route::Publish,
        ["api", "-", "namespace", "create"] => Route::CreateNamespace,
        ["api", "-", "public-key", id] => Route::PublicKey(owned(id)),
        ["api", namespace] if valid(&[namespace]) => Route::Doc(Doc::Namespace(owned(namespace))),
        ["api", namespace, "verify-pat"] if valid(&[namespace]) => {
            Route::VerifyPat(owned(namespace))
        }
        ["api", namespace, name] if valid(&[namespace, name]) => {
            extension(namespace, name, None, None)
        }
        ["api", namespace, name, "reviews"] if valid(&[namespace, name]) => {
            Route::Doc(Doc::Reviews {
                namespace: owned(namespace),
                name: owned(name),
            })
        }
        ["api", namespace, name, "delete"] if valid(&[namespace, name]) => {
            Route::Delete(owned(namespace), owned(name))
        }
        [
            "api",
            namespace,
            name,
            kind @ ("versions" | "version-references"),
        ] if valid(&[namespace, name]) => versions(namespace, name, None, kind),
        ["api", namespace, name, platform]
            if valid(&[namespace, name]) && names::is_platform(platform) =>
        {
            extension(namespace, name, Some(platform), None)
        }
        ["api", namespace, name, version] if valid(&[namespace, name]) && version_like(version) => {
            extension(namespace, name, None, Some(version))
        }
        [
            "api",
            namespace,
            name,
            platform,
            kind @ ("versions" | "version-references"),
        ] if valid(&[namespace, name]) && names::is_platform(platform) => {
            versions(namespace, name, Some(platform), kind)
        }
        ["api", namespace, name, platform, version]
            if valid(&[namespace, name])
                && names::is_platform(platform)
                && version_like(version) =>
        {
            extension(namespace, name, Some(platform), Some(version))
        }
        ["vscode", "gallery", "extensionquery"] => Route::GalleryQuery,
        ["vscode", "gallery", namespace, name, "latest"] if valid(&[namespace, name]) => {
            Route::Doc(Doc::GalleryLatest {
                namespace: owned(namespace),
                name: owned(name),
            })
        }
        [
            "vscode",
            "gallery",
            "publishers",
            namespace,
            "vsextensions",
            name,
            version,
            "vspackage",
        ] if valid(&[namespace, name]) && version_like(version) => Route::Package {
            namespace: owned(namespace),
            name: owned(name),
            version: owned(version),
        },
        [
            "vscode",
            "asset",
            namespace,
            name,
            version,
            asset_type,
            rest @ ..,
        ] if valid(&[namespace, name]) && version_like(version) => Route::Asset {
            namespace: owned(namespace),
            name: owned(name),
            version: owned(version),
            asset_type: owned(asset_type),
            rest: rest.join("/"),
        },
        ["vscode", "unpkg", namespace, name, version, rest @ ..]
            if valid(&[namespace, name]) && version_like(version) =>
        {
            let mut inner = rest.join("/");
            if directory && !inner.is_empty() {
                inner.push('/');
            }
            Route::Unpkg {
                namespace: owned(namespace),
                name: owned(name),
                version: owned(version),
                path: inner,
            }
        }
        ["vscode", "item"] => Route::Item,
        _ => Route::Other,
    }
}

fn query_value(query: Option<&str>, key: &str) -> Option<String> {
    url::form_urlencoded::parse(query.unwrap_or("").as_bytes())
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn json_response(
    method: &Method,
    status: StatusCode,
    content_type: &str,
    value: &Value,
) -> Response {
    let body = value.to_string();
    let builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-cache");
    match *method {
        Method::HEAD => builder
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        _ => builder.body(Body::from(body)),
    }
    .expect("a JSON response is valid")
}

fn bytes_response(method: &Method, data: Vec<u8>, content_type: String) -> Response {
    let builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, data.len());
    match *method {
        Method::HEAD => builder.body(Body::empty()),
        _ => builder.body(Body::from(data)),
    }
    .expect("a file response is valid")
}

fn api_error(error: ContentError) -> Response {
    let status = error.status();
    let message = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(error = format!("{inner:#}"), "an Open VSX request failed");
            "the server failed; see its log".to_owned()
        }
        other => other.to_string(),
    };
    let mut response = json_response(&Method::GET, status, JSON_TYPE, &json!({"error": message}));
    let headers = response.headers_mut();
    match error {
        ContentError::Unauthorized => {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(CHALLENGE),
            );
        }
        ContentError::MethodNotAllowed { allow, .. } => {
            headers.insert(header::ALLOW, HeaderValue::from_static(allow));
        }
        _ => {}
    }
    response
}

fn preflight(request: &ContentRequest) -> Response {
    let wanted: String = request
        .header("access-control-request-headers")
        .unwrap_or("content-type")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ',' | ' '))
        .collect();
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, HEAD, POST, OPTIONS",
        )
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, wanted)
        .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
        .body(Body::empty())
        .expect("a preflight answer is valid")
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
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(format!(
            "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is an Open VSX repository of VS Code extensions. Editors such as VSCodium read it as their extension gallery at <code>{url}/vscode/gallery</code>; <code>ovsx</code> uses <code>{url}</code> as its registry URL.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
            escape(&ctx.base_url),
        )))
        .map_err(ContentError::internal)
}

fn may_read(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Read).any() {
        true => Ok(()),
        false => ctx.require(View::Read, "/"),
    }
}

fn signed_in(ctx: &Ctx) -> Result<(), ContentError> {
    match &ctx.subject {
        Some(subject) if !subject.anonymous => Ok(()),
        _ => Err(ContentError::Unauthorized),
    }
}

fn not_hosted(ctx: &Ctx) -> ContentError {
    ContentError::BadRequest(format!(
        "{} is a {} repository; publish extensions to a hosted Open VSX repository",
        ctx.name(),
        ctx.kind()
    ))
}

fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => "GET,HEAD,POST,DELETE,OPTIONS",
            RepositoryType::Proxy => "GET,HEAD,POST,DELETE,OPTIONS",
            RepositoryType::Group => "GET,HEAD,POST,OPTIONS",
        },
    }
}

fn success(status: StatusCode, message: String) -> Response {
    json_response(
        &Method::POST,
        status,
        JSON_TYPE,
        &json!({"success": message}),
    )
}

fn decode(path: &str) -> String {
    percent_encoding::percent_decode_str(path)
        .decode_utf8_lossy()
        .into_owned()
}

enum Served {
    Asset(Box<(Ctx, Asset)>),
    Bytes(Vec<u8>, String),
}

enum Unpkg {
    Listing(Value),
    Served(Served),
}

fn content_type_of(file: &str) -> String {
    astrum_content::mime::from_path(file)
        .unwrap_or_else(|| astrum_content::mime::OCTET_STREAM.to_owned())
}

async fn send(request: &ContentRequest, served: Served) -> Result<Response, ContentError> {
    match served {
        Served::Asset(stored) => {
            let (owner, found) = *stored;
            let options = match found.kind == VSIX {
                true => Serve::download(),
                false => Serve::default(),
            };
            serve(&owner, &found, &request.method, &request.headers, options).await
        }
        Served::Bytes(data, content_type) => {
            Ok(bytes_response(&request.method, data, content_type))
        }
    }
}

fn log_member(ctx: &Ctx, member: &Ctx, error: &ContentError) {
    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
}

impl OpenVsx {
    async fn hosted_doc(&self, ctx: &Ctx, doc: &Doc) -> Result<Option<Value>, ContentError> {
        let urls = Urls::of(ctx);
        let scope = match doc {
            Doc::Namespace(namespace) => Scope::Namespace(namespace),
            Doc::Extension {
                namespace, name, ..
            }
            | Doc::Versions {
                namespace, name, ..
            }
            | Doc::Reviews { namespace, name }
            | Doc::GalleryLatest { namespace, name } => Scope::Extension(namespace, name),
            Doc::Search(_) | Doc::Lookup(..) | Doc::Gallery(_) => Scope::All,
        };
        let releases = hosted::releases(ctx, scope).await?;
        let exts = extensions(&releases);
        let first = exts.first();
        Ok(match doc {
            Doc::Namespace(_) => first.map(|first| {
                let listed: Map<String, Value> = exts
                    .iter()
                    .map(|ext| (ext.name.to_owned(), json!(urls.extension(ext.namespace, ext.name))))
                    .collect();
                json!({"name": first.namespace, "extensions": listed, "verified": true, "access": "restricted"})
            }),
            Doc::Extension {
                platform, version, ..
            } => first.and_then(|ext| {
                ext.aliased(version.as_deref().unwrap_or("latest"), platform.as_deref())
                    .map(|release| extension_document(&urls, ext, release))
            }),
            Doc::Versions {
                platform,
                references,
                offset,
                size,
                ..
            } => first.map(|ext| {
                api::versions(&urls, ext, platform.as_deref(), *references, *offset, *size)
            }),
            Doc::Reviews { .. } => first.map(|ext| {
                json!({
                    "postUrl": urls.under(ext.namespace, ext.name, "review"),
                    "deleteUrl": urls.under(ext.namespace, ext.name, "review/delete"),
                    "reviews": [],
                })
            }),
            Doc::Search(search) => Some(api::search(&urls, &exts, search)),
            Doc::Lookup(lookup, _) => Some(api::lookup(&urls, &exts, lookup)),
            Doc::Gallery(query) => Some(gallery::answer(&urls, &exts, query)),
            Doc::GalleryLatest { .. } => first.map(|ext| {
                gallery::extension(
                    &urls,
                    ext,
                    gallery::ALL_FLAGS | gallery::FLAG_LATEST_ONLY,
                    &[],
                )
            }),
        })
    }

    async fn doc(&self, ctx: &Ctx, doc: &Doc) -> Result<Option<Value>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => self.hosted_doc(ctx, doc).await,
            RepositoryType::Proxy => {
                let path = doc.path();
                let query = doc.query();
                remote::document(
                    ctx,
                    remote::Wanted {
                        cache: match &query {
                            Some(query) => remote::hashed(&path, query),
                            None => path.clone(),
                        },
                        path: &path,
                        query,
                        accept: match doc.is_gallery() {
                            true => gallery::CONTENT_TYPE,
                            false => JSON_TYPE,
                        },
                    },
                )
                .await
            }
            RepositoryType::Group => {
                let asked = doc.widened();
                let mut answers = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.doc(&member, &asked)).await {
                        Ok(Some(mut answer)) => {
                            rewrite(&mut answer, &member.repository_url(), &ctx.repository_url());
                            answers.push(answer);
                        }
                        Ok(None) => {}
                        Err(error) => log_member(ctx, &member, &error),
                    }
                }
                Ok(match doc {
                    Doc::Namespace(_) => api::merge_namespace(answers),
                    Doc::Extension { version, .. } => {
                        api::merge_extension(answers, version.as_deref())
                    }
                    Doc::Versions {
                        references,
                        offset,
                        size,
                        ..
                    } => (!answers.is_empty())
                        .then(|| api::merge_versions(answers, *references, *offset, *size)),
                    Doc::Reviews { .. } => answers.into_iter().next(),
                    Doc::Search(search) => Some(api::merge_search(search, answers)),
                    Doc::Lookup(lookup, _) => Some(api::merge_lookup(lookup, answers)),
                    Doc::Gallery(query) => Some(gallery::merge(query, answers)),
                    Doc::GalleryLatest { .. } => {
                        gallery::merge_extensions(answers, true).into_iter().next()
                    }
                })
            }
        }
    }

    async fn read_doc(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        doc: &Doc,
    ) -> Result<Response, ContentError> {
        may_read(ctx)?;
        let document = self
            .doc(ctx, doc)
            .await?
            .ok_or_else(|| ContentError::not_found(doc.missing()))?;
        let content_type = match doc.is_gallery() {
            true => gallery::CONTENT_TYPE,
            false => JSON_TYPE,
        };
        Ok(json_response(
            &request.method,
            StatusCode::OK,
            content_type,
            &document,
        ))
    }

    async fn file(&self, ctx: &Ctx, file: &FileRef) -> Result<Option<Served>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let releases =
                    hosted::releases(ctx, Scope::Extension(&file.namespace, &file.name)).await?;
                let Some(release) = releases.into_iter().find(|release| {
                    release.version == file.version && release.platform == file.platform
                }) else {
                    return Ok(None);
                };
                if file.is_vsix() {
                    return Ok(Some(Served::Asset(Box::new((ctx.clone(), release.asset)))));
                }
                if file.is_sha256() {
                    return Ok(Some(Served::Bytes(
                        release.asset.sha256.clone().into_bytes(),
                        "text/plain".to_owned(),
                    )));
                }
                let Some(entry) = release.entry_for(&file.file) else {
                    return Ok(None);
                };
                Ok(hosted::entry(ctx, &release, entry, MAX_FILE)
                    .await?
                    .map(|data| Served::Bytes(data, content_type_of(&file.file))))
            }
            RepositoryType::Proxy => Ok(remote::file(ctx, file)
                .await?
                .map(|found| Served::Asset(Box::new((ctx.clone(), found))))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.file(&member, file)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member(ctx, &member, &error),
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
        file: &FileRef,
    ) -> Result<Response, ContentError> {
        ctx.require(
            View::Read,
            &names::vsix_path(&file.namespace, &file.name, &file.platform, &file.version),
        )?;
        let served = self
            .file(ctx, file)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("{} was not found", file.file)))?;
        send(request, served).await
    }

    async fn resolve(
        &self,
        ctx: &Ctx,
        namespace: &str,
        name: &str,
        version: &str,
        platform: Option<String>,
        kind: &str,
    ) -> Result<Option<FileRef>, ContentError> {
        let doc = Doc::Extension {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            platform,
            version: Some(version.to_owned()),
        };
        let Some(document) = self.doc(ctx, &doc).await? else {
            return Ok(None);
        };
        let base = format!("{}/", ctx.repository_url());
        Ok(document
            .pointer(&format!("/files/{kind}"))
            .and_then(Value::as_str)
            .and_then(|url| url.strip_prefix(&base))
            .and_then(|path| FileRef::parse(&format!("/{}", decode(path)))))
    }

    async fn asset(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        namespace: &str,
        name: &str,
        version: &str,
        asset_type: &str,
    ) -> Result<Response, ContentError> {
        may_read(ctx)?;
        let platform = query_value(request.query.as_deref(), "targetPlatform")
            .filter(|platform| names::is_platform(platform));
        let missing = || {
            ContentError::not_found(format!(
                "{asset_type} of {namespace}.{name} {version} was not found"
            ))
        };
        let kind = vsix::kind_of(asset_type).ok_or_else(missing)?;
        let file = self
            .resolve(ctx, namespace, name, version, platform, kind)
            .await?
            .ok_or_else(missing)?;
        self.download(ctx, request, &file).await
    }

    async fn unpkg(
        &self,
        ctx: &Ctx,
        namespace: &str,
        name: &str,
        version: &str,
        path: &str,
    ) -> Result<Option<Unpkg>, ContentError> {
        let listing = path.is_empty() || path.ends_with('/');
        match ctx.kind() {
            RepositoryType::Hosted => {
                let releases = hosted::releases(ctx, Scope::Extension(namespace, name)).await?;
                let exts = extensions(&releases);
                let Some(release) = exts.first().and_then(|ext| ext.version(version, None)) else {
                    return Ok(None);
                };
                let urls = Urls::of(ctx);
                match listing {
                    true => {
                        let children = hosted::listing(ctx, release, path).await?;
                        if children.is_empty() {
                            return Ok(None);
                        }
                        Ok(Some(Unpkg::Listing(Value::Array(
                            children
                                .iter()
                                .map(|child| {
                                    json!(urls.unpkg(
                                        &release.namespace,
                                        &release.name,
                                        &release.version,
                                        child
                                    ))
                                })
                                .collect(),
                        ))))
                    }
                    false => Ok(hosted::entry(ctx, release, path, MAX_FILE)
                        .await?
                        .map(|data| Unpkg::Served(Served::Bytes(data, content_type_of(path))))),
                }
            }
            RepositoryType::Proxy => {
                let remote_path = format!(
                    "/vscode/unpkg/{namespace}/{name}/{version}/{}",
                    names::encode_path(path)
                );
                let cache = format!("/vscode/unpkg/{namespace}/{name}/{version}/{path}");
                match listing {
                    true => Ok(remote::document(
                        ctx,
                        remote::Wanted {
                            cache: format!("{}/-/listing.json", cache.trim_end_matches('/')),
                            path: &remote_path,
                            query: None,
                            accept: JSON_TYPE,
                        },
                    )
                    .await?
                    .map(Unpkg::Listing)),
                    false => Ok(remote::passthrough(ctx, &remote_path, None, &cache)
                        .await?
                        .map(|found| Unpkg::Served(Served::Asset(Box::new((ctx.clone(), found)))))),
                }
            }
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.unpkg(&member, namespace, name, version, path)).await {
                        Ok(Some(Unpkg::Listing(mut listing))) => {
                            rewrite(
                                &mut listing,
                                &member.repository_url(),
                                &ctx.repository_url(),
                            );
                            return Ok(Some(Unpkg::Listing(listing)));
                        }
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn item(
        &self,
        ctx: &Ctx,
        namespace: &str,
        name: &str,
    ) -> Result<Option<String>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let releases = hosted::releases(ctx, Scope::Extension(namespace, name)).await?;
                Ok(releases.first().map(|release| {
                    format!(
                        "{}/browse/{}/{}/{}",
                        ctx.base_url,
                        names::encode(ctx.name()),
                        release.namespace,
                        release.name
                    )
                }))
            }
            RepositoryType::Proxy => Ok(remote::item(ctx, &names::id(namespace, name))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.item(&member, namespace, name)).await {
                        Ok(Some(location)) => return Ok(Some(location)),
                        Ok(None) => {}
                        Err(error) => log_member(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(ctx));
        }
        signed_in(ctx)?;
        hosted::may_publish(ctx)?;
        if request
            .header("content-length")
            .and_then(|value| value.parse::<u64>().ok())
            .is_some_and(|length| length > MAX_VSIX)
        {
            return Err(ContentError::BadRequest(format!(
                "the extension package is larger than {MAX_VSIX} bytes"
            )));
        }
        let limited = Body::new(http_body_util::Limited::new(
            request.take_body(),
            MAX_VSIX as usize,
        ));
        let body: astrum_blob::ContentStream =
            Box::pin(limited.into_data_stream().map_err(io::Error::other));
        let written = match store::write_blob(ctx, body).await {
            Ok(written) => written,
            Err(error) => {
                tracing::info!(repository = ctx.name(), %error, "an extension package could not be stored");
                return Err(ContentError::BadRequest(format!(
                    "the extension package could not be received; it must be at most {MAX_VSIX} bytes"
                )));
            }
        };
        if written.size == 0 {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(ContentError::BadRequest(
                "send the .vsix file as the body of the request".to_owned(),
            ));
        }
        let adopted = hosted::adopt(ctx, written).await?;
        let document = self
            .doc(
                ctx,
                &Doc::Extension {
                    namespace: adopted.namespace.clone(),
                    name: adopted.name.clone(),
                    platform: Some(adopted.platform.clone()),
                    version: Some(adopted.version.clone()),
                },
            )
            .await?
            .unwrap_or_else(|| json!({"namespace": adopted.namespace, "name": adopted.name, "version": adopted.version}));
        let mut document = document;
        if adopted.unchanged
            && let Some(object) = document.as_object_mut()
        {
            object.insert(
                "warning".into(),
                json!(format!(
                    "{}.{} {} is already published with the same package",
                    adopted.namespace, adopted.name, adopted.version
                )),
            );
        }
        let mut response = json_response(&Method::POST, StatusCode::CREATED, JSON_TYPE, &document);
        if let Some(url) = document.get("url").and_then(Value::as_str)
            && let Ok(location) = HeaderValue::from_str(url)
        {
            response.headers_mut().insert(header::LOCATION, location);
        }
        Ok(response)
    }

    fn verify_token(&self, ctx: &Ctx, namespace: &str) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(ctx));
        }
        signed_in(ctx)?;
        hosted::may_publish(ctx)?;
        Ok(success(
            StatusCode::OK,
            format!(
                "The token is valid and can publish extensions of {namespace} to {}",
                ctx.name()
            ),
        ))
    }

    async fn create_namespace(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
    ) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(ctx));
        }
        signed_in(ctx)?;
        hosted::may_publish(ctx)?;
        let body = request.bytes(MAX_BODY).await?;
        let given: Value = serde_json::from_slice(&body)
            .map_err(|error| ContentError::BadRequest(format!("the body is not JSON: {error}")))?;
        let name = given
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| names::valid_name(name))
            .ok_or_else(|| ContentError::BadRequest("give the namespace as {\"name\": \"<namespace>\"} with letters, digits, - and _".to_owned()))?;
        Ok(success(
            StatusCode::CREATED,
            format!(
                "Created namespace {name}; {} takes extensions of any namespace its users may publish to",
                ctx.name()
            ),
        ))
    }

    async fn unpublish(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        namespace: &str,
        name: &str,
    ) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(ctx));
        }
        signed_in(ctx)?;
        let body = request.bytes(MAX_BODY).await?;
        let all = query_value(request.query.as_deref(), "allVersions").as_deref() == Some("true");
        let wanted = match (all, body.iter().all(u8::is_ascii_whitespace)) {
            (true, _) => None,
            (false, true) => {
                return Err(ContentError::BadRequest(
                    "name the versions to delete in the body or pass allVersions=true".to_owned(),
                ));
            }
            (false, false) => {
                let given: Value = serde_json::from_slice(&body).map_err(|error| {
                    ContentError::BadRequest(format!("the body is not JSON: {error}"))
                })?;
                Some(
                    given
                        .as_array()
                        .ok_or_else(|| {
                            ContentError::BadRequest(
                                "give the versions as a list of {\"version\", \"targetPlatform\"}"
                                    .to_owned(),
                            )
                        })?
                        .iter()
                        .filter_map(|item| {
                            Some((
                                item.get("version")?.as_str()?.to_owned(),
                                item.get("targetPlatform")
                                    .and_then(Value::as_str)
                                    .map(str::to_owned),
                            ))
                        })
                        .collect(),
                )
            }
        };
        let deleted = hosted::delete(ctx, namespace, name, wanted).await?;
        if deleted == 0 {
            return Err(ContentError::not_found(format!(
                "Extension not found: {namespace}.{name}"
            )));
        }
        Ok(success(
            StatusCode::OK,
            format!("Deleted {deleted} package(s) of {namespace}.{name}"),
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

    async fn answer(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        let method = request.method.clone();
        let kind = ctx.kind();
        let reading = matches!(method, Method::GET | Method::HEAD);
        match (route, reading) {
            (Route::Root, true) => directory_page(ctx),
            (Route::Registry, true) => Ok(json_response(
                &method,
                StatusCode::OK,
                JSON_TYPE,
                &json!({"version": env!("CARGO_PKG_VERSION"), "maxExtensionSize": MAX_VSIX}),
            )),
            (Route::BadQuery(message), _) => Err(ContentError::BadRequest(message)),
            (Route::Doc(doc), true) => self.read_doc(ctx, &request, &doc).await,
            (Route::Lookup(v2), true) => {
                let lookup = Lookup::parse(request.query.as_deref(), v2)
                    .map_err(ContentError::BadRequest)?;
                self.read_doc(ctx, &request, &Doc::Lookup(lookup, v2)).await
            }
            (Route::Lookup(v2), false) if method == Method::POST => {
                let body = request.bytes(MAX_BODY).await?;
                let given: Value = serde_json::from_slice(&body).map_err(|error| {
                    ContentError::BadRequest(format!("the body is not JSON: {error}"))
                })?;
                let lookup = Lookup::from_body(&given).map_err(ContentError::BadRequest)?;
                self.read_doc(ctx, &request, &Doc::Lookup(lookup, v2)).await
            }
            (Route::GalleryQuery, true) => {
                let given = query_value(request.query.as_deref(), "q").ok_or_else(|| {
                    ContentError::BadRequest("give the query as q=<JSON> or POST it".to_owned())
                })?;
                let value: Value = serde_json::from_str(&given)
                    .map_err(|error| ContentError::BadRequest(format!("q is not JSON: {error}")))?;
                let query = gallery::Query::parse(&value).map_err(ContentError::BadRequest)?;
                self.read_doc(ctx, &request, &Doc::Gallery(query)).await
            }
            (Route::GalleryQuery, false) if method == Method::POST => {
                let body = request.bytes(MAX_BODY).await?;
                let value: Value = serde_json::from_slice(&body).map_err(|error| {
                    ContentError::BadRequest(format!("the query is not JSON: {error}"))
                })?;
                let query = gallery::Query::parse(&value).map_err(ContentError::BadRequest)?;
                let response = self.read_doc(ctx, &request, &Doc::Gallery(query)).await?;
                Ok(response)
            }
            (Route::File(file), true) => {
                may_read(ctx)?;
                self.download(ctx, &request, &file).await
            }
            (
                Route::Package {
                    namespace,
                    name,
                    version,
                },
                true,
            ) => {
                self.asset(
                    ctx,
                    &request,
                    &namespace,
                    &name,
                    &version,
                    "Microsoft.VisualStudio.Services.VSIXPackage",
                )
                .await
            }
            (
                Route::Asset {
                    namespace,
                    name,
                    version,
                    asset_type,
                    rest,
                },
                true,
            ) => match asset_type.as_str() {
                "Microsoft.VisualStudio.Code.WebResources" => {
                    self.read_unpkg(ctx, &request, &namespace, &name, &version, &rest)
                        .await
                }
                _ => {
                    self.asset(ctx, &request, &namespace, &name, &version, &asset_type)
                        .await
                }
            },
            (
                Route::Unpkg {
                    namespace,
                    name,
                    version,
                    path,
                },
                true,
            ) => {
                self.read_unpkg(ctx, &request, &namespace, &name, &version, &path)
                    .await
            }
            (Route::Item, true) => {
                may_read(ctx)?;
                let item = query_value(request.query.as_deref(), "itemName").unwrap_or_default();
                let (namespace, name) = names::split_id(&item).ok_or_else(|| {
                    ContentError::BadRequest(format!("{item} is not <publisher>.<name>"))
                })?;
                let location = self.item(ctx, namespace, name).await?.ok_or_else(|| {
                    ContentError::not_found(format!("Extension not found: {item}"))
                })?;
                Response::builder()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, location)
                    .body(Body::empty())
                    .map_err(ContentError::internal)
            }
            (Route::PublicKey(id), true) => {
                may_read(ctx)?;
                if kind != RepositoryType::Proxy {
                    return Err(ContentError::not_found(format!(
                        "Public key not found: {id}"
                    )));
                }
                let path = format!("/api/-/public-key/{}", names::encode(&id));
                let found = remote::passthrough(ctx, &path, None, &path)
                    .await?
                    .ok_or_else(|| {
                        ContentError::not_found(format!("Public key not found: {id}"))
                    })?;
                send(&request, Served::Asset(Box::new((ctx.clone(), found)))).await
            }
            (Route::VerifyPat(namespace), true) => self.verify_token(ctx, &namespace),
            (Route::Publish, false) if method == Method::POST => self.publish(ctx, request).await,
            (Route::CreateNamespace, false) if method == Method::POST => {
                self.create_namespace(ctx, request).await
            }
            (Route::Delete(namespace, name), false) if method == Method::POST => {
                self.unpublish(ctx, request, &namespace, &name).await
            }
            (Route::File(file), false)
                if method == Method::DELETE && kind != RepositoryType::Group =>
            {
                self.delete(ctx, &file.path()).await
            }
            (Route::Doc(doc), false)
                if method == Method::DELETE && kind == RepositoryType::Proxy =>
            {
                let path = doc.path();
                let cache = match doc.query() {
                    Some(query) => remote::hashed(&path, &query),
                    None => path,
                };
                self.delete(ctx, &cache).await
            }
            (Route::Other, false) if method == Method::DELETE && kind != RepositoryType::Group => {
                self.delete(ctx, &request.path).await
            }
            (Route::Other, true) => Err(ContentError::not_found(request.path.clone())),
            _ => Err(not_allowed(&method, kind)),
        }
    }

    async fn read_unpkg(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        namespace: &str,
        name: &str,
        version: &str,
        path: &str,
    ) -> Result<Response, ContentError> {
        may_read(ctx)?;
        match self.unpkg(ctx, namespace, name, version, path).await? {
            Some(Unpkg::Listing(listing)) => Ok(json_response(
                &request.method,
                StatusCode::OK,
                JSON_TYPE,
                &listing,
            )),
            Some(Unpkg::Served(served)) => send(request, served).await,
            None => Err(ContentError::not_found(format!(
                "{path} of {namespace}.{name} {version} was not found"
            ))),
        }
    }
}

#[async_trait]
impl Format for OpenVsx {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path, request.query.as_deref());
        let api = route.is_api();
        let mut response = match request.method {
            Method::OPTIONS => preflight(&request),
            _ => match self.answer(&ctx, request, route).await {
                Ok(response) => response,
                Err(error) if api => api_error(error),
                Err(error) => error.into_response(),
            },
        };
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        Ok(response)
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match FileRef::parse(path) {
            Some(file) => Ok(match self.file(ctx, &file).await? {
                Some(Served::Asset(stored)) => Some(stored.1),
                _ => None,
            }),
            None => match ctx.kind() {
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
            component_fields: Vec::new(),
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
            .get(UPLOAD_FIELD)
            .ok_or_else(|| ContentError::BadRequest(format!("{UPLOAD_FIELD} is required")))?;
        hosted::may_publish(&ctx)?;
        if file.size > MAX_VSIX {
            return Err(ContentError::BadRequest(format!(
                "the extension package is larger than {MAX_VSIX} bytes"
            )));
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::adopt(&ctx, written).await.map(|_| ())
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        KEYWORDS
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN a.path ~ '^/api/[^/]+/[^/]+/[^/]+/[^/]+/file/[^/]+$' THEN regexp_replace(a.path, '^/api/([^/]+)/([^/]+)/([^/]+)/([^/]+)/file/', '/\\1/\\2/\\4/\\3/') WHEN a.path ~ '^/api/[^/]+/[^/]+/[^/]+/file/[^/]+$' THEN regexp_replace(a.path, '^/api/([^/]+)/([^/]+)/([^/]+)/file/', '/\\1/\\2/\\3/') ELSE a.path END",
            ),
            generated: &[],
        }
    }
}

pub mod testing {
    pub use crate::vsix::testing::{Spec, vsix};
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(path: &str) -> Doc {
        match route(path, None) {
            Route::Doc(doc) => doc,
            other => panic!("{path}: {other:?}"),
        }
    }

    #[test]
    fn paths_route_to_the_registry_api_and_the_gallery() {
        assert_eq!(route("/", None), Route::Root);
        assert_eq!(route("/api/version", None), Route::Registry);
        assert_eq!(doc("/api/acme"), Doc::Namespace("acme".to_owned()));
        assert_eq!(
            doc("/api/acme/hello"),
            Doc::Extension {
                namespace: "acme".to_owned(),
                name: "hello".to_owned(),
                platform: None,
                version: None
            }
        );
        assert_eq!(
            doc("/api/acme/hello/linux-x64/1.0.0"),
            Doc::Extension {
                namespace: "acme".to_owned(),
                name: "hello".to_owned(),
                platform: Some("linux-x64".to_owned()),
                version: Some("1.0.0".to_owned())
            }
        );
        assert_eq!(
            doc("/api/acme/hello/pre-release"),
            Doc::Extension {
                namespace: "acme".to_owned(),
                name: "hello".to_owned(),
                platform: None,
                version: Some("pre-release".to_owned())
            }
        );
        assert!(matches!(
            doc("/api/acme/hello/web/version-references"),
            Doc::Versions {
                references: true,
                platform: Some(_),
                ..
            }
        ));
        assert!(matches!(
            route("/api/acme/hello/versions", Some("size=x")),
            Route::BadQuery(_)
        ));
        assert!(matches!(
            route("/api/-/search", Some("query=a")),
            Route::Doc(Doc::Search(_))
        ));
        assert_eq!(route("/api/-/query", None), Route::Lookup(false));
        assert_eq!(route("/api/v2/-/query", None), Route::Lookup(true));
        assert_eq!(route("/api/-/publish", None), Route::Publish);
        assert_eq!(
            route("/api/-/namespace/create", None),
            Route::CreateNamespace
        );
        assert_eq!(
            route("/api/acme/verify-pat", None),
            Route::VerifyPat("acme".to_owned())
        );
        assert_eq!(
            route("/api/acme/hello/delete", None),
            Route::Delete("acme".to_owned(), "hello".to_owned())
        );
        assert!(matches!(
            route("/api/acme/hello/1.0.0/file/README.md", None),
            Route::File(_)
        ));
        assert_eq!(
            route("/vscode/gallery/extensionquery", None),
            Route::GalleryQuery
        );
        assert!(matches!(
            doc("/vscode/gallery/acme/hello/latest"),
            Doc::GalleryLatest { .. }
        ));
        assert!(matches!(
            route(
                "/vscode/gallery/publishers/acme/vsextensions/hello/1.0.0/vspackage",
                None
            ),
            Route::Package { .. }
        ));
        assert_eq!(
            route(
                "/vscode/asset/acme/hello/1.0.0/Microsoft.VisualStudio.Code.WebResources/extension/a.js",
                None
            ),
            Route::Asset {
                namespace: "acme".to_owned(),
                name: "hello".to_owned(),
                version: "1.0.0".to_owned(),
                asset_type: "Microsoft.VisualStudio.Code.WebResources".to_owned(),
                rest: "extension/a.js".to_owned()
            }
        );
        assert_eq!(
            route("/vscode/unpkg/acme/hello/1.0.0/extension/", None),
            Route::Unpkg {
                namespace: "acme".to_owned(),
                name: "hello".to_owned(),
                version: "1.0.0".to_owned(),
                path: "extension/".to_owned()
            }
        );
        assert_eq!(route("/vscode/item", None), Route::Item);
        for other in [
            "/api",
            "/api/a.b",
            "/api/acme/hello/mars/x",
            "/vscode/gallery",
            "/other",
        ] {
            assert_eq!(route(other, None), Route::Other, "{other}");
        }
        assert!(route("/api/-/search", None).is_api());
        assert!(!route("/vscode/item", None).is_api());
    }

    #[test]
    fn documents_name_their_remote_paths() {
        assert_eq!(
            doc("/api/acme/hello/linux-x64/1.0.0").path(),
            "/api/acme/hello/linux-x64/1.0.0"
        );
        let versions = match route("/api/acme/hello/versions", Some("offset=20&size=10")) {
            Route::Doc(doc) => doc,
            other => panic!("{other:?}"),
        };
        assert_eq!(versions.query().as_deref(), Some("offset=20&size=10"));
        assert!(matches!(
            versions.widened(),
            Doc::Versions {
                offset: 0,
                size: 30,
                ..
            }
        ));
        assert_eq!(
            doc("/vscode/gallery/acme/hello/latest").path(),
            "/vscode/gallery/acme/hello/latest"
        );
    }
}
