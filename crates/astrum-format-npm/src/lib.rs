mod attachment;
mod documents;
mod hosted;
mod packument;
mod path;
mod reply;
mod search;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::store::PutAsset;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use serde_json::{Value, json};

use crate::documents::not_found;
use crate::path::{Name, Route};

pub const ID: &str = "npm";
pub const REALM: &str = "NpmToken";
pub const PACKUMENT: &str = "packument";
pub const TARBALL: &str = "tarball";
pub const TARBALL_TYPE: &str = "application/x-tgz";

const SEARCH_KEYS: &[SearchKey] = &[SearchKey {
    param: "npm.scope",
    target: SearchTarget::Component,
    pointer: &["npm", "scope"],
}];

type Locks = HashMap<(String, Name), Weak<tokio::sync::Mutex<()>>>;

#[derive(Default)]
pub struct Npm {
    locks: Mutex<Locks>,
}

impl Npm {
    async fn lock(&self, ctx: &Ctx, name: &Name) -> tokio::sync::OwnedMutexGuard<()> {
        let lock = {
            let mut locks = self
                .locks
                .lock()
                .expect("the npm lock map is never poisoned");
            let key = (ctx.name().to_owned(), name.clone());
            match locks.get(&key).and_then(Weak::upgrade) {
                Some(existing) => existing,
                None => {
                    locks.retain(|_, weak| weak.strong_count() > 0);
                    let fresh = Arc::new(tokio::sync::Mutex::new(()));
                    locks.insert(key, Arc::downgrade(&fresh));
                    fresh
                }
            }
        };
        lock.lock_owned().await
    }

    async fn answer(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let method = request.method.clone();
        let read = matches!(method, Method::GET | Method::HEAD);
        match path::route(&request.path) {
            Route::Ping if read => Ok(reply::json(StatusCode::OK, &json!({}))),
            Route::Whoami if read => whoami(ctx),
            Route::Search if read => search::search(ctx, &request).await,
            Route::WebLogin => Err(ContentError::not_found(
                "web sign-in is not offered; use npm login --auth-type=legacy",
            )),
            Route::Audit => Err(ContentError::not_found(
                "security audits are not offered by this repository",
            )),
            Route::DistTags { name, tag } => match (method, tag) {
                (Method::GET | Method::HEAD, tag) => self.tags(ctx, &name, tag.as_deref()).await,
                (Method::PUT | Method::POST, Some(tag)) => {
                    hosted::set_tag(self, ctx, &name, &tag, request).await
                }
                (Method::DELETE, Some(tag)) => hosted::remove_tag(self, ctx, &name, &tag).await,
                (Method::PUT, None) => hosted::replace_tags(self, ctx, &name, request, false).await,
                (Method::POST, None) => hosted::replace_tags(self, ctx, &name, request, true).await,
                (method, _) => Err(not_allowed(ctx, method)),
            },
            Route::Packument { name } => match method {
                Method::GET | Method::HEAD => self.packument(ctx, &name, &request).await,
                Method::PUT => hosted::put(self, ctx, &name, request).await,
                Method::DELETE => hosted::unpublish(self, ctx, &name).await,
                method => Err(not_allowed(ctx, method)),
            },
            Route::Version { name, version } if read => {
                self.version(ctx, &name, &version, &request).await
            }
            Route::Tarball { name, file } => match method {
                Method::GET | Method::HEAD => self.tarball(ctx, &name, &file, &request).await,
                Method::DELETE => hosted::delete_tarball(self, ctx, &name, &file).await,
                method => Err(not_allowed(ctx, method)),
            },
            Route::Unknown | Route::User => Err(ContentError::not_found("not found")),
            _ => Err(not_allowed(ctx, method)),
        }
    }

    async fn packument(
        &self,
        ctx: &Ctx,
        name: &Name,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &name.packument_path())?;
        let loaded = documents::load(ctx, name)
            .await?
            .ok_or_else(|| not_found(name))?;
        let mut doc = loaded.doc;
        packument::rewrite_tarballs(&mut doc, &ctx.repository_url(), name);
        let for_writing = request
            .query
            .as_deref()
            .is_some_and(|query| query.split('&').any(|pair| pair == "write=true"));
        if reply::wants_abbreviated(request) && !for_writing {
            let short = packument::abbreviate(&doc);
            return reply::document(request, &short, reply::ABBREVIATED, loaded.modified);
        }
        reply::document(request, &doc, reply::JSON, loaded.modified)
    }

    async fn version(
        &self,
        ctx: &Ctx,
        name: &Name,
        wanted: &str,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &name.packument_path())?;
        let loaded = documents::load(ctx, name)
            .await?
            .ok_or_else(|| not_found(name))?;
        let mut doc = loaded.doc;
        packument::rewrite_tarballs(&mut doc, &ctx.repository_url(), name);
        let manifest = packument::resolve(&doc, wanted).ok_or_else(|| {
            ContentError::not_found(format!("version {wanted} of {name} not found"))
        })?;
        reply::document(request, manifest, reply::JSON, loaded.modified)
    }

    async fn tags(
        &self,
        ctx: &Ctx,
        name: &Name,
        tag: Option<&str>,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &name.packument_path())?;
        let loaded = documents::load(ctx, name)
            .await?
            .ok_or_else(|| not_found(name))?;
        let tags = packument::dist_tags(&loaded.doc);
        let value = match tag {
            None => Value::Object(tags),
            Some(tag) => tags
                .get(tag)
                .cloned()
                .ok_or_else(|| ContentError::not_found(format!("{name} has no tag {tag}")))?,
        };
        Ok(reply::json(StatusCode::OK, &value))
    }

    async fn tarball(
        &self,
        ctx: &Ctx,
        name: &Name,
        file: &str,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let path = name.tarball_path(file);
        ctx.require(View::Read, &path)?;
        let asset = self
            .get(ctx, &path)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("{path} not found")))?;
        serve(
            ctx,
            &asset,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }
}

fn not_allowed(ctx: &Ctx, method: Method) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match ctx.kind() {
            RepositoryType::Hosted => "GET,HEAD,PUT,POST,DELETE",
            _ => "GET,HEAD",
        },
    }
}

fn whoami(ctx: &Ctx) -> Result<Response, ContentError> {
    match &ctx.subject {
        Some(subject) if !subject.anonymous => Ok(reply::json(
            StatusCode::OK,
            &json!({"username": subject.user_id}),
        )),
        _ => Err(ContentError::Unauthorized),
    }
}

#[async_trait]
impl Format for Npm {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        Ok(self
            .answer(&ctx, request)
            .await
            .unwrap_or_else(reply::error))
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match path::route(path) {
            Route::Tarball { name, file } => {
                let path = name.tarball_path(&file);
                let version = name.version_of_file(&file);
                astrum_content::fetch(self, ctx, &path, Freshness::Content, || {
                    documents::tarball_put(&name, PutAsset::cached(path.clone()), version.clone())
                })
                .await
            }
            Route::Packument { name } => {
                let path = name.packument_path();
                astrum_content::fetch(self, ctx, &path, Freshness::Metadata, || {
                    documents::packument_put(&name, path.clone())
                })
                .await
            }
            _ => Ok(None),
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}
