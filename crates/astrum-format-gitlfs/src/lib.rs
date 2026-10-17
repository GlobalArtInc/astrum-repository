mod batch;
mod locks;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Serve, UploadField, UploadForm, UploadSpec,
    View, mime, serve,
};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};

pub const ID: &str = "gitlfs";
pub const OBJECT: &str = "OBJECT";
pub const MEDIA_TYPE: &str = "application/vnd.git-lfs+json";
const API_PREFIX: &str = "/info/lfs";
const OBJECTS_PREFIX: &str = "/objects/";
const MAX_JSON: usize = 4 * 1024 * 1024;

#[derive(Default)]
pub struct GitLfs;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Endpoint {
    Batch,
    Verify,
    Object(String),
    Asset(String),
    Locks,
    LocksVerify,
    Unlock(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Matched {
    endpoint: Endpoint,
    base: &'static str,
}

pub fn valid_oid(oid: &str) -> bool {
    oid.len() == 64
        && oid
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn asset_path(oid: &str) -> String {
    format!("{OBJECTS_PREFIX}{}/{}/{oid}", &oid[..2], &oid[2..4])
}

fn oid_of_asset_path(path: &str) -> Option<&str> {
    let rest = path.strip_prefix(OBJECTS_PREFIX)?;
    let mut parts = rest.split('/');
    let (first, second, oid) = (parts.next()?, parts.next()?, parts.next()?);
    (parts.next().is_none()
        && first.len() == 2
        && second.len() == 2
        && valid_oid(oid)
        && oid.starts_with(&format!("{first}{second}")))
    .then_some(oid)
}

fn matched(path: &str) -> Option<Matched> {
    if let Some(oid) = oid_of_asset_path(path) {
        return Some(Matched {
            endpoint: Endpoint::Asset(oid.to_owned()),
            base: "",
        });
    }
    let (base, relative) = match path.strip_prefix(API_PREFIX) {
        Some(rest) if rest.starts_with('/') => (API_PREFIX, rest),
        _ => ("", path),
    };
    let endpoint = match relative {
        "/objects/batch" => Endpoint::Batch,
        "/objects/verify" => Endpoint::Verify,
        "/locks" => Endpoint::Locks,
        "/locks/verify" => Endpoint::LocksVerify,
        _ => match relative.strip_prefix(OBJECTS_PREFIX) {
            Some(oid) if valid_oid(oid) => Endpoint::Object(oid.to_owned()),
            _ => {
                let id = relative
                    .strip_prefix("/locks/")?
                    .strip_suffix("/unlock")
                    .filter(|id| !id.is_empty() && !id.contains('/'))?;
                Endpoint::Unlock(id.to_owned())
            }
        },
    };
    Some(Matched { endpoint, base })
}

pub(crate) fn json_response(status: StatusCode, body: &Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, MEDIA_TYPE)
        .body(Body::from(
            serde_json::to_vec(body).expect("a JSON value always serializes"),
        ))
        .expect("a JSON response is valid")
}

pub(crate) fn error(status: StatusCode, message: impl Into<String>) -> Response {
    json_response(status, &json!({ "message": message.into() }))
}

pub(crate) fn read_only(ctx: &Ctx) -> bool {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        == Some(WritePolicy::Deny)
}

pub(crate) fn read_only_error(ctx: &Ctx) -> Response {
    error(
        StatusCode::FORBIDDEN,
        format!("Repository {} is read-only", ctx.name()),
    )
}

pub(crate) fn require_some(ctx: &Ctx, view: View, path: &str) -> Result<(), ContentError> {
    match ctx.grant(view).any() {
        true => Ok(()),
        false => ctx.require(view, path),
    }
}

pub(crate) enum JsonBody {
    Missing,
    Object(serde_json::Map<String, Value>),
}

pub(crate) struct Rejected(pub StatusCode, pub String);

impl Rejected {
    pub fn response(self) -> Response {
        error(self.0, self.1)
    }
}

pub(crate) async fn read_json(request: &ContentRequest) -> Result<JsonBody, Rejected> {
    let bytes = request
        .bytes(MAX_JSON)
        .await
        .map_err(|problem| Rejected(StatusCode::BAD_REQUEST, problem.to_string()))?;
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(JsonBody::Missing);
    }
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(map)) => Ok(JsonBody::Object(map)),
        Ok(_) => Err(Rejected(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Request body must be a JSON object".to_owned(),
        )),
        Err(problem) => Err(Rejected(
            StatusCode::BAD_REQUEST,
            format!("Invalid JSON: {problem}"),
        )),
    }
}

pub(crate) async fn stored(
    ctx: &Ctx,
    oids: &[&str],
) -> Result<std::collections::HashMap<String, i64>, ContentError> {
    let paths: Vec<String> = oids.iter().map(|oid| asset_path(oid)).collect();
    if paths.is_empty() {
        return Ok(Default::default());
    }
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Path.is_in(paths))
        .all(ctx.db())
        .await?
        .into_iter()
        .filter_map(|found| oid_of_asset_path(&found.path).map(|oid| (oid.to_owned(), found.size)))
        .collect())
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Git LFS repository. Point a git repository at it with <code>git config -f .lfsconfig lfs.url {url}/info/lfs</code>.</p>\n    <p>The objects of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn api_error(ctx: &Ctx, result: Result<Response, ContentError>) -> Result<Response, ContentError> {
    match result {
        Ok(response) => Ok(response),
        Err(ContentError::Unauthorized) => Err(ContentError::Unauthorized),
        Err(ContentError::Forbidden) => Ok(error(
            StatusCode::FORBIDDEN,
            format!("Not permitted in repository {}", ctx.name()),
        )),
        Err(ContentError::Internal(problem)) => {
            tracing::error!(
                repository = ctx.name(),
                error = format!("{problem:#}"),
                "a Git LFS request failed"
            );
            Ok(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "the server failed; see its log",
            ))
        }
        Err(other) => Ok(error(other.status(), other.to_string())),
    }
}

async fn detect(ctx: &Ctx, written: &Written) -> String {
    let sample = async {
        let store = ctx.content.blob_stores().get(&written.store).await.ok()?;
        let content = store
            .read_range(written.id, 0, mime::SAMPLE as u64)
            .await
            .ok()??;
        let mut bytes = Vec::new();
        let mut stream = content.stream;
        while let Some(chunk) = stream.try_next().await.ok()? {
            bytes.extend_from_slice(&chunk);
        }
        Some(bytes)
    }
    .await
    .unwrap_or_default();
    mime::detect(&sample).unwrap_or_else(|| mime::OCTET_STREAM.to_owned())
}

pub(crate) enum Stored {
    Kept,
    Added,
    Mismatch(String),
}

pub(crate) async fn store_object(
    ctx: &Ctx,
    written: Written,
    expected: Option<&str>,
) -> Result<Stored, ContentError> {
    let oid = written.digests.sha256.clone();
    let discard = || store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    if let Some(expected) = expected
        && expected != oid
    {
        discard();
        return Ok(Stored::Mismatch(oid));
    }
    let path = asset_path(&oid);
    if let Err(problem) = ctx.require(View::Add, &path) {
        discard();
        return Err(problem);
    }
    if store::find(ctx.db(), ctx.name(), &path).await?.is_some() {
        discard();
        return Ok(Stored::Kept);
    }
    let content_type = detect(ctx, &written).await;
    let mut put = PutAsset::hosted(ctx, &path)
        .kind(OBJECT)
        .content_type(content_type)
        .component(ComponentKey {
            namespace: String::new(),
            name: oid.clone(),
            version: String::new(),
            kind: String::new(),
            attributes: json!({}),
        });
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::Allow);
    store::adopt(ctx, put, &written).await?;
    Ok(Stored::Added)
}

impl GitLfs {
    async fn upload(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        oid: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Add, &asset_path(oid))?;
        if read_only(ctx) {
            return Ok(read_only_error(ctx));
        }
        if store::find(ctx.db(), ctx.name(), &asset_path(oid))
            .await?
            .is_none()
        {
            let written = store::write_blob(ctx, request.into_stream()).await?;
            if let Stored::Mismatch(actual) = store_object(ctx, written, Some(oid)).await? {
                return Ok(error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Object content hashes to {actual}, not to {oid}"),
                ));
            }
            tracing::debug!(repository = ctx.name(), oid, "an object was stored");
        }
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    async fn verify(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        require_some(ctx, View::Read, &request.path)?;
        let body = match read_json(request).await {
            Ok(body) => body,
            Err(rejected) => return Ok(rejected.response()),
        };
        let JsonBody::Object(body) = body else {
            return Ok(error(StatusCode::BAD_REQUEST, "Missing request body"));
        };
        let oid = body.get("oid").and_then(Value::as_str).unwrap_or_default();
        let size = body.get("size").and_then(Value::as_i64);
        let (true, Some(size)) = (valid_oid(oid), size) else {
            return Ok(error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Verify needs the oid and size of an object",
            ));
        };
        let Some(found) = stored(ctx, &[oid]).await?.get(oid).copied() else {
            return Ok(error(
                StatusCode::NOT_FOUND,
                format!("Object {oid} does not exist"),
            ));
        };
        if found != size {
            return Ok(error(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Object {oid} has {found} bytes, not {size}"),
            ));
        }
        Ok(json_response(
            StatusCode::OK,
            &json!({ "oid": oid, "size": found }),
        ))
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        oid: &str,
    ) -> Result<Response, ContentError> {
        let path = asset_path(oid);
        ctx.require(View::Read, &path)?;
        let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? else {
            return Err(ContentError::not_found(format!(
                "Object {oid} does not exist"
            )));
        };
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
impl Format for GitLfs {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let reading = matches!(request.method, Method::GET | Method::HEAD);
        if request.is_directory() && reading {
            return directory_page(&ctx);
        }
        let Some(found) = matched(&request.path) else {
            return Err(ContentError::not_found(request.path.clone()));
        };
        let api = format!("{}{}", ctx.repository_url(), found.base);
        let method = request.method.clone();
        match (found.endpoint, method) {
            (Endpoint::Batch, Method::POST) => {
                let result = batch::batch(&ctx, &request, &api).await;
                api_error(&ctx, result)
            }
            (Endpoint::Verify, Method::POST) => {
                let result = self.verify(&ctx, &request).await;
                api_error(&ctx, result)
            }
            (Endpoint::Object(oid), Method::PUT) => {
                let result = self.upload(&ctx, request, &oid).await;
                api_error(&ctx, result)
            }
            (Endpoint::Object(oid) | Endpoint::Asset(oid), Method::GET | Method::HEAD) => {
                self.download(&ctx, &request, &oid).await
            }
            (Endpoint::Locks, Method::GET | Method::HEAD) => {
                let result = locks::list(&ctx, &request).await;
                api_error(&ctx, result)
            }
            (Endpoint::Locks, Method::POST) => {
                let result = locks::create(&ctx, &request).await;
                api_error(&ctx, result)
            }
            (Endpoint::LocksVerify, Method::POST) => {
                let result = locks::verify(&ctx, &request).await;
                api_error(&ctx, result)
            }
            (Endpoint::Unlock(id), Method::POST) => {
                let result = locks::unlock(&ctx, &request, &id).await;
                api_error(&ctx, result)
            }
            _ => Err(ContentError::not_found(request.path.clone())),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        store::find(ctx.db(), ctx.name(), path).await
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
        let file = form
            .files
            .get("gitlfs.asset")
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest("gitlfs.asset is required".to_owned()))?;
        if read_only(&ctx) {
            return Err(ContentError::BadRequest(format!(
                "Repository {} is read-only",
                ctx.name()
            )));
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        store_object(&ctx, written, None).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OID: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    #[test]
    fn paths_resolve_to_endpoints_at_the_root_and_under_info_lfs() {
        let at = |path: &str| matched(path).map(|found| (found.endpoint, found.base));
        assert_eq!(
            at("/info/lfs/objects/batch"),
            Some((Endpoint::Batch, API_PREFIX))
        );
        assert_eq!(at("/objects/batch"), Some((Endpoint::Batch, "")));
        assert_eq!(
            at(&format!("/info/lfs/objects/{OID}")),
            Some((Endpoint::Object(OID.to_owned()), API_PREFIX))
        );
        assert_eq!(
            at(&asset_path(OID)),
            Some((Endpoint::Asset(OID.to_owned()), ""))
        );
        assert_eq!(
            at("/info/lfs/locks/12/unlock"),
            Some((Endpoint::Unlock("12".to_owned()), API_PREFIX))
        );
        assert_eq!(at("/locks/verify"), Some((Endpoint::LocksVerify, "")));
        assert_eq!(at("/objects/ab/cd/x"), None);
        assert_eq!(at(&format!("/objects/2c/00/{OID}")), None);
        assert_eq!(at("/info/lfsx/locks"), None);
        assert_eq!(asset_path(OID), format!("/objects/2c/f2/{OID}"));
        assert!(!valid_oid(&OID.to_uppercase()));
    }
}
