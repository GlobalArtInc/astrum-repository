use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::TryStreamExt;

pub const ID: &str = "gradlecache";
const OCTET_STREAM: &str = "application/octet-stream";
const ALLOW: &str = "GET,HEAD,PUT,DELETE";
const MIN_KEY: usize = 8;
const MAX_KEY: usize = 128;

pub struct GradleCache;

fn valid_key(key: &str) -> bool {
    (MIN_KEY..=MAX_KEY).contains(&key.len()) && key.chars().all(|c| c.is_ascii_hexdigit())
}

fn cache_key(path: &str) -> Option<String> {
    path.strip_prefix('/')
        .filter(|key| valid_key(key))
        .map(str::to_ascii_lowercase)
}

fn stored_path(key: &str) -> String {
    format!("/{key}")
}

fn component(key: &str) -> ComponentKey {
    ComponentKey {
        namespace: "/".to_owned(),
        name: key.to_owned(),
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

impl GradleCache {
    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        key: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(key);
        ctx.require(View::Read, &path)?;
        let asset = store::find(ctx.db(), ctx.name(), &path)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("no cache entry {key}")))?;
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
        key: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(key);
        ctx.require(View::Add, &path)?;
        if store::find(ctx.db(), ctx.name(), &path).await?.is_some() {
            drain(request).await?;
            return Ok(plain(StatusCode::OK, "Already stored"));
        }
        let mut put = PutAsset::hosted(ctx, path)
            .component(component(key))
            .content_type(OCTET_STREAM);
        put.check_content_type = false;
        let written = store::write_blob(ctx, request.into_stream()).await?;
        match store::adopt(ctx, put, &written).await {
            Ok(_) => Ok(plain(StatusCode::OK, "Uploaded")),
            Err(ContentError::Conflict(_)) => Ok(plain(StatusCode::OK, "Already stored")),
            Err(error) => Err(error),
        }
    }

    async fn delete(&self, ctx: &Ctx, key: &str) -> Result<Response, ContentError> {
        let path = stored_path(key);
        ctx.require(View::Delete, &path)?;
        match store::delete(ctx, &path).await? {
            true => Ok(StatusCode::NO_CONTENT.into_response()),
            false => Err(ContentError::not_found(format!("no cache entry {key}"))),
        }
    }
}

#[async_trait]
impl Format for GradleCache {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "Gradle build caches are hosted repositories".to_owned(),
            ));
        }
        let Some(key) = cache_key(&request.path) else {
            return match request.method {
                Method::PUT => Err(ContentError::BadRequest(
                    "a Gradle cache key is 8 to 128 hexadecimal characters".to_owned(),
                )),
                _ => Err(ContentError::not_found(
                    "a Gradle build cache answers under a hexadecimal key",
                )),
            };
        };
        match request.method {
            Method::GET | Method::HEAD => self.download(&ctx, &request, &key).await,
            Method::PUT => self.upload(&ctx, request, &key).await,
            Method::DELETE => self.delete(&ctx, &key).await,
            method => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: ALLOW,
            }),
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
    fn a_key_is_hexadecimal_between_eight_and_128_characters() {
        assert!(valid_key("0123abcdEF01"));
        assert!(valid_key(&"f".repeat(MAX_KEY)));
        assert!(!valid_key("0123abc"));
        assert!(!valid_key(&"f".repeat(MAX_KEY + 1)));
        assert!(!valid_key("0123abcg"));
        assert!(!valid_key("../x"));
    }

    #[test]
    fn keys_are_stored_lower_case_at_the_root() {
        assert_eq!(cache_key("/0123ABCDef01"), Some("0123abcdef01".to_owned()));
        assert_eq!(cache_key("/0123abcd/x"), None);
        assert_eq!(cache_key("/"), None);
        assert_eq!(stored_path("0123abcdef01"), "/0123abcdef01");
        let key = component("0123abcdef01");
        assert_eq!(
            (key.namespace.as_str(), key.name.as_str()),
            ("/", "0123abcdef01")
        );
    }
}
