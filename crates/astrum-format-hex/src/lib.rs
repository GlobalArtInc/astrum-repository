mod api;
pub mod etf;
mod files;
mod group;
mod hosted;
pub mod keys;
pub mod proto;
pub mod registry;
mod remote;
pub mod tarball;
pub mod task;
pub mod terms;
pub mod testing;

use std::sync::Arc;

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Serve, UploadField, UploadForm,
    UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::api::Api;
use crate::keys::Keys;

pub const ID: &str = "hex";
pub const TARBALL: &str = "tarball";
pub const REGISTRY: &str = "registry";
pub const KEY: &str = "key";
pub const DOCS: &str = "docs";
pub const DOCS_TYPE: &str = "application/gzip";
pub const OTHER: &str = "other";
pub const TAR_TYPE: &str = "application/octet-stream";
pub const REGISTRY_TYPE: &str = "application/octet-stream";
pub const NAMES: &str = "/names";
pub const VERSIONS: &str = "/versions";
pub const PUBLIC_KEY: &str = "/public_key";
const UPLOAD_FIELD: &str = "hex.asset";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "hex.license",
        target: SearchTarget::Component,
        pointer: &["hex", "licenses"],
    },
    SearchKey {
        param: "hex.buildTool",
        target: SearchTarget::Component,
        pointer: &["hex", "buildTools"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["hex", "description"]];

#[derive(Default)]
pub struct Hex {
    keys: Arc<Keys>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resource {
    Names,
    Versions,
    Package(String),
}

impl Resource {
    pub fn path(&self) -> String {
        match self {
            Resource::Names => NAMES.to_owned(),
            Resource::Versions => VERSIONS.to_owned(),
            Resource::Package(name) => format!("/packages/{name}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Directory,
    Resource(Resource),
    Tarball(String, String),
    PublicKey,
    Passed,
    Api(Api),
    Other,
}

pub fn tarball_path(name: &str, version: &str) -> String {
    format!("/tarballs/{}", tarball::file_name(name, version))
}

pub fn docs_path(name: &str, version: &str) -> String {
    format!("/docs/{name}-{version}.tar.gz")
}

fn package_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub(crate) fn route(path: &str) -> Route {
    if path == "/" || path.is_empty() {
        return Route::Directory;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["names"] => Route::Resource(Resource::Names),
        ["versions"] => Route::Resource(Resource::Versions),
        ["packages", name] if package_name(name) => {
            Route::Resource(Resource::Package((*name).to_owned()))
        }
        ["tarballs", file] => match tarball::split_file(file) {
            Some((name, version)) => Route::Tarball(name.to_owned(), version.to_owned()),
            None => Route::Other,
        },
        ["public_key"] => Route::PublicKey,
        ["docs", file] if file.ends_with(".tar.gz") => Route::Passed,
        ["installs", ..] if !path.ends_with('/') => Route::Passed,
        ["api", rest @ ..] => Route::Api(api::route(rest)),
        _ => Route::Other,
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx, fingerprint: Option<&str>) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let setup = match (ctx.kind(), fingerprint) {
        (RepositoryType::Proxy, _) => format!(
            "<p>It mirrors a Hex repository and passes its signed registry through unchanged, so clients keep checking it with the key of that repository. Use it as the mirror of hexpm: <code>HEX_MIRROR={url}</code> for Mix, <code>HEX_CDN={url}</code> for rebar3.</p>"
        ),
        (_, Some(fingerprint)) => {
            let fingerprint = escape(fingerprint);
            format!(
                "<p>Its registry is signed with its own key, served at <a href=\"{url}/public_key\">public_key</a> (fingerprint <code>{fingerprint}</code>). Add it to Mix under its own name with <code>mix hex.repo add {name} {url} --fetch-public-key {fingerprint}</code> and name it in dependencies with <code>repo: \"{name}\"</code>.</p>"
            )
        }
        (_, None) => format!(
            "<p>Its registry is signed with its own key, served at <code>{url}/public_key</code> to users who may read this repository. Add it to Mix under its own name with <code>mix hex.repo add {name} {url} --public-key &lt;file&gt;</code>.</p>"
        ),
    };
    let publish = match ctx.kind() {
        RepositoryType::Hosted => format!(
            "<p>Publish to it with <code>HEX_API_URL={url}/api HEX_API_KEY=&lt;access token&gt; mix hex.publish</code>.</p>\n    "
        ),
        _ => String::new(),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(format!(
            "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Hex repository for Elixir and Erlang packages.</p>\n    {setup}\n    {publish}<p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
            escape(&ctx.base_url),
        )))
        .map_err(ContentError::internal)
}

fn bytes_response(
    request: &ContentRequest,
    content_type: &str,
    body: Vec<u8>,
) -> Result<Response, ContentError> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CACHE_CONTROL, "no-cache")
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}

fn allow(kind: RepositoryType) -> &'static str {
    match kind {
        RepositoryType::Hosted => "GET,HEAD,PUT,POST,DELETE",
        RepositoryType::Proxy => "GET,HEAD,DELETE",
        RepositoryType::Group => "GET,HEAD",
    }
}

fn options(found: &Asset) -> Serve {
    match found.kind.as_str() {
        TARBALL => Serve::download(),
        _ => Serve::default(),
    }
}

impl Hex {
    pub(crate) async fn resource(
        &self,
        ctx: &Ctx,
        resource: &Resource,
    ) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::ensure(ctx, &self.keys, resource).await,
            RepositoryType::Proxy => remote::get(ctx, &resource.path()).await,
            RepositoryType::Group => self.merged_resource(ctx, resource).await,
        }
    }

    async fn file(&self, ctx: &Ctx, path: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(store::find(ctx.db(), ctx.name(), path)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => Ok(remote::get(ctx, path)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                astrum_content::group::first(ctx, |member| async move {
                    Box::pin(self.file(&member, path))
                        .await
                        .map(|found| found.map(|(_, found)| found))
                })
                .await
            }
        }
    }

    async fn public_key(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        match ctx.kind() {
            RepositoryType::Proxy => {
                let found = remote::get(ctx, PUBLIC_KEY)
                    .await?
                    .ok_or_else(|| ContentError::not_found(PUBLIC_KEY.to_owned()))?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::default(),
                )
                .await
            }
            _ => {
                let key = self.keys.get(ctx).await?;
                bytes_response(request, keys::KEY_TYPE, key.public_key.clone().into_bytes())
            }
        }
    }

    async fn read(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        match route {
            Route::Directory => {
                let fingerprint = match ctx.kind() {
                    RepositoryType::Proxy => None,
                    _ if ctx.allowed(View::Read, PUBLIC_KEY) => {
                        Some(self.keys.get(ctx).await?.fingerprint.clone())
                    }
                    _ => None,
                };
                directory_page(ctx, fingerprint.as_deref())
            }
            Route::Resource(resource) => {
                ctx.require(View::Read, path)?;
                let found = self
                    .resource(ctx, &resource)
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
            Route::PublicKey => {
                ctx.require(View::Read, path)?;
                self.public_key(ctx, request).await
            }
            Route::Tarball(_, _) | Route::Passed => {
                ctx.require(View::Read, path)?;
                let (owner, found) = self
                    .file(ctx, path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                serve(
                    &owner,
                    &found,
                    &request.method,
                    &request.headers,
                    options(&found),
                )
                .await
            }
            Route::Api(_) | Route::Other => Err(ContentError::not_found(path.to_owned())),
        }
    }

    async fn delete(&self, ctx: &Ctx, route: Route, path: &str) -> Result<Response, ContentError> {
        match (ctx.kind(), route) {
            (RepositoryType::Hosted, Route::Tarball(name, version)) => {
                hosted::remove(ctx, &self.keys, &name, &version).await?;
            }
            (RepositoryType::Hosted, Route::Passed) => {
                ctx.require(View::Delete, path)?;
                if !store::delete(ctx, path).await? {
                    return Err(ContentError::not_found(path.to_owned()));
                }
            }
            (RepositoryType::Hosted, _) => {
                return Err(ContentError::BadRequest(
                    "the registry of a hosted Hex repository is generated; delete package tarballs instead"
                        .to_owned(),
                ));
            }
            _ => {
                ctx.require(View::Delete, path)?;
                if !store::delete(ctx, path).await? {
                    return Err(ContentError::not_found(path.to_owned()));
                }
            }
        }
        Ok(StatusCode::NO_CONTENT.into_response())
    }

    pub async fn rebuild(&self, ctx: &Ctx) -> Result<u64, ContentError> {
        hosted::rebuild(ctx, &self.keys, None, true).await
    }
}

#[async_trait]
impl Format for Hex {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        match (&request.method, route) {
            (_, Route::Api(api)) => Ok(self.api(&ctx, request, api).await),
            (&Method::GET | &Method::HEAD, route) => self.read(&ctx, &request, route).await,
            (&Method::PUT, Route::Tarball(name, version))
                if ctx.kind() == RepositoryType::Hosted =>
            {
                hosted::may_add(&ctx)?;
                let body = request.bytes(tarball::MAX_TARBALL as usize).await?;
                hosted::publish(&ctx, &self.keys, body, true, Some(&name), Some(&version)).await?;
                Ok(StatusCode::CREATED.into_response())
            }
            (&Method::DELETE, route) if ctx.kind() != RepositoryType::Group => {
                let path = request.path.clone();
                self.delete(&ctx, route, &path).await
            }
            (method, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: allow(ctx.kind()),
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Resource(resource) => self.resource(ctx, &resource).await,
            Route::PublicKey if ctx.kind() != RepositoryType::Proxy => Ok(None),
            _ => Ok(self.file(ctx, path).await?.map(|(_, found)| found)),
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
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest(format!("{UPLOAD_FIELD} is required")))?;
        hosted::may_add(&ctx)?;
        if file.size > tarball::MAX_TARBALL {
            return Err(ContentError::BadRequest(format!(
                "the package tarball is larger than {} bytes",
                tarball::MAX_TARBALL
            )));
        }
        let body = tokio::fs::read(&file.path)
            .await
            .map_err(ContentError::internal)?;
        hosted::publish(&ctx, &self.keys, body.into(), true, None, None).await?;
        Ok(())
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        KEYWORDS
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: None,
            generated: &[PUBLIC_KEY],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_registry_the_tarballs_and_the_api() {
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/names"), Route::Resource(Resource::Names));
        assert_eq!(route("/versions"), Route::Resource(Resource::Versions));
        assert_eq!(
            route("/packages/plug_crypto"),
            Route::Resource(Resource::Package("plug_crypto".to_owned()))
        );
        assert_eq!(
            route("/tarballs/plug_crypto-2.1.0-rc.1+build.tar"),
            Route::Tarball("plug_crypto".to_owned(), "2.1.0-rc.1+build".to_owned())
        );
        assert_eq!(route("/public_key"), Route::PublicKey);
        assert_eq!(route("/docs/plug-1.0.0.tar.gz"), Route::Passed);
        assert_eq!(route("/installs/hex-1.x.csv"), Route::Passed);
        assert_eq!(route("/api/publish"), Route::Api(Api::Publish));
        assert_eq!(
            route("/api/repos/acme/packages/a/releases/1.0.0"),
            Route::Api(Api::Release("a".to_owned(), "1.0.0".to_owned()))
        );
        for other in [
            "/packages/a/b",
            "/packages/a.b",
            "/tarballs/a.tar",
            "/tarballs/a-1.0.0.tar/x",
            "/names/x",
            "/docs/a",
            "/installs/",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        assert_eq!(tarball_path("a", "1.0.0"), "/tarballs/a-1.0.0.tar");
        assert_eq!(Resource::Package("a".to_owned()).path(), "/packages/a");
    }
}
