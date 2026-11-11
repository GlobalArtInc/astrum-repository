use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub const ID: &str = "bazelcache";
const OCTET_STREAM: &str = "application/octet-stream";
const ALLOW: &str = "GET,HEAD,PUT,DELETE";
const HASH_LEN: usize = 64;

pub struct BazelCache;

#[derive(Clone, Copy)]
enum Namespace {
    ActionCache,
    ContentAddressable,
}

impl Namespace {
    fn parse(segment: &str) -> Option<Self> {
        match segment {
            "ac" => Some(Self::ActionCache),
            "cas" => Some(Self::ContentAddressable),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::ActionCache => "ac",
            Self::ContentAddressable => "cas",
        }
    }
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == HASH_LEN && hash.chars().all(|c| c.is_ascii_hexdigit())
}

fn cache_path(path: &str) -> Option<(Namespace, &str)> {
    let (segment, hash) = path.strip_prefix('/')?.split_once('/')?;
    Some((Namespace::parse(segment)?, hash))
}

fn stored_path(namespace: Namespace, key: &str) -> String {
    format!("/{}/{key}", namespace.name())
}

fn component(namespace: Namespace, key: &str) -> ComponentKey {
    ComponentKey {
        namespace: "/".to_owned(),
        name: format!("{}/{key}", namespace.name()),
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

async fn store_verified(
    ctx: &Ctx,
    put: PutAsset,
    key: &str,
    request: ContentRequest,
) -> Result<(), ContentError> {
    let written = store::write_blob(ctx, request.into_stream()).await?;
    if written.digests.sha256 != key {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Err(ContentError::BadRequest(
            "the body does not hash to its SHA-256 key".to_owned(),
        ));
    }
    store::adopt(ctx, put, &written).await?;
    Ok(())
}

impl BazelCache {
    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        namespace: Namespace,
        key: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(namespace, key);
        ctx.require(View::Read, &path)?;
        let asset = store::find(ctx.db(), ctx.name(), &path)
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
        namespace: Namespace,
        key: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(namespace, key);
        ctx.require(View::Add, &path)?;
        if store::find(ctx.db(), ctx.name(), &path).await?.is_some() {
            return Ok(plain(StatusCode::OK, "Already stored"));
        }
        let mut put = PutAsset::hosted(ctx, path)
            .component(component(namespace, key))
            .content_type(OCTET_STREAM);
        put.check_content_type = false;
        let stored = match namespace {
            Namespace::ActionCache => store::put(ctx, put, request.into_stream())
                .await
                .map(|_| ()),
            Namespace::ContentAddressable => store_verified(ctx, put, key, request).await,
        };
        match stored {
            Ok(()) => Ok(plain(StatusCode::OK, "Uploaded")),
            Err(ContentError::Conflict(_)) => Ok(plain(StatusCode::OK, "Already stored")),
            Err(error) => Err(error),
        }
    }

    async fn delete(
        &self,
        ctx: &Ctx,
        namespace: Namespace,
        key: &str,
    ) -> Result<Response, ContentError> {
        let path = stored_path(namespace, key);
        ctx.require(View::Delete, &path)?;
        match store::delete(ctx, &path).await? {
            true => Ok(StatusCode::NO_CONTENT.into_response()),
            false => Err(ContentError::not_found(format!("no cache entry {path}"))),
        }
    }
}

#[async_trait]
impl Format for BazelCache {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "Bazel remote caches are hosted repositories".to_owned(),
            ));
        }
        let Some((namespace, hash)) = cache_path(&request.path) else {
            return Err(ContentError::not_found(
                "a Bazel remote cache answers under /ac/ and /cas/",
            ));
        };
        if !valid_hash(hash) {
            return match request.method {
                Method::PUT => Err(ContentError::BadRequest(
                    "a Bazel cache key is a SHA-256 digest: 64 hexadecimal characters".to_owned(),
                )),
                _ => Err(ContentError::not_found(
                    "a Bazel cache key is 64 hexadecimal characters",
                )),
            };
        }
        let key = hash.to_ascii_lowercase();
        match request.method {
            Method::GET | Method::HEAD => self.download(&ctx, &request, namespace, &key).await,
            Method::PUT => self.upload(&ctx, request, namespace, &key).await,
            Method::DELETE => self.delete(&ctx, namespace, &key).await,
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
    fn a_key_is_a_sha256_hex_digest() {
        assert!(valid_hash(&"0123abcdEF".repeat(7)[..HASH_LEN]));
        assert!(!valid_hash(&"f".repeat(HASH_LEN - 1)));
        assert!(!valid_hash(&"f".repeat(HASH_LEN + 1)));
        assert!(!valid_hash(&format!("{}g", "f".repeat(HASH_LEN - 1))));
    }

    #[test]
    fn the_namespace_is_the_first_segment_of_the_path() {
        let key = "f".repeat(HASH_LEN);
        assert!(matches!(
            cache_path(&format!("/cas/{key}")),
            Some((Namespace::ContentAddressable, _))
        ));
        assert!(matches!(
            cache_path(&format!("/ac/{key}")),
            Some((Namespace::ActionCache, _))
        ));
        assert!(cache_path(&format!("/other/{key}")).is_none());
        assert!(cache_path(&format!("/{key}")).is_none());
        assert_eq!(
            stored_path(Namespace::ActionCache, &key),
            format!("/ac/{key}")
        );
    }
}
