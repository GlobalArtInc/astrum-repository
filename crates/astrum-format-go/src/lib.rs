pub mod module;

use std::collections::BTreeSet;

use astrum_blob::Written;
use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip::{self, ZipError};
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, UploadField,
    UploadForm, UploadSpec, View, group, serve,
};
use astrum_db::entity::component;
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use chrono::{DateTime, SecondsFormat, Utc};
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};

use crate::module::Request;

pub const ID: &str = "go";
pub const ZIP: &str = "zip";
pub const MOD: &str = "mod";
pub const INFO: &str = "info";
pub const LIST: &str = "list";
pub const LATEST: &str = "latest";
const ZIP_TYPE: &str = "application/zip";
const MOD_TYPE: &str = "text/plain";
const INFO_TYPE: &str = "application/json";
const MAX_MOD: u64 = 16 * 1024 * 1024;
const MAX_METADATA: u64 = 16 * 1024 * 1024;

#[derive(Default)]
pub struct Go;

fn text(status: StatusCode, content_type: &str, body: Vec<u8>, method: &Method) -> Response {
    let length = body.len();
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .body(match *method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .expect("a text response is valid")
}

fn info_document(version: &str, time: DateTime<Utc>) -> Vec<u8> {
    json!({"Version": version, "Time": time.to_rfc3339_opts(SecondsFormat::Secs, true)})
        .to_string()
        .into_bytes()
}

async fn read(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
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

fn component_key(module: &str, version: &str) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: module.to_owned(),
        version: version.to_owned(),
        kind: String::new(),
        attributes: json!({}),
    }
}

fn kind_of(request: &Request) -> (&'static str, &'static str, Freshness) {
    match request {
        Request::List(_) => (LIST, MOD_TYPE, Freshness::Metadata),
        Request::Latest(_) => (LATEST, INFO_TYPE, Freshness::Metadata),
        Request::Info(..) => (INFO, INFO_TYPE, Freshness::Content),
        Request::Mod(..) => (MOD, MOD_TYPE, Freshness::Content),
        Request::Zip(..) => (ZIP, ZIP_TYPE, Freshness::Content),
    }
}

enum Answer {
    Stored(Box<(Ctx, Asset)>),
    Built(&'static str, Vec<u8>),
}

impl Answer {
    async fn bytes(self, limit: u64) -> Result<Vec<u8>, ContentError> {
        match self {
            Answer::Stored(stored) => read(&stored.0, &stored.1, limit).await,
            Answer::Built(_, body) => Ok(body),
        }
    }
}

async fn hosted_versions(
    ctx: &Ctx,
    module: &str,
) -> Result<Vec<(String, DateTime<Utc>)>, ContentError> {
    let rows = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Name.eq(module))
        .all(ctx.db())
        .await?;
    Ok(rows
        .into_iter()
        .filter(|row| {
            ctx.allowed(
                View::Read,
                &Request::Zip(module.to_owned(), row.version.clone()).path(),
            )
        })
        .map(|row| (row.version, row.created_at.with_timezone(&Utc)))
        .collect())
}

fn list_body(mut versions: Vec<String>) -> Vec<u8> {
    versions.sort_by(|left, right| module::compare(left, right));
    versions.dedup();
    let mut body = String::new();
    for version in versions
        .iter()
        .filter(|version| !module::is_pseudo(version))
    {
        body.push_str(version);
        body.push('\n');
    }
    body.into_bytes()
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

fn zip_error(error: ZipError) -> ContentError {
    ContentError::BadRequest(error.to_string())
}

pub struct ModuleZip {
    pub module: String,
    pub version: String,
    pub go_mod: Vec<u8>,
}

pub async fn inspect(
    source: &dyn zip::Source,
    module: Option<&str>,
    version: Option<&str>,
) -> Result<ModuleZip, ContentError> {
    let entries = zip::entries(source).await.map_err(zip_error)?;
    let first = entries
        .iter()
        .find(|entry| !entry.name.is_empty())
        .ok_or_else(|| ContentError::BadRequest("the module zip is empty".to_owned()))?;
    let prefix = first
        .name
        .split_once('/')
        .map(|(prefix, _)| prefix)
        .unwrap_or(&first.name);
    let (found_module, found_version) = match first.name.split_once('@') {
        Some((module, rest)) => (
            module.to_owned(),
            rest.split('/').next().unwrap_or_default().to_owned(),
        ),
        None => (String::new(), String::new()),
    };
    let module = module
        .map(str::to_owned)
        .unwrap_or_else(|| found_module.clone());
    let version = version
        .map(str::to_owned)
        .unwrap_or_else(|| found_version.clone());
    if !module::valid_module(&module) {
        return Err(ContentError::BadRequest(format!(
            "`{module}` is not a valid module path; name it, or put the files under <module>@<version>/ in the zip"
        )));
    }
    if !module::valid_version(&version) {
        return Err(ContentError::BadRequest(format!(
            "`{version}` is not a valid module version (v<major>.<minor>.<patch>)"
        )));
    }
    let expected = format!("{module}@{version}/");
    if let Some(stray) = entries
        .iter()
        .find(|entry| !entry.name.starts_with(&expected))
    {
        return Err(ContentError::BadRequest(format!(
            "every file of the module zip must be under {expected}, {} is not (the zip starts with {prefix})",
            stray.name
        )));
    }
    let go_mod = match entries
        .iter()
        .find(|entry| entry.name == format!("{expected}go.mod"))
    {
        Some(entry) => zip::read(source, entry, MAX_MOD).await.map_err(zip_error)?,
        None => format!("module {module}\n").into_bytes(),
    };
    Ok(ModuleZip {
        module,
        version,
        go_mod,
    })
}

impl Go {
    async fn hosted_store(
        &self,
        ctx: &Ctx,
        written: Written,
        module: Option<&str>,
        version: Option<&str>,
    ) -> Result<Asset, ContentError> {
        let store = ctx.content.blob_stores().get(&written.store).await?;
        let source = zip::Blob {
            store,
            id: written.id,
            size: written.size,
        };
        let inspected = match inspect(&source, module, version).await {
            Ok(inspected) => inspected,
            Err(error) => {
                store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                return Err(error);
            }
        };
        let zip_path = Request::Zip(inspected.module.clone(), inspected.version.clone()).path();
        let mod_path = Request::Mod(inspected.module.clone(), inspected.version.clone()).path();
        let exists = store::find(ctx.db(), ctx.name(), &zip_path)
            .await?
            .is_some();
        let allowed = if exists && write_policy(ctx) == WritePolicy::Allow {
            ctx.require(View::Edit, &zip_path)
        } else {
            ctx.require(View::Add, &zip_path)
        };
        if let Err(error) = allowed {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(error);
        }
        let key = component_key(&inspected.module, &inspected.version);
        let put = PutAsset::hosted(ctx, zip_path)
            .component(key.clone())
            .kind(ZIP)
            .content_type(ZIP_TYPE);
        let stored = store::adopt(ctx, put, &written).await?;
        let mut put = PutAsset::hosted(ctx, mod_path)
            .component(key)
            .kind(MOD)
            .content_type(MOD_TYPE);
        put.check_content_type = false;
        put.write_policy = Some(WritePolicy::Allow);
        store::put(ctx, put, astrum_blob::content_of(inspected.go_mod)).await?;
        Ok(stored)
    }

    async fn answer(&self, ctx: &Ctx, request: &Request) -> Result<Option<Answer>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => self.hosted_answer(ctx, request).await,
            RepositoryType::Proxy => {
                let (kind, content_type, freshness) = kind_of(request);
                let path = request.path();
                let remote = request.remote_path();
                let key = request
                    .version()
                    .map(|version| component_key(request.module(), version));
                let found = proxy::fetch_from(
                    ctx,
                    &path,
                    freshness,
                    || async {
                        proxy::remote_url(&ctx.repository, &remote, None)
                            .map(|url| Some(proxy::Origin::new(url)))
                    },
                    || {
                        let put = PutAsset::cached(path.clone())
                            .kind(kind)
                            .content_type(content_type);
                        match key {
                            Some(key) => put.component(key),
                            None => put,
                        }
                    },
                )
                .await?;
                Ok(found.map(|found| Answer::Stored(Box::new((ctx.clone(), found)))))
            }
            RepositoryType::Group => self.group_answer(ctx, request).await,
        }
    }

    async fn hosted_answer(
        &self,
        ctx: &Ctx,
        request: &Request,
    ) -> Result<Option<Answer>, ContentError> {
        match request {
            Request::List(module) => {
                let versions = hosted_versions(ctx, module).await?;
                if versions.is_empty() {
                    return Ok(None);
                }
                Ok(Some(Answer::Built(
                    MOD_TYPE,
                    list_body(versions.into_iter().map(|(version, _)| version).collect()),
                )))
            }
            Request::Latest(module) => {
                let versions = hosted_versions(ctx, module).await?;
                let Some(latest) =
                    module::latest(versions.iter().map(|(version, _)| version.as_str()))
                else {
                    return Ok(None);
                };
                let time = versions
                    .iter()
                    .find(|(version, _)| version == latest)
                    .map(|(_, time)| *time)
                    .unwrap_or_else(Utc::now);
                Ok(Some(Answer::Built(INFO_TYPE, info_document(latest, time))))
            }
            Request::Info(module, version) => {
                let zip_path = Request::Zip(module.clone(), version.clone()).path();
                Ok(store::find(ctx.db(), ctx.name(), &zip_path)
                    .await?
                    .map(|found| {
                        Answer::Built(
                            INFO_TYPE,
                            info_document(version, found.created_at.with_timezone(&Utc)),
                        )
                    }))
            }
            Request::Mod(..) | Request::Zip(..) => {
                Ok(store::find(ctx.db(), ctx.name(), &request.path())
                    .await?
                    .map(|found| Answer::Stored(Box::new((ctx.clone(), found)))))
            }
        }
    }

    async fn group_answer(
        &self,
        ctx: &Ctx,
        request: &Request,
    ) -> Result<Option<Answer>, ContentError> {
        match request {
            Request::List(_) | Request::Latest(_) => {
                let mut versions = BTreeSet::new();
                let mut times: Vec<(String, String)> = Vec::new();
                let mut any = false;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.answer(&member, request)).await {
                        Ok(Some(found)) => {
                            any = true;
                            let body = found.bytes(MAX_METADATA).await?;
                            match request {
                                Request::List(_) => versions.extend(
                                    String::from_utf8_lossy(&body)
                                        .lines()
                                        .map(str::trim)
                                        .filter(|line| !line.is_empty())
                                        .map(str::to_owned),
                                ),
                                _ => {
                                    if let Ok(info) = serde_json::from_slice::<Value>(&body)
                                        && let Some(version) =
                                            info.get("Version").and_then(Value::as_str)
                                    {
                                        let time = info
                                            .get("Time")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned();
                                        times.push((version.to_owned(), time));
                                    }
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                if !any {
                    return Ok(None);
                }
                match request {
                    Request::List(_) => Ok(Some(Answer::Built(
                        MOD_TYPE,
                        list_body(versions.into_iter().collect()),
                    ))),
                    _ => {
                        let Some(latest) =
                            module::latest(times.iter().map(|(version, _)| version.as_str()))
                        else {
                            return Ok(None);
                        };
                        let time = times
                            .iter()
                            .find(|(version, _)| version == latest)
                            .map(|(_, time)| time.clone())
                            .unwrap_or_default();
                        Ok(Some(Answer::Built(
                            INFO_TYPE,
                            json!({"Version": latest, "Time": time})
                                .to_string()
                                .into_bytes(),
                        )))
                    }
                }
            }
            _ => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.answer(&member, request)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if path == "/" || path.ends_with('/') {
            return directory_page(ctx);
        }
        let Some(parsed) = module::parse(path) else {
            return Err(ContentError::not_found(format!("not found: {path}")));
        };
        ctx.require(View::Read, &parsed.path())?;
        let (kind, content_type, _) = kind_of(&parsed);
        match self.answer(ctx, &parsed).await? {
            Some(Answer::Built(content_type, body)) => {
                Ok(text(StatusCode::OK, content_type, body, &request.method))
            }
            Some(Answer::Stored(stored)) => {
                let (owner, found) = *stored;
                let options = Serve {
                    content_type: Some(content_type.to_owned()),
                    count_download: kind == ZIP,
                    ..Serve::default()
                };
                serve(&owner, &found, &request.method, &request.headers, options).await
            }
            None => Err(ContentError::not_found(format!(
                "not found: {}@{}",
                parsed.module(),
                parsed.version().unwrap_or("latest")
            ))),
        }
    }

    async fn put(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let Some(Request::Zip(module, version)) = module::parse(&request.path) else {
            return Err(ContentError::BadRequest(format!(
                "{} is not a place for a module: PUT <module>/@v/<version>.zip",
                request.path
            )));
        };
        let path = Request::Zip(module.clone(), version.clone()).path();
        if !ctx.allowed(View::Add, &path) && !ctx.allowed(View::Edit, &path) {
            ctx.require(View::Add, &path)?;
        }
        let written = store::write_blob(ctx, request.into_stream()).await?;
        self.hosted_store(ctx, written, Some(&module), Some(&version))
            .await?;
        Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        let Some(parsed) = module::parse(path) else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        let stored = parsed.path();
        ctx.require(View::Delete, &stored)?;
        if !store::delete(ctx, &stored).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        if let Request::Zip(module, version) = &parsed
            && ctx.kind() == RepositoryType::Hosted
        {
            store::delete(ctx, &Request::Mod(module.clone(), version.clone()).path()).await?;
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape_html(ctx.name());
    let url = escape_html(&ctx.repository_url());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Go module proxy. Use it with <code>GOPROXY={url}</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape_html(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

#[async_trait]
impl Format for Go {
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

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE regexp_replace(a.path, '/@v/[^/]*$', '') || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[],
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let Some(parsed) = module::parse(path) else {
            return match ctx.kind() {
                RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                    store::find(member.db(), member.name(), path).await
                })
                .await?
                .map(|(_, found)| found)),
                _ => store::find(ctx.db(), ctx.name(), path).await,
            };
        };
        match self.answer(ctx, &parsed).await? {
            Some(Answer::Stored(stored)) => Ok(Some(stored.1)),
            _ => Ok(None),
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![
                UploadField::text("module", true)
                    .describe("Module path, read from the zip when empty")
                    .in_group("Module"),
                UploadField::text("version", true)
                    .describe("Module version, read from the zip when empty")
                    .in_group("Module"),
            ],
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
            .get("go.asset")
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest("go.asset is required".to_owned()))?;
        let module = form
            .field("go.module")
            .or_else(|| form.field("module"))
            .map(str::trim);
        let version = form
            .field("go.version")
            .or_else(|| form.field("version"))
            .map(str::trim);
        if !(ctx.grant(View::Add).any() || ctx.grant(View::Edit).any()) {
            ctx.require(View::Add, "/")?;
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        self.hosted_store(&ctx, written, module, version).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn module_zips_name_their_module_and_version() {
        let archive = zip::write(&[
            (
                "example.com/hello@v1.0.0/go.mod",
                b"module example.com/hello\n\ngo 1.22\n",
            ),
            ("example.com/hello@v1.0.0/hello.go", b"package hello\n"),
        ]);
        let inspected = inspect(&zip::Memory(archive.clone()), None, None)
            .await
            .unwrap();
        assert_eq!(inspected.module, "example.com/hello");
        assert_eq!(inspected.version, "v1.0.0");
        assert!(String::from_utf8_lossy(&inspected.go_mod).contains("go 1.22"));
        let wrong = inspect(&zip::Memory(archive), Some("example.com/other"), None).await;
        assert!(wrong.is_err());
        let bare = zip::write(&[("example.com/plain@v0.1.0/plain.go", b"package plain\n")]);
        let inspected = inspect(&zip::Memory(bare), None, None).await.unwrap();
        assert_eq!(inspected.go_mod, b"module example.com/plain\n");
    }

    #[test]
    fn lists_leave_out_pseudo_versions_and_sort() {
        let body = list_body(vec![
            "v1.10.0".to_owned(),
            "v1.2.0".to_owned(),
            "v0.0.0-20240101120000-abcdefabcdef".to_owned(),
            "v1.2.0".to_owned(),
        ]);
        assert_eq!(String::from_utf8(body).unwrap(), "v1.2.0\nv1.10.0\n");
    }
}
