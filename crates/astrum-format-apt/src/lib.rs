pub mod deb;
mod hosted;
pub mod metadata;
mod signing;
pub mod task;
pub mod testing;

use std::sync::Arc;

use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, UploadField,
    UploadForm, UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use futures_util::TryStreamExt;
use serde_json::json;

pub const ID: &str = "apt";
pub const DEB: &str = "deb";
pub const METADATA: &str = "metadata";
pub const DEB_TYPE: &str = "application/x-debian-package";

#[derive(Default)]
pub struct Apt {
    keys: Arc<signing::Keys>,
}

pub fn distribution(ctx: &Ctx) -> String {
    ctx.repository
        .attributes
        .apt
        .as_ref()
        .and_then(|apt| apt.distribution.clone())
        .unwrap_or_default()
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
    let line = match ctx.kind() {
        RepositoryType::Hosted => format!("deb {url}/ {} main", escape(&distribution(ctx))),
        _ => format!("deb {url}/ &lt;distribution&gt; &lt;components&gt;"),
    };
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is an APT repository. Add <code>{line}</code> to a file in /etc/apt/sources.list.d/.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn metadata_path(path: &str) -> bool {
    if path.contains("/by-hash/") {
        return false;
    }
    let file = path.rsplit('/').next().unwrap_or_default();
    matches!(file, "Release" | "InRelease" | "Release.gpg")
        || file.starts_with("Packages")
        || file.starts_with("Sources")
        || file.starts_with("Contents-")
        || file.starts_with("Translation-")
        || path.starts_with("/dists/")
}

fn content_type(path: &str) -> Option<&'static str> {
    let file = path.rsplit('/').next().unwrap_or_default();
    if file.ends_with(".deb") || file.ends_with(".udeb") {
        Some(DEB_TYPE)
    } else if file.ends_with(".gz") {
        Some("application/x-gzip")
    } else if file.ends_with(".bz2") {
        Some("application/x-bzip2")
    } else if file.ends_with(".xz") {
        Some("application/x-xz")
    } else if file == "Release.gpg" {
        Some("application/pgp-signature")
    } else if matches!(file, "Release" | "InRelease" | "Packages") {
        Some("text/plain")
    } else {
        None
    }
}

impl Apt {
    async fn proxied(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let freshness = if metadata_path(path) {
            Freshness::Metadata
        } else {
            Freshness::Content
        };
        let file = path.rsplit('/').next().unwrap_or_default().to_owned();
        let parsed = deb::from_file_name(&file);
        proxy::fetch(ctx, path, None, freshness, || {
            let mut put = PutAsset::cached(path.to_owned()).kind(if parsed.is_some() {
                DEB
            } else {
                METADATA
            });
            if let Some(content_type) = content_type(path) {
                put = put.content_type(content_type);
            }
            match parsed {
                Some((package, version, architecture)) => put.component(ComponentKey {
                    namespace: architecture,
                    name: package,
                    version,
                    kind: String::new(),
                    attributes: json!({}),
                }),
                None => put,
            }
        })
        .await
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if path == "/" || path.ends_with('/') {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        let found = match ctx.kind() {
            RepositoryType::Hosted => {
                if path.starts_with(&format!("/dists/{}/", distribution(ctx))) {
                    hosted::ensure_fresh(ctx, &self.keys).await?;
                }
                store::find(ctx.db(), ctx.name(), path).await?
            }
            _ => self.proxied(ctx, path).await?,
        };
        let found = found.ok_or_else(|| ContentError::not_found(path.to_owned()))?;
        let options = Serve {
            count_download: found.kind == DEB,
            ..Serve::default()
        };
        serve(ctx, &found, &request.method, &request.headers, options).await
    }

    async fn upload_body(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
    ) -> Result<Response, ContentError> {
        if !(ctx.grant(View::Add).any() || ctx.grant(View::Edit).any()) {
            ctx.require(View::Add, "/pool/")?;
        }
        let boundary = request
            .header("content-type")
            .and_then(|value| multer::parse_boundary(value).ok());
        let written = match boundary {
            Some(boundary) => {
                let mut form =
                    multer::Multipart::new(request.take_body().into_data_stream(), boundary);
                let mut written = None;
                while let Some(field) = form
                    .next_field()
                    .await
                    .map_err(|error| ContentError::BadRequest(error.to_string()))?
                {
                    if field.file_name().is_none() && written.is_some() {
                        continue;
                    }
                    let stream: astrum_blob::ContentStream =
                        Box::pin(field.map_err(std::io::Error::other));
                    written = Some(store::write_blob(ctx, stream).await?);
                    break;
                }
                written.ok_or_else(|| {
                    ContentError::BadRequest("the form holds no package file".to_owned())
                })?
            }
            None => store::write_blob(ctx, request.into_stream()).await?,
        };
        hosted::adopt(ctx, &self.keys, written).await?;
        Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        if ctx.kind() == RepositoryType::Hosted && found.kind != DEB {
            return Err(ContentError::BadRequest(
                "the metadata of a hosted APT repository is generated; delete packages instead"
                    .to_owned(),
            ));
        }
        store::delete_asset(&ctx.content, found).await?;
        if ctx.kind() == RepositoryType::Hosted {
            hosted::rebuild(ctx, &self.keys, false).await?;
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    pub async fn rebuild(&self, ctx: &Ctx) -> Result<(), ContentError> {
        hosted::rebuild(ctx, &self.keys, true).await
    }
}

#[async_trait]
impl Format for Apt {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::POST, RepositoryType::Hosted) if request.path == "/" => {
                self.upload_body(&ctx, request).await
            }
            (&Method::DELETE, _) => self.delete(&ctx, &request.path).await,
            (method, kind) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: match kind {
                    RepositoryType::Hosted => "GET,HEAD,POST,DELETE",
                    _ => "GET,HEAD,DELETE",
                },
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            _ => self.proxied(ctx, path).await,
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
            .get("apt.asset")
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest("apt.asset is required".to_owned()))?;
        if !(ctx.grant(View::Add).any() || ctx.grant(View::Edit).any()) {
            ctx.require(View::Add, "/pool/")?;
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::adopt(&ctx, &self.keys, written).await?;
        Ok(())
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN '/metadata' || a.path ELSE '/packages/' || left(c.name, 1) || '/' || c.name || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_and_packages_are_told_apart() {
        assert!(metadata_path("/dists/bookworm/InRelease"));
        assert!(metadata_path(
            "/dists/bookworm/main/binary-amd64/Packages.xz"
        ));
        assert!(metadata_path("/Packages.gz"));
        assert!(!metadata_path(
            "/dists/bookworm/main/binary-amd64/by-hash/SHA256/abc"
        ));
        assert!(!metadata_path("/pool/main/b/bash/bash_5.2_amd64.deb"));
        assert_eq!(
            content_type("/pool/h/hello/hello_1_all.deb"),
            Some(DEB_TYPE)
        );
        assert_eq!(
            content_type("/dists/x/Release.gpg"),
            Some("application/pgp-signature")
        );
    }
}
