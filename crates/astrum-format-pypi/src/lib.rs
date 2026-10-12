mod archive;
mod hosted;
mod html;
mod index;
mod json;
mod metadata;
mod name;
mod negotiate;
mod remote;
mod upload;

use astrum_content::format::SearchTarget;
use astrum_content::store;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, UploadField, UploadForm,
    UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use sha2::{Digest, Sha256};

pub use index::{File, Project};
pub use name::normalize;

use crate::index::{merge, merge_names};
use crate::negotiate::Wanted;

pub const ID: &str = "pypi";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "pypi.classifiers",
        target: SearchTarget::Component,
        pointer: &["pypi", "classifiers"],
    },
    SearchKey {
        param: "pypi.description",
        target: SearchTarget::Component,
        pointer: &["pypi", "description"],
    },
    SearchKey {
        param: "pypi.keywords",
        target: SearchTarget::Component,
        pointer: &["pypi", "keywords"],
    },
    SearchKey {
        param: "pypi.summary",
        target: SearchTarget::Component,
        pointer: &["pypi", "summary"],
    },
];

pub struct PyPi;

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Root { slash: bool },
    Project { name: String, slash: bool },
    Package,
    Directory,
    Other,
}

fn route(path: &str) -> Route {
    let slash = path.ends_with('/');
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    match parts.as_slice() {
        ["simple"] => Route::Root { slash },
        ["simple", project] => Route::Project {
            name: (*project).to_owned(),
            slash,
        },
        ["packages", _, _, _] if !slash => Route::Package,
        _ if slash => Route::Directory,
        _ => Route::Other,
    }
}

fn page(request: &ContentRequest, wanted: Wanted, body: String) -> Result<Response, ContentError> {
    let content_type = wanted.content_type();
    let mut digest = Sha256::new();
    digest.update(content_type.as_bytes());
    digest.update(body.as_bytes());
    let etag = format!("\"{}\"", hex::encode(&digest.finalize()[..20]));
    let builder = Response::builder()
        .header(header::VARY, "Accept")
        .header(header::ETAG, &etag);
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    if unchanged {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    let builder = builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len());
    let body = match request.method {
        Method::HEAD => Body::empty(),
        _ => Body::from(body),
    };
    builder.body(body).map_err(ContentError::internal)
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = html::escape(ctx.name());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a PyPI repository. Point pip at <code>{}/simple/</code>; there is no file listing at this address.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        html::escape(&ctx.repository_url()),
        html::escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

impl PyPi {
    async fn names(&self, ctx: &Ctx) -> Result<Vec<String>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => {
                let mut lists = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match leaf_names(&member).await {
                        Ok(names) => lists.push(names),
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no project list");
                        }
                    }
                }
                Ok(merge_names(lists))
            }
            _ => Ok(merge_names(vec![leaf_names(ctx).await?])),
        }
    }

    async fn project(&self, ctx: &Ctx, project: &str) -> Result<Option<Project>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => {
                let mut pages = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match leaf_project(&member, project).await {
                        Ok(Some(found)) => pages.push(found),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no page for the project");
                        }
                    }
                }
                Ok(merge(pages))
            }
            _ => leaf_project(ctx, project).await,
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        match route(&request.path) {
            Route::Root { slash } => {
                ctx.require(View::Read, &request.path)?;
                let names = self.names(ctx).await?;
                let wanted = wanted(request);
                let body = match wanted {
                    Wanted::Json => json::render_root(&names),
                    Wanted::Html(_) => {
                        html::render_root(&names, if slash { "" } else { "simple/" })
                    }
                };
                page(request, wanted, body)
            }
            Route::Project { name, slash } => {
                ctx.require(View::Read, &request.path)?;
                let project = self
                    .project(ctx, &name::normalize(&name))
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                let prefix = if slash { "../../" } else { "../" };
                let wanted = wanted(request);
                let body = match wanted {
                    Wanted::Json => json::render_project(&project, prefix),
                    Wanted::Html(_) => html::render_project(&project, prefix),
                };
                page(request, wanted, body)
            }
            Route::Package => {
                ctx.require(View::Read, &request.path)?;
                let asset = self
                    .get(ctx, &request.path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
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
            Route::Other => Err(ContentError::not_found(request.path.clone())),
        }
    }
}

fn wanted(request: &ContentRequest) -> Wanted {
    let format = negotiate::format_parameter(request.query.as_deref());
    negotiate::choose(request.header("accept"), format.as_deref())
}

async fn leaf_names(ctx: &Ctx) -> Result<Vec<String>, ContentError> {
    match ctx.kind() {
        RepositoryType::Proxy => remote::names(ctx).await,
        _ => hosted::names(ctx).await,
    }
}

async fn leaf_project(ctx: &Ctx, project: &str) -> Result<Option<Project>, ContentError> {
    match ctx.kind() {
        RepositoryType::Proxy => Ok(remote::page(ctx, project).await?.map(Project::localize)),
        _ => hosted::project(ctx, project).await,
    }
}

#[async_trait]
impl Format for PyPi {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match request.method {
            Method::GET | Method::HEAD => self.read(&ctx, &request).await,
            Method::POST if ctx.kind() == RepositoryType::Hosted && request.path == "/" => {
                upload::legacy(&ctx, request).await
            }
            Method::POST => Err(ContentError::not_found(request.path.clone())),
            method => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: match ctx.kind() {
                    RepositoryType::Hosted => "GET,HEAD,POST",
                    _ => "GET,HEAD",
                },
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            RepositoryType::Proxy => remote::file(ctx, path).await,
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
            .get("pypi.asset")
            .ok_or_else(|| ContentError::BadRequest("pypi.asset is required".to_owned()))?;
        let filename = hosted::file_name(file.filename.as_deref())?;
        let source = archive::LocalFile {
            path: file.path.clone(),
            size: file.size,
        };
        let text = archive::metadata(&source, &filename)
            .await
            .map_err(|error| ContentError::BadRequest(error.to_string()))?
            .ok_or_else(|| {
                ContentError::BadRequest(format!(
                    "{filename} holds no package metadata (PKG-INFO or METADATA)"
                ))
            })?;
        let package = hosted::Package {
            metadata: metadata::CoreMetadata::parse(&text),
            filename,
        };
        hosted::store_package(&ctx, &package, UploadForm::stream(file).await?).await?;
        Ok(())
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_pages_and_files() {
        assert_eq!(route("/simple/"), Route::Root { slash: true });
        assert_eq!(route("/simple"), Route::Root { slash: false });
        assert_eq!(
            route("/simple/Foo_Bar/"),
            Route::Project {
                name: "Foo_Bar".to_owned(),
                slash: true
            }
        );
        assert_eq!(
            route("/simple/six"),
            Route::Project {
                name: "six".to_owned(),
                slash: false
            }
        );
        assert_eq!(route("/packages/six/1.0/six-1.0.tar.gz"), Route::Package);
        assert_eq!(route("/packages/six/1.0/"), Route::Directory);
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/simple/six/x"), Route::Other);
        assert_eq!(route("/other/path"), Route::Other);
    }
}
