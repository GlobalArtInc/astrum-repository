mod auth;
mod error;
pub mod gc;
pub mod import;
mod manifest;
mod reference;
mod routes;
mod storage;
mod upload;
mod upstream;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use astrum_content::format::SearchTarget;
use astrum_content::store::{self, Asset};
use astrum_content::{
    BrowseLayout, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, View, group, serve,
};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

pub use auth::REALM;
pub use routes::{Connector, FIRST_SUGGESTED_PORT, routes};

use crate::error::RegistryError;
use crate::manifest::Manifest;
use crate::reference::{Digest, Reference, Route};
use crate::upstream::Upstreams;

pub const ID: &str = "docker";
pub const MANIFEST_LIMIT: usize = 4 * 1024 * 1024;
const CONTENT_DIGEST: &str = "docker-content-digest";
const UPLOAD_UUID: &str = "docker-upload-uuid";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "docker.imageName",
        target: SearchTarget::Asset,
        pointer: &["docker", "name"],
    },
    SearchKey {
        param: "docker.imageTag",
        target: SearchTarget::Asset,
        pointer: &["docker", "tag"],
    },
    SearchKey {
        param: "docker.contentDigest",
        target: SearchTarget::Asset,
        pointer: &["docker", "digest"],
    },
    SearchKey {
        param: "docker.mediaType",
        target: SearchTarget::Asset,
        pointer: &["docker", "mediaType"],
    },
    SearchKey {
        param: "docker.artifactType",
        target: SearchTarget::Asset,
        pointer: &["docker", "artifactType"],
    },
    SearchKey {
        param: "docker.artifactKind",
        target: SearchTarget::Asset,
        pointer: &["docker", "artifactKind"],
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mount {
    Repository(String),
    Path(String),
    Root,
}

impl Mount {
    fn prefix(&self, image: &str) -> String {
        match self {
            Mount::Repository(repository) => format!("/repository/{repository}/v2/{image}"),
            Mount::Path(repository) => format!("/v2/{repository}/{image}"),
            Mount::Root => format!("/v2/{image}"),
        }
    }

    fn catalog(&self) -> String {
        match self {
            Mount::Repository(repository) => format!("/repository/{repository}/v2/_catalog"),
            Mount::Path(_) | Mount::Root => "/v2/_catalog".to_owned(),
        }
    }

    fn name(&self, image: &str) -> String {
        match self {
            Mount::Repository(_) | Mount::Root => image.to_owned(),
            Mount::Path(repository) => format!("{repository}/{image}"),
        }
    }
}

#[derive(Default)]
pub struct Docker {
    upstreams: Upstreams,
}

enum BlobHit {
    Stored(Box<Asset>),
    Remote(u64),
}

struct Call {
    ctx: Ctx,
    mount: Mount,
    method: Method,
    headers: HeaderMap,
    query: HashMap<String, Vec<String>>,
    body: Arc<Mutex<Option<Body>>>,
}

impl Call {
    fn param(&self, name: &str) -> Option<&str> {
        self.query
            .get(name)
            .and_then(|values| values.first())
            .map(String::as_str)
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|value| value.to_str().ok())
    }

    fn take_body(&self) -> Body {
        self.body
            .lock()
            .expect("the body lock is never poisoned")
            .take()
            .unwrap_or_else(Body::empty)
    }

    fn stream(&self) -> astrum_blob::ContentStream {
        Box::pin(futures_util::TryStreamExt::map_err(
            self.take_body().into_data_stream(),
            std::io::Error::other,
        ))
    }
}

fn parse_query(raw: Option<&str>) -> HashMap<String, Vec<String>> {
    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(raw) = raw {
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            parsed
                .entry(key.into_owned())
                .or_default()
                .push(value.into_owned());
        }
    }
    parsed
}

pub fn admit(mut ctx: Ctx) -> Ctx {
    let anonymous = ctx
        .subject
        .as_ref()
        .is_some_and(|subject| subject.anonymous);
    if anonymous && ctx.repository.forces_basic_auth() {
        ctx.subject = None;
    }
    ctx
}

fn scope(route: &Route, mount: &Mount, method: &Method) -> Option<String> {
    let actions = match *method {
        Method::GET | Method::HEAD => "pull",
        _ => "pull,push",
    };
    let image = match route {
        Route::Base => return None,
        Route::Catalog => return Some("registry:catalog:*".to_owned()),
        Route::Tags(image)
        | Route::Manifest(image, _)
        | Route::Blob(image, _)
        | Route::Uploads(image)
        | Route::Upload(image, _)
        | Route::Referrers(image, _) => image,
    };
    Some(format!("repository:{}:{actions}", mount.name(image)))
}

fn created(location: String, digest: &Digest) -> Response {
    let mut response = StatusCode::CREATED.into_response();
    let headers = response.headers_mut();
    insert(headers, header::LOCATION.as_str(), &location);
    insert(headers, CONTENT_DIGEST, &digest.to_string());
    insert(headers, header::CONTENT_LENGTH.as_str(), "0");
    response
}

fn insert(headers: &mut HeaderMap, name: &str, value: &str) {
    if let (Ok(name), Ok(value)) = (
        axum::http::HeaderName::from_bytes(name.as_bytes()),
        HeaderValue::from_str(value),
    ) {
        headers.insert(name, value);
    }
}

fn json_response(status: StatusCode, content_type: &str, body: &Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, content_type.to_owned())],
        body.to_string(),
    )
        .into_response()
}

type Page = (Vec<String>, Option<(usize, String)>);

fn paginate(
    mut items: Vec<String>,
    n: Option<&str>,
    last: Option<&str>,
) -> Result<Page, RegistryError> {
    items.sort();
    items.dedup();
    if let Some(last) = last {
        items.retain(|item| item.as_str() > last);
    }
    let Some(n) = n else {
        return Ok((items, None));
    };
    let n: usize = n.parse().map_err(|_| {
        RegistryError::new(
            StatusCode::BAD_REQUEST,
            "PAGINATION_NUMBER_INVALID",
            "n must be a number",
        )
    })?;
    if n == 0 {
        return Ok((Vec::new(), None));
    }
    let more = items.len() > n;
    items.truncate(n);
    let next = match (more, items.last()) {
        (true, Some(last)) => Some((n, last.clone())),
        _ => None,
    };
    Ok((items, next))
}

fn link(path: &str, next: Option<(usize, String)>) -> Option<String> {
    next.map(|(n, last)| {
        let encoded: String = url::form_urlencoded::byte_serialize(last.as_bytes()).collect();
        format!("<{path}?n={n}&last={encoded}>; rel=\"next\"")
    })
}

fn wants_range_backwards(headers: &HeaderMap) -> bool {
    headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().strip_prefix("bytes="))
        .and_then(|spec| spec.split_once('-'))
        .and_then(|(start, end)| {
            Some(start.trim().parse::<u64>().ok()? > end.trim().parse::<u64>().ok()?)
        })
        .unwrap_or(false)
}

pub async fn expire_uploads(content: &astrum_content::Content) -> Result<u64, ContentError> {
    upload::sweep(content).await
}

impl Docker {
    pub async fn serve(&self, ctx: Ctx, request: ContentRequest, mount: Mount) -> Response {
        let ctx = admit(ctx);
        let base = ctx.base_url.clone();
        let security = ctx.security.clone();
        let route = reference::route(&request.path);
        let scope = route
            .as_ref()
            .and_then(|route| scope(route, &mount, &request.method));
        let body = request.take_body();
        let call = Call {
            ctx,
            mount,
            method: request.method.clone(),
            query: parse_query(request.query.as_deref()),
            headers: request.headers.clone(),
            body: Arc::new(Mutex::new(Some(body))),
        };
        let leftover = call.body.clone();
        let result = match route {
            Some(route) => self.dispatch(call, route).await,
            None => Err(RegistryError::new(
                StatusCode::NOT_FOUND,
                "UNSUPPORTED",
                "this is not a registry endpoint",
            )),
        };
        if result.is_err() {
            let unread = leftover
                .lock()
                .expect("the body lock is never poisoned")
                .take();
            if let Some(unread) = unread {
                upload::drain(Box::pin(futures_util::TryStreamExt::map_err(
                    unread.into_data_stream(),
                    std::io::Error::other,
                )))
                .await;
            }
        }
        let response = result.unwrap_or_else(IntoResponse::into_response);
        auth::with_challenge(response, &security, &base, scope.as_deref())
    }

    async fn dispatch(&self, call: Call, route: Route) -> Result<Response, RegistryError> {
        let image = match &route {
            Route::Tags(image)
            | Route::Manifest(image, _)
            | Route::Blob(image, _)
            | Route::Uploads(image)
            | Route::Upload(image, _)
            | Route::Referrers(image, _) => Some(image.clone()),
            Route::Base | Route::Catalog => None,
        };
        if let Some(image) = &image
            && !reference::valid_name(image)
        {
            return Err(RegistryError::name_invalid(image));
        }
        let method = call.method.clone();
        match (route, method) {
            (Route::Base, Method::GET | Method::HEAD) => match &call.ctx.subject {
                Some(_) => Ok(json_response(
                    StatusCode::OK,
                    "application/json",
                    &json!({}),
                )),
                None => Err(RegistryError::unauthorized()),
            },
            (Route::Catalog, Method::GET) => self.catalog(call).await,
            (Route::Tags(image), Method::GET) => self.tags(call, &image).await,
            (Route::Manifest(image, reference), Method::GET | Method::HEAD) => {
                self.get_manifest(call, &image, &reference).await
            }
            (Route::Manifest(image, reference), Method::PUT) => {
                self.put_manifest(call, &image, &reference).await
            }
            (Route::Manifest(image, reference), Method::DELETE) => {
                self.delete_manifest(call, &image, &reference).await
            }
            (Route::Blob(image, digest), Method::GET | Method::HEAD) => {
                self.get_blob(call, &image, &digest).await
            }
            (Route::Blob(image, digest), Method::DELETE) => {
                self.delete_blob(call, &image, &digest).await
            }
            (Route::Uploads(image), Method::POST) => self.start_upload(call, &image).await,
            (Route::Upload(image, id), Method::PATCH) => self.patch_upload(call, &image, &id).await,
            (Route::Upload(image, id), Method::PUT) => self.finish_upload(call, &image, &id).await,
            (Route::Upload(image, id), Method::GET) => self.upload_status(call, &image, &id).await,
            (Route::Upload(image, id), Method::DELETE) => {
                self.cancel_upload(call, &image, &id).await
            }
            (Route::Referrers(image, digest), Method::GET) => {
                self.referrers(call, &image, &digest).await
            }
            (_, method) => Err(RegistryError::unsupported(format!(
                "{method} is not supported here"
            ))),
        }
    }

    fn writable_policy(target: &Ctx) -> Option<WritePolicy> {
        target
            .repository
            .attributes
            .storage
            .as_ref()
            .and_then(|storage| storage.write_policy)
    }

    async fn writable(&self, ctx: &Ctx, path: &str) -> Result<Ctx, RegistryError> {
        if !ctx.allowed(View::Add, path) && !ctx.allowed(View::Edit, path) {
            ctx.require(View::Add, path)?;
        }
        let target = match ctx.kind() {
            RepositoryType::Hosted => ctx.clone(),
            RepositoryType::Group => match group::writable_member(ctx).await? {
                Some(member) => ctx.member(member),
                None => {
                    return Err(RegistryError::unsupported(format!(
                        "the group {} has no writable member",
                        ctx.name()
                    )));
                }
            },
            RepositoryType::Proxy => {
                return Err(RegistryError::unsupported(format!(
                    "the proxy {} does not take pushes",
                    ctx.name()
                )));
            }
        };
        if Self::writable_policy(&target) == Some(WritePolicy::Deny) {
            return Err(RegistryError::refused(format!(
                "the repository {} is read-only",
                target.name()
            )));
        }
        Ok(target)
    }

    async fn leaf_manifest(
        &self,
        ctx: &Ctx,
        image: &str,
        reference: &Reference,
    ) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Proxy => self.upstreams.manifest(ctx, image, reference).await,
            _ => storage::find(ctx, &storage::manifest_path(image, &reference.as_str())).await,
        }
    }

    async fn find_manifest(
        &self,
        ctx: &Ctx,
        image: &str,
        reference: &Reference,
    ) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                self.leaf_manifest(&member, image, reference).await
            })
            .await?
            .map(|(_, asset)| asset)),
            _ => self.leaf_manifest(ctx, image, reference).await,
        }
    }

    async fn leaf_blob(
        &self,
        ctx: &Ctx,
        image: &str,
        digest: &Digest,
        head: bool,
    ) -> Result<Option<BlobHit>, ContentError> {
        if let Some(found) = storage::find(ctx, &storage::blob_path(digest)).await? {
            return Ok(Some(BlobHit::Stored(Box::new(found))));
        }
        if ctx.kind() != RepositoryType::Proxy {
            return Ok(None);
        }
        if head {
            return Ok(self
                .upstreams
                .blob_head(ctx, image, digest)
                .await?
                .map(BlobHit::Remote));
        }
        Ok(self
            .upstreams
            .blob(ctx, image, digest)
            .await?
            .map(|found| BlobHit::Stored(Box::new(found))))
    }

    async fn find_blob(
        &self,
        ctx: &Ctx,
        image: &str,
        digest: &Digest,
        head: bool,
    ) -> Result<Option<BlobHit>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                self.leaf_blob(&member, image, digest, head).await
            })
            .await?
            .map(|(_, hit)| hit)),
            _ => self.leaf_blob(ctx, image, digest, head).await,
        }
    }

    async fn get_manifest(
        &self,
        call: Call,
        image: &str,
        raw: &str,
    ) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        ctx.require(View::Read, &storage::manifest_path(image, raw))?;
        let name = call.mount.name(image);
        let reference =
            Reference::parse(raw).ok_or_else(|| RegistryError::manifest_unknown(&name, raw))?;
        let asset = self
            .find_manifest(ctx, image, &reference)
            .await?
            .ok_or_else(|| RegistryError::manifest_unknown(&name, raw))?;
        let digest = match &reference {
            Reference::Digest(digest) => digest.to_string(),
            Reference::Tag(_) => storage::digest_of(&asset)
                .map(|digest| digest.to_string())
                .unwrap_or_else(|| format!("sha256:{}", asset.sha256)),
        };
        let etag = format!("\"{digest}\"");
        let mut response = Response::builder()
            .header(header::CONTENT_TYPE, storage::media_type_of(&asset))
            .header(CONTENT_DIGEST, &digest)
            .header(header::ETAG, &etag);
        let unchanged = call
            .header(header::IF_NONE_MATCH.as_str())
            .is_some_and(|tags| tags.split(',').any(|tag| tag.trim() == etag));
        if unchanged {
            return response
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty())
                .map_err(|error| ContentError::internal(error).into());
        }
        response = response
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, asset.size.max(0));
        if call.method == Method::HEAD {
            return response
                .body(Body::empty())
                .map_err(|error| ContentError::internal(error).into());
        }
        let stream = storage::stream_of(ctx, &asset).await?;
        ctx.content.note_download(asset.id);
        response
            .body(Body::from_stream(stream))
            .map_err(|error| ContentError::internal(error).into())
    }

    async fn put_manifest(
        &self,
        call: Call,
        image: &str,
        raw: &str,
    ) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        let path = storage::manifest_path(image, raw);
        let target = self.writable(ctx, &path).await?;
        let reference = Reference::parse(raw).ok_or_else(|| match raw.contains(':') {
            true => RegistryError::digest_invalid(format!("{raw} is not a supported digest")),
            false => RegistryError::manifest_invalid(format!("{raw} is not a valid tag")),
        })?;
        let exists = storage::find(&target, &path).await?.is_some();
        ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
        let declared = call
            .header(header::CONTENT_LENGTH.as_str())
            .and_then(|value| value.parse::<usize>().ok());
        if declared.is_some_and(|length| length > MANIFEST_LIMIT) {
            return Err(RegistryError::too_large(MANIFEST_LIMIT));
        }
        let content_type = call
            .header(header::CONTENT_TYPE.as_str())
            .map(str::to_owned);
        let tag_params: Vec<String> = call.query.get("tag").cloned().unwrap_or_default();
        let bytes = axum::body::to_bytes(call.take_body(), MANIFEST_LIMIT)
            .await
            .map_err(|_| RegistryError::too_large(MANIFEST_LIMIT))?;
        let parsed = manifest::parse(&bytes, content_type.as_deref())
            .map_err(RegistryError::manifest_invalid)?;
        let digest = match &reference {
            Reference::Digest(digest) => {
                if !digest.matches_bytes(&bytes) {
                    return Err(RegistryError::digest_invalid(format!(
                        "the manifest does not match {digest}"
                    )));
                }
                digest.clone()
            }
            Reference::Tag(_) => Digest::sha256(&bytes),
        };
        let mut tags: Vec<String> = match &reference {
            Reference::Tag(tag) => vec![tag.clone()],
            Reference::Digest(_) => tag_params,
        };
        tags.dedup();
        for tag in &tags {
            if !reference::valid_tag(tag) {
                return Err(RegistryError::manifest_invalid(format!(
                    "{tag} is not a valid tag"
                )));
            }
        }
        for blob in &parsed.blobs {
            if storage::find(&target, &storage::blob_path(blob))
                .await?
                .is_some()
            {
                continue;
            }
            let elsewhere = match ctx.kind() {
                RepositoryType::Group => self.find_blob(ctx, image, blob, false).await?,
                _ => None,
            };
            let copied = match elsewhere {
                Some(BlobHit::Stored(found)) => self.copy_blob(ctx, &found, &target, blob).await?,
                _ => false,
            };
            if !copied {
                return Err(RegistryError::manifest_blob_unknown(&blob.to_string()));
            }
        }
        let mut changed = Vec::new();
        for tag in &tags {
            let tag_path = storage::manifest_path(image, tag);
            match storage::find(&target, &tag_path).await? {
                Some(existing) if storage::digest_of(&existing).as_ref() == Some(&digest) => {}
                Some(_) => {
                    if !self.may_move_tag(&target, tag) {
                        return Err(RegistryError::refused(format!(
                            "repository {} does not allow updating tag {tag} of {image}",
                            target.name()
                        )));
                    }
                    if tag_path != path {
                        ctx.require(View::Edit, &tag_path)?;
                    }
                    changed.push(tag.clone());
                }
                None => {
                    if tag_path != path {
                        ctx.require(View::Add, &tag_path)?;
                    }
                    changed.push(tag.clone());
                }
            }
        }
        self.store_manifest(&target, image, &digest, &parsed, &bytes, &changed)
            .await?;
        let mut response = created(
            format!("{}/manifests/{digest}", call.mount.prefix(image)),
            &digest,
        );
        if let Some(subject) = &parsed.subject {
            insert(response.headers_mut(), "oci-subject", &subject.to_string());
        }
        if matches!(reference, Reference::Digest(_)) && !tags.is_empty() {
            insert(response.headers_mut(), "oci-tag", &tags.join(", "));
        }
        Ok(response)
    }

    fn may_move_tag(&self, target: &Ctx, tag: &str) -> bool {
        match Self::writable_policy(target) {
            Some(WritePolicy::AllowOnce) => {
                tag == "latest"
                    && target
                        .repository
                        .attributes
                        .storage
                        .as_ref()
                        .and_then(|storage| storage.latest_policy)
                        .unwrap_or(false)
            }
            Some(WritePolicy::Deny) => false,
            _ => true,
        }
    }

    async fn store_manifest(
        &self,
        target: &Ctx,
        image: &str,
        digest: &Digest,
        parsed: &Manifest,
        bytes: &[u8],
        tags: &[String],
    ) -> Result<(), ContentError> {
        let by_digest = storage::manifest_path(image, &digest.to_string());
        if storage::find(target, &by_digest).await?.is_none() {
            let written =
                store::write_blob(target, astrum_blob::content_of(bytes.to_vec())).await?;
            let put = storage::manifest_put(image, &digest.to_string(), digest, parsed, None);
            store::adopt(target, put, &written).await?;
        }
        for tag in tags {
            let written =
                store::write_blob(target, astrum_blob::content_of(bytes.to_vec())).await?;
            let put = storage::manifest_put(image, tag, digest, parsed, Some(tag));
            store::adopt(target, put, &written).await?;
        }
        Ok(())
    }

    async fn delete_manifest(
        &self,
        call: Call,
        image: &str,
        raw: &str,
    ) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        if ctx.kind() == RepositoryType::Group {
            return Err(RegistryError::unsupported(
                "a group does not delete content; delete it in its member",
            ));
        }
        ctx.require(View::Delete, &storage::manifest_path(image, raw))?;
        let name = call.mount.name(image);
        let deleted = match Reference::parse(raw) {
            Some(Reference::Tag(tag)) => {
                match storage::find(ctx, &storage::manifest_path(image, &tag)).await? {
                    Some(found) => {
                        store::delete_asset(&ctx.content, found).await?;
                        true
                    }
                    None => false,
                }
            }
            Some(Reference::Digest(digest)) => self.delete_cascade(ctx, image, &digest).await?,
            None => false,
        };
        match deleted {
            true => Ok(StatusCode::ACCEPTED.into_response()),
            false => Err(RegistryError::manifest_unknown(&name, raw)),
        }
    }

    async fn delete_cascade(
        &self,
        ctx: &Ctx,
        image: &str,
        digest: &Digest,
    ) -> Result<bool, ContentError> {
        let mut queue = vec![digest.clone()];
        let mut seen = HashSet::new();
        let mut deleted = false;
        while let Some(current) = queue.pop() {
            if !seen.insert(current.clone()) {
                continue;
            }
            let own =
                storage::find(ctx, &storage::manifest_path(image, &current.to_string())).await?;
            let first = current == *digest;
            if first && own.is_none() {
                return Ok(false);
            }
            for referrer in storage::referrers(ctx, image, &current).await? {
                if let Some(next) = storage::digest_of(&referrer) {
                    queue.push(next);
                }
            }
            let mut doomed = storage::tags_pointing_at(ctx, image, &current).await?;
            let schema = current.referrers_tag();
            for suffix in ["", ".sig", ".att", ".sbom"] {
                let path = storage::manifest_path(image, &format!("{schema}{suffix}"));
                if let Some(found) = storage::find(ctx, &path).await?
                    && !doomed.iter().any(|known| known.id == found.id)
                {
                    if let Some(target) = storage::digest_of(&found) {
                        queue.push(target);
                    }
                    doomed.push(found);
                }
            }
            if let Some(own) = own {
                doomed.push(own);
            }
            for asset in doomed {
                store::delete_asset(&ctx.content, asset).await?;
                deleted = true;
            }
        }
        Ok(deleted)
    }

    async fn get_blob(
        &self,
        call: Call,
        image: &str,
        raw: &str,
    ) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        ctx.require(View::Read, &format!("/v2/{image}/blobs/{raw}"))?;
        let digest = Digest::parse(raw).ok_or_else(|| RegistryError::blob_unknown(raw))?;
        let head = call.method == Method::HEAD;
        let hit = self
            .find_blob(ctx, image, &digest, head)
            .await?
            .ok_or_else(|| RegistryError::blob_unknown(raw))?;
        let asset = match hit {
            BlobHit::Stored(asset) => *asset,
            BlobHit::Remote(size) => {
                let mut response = StatusCode::OK.into_response();
                let headers = response.headers_mut();
                insert(headers, header::CONTENT_LENGTH.as_str(), &size.to_string());
                insert(
                    headers,
                    header::CONTENT_TYPE.as_str(),
                    storage::OCTET_STREAM,
                );
                insert(headers, CONTENT_DIGEST, &digest.to_string());
                return Ok(response);
            }
        };
        if wants_range_backwards(&call.headers) {
            return Err(ContentError::RangeNotSatisfiable(asset.size.max(0) as u64).into());
        }
        let mut response = serve(
            ctx,
            &asset,
            &call.method,
            &call.headers,
            Serve {
                content_type: Some(storage::OCTET_STREAM.to_owned()),
                ..Serve::download()
            },
        )
        .await?;
        let headers = response.headers_mut();
        insert(headers, header::ETAG.as_str(), &format!("\"{digest}\""));
        insert(headers, CONTENT_DIGEST, &digest.to_string());
        Ok(response)
    }

    async fn delete_blob(
        &self,
        call: Call,
        image: &str,
        raw: &str,
    ) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        if ctx.kind() == RepositoryType::Group {
            return Err(RegistryError::unsupported(
                "a group does not delete content; delete it in its member",
            ));
        }
        ctx.require(View::Delete, &format!("/v2/{image}/blobs/{raw}"))?;
        let digest = Digest::parse(raw).ok_or_else(|| RegistryError::blob_unknown(raw))?;
        match storage::find(ctx, &storage::blob_path(&digest)).await? {
            Some(found) => {
                store::delete_asset(&ctx.content, found).await?;
                Ok(StatusCode::ACCEPTED.into_response())
            }
            None => Err(RegistryError::blob_unknown(raw)),
        }
    }

    async fn start_upload(&self, call: Call, image: &str) -> Result<Response, RegistryError> {
        let ctx = call.ctx.clone();
        let uploads = format!("/v2/{image}/blobs/uploads/");
        let target = self.writable(&ctx, &uploads).await?;
        ctx.require(View::Add, &uploads)?;
        let prefix = call.mount.prefix(image);
        if let Some(digest) = call.param("mount").and_then(Digest::parse)
            && self
                .mount_blob(&ctx, &target, &call.mount, &digest, call.param("from"))
                .await?
        {
            return Ok(created(format!("{prefix}/blobs/{digest}"), &digest));
        }
        if call.param("mount").is_none()
            && let Some(raw) = call.param("digest")
        {
            let digest = Digest::parse(raw)
                .ok_or_else(|| RegistryError::digest_invalid(format!("{raw} is not supported")))?;
            let body = call.stream();
            let written = store::write_blob(&target, body).await?;
            if !digest.matches(&written.digests) {
                storage::discard(&target, &written);
                return Err(RegistryError::digest_invalid(format!(
                    "the content does not match {digest}"
                )));
            }
            storage::adopt_blob(&target, &digest, written).await?;
            return Ok(created(format!("{prefix}/blobs/{digest}"), &digest));
        }
        let session = upload::create(&target, image).await?;
        let mut response = StatusCode::ACCEPTED.into_response();
        let headers = response.headers_mut();
        insert(
            headers,
            header::LOCATION.as_str(),
            &format!("{prefix}/blobs/uploads/{}", session.id),
        );
        insert(headers, UPLOAD_UUID, &session.id.to_string());
        insert(headers, header::RANGE.as_str(), "0-0");
        insert(headers, header::CONTENT_LENGTH.as_str(), "0");
        Ok(response)
    }

    async fn mount_blob(
        &self,
        ctx: &Ctx,
        target: &Ctx,
        mount: &Mount,
        digest: &Digest,
        from: Option<&str>,
    ) -> Result<bool, RegistryError> {
        if storage::find(target, &storage::blob_path(digest))
            .await?
            .is_some()
        {
            return Ok(true);
        }
        let candidates: Vec<(String, Option<String>)> = match (from, mount) {
            (Some(from), Mount::Path(_)) => match from.split_once('/') {
                Some((repository, image)) => vec![(repository.to_owned(), Some(image.to_owned()))],
                None => Vec::new(),
            },
            (Some(_), Mount::Repository(_) | Mount::Root) => Vec::new(),
            (None, _) => storage::repositories_holding(target, digest)
                .await?
                .into_iter()
                .map(|repository| (repository, None))
                .collect(),
        };
        for (name, source_image) in candidates {
            if name == target.name() {
                continue;
            }
            let Some(repository) = ctx.content.repository(&name).await? else {
                continue;
            };
            if repository.format.id != ID || !repository.online {
                continue;
            }
            let source = admit(ctx.content.context(
                repository,
                ctx.security.clone(),
                ctx.subject.clone(),
                ctx.base_url.clone(),
                ctx.client_ip.clone(),
            ));
            let image = source_image.as_deref().unwrap_or("-");
            if !source.allowed(View::Read, &format!("/v2/{image}/blobs/{digest}")) {
                continue;
            }
            let Some(BlobHit::Stored(found)) =
                self.find_blob(&source, image, digest, false).await?
            else {
                continue;
            };
            if self.copy_blob(&source, &found, target, digest).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn copy_blob(
        &self,
        source: &Ctx,
        found: &Asset,
        target: &Ctx,
        digest: &Digest,
    ) -> Result<bool, ContentError> {
        let stream = storage::stream_of(source, found).await?;
        let written = store::write_blob(target, stream).await?;
        if !digest.matches(&written.digests) {
            storage::discard(target, &written);
            return Ok(false);
        }
        storage::adopt_blob(target, digest, written).await?;
        Ok(true)
    }

    fn upload_location(call: &Call, image: &str, id: &str) -> String {
        format!("{}/blobs/uploads/{id}", call.mount.prefix(image))
    }

    async fn upload_target(&self, call: &Call, image: &str) -> Result<Ctx, RegistryError> {
        let uploads = format!("/v2/{image}/blobs/uploads/");
        let target = self.writable(&call.ctx, &uploads).await?;
        call.ctx.require(View::Add, &uploads)?;
        Ok(target)
    }

    fn claimed_range(call: &Call) -> Result<Option<(u64, u64)>, RegistryError> {
        match call.header(header::CONTENT_RANGE.as_str()) {
            None => Ok(None),
            Some(value) => upload::parse_range(value).map(Some).ok_or_else(|| {
                RegistryError::new(
                    StatusCode::RANGE_NOT_SATISFIABLE,
                    "BLOB_UPLOAD_INVALID",
                    format!("{value} is not a valid chunk range"),
                )
            }),
        }
    }

    async fn patch_upload(
        &self,
        call: Call,
        image: &str,
        id: &str,
    ) -> Result<Response, RegistryError> {
        let target = self.upload_target(&call, image).await?;
        let location = Self::upload_location(&call, image, id);
        let with_location = |error: RegistryError| {
            error
                .header(header::LOCATION, location.clone())
                .header(axum::http::HeaderName::from_static(UPLOAD_UUID), id)
        };
        let session = upload::load(&target, id, image)
            .await
            .map_err(with_location)?;
        let claimed = Self::claimed_range(&call).map_err(with_location)?;
        let body = call.stream();
        let updated = upload::append(&target, &session, claimed, body)
            .await
            .map_err(with_location)?;
        let mut response = StatusCode::ACCEPTED.into_response();
        let headers = response.headers_mut();
        insert(headers, header::LOCATION.as_str(), &location);
        insert(
            headers,
            header::RANGE.as_str(),
            &upload::range(updated.size),
        );
        insert(headers, UPLOAD_UUID, id);
        insert(headers, header::CONTENT_LENGTH.as_str(), "0");
        Ok(response)
    }

    async fn finish_upload(
        &self,
        call: Call,
        image: &str,
        id: &str,
    ) -> Result<Response, RegistryError> {
        let target = self.upload_target(&call, image).await?;
        let location = Self::upload_location(&call, image, id);
        let with_location = |error: RegistryError| {
            error
                .header(header::LOCATION, location.clone())
                .header(axum::http::HeaderName::from_static(UPLOAD_UUID), id)
        };
        let session = upload::load(&target, id, image)
            .await
            .map_err(with_location)?;
        let raw = call
            .param("digest")
            .ok_or_else(|| RegistryError::digest_invalid("the digest parameter is missing"))?
            .to_owned();
        let digest = Digest::parse(&raw)
            .ok_or_else(|| RegistryError::digest_invalid(format!("{raw} is not supported")))?;
        let claimed = Self::claimed_range(&call).map_err(with_location)?;
        let empty = call
            .header(header::CONTENT_LENGTH.as_str())
            .is_some_and(|length| length.trim() == "0");
        let session = match empty {
            true => session,
            false => {
                let body = call.stream();
                upload::append(&target, &session, claimed, body)
                    .await
                    .map_err(with_location)?
            }
        };
        let session = upload::claim(&target, &session).await?;
        let written = upload::assemble(&target, &session).await?;
        if !digest.matches(&written.digests) {
            storage::discard(&target, &written);
            return Err(RegistryError::digest_invalid(format!(
                "the upload does not match {digest}"
            )));
        }
        storage::adopt_blob(&target, &digest, written).await?;
        Ok(created(
            format!("{}/blobs/{digest}", call.mount.prefix(image)),
            &digest,
        ))
    }

    async fn upload_status(
        &self,
        call: Call,
        image: &str,
        id: &str,
    ) -> Result<Response, RegistryError> {
        let target = self.upload_target(&call, image).await?;
        let session = upload::load(&target, id, image).await?;
        let mut response = StatusCode::NO_CONTENT.into_response();
        let headers = response.headers_mut();
        insert(
            headers,
            header::LOCATION.as_str(),
            &Self::upload_location(&call, image, id),
        );
        insert(
            headers,
            header::RANGE.as_str(),
            &upload::range(session.size),
        );
        insert(headers, UPLOAD_UUID, id);
        Ok(response)
    }

    async fn cancel_upload(
        &self,
        call: Call,
        image: &str,
        id: &str,
    ) -> Result<Response, RegistryError> {
        let target = self.upload_target(&call, image).await?;
        let session = upload::load(&target, id, image).await?;
        let session = upload::claim(&target, &session).await?;
        upload::discard_chunks(&target, &session, &upload::chunks(&session));
        Ok(StatusCode::NO_CONTENT.into_response())
    }

    async fn leaf_tags(&self, ctx: &Ctx, image: &str) -> Result<Option<Vec<String>>, ContentError> {
        let cached = || async {
            let found = storage::tags(ctx, image).await?;
            let names: Vec<String> = found
                .iter()
                .map(|asset| storage::tag_name(asset, image))
                .collect();
            Ok::<_, ContentError>(
                match names.is_empty() && !storage::has_image(ctx, image).await? {
                    true => None,
                    false => Some(names),
                },
            )
        };
        if ctx.kind() != RepositoryType::Proxy {
            return cached().await;
        }
        match self.upstreams.tags(ctx, image, None).await {
            Ok(Some(tags)) => Ok(Some(tags)),
            Ok(None) => cached().await,
            Err(error) => {
                tracing::info!(repository = ctx.name(), image, %error, "listing cached tags");
                cached().await
            }
        }
    }

    async fn tags(&self, call: Call, image: &str) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        ctx.require(View::Read, &format!("/v2/{image}/tags/list"))?;
        let found = match ctx.kind() {
            RepositoryType::Group => {
                let mut union: Option<Vec<String>> = None;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match self.leaf_tags(&member, image).await {
                        Ok(Some(tags)) => union.get_or_insert_with(Vec::new).extend(tags),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed");
                        }
                    }
                }
                union
            }
            _ => self.leaf_tags(ctx, image).await?,
        };
        let name = call.mount.name(image);
        let tags = found.ok_or_else(|| RegistryError::name_unknown(&name))?;
        let (tags, next) = paginate(tags, call.param("n"), call.param("last"))?;
        let mut response = json_response(
            StatusCode::OK,
            "application/json",
            &json!({"name": name, "tags": tags}),
        );
        if let Some(link) = link(&format!("{}/tags/list", call.mount.prefix(image)), next) {
            insert(response.headers_mut(), header::LINK.as_str(), &link);
        }
        Ok(response)
    }

    async fn stored_referrers(
        &self,
        ctx: &Ctx,
        image: &str,
        subject: &Digest,
    ) -> Result<Vec<Value>, ContentError> {
        let mut entries = Vec::new();
        for asset in storage::referrers(ctx, image, subject).await? {
            let Some(digest) = storage::digest_of(&asset) else {
                continue;
            };
            let docker = asset.attributes.get("docker").cloned().unwrap_or_default();
            let mut descriptor = json!({
                "mediaType": storage::media_type_of(&asset),
                "digest": digest.to_string(),
                "size": asset.size,
            });
            if let Some(artifact_type) = docker.get("artifactType").and_then(Value::as_str) {
                descriptor["artifactType"] = artifact_type.into();
            }
            let annotations = match docker.get("largeAnnotations").and_then(Value::as_bool) {
                Some(true) => storage::read_all(ctx, &asset, MANIFEST_LIMIT)
                    .await
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .and_then(|manifest| manifest.get("annotations").cloned()),
                _ => docker.get("annotations").cloned(),
            };
            if let Some(annotations) = annotations.filter(|value| value.is_object()) {
                descriptor["annotations"] = annotations;
            }
            entries.push(descriptor);
        }
        let schema = storage::manifest_path(image, &subject.referrers_tag());
        if let Some(index) = storage::find(ctx, &schema).await?
            && let Ok(bytes) = storage::read_all(ctx, &index, MANIFEST_LIMIT).await
        {
            entries.extend(upstream::index_entries(&bytes));
        }
        Ok(entries)
    }

    async fn leaf_referrers(
        &self,
        ctx: &Ctx,
        image: &str,
        subject: &Digest,
    ) -> Result<Vec<Value>, ContentError> {
        if ctx.kind() != RepositoryType::Proxy {
            return self.stored_referrers(ctx, image, subject).await;
        }
        match self.upstreams.referrers(ctx, image, subject).await {
            Ok(entries) => Ok(entries),
            Err(error) => {
                tracing::info!(repository = ctx.name(), image, %error, "listing cached referrers");
                self.stored_referrers(ctx, image, subject).await
            }
        }
    }

    async fn referrers(
        &self,
        call: Call,
        image: &str,
        raw: &str,
    ) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        ctx.require(View::Read, &format!("/v2/{image}/referrers/{raw}"))?;
        let subject = Digest::parse(raw)
            .ok_or_else(|| RegistryError::digest_invalid(format!("{raw} is not a valid digest")))?;
        let found = match ctx.kind() {
            RepositoryType::Group => {
                let mut union = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match self.leaf_referrers(&member, image, &subject).await {
                        Ok(entries) => union.extend(entries),
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed");
                        }
                    }
                }
                union
            }
            _ => self.leaf_referrers(ctx, image, &subject).await?,
        };
        let mut seen = HashSet::new();
        let mut manifests: Vec<Value> = found
            .into_iter()
            .filter(|entry| {
                entry
                    .get("digest")
                    .and_then(Value::as_str)
                    .is_some_and(|digest| seen.insert(digest.to_owned()))
            })
            .collect();
        let filter = call.param("artifactType").filter(|value| !value.is_empty());
        if let Some(wanted) = filter {
            manifests
                .retain(|entry| entry.get("artifactType").and_then(Value::as_str) == Some(wanted));
        }
        let body = json!({
            "schemaVersion": 2,
            "mediaType": manifest::OCI_INDEX,
            "manifests": manifests,
        });
        let mut response = json_response(StatusCode::OK, manifest::OCI_INDEX, &body);
        if filter.is_some() {
            insert(
                response.headers_mut(),
                "oci-filters-applied",
                "artifactType",
            );
        }
        Ok(response)
    }

    async fn leaf_images(&self, ctx: &Ctx) -> Result<Vec<String>, ContentError> {
        storage::images(ctx).await
    }

    pub(crate) async fn images_of(&self, ctx: &Ctx) -> Result<Vec<String>, ContentError> {
        let names = match ctx.kind() {
            RepositoryType::Group => {
                let mut union = BTreeSet::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    union.extend(self.leaf_images(&member).await?);
                }
                union.into_iter().collect()
            }
            _ => self.leaf_images(ctx).await?,
        };
        Ok(names
            .into_iter()
            .filter(|image| {
                ctx.allowed(View::Browse, &format!("/v2/{image}/"))
                    || ctx.allowed(View::Read, &format!("/v2/{image}/"))
            })
            .collect())
    }

    async fn catalog(&self, call: Call) -> Result<Response, RegistryError> {
        let ctx = &call.ctx;
        if ctx.subject.is_none() {
            return Err(RegistryError::unauthorized());
        }
        let images = self.images_of(ctx).await?;
        let (repositories, next) = paginate(images, call.param("n"), call.param("last"))?;
        let mut response = json_response(
            StatusCode::OK,
            "application/json",
            &json!({ "repositories": repositories }),
        );
        if let Some(link) = link(&call.mount.catalog(), next) {
            insert(response.headers_mut(), header::LINK.as_str(), &link);
        }
        Ok(response)
    }

    async fn leaf_get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        if let Some(rest) = path.strip_prefix("/v2/")
            && let Some((image, reference)) = rest.rsplit_once("/manifests/")
            && let Some(reference) = Reference::parse(reference)
            && image != "-"
        {
            return self.leaf_manifest(ctx, image, &reference).await;
        }
        storage::find(ctx, path).await
    }
}

#[async_trait]
impl Format for Docker {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(
        &self,
        ctx: Ctx,
        mut request: ContentRequest,
    ) -> Result<Response, ContentError> {
        let Some(rest) = request
            .path
            .strip_prefix("/v2")
            .filter(|rest| rest.is_empty() || rest.starts_with('/'))
        else {
            return Err(ContentError::not_found(
                "Docker repositories answer under /v2/ only",
            ));
        };
        request.path = rest.to_owned();
        let mount = Mount::Repository(ctx.name().to_owned());
        Ok(self.serve(ctx, request, mount).await)
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => {
                Ok(group::first(
                    ctx,
                    |member| async move { self.leaf_get(&member, path).await },
                )
                .await?
                .map(|(_, asset)| asset))
            }
            _ => self.leaf_get(ctx, path).await,
        }
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN a.kind = 'tag' THEN regexp_replace(a.path, '^/v2(/.+)/manifests/([^/]+)$', '\\1/\\2') END",
            ),
            generated: &[],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_follow_last_and_n() {
        let tags: Vec<String> = ["b", "a", "d", "c", "a"]
            .iter()
            .map(|tag| (*tag).to_owned())
            .collect();
        let (all, next) = paginate(tags.clone(), None, None).unwrap();
        assert_eq!(all, ["a", "b", "c", "d"]);
        assert_eq!(next, None);
        let (page, next) = paginate(tags.clone(), Some("2"), None).unwrap();
        assert_eq!(page, ["a", "b"]);
        assert_eq!(next, Some((2, "b".to_owned())));
        let (rest, next) = paginate(tags.clone(), Some("2"), Some("b")).unwrap();
        assert_eq!(rest, ["c", "d"]);
        assert_eq!(next, None);
        let (after, _) = paginate(tags.clone(), None, Some("b")).unwrap();
        assert_eq!(after, ["c", "d"]);
        let (none, next) = paginate(tags.clone(), Some("0"), None).unwrap();
        assert!(none.is_empty() && next.is_none());
        assert!(paginate(tags, Some("x"), None).is_err());
        assert_eq!(
            link("/v2/r/a/tags/list", Some((2, "b+c".to_owned()))).unwrap(),
            "</v2/r/a/tags/list?n=2&last=b%2Bc>; rel=\"next\""
        );
    }

    #[test]
    fn mounts_name_images_and_locations() {
        let path = Mount::Path("hosted".to_owned());
        assert_eq!(path.prefix("team/app"), "/v2/hosted/team/app");
        assert_eq!(path.name("team/app"), "hosted/team/app");
        assert_eq!(path.catalog(), "/v2/_catalog");
        let repository = Mount::Repository("hosted".to_owned());
        assert_eq!(repository.prefix("app"), "/repository/hosted/v2/app");
        assert_eq!(repository.name("app"), "app");
        assert_eq!(
            scope(&Route::Tags("app".to_owned()), &path, &Method::GET).as_deref(),
            Some("repository:hosted/app:pull")
        );
        assert_eq!(
            scope(&Route::Uploads("app".to_owned()), &path, &Method::POST).as_deref(),
            Some("repository:hosted/app:pull,push")
        );
        assert_eq!(scope(&Route::Base, &path, &Method::GET), None);
    }

    #[test]
    fn backwards_ranges_are_spotted() {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, "bytes=500-0".parse().unwrap());
        assert!(wants_range_backwards(&headers));
        headers.insert(header::RANGE, "bytes=0-500".parse().unwrap());
        assert!(!wants_range_backwards(&headers));
        headers.insert(header::RANGE, "bytes=-500".parse().unwrap());
        assert!(!wants_range_backwards(&headers));
    }
}
