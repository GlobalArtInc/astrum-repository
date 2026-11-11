use astrum_content::store::{self, PutAsset};
use astrum_content::{ContentError, ContentRequest, Ctx, View};
use astrum_repo::config::WritePolicy;
use axum::http::StatusCode;
use axum::response::Response;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::http::{HeaderValue, header};
use bytes::Bytes;
use chrono::{SecondsFormat, Utc};
use futures_util::{Stream, StreamExt};
use sea_orm::{ConnectionTrait, DatabaseTransaction, Statement, TransactionTrait};
use serde_json::{Map, Value, json};
use tokio::io::AsyncReadExt;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::io::ReaderStream;

use crate::archive::{self, ArchiveError, MAX_COMPRESSED, Unpacked};
use crate::documents;
use crate::hosted::{self, FILE_KIND, META_KIND};
use crate::jsonc;
use crate::names::{self, Package};
use crate::reply;

const MAX_CONFIG: u64 = 1024 * 1024;
const DEFAULT_CONFIG: &str = "/deno.json";
const EXTRA_DRAIN: usize = 2 * MAX_COMPRESSED;
const RETRY_AFTER: &str = "5";
const INTAKE_SLOTS: usize = 32;
const PERMIT_WAIT: Duration = Duration::from_secs(30);
const BODY_DEADLINE: Duration = Duration::from_secs(300);

pub type Hook = Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone)]
pub struct Limits {
    pub intake: usize,
    pub writes: Option<usize>,
    pub permit_wait: Duration,
    pub body_deadline: Duration,
    pub spool_dir: Option<PathBuf>,
    pub after_file: Option<Hook>,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            intake: INTAKE_SLOTS,
            writes: None,
            permit_wait: PERMIT_WAIT,
            body_deadline: BODY_DEADLINE,
            spool_dir: None,
            after_file: None,
        }
    }
}

pub struct Gates {
    pub intake: Arc<Semaphore>,
    pub writing: Arc<Semaphore>,
    pub limits: Limits,
}

pub enum Failure {
    Coded(StatusCode, &'static str, String),
    Duplicate(String, Value),
    Busy,
    Timeout,
    Content(ContentError),
}

impl From<ContentError> for Failure {
    fn from(error: ContentError) -> Self {
        Failure::Content(error)
    }
}

impl Failure {
    pub fn into_response(self) -> Response {
        match self {
            Failure::Coded(status, code, message) => reply::failure(status, code, &message),
            Failure::Duplicate(message, task) => reply::json(
                StatusCode::BAD_REQUEST,
                &json!({"code": "duplicateVersionPublish", "message": message, "task": task}),
            ),
            Failure::Busy => {
                let mut response = reply::failure(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "busy",
                    "too many packages are being published; try again shortly",
                );
                response
                    .headers_mut()
                    .insert(header::RETRY_AFTER, HeaderValue::from_static(RETRY_AFTER));
                response
            }
            Failure::Timeout => reply::failure(
                StatusCode::REQUEST_TIMEOUT,
                "requestTimeout",
                "the package did not arrive in time; try again",
            ),
            Failure::Content(error) => reply::from_error(error),
        }
    }
}

fn invalid_config(message: impl Into<String>) -> Failure {
    Failure::Coded(StatusCode::BAD_REQUEST, "invalidConfig", message.into())
}

fn task(package: &Package, version: &str) -> Value {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "status": "success",
        "error": null,
        "packageScope": package.scope,
        "packageName": package.name,
        "packageVersion": version,
        "createdAt": now,
        "updatedAt": now,
    })
}

fn duplicate(ctx: &Ctx, package: &Package, version: &str) -> Failure {
    Failure::Duplicate(
        format!(
            "{} {version} is already published to {}; a published version is never replaced, publish a new version",
            package.display(),
            ctx.name()
        ),
        task(package, version),
    )
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

async fn drain_stream(
    mut stream: impl Stream<Item = Result<Bytes, axum::Error>> + Unpin,
    limit: usize,
) {
    let mut seen = 0usize;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                seen += bytes.len();
                if seen > limit {
                    return;
                }
            }
            Err(error) => {
                tracing::debug!(%error, "a publish body ended early while it was being read out");
                return;
            }
        }
    }
}

pub async fn drain(request: &ContentRequest) {
    drain_stream(request.take_body().into_data_stream(), EXTRA_DRAIN).await;
}

async fn read_body(request: &ContentRequest) -> Result<Vec<u8>, Failure> {
    let mut stream = request.take_body().into_data_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| {
            Failure::Coded(
                StatusCode::BAD_REQUEST,
                "invalidTarball",
                format!("the request body cannot be read: {error}"),
            )
        })?;
        if body.len() + chunk.len() > MAX_COMPRESSED {
            drain_stream(stream, EXTRA_DRAIN).await;
            return Err(Failure::Coded(
                StatusCode::PAYLOAD_TOO_LARGE,
                "tarballTooLarge",
                format!("the archive is larger than {MAX_COMPRESSED} bytes"),
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn config_path(request: &ContentRequest) -> Result<String, Failure> {
    let given = request
        .query
        .as_deref()
        .and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(key, _)| key == "config")
                .map(|(_, value)| value.into_owned())
        })
        .unwrap_or_else(|| DEFAULT_CONFIG.to_owned());
    archive::clean_path(given.as_bytes()).map_err(|error| match error {
        ArchiveError::Invalid(message)
        | ArchiveError::TooLarge(message)
        | ArchiveError::Storage(message) => {
            invalid_config(format!("the config path is not usable: {message}"))
        }
    })
}

pub fn exports_of(
    config: &Value,
    package: &Package,
    version: &str,
) -> Result<Map<String, Value>, String> {
    let object = config
        .as_object()
        .ok_or_else(|| "the config is not a JSON object".to_owned())?;
    let name = object.get("name").and_then(Value::as_str);
    if name != Some(package.display().as_str()) {
        return Err(format!(
            "the config names {} but the package is {}",
            name.unwrap_or("nothing"),
            package.display()
        ));
    }
    let declared = object.get("version").and_then(Value::as_str);
    if declared != Some(version) {
        return Err(format!(
            "the config has version {} but {version} is being published",
            declared.unwrap_or("nothing")
        ));
    }
    let exports = match object.get("exports") {
        Some(Value::String(target)) => {
            let mut single = Map::new();
            single.insert(".".to_owned(), Value::String(target.clone()));
            single
        }
        Some(Value::Object(map)) if !map.is_empty() => map.clone(),
        _ => return Err("the config needs exports: a string or a non-empty object".to_owned()),
    };
    for (key, target) in &exports {
        if key != "." && !key.starts_with("./") {
            return Err(format!("the export {key} is not \".\" or a ./ path"));
        }
        if !target.is_string() {
            return Err(format!("the target of the export {key} is not a string"));
        }
    }
    Ok(exports)
}

pub fn check_targets(
    exports: &Map<String, Value>,
    has_file: impl Fn(&str) -> bool,
) -> Result<(), String> {
    for (key, target) in exports {
        let target = target.as_str().unwrap_or_default();
        let path = target.strip_prefix("./").unwrap_or(target);
        let usable = archive::clean_path(path.as_bytes()).is_ok_and(|clean| clean == path);
        if !usable || target.starts_with('/') || !has_file(path) {
            return Err(format!(
                "the export {key} points at {target}, which is not a file of the archive"
            ));
        }
    }
    Ok(())
}

fn archive_failure(error: ArchiveError) -> Failure {
    match error {
        ArchiveError::Invalid(message) => {
            Failure::Coded(StatusCode::BAD_REQUEST, "invalidTarball", message)
        }
        ArchiveError::TooLarge(message) => {
            Failure::Coded(StatusCode::PAYLOAD_TOO_LARGE, "tarballTooLarge", message)
        }
        ArchiveError::Storage(message) => {
            Failure::Content(ContentError::internal(std::io::Error::other(message)))
        }
    }
}

fn read_config(
    unpacked: &Unpacked,
    config_path: &str,
    package: &Package,
    version: &str,
) -> Result<Map<String, Value>, Failure> {
    let entry = unpacked
        .find(config_path)
        .ok_or_else(|| invalid_config(format!("the archive has no /{config_path}")))?;
    let bytes = unpacked
        .read_all(entry, MAX_CONFIG)
        .map_err(ContentError::internal)?
        .ok_or_else(|| {
            invalid_config(format!("/{config_path} is larger than {MAX_CONFIG} bytes"))
        })?;
    let plain = jsonc::to_json(&bytes).map_err(|error| {
        invalid_config(format!(
            "/{config_path} is not valid JSON with comments: {error}"
        ))
    })?;
    let config: Value = serde_json::from_slice(&plain).map_err(|error| {
        invalid_config(format!(
            "/{config_path} is not valid JSON with comments: {error}"
        ))
    })?;
    let exports = exports_of(&config, package, version).map_err(invalid_config)?;
    check_targets(&exports, |path| unpacked.find(path).is_some()).map_err(invalid_config)?;
    Ok(exports)
}

async fn lock(
    ctx: &Ctx,
    package: &Package,
    version: &str,
) -> Result<DatabaseTransaction, ContentError> {
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("jsr:{}:{}/{version}", ctx.name(), package.display()).into()],
    ))
    .await?;
    Ok(txn)
}

fn put_of(
    ctx: &Ctx,
    package: &Package,
    version: &str,
    path: &str,
    kind: &str,
    content_type: &str,
) -> PutAsset {
    let mut put = PutAsset::hosted(ctx, path)
        .kind(kind)
        .content_type(content_type)
        .component(hosted::component_key(package, version));
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::AllowOnce);
    put
}

fn refused(ctx: &Ctx, package: &Package, version: &str, error: ContentError) -> Failure {
    match error {
        ContentError::Conflict(_) => duplicate(ctx, package, version),
        other => Failure::Content(other),
    }
}

struct Staged<'a> {
    meta_path: &'a str,
    meta: Vec<u8>,
    hook: Option<&'a Hook>,
}

async fn store_all(
    ctx: &Ctx,
    package: &Package,
    version: &str,
    unpacked: &Unpacked,
    staged: Staged<'_>,
    written: &mut Vec<String>,
) -> Result<(), Failure> {
    let Staged {
        meta_path,
        meta,
        hook,
    } = staged;
    for entry in &unpacked.files {
        let path = package.file_path(version, &entry.path);
        let put = put_of(
            ctx,
            package,
            version,
            &path,
            FILE_KIND,
            names::content_type(&entry.path),
        );
        let file = tokio::fs::File::from_std(unpacked.open(entry).map_err(ContentError::internal)?);
        let blob = store::write_blob(ctx, ReaderStream::new(file.take(entry.size)).boxed()).await?;
        if blob.digests.sha256 != entry.sha256 || blob.size != entry.size {
            store::discard(ctx.content.clone(), &blob.store, blob.id.uuid());
            return Err(ContentError::internal(std::io::Error::other(
                "a stored file does not match the archive entry it came from",
            ))
            .into());
        }
        store::adopt(ctx, put, &blob)
            .await
            .map_err(|error| refused(ctx, package, version, error))?;
        written.push(path.clone());
        if let Some(hook) = hook {
            hook(path).await;
        }
    }
    let put = put_of(
        ctx,
        package,
        version,
        meta_path,
        META_KIND,
        reply::JSON_TYPE,
    );
    let blob = store::write_blob(ctx, astrum_blob::content_of(meta)).await?;
    store::adopt(ctx, put, &blob)
        .await
        .map_err(|error| refused(ctx, package, version, error))?;
    written.push(meta_path.to_owned());
    Ok(())
}

async fn write(
    ctx: &Ctx,
    package: &Package,
    version: &str,
    unpacked: &Unpacked,
    meta_path: &str,
    meta: Vec<u8>,
    hook: Option<&Hook>,
) -> Result<(), Failure> {
    let txn = lock(ctx, package, version).await?;
    if store::find(ctx.db(), ctx.name(), meta_path)
        .await?
        .is_some()
    {
        return Err(duplicate(ctx, package, version));
    }
    hosted::discard_version(ctx, package, version).await?;
    let mut written = Vec::new();
    match store_all(
        ctx,
        package,
        version,
        unpacked,
        Staged {
            meta_path,
            meta,
            hook,
        },
        &mut written,
    )
    .await
    {
        Ok(()) => {
            txn.commit().await.map_err(ContentError::from)?;
            Ok(())
        }
        Err(failure) => {
            for path in written.iter().rev() {
                if let Err(error) = store::delete(ctx, path).await {
                    tracing::warn!(repository = ctx.name(), path, %error, "a file of a failed publish stays in the repository");
                }
            }
            Err(failure)
        }
    }
}

async fn acquire(gate: &Arc<Semaphore>, wait: Duration) -> Result<OwnedSemaphorePermit, Failure> {
    match tokio::time::timeout(wait, gate.clone().acquire_owned()).await {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(error)) => Err(ContentError::internal(error).into()),
        Err(_) => Err(Failure::Busy),
    }
}

struct Job {
    rights: Ctx,
    target: Ctx,
    package: Package,
    version: String,
    config: String,
    body: Vec<u8>,
    limits: Limits,
}

async fn run(job: Job, permit: OwnedSemaphorePermit) -> Result<(), Failure> {
    let outcome = process(job).await;
    drop(permit);
    outcome
}

async fn process(job: Job) -> Result<(), Failure> {
    let Job {
        rights,
        target,
        package,
        version,
        config,
        body,
        limits,
    } = job;
    let meta_path = package.version_meta_path(&version);
    let spool = limits.spool_dir.clone();
    let unpacked = tokio::task::spawn_blocking(move || archive::unpack(&body, spool.as_deref()))
        .await
        .map_err(ContentError::internal)?
        .map_err(archive_failure)?;
    let exports = read_config(&unpacked, &config, &package, &version)?;
    let meta = serde_json::to_vec(&documents::version_meta(&unpacked, &exports))
        .map_err(ContentError::internal)?;
    for entry in &unpacked.files {
        rights.require(View::Add, &package.file_path(&version, &entry.path))?;
    }
    write(
        &target,
        &package,
        &version,
        &unpacked,
        &meta_path,
        meta,
        limits.after_file.as_ref(),
    )
    .await
}

pub async fn publish(
    rights: &Ctx,
    target: &Ctx,
    gates: &Gates,
    request: &ContentRequest,
    package: &Package,
    version: &str,
) -> Result<Response, Failure> {
    let meta_path = package.version_meta_path(version);
    if let Err(error) = rights.require(View::Add, &meta_path) {
        drain(request).await;
        return Err(error.into());
    }
    if write_policy(target) == WritePolicy::Deny {
        drain(request).await;
        return Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            target.name()
        ))
        .into());
    }
    if store::find(target.db(), target.name(), &meta_path)
        .await?
        .is_some()
    {
        drain(request).await;
        return Err(duplicate(target, package, version));
    }
    let limits = &gates.limits;
    let intake = match acquire(&gates.intake, limits.permit_wait).await {
        Ok(permit) => permit,
        Err(failure) => {
            drain(request).await;
            return Err(failure);
        }
    };
    let body = match tokio::time::timeout(limits.body_deadline, read_body(request)).await {
        Ok(read) => read?,
        Err(_) => return Err(Failure::Timeout),
    };
    let config = config_path(request)?;
    let writing = acquire(&gates.writing, limits.permit_wait).await?;
    drop(intake);
    let job = Job {
        rights: rights.clone(),
        target: target.clone(),
        package: package.clone(),
        version: version.to_owned(),
        config,
        body,
        limits: limits.clone(),
    };
    let handle = tokio::spawn(run(job, writing));
    match handle.await {
        Ok(outcome) => outcome?,
        Err(error) => return Err(ContentError::internal(error).into()),
    }
    Ok(reply::json(StatusCode::OK, &task(package, version)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Package {
        Package::new("acme", "hello").unwrap()
    }

    #[test]
    fn exports_may_be_a_string_or_an_object_and_the_names_must_match() {
        let string = json!({"name": "@acme/hello", "version": "0.1.0", "exports": "./mod.ts"});
        let exports = exports_of(&string, &package(), "0.1.0").unwrap();
        assert_eq!(Value::Object(exports), json!({".": "./mod.ts"}));
        let object = json!({"name": "@acme/hello", "version": "0.1.0", "exports": {".": "./mod.ts", "./util": "./util.ts"}});
        assert_eq!(exports_of(&object, &package(), "0.1.0").unwrap().len(), 2);

        let other_name = json!({"name": "@acme/other", "version": "0.1.0", "exports": "./mod.ts"});
        assert!(
            exports_of(&other_name, &package(), "0.1.0")
                .unwrap_err()
                .contains("names")
        );
        let other_version =
            json!({"name": "@acme/hello", "version": "0.2.0", "exports": "./mod.ts"});
        assert!(
            exports_of(&other_version, &package(), "0.1.0")
                .unwrap_err()
                .contains("version")
        );
        for bad in [
            json!({"name": "@acme/hello", "version": "0.1.0"}),
            json!({"name": "@acme/hello", "version": "0.1.0", "exports": {}}),
            json!({"name": "@acme/hello", "version": "0.1.0", "exports": {"util": "./u.ts"}}),
            json!({"name": "@acme/hello", "version": "0.1.0", "exports": {".": 5}}),
            json!([]),
        ] {
            assert!(exports_of(&bad, &package(), "0.1.0").is_err(), "{bad}");
        }
    }

    #[test]
    fn every_export_target_must_be_a_file_of_the_archive() {
        let exports = |target: &str| {
            let mut map = Map::new();
            map.insert(".".to_owned(), json!(target));
            map
        };
        let has = |path: &str| path == "mod.ts" || path == "sub/a.ts";
        assert!(check_targets(&exports("./mod.ts"), has).is_ok());
        assert!(check_targets(&exports("sub/a.ts"), has).is_ok());
        for bad in [
            "./missing.ts",
            "/mod.ts",
            "./../mod.ts",
            "./sub/",
            ".//mod.ts",
            "",
        ] {
            assert!(check_targets(&exports(bad), has).is_err(), "{bad}");
        }
    }
}
