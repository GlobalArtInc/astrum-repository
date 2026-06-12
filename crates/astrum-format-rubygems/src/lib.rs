pub mod gem;
mod group;
mod hosted;
pub mod import;
pub mod index;
mod marshal;
pub mod testing;

use std::collections::HashMap;

use astrum_blob::{BlobId, Digests, Written};
use astrum_content::format::SearchTarget;
use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, BrowseLayout, CHALLENGE, ContentError, ContentRequest, Ctx, Format, Freshness,
    SearchKey, Serve, UploadField, UploadForm, UploadSpec, View, group as members, serve,
};
use astrum_repo::RepositoryType;
use astrum_security::api_keys;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use futures_util::TryStreamExt;
use serde_json::json;

use crate::gem::{GemError, Spec};
use crate::index::{InfoEntry, Specs};

pub const ID: &str = "rubygems";
pub const GEM: &str = "GEM";
pub const GEMSPEC: &str = "GEMSPEC";
pub const INDEX: &str = "INDEX";
pub const REALM: &str = api_keys::RUBYGEMS;
const TEXT: &str = "text/plain; charset=utf-8";
const BINARY: &str = "application/octet-stream";
const GEM_TYPE: &str = "application/x-tar";
const MAX_GEMS: usize = 200;
const MAX_INDEX: u64 = 256 * 1024 * 1024;
const MAX_FORM: usize = 64 * 1024;
const API_KEY: &str = "/api/v1/api_key";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "rubygems.platform",
        target: SearchTarget::Asset,
        pointer: &["rubygems", "platform"],
    },
    SearchKey {
        param: "rubygems.summary",
        target: SearchTarget::Asset,
        pointer: &["rubygems", "summary"],
    },
    SearchKey {
        param: "rubygems.description",
        target: SearchTarget::Asset,
        pointer: &["rubygems", "description"],
    },
];

#[derive(Default)]
pub struct Rubygems;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Versions,
    Names,
    Info(String),
    Specs(Specs),
    Quick(String),
    Gem(String),
    Dependencies,
    Push,
    Yank,
    ApiKey,
    Directory,
    Other,
}

impl Route {
    fn path(&self) -> String {
        match self {
            Route::Versions => "/versions".to_owned(),
            Route::Names => "/names".to_owned(),
            Route::Info(name) => format!("/info/{name}"),
            Route::Specs(kind) => kind.path().to_owned(),
            Route::Quick(full) => format!("{}{full}.gemspec.rz", gem::QUICK_PREFIX),
            Route::Gem(full) => gem::gem_path(full),
            Route::Dependencies => "/api/v1/dependencies".to_owned(),
            Route::Push => "/api/v1/gems".to_owned(),
            Route::Yank => "/api/v1/gems/yank".to_owned(),
            Route::ApiKey => API_KEY.to_owned(),
            Route::Directory | Route::Other => "/".to_owned(),
        }
    }

    fn cached(&self) -> Option<(&'static str, Freshness, &'static str)> {
        match self {
            Route::Versions | Route::Names | Route::Info(_) => {
                Some((INDEX, Freshness::Metadata, TEXT))
            }
            Route::Specs(_) => Some((INDEX, Freshness::Metadata, BINARY)),
            Route::Quick(_) => Some((GEMSPEC, Freshness::Content, BINARY)),
            Route::Gem(_) => Some((GEM, Freshness::Content, GEM_TYPE)),
            _ => None,
        }
    }
}

fn file_name<'a>(path: &'a str, prefix: &str, extension: &str) -> Option<&'a str> {
    let file = path.strip_prefix(prefix)?;
    if !gem::valid_file_name(file) {
        return None;
    }
    file.strip_suffix(extension).filter(|full| !full.is_empty())
}

fn route(path: &str) -> Route {
    match path {
        "/versions" => return Route::Versions,
        "/names" => return Route::Names,
        "/api/v1/dependencies" => return Route::Dependencies,
        "/api/v1/gems" => return Route::Push,
        "/api/v1/gems/yank" => return Route::Yank,
        API_KEY => return Route::ApiKey,
        _ => {}
    }
    if path.ends_with('/') {
        return Route::Directory;
    }
    if let Some(kind) = Specs::from_path(path) {
        return Route::Specs(kind);
    }
    if let Some(name) = path.strip_prefix("/info/")
        && gem::valid_name(name)
    {
        return Route::Info(name.to_owned());
    }
    if let Some(full) = file_name(path, gem::QUICK_PREFIX, ".gemspec.rz") {
        return Route::Quick(full.to_owned());
    }
    if let Some(full) = file_name(path, gem::GEMS_PREFIX, ".gem") {
        return Route::Gem(full.to_owned());
    }
    Route::Other
}

fn text(status: StatusCode, message: impl Into<String>) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(message.into()))
        .expect("a plain response is valid")
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a RubyGems repository. Use it as a gem source: <code>gem sources --add {url}/</code>, or <code>source \"{url}/\"</code> in a Gemfile.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn unchanged(request: &ContentRequest, etag: &str) -> bool {
    request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    })
}

fn body_response(
    request: &ContentRequest,
    content_type: &str,
    body: Vec<u8>,
) -> Result<Response, ContentError> {
    let etag = format!("\"{}\"", index::md5_hex(&body));
    let builder = Response::builder().header(header::ETAG, &etag);
    if unchanged(request, &etag) {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}

pub(crate) enum Index {
    Built(Vec<u8>),
    Stored(Box<(Ctx, Asset)>),
}

impl Index {
    fn md5(&self) -> String {
        match self {
            Index::Built(bytes) => index::md5_hex(bytes),
            Index::Stored(stored) => stored.1.md5.clone(),
        }
    }

    async fn bytes(self) -> Result<Vec<u8>, ContentError> {
        match self {
            Index::Built(bytes) => Ok(bytes),
            Index::Stored(stored) => read_bytes(&stored.0, &stored.1, MAX_INDEX).await,
        }
    }

    async fn respond(
        self,
        request: &ContentRequest,
        content_type: &str,
    ) -> Result<Response, ContentError> {
        match self {
            Index::Built(bytes) => body_response(request, content_type, bytes),
            Index::Stored(stored) => {
                let (owner, found) = *stored;
                let options = Serve {
                    content_type: Some(content_type.to_owned()),
                    ..Serve::default()
                }
                .etag(found.md5.clone());
                serve(&owner, &found, &request.method, &request.headers, options).await
            }
        }
    }
}

fn spec_of(found: &Asset) -> Option<Spec> {
    Spec::from_attributes(found.attributes.get(ID)?)
}

fn component(spec: &Spec) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: spec.name.clone(),
        version: spec.version.clone(),
        kind: String::new(),
        attributes: json!({}),
    }
}

async fn read_bytes(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
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

pub(crate) async fn read_spec(ctx: &Ctx, store_name: &str, id: BlobId) -> Result<Spec, GemError> {
    let store = ctx
        .content
        .blob_stores()
        .get(store_name)
        .await
        .map_err(|error| GemError::NotGem(error.to_string()))?;
    let content = store
        .read(id)
        .await
        .map_err(|error| GemError::NotGem(error.to_string()))?
        .ok_or_else(|| GemError::NotGem("the stored gem is missing".to_owned()))?;
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|error| GemError::NotGem(error.to_string()))?
    {
        body.extend_from_slice(&chunk);
    }
    tokio::task::spawn_blocking(move || gem::read_gem(body.as_slice()))
        .await
        .map_err(|error| GemError::NotGem(error.to_string()))?
}

pub(crate) async fn adopt(
    ctx: &Ctx,
    spec: &Spec,
    put: PutAsset,
    written: &Written,
) -> Result<Asset, ContentError> {
    let put = put
        .component(component(spec))
        .kind(GEM)
        .content_type(GEM_TYPE)
        .attributes(json!({ ID: spec.attributes() }));
    store::adopt(ctx, put, written).await
}

impl Rubygems {
    pub(crate) async fn index(
        &self,
        ctx: &Ctx,
        route: &Route,
    ) -> Result<Option<Index>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::index(ctx, route).await?.map(Index::Built)),
            RepositoryType::Proxy => Ok(self
                .cache(ctx, &route.path())
                .await?
                .map(|found| Index::Stored(Box::new((ctx.clone(), found))))),
            RepositoryType::Group => group::index(self, ctx, route).await,
        }
    }

    async fn cache(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let Some((kind, freshness, content_type)) = route(path).cached() else {
            return Ok(None);
        };
        let found = proxy::fetch(ctx, path, None, freshness, || {
            PutAsset::cached(path).kind(kind).content_type(content_type)
        })
        .await?;
        let Some(found) = found else {
            return Ok(None);
        };
        if kind != GEM || found.attributes.get(ID).is_some() {
            return Ok(Some(found));
        }
        match read_spec(ctx, &found.blob_store, found.blob_id.into()).await {
            Ok(spec) => {
                if spec.gem_path() != path {
                    tracing::warn!(
                        repository = ctx.name(),
                        path,
                        gem = spec.full_name(),
                        "a cached gem holds another gem"
                    );
                }
                let written = Written {
                    id: found.blob_id.into(),
                    store: found.blob_store.clone(),
                    size: found.size.max(0) as u64,
                    digests: Digests {
                        sha1: found.sha1.clone(),
                        sha256: found.sha256.clone(),
                        sha512: found.sha512.clone(),
                        md5: found.md5.clone(),
                    },
                };
                let mut put = PutAsset::cached(path);
                put.remote_etag = found.remote_etag.clone();
                put.remote_last_modified = found.remote_last_modified;
                Ok(Some(adopt(ctx, &spec, put, &written).await?))
            }
            Err(error) => {
                tracing::warn!(repository = ctx.name(), path, %error, "the specification of a cached gem cannot be read");
                Ok(Some(found))
            }
        }
    }

    async fn info_entries(
        &self,
        ctx: &Ctx,
        name: &str,
    ) -> Result<Option<Vec<InfoEntry>>, ContentError> {
        match self.index(ctx, &Route::Info(name.to_owned())).await? {
            Some(found) => {
                let bytes = found.bytes().await?;
                Ok(Some(index::parse_info(&String::from_utf8_lossy(&bytes))))
            }
            None => Ok(None),
        }
    }

    async fn dependencies(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let mut names: Vec<String> = Vec::new();
        for (key, value) in
            url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
        {
            if key != "gems" {
                continue;
            }
            for name in value
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
            {
                if !gem::valid_name(name) {
                    return Ok(text(
                        StatusCode::BAD_REQUEST,
                        format!("Invalid gem name: {name}"),
                    ));
                }
                if !names.iter().any(|known| known == name) {
                    names.push(name.to_owned());
                }
            }
        }
        if names.len() > MAX_GEMS {
            return Ok(text(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Too many gems (use less than {MAX_GEMS} gems)"),
            ));
        }
        let mut entries = Vec::new();
        for name in names {
            if let Some(list) = self.info_entries(ctx, &name).await? {
                entries.push((name, list));
            }
        }
        body_response(request, BINARY, index::dependencies(&entries))
    }

    async fn quick(&self, ctx: &Ctx, full: &str) -> Result<Option<Index>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let found = store::find(ctx.db(), ctx.name(), &gem::gem_path(full)).await?;
                Ok(found
                    .filter(|found| found.kind == GEM && ctx.allowed(View::Read, &found.path))
                    .and_then(|found| spec_of(&found))
                    .map(|spec| Index::Built(index::quick_spec(&spec))))
            }
            RepositoryType::Proxy => {
                let path = Route::Quick(full.to_owned()).path();
                Ok(self
                    .cache(ctx, &path)
                    .await?
                    .map(|found| Index::Stored(Box::new((ctx.clone(), found)))))
            }
            RepositoryType::Group => {
                for member in members::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.quick(&member, full)).await {
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

    async fn lookup(&self, ctx: &Ctx, path: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => {
                Ok(
                    members::first(ctx, |member| async move { self.get(&member, path).await })
                        .await?,
                )
            }
            _ => Ok(self.get(ctx, path).await?.map(|found| (ctx.clone(), found))),
        }
    }

    async fn api_key(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        ctx.require(View::Read, API_KEY)?;
        let Some(subject) = ctx.subject.as_ref().filter(|subject| !subject.anonymous) else {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(header::WWW_AUTHENTICATE, CHALLENGE)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(
                    "Authentication required to obtain a RubyGems API key",
                ))
                .map_err(ContentError::internal);
        };
        if request.method == Method::DELETE {
            let deleted = api_keys::delete_kept_api_key(ctx.db(), REALM, &subject.user_id)
                .await
                .map_err(ContentError::internal)?;
            return Ok(match deleted {
                true => text(StatusCode::OK, "RubyGems API key revoked"),
                false => text(StatusCode::NOT_FOUND, "No RubyGems API key to revoke"),
            });
        }
        if !ctx.security.realm_active(REALM) {
            return Ok(text(
                StatusCode::CONFLICT,
                "The RubyGems API Key Realm is not active; ask an administrator to activate it, or use 'Basic <base64 of user:password>' as the API key",
            ));
        }
        let key = api_keys::kept_api_key(
            ctx.db(),
            ctx.content.secrets(),
            REALM,
            &subject.user_id,
            || format!("{REALM}.{}", astrum_security::password::random(40)),
        )
        .await
        .map_err(ContentError::internal)?;
        Ok(text(StatusCode::OK, key.key))
    }

    async fn read(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.clone();
        let route = route(&path);
        if route == Route::Directory {
            return directory_page(ctx);
        }
        ctx.require(View::Read, &path)?;
        match &route {
            Route::Versions | Route::Names => match self.index(ctx, &route).await? {
                Some(found) => found.respond(&request, TEXT).await,
                None => Err(ContentError::not_found(path)),
            },
            Route::Info(name) => match self.index(ctx, &route).await? {
                Some(found) => found.respond(&request, TEXT).await,
                None => Err(ContentError::not_found(format!("gem {name} not found"))),
            },
            Route::Specs(_) => match self.index(ctx, &route).await? {
                Some(found) => found.respond(&request, BINARY).await,
                None => Err(ContentError::not_found(path)),
            },
            Route::Dependencies => self.dependencies(ctx, &request).await,
            Route::Quick(full) => match self.quick(ctx, full).await? {
                Some(found) => found.respond(&request, BINARY).await,
                None => Err(ContentError::not_found(path)),
            },
            Route::Gem(_) => {
                let (owner, found) = self
                    .lookup(ctx, &path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.clone()))?;
                serve(
                    &owner,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            _ => Err(ContentError::not_found(path)),
        }
    }
}

fn read_only(ctx: &Ctx) -> Response {
    text(
        StatusCode::METHOD_NOT_ALLOWED,
        format!(
            "Repository {} is a {} repository; push gems to a rubygems hosted repository instead",
            ctx.name(),
            ctx.kind().as_str()
        ),
    )
}

#[async_trait]
impl Format for Rubygems {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        match (&request.method, route, ctx.kind()) {
            (
                &Method::GET | &Method::HEAD | &Method::POST | &Method::PUT | &Method::DELETE,
                Route::ApiKey,
                _,
            ) => self.api_key(&ctx, &request).await,
            (&Method::GET | &Method::HEAD, _, _) => self.read(&ctx, request).await,
            (&Method::POST, Route::Push, RepositoryType::Hosted) => {
                hosted::push(&ctx, request.into_stream()).await
            }
            (&Method::DELETE, Route::Yank, RepositoryType::Hosted) => {
                hosted::yank(&ctx, request).await
            }
            (&Method::POST, Route::Push, _) | (&Method::DELETE, Route::Yank, _) => {
                Ok(read_only(&ctx))
            }
            _ => Err(ContentError::not_found(request.path)),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            RepositoryType::Proxy => self.cache(ctx, path).await,
            RepositoryType::Group => {
                Ok(
                    members::first(ctx, |member| async move { self.get(&member, path).await })
                        .await?
                        .map(|(_, found)| found),
                )
            }
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
            .get("rubygems.asset")
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest("rubygems.asset is required".to_owned()))?;
        let response = hosted::push(&ctx, UploadForm::stream(file).await?).await?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body = axum::body::to_bytes(response.into_body(), MAX_FORM)
            .await
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default();
        Err(match status {
            StatusCode::CONFLICT => ContentError::Conflict(body),
            StatusCode::UNAUTHORIZED => ContentError::Unauthorized,
            StatusCode::FORBIDDEN => ContentError::Forbidden,
            _ => ContentError::BadRequest(body),
        })
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE '/' || c.name || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[
                "/versions",
                "/names",
                "/specs.4.8.gz",
                "/latest_specs.4.8.gz",
                "/prerelease_specs.4.8.gz",
            ],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

pub(crate) fn form_fields(query: Option<&str>, body: &[u8]) -> HashMap<String, String> {
    let mut form: HashMap<String, String> =
        url::form_urlencoded::parse(query.unwrap_or("").as_bytes())
            .into_owned()
            .collect();
    for (key, value) in url::form_urlencoded::parse(body).into_owned() {
        form.entry(key).or_insert(value);
    }
    form
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_indexes_gems_and_the_api() {
        assert_eq!(route("/versions"), Route::Versions);
        assert_eq!(route("/info/rails"), Route::Info("rails".to_owned()));
        assert_eq!(route("/info/../x"), Route::Other);
        assert_eq!(route("/specs.4.8.gz"), Route::Specs(Specs::All));
        assert_eq!(
            route("/gems/hola-1.0.0.gem"),
            Route::Gem("hola-1.0.0".to_owned())
        );
        assert_eq!(
            route("/quick/Marshal.4.8/hola-1.0.0.gemspec.rz"),
            Route::Quick("hola-1.0.0".to_owned())
        );
        assert_eq!(route("/api/v1/gems"), Route::Push);
        assert_eq!(route("/api/v1/gems/yank"), Route::Yank);
        assert_eq!(route("/api/v1/api_key"), Route::ApiKey);
        assert_eq!(route("/gems/.hidden.gem"), Route::Other);
        assert_eq!(route("/"), Route::Directory);
        for path in [
            "/versions",
            "/info/rails",
            "/latest_specs.4.8.gz",
            "/quick/Marshal.4.8/hola-1.0.0-java.gemspec.rz",
            "/gems/hola-1.0.0.gem",
        ] {
            assert_eq!(route(path).path(), path);
        }
    }

    #[test]
    fn forms_merge_the_query_and_the_body() {
        let form = form_fields(
            Some("gem_name=hola&version=1.0.0"),
            b"version=2.0.0&platform=java",
        );
        assert_eq!(form["gem_name"], "hola");
        assert_eq!(form["version"], "1.0.0");
        assert_eq!(form["platform"], "java");
    }
}
