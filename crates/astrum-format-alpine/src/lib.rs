pub mod archive;
mod files;
mod group;
mod hosted;
pub mod index;
pub mod package;
pub mod paths;
pub mod pem;
mod remote;
pub mod signing;
pub mod task;
pub mod testing;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
use uuid::Uuid;

use crate::paths::Route;

pub const ID: &str = "alpine";
pub const APK: &str = "apk";
pub const INDEX: &str = "index";
pub const KEY: &str = "key";
pub const OTHER: &str = "other";
pub const APK_TYPE: &str = "application/gzip";
pub const INDEX_TYPE: &str = "application/gzip";

type SignatureCache = HashMap<(String, String), (Uuid, String, Vec<u8>)>;

#[derive(Default)]
pub struct Alpine {
    keys: Arc<signing::Keys>,
    signatures: Mutex<SignatureCache>,
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx, key: Option<&str>) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let key = match key {
        Some(key) => {
            let key = escape(key);
            format!(
                "<p>Its indexes are signed: save <a href=\"{url}/{key}\">{key}</a> as <code>/etc/apk/keys/{key}</code>.</p>\n    "
            )
        }
        None => String::new(),
    };
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is an Alpine package repository. Add <code>{url}/&lt;directory&gt;</code> to <code>/etc/apk/repositories</code>, where <code>&lt;directory&gt;</code> holds a folder per architecture, for example <code>v3.20/main</code>.</p>\n    {key}<p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
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

fn allow(ctx: &Ctx) -> &'static str {
    match ctx.kind() {
        RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
        RepositoryType::Proxy => "GET,HEAD,DELETE",
        RepositoryType::Group => "GET,HEAD",
    }
}

fn options(found: &Asset) -> Serve {
    Serve {
        count_download: found.kind == APK,
        ..Serve::default()
    }
}

async fn serve_found(
    ctx: &Ctx,
    request: &ContentRequest,
    found: Option<Asset>,
) -> Result<Response, ContentError> {
    let found = found.ok_or_else(|| ContentError::not_found(request.path.clone()))?;
    let options = options(&found);
    serve(ctx, &found, &request.method, &request.headers, options).await
}

impl Alpine {
    async fn key_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        file: &str,
    ) -> Result<Response, ContentError> {
        if let Some(key) = self.keys.get(ctx)?
            && key.name == file
        {
            return bytes_response(
                request,
                signing::KEY_TYPE,
                key.public_key.clone().into_bytes(),
            );
        }
        match ctx.kind() {
            RepositoryType::Hosted => Err(ContentError::not_found(request.path.clone())),
            RepositoryType::Proxy => {
                let found = self.get(ctx, &request.path).await?;
                serve_found(ctx, request, found).await
            }
            RepositoryType::Group => {
                for member in astrum_content::group::members(ctx).await? {
                    let member = ctx.member(member);
                    if signing::key_name(member.name()) == file
                        && let Some(key) = self.keys.get(&member)?
                    {
                        return bytes_response(
                            request,
                            signing::KEY_TYPE,
                            key.public_key.clone().into_bytes(),
                        );
                    }
                }
                match group::file(self, ctx, &request.path).await? {
                    Some((member, found)) => serve_found(&member, request, Some(found)).await,
                    None => Err(ContentError::not_found(request.path.clone())),
                }
            }
        }
    }

    async fn index_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        directory: &str,
    ) -> Result<Response, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let found = hosted::ensure(ctx, &self.keys, directory).await?;
                serve_found(ctx, request, found).await
            }
            RepositoryType::Proxy => {
                let found = self
                    .get(ctx, &request.path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                match self.keys.get(ctx)? {
                    Some(key) => {
                        let file = self.resigned(ctx, key, &found).await?;
                        bytes_response(request, INDEX_TYPE, file)
                    }
                    None => serve_found(ctx, request, Some(found)).await,
                }
            }
            RepositoryType::Group => match group::index(self, ctx, directory).await? {
                Some(group::GroupIndex::Passed(member, found)) => {
                    match (member.kind(), self.keys.get(&member)?) {
                        (RepositoryType::Proxy, Some(key)) => {
                            let file = self.resigned(&member, key, &found).await?;
                            bytes_response(request, INDEX_TYPE, file)
                        }
                        _ => serve_found(&member, request, Some(found)).await,
                    }
                }
                Some(group::GroupIndex::Merged(found)) => {
                    serve_found(ctx, request, Some(found)).await
                }
                None => Err(ContentError::not_found(request.path.clone())),
            },
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = paths::route(path);
        if route == Route::Directory {
            let key = self.keys.get(ctx).ok().flatten();
            return directory_page(ctx, key.as_ref().map(|key| key.name.as_str()));
        }
        ctx.require(View::Read, path)?;
        match route {
            Route::Index(directory) => self.index_file(ctx, request, &directory).await,
            Route::Key(file) => self.key_file(ctx, request, &file).await,
            Route::Package | Route::File | Route::Directory => match ctx.kind() {
                RepositoryType::Group => match group::file(self, ctx, path).await? {
                    Some((member, found)) => serve_found(&member, request, Some(found)).await,
                    None => Err(ContentError::not_found(path.to_owned())),
                },
                _ => {
                    let found = self.get(ctx, path).await?;
                    serve_found(ctx, request, found).await
                }
            },
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        if ctx.kind() == RepositoryType::Hosted && found.kind != APK {
            return Err(ContentError::BadRequest(
                "the indexes of a hosted Alpine repository are generated; delete packages instead"
                    .to_owned(),
            ));
        }
        store::delete_asset(&ctx.content, found).await?;
        if ctx.kind() == RepositoryType::Hosted {
            hosted::rebuild(ctx, &self.keys, paths::directory(path), false).await?;
        }
        Ok(StatusCode::NO_CONTENT.into_response())
    }

    pub async fn rebuild(&self, ctx: &Ctx) -> Result<u64, ContentError> {
        hosted::rebuild_all(ctx, &self.keys).await
    }
}

#[async_trait]
impl Format for Alpine {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::PUT, RepositoryType::Hosted) if !request.is_directory() => {
                let path = request.path.clone();
                hosted::put(&ctx, &self.keys, &path, request.into_stream()).await?;
                Ok(StatusCode::CREATED.into_response())
            }
            (&Method::DELETE, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (method, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: allow(&ctx),
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match (ctx.kind(), paths::route(path)) {
            (RepositoryType::Hosted, Route::Index(directory)) => {
                hosted::ensure(ctx, &self.keys, &directory).await
            }
            (RepositoryType::Hosted, _) => store::find(ctx.db(), ctx.name(), path).await,
            (RepositoryType::Proxy, _) => remote::get(ctx, path).await,
            (RepositoryType::Group, Route::Index(directory)) => {
                Ok(group::index(self, ctx, &directory)
                    .await?
                    .map(|found| found.asset().clone()))
            }
            (RepositoryType::Group, _) => {
                Ok(group::file(self, ctx, path).await?.map(|(_, found)| found))
            }
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![
                UploadField::text("directory", true)
                    .describe("Directory of the package, for example v3.20/main/x86_64; empty stores it under its architecture")
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
            .get("alpine.asset")
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest("alpine.asset is required".to_owned()))?;
        let directory = form
            .field("alpine.directory")
            .or_else(|| form.field("directory"))
            .unwrap_or("")
            .trim()
            .to_owned();
        hosted::upload(
            &ctx,
            &self.keys,
            &directory,
            UploadForm::stream(file).await?,
        )
        .await?;
        Ok(())
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout::default()
    }
}
