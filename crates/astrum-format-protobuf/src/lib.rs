mod account;
mod catalog;
mod commits;
pub mod connect;
pub mod digest;
mod labels;
pub mod model;
mod modules;
pub mod names;
pub mod proto;
mod resolve;
mod upload;

use astrum_content::store;
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;

use crate::connect::{Code, Status, failure, unary};

pub const ID: &str = "protobuf";
const ALLOW: &str = "GET,HEAD,POST";

const MODULE_SERVICE: &str = "buf.registry.module.v1.ModuleService";
const UPLOAD_SERVICE: &str = "buf.registry.module.v1.UploadService";
const COMMIT_SERVICE: &str = "buf.registry.module.v1.CommitService";
const GRAPH_SERVICE: &str = "buf.registry.module.v1.GraphService";
const DOWNLOAD_SERVICE: &str = "buf.registry.module.v1.DownloadService";
const LABEL_SERVICE: &str = "buf.registry.module.v1.LabelService";
const RESOURCE_SERVICE: &str = "buf.registry.module.v1.ResourceService";
const OWNER_SERVICE: &str = "buf.registry.owner.v1.OwnerService";
const AUTHN_SERVICE: &str = "buf.alpha.registry.v1alpha1.AuthnService";

pub struct Protobuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Route<'a> {
    Root,
    Rpc(&'a str, &'a str),
    Other,
}

pub(crate) fn route(path: &str) -> Route<'_> {
    if path.is_empty() || path == "/" {
        return Route::Root;
    }
    let mut parts = path.trim_start_matches('/').split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(service), Some(method), None)
            if service.contains('.') && !method.is_empty() && !method.contains('.') =>
        {
            Route::Rpc(service, method)
        }
        _ => Route::Other,
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let host = escape(request.header("host").unwrap_or("registry.example.com"));
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Buf Schema Registry compatible repository for Protobuf modules. It must be served at the root of its own domain, because buf sends its calls to the host without a path.</p>\n    <p>Name a module <code>{host}/&lt;owner&gt;/&lt;module&gt;</code> in <code>buf.yaml</code>, sign in with <code>buf registry login {host}</code> and publish it with <code>buf push --create --create-visibility private</code>.</p>\n    <p>The modules of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

async fn rpc(ctx: &Ctx, request: &ContentRequest, service: &str, method: &str) -> Response {
    match (service, method) {
        (MODULE_SERVICE, "GetModules") => {
            unary(request, false, |query| modules::get(ctx, query)).await
        }
        (MODULE_SERVICE, "ListModules") => {
            unary(request, false, |query| modules::list(ctx, query)).await
        }
        (MODULE_SERVICE, "CreateModules") => {
            unary(request, true, |query| modules::create(ctx, query)).await
        }
        (MODULE_SERVICE, "UpdateModules") => {
            unary(request, true, |query| modules::update(ctx, query)).await
        }
        (MODULE_SERVICE, "DeleteModules") => {
            unary(request, true, |query| modules::delete(ctx, query)).await
        }
        (UPLOAD_SERVICE, "Upload") => {
            unary(request, true, |query| upload::upload(ctx, query)).await
        }
        (COMMIT_SERVICE, "GetCommits") => {
            unary(request, false, |query| commits::get(ctx, query)).await
        }
        (COMMIT_SERVICE, "ListCommits") => {
            unary(request, false, |query| commits::list(ctx, query)).await
        }
        (GRAPH_SERVICE, "GetGraph") => {
            unary(request, false, |query| commits::graph(ctx, query)).await
        }
        (DOWNLOAD_SERVICE, "Download") => {
            unary(request, false, |query| commits::download(ctx, query)).await
        }
        (LABEL_SERVICE, "GetLabels") => {
            unary(request, false, |query| labels::get(ctx, query)).await
        }
        (LABEL_SERVICE, "ListLabels") => {
            unary(request, false, |query| labels::list(ctx, query)).await
        }
        (LABEL_SERVICE, "ListLabelHistory") => {
            unary(request, false, |query| labels::history(ctx, query)).await
        }
        (LABEL_SERVICE, "CreateOrUpdateLabels") => {
            unary(request, true, |query| labels::create_or_update(ctx, query)).await
        }
        (LABEL_SERVICE, "ArchiveLabels") => {
            unary(request, true, |query| labels::archive(ctx, query)).await
        }
        (LABEL_SERVICE, "UnarchiveLabels") => {
            unary(request, true, |query| labels::unarchive(ctx, query)).await
        }
        (RESOURCE_SERVICE, "GetResources") => {
            unary(request, false, |query| commits::resources(ctx, query)).await
        }
        (OWNER_SERVICE, "GetOwners") => {
            unary(request, false, |query| commits::owners(ctx, query)).await
        }
        (AUTHN_SERVICE, "GetCurrentUser") => {
            unary(request, false, |query| account::current_user(ctx, query)).await
        }
        (AUTHN_SERVICE, "GetCurrentUserSubject") => {
            unary(request, false, |query| account::current_subject(ctx, query)).await
        }
        _ => failure(&Status::new(
            Code::Unimplemented,
            format!("{service}/{method} is not implemented by this registry"),
        )),
    }
}

impl Protobuf {
    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        ctx.require(View::Read, path)?;
        let found = store::find(ctx.db(), ctx.name(), path)
            .await?
            .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
        serve(
            ctx,
            &found,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }
}

#[async_trait]
impl Format for Protobuf {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "Protobuf registries are hosted repositories".to_owned(),
            ));
        }
        match route(&request.path) {
            Route::Rpc(service, method) => Ok(rpc(&ctx, &request, service, method).await),
            Route::Root => match request.method {
                Method::GET | Method::HEAD => directory_page(&ctx, &request),
                _ => Err(ContentError::MethodNotAllowed {
                    method: request.method.to_string(),
                    allow: "GET,HEAD",
                }),
            },
            Route::Other => match request.method {
                Method::GET | Method::HEAD => self.read(&ctx, &request).await,
                _ => Err(ContentError::MethodNotAllowed {
                    method: request.method.to_string(),
                    allow: ALLOW,
                }),
            },
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        store::find(ctx.db(), ctx.name(), path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_rpc_methods_the_root_page_or_the_stored_files() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route(""), Route::Root);
        assert_eq!(
            route("/buf.registry.module.v1.ModuleService/GetModules"),
            Route::Rpc("buf.registry.module.v1.ModuleService", "GetModules")
        );
        assert_eq!(
            route("/buf.registry.owner.v1.OwnerService/GetOwners"),
            Route::Rpc("buf.registry.owner.v1.OwnerService", "GetOwners")
        );
        for other in [
            "/acme/greeter/module.json",
            "/acme/greeter/commits/abc/commit.json",
            "/buf.registry.module.v1.ModuleService",
            "/buf.registry.module.v1.ModuleService/GetModules/x",
            "/acme/owner.json",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }
}
