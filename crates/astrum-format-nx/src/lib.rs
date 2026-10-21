use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub const ID: &str = "nx";
const OCTET_STREAM: &str = "application/octet-stream";
const ALLOW: &str = "GET,HEAD,PUT,DELETE";
const PREFIX: &str = "/v1/cache/";
const MAX_HASH: usize = 128;

pub struct Nx;

fn valid_hash(hash: &str) -> bool {
    !hash.is_empty()
        && hash.len() <= MAX_HASH
        && hash
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

fn cached_hash(path: &str) -> Option<&str> {
    path.strip_prefix(PREFIX).filter(|hash| valid_hash(hash))
}

fn stored_hash(path: &str) -> Option<&str> {
    path.strip_prefix('/').filter(|hash| valid_hash(hash))
}

fn stored_path(hash: &str) -> String {
    format!("/{hash}")
}

fn component(hash: &str) -> ComponentKey {
    ComponentKey {
        namespace: "/".to_owned(),
        name: hash.to_owned(),
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

impl Nx {
    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        hash: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(hash);
        ctx.require(View::Read, &path)?;
        let asset = store::find(ctx.db(), ctx.name(), &path)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("no cached task output {hash}")))?;
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
        hash: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(hash);
        ctx.require(View::Add, &path)?;
        if store::find(ctx.db(), ctx.name(), &path).await?.is_some() {
            return Ok(plain(
                StatusCode::CONFLICT,
                "Cannot override an existing record",
            ));
        }
        let mut put = PutAsset::hosted(ctx, path)
            .component(component(hash))
            .content_type(OCTET_STREAM);
        put.check_content_type = false;
        store::put(ctx, put, request.into_stream()).await?;
        Ok(plain(StatusCode::OK, "Uploaded"))
    }

    async fn delete(&self, ctx: &Ctx, hash: &str) -> Result<Response, ContentError> {
        let path = stored_path(hash);
        ctx.require(View::Delete, &path)?;
        match store::delete(ctx, &path).await? {
            true => Ok(StatusCode::NO_CONTENT.into_response()),
            false => Err(ContentError::not_found(format!(
                "no cached task output {hash}"
            ))),
        }
    }
}

#[async_trait]
impl Format for Nx {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "Nx caches are hosted repositories".to_owned(),
            ));
        }
        let path = request.path.clone();
        if let Some(hash) = cached_hash(&path) {
            let hash = hash.to_owned();
            return match request.method {
                Method::GET | Method::HEAD => self.download(&ctx, &request, &hash).await,
                Method::PUT => self.upload(&ctx, request, &hash).await,
                Method::DELETE => self.delete(&ctx, &hash).await,
                method => Err(ContentError::MethodNotAllowed {
                    method: method.to_string(),
                    allow: ALLOW,
                }),
            };
        }
        match (stored_hash(&path), &request.method) {
            (Some(hash), &Method::GET | &Method::HEAD) => {
                let hash = hash.to_owned();
                self.download(&ctx, &request, &hash).await
            }
            _ => Err(ContentError::not_found(
                "an Nx cache answers under /v1/cache",
            )),
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
    fn the_cache_lives_under_v1_cache() {
        assert_eq!(
            cached_hash("/v1/cache/12345678901234567890"),
            Some("12345678901234567890")
        );
        assert_eq!(cached_hash("/v1/cache/abc-DEF_1"), Some("abc-DEF_1"));
        assert_eq!(cached_hash("/v1/cache/"), None);
        assert_eq!(cached_hash("/v1/cache/a/b"), None);
        assert_eq!(cached_hash("/v1/cache/../x"), None);
        assert_eq!(cached_hash("/v2/cache/1"), None);
    }

    #[test]
    fn stored_outputs_sit_at_the_root_by_hash() {
        assert_eq!(stored_path("123"), "/123");
        assert_eq!(stored_hash("/123"), Some("123"));
        assert_eq!(stored_hash("/v1/cache/123"), None);
        let key = component("123");
        assert_eq!((key.namespace.as_str(), key.name.as_str()), ("/", "123"));
    }

    #[test]
    fn a_hash_is_bounded() {
        assert!(valid_hash(&"9".repeat(MAX_HASH)));
        assert!(!valid_hash(&"9".repeat(MAX_HASH + 1)));
        assert!(!valid_hash("a.b"));
    }
}
