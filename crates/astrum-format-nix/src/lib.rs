mod db;
pub mod hash;
mod hosted;
pub mod names;
pub mod narinfo;
mod remote;
pub mod task;
pub mod testing;

use astrum_content::format::SearchTarget;
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, View, group,
    serve,
};
use astrum_repo::RepositoryType;
use astrum_repo::config::{DEFAULT_NIX_PRIORITY, Nix as Settings};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use sha2::{Digest, Sha256};

use crate::names::narinfo_path;

pub const ID: &str = "nix";
pub const NARINFO: &str = "narinfo";
pub const NAR: &str = "nar";
pub const LISTING: &str = "listing";
pub const LOG: &str = "log";
pub const REALISATION: &str = "realisation";
pub const DEBUG_INFO: &str = "debuginfo";
pub const CACHE_INFO_TYPE: &str = "text/x-nix-cache-info";
pub const NARINFO_TYPE: &str = "text/x-nix-narinfo";
pub const NAR_TYPE: &str = "application/x-nix-nar";
const JSON_TYPE: &str = "application/json";
const LOG_TYPE: &str = "text/plain";
const CACHE_INFO: &str = "/nix-cache-info";
const MAX_NARINFO: usize = 4 * 1024 * 1024;
const MAX_DOCUMENT: usize = 1024 * 1024;

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "nix.storePath",
        target: SearchTarget::Component,
        pointer: &["nix", "storePath"],
    },
    SearchKey {
        param: "nix.narHash",
        target: SearchTarget::Component,
        pointer: &["nix", "narHash"],
    },
];

#[derive(Default)]
pub struct Nix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum File {
    Nar,
    Listing,
    Log,
    Realisation,
    DebugInfo,
}

impl File {
    fn kind(self) -> &'static str {
        match self {
            File::Nar => NAR,
            File::Listing => LISTING,
            File::Log => LOG,
            File::Realisation => REALISATION,
            File::DebugInfo => DEBUG_INFO,
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            File::Nar => NAR_TYPE,
            File::Log => LOG_TYPE,
            File::Listing | File::Realisation | File::DebugInfo => JSON_TYPE,
        }
    }

    fn document(self) -> bool {
        matches!(self, File::Realisation | File::DebugInfo)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    CacheInfo,
    NarInfo(String),
    File(File),
    Other,
}

fn document_name(name: &str) -> bool {
    name.strip_suffix(".doi").is_some_and(|stem| {
        !stem.is_empty()
            && stem.len() <= 256
            && stem
                .chars()
                .all(|c| c.is_ascii_graphic() && c != '/' && c != '\\')
    })
}

pub(crate) fn route(path: &str) -> Route {
    if path == CACHE_INFO {
        return Route::CacheInfo;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [file] => {
            if let Some(hash) = file
                .strip_suffix(".narinfo")
                .filter(|hash| names::valid_hash(hash))
            {
                return Route::NarInfo(hash.to_owned());
            }
            match file.strip_suffix(".ls").is_some_and(names::valid_hash) {
                true => Route::File(File::Listing),
                false => Route::Other,
            }
        }
        ["nar", file] if names::valid_nar_file(file) => Route::File(File::Nar),
        ["log", base] if names::valid_base(base) => Route::File(File::Log),
        ["realisations", id] if document_name(id) => Route::File(File::Realisation),
        ["build-trace-v2", derivation, output]
            if names::valid_base(derivation) && document_name(output) =>
        {
            Route::File(File::Realisation)
        }
        ["debuginfo", id]
            if !id.is_empty() && id.len() <= 128 && id.bytes().all(|c| c.is_ascii_hexdigit()) =>
        {
            Route::File(File::DebugInfo)
        }
        _ => Route::Other,
    }
}

pub(crate) fn settings(ctx: &Ctx) -> Settings {
    ctx.repository.attributes.nix.clone().unwrap_or_default()
}

pub(crate) fn store_dir(ctx: &Ctx) -> String {
    settings(ctx).store_dir().to_owned()
}

pub fn cache_info(settings: &Settings) -> String {
    format!(
        "StoreDir: {}\nWantMassQuery: {}\nPriority: {}\n",
        settings.store_dir(),
        if settings.want_mass_query.unwrap_or(true) {
            1
        } else {
            0
        },
        settings.priority.unwrap_or(DEFAULT_NIX_PRIORITY)
    )
}

fn may_read(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Read).any() {
        true => Ok(()),
        false => ctx.require(View::Read, CACHE_INFO),
    }
}

pub(crate) fn generated(request: &ContentRequest, content_type: &str, body: Vec<u8>) -> Response {
    let etag = format!("\"{}\"", hex::encode(&Sha256::digest(&body)[..16]));
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CONTENT_TYPE, content_type);
    let response = match (unchanged, request.method == Method::HEAD) {
        (true, _) => builder.status(StatusCode::NOT_MODIFIED).body(Body::empty()),
        (false, true) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        (false, false) => builder.status(StatusCode::OK).body(Body::from(body)),
    };
    response.expect("a generated response is valid")
}

pub(crate) fn status(code: StatusCode) -> Response {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .expect("an empty response is valid")
}

fn content_encoding(found: &Asset) -> Option<&str> {
    found
        .attributes
        .pointer("/nix/contentEncoding")
        .and_then(serde_json::Value::as_str)
}

fn read_only(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
            RepositoryType::Proxy => "GET,HEAD,DELETE",
            RepositoryType::Group => "GET,HEAD",
        },
    }
}

impl Nix {
    async fn narinfo(&self, ctx: &Ctx, hash: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::narinfo(ctx, hash)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => Ok(remote::narinfo(ctx, hash)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                group::first(ctx, |member| async move {
                    Ok(Box::pin(self.narinfo(&member, hash))
                        .await?
                        .map(|(_, found)| found))
                })
                .await
            }
        }
    }

    async fn file(
        &self,
        ctx: &Ctx,
        path: &str,
        file: File,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(store::find(ctx.db(), ctx.name(), path)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => Ok(remote::file(ctx, path, file)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                group::first(ctx, |member| async move {
                    Ok(Box::pin(self.file(&member, path, file))
                        .await?
                        .map(|(_, found)| found))
                })
                .await
            }
        }
    }

    async fn serve_narinfo(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        hash: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &narinfo_path(hash))?;
        let (owner, found) = self.narinfo(ctx, hash).await?.ok_or_else(|| {
            ContentError::not_found(format!("{hash}.narinfo is not in {}", ctx.name()))
        })?;
        if let Some(body) = remote::rewritten(&owner, &found).await? {
            return Ok(generated(request, NARINFO_TYPE, body));
        }
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve {
                content_type: Some(NARINFO_TYPE.to_owned()),
                ..Serve::download()
            },
        )
        .await
    }

    async fn serve_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        file: File,
    ) -> Result<Response, ContentError> {
        let path = &request.path;
        ctx.require(View::Read, path)?;
        let (owner, found) = self
            .file(ctx, path, file)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("{path} is not in {}", ctx.name())))?;
        let mut response = serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await?;
        if let Some(value) =
            content_encoding(&found).and_then(|value| HeaderValue::from_str(value).ok())
        {
            response
                .headers_mut()
                .insert(header::CONTENT_ENCODING, value);
        }
        Ok(response)
    }

    async fn delete(&self, ctx: &Ctx, path: &str, route: Route) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        let deleted = match (ctx.kind(), route) {
            (RepositoryType::Hosted, Route::NarInfo(hash)) => {
                hosted::delete_narinfo(ctx, &hash).await?
            }
            _ => store::delete(ctx, path).await?,
        };
        match deleted {
            true => Ok(status(StatusCode::NO_CONTENT)),
            false => Err(ContentError::not_found(path.to_owned())),
        }
    }
}

#[async_trait]
impl Format for Nix {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let method = request.method.clone();
        match (&method, route(&request.path)) {
            (&Method::GET | &Method::HEAD, Route::CacheInfo) => {
                may_read(&ctx)?;
                Ok(generated(
                    &request,
                    CACHE_INFO_TYPE,
                    cache_info(&settings(&ctx)).into_bytes(),
                ))
            }
            (&Method::GET | &Method::HEAD, Route::NarInfo(hash)) => {
                self.serve_narinfo(&ctx, &request, &hash).await
            }
            (&Method::GET | &Method::HEAD, Route::File(file)) => {
                self.serve_file(&ctx, &request, file).await
            }
            (&Method::GET | &Method::HEAD, Route::Other) => Err(ContentError::not_found(format!(
                "{} is not a file of a Nix binary cache",
                request.path
            ))),
            (&Method::PUT, route) if ctx.kind() == RepositoryType::Hosted => {
                hosted::put(&ctx, request, route).await
            }
            (&Method::DELETE, route) if ctx.kind() != RepositoryType::Group => {
                let path = request.path.clone();
                self.delete(&ctx, &path, route).await
            }
            (method, _) => Err(read_only(method, ctx.kind())),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let found = match route(path) {
            Route::NarInfo(hash) => self.narinfo(ctx, &hash).await?,
            Route::File(file) => self.file(ctx, path, file).await?,
            Route::CacheInfo | Route::Other => match ctx.kind() {
                RepositoryType::Group => {
                    group::first(ctx, |member| async move { self.get(&member, path).await }).await?
                }
                _ => store::find(ctx.db(), ctx.name(), path)
                    .await?
                    .map(|found| (ctx.clone(), found)),
            },
        };
        Ok(found.map(|(_, found)| found))
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        &[&["nix", "storePath"]]
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE '/' || c.name || '/' || CASE WHEN c.version = '' THEN '' ELSE c.version || '/' END || regexp_replace(a.path, '^.*/', '') END",
            ),
            generated: &[CACHE_INFO],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_files_of_a_binary_cache() {
        let hash = "mw9ls4zxx884y7yq0a04ildfdjfy1vn2";
        assert_eq!(route("/nix-cache-info"), Route::CacheInfo);
        assert_eq!(
            route(&format!("/{hash}.narinfo")),
            Route::NarInfo(hash.to_owned())
        );
        assert_eq!(route(&format!("/{hash}.ls")), Route::File(File::Listing));
        assert_eq!(
            route("/nar/0xfhhhb46a15v23pvkw97a5h578d1qkl34w351p5xxh52f1jbhyl.nar.zst"),
            Route::File(File::Nar)
        );
        assert_eq!(
            route(&format!("/log/{hash}-hello-2.12.1.drv")),
            Route::File(File::Log)
        );
        assert_eq!(
            route(
                "/realisations/sha256:1b8m7l4rflsmqbqdsng5lbrkbfgaqh7a43f6dz4wnflzk1kwnxvj!out.doi"
            ),
            Route::File(File::Realisation)
        );
        assert_eq!(
            route(&format!("/build-trace-v2/{hash}-hello.drv/out.doi")),
            Route::File(File::Realisation)
        );
        assert_eq!(
            route("/debuginfo/0123456789abcdef0123456789abcdef01234567"),
            Route::File(File::DebugInfo)
        );
        for other in [
            "/",
            "/nix-cache-info/",
            "/short.narinfo",
            &format!("/nar/{hash}.tar.gz"),
            "/log/hello.drv",
            "/realisations/x.json",
            "/debuginfo/xyz",
            &format!("/a/{hash}.narinfo"),
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }

    #[test]
    fn cache_info_comes_from_the_settings() {
        assert_eq!(
            cache_info(&Settings::default()),
            "StoreDir: /nix/store\nWantMassQuery: 1\nPriority: 40\n"
        );
        let settings = Settings {
            store_dir: Some("/opt/store".to_owned()),
            priority: Some(10),
            want_mass_query: Some(false),
        };
        assert_eq!(
            cache_info(&settings),
            "StoreDir: /opt/store\nWantMassQuery: 0\nPriority: 10\n"
        );
    }
}
