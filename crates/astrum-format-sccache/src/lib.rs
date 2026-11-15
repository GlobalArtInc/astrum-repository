use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_db::entity::repository;
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderName, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use futures_util::TryStreamExt;
use sea_orm::EntityTrait;
use std::time::SystemTime;

pub const ID: &str = "sccache";
const OCTET_STREAM: &str = "application/octet-stream";
const ALLOW: &str = "GET,HEAD,PUT,DELETE,PROPFIND,MKCOL,OPTIONS";
const DAV: HeaderName = HeaderName::from_static("dav");
const MULTISTATUS: &str = "application/xml; charset=utf-8";
const MAX_SEGMENTS: usize = 16;
const MAX_PATH: usize = 512;
const PATH_RULE: &str =
    "an sccache path has up to 16 segments of letters, digits, dots, dashes and underscores";

pub struct Sccache;

enum Target {
    Entry(String),
    Directory,
}

fn valid_segment(segment: &str) -> bool {
    segment != "."
        && segment != ".."
        && !segment.is_empty()
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn segments(path: &str) -> Option<Vec<&str>> {
    if path.len() > MAX_PATH {
        return None;
    }
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    (parts.len() <= MAX_SEGMENTS && parts.iter().all(|part| valid_segment(part))).then_some(parts)
}

fn classify(path: &str) -> Option<Target> {
    let parts = segments(path)?;
    Some(match parts.is_empty() || path.ends_with('/') {
        true => Target::Directory,
        false => Target::Entry(path.to_owned()),
    })
}

fn component(path: &str) -> ComponentKey {
    ComponentKey {
        namespace: "/".to_owned(),
        name: path.rsplit('/').next().unwrap_or_default().to_owned(),
        ..Default::default()
    }
}

fn plain(status: StatusCode, text: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain;charset=utf-8")
        .body(Body::from(text.to_owned()))
        .expect("a plain answer is valid")
}

fn multistatus(body: String) -> Response {
    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header(header::CONTENT_TYPE, MULTISTATUS)
        .body(Body::from(body))
        .expect("a multistatus answer is valid")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn file_body(href: &str, asset: &Asset) -> String {
    let href = escape(href);
    let size = asset.size;
    let etag = &asset.sha1;
    let modified = httpdate::fmt_http_date(asset.blob_updated_at.with_timezone(&Utc).into());
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:"><D:response><D:href>{href}</D:href><D:propstat><D:prop><D:resourcetype/><D:getcontentlength>{size}</D:getcontentlength><D:getlastmodified>{modified}</D:getlastmodified><D:getetag>"{etag}"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response></D:multistatus>"#
    )
}

fn collection_body(href: &str, created: SystemTime) -> String {
    let href = escape(href);
    let modified = httpdate::fmt_http_date(created);
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:"><D:response><D:href>{href}</D:href><D:propstat><D:prop><D:resourcetype><D:collection/></D:resourcetype><D:getlastmodified>{modified}</D:getlastmodified></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response></D:multistatus>"#
    )
}

async fn created_at(ctx: &Ctx) -> Result<SystemTime, ContentError> {
    let row = repository::Entity::find_by_id(ctx.name().to_owned())
        .one(ctx.db())
        .await?
        .ok_or_else(|| ContentError::not_found("Repository not found"))?;
    Ok(row.created_at.with_timezone(&Utc).into())
}

async fn drain(request: ContentRequest) -> Result<(), ContentError> {
    let mut stream = request.into_stream();
    while stream
        .try_next()
        .await
        .map_err(ContentError::internal)?
        .is_some()
    {}
    Ok(())
}

async fn refused(request: ContentRequest, error: ContentError) -> ContentError {
    let _ = drain(request).await;
    error
}

impl Sccache {
    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        path: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, path)?;
        let asset = store::find(ctx.db(), ctx.name(), path)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("no cache entry {path}")))?;
        serve(
            ctx,
            &asset,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }

    async fn upload(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        path: &str,
    ) -> Result<Response, ContentError> {
        if let Err(error) = ctx.require(View::Add, path) {
            return Err(refused(request, error).await);
        }
        let mut put = PutAsset::hosted(ctx, path)
            .component(component(path))
            .content_type(OCTET_STREAM);
        put.check_content_type = false;
        if put.write_policy == Some(WritePolicy::Deny) {
            let error =
                ContentError::BadRequest(format!("the repository {} is read-only", ctx.name()));
            return Err(refused(request, error).await);
        }
        let existed = store::find(ctx.db(), ctx.name(), path).await?.is_some();
        let written = store::write_blob(ctx, request.into_stream()).await?;
        match store::adopt(ctx, put, &written).await {
            Ok(_) if existed => Ok(StatusCode::NO_CONTENT.into_response()),
            Ok(_) => Ok(StatusCode::CREATED.into_response()),
            Err(ContentError::Conflict(_)) => Ok(StatusCode::NO_CONTENT.into_response()),
            Err(error) => Err(error),
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        match store::delete(ctx, path).await? {
            true => Ok(StatusCode::NO_CONTENT.into_response()),
            false => Err(ContentError::not_found(format!("no cache entry {path}"))),
        }
    }

    async fn mkcol(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if let Err(error) = ctx.require(View::Add, &request.path) {
            return Err(refused(request, error).await);
        }
        Ok(StatusCode::CREATED.into_response())
    }

    async fn options(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        ctx.require(View::Read, &request.path)?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(DAV, "1")
            .header(header::ALLOW, ALLOW)
            .body(Body::empty())
            .expect("an options answer is valid"))
    }

    async fn propfind(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        target: &Target,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &request.path)?;
        if request.header("depth") != Some("0") {
            return Ok(plain(
                StatusCode::FORBIDDEN,
                "listing is not offered: a PROPFIND needs Depth: 0",
            ));
        }
        let href = format!("/repository/{}{}", ctx.name(), request.path);
        let entry = match target {
            Target::Entry(path) => store::find(ctx.db(), ctx.name(), path).await?,
            Target::Directory => None,
        };
        let body = match entry {
            Some(asset) => file_body(&href, &asset),
            None => collection_body(&href, created_at(ctx).await?),
        };
        Ok(multistatus(body))
    }
}

#[async_trait]
impl Format for Sccache {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "sccache caches are hosted repositories".to_owned(),
            ));
        }
        let method = request.method.clone();
        if !ALLOW.split(',').any(|allowed| allowed == method.as_str()) {
            let error = ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: ALLOW,
            };
            return Err(refused(request, error).await);
        }
        let Some(target) = classify(&request.path) else {
            let error = match method.as_str() {
                "GET" | "HEAD" => ContentError::not_found(PATH_RULE),
                _ => ContentError::BadRequest(PATH_RULE.to_owned()),
            };
            return Err(refused(request, error).await);
        };
        match (method.as_str(), target) {
            ("GET" | "HEAD", Target::Entry(path)) => self.download(&ctx, &request, &path).await,
            ("PUT", Target::Entry(path)) => self.upload(&ctx, request, &path).await,
            ("DELETE", Target::Entry(path)) => self.delete(&ctx, &path).await,
            ("PROPFIND", target) => self.propfind(&ctx, &request, &target).await,
            ("MKCOL", _) => self.mkcol(&ctx, request).await,
            ("OPTIONS", _) => self.options(&ctx, &request).await,
            ("GET" | "HEAD" | "DELETE", Target::Directory) => Err(refused(
                request,
                ContentError::not_found("a directory is not a cache entry"),
            )
            .await),
            _ => Err(refused(
                request,
                ContentError::BadRequest("a cache entry is stored under a file path".to_owned()),
            )
            .await),
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
    fn a_path_has_at_most_sixteen_segments_of_safe_characters() {
        assert!(segments(&format!("/{}", vec!["a"; 16].join("/"))).is_some());
        assert!(segments(&format!("/{}", vec!["a"; 17].join("/"))).is_none());
        assert!(segments("/a/../b").is_none());
        assert!(segments("/a b").is_none());
    }
}
