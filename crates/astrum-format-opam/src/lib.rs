mod hosted;
mod index;
pub mod names;
mod remote;

use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Serve, View, serve, store,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;

use crate::hosted::{Writers, Written};
use crate::names::{REPO_PATH, REPO_TEXT, Route};

pub const ID: &str = "opam";
pub const OPAM_KIND: &str = "opam";
pub const FILE_KIND: &str = "file";
pub const CACHE_KIND: &str = "cache";
pub const INDEX_KIND: &str = "index";
pub const REPO_KIND: &str = "repo";

const ALLOW_HOSTED: &str = "GET,HEAD,PUT,DELETE";
const ALLOW_PROXY: &str = "GET,HEAD";

pub struct Opam {
    max_entries: usize,
    writers: Writers,
    flights: remote::Flights,
}

impl Default for Opam {
    fn default() -> Self {
        Self::with_max_entries(hosted::MAX_ENTRIES)
    }
}

impl Opam {
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            max_entries,
            writers: Writers::default(),
            flights: remote::Flights::default(),
        }
    }
}

fn is_read(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD)
}

fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => ALLOW_HOSTED,
            RepositoryType::Proxy | RepositoryType::Group => ALLOW_PROXY,
        },
    }
}

fn status_only(status: StatusCode) -> Result<Response, ContentError> {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .map_err(ContentError::internal)
}

fn message_only(status: StatusCode, message: String) -> Result<Response, ContentError> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain;charset=utf-8")
        .body(Body::from(message))
        .map_err(ContentError::internal)
}

fn generated_repo(method: &Method) -> Result<Response, ContentError> {
    let builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache");
    match *method {
        Method::HEAD => builder
            .header(header::CONTENT_LENGTH, REPO_TEXT.len())
            .body(Body::empty()),
        _ => builder.body(Body::from(REPO_TEXT)),
    }
    .map_err(ContentError::internal)
}

fn refusal(route: &Route) -> ContentError {
    match route {
        Route::Repo | Route::Index => ContentError::BadRequest(
            "repo and index.tar.gz are generated from the packages of a hosted OPAM repository"
                .to_owned(),
        ),
        Route::Refused(message) => ContentError::BadRequest(message.clone()),
        _ => ContentError::BadRequest(
            "a hosted OPAM repository holds packages/<name>/<name>.<version>/opam, packages/<name>/<name>.<version>/files/<path> and cache/<sha256|sha512>/<xx>/<hex digest>"
                .to_owned(),
        ),
    }
}

impl Opam {
    async fn read_hosted(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        match route {
            Route::Repo => {
                ctx.require(View::Read, REPO_PATH)?;
                generated_repo(&request.method)
            }
            Route::Index => {
                let found = hosted::index_asset(ctx, &self.writers, self.max_entries).await?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::default(),
                )
                .await
            }
            Route::Package(_) | Route::Cache(_) => {
                let path = match &route {
                    Route::Package(file) => file.path(),
                    Route::Cache(key) => key.path(),
                    _ => unreachable!("only packages and cache files are stored"),
                };
                ctx.require(View::Read, &path)?;
                let found = store::find(ctx.db(), ctx.name(), &path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(format!("{path} was not found")))?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            Route::Refused(_) | Route::Other => Err(ContentError::not_found(request.path.clone())),
        }
    }

    async fn write_hosted(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        let status = match (&route, &request.method) {
            (Route::Package(file), &Method::PUT) => {
                match hosted::put_package(ctx, &self.writers, self.max_entries, file, request)
                    .await?
                {
                    Written::Status(status) => status,
                    Written::TooLarge(message) => {
                        return message_only(StatusCode::PAYLOAD_TOO_LARGE, message);
                    }
                }
            }
            (Route::Package(file), _) => {
                hosted::delete_package(ctx, &self.writers, self.max_entries, file).await?
            }
            (Route::Cache(key), &Method::PUT) => hosted::put_cache(ctx, key, request).await?,
            (Route::Cache(key), _) => hosted::delete_cache(ctx, key).await?,
            _ => return Err(refusal(&route)),
        };
        status_only(status)
    }

    async fn read_proxy(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        route: &Route,
    ) -> Result<Response, ContentError> {
        let (path, options) = match route {
            Route::Index => (names::INDEX_PATH.to_owned(), Serve::default()),
            Route::Repo => (REPO_PATH.to_owned(), Serve::default()),
            Route::Cache(key) => (key.path(), Serve::download()),
            _ => return Err(ContentError::not_found(request.path.clone())),
        };
        ctx.require(View::Read, &path)?;
        let found = self
            .proxied(ctx, route)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("{path} was not found")))?;
        serve(ctx, &found, &request.method, &request.headers, options).await
    }

    async fn proxied(&self, ctx: &Ctx, route: &Route) -> Result<Option<Asset>, ContentError> {
        match route {
            Route::Index => remote::index(ctx).await,
            Route::Repo => remote::repo(ctx).await,
            Route::Cache(key) => remote::cache(ctx, &self.flights, key).await,
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl Format for Opam {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = names::parse(&request.path);
        match ctx.kind() {
            RepositoryType::Hosted => match &request.method {
                method if is_read(method) => self.read_hosted(&ctx, &request, route).await,
                &Method::PUT | &Method::DELETE => self.write_hosted(&ctx, request, route).await,
                method => Err(not_allowed(method, ctx.kind())),
            },
            RepositoryType::Proxy => match &request.method {
                method if is_read(method) => self.read_proxy(&ctx, &request, &route).await,
                method => Err(not_allowed(method, ctx.kind())),
            },
            RepositoryType::Group => Err(ContentError::not_found("OPAM has no group repositories")),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            RepositoryType::Proxy => self.proxied(ctx, &names::parse(path)).await,
            RepositoryType::Group => Ok(None),
        }
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: None,
            generated: &[REPO_PATH],
        }
    }
}
