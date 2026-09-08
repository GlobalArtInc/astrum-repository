pub mod archive;
pub mod dcf;
mod hosted;
pub mod index;
pub mod paths;
pub mod rds;
pub mod task;

use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

use astrum_blob::Written;
use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, UploadField, UploadForm,
    UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use bytes::Bytes;
use futures_util::TryStreamExt;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::archive::Packing;
use crate::index::{File, Index, Kind};
use crate::paths::{PackageFile, Route, route};

pub const ID: &str = "r";
pub const PACKAGE: &str = "package";
pub const METADATA: &str = "metadata";
const MAX_INDEX: u64 = 512 * 1024 * 1024;
const MAX_MERGED: usize = 16;

struct Merged {
    signature: String,
    plain: Bytes,
    gz: Bytes,
    rds: Bytes,
    etags: [String; 3],
}

impl Merged {
    fn build(signature: String, index: &Index) -> Merged {
        let files = index.files();
        let etag = |bytes: &[u8]| hex::encode(&Sha256::digest(bytes)[..16]);
        Merged {
            signature,
            etags: [etag(&files.plain), etag(&files.gz), etag(&files.rds)],
            plain: Bytes::from(files.plain),
            gz: Bytes::from(files.gz),
            rds: Bytes::from(files.rds),
        }
    }

    fn file(&self, file: File) -> (Bytes, &str) {
        match file {
            File::Plain => (self.plain.clone(), &self.etags[0]),
            File::Gz => (self.gz.clone(), &self.etags[1]),
            File::Rds => (self.rds.clone(), &self.etags[2]),
        }
    }
}

#[derive(Default)]
pub struct R {
    merged: Mutex<HashMap<(String, String), Arc<Merged>>>,
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is an R package repository. Install from it with <code>install.packages(\"&lt;package&gt;\", repos = \"{url}/\")</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn bytes_response(
    request: &ContentRequest,
    body: Bytes,
    content_type: &str,
    etag: &str,
) -> Result<Response, ContentError> {
    let etag = format!("\"{etag}\"");
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    if unchanged {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}

fn empty_index(request: &ContentRequest, kind: Kind, file: File) -> Result<Response, ContentError> {
    let files = Index {
        kind,
        entries: Vec::new(),
    }
    .files();
    let body = files.get(file).to_vec();
    let etag = hex::encode(&Sha256::digest(&body)[..16]);
    bytes_response(request, Bytes::from(body), file.content_type(), &etag)
}

async fn read_asset(ctx: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > MAX_INDEX {
        return Err(ContentError::BadGateway(format!(
            "{} is too large",
            found.path
        )));
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn gunzip(bytes: &[u8]) -> Result<Vec<u8>, ContentError> {
    let mut out = Vec::new();
    flate2::read::MultiGzDecoder::new(bytes)
        .take(MAX_INDEX)
        .read_to_end(&mut out)
        .map_err(|error| {
            ContentError::BadGateway(format!("PACKAGES.gz cannot be read: {error}"))
        })?;
    Ok(out)
}

fn proxied_content_type(path: &str, route: &Route) -> Option<&'static str> {
    match route {
        Route::Index { file, .. } => Some(file.content_type()),
        Route::Package(file) => Some(paths::content_type(file.extension)),
        _ => paths::EXTENSIONS
            .iter()
            .find(|extension| path.ends_with(**extension))
            .map(|extension| paths::content_type(extension)),
    }
}

impl R {
    async fn proxied(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let route = route(path);
        let freshness = match &route {
            Route::Package(_) => Freshness::Content,
            _ if paths::is_archive(path) => Freshness::Content,
            _ => Freshness::Metadata,
        };
        proxy::fetch(ctx, path, None, freshness, || {
            let mut put = PutAsset::cached(path.to_owned());
            if let Some(content_type) = proxied_content_type(path, &route) {
                put = put.content_type(content_type);
            }
            match &route {
                Route::Package(file) => put.kind(PACKAGE).component(ComponentKey {
                    namespace: String::new(),
                    name: file.name.clone(),
                    version: file.version.clone(),
                    kind: String::new(),
                    attributes: json!({}),
                }),
                _ => put.kind(METADATA),
            }
        })
        .await
    }

    async fn member_index(
        &self,
        member: &Ctx,
        directory: &str,
    ) -> Result<Option<(Asset, bool)>, ContentError> {
        match member.kind() {
            RepositoryType::Hosted => {
                hosted::ensure_fresh(member, directory).await?;
                Ok(store::find(
                    member.db(),
                    member.name(),
                    &hosted::index_path(directory, File::Plain),
                )
                .await?
                .map(|found| (found, false)))
            }
            RepositoryType::Proxy => {
                let packed = hosted::index_path(directory, File::Gz);
                if let Some(found) = self.proxied(member, &packed).await? {
                    return Ok(Some((found, true)));
                }
                let plain = hosted::index_path(directory, File::Plain);
                Ok(self
                    .proxied(member, &plain)
                    .await?
                    .map(|found| (found, false)))
            }
            RepositoryType::Group => Ok(None),
        }
    }

    async fn merged(
        &self,
        ctx: &Ctx,
        directory: &str,
    ) -> Result<Option<Arc<Merged>>, ContentError> {
        let mut sources = Vec::new();
        for member in group::members(ctx).await? {
            let member = ctx.member(member);
            match self.member_index(&member, directory).await {
                Ok(Some((found, packed))) => sources.push((member, found, packed)),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                }
            }
        }
        if sources.is_empty() {
            return Ok(None);
        }
        let signature = sources
            .iter()
            .map(|(member, found, _)| format!("{}:{}:{}", member.name(), found.path, found.sha1))
            .collect::<Vec<_>>()
            .join("|");
        let key = (ctx.name().to_owned(), directory.to_owned());
        if let Some(cached) = self
            .merged
            .lock()
            .expect("the merged index lock is never poisoned")
            .get(&key)
            .filter(|cached| cached.signature == signature)
        {
            return Ok(Some(cached.clone()));
        }
        let mut texts = Vec::new();
        let mut complete = true;
        for (member, found, packed) in &sources {
            let text = match read_asset(member, found).await {
                Ok(bytes) if *packed => gunzip(&bytes),
                other => other,
            };
            match text {
                Ok(text) => texts.push(text),
                Err(error) => {
                    complete = false;
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "the index of a group member cannot be read");
                }
            }
        }
        let kind = Kind::of_directory(directory);
        let merged = tokio::task::spawn_blocking(move || {
            let lists = texts
                .iter()
                .map(|text| dcf::parse(&dcf::decode(text)))
                .collect();
            Merged::build(signature, &index::merge(kind, lists))
        })
        .await
        .map_err(ContentError::internal)?;
        let merged = Arc::new(merged);
        if complete {
            let mut cache = self
                .merged
                .lock()
                .expect("the merged index lock is never poisoned");
            if cache.len() >= MAX_MERGED && !cache.contains_key(&key) {
                cache.clear();
            }
            cache.insert(key, merged.clone());
        }
        Ok(Some(merged))
    }

    async fn first(&self, ctx: &Ctx, path: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        group::first(ctx, |member| async move { self.get(&member, path).await }).await
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = route(path);
        if route == Route::Directory {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        let options = match &route {
            Route::Package(_) => Serve::download(),
            _ => Serve::default(),
        };
        if let Route::Index { directory, file } = &route {
            match ctx.kind() {
                RepositoryType::Hosted => {
                    hosted::ensure_fresh(ctx, directory).await?;
                    if store::find(ctx.db(), ctx.name(), path).await?.is_none()
                        && let Some(kind) = paths::standard_kind(directory)
                    {
                        return empty_index(request, kind, *file);
                    }
                }
                RepositoryType::Group => {
                    return match self.merged(ctx, directory).await? {
                        Some(merged) => {
                            let (body, etag) = merged.file(*file);
                            bytes_response(request, body, file.content_type(), etag)
                        }
                        None => match paths::standard_kind(directory) {
                            Some(kind) => empty_index(request, kind, *file),
                            None => Err(ContentError::not_found(path.to_owned())),
                        },
                    };
                }
                RepositoryType::Proxy => {}
            }
        }
        let (owner, found) = match ctx.kind() {
            RepositoryType::Hosted => (ctx.clone(), store::find(ctx.db(), ctx.name(), path).await?),
            RepositoryType::Proxy => (ctx.clone(), self.proxied(ctx, path).await?),
            RepositoryType::Group => match self.first(ctx, path).await? {
                Some((member, found)) => (member, Some(found)),
                None => (ctx.clone(), None),
            },
        };
        let found = found.ok_or_else(|| ContentError::not_found(path.to_owned()))?;
        serve(&owner, &found, &request.method, &request.headers, options).await
    }

    async fn store(
        &self,
        ctx: &Ctx,
        written: Written,
        target: impl FnOnce(&archive::Description, Option<Packing>) -> Result<PackageFile, String>,
    ) -> Result<Asset, ContentError> {
        let described = hosted::describe(ctx, &written).await;
        let (packing, description) = match described {
            Ok(found) => found,
            Err(error) => {
                store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                return Err(error);
            }
        };
        let file = match target(&description, packing) {
            Ok(file) => file,
            Err(message) => {
                store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                return Err(ContentError::BadRequest(message));
            }
        };
        hosted::adopt(ctx, written, file, packing, description).await
    }

    async fn put(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.clone();
        let file = match route(&path) {
            Route::Package(file) if paths::hosted_kind(&file.directory).is_some() => file,
            Route::Index { .. } => {
                return Err(ContentError::BadRequest(
                    "the PACKAGES files of a hosted R repository are generated; upload packages instead"
                        .to_owned(),
                ));
            }
            _ => return Err(ContentError::BadRequest(paths::PLACES.to_owned())),
        };
        if !ctx.allowed(View::Add, &path) && !ctx.allowed(View::Edit, &path) {
            ctx.require(View::Add, &path)?;
        }
        let written = store::write_blob(ctx, request.into_stream()).await?;
        self.store(ctx, written, |_, _| Ok(file)).await?;
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
        let hosted = ctx.kind() == RepositoryType::Hosted;
        if hosted && found.kind == METADATA {
            return Err(ContentError::BadRequest(
                "the PACKAGES files of a hosted R repository are generated; delete packages instead"
                    .to_owned(),
            ));
        }
        store::delete_asset(&ctx.content, found).await?;
        if hosted && let Some((directory, _)) = path.rsplit_once('/') {
            hosted::rebuild(ctx, directory, false).await?;
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    pub async fn rebuild(&self, ctx: &Ctx) -> Result<usize, ContentError> {
        hosted::rebuild_all(ctx).await
    }
}

fn upload_target(
    path_id: Option<&str>,
    uploaded: Option<&str>,
    description: &archive::Description,
    packing: Option<Packing>,
) -> Result<PackageFile, String> {
    let record = &description.record;
    let name = record.get("Package").unwrap_or_default().to_owned();
    let version = record.get("Version").unwrap_or_default().to_owned();
    if !dcf::valid_package(&name) || !dcf::valid_version(&version) {
        return Err(format!(
            "the DESCRIPTION file names the package {name} {version}, which is not a valid package name and version"
        ));
    }
    let given = path_id
        .map(|path| format!("/{}", path.trim().trim_matches('/')))
        .filter(|path| path != "/");
    if let Some(given) = &given
        && let Some(file) = paths::package_file(given)
    {
        return Ok(file);
    }
    let directory = match given {
        Some(directory) => directory,
        None => match (packing, record.get("Built")) {
            (Some(Packing::Zip), Some(built)) => {
                paths::windows_directory(built).ok_or_else(|| {
                    format!("the Built field {built} names no R version; give the package path")
                })?
            }
            (Some(Packing::Zip), None) => {
                return Err("give the package path of this zip archive".to_owned());
            }
            (_, Some(_)) => {
                return Err(
                    "this is a binary package; give its package path, for example bin/macosx/big-sur-arm64/contrib/4.4"
                        .to_owned(),
                );
            }
            (_, None) => "/src/contrib".to_owned(),
        },
    };
    let kind = paths::hosted_kind(&directory).ok_or_else(|| paths::PLACES.to_owned())?;
    let named = uploaded
        .and_then(|name| paths::package_file(&format!("/{name}")))
        .filter(|file| kind.extensions().contains(&file.extension));
    let extension = match named {
        Some(file) => file.extension,
        None => match (kind, packing) {
            (Kind::Windows, _) => ".zip",
            (Kind::Mac, _) => ".tgz",
            (Kind::Source, Some(Packing::Bzip2)) => ".tar.bz2",
            (Kind::Source, Some(Packing::Xz)) => ".tar.xz",
            (Kind::Source, _) => ".tar.gz",
        },
    };
    Ok(PackageFile {
        directory,
        name,
        version,
        extension,
    })
}

#[async_trait]
impl Format for R {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::PUT, RepositoryType::Hosted) => self.put(&ctx, request).await,
            (&Method::DELETE, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (method, kind) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: match kind {
                    RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
                    RepositoryType::Proxy => "GET,HEAD,DELETE",
                    RepositoryType::Group => "GET,HEAD",
                },
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                if let Route::Index { directory, .. } = route(path) {
                    hosted::ensure_fresh(ctx, &directory).await?;
                }
                store::find(ctx.db(), ctx.name(), path).await
            }
            RepositoryType::Proxy => self.proxied(ctx, path).await,
            RepositoryType::Group => Ok(Box::pin(self.first(ctx, path))
                .await?
                .map(|(_, found)| found)),
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![],
            asset_fields: vec![
                UploadField::text("pathId", true).describe(
                    "Package path, such as src/contrib or bin/windows/contrib/4.4; source packages go to src/contrib when it is empty",
                ),
                UploadField::file("asset"),
            ],
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
            .get("r.asset")
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest("r.asset is required".to_owned()))?;
        let path_id = form
            .field("r.asset.pathId")
            .or_else(|| form.field("r.pathId"))
            .map(str::to_owned);
        if !(ctx.grant(View::Add).any() || ctx.grant(View::Edit).any()) {
            ctx.require(View::Add, "/src/contrib/")?;
        }
        let uploaded = file
            .filename
            .as_deref()
            .map(|name| name.rsplit(['/', '\\']).next().unwrap_or(name).to_owned());
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        self.store(&ctx, written, |description, packing| {
            upload_target(
                path_id.as_deref(),
                uploaded.as_deref(),
                description,
                packing,
            )
        })
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dcf::Record;

    fn description(fields: &[(&str, &str)]) -> archive::Description {
        archive::Description {
            folder: fields[0].1.to_owned(),
            record: Record {
                fields: fields
                    .iter()
                    .map(|(tag, value)| ((*tag).to_owned(), (*value).to_owned()))
                    .collect(),
                raw: String::new(),
            },
        }
    }

    #[test]
    fn uploads_find_their_place_from_the_path_or_the_package() {
        let source = description(&[("Package", "hello1"), ("Version", "1.0")]);
        let file = upload_target(
            None,
            Some("hello1_1.0.tar.gz"),
            &source,
            Some(Packing::Gzip),
        )
        .unwrap();
        assert_eq!(file.path(), "/src/contrib/hello1_1.0.tar.gz");
        let file = upload_target(
            Some("src/contrib/Archive/hello1/"),
            Some("upload.bin"),
            &source,
            Some(Packing::Xz),
        )
        .unwrap();
        assert_eq!(file.path(), "/src/contrib/Archive/hello1/hello1_1.0.tar.xz");
        let file = upload_target(
            Some("/src/contrib/hello1_1.0.tar.gz"),
            None,
            &source,
            Some(Packing::Gzip),
        )
        .unwrap();
        assert_eq!(file.path(), "/src/contrib/hello1_1.0.tar.gz");
        let windows = description(&[
            ("Package", "hello1"),
            ("Version", "1.0"),
            ("Built", "R 4.4.1; ; 2026-10-01 10:00:00 UTC; windows"),
        ]);
        let file = upload_target(None, None, &windows, Some(Packing::Zip)).unwrap();
        assert_eq!(file.path(), "/bin/windows/contrib/4.4/hello1_1.0.zip");
        let mac = description(&[
            ("Package", "hello1"),
            ("Version", "1.0"),
            ("Built", "R 4.4.1; aarch64-apple-darwin20; 2026-10-01; unix"),
        ]);
        assert!(upload_target(None, None, &mac, Some(Packing::Gzip)).is_err());
        let file = upload_target(
            Some("bin/macosx/big-sur-arm64/contrib/4.4"),
            None,
            &mac,
            Some(Packing::Gzip),
        )
        .unwrap();
        assert_eq!(
            file.path(),
            "/bin/macosx/big-sur-arm64/contrib/4.4/hello1_1.0.tgz"
        );
        assert!(upload_target(Some("elsewhere"), None, &source, Some(Packing::Gzip)).is_err());
        let invalid = description(&[("Package", "x"), ("Version", "1.0")]);
        assert!(upload_target(None, None, &invalid, Some(Packing::Gzip)).is_err());
    }
}
