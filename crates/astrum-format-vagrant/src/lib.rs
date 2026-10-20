pub mod catalog;
mod hosted;
pub mod names;
mod remote;

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, CHALLENGE, ContentError, ContentRequest, Ctx, Format, Grant, Serve, UploadField,
    UploadForm, UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::catalog::Catalog;
use crate::names::{ARCHITECTURES, BoxFile, BoxName};

pub use hosted::{ARCHIVE_TYPES, Published};

pub const ID: &str = "vagrant";
pub const BOX: &str = "box";
pub const METADATA: &str = "metadata";
pub const JSON_TYPE: &str = "application/json";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "vagrant.provider",
        target: SearchTarget::Asset,
        pointer: &["vagrant", "provider"],
    },
    SearchKey {
        param: "vagrant.architecture",
        target: SearchTarget::Asset,
        pointer: &["vagrant", "architecture"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["vagrant", "description"]];

#[derive(Default)]
pub struct Vagrant;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Authenticate,
    Metadata(BoxName),
    File(BoxFile),
    Other,
}

impl Route {
    fn is_api(&self) -> bool {
        matches!(self, Route::Authenticate | Route::Metadata(_))
    }
}

fn route(path: &str) -> Route {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    if trimmed.ends_with('/') {
        return Route::Other;
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    let metadata = |org: &str, name: &str| {
        BoxName::new(org, name)
            .map(Route::Metadata)
            .unwrap_or(Route::Other)
    };
    match parts.as_slice() {
        ["api", "v1" | "v2", "authenticate"] => Route::Authenticate,
        ["api", "v2", "vagrant", org, name] => metadata(org, name),
        [org, name] | [org, name, "metadata.json"] => metadata(org, name),
        [_, _, _, _, _] => BoxFile::parse(path)
            .map(Route::File)
            .unwrap_or(Route::Other),
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Vagrant repository of boxes. Set <code>VAGRANT_SERVER_URL={url}</code> to add boxes by their short name, for example <code>vagrant box add &lt;org&gt;/&lt;name&gt;</code>; the metadata of a box is at <code>{url}/&lt;org&gt;/&lt;name&gt;</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn json_response(status: StatusCode, value: &Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .body(Body::from(value.to_string()))
        .expect("a JSON response is valid")
}

fn api_error(ctx: &Ctx, error: ContentError) -> Response {
    let status = error.status();
    let message = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(error = format!("{inner:#}"), "a vagrant request failed");
            "the server failed; see its log".to_owned()
        }
        ContentError::Unauthorized => format!(
            "sign in with an access token of Astrum Repository: create one at {}/user/access-tokens and give it to Vagrant as VAGRANT_CLOUD_TOKEN",
            ctx.base_url
        ),
        other => other.to_string(),
    };
    let mut response = json_response(status, &json!({"errors": [message], "success": false}));
    let headers = response.headers_mut();
    match &error {
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

fn etagged(request: &ContentRequest, value: &Value) -> Response {
    use sha2::{Digest, Sha256};
    let body = value.to_string().into_bytes();
    let etag = format!("\"{}\"", hex::encode(&Sha256::digest(&body)[..16]));
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    let response = match (unchanged, &request.method) {
        (true, _) => builder.status(StatusCode::NOT_MODIFIED).body(Body::empty()),
        (false, &Method::HEAD) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, JSON_TYPE)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        (false, _) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, JSON_TYPE)
            .body(Body::from(body)),
    };
    response.expect("a JSON response is valid")
}

fn may_read(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Read).any() {
        true => Ok(()),
        false => ctx.require(View::Read, "/"),
    }
}

fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
            RepositoryType::Proxy => "GET,HEAD,DELETE",
            RepositoryType::Group => "GET,HEAD",
        },
    }
}

fn not_hosted(ctx: &Ctx) -> ContentError {
    ContentError::BadRequest(format!(
        "{} is a {} repository; upload boxes to a hosted Vagrant repository",
        ctx.name(),
        ctx.kind()
    ))
}

fn unknown_path() -> ContentError {
    ContentError::BadRequest(
        "upload a box to /<org>/<name>/<version>/<provider>/<architecture>.box".to_owned(),
    )
}

fn authenticated(ctx: &Ctx) -> Result<Response, ContentError> {
    match &ctx.subject {
        Some(subject) if !subject.anonymous => Ok(json_response(
            StatusCode::OK,
            &json!({"user": {"username": subject.user_id}}),
        )),
        _ => Err(ContentError::Unauthorized),
    }
}

fn sign_in_by_token(ctx: &Ctx) -> ContentError {
    ContentError::BadRequest(format!(
        "Astrum Repository does not create tokens for user names and passwords; create an access token at {}/user/access-tokens and log in with vagrant cloud auth login --token <token>",
        ctx.base_url
    ))
}

impl Vagrant {
    async fn catalog(&self, ctx: &Ctx, boxed: &BoxName) -> Result<Option<Catalog>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::catalog(ctx, boxed).await,
            RepositoryType::Proxy => remote::catalog(ctx, boxed).await,
            RepositoryType::Group => {
                let mut catalogs = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.catalog(&member, boxed)).await {
                        Ok(Some(found)) => catalogs.push(found),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no box metadata");
                        }
                    }
                }
                Ok(Catalog::merge(boxed, catalogs))
            }
        }
    }

    async fn visible_catalog(&self, ctx: &Ctx, boxed: &BoxName) -> Result<Catalog, ContentError> {
        may_read(ctx)?;
        let mut found = self.catalog(ctx, boxed).await?;
        if let Some(catalog) = &mut found
            && !matches!(ctx.grant(View::Read), Grant::Whole)
        {
            catalog.retain(|file| ctx.allowed(View::Read, &file.path()));
        }
        found
            .filter(|catalog| !catalog.is_empty())
            .ok_or_else(|| ContentError::not_found(format!("box {} was not found", boxed.tag())))
    }

    async fn box_file(
        &self,
        ctx: &Ctx,
        file: &BoxFile,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(store::find(ctx.db(), ctx.name(), &file.path())
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => Ok(remote::box_file(ctx, file)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.box_file(&member, file)).await {
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

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        file: &BoxFile,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &file.path())?;
        let (owner, found) = self.box_file(ctx, file).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "version {} of {} has no {} box for {}",
                file.version,
                file.boxed.tag(),
                file.provider,
                file.architecture
            ))
        })?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::download().disposition("attachment"),
        )
        .await
    }

    async fn put(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        file: &BoxFile,
    ) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_allowed(&request.method, ctx.kind()));
        }
        let published = hosted::publish(ctx, file, None, request.into_stream()).await?;
        Ok(json_response(
            StatusCode::CREATED,
            &json!({
                "name": published.file.boxed.tag(),
                "version": published.file.version,
                "provider": published.file.provider,
                "architecture": published.file.architecture,
                "checksum_type": "sha256",
                "checksum": published.sha256,
                "size": published.size,
                "url": format!("{}{}", ctx.repository_url(), published.file.path()),
            }),
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
}

#[async_trait]
impl Format for Vagrant {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let api = route.is_api() || request.method == Method::PUT;
        let method = request.method.clone();
        let result = match (&method, route) {
            (&Method::GET | &Method::HEAD, Route::Root) => directory_page(&ctx),
            (&Method::GET | &Method::HEAD, Route::Authenticate) => authenticated(&ctx),
            (&Method::POST, Route::Authenticate) => Err(sign_in_by_token(&ctx)),
            (&Method::GET | &Method::HEAD, Route::Metadata(boxed)) => self
                .visible_catalog(&ctx, &boxed)
                .await
                .map(|found| etagged(&request, &found.render(&ctx.repository_url()))),
            (&Method::GET | &Method::HEAD, Route::File(file)) => {
                self.download(&ctx, &request, &file).await
            }
            (&Method::GET | &Method::HEAD, Route::Other) => {
                Err(ContentError::not_found(request.path.clone()))
            }
            (&Method::PUT, Route::File(file)) => self.put(&ctx, request, &file).await,
            (&Method::PUT, _) if ctx.kind() == RepositoryType::Hosted => Err(unknown_path()),
            (&Method::DELETE, Route::File(file)) if ctx.kind() != RepositoryType::Group => {
                self.delete(&ctx, &file.path()).await
            }
            (&Method::DELETE, Route::Metadata(boxed)) if ctx.kind() == RepositoryType::Proxy => {
                self.delete(&ctx, &boxed.metadata_path()).await
            }
            (method, _) => Err(not_allowed(method, ctx.kind())),
        };
        match result {
            Ok(response) => Ok(response),
            Err(error) if api => Ok(api_error(&ctx, error)),
            Err(error) => Ok(error.into_response()),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::File(file) => Ok(self.box_file(ctx, &file).await?.map(|(_, found)| found)),
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
        let field = |name, optional, description| {
            UploadField::text(name, optional)
                .describe(description)
                .in_group("Box")
        };
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![
                field(
                    "organization",
                    false,
                    "Organization of the box, the first part of a name like acme/base",
                ),
                field(
                    "name",
                    false,
                    "Name of the box, the second part of acme/base",
                ),
                field(
                    "version",
                    false,
                    "Version of the box, for example 1.2.0; Vagrant compares versions like RubyGems",
                ),
                field(
                    "provider",
                    false,
                    "Provider the box is for, for example virtualbox, libvirt or vmware_desktop",
                ),
                UploadField {
                    options: Some(ARCHITECTURES.to_vec()),
                    ..field(
                        "architecture",
                        false,
                        "Architecture of the box, for example amd64 or arm64",
                    )
                },
                field(
                    "description",
                    true,
                    "Description of this version, shown in the box metadata",
                ),
            ],
            asset_fields: vec![UploadField::file("asset")],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(&ctx));
        }
        hosted::may_add(&ctx)?;
        let value = |name: &str| {
            form.field(&format!("{ID}.{name}"))
                .or_else(|| form.field(name))
                .map(str::trim)
        };
        let required = |name: &str| {
            value(name).ok_or_else(|| ContentError::BadRequest(format!("{ID}.{name} is required")))
        };
        let file = form
            .files
            .get(&format!("{ID}.asset"))
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest(format!("{ID}.asset is required")))?;
        let organization = required("organization")?;
        let name = required("name")?;
        let boxed = BoxName::new(organization, name).ok_or_else(|| {
            ContentError::BadRequest(format!(
                "{organization}/{name} is not a box name; use letters, digits, '-', '_' and '.'"
            ))
        })?;
        let version = required("version")?;
        let provider = required("provider")?;
        let architecture = required("architecture")?;
        hosted::check_architecture(architecture)?;
        let target = BoxFile::new(boxed, version, provider, architecture).ok_or_else(|| {
            ContentError::BadRequest(format!(
                "version {version} or provider {provider} is not valid for a Vagrant box"
            ))
        })?;
        hosted::publish(
            &ctx,
            &target,
            value("description"),
            UploadForm::stream(file).await?,
        )
        .await
        .map(|_| ())
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

    fn boxed() -> BoxName {
        BoxName::new("acme", "base").unwrap()
    }

    #[test]
    fn paths_route_to_metadata_files_and_the_api() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/acme/base"), Route::Metadata(boxed()));
        assert_eq!(route("/acme/base/metadata.json"), Route::Metadata(boxed()));
        assert_eq!(route("/api/v2/vagrant/acme/base"), Route::Metadata(boxed()));
        assert_eq!(route("/api/v2/authenticate"), Route::Authenticate);
        assert_eq!(route("/api/v1/authenticate"), Route::Authenticate);
        assert_eq!(
            route("/acme/base/1.0.0/virtualbox/amd64.box"),
            Route::File(BoxFile::new(boxed(), "1.0.0", "virtualbox", "amd64").unwrap())
        );
        for other in [
            "/acme/base/",
            "/acme",
            "/acme/base/1.0.0",
            "/acme/base/1.0.0/virtualbox/amd64.tar",
            "/acme/base/v1/virtualbox/amd64.box",
            "/.acme/base",
            "/api/v2/box/acme/base",
            "/acme/base/other.json",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        assert!(route("/acme/base").is_api());
        assert!(!route("/acme/base/1.0.0/virtualbox/amd64.box").is_api());
    }

    #[test]
    fn archives_are_recognised_by_their_first_bytes() {
        let detected = |sample: &[u8]| astrum_content::mime::detect(sample).unwrap();
        let mut tar = vec![0u8; 512];
        tar[257..262].copy_from_slice(b"ustar");
        for (sample, expected) in [
            (
                b"\x1f\x8b\x08\x00\x00\x00\x00\x00".to_vec(),
                "application/gzip",
            ),
            (tar, "application/x-tar"),
            (b"PK\x03\x04\x14\x00\x00\x00".to_vec(), "application/zip"),
            (b"BZh91AY&SY".to_vec(), "application/x-bzip2"),
            (b"\xfd7zXZ\x00\x00\x04".to_vec(), "application/x-xz"),
            (b"\x28\xb5\x2f\xfd\x04\x00".to_vec(), "application/zstd"),
        ] {
            let found = detected(&sample);
            assert_eq!(found, expected);
            assert!(ARCHIVE_TYPES.contains(&found.as_str()));
        }
        assert!(!ARCHIVE_TYPES.contains(&detected(b"plain text").as_str()));
    }
}
