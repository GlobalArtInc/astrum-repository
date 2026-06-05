mod chart;
mod hosted;
pub mod import;
mod index;
mod provenance;
mod push;
mod remote;

use astrum_content::format::SearchTarget;
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, UploadField,
    UploadForm, UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};

pub use chart::Chart;
pub use index::Listing;

pub const ID: &str = "helm";
pub const INDEX: &str = "/index.yaml";
pub const INDEX_KIND: &str = "index";
pub const INDEX_TYPE: &str = "text/x-yaml";
pub const CHART: &str = "chart";
pub const CHART_TYPE: &str = "application/x-tgz";
pub const PROVENANCE: &str = "provenance";
pub const PROVENANCE_TYPE: &str = "text/plain";
pub const PROVENANCE_SUFFIX: &str = ".prov";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "helm.appVersion",
        target: SearchTarget::Component,
        pointer: &["helm", "appVersion"],
    },
    SearchKey {
        param: "helm.description",
        target: SearchTarget::Component,
        pointer: &["helm", "description"],
    },
    SearchKey {
        param: "helm.keywords",
        target: SearchTarget::Component,
        pointer: &["helm", "keywords"],
    },
];

#[derive(Default)]
pub struct Helm {
    cache: remote::Cache,
}

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Index,
    Chart(String),
    Provenance(String),
    PushChart,
    PushProvenance,
    Directory,
    Other,
}

fn route(path: &str) -> Route {
    if path == INDEX {
        return Route::Index;
    }
    if path.ends_with('/') {
        return Route::Directory;
    }
    match path {
        "/api/charts" => return Route::PushChart,
        "/api/prov" => return Route::PushProvenance,
        _ => {}
    }
    let name = path.trim_start_matches('/');
    if name.contains('/') || name.starts_with('.') {
        return Route::Other;
    }
    match name.strip_suffix(PROVENANCE_SUFFIX) {
        Some(chart) if chart.ends_with(".tgz") && chart.len() > 4 => {
            Route::Provenance(name.to_owned())
        }
        Some(_) => Route::Other,
        None if name.ends_with(".tgz") && name.len() > 4 => Route::Chart(name.to_owned()),
        None => Route::Other,
    }
}

fn index_response(request: &ContentRequest, body: String) -> Result<Response, ContentError> {
    let etag = format!(
        "\"{}\"",
        hex::encode(&Sha256::digest(body.as_bytes())[..20])
    );
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    if unchanged {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    let length = body.len();
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, INDEX_TYPE)
        .header(header::CONTENT_LENGTH, length)
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Helm chart repository. Add it with <code>helm repo add {name} {url}</code>; its charts are listed in <a href=\"{url}/index.yaml\">index.yaml</a>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn allow(ctx: &Ctx) -> &'static str {
    match ctx.kind() {
        RepositoryType::Hosted => "GET,HEAD,PUT,POST,DELETE",
        RepositoryType::Proxy => "GET,HEAD,DELETE",
        RepositoryType::Group => "GET,HEAD",
    }
}

fn readable(ctx: &Ctx, listing: &mut Listing) {
    if matches!(ctx.grant(View::Read), astrum_content::Grant::Whole) {
        return;
    }
    for charts in listing.entries.values_mut() {
        charts.retain(|chart| ctx.allowed(View::Read, &format!("/{}", chart.file_name())));
    }
    listing.entries.retain(|_, charts| !charts.is_empty());
}

impl Helm {
    async fn leaf_listing(&self, ctx: &Ctx) -> Result<Listing, ContentError> {
        match ctx.kind() {
            RepositoryType::Proxy => Ok(remote::index(ctx, &self.cache)
                .await?
                .map(|parsed| parsed.upstream.listing.clone())
                .unwrap_or_default()),
            _ => hosted::listing(ctx).await,
        }
    }

    async fn listing(&self, ctx: &Ctx) -> Result<Option<Listing>, ContentError> {
        let mut listing = match ctx.kind() {
            RepositoryType::Group => {
                let mut listings = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match self.leaf_listing(&member).await {
                        Ok(listing) => listings.push(listing),
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no chart index");
                        }
                    }
                }
                index::merge(listings)
            }
            RepositoryType::Proxy => match remote::index(ctx, &self.cache).await? {
                Some(parsed) => parsed.upstream.listing.clone(),
                None => return Ok(None),
            },
            RepositoryType::Hosted => hosted::listing(ctx).await?,
        };
        readable(ctx, &mut listing);
        Ok(Some(listing))
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = &request.path;
        match route(path) {
            Route::Index => {
                ctx.require(View::Read, path)?;
                let listing = self
                    .listing(ctx)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.clone()))?;
                let body = index::render(&listing).map_err(ContentError::internal)?;
                index_response(request, body)
            }
            Route::Chart(_) | Route::Provenance(_) => {
                ctx.require(View::Read, path)?;
                let asset = self
                    .get(ctx, path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.clone()))?;
                serve(
                    ctx,
                    &asset,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            Route::Directory => directory_page(ctx),
            _ => Err(ContentError::not_found(path.clone())),
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        let route = route(path);
        let deletable = match ctx.kind() {
            RepositoryType::Hosted => matches!(route, Route::Chart(_) | Route::Provenance(_)),
            _ => matches!(route, Route::Chart(_) | Route::Provenance(_) | Route::Index),
        };
        if !deletable {
            return Err(ContentError::not_found(path.to_owned()));
        }
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        if let Route::Chart(_) = route {
            let companion = format!("{path}{PROVENANCE_SUFFIX}");
            if ctx.allowed(View::Delete, &companion) {
                store::delete(ctx, &companion).await?;
            }
        }
        if route == Route::Index {
            self.cache.forget(ctx.name());
        }
        Ok(StatusCode::OK.into_response())
    }

    async fn write(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (route(&request.path), &request.method) {
            (Route::Chart(file), &Method::PUT) => {
                hosted::put_chart(ctx, &file, request.into_stream()).await?;
                Ok(StatusCode::OK.into_response())
            }
            (Route::Provenance(file), &Method::PUT) => {
                let text = request.bytes(provenance::MAX_PROVENANCE).await?;
                hosted::put_provenance(ctx, Some(&file), &text).await?;
                Ok(StatusCode::OK.into_response())
            }
            (Route::PushChart, &Method::POST) => Ok(push::charts(ctx, request)
                .await
                .unwrap_or_else(IntoResponse::into_response)),
            (Route::PushProvenance, &Method::POST) => Ok(push::provenance(ctx, request)
                .await
                .unwrap_or_else(IntoResponse::into_response)),
            (Route::Chart(_) | Route::Provenance(_), &Method::POST)
            | (Route::PushChart | Route::PushProvenance, &Method::PUT) => {
                Err(ContentError::MethodNotAllowed {
                    method: request.method.to_string(),
                    allow: allow(ctx),
                })
            }
            (Route::Index, _) => Err(ContentError::MethodNotAllowed {
                method: request.method.to_string(),
                allow: "GET,HEAD",
            }),
            _ => Err(ContentError::BadRequest(format!(
                "{} is not a place for charts: upload <name>-<version>.tgz at the top of the repository or POST it to api/charts",
                request.path
            ))),
        }
    }
}

#[async_trait]
impl Format for Helm {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::DELETE, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (&Method::PUT | &Method::POST, RepositoryType::Hosted) => {
                self.write(&ctx, request).await
            }
            (method, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: allow(&ctx),
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            RepositoryType::Proxy => remote::file(ctx, &self.cache, path).await,
            RepositoryType::Group => {
                Ok(
                    group::first(ctx, |member| async move { self.get(&member, path).await })
                        .await?
                        .map(|(_, asset)| asset),
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
            .get("helm.asset")
            .ok_or_else(|| ContentError::BadRequest("helm.asset is required".to_owned()))?;
        hosted::upload_chart(&ctx, UploadForm::stream(file).await?).await?;
        Ok(())
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE '/' || c.name || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[INDEX],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_index_charts_and_the_push_api() {
        assert_eq!(route("/index.yaml"), Route::Index);
        assert_eq!(
            route("/demo-0.1.0.tgz"),
            Route::Chart("demo-0.1.0.tgz".to_owned())
        );
        assert_eq!(
            route("/demo-0.1.0.tgz.prov"),
            Route::Provenance("demo-0.1.0.tgz.prov".to_owned())
        );
        assert_eq!(route("/api/charts"), Route::PushChart);
        assert_eq!(route("/api/prov"), Route::PushProvenance);
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/charts/"), Route::Directory);
        for other in [
            "/charts/demo-0.1.0.tgz",
            "/demo.zip",
            "/.tgz",
            "/.hidden.tgz",
            "/x.prov",
            "/index.yml",
            "/api/charts/demo",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }
}
