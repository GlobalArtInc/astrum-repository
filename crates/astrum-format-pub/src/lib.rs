pub mod archive;
mod hosted;
pub mod listing;
mod remote;
pub mod version;

use std::io;

use astrum_blob::ContentStream;
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, CHALLENGE, ContentError, ContentRequest, Ctx, Format, Serve, UploadField,
    UploadForm, UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use futures_util::TryStreamExt;
use serde_json::{Value, json};

use crate::listing::{Listing, archive_path};

pub use hosted::Published;

pub const ID: &str = "pub";
pub const ARCHIVE: &str = "archive";
pub const METADATA: &str = "metadata";
pub const ARCHIVE_TYPE: &str = "application/gzip";
pub const V2_TYPE: &str = "application/vnd.pub.v2+json";
pub const ACCEPT_V2: &str = "application/vnd.pub.v2+json";
pub const MAX_ARCHIVE: u64 = 100 * 1024 * 1024;
const MAX_OPTIONS: usize = 64 * 1024;
const MAX_FIELD: u64 = 64 * 1024;
const FILE_FIELD: &str = "file";
const UPLOAD_FIELD: &str = "pub.asset";

#[derive(Default)]
pub struct Pub;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Directory,
    Listing(String),
    Version(String, String),
    Advisories(String),
    PackageOptions(String),
    VersionOptions(String, String),
    NewUpload,
    Upload,
    Finalize,
    Completion,
    Archive(String, String),
    Other,
}

fn named(name: &str) -> bool {
    archive::valid_name(name)
}

fn versioned(name: &str, version: &str) -> bool {
    named(name) && version::valid(version)
}

fn route(path: &str) -> Route {
    if path == "/" || path.is_empty() {
        return Route::Directory;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["api", "packages", "versions", "new"] => Route::NewUpload,
        ["api", "packages", "versions", "upload"] => Route::Upload,
        ["api", "packages", "versions", "finalize"] => Route::Finalize,
        ["api", "package-name-completion-data"] => Route::Completion,
        ["api", "packages", name] if named(name) => Route::Listing((*name).to_owned()),
        ["api", "packages", name, "advisories"] if named(name) => {
            Route::Advisories((*name).to_owned())
        }
        ["api", "packages", name, "options"] if named(name) => {
            Route::PackageOptions((*name).to_owned())
        }
        ["api", "packages", name, "versions", version] if versioned(name, version) => {
            Route::Version((*name).to_owned(), (*version).to_owned())
        }
        ["api", "packages", name, "versions", version, "options"] if versioned(name, version) => {
            Route::VersionOptions((*name).to_owned(), (*version).to_owned())
        }
        ["packages", name, "versions", file] => match file.strip_suffix(".tar.gz") {
            Some(version) if versioned(name, version) => {
                Route::Archive((*name).to_owned(), version.to_owned())
            }
            _ => Route::Other,
        },
        ["api", "archives", file] => match file
            .strip_suffix(".tar.gz")
            .and_then(|stem| stem.split_once('-'))
        {
            Some((name, version)) if versioned(name, version) => {
                Route::Archive(name.to_owned(), version.to_owned())
            }
            _ => Route::Other,
        },
        _ => Route::Other,
    }
}

fn json_v2(status: StatusCode, value: &Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, V2_TYPE)
        .body(Body::from(value.to_string()))
        .expect("a JSON response is valid")
}

fn quoted(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .map(|c| match c {
            '"' | '\\' => '\'',
            other => other,
        })
        .collect()
}

fn bearer_challenge(message: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(&format!(
        "Bearer realm=\"pub\", message=\"{}\"",
        quoted(message)
    ))
    .ok()
}

fn sign_in_message(ctx: &Ctx) -> String {
    format!(
        "Create an access token in Astrum Repository at {}/user/access-tokens and add it with: dart pub token add {}",
        ctx.base_url,
        ctx.repository_url()
    )
}

fn permission_message(ctx: &Ctx) -> String {
    format!(
        "Your user lacks the privileges this needs on the repository {}; ask an administrator of Astrum Repository for them",
        ctx.name()
    )
}

fn code_of(error: &ContentError) -> &'static str {
    match error {
        ContentError::NotFound(_) => "NotFound",
        ContentError::Unauthorized => "MissingAuthentication",
        ContentError::Forbidden => "InsufficientPermissions",
        ContentError::MethodNotAllowed { .. } => "MethodNotAllowed",
        ContentError::BadRequest(_) => "InvalidInput",
        ContentError::Conflict(_) => "PackageRejected",
        ContentError::PreconditionFailed => "PreconditionFailed",
        ContentError::RangeNotSatisfiable(_) => "RangeNotSatisfiable",
        ContentError::Unavailable(_) | ContentError::BadGateway(_) => "ServiceUnavailable",
        ContentError::Internal(_) => "InternalError",
    }
}

fn api_error(ctx: &Ctx, error: ContentError) -> Response {
    let status = error.status();
    let message = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(error = format!("{inner:#}"), "a pub request failed");
            "the server failed; see its log".to_owned()
        }
        ContentError::Unauthorized => sign_in_message(ctx),
        ContentError::Forbidden => permission_message(ctx),
        other => other.to_string(),
    };
    let mut response = json_v2(
        status,
        &json!({"error": {"code": code_of(&error), "message": message}}),
    );
    let headers = response.headers_mut();
    match &error {
        ContentError::Unauthorized => {
            if let Some(value) = bearer_challenge(&message) {
                headers.append(header::WWW_AUTHENTICATE, value);
            }
            headers.append(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(CHALLENGE),
            );
        }
        ContentError::Forbidden => {
            if let Some(value) = bearer_challenge(&message) {
                headers.append(header::WWW_AUTHENTICATE, value);
            }
        }
        ContentError::MethodNotAllowed { allow, .. } => {
            headers.insert(header::ALLOW, HeaderValue::from_static(allow));
        }
        _ => {}
    }
    response
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
        _ if ctx.is_trusted() => Ok(()),
        _ => Err(ContentError::Unauthorized),
    }
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
            .header(header::CONTENT_TYPE, V2_TYPE)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        (false, _) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, V2_TYPE)
            .body(Body::from(body)),
    };
    response.expect("a JSON response is valid")
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
            "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a pub repository for Dart and Flutter packages. Use <code>{url}</code> as the hosted URL of a dependency in <code>pubspec.yaml</code> or as <code>PUB_HOSTED_URL</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
            escape(&ctx.base_url),
        )))
        .map_err(ContentError::internal)
}

fn upload_url(ctx: &Ctx) -> String {
    format!("{}/api/packages/versions/upload", ctx.repository_url())
}

fn finalize_url(ctx: &Ctx, outcome: &Value) -> String {
    let sealed = ctx.content.secrets().seal(&outcome.to_string());
    let query: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("upload", &sealed)
        .finish();
    format!(
        "{}/api/packages/versions/finalize?{query}",
        ctx.repository_url()
    )
}

fn accepted(ctx: &Ctx, outcome: Value) -> Response {
    let location = finalize_url(ctx, &outcome);
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(header::LOCATION, location)
        .body(Body::empty())
        .expect("an upload answer is valid")
}

fn published_message(ctx: &Ctx, published: &Published) -> String {
    match published.unchanged {
        true => format!(
            "Package {} version {} is already in {} with the same archive.",
            published.name,
            published.version,
            ctx.name()
        ),
        false => format!(
            "Successfully uploaded package {} version {} to {}.",
            published.name,
            published.version,
            ctx.name()
        ),
    }
}

fn not_hosted(ctx: &Ctx) -> ContentError {
    ContentError::BadRequest(format!(
        "{} is a {} repository; publish packages to a hosted pub repository",
        ctx.name(),
        ctx.kind()
    ))
}

fn read_only(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => "GET,HEAD,POST,PUT,DELETE",
            RepositoryType::Proxy => "GET,HEAD,DELETE",
            RepositoryType::Group => "GET,HEAD",
        },
    }
}

impl Pub {
    async fn listing(&self, ctx: &Ctx, name: &str) -> Result<Option<Listing>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::listing(ctx, name).await,
            RepositoryType::Proxy => remote::listing(ctx, name).await,
            RepositoryType::Group => {
                let mut listings = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.listing(&member, name)).await {
                        Ok(Some(found)) => listings.push(found),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no listing");
                        }
                    }
                }
                Ok(Listing::merge(name, listings))
            }
        }
    }

    async fn visible_listing(&self, ctx: &Ctx, name: &str) -> Result<Listing, ContentError> {
        may_read(ctx)?;
        self.listing(ctx, name)
            .await?
            .and_then(|found| hosted::visible(ctx, found))
            .ok_or_else(|| ContentError::not_found(format!("package \"{name}\" was not found")))
    }

    async fn archive(
        &self,
        ctx: &Ctx,
        name: &str,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                Ok(
                    store::find(ctx.db(), ctx.name(), &archive_path(name, version))
                        .await?
                        .map(|found| (ctx.clone(), found)),
                )
            }
            RepositoryType::Proxy => Ok(remote::archive(ctx, name, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.archive(&member, name, version)).await {
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
        name: &str,
        version: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &archive_path(name, version))?;
        let (owner, found) = self.archive(ctx, name, version).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "version {version} of package \"{name}\" was not found"
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

    async fn advisories(&self, ctx: &Ctx, name: &str) -> Result<Value, ContentError> {
        match ctx.kind() {
            RepositoryType::Proxy => Ok(remote::advisories(ctx, name)
                .await?
                .unwrap_or_else(|| json!({"advisories": []}))),
            RepositoryType::Hosted => Ok(json!({"advisories": []})),
            RepositoryType::Group => {
                let mut advisories: Vec<Value> = Vec::new();
                let mut updated: Option<String> = None;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    let found = match Box::pin(self.advisories(&member, name)).await {
                        Ok(found) => found,
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no advisories");
                            continue;
                        }
                    };
                    if let Some(when) = found.get("advisoriesUpdated").and_then(Value::as_str) {
                        updated = Some(match updated {
                            Some(known) if known.as_str() >= when => known,
                            _ => when.to_owned(),
                        });
                    }
                    for advisory in found
                        .get("advisories")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        let id = advisory.get("id");
                        if id.is_none() || !advisories.iter().any(|known| known.get("id") == id) {
                            advisories.push(advisory.clone());
                        }
                    }
                }
                let mut document = json!({"advisories": advisories});
                if let Some(updated) = updated {
                    document["advisoriesUpdated"] = json!(updated);
                }
                Ok(document)
            }
        }
    }

    async fn names(&self, ctx: &Ctx) -> Result<Vec<String>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let names = hosted::names(ctx).await?;
                if matches!(ctx.grant(View::Read), astrum_content::Grant::Whole) {
                    return Ok(names);
                }
                let mut visible = Vec::new();
                for name in names {
                    if let Some(found) = hosted::listing(ctx, &name).await?
                        && hosted::visible(ctx, found).is_some()
                    {
                        visible.push(name);
                    }
                }
                Ok(visible)
            }
            RepositoryType::Proxy => Ok(remote::completion(ctx)
                .await?
                .and_then(|document| document.get("packages").cloned())
                .and_then(|packages| serde_json::from_value::<Vec<String>>(packages).ok())
                .unwrap_or_default()),
            RepositoryType::Group => {
                let mut lists = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.names(&member)).await {
                        Ok(names) => lists.push(names),
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no package names");
                        }
                    }
                }
                Ok(hosted::unique_names(lists))
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        match route(&request.path) {
            Route::Directory => directory_page(ctx),
            Route::Listing(name) => {
                let found = self.visible_listing(ctx, &name).await?;
                Ok(etagged(request, &found.render(&ctx.repository_url())))
            }
            Route::Version(name, version) => {
                let found = self.visible_listing(ctx, &name).await?;
                let document = found
                    .render_version(&ctx.repository_url(), &version)
                    .ok_or_else(|| {
                        ContentError::not_found(format!(
                            "version {version} of package \"{name}\" was not found"
                        ))
                    })?;
                Ok(etagged(request, &document))
            }
            Route::Advisories(name) => {
                may_read(ctx)?;
                Ok(etagged(request, &self.advisories(ctx, &name).await?))
            }
            Route::Completion => {
                may_read(ctx)?;
                Ok(etagged(
                    request,
                    &json!({"packages": self.names(ctx).await?}),
                ))
            }
            Route::PackageOptions(name) => {
                let found = self.visible_listing(ctx, &name).await?;
                Ok(json_v2(
                    StatusCode::OK,
                    &json!({
                        "isDiscontinued": found.discontinued,
                        "replacedBy": found.replaced_by,
                        "isUnlisted": false,
                    }),
                ))
            }
            Route::VersionOptions(name, version) => {
                let found = self.visible_listing(ctx, &name).await?;
                let entry = found.find(&version).ok_or_else(|| {
                    ContentError::not_found(format!(
                        "version {version} of package \"{name}\" was not found"
                    ))
                })?;
                Ok(json_v2(
                    StatusCode::OK,
                    &json!({"isRetracted": entry.retracted}),
                ))
            }
            Route::NewUpload => self.new_upload(ctx),
            Route::Finalize => self.finalize(ctx, request),
            Route::Archive(name, version) => self.download(ctx, request, &name, &version).await,
            Route::Upload | Route::Other => Err(ContentError::not_found(request.path.clone())),
        }
    }

    fn new_upload(&self, ctx: &Ctx) -> Result<Response, ContentError> {
        signed_in(ctx)?;
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(ctx));
        }
        hosted::may_add(ctx)?;
        Ok(json_v2(
            StatusCode::OK,
            &json!({"url": upload_url(ctx), "fields": {}}),
        ))
    }

    fn finalize(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let sealed = url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
            .find(|(key, _)| key == "upload")
            .map(|(_, value)| value.into_owned())
            .unwrap_or_default();
        let outcome = ctx
            .content
            .secrets()
            .open(&sealed)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .filter(|outcome| outcome["repository"] == ctx.name());
        let Some(outcome) = outcome else {
            return Err(ContentError::BadRequest(
                "this upload is not known to the repository; run dart pub publish again".to_owned(),
            ));
        };
        let message = outcome["message"].as_str().unwrap_or_default();
        match outcome["ok"].as_bool() {
            Some(true) => Ok(json_v2(
                StatusCode::OK,
                &json!({"success": {"message": message}}),
            )),
            _ => Ok(json_v2(
                StatusCode::BAD_REQUEST,
                &json!({"error": {"code": outcome["code"].as_str().unwrap_or("PackageRejected"), "message": message}}),
            )),
        }
    }

    async fn upload(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        signed_in(ctx)?;
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_hosted(ctx));
        }
        hosted::may_add(ctx)?;
        let rejected = |message: String| {
            accepted(
                ctx,
                json!({"repository": ctx.name(), "ok": false, "code": "PackageRejected", "message": message}),
            )
        };
        if request
            .header("content-length")
            .and_then(|value| value.parse::<u64>().ok())
            .is_some_and(|length| length > MAX_ARCHIVE + MAX_FIELD * 16)
        {
            return Ok(rejected(format!(
                "the package archive is larger than {MAX_ARCHIVE} bytes"
            )));
        }
        let Some(boundary) = request
            .header("content-type")
            .and_then(|value| multer::parse_boundary(value).ok())
        else {
            return Err(ContentError::BadRequest(
                "the upload must be a multipart/form-data request with a file field".to_owned(),
            ));
        };
        let constraints = multer::Constraints::new().size_limit(
            multer::SizeLimit::new()
                .per_field(MAX_FIELD)
                .for_field(FILE_FIELD, MAX_ARCHIVE),
        );
        let mut form = multer::Multipart::with_constraints(
            request.take_body().into_data_stream(),
            boundary,
            constraints,
        );
        let bad = |error: multer::Error| match error {
            multer::Error::FieldSizeExceeded { .. } | multer::Error::StreamSizeExceeded { .. } => {
                ContentError::BadRequest(format!(
                    "the package archive is larger than {MAX_ARCHIVE} bytes"
                ))
            }
            other => ContentError::BadRequest(format!("the upload form cannot be read: {other}")),
        };
        let mut outcome = None;
        loop {
            let field = match form.next_field().await {
                Ok(Some(field)) => field,
                Ok(None) => break,
                Err(error) => return Ok(rejected(bad(error).to_string())),
            };
            if field.name() != Some(FILE_FIELD) || outcome.is_some() {
                let mut field = field;
                loop {
                    match field.chunk().await {
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(error) => return Ok(rejected(bad(error).to_string())),
                    }
                }
                continue;
            }
            let body: ContentStream = Box::pin(field.map_err(io::Error::other));
            let written = match store::write_blob(ctx, body).await {
                Ok(written) => written,
                Err(error) => {
                    tracing::info!(repository = ctx.name(), %error, "a pub upload could not be stored");
                    return Ok(rejected(
                        "the package archive could not be received; check its size and publish again"
                            .to_owned(),
                    ));
                }
            };
            outcome = Some(hosted::adopt(ctx, written).await);
        }
        match outcome {
            None => Ok(rejected(
                "the upload form has no file field with the package archive".to_owned(),
            )),
            Some(Ok(published)) => Ok(accepted(
                ctx,
                json!({"repository": ctx.name(), "ok": true, "message": published_message(ctx, &published)}),
            )),
            Some(Err(
                error @ (ContentError::Unauthorized
                | ContentError::Forbidden
                | ContentError::Internal(_)),
            )) => Err(error),
            Some(Err(error)) => Ok(rejected(error.to_string())),
        }
    }

    async fn options(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        name: &str,
        version: Option<&str>,
    ) -> Result<Response, ContentError> {
        signed_in(ctx)?;
        if ctx.kind() != RepositoryType::Hosted {
            return Err(read_only(&request.method, ctx.kind()));
        }
        let body = request.bytes(MAX_OPTIONS).await?;
        let given: Value = serde_json::from_slice(&body)
            .map_err(|error| ContentError::BadRequest(format!("the body is not JSON: {error}")))?;
        match version {
            Some(version) => {
                let retracted = given
                    .get("isRetracted")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        ContentError::BadRequest("give isRetracted as true or false".to_owned())
                    })?;
                hosted::set_retracted(ctx, name, version, retracted).await?;
                Ok(json_v2(StatusCode::OK, &json!({"isRetracted": retracted})))
            }
            None => {
                let discontinued = given
                    .get("isDiscontinued")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| {
                        ContentError::BadRequest("give isDiscontinued as true or false".to_owned())
                    })?;
                let replaced_by = match given.get("replacedBy") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(text)) if text.trim().is_empty() => None,
                    Some(Value::String(text))
                        if archive::valid_name(text.trim()) && text.trim() != name =>
                    {
                        Some(text.trim().to_owned())
                    }
                    Some(_) => {
                        return Err(ContentError::BadRequest(
                            "replacedBy must name another package".to_owned(),
                        ));
                    }
                };
                let flags = hosted::Flags {
                    discontinued,
                    replaced_by: replaced_by.filter(|_| discontinued),
                };
                hosted::set_flags(ctx, name, &flags).await?;
                Ok(json_v2(
                    StatusCode::OK,
                    &json!({
                        "isDiscontinued": flags.discontinued,
                        "replacedBy": flags.replaced_by,
                        "isUnlisted": false,
                    }),
                ))
            }
        }
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
impl Format for Pub {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let method = request.method.clone();
        let answer = match (&method, &route) {
            (&Method::GET | &Method::HEAD, Route::Directory) => return directory_page(&ctx),
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::POST, Route::Upload) => self.upload(&ctx, request).await,
            (&Method::PUT, Route::PackageOptions(name)) => {
                self.options(&ctx, request, name, None).await
            }
            (&Method::PUT, Route::VersionOptions(name, version)) => {
                self.options(&ctx, request, name, Some(version)).await
            }
            (&Method::DELETE, Route::Archive(name, version))
                if ctx.kind() != RepositoryType::Group =>
            {
                self.delete(&ctx, &archive_path(name, version)).await
            }
            (&Method::DELETE, Route::Listing(name)) if ctx.kind() == RepositoryType::Proxy => {
                self.delete(&ctx, &remote::listing_path(name)).await
            }
            (&Method::DELETE, Route::Advisories(name)) if ctx.kind() == RepositoryType::Proxy => {
                self.delete(&ctx, &remote::advisories_path(name)).await
            }
            (&Method::DELETE, Route::Completion) if ctx.kind() == RepositoryType::Proxy => {
                self.delete(&ctx, remote::COMPLETION_PATH).await
            }
            (method, _) => Err(read_only(method, ctx.kind())),
        };
        Ok(answer.unwrap_or_else(|error| api_error(&ctx, error)))
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Archive(name, version) => Ok(self
                .archive(ctx, &name, &version)
                .await?
                .map(|(_, found)| found)),
            _ => match ctx.kind() {
                RepositoryType::Group => {
                    Ok(
                        group::first(ctx, |member| async move { self.get(&member, path).await })
                            .await?
                            .map(|(_, found)| found),
                    )
                }
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
        hosted::may_add(&ctx)?;
        if file.size > MAX_ARCHIVE {
            return Err(ContentError::BadRequest(format!(
                "the package archive is larger than {MAX_ARCHIVE} bytes"
            )));
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::adopt(&ctx, written).await.map(|_| ())
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        &[&["pub", "description"]]
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE '/' || c.name || '/' || c.version || '/' || c.name || '-' || c.version || '.tar.gz' END",
            ),
            generated: &[],
        }
    }
}

pub mod testing {
    pub fn archive(pubspec: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut all: Vec<(&str, &[u8])> = vec![("pubspec.yaml", pubspec.as_bytes())];
        all.extend_from_slice(files);
        crate::archive::build(&all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_api_and_the_archives() {
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(
            route("/api/packages/hello"),
            Route::Listing("hello".to_owned())
        );
        assert_eq!(
            route("/api/packages/hello/versions/1.0.0+2"),
            Route::Version("hello".to_owned(), "1.0.0+2".to_owned())
        );
        assert_eq!(
            route("/api/packages/hello/advisories"),
            Route::Advisories("hello".to_owned())
        );
        assert_eq!(
            route("/api/packages/hello/options"),
            Route::PackageOptions("hello".to_owned())
        );
        assert_eq!(
            route("/api/packages/hello/versions/1.0.0-dev.1/options"),
            Route::VersionOptions("hello".to_owned(), "1.0.0-dev.1".to_owned())
        );
        assert_eq!(route("/api/packages/versions/new"), Route::NewUpload);
        assert_eq!(route("/api/packages/versions/upload"), Route::Upload);
        assert_eq!(route("/api/packages/versions/finalize"), Route::Finalize);
        assert_eq!(
            route("/api/packages/versions"),
            Route::Listing("versions".to_owned())
        );
        assert_eq!(
            route("/api/package-name-completion-data"),
            Route::Completion
        );
        assert_eq!(
            route("/packages/hello/versions/1.2.3-beta.1.tar.gz"),
            Route::Archive("hello".to_owned(), "1.2.3-beta.1".to_owned())
        );
        assert_eq!(
            route("/api/archives/hello_world-1.2.3-beta.1.tar.gz"),
            Route::Archive("hello_world".to_owned(), "1.2.3-beta.1".to_owned())
        );
        for other in [
            "/api/packages/hello-world",
            "/api/packages/hello/versions/1.0",
            "/packages/hello/versions/1.0.0.zip",
            "/packages/hello/1.0.0.tar.gz",
            "/api/archives/hello.tar.gz",
            "/api/packages/hello/score",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }

    #[test]
    fn challenges_quote_the_message_safely() {
        let value = bearer_challenge("say \"hi\"\nnow").unwrap();
        assert_eq!(
            value.to_str().unwrap(),
            "Bearer realm=\"pub\", message=\"say 'hi'now\""
        );
    }
}
