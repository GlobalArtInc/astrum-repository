mod files;
mod group;
mod hosted;
pub mod merge;
pub mod paths;
mod remote;
pub mod repodata;
pub mod rpm;
pub mod signing;
pub mod task;
pub mod testing;
pub mod version;
mod xml;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use astrum_content::format::SearchTarget;
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, UploadField,
    UploadForm, UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

pub const ID: &str = "yum";
pub const RPM: &str = "RPM";
pub const REPOMD: &str = "REPOMD";
pub const REPODATA: &str = "REPODATA";
pub const OTHER: &str = "OTHER";
pub const RPM_TYPE: &str = "application/x-rpm";
pub const XML_TYPE: &str = "application/xml";
pub const GZIP_TYPE: &str = "application/x-gzip";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "yum.name",
        target: SearchTarget::Component,
        pointer: &["yum", "name"],
    },
    SearchKey {
        param: "yum.version",
        target: SearchTarget::Asset,
        pointer: &["yum", "version"],
    },
    SearchKey {
        param: "yum.release",
        target: SearchTarget::Asset,
        pointer: &["yum", "release"],
    },
    SearchKey {
        param: "yum.architecture",
        target: SearchTarget::Asset,
        pointer: &["yum", "architecture"],
    },
];

#[derive(Default)]
pub struct Yum {
    keys: Arc<signing::Keys>,
    pending: Arc<Mutex<HashSet<String>>>,
    seen: Mutex<HashMap<String, Uuid>>,
}

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Repomd(String),
    Signature(String),
    PublicKey(String),
    Repodata(String),
    Directory,
    File,
}

fn route(path: &str) -> Route {
    if path.ends_with('/') {
        return Route::Directory;
    }
    match paths::base_of_metadata(path) {
        Some(base) => match paths::file_name(path) {
            paths::REPOMD => Route::Repomd(base),
            paths::SIGNATURE => Route::Signature(base),
            paths::PUBLIC_KEY => Route::PublicKey(base),
            _ => Route::Repodata(base),
        },
        None => Route::File,
    }
}

fn allow(ctx: &Ctx) -> &'static str {
    match ctx.kind() {
        RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
        _ => "GET,HEAD",
    }
}

fn bytes_response(
    request: &ContentRequest,
    content_type: &str,
    body: Vec<u8>,
) -> Result<Response, ContentError> {
    let length = body.len();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Yum repository. Point a <code>.repo</code> file at <code>{url}/</code> (or the directory that holds its <code>repodata</code>) to install its packages with dnf or yum.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

impl Yum {
    fn pending(&self, key: &str) -> bool {
        self.pending
            .lock()
            .expect("the pending rebuild lock is never poisoned")
            .insert(key.to_owned())
    }

    fn first_sight(&self, repository: &str, found: &Asset) -> bool {
        let key = format!("{repository}\n{}", found.path);
        let mut seen = self
            .seen
            .lock()
            .expect("the seen repomd lock is never poisoned");
        seen.insert(key, found.blob_id) != Some(found.blob_id)
    }

    async fn serve_asset(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        found: Option<Asset>,
    ) -> Result<Response, ContentError> {
        let found = found.ok_or_else(|| ContentError::not_found(request.path.clone()))?;
        serve(
            ctx,
            &found,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }

    async fn signature(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        base: &str,
    ) -> Result<Response, ContentError> {
        let key = self.keys.get(ctx)?;
        match ctx.kind() {
            RepositoryType::Hosted => {
                hosted::ensure(ctx, &self.keys, base, false).await?;
                let found = store::find(ctx.db(), ctx.name(), &request.path).await?;
                self.serve_asset(ctx, request, found).await
            }
            RepositoryType::Proxy => match key {
                Some(key) => {
                    let repomd = self
                        .get(ctx, &paths::repomd_path(base))
                        .await?
                        .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                    let bytes = files::bytes(ctx, &repomd).await?;
                    bytes_response(request, signing::SIGNATURE_TYPE, sign(&key, &bytes)?)
                }
                None => {
                    let found = self.get(ctx, &request.path).await?;
                    self.serve_asset(ctx, request, found).await
                }
            },
            RepositoryType::Group => {
                let repomd = group::repomd(self, ctx, base)
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                match (key, repomd) {
                    (Some(key), repomd) => {
                        let bytes = files::bytes(ctx, repomd.asset()).await?;
                        bytes_response(request, signing::SIGNATURE_TYPE, sign(&key, &bytes)?)
                    }
                    (None, group::GroupRepomd::Passed(member, _)) => {
                        let found = self.get(&member, &request.path).await?;
                        self.serve_asset(&member, request, found).await
                    }
                    (None, group::GroupRepomd::Merged(_)) => {
                        Err(ContentError::not_found(request.path.clone()))
                    }
                }
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if route(path) == Route::Directory {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        match route(path) {
            Route::Repomd(base) => match ctx.kind() {
                RepositoryType::Group => {
                    let found = group::repomd(self, ctx, &base).await?;
                    match found {
                        Some(group::GroupRepomd::Passed(member, found)) => {
                            serve(
                                member.as_ref(),
                                &found,
                                &request.method,
                                &request.headers,
                                Serve::download(),
                            )
                            .await
                        }
                        Some(group::GroupRepomd::Merged(found)) => {
                            self.serve_asset(ctx, request, Some(found)).await
                        }
                        None => Err(ContentError::not_found(path.to_owned())),
                    }
                }
                _ => {
                    let found = self.get(ctx, path).await?;
                    self.serve_asset(ctx, request, found).await
                }
            },
            Route::Signature(base) => self.signature(ctx, request, &base).await,
            Route::PublicKey(_) => match self.keys.get(ctx)? {
                Some(key) => bytes_response(request, signing::KEY_TYPE, key.public_key.clone()),
                None if ctx.kind() == RepositoryType::Hosted => {
                    Err(ContentError::not_found(path.to_owned()))
                }
                None => {
                    let found = self.lookup(ctx, path).await?;
                    match found {
                        Some((owner, found)) => {
                            serve(
                                &owner,
                                &found,
                                &request.method,
                                &request.headers,
                                Serve::download(),
                            )
                            .await
                        }
                        None => Err(ContentError::not_found(path.to_owned())),
                    }
                }
            },
            Route::Repodata(_) | Route::File => match self.lookup(ctx, path).await? {
                Some((owner, found)) => {
                    serve(
                        &owner,
                        &found,
                        &request.method,
                        &request.headers,
                        Serve::download(),
                    )
                    .await
                }
                None => Err(ContentError::not_found(path.to_owned())),
            },
            Route::Directory => directory_page(ctx),
        }
    }

    async fn lookup(&self, ctx: &Ctx, path: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => group::metadata(self, ctx, path).await,
            _ => Ok(self.get(ctx, path).await?.map(|found| (ctx.clone(), found))),
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        if found.kind == REPOMD || found.kind == REPODATA {
            return Err(ContentError::not_found(path.to_owned()));
        }
        ctx.require(View::Delete, path)?;
        store::delete_asset(&ctx.content, found.clone()).await?;
        if found.kind == RPM {
            hosted::schedule(self, ctx, path);
        }
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

fn sign(key: &signing::SigningKey, bytes: &[u8]) -> Result<Vec<u8>, ContentError> {
    key.sign(bytes)
        .map_err(|error| ContentError::internal(std::io::Error::other(error)))
}

#[async_trait]
impl Format for Yum {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::PUT, RepositoryType::Hosted) if route(&request.path) != Route::Directory => {
                let path = request.path.clone();
                let stored = hosted::put(&ctx, &path, request.into_stream()).await?;
                if stored.kind == RPM {
                    hosted::schedule(self, &ctx, &path);
                }
                Ok(StatusCode::CREATED.into_response())
            }
            (&Method::DELETE, RepositoryType::Hosted) => self.delete(&ctx, &request.path).await,
            (method, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: allow(&ctx),
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => match route(path) {
                Route::Repomd(base) => hosted::ensure(ctx, &self.keys, &base, false).await,
                _ => store::find(ctx.db(), ctx.name(), path).await,
            },
            RepositoryType::Proxy => remote::get(self, ctx, path).await,
            RepositoryType::Group => match route(path) {
                Route::Repomd(base) => Ok(group::repomd(self, ctx, &base)
                    .await?
                    .map(|found| found.asset().clone())),
                _ => Ok(group::metadata(self, ctx, path)
                    .await?
                    .map(|(_, found)| found)),
            },
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![
                UploadField::text("directory", true)
                    .describe("Directory of the package, for example el9/x86_64/Packages")
                    .in_group("Component attributes"),
            ],
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
            .get("yum.asset")
            .ok_or_else(|| ContentError::BadRequest("yum.asset is required".to_owned()))?;
        let directory = form.field("yum.directory").unwrap_or("").trim().to_owned();
        let stored = hosted::upload(
            &ctx,
            &directory,
            file.filename.as_deref(),
            UploadForm::stream(file).await?,
        )
        .await?;
        hosted::schedule(self, &ctx, &stored.path);
        Ok(())
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout::default()
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_metadata_and_files() {
        assert_eq!(route("/repodata/repomd.xml"), Route::Repomd(String::new()));
        assert_eq!(
            route("/el9/x86_64/repodata/repomd.xml"),
            Route::Repomd("/el9/x86_64".to_owned())
        );
        assert_eq!(
            route("/el9/repodata/repomd.xml.asc"),
            Route::Signature("/el9".to_owned())
        );
        assert_eq!(
            route("/repodata/repomd.xml.key"),
            Route::PublicKey(String::new())
        );
        assert_eq!(
            route("/repodata/abc-primary.xml.gz"),
            Route::Repodata(String::new())
        );
        assert_eq!(route("/Packages/a-1-1.x86_64.rpm"), Route::File);
        assert_eq!(route("/el9/"), Route::Directory);
        assert_eq!(route("/repodata/sub/x"), Route::File);
    }
}
