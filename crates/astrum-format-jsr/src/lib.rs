mod archive;
mod blobs;
mod documents;
mod hosted;
mod jsonc;
mod names;
mod publish;
mod remote;
mod reply;

use std::sync::{Arc, OnceLock};

use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Grant, Serve, View, group, serve, store,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use tokio::sync::Semaphore;

use crate::documents::Versions;
use crate::names::{Document, Item, Package, Route};

pub const ID: &str = "jsr";

const ALLOW_HOSTED: &str = "GET,HEAD,POST";
const ALLOW_READ: &str = "GET,HEAD";

const MAX_PUBLISHES: usize = 8;

pub use publish::{Hook, Limits};

#[derive(Default)]
pub struct Jsr {
    limits: Limits,
    intake: OnceLock<Arc<Semaphore>>,
    writing: OnceLock<Arc<Semaphore>>,
}

impl Jsr {
    pub fn with_limits(limits: Limits) -> Self {
        Self {
            limits,
            ..Self::default()
        }
    }
}

fn pool_size(db: &DatabaseConnection) -> Option<u32> {
    match db {
        DatabaseConnection::SqlxPostgresPoolConnection(_) => Some(
            db.get_postgres_connection_pool()
                .options()
                .get_max_connections(),
        ),
        _ => None,
    }
}

fn permits_for(pool: Option<u32>) -> usize {
    match pool {
        Some(pool) => (pool as usize / 2).clamp(1, MAX_PUBLISHES),
        None => MAX_PUBLISHES,
    }
}

fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => ALLOW_HOSTED,
            RepositoryType::Proxy | RepositoryType::Group => ALLOW_READ,
        },
    }
}

fn invalid_name() -> Response {
    reply::failure(
        StatusCode::BAD_REQUEST,
        "invalidPackageName",
        "a scope has 2 to 32 and a package 1 to 58 characters of a-z, 0-9 and -, not starting with -",
    )
}

fn is_read(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD)
}

fn generated(request: &ContentRequest, document: &Value) -> Response {
    let body = serde_json::to_vec(document).expect("a JSON value serialises");
    let etag = reply::etag_of(&body);
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
            .header(header::CONTENT_TYPE, reply::JSON_TYPE)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        (false, _) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, reply::JSON_TYPE)
            .body(Body::from(body)),
    };
    response.expect("a JSON response is valid")
}

fn fatal(error: &ContentError) -> bool {
    matches!(error, ContentError::Forbidden | ContentError::Unauthorized)
}

async fn members(ctx: &Ctx, package: &Package) -> Result<Vec<Ctx>, ContentError> {
    let mut members: Vec<Ctx> = group::members(ctx)
        .await?
        .into_iter()
        .map(|member| ctx.member(member))
        .collect();
    let mut owned = false;
    for member in members
        .iter()
        .filter(|member| member.kind() == RepositoryType::Hosted)
    {
        if hosted::known(member, package).await? {
            owned = true;
            break;
        }
    }
    if owned {
        members.retain(|member| member.kind() == RepositoryType::Hosted);
    }
    Ok(members)
}

fn visible(ctx: &Ctx, package: &Package, mut versions: Versions) -> Versions {
    if !matches!(ctx.grant(View::Read), Grant::Whole) {
        versions
            .retain(|(version, _)| ctx.allowed(View::Read, &package.version_meta_path(version)));
    }
    versions
}

fn serve_options(item: &Item) -> Serve {
    match &item.document {
        Document::File(_, file) => Serve {
            content_type: Some(names::content_type(file).to_owned()),
            ..Serve::download()
        },
        Document::VersionMeta(_) | Document::Listing => Serve {
            content_type: Some(reply::JSON_TYPE.to_owned()),
            ..Serve::default()
        },
    }
}

impl Jsr {
    async fn leaf(&self, member: &Ctx, item: &Item) -> Result<Option<Asset>, ContentError> {
        match member.kind() {
            RepositoryType::Hosted => match item.document {
                Document::Listing => Ok(None),
                _ => store::find(member.db(), member.name(), &item.path()).await,
            },
            RepositoryType::Proxy => remote::fetch(member, item).await,
            RepositoryType::Group => Ok(None),
        }
    }

    async fn asset(&self, ctx: &Ctx, item: &Item) -> Result<Option<(Ctx, Asset)>, ContentError> {
        if ctx.kind() != RepositoryType::Group {
            return Ok(self
                .leaf(ctx, item)
                .await?
                .map(|found| (ctx.clone(), found)));
        }
        for member in members(ctx, &item.package).await? {
            match self.leaf(&member, item).await {
                Ok(Some(found)) => return Ok(Some((member, found))),
                Ok(None) => {}
                Err(error) if fatal(&error) => return Err(error),
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                }
            }
        }
        Ok(None)
    }

    async fn group_listing(
        &self,
        ctx: &Ctx,
        item: &Item,
    ) -> Result<Option<Versions>, ContentError> {
        let mut lists = Vec::new();
        for member in members(ctx, &item.package).await? {
            let found = match member.kind() {
                RepositoryType::Hosted => hosted::listing(&member, &item.package).await,
                RepositoryType::Proxy => remote::listing(&member, item).await,
                RepositoryType::Group => Ok(None),
            };
            match found {
                Ok(Some(list)) => lists.push(list),
                Ok(None) => {}
                Err(error) if fatal(&error) => return Err(error),
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no usable listing");
                }
            }
        }
        match lists.is_empty() {
            true => Ok(None),
            false => Ok(Some(documents::merge(lists))),
        }
    }

    async fn read(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        item: Item,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &item.path())?;
        let missing = || ContentError::not_found(format!("{} was not found", item.path()));
        let listing = match (&item.document, ctx.kind()) {
            (Document::Listing, RepositoryType::Hosted) => {
                Some(hosted::listing(ctx, &item.package).await?)
            }
            (Document::Listing, RepositoryType::Group) => {
                self.group_listing(ctx, &item).await.map(Some)?
            }
            _ => None,
        };
        if let Some(found) = listing {
            let versions = found.ok_or_else(missing)?;
            let versions = visible(ctx, &item.package, versions);
            return Ok(generated(
                request,
                &documents::listing(&item.package, &versions),
            ));
        }
        let (owner, found) = self.asset(ctx, &item).await?.ok_or_else(missing)?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            serve_options(&item),
        )
        .await
    }

    fn gates(&self, ctx: &Ctx) -> publish::Gates {
        let intake = self
            .intake
            .get_or_init(|| Arc::new(Semaphore::new(self.limits.intake.max(1))));
        let writing = self.writing.get_or_init(|| {
            let permits = match self.limits.writes {
                Some(permits) => permits.max(1),
                None => permits_for(pool_size(ctx.db())),
            };
            Arc::new(Semaphore::new(permits))
        });
        publish::Gates {
            intake: intake.clone(),
            writing: writing.clone(),
            limits: self.limits.clone(),
        }
    }

    async fn writable(&self, ctx: &Ctx) -> Result<Option<Ctx>, ContentError> {
        Ok(group::writable_member(ctx)
            .await?
            .map(|member| ctx.member(member)))
    }

    async fn probe(&self, ctx: &Ctx, package: &Package) -> Result<Response, ContentError> {
        let may_write = match ctx.kind() {
            RepositoryType::Hosted => ctx.grant(View::Add).any(),
            RepositoryType::Group => {
                self.writable(ctx).await?.is_some() && ctx.grant(View::Add).any()
            }
            RepositoryType::Proxy => return Err(not_allowed(&Method::POST, ctx.kind())),
        };
        if !may_write {
            ctx.require(View::Read, &package.listing_path())?;
        }
        let known = may_write
            || match ctx.kind() {
                RepositoryType::Group => {
                    let item = Item {
                        package: package.clone(),
                        document: Document::Listing,
                    };
                    self.group_listing(ctx, &item).await?.is_some()
                }
                _ => hosted::known(ctx, package).await?,
            };
        if known {
            return Ok(reply::json(
                StatusCode::OK,
                &json!({"scope": package.scope, "name": package.name}),
            ));
        }
        Ok(reply::failure(
            StatusCode::NOT_FOUND,
            "packageNotFound",
            &format!("{} does not exist in {}", package.display(), ctx.name()),
        ))
    }

    async fn publish_target(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Ctx, Box<Response>> {
        let refused = |message: &str| {
            let mut response =
                reply::failure(StatusCode::METHOD_NOT_ALLOWED, "methodNotAllowed", message);
            response
                .headers_mut()
                .insert(header::ALLOW, header::HeaderValue::from_static(ALLOW_READ));
            response
        };
        match ctx.kind() {
            RepositoryType::Hosted => Ok(ctx.clone()),
            RepositoryType::Proxy => {
                publish::drain(request).await;
                Err(Box::new(refused(
                    "publishing is not supported: a proxy repository is read-only, publish to a hosted repository",
                )))
            }
            RepositoryType::Group => match self.writable(ctx).await {
                Ok(Some(target)) => Ok(target),
                Ok(None) => {
                    publish::drain(request).await;
                    Err(Box::new(refused(
                        "publishing is not supported: the group has no writable member, set one over the REST API or publish to a hosted repository",
                    )))
                }
                Err(error) => Err(Box::new(reply::from_error(error))),
            },
        }
    }

    async fn api(&self, ctx: &Ctx, request: &ContentRequest, route: Route) -> Response {
        let read = is_read(&request.method);
        let answered = match route {
            Route::Probe(scope, name) if read => match Package::new(&scope, &name) {
                Some(package) => self.probe(ctx, &package).await,
                None => return invalid_name(),
            },
            Route::Publish(scope, name, version) if request.method == Method::POST => {
                let target = match self.publish_target(ctx, request).await {
                    Ok(target) => target,
                    Err(refusal) => return *refusal,
                };
                let Some(package) = Package::new(&scope, &name) else {
                    publish::drain(request).await;
                    return invalid_name();
                };
                if names::parse_version(&version).is_none() {
                    publish::drain(request).await;
                    return reply::failure(
                        StatusCode::BAD_REQUEST,
                        "invalidVersion",
                        &format!("{version} is not a semantic version such as 1.2.3"),
                    );
                }
                return match publish::publish(
                    ctx,
                    &target,
                    &self.gates(ctx),
                    request,
                    &package,
                    &version,
                )
                .await
                {
                    Ok(response) => response,
                    Err(failure) => failure.into_response(),
                };
            }
            Route::Probe(..) | Route::Publish(..) => Err(not_allowed(&request.method, ctx.kind())),
            Route::Task | Route::Api => Ok(reply::failure(
                StatusCode::NOT_FOUND,
                "notFound",
                "this registry has no such call; publish with deno publish --token",
            )),
            Route::Item(_) | Route::Other => Err(ContentError::not_found("not found")),
        };
        answered.unwrap_or_else(reply::from_error)
    }
}

#[async_trait]
impl Format for Jsr {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match names::route(&request.path) {
            Route::Item(item) if is_read(&request.method) => self.read(&ctx, &request, item).await,
            Route::Item(_) => Err(not_allowed(&request.method, ctx.kind())),
            Route::Other if is_read(&request.method) => {
                Err(ContentError::not_found(request.path.clone()))
            }
            Route::Other => Err(not_allowed(&request.method, ctx.kind())),
            route => Ok(self.api(&ctx, &request, route).await),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match names::route(path) {
            Route::Item(item) => Ok(self.asset(ctx, &item).await?.map(|(_, found)| found)),
            _ => Ok(None),
        }
    }
}
