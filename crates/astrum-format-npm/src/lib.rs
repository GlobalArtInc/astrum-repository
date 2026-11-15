mod attachment;
mod audit;
mod documents;
mod hosted;
pub mod import;
mod login;
mod packument;
mod path;
mod reply;
mod search;
mod transfer;
mod upload;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::inventory::{Inventory, LicenseFact, PackageRef};
use astrum_content::store::{self, Component, PutAsset};
use astrum_content::transfer::Transfer;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, UploadForm,
    UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use serde_json::{Value, json};

use crate::documents::{not_found, read_json};
use crate::path::{Name, Route};

pub use login::routes;

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
            Route::Audit => audit::audit(ctx, request).await,
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
            &json!({"username": subject.public_identity().id}),
        )),
        _ => Err(ContentError::Unauthorized),
    }
}

fn package_name(namespace: &str, name: &str) -> Option<Name> {
    let full = match namespace {
        "" => name.to_owned(),
        scope => format!("@{scope}/{name}"),
    };
    Name::parse(&full)
}

pub(crate) fn package_inventory(namespace: &str, name: &str, version: &str) -> Option<Inventory> {
    if name.is_empty() || version.is_empty() {
        return None;
    }
    let package = package_name(namespace, name)?;
    let purl = match &package.scope {
        Some(scope) => format!("pkg:npm/%40{scope}/{}@{version}", package.bare),
        None => format!("pkg:npm/{}@{version}", package.bare),
    };
    Some(Inventory {
        key: purl.clone(),
        packages: vec![PackageRef {
            ecosystem: "npm".to_owned(),
            name: package.to_string(),
            version: version.to_owned(),
            purl,
        }],
        license: None,
        complete: true,
        ..Inventory::default()
    })
}

async fn stored_license(
    ctx: &Ctx,
    component: &Component,
) -> Result<Option<LicenseFact>, ContentError> {
    let Some(name) = package_name(&component.namespace, &component.name) else {
        return Ok(None);
    };
    let Some(asset) = store::find(ctx.db(), ctx.name(), &name.packument_path()).await? else {
        return Ok(None);
    };
    let doc = read_json(ctx, &asset).await?;
    Ok(
        packument::license(&doc, &component.version).map(|declared| LicenseFact {
            declared,
            source: "npm.packument".to_owned(),
        }),
    )
}

#[async_trait]
impl Format for Npm {
    fn id(&self) -> &'static str {
        ID
    }

    async fn inventory(
        &self,
        ctx: &Ctx,
        component: &Component,
    ) -> Result<Option<Inventory>, ContentError> {
        let Some(inventory) =
            package_inventory(&component.namespace, &component.name, &component.version)
        else {
            return Ok(None);
        };
        let license = stored_license(ctx, component).await?;
        Ok(Some(Inventory {
            license,
            ..inventory
        }))
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
                let published = match &version {
                    Some(version) if ctx.kind() == RepositoryType::Proxy => {
                        documents::cached_date(ctx, &name, &path, version).await?
                    }
                    _ => None,
                };
                astrum_content::fetch(self, ctx, &path, Freshness::Content, || {
                    let put = documents::tarball_put(
                        &name,
                        PutAsset::cached(path.clone()),
                        version.clone(),
                    );
                    match published {
                        Some(at) => put.published(at),
                        None => put,
                    }
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

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(upload::spec())
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        upload::upload(self, &ctx, form).await
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some("CASE WHEN a.kind = 'tarball' THEN replace(a.path, '/-/', '/') END"),
            generated: &[],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn transfer(&self) -> Option<&dyn Transfer> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosted_and_proxy_packages_name_themselves_by_name_and_version() {
        let hosted = package_inventory("", "left-pad", "1.3.0").unwrap();
        assert_eq!(hosted.key, "pkg:npm/left-pad@1.3.0");
        assert_eq!(hosted.packages[0].name, "left-pad");
        let proxy = package_inventory("", "express", "4.19.2").unwrap();
        assert_eq!(proxy.key, "pkg:npm/express@4.19.2");
        assert!(package_inventory("", "left-pad", "").is_none());
    }

    #[test]
    fn scoped_packages_keep_the_scope_in_name_and_purl() {
        let found = package_inventory("babel", "core", "7.24.0").unwrap();
        assert_eq!(found.key, "pkg:npm/%40babel/core@7.24.0");
        assert_eq!(found.packages[0].name, "@babel/core");
    }

    #[test]
    fn scoped_packages_read_their_packument_from_the_scoped_path() {
        let package = package_name("babel", "core").unwrap();
        assert_eq!(package.packument_path(), "/@babel/core");
        assert_eq!(
            package_name("", "left-pad").unwrap().packument_path(),
            "/left-pad"
        );
    }
}
