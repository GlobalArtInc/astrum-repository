use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, View};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use axum::http::StatusCode;
use futures_util::StreamExt;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, Statement, TransactionTrait,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::io::ReaderStream;

use crate::index;
use crate::names::{Algorithm, CacheKey, Entry, INDEX_PATH, PackageFile};
use crate::{CACHE_KIND, FILE_KIND, ID, INDEX_KIND, OPAM_KIND};

pub const MAX_OPAM: usize = 256 * 1024;
pub const MAX_FILE: usize = index::MAX_STORED_FILE;
pub const MAX_ENTRIES: usize = 50_000;
pub const MAX_TOTAL: usize = index::MAX_TOTAL;

const RETRY_AFTER_FAILURE: Duration = Duration::from_secs(30);
const GENERATION: &str = "1";
const GZIP_TYPE: &str = "application/gzip";
const TEXT_TYPE: &str = "text/plain";
const OCTET_STREAM: &str = "application/octet-stream";

pub enum Written {
    Status(StatusCode),
    TooLarge(String),
}

struct RepositoryState {
    permit: Arc<Semaphore>,
    failed_at: Mutex<Option<Instant>>,
}

impl RepositoryState {
    fn new() -> Self {
        Self {
            permit: Arc::new(Semaphore::new(1)),
            failed_at: Mutex::new(None),
        }
    }

    fn recently_failed(&self) -> bool {
        self.failed_at
            .lock()
            .expect("the failure time lock is never poisoned")
            .is_some_and(|at| at.elapsed() < RETRY_AFTER_FAILURE)
    }

    fn record_failure(&self) {
        *self
            .failed_at
            .lock()
            .expect("the failure time lock is never poisoned") = Some(Instant::now());
    }

    fn clear_failure(&self) {
        *self
            .failed_at
            .lock()
            .expect("the failure time lock is never poisoned") = None;
    }
}

#[derive(Default)]
pub struct Writers {
    repositories: Mutex<HashMap<String, Arc<RepositoryState>>>,
}

pub struct Locked {
    _permit: OwnedSemaphorePermit,
    txn: DatabaseTransaction,
    state: Arc<RepositoryState>,
}

impl Writers {
    fn state(&self, repository: &str) -> Arc<RepositoryState> {
        self.repositories
            .lock()
            .expect("the repository state lock is never poisoned")
            .entry(repository.to_owned())
            .or_insert_with(|| Arc::new(RepositoryState::new()))
            .clone()
    }

    async fn lock(&self, ctx: &Ctx) -> Result<Locked, ContentError> {
        let state = self.state(ctx.name());
        let permit = state
            .permit
            .clone()
            .acquire_owned()
            .await
            .map_err(ContentError::internal)?;
        let txn = ctx.db().begin().await?;
        txn.execute(Statement::from_sql_and_values(
            txn.get_database_backend(),
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            [format!("{ID}:{}", ctx.name()).into()],
        ))
        .await?;
        Ok(Locked {
            _permit: permit,
            txn,
            state,
        })
    }
}

fn files_limit(max_entries: usize) -> usize {
    max_entries.saturating_sub(1)
}

fn too_many() -> ContentError {
    ContentError::Conflict("too many package files".to_owned())
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

async fn authorize(ctx: &Ctx, path: &str) -> Result<(), ContentError> {
    let redeploys = write_policy(ctx) == WritePolicy::Allow;
    let exists = store::find(ctx.db(), ctx.name(), path).await?.is_some();
    ctx.require(
        if exists && redeploys {
            View::Edit
        } else {
            View::Add
        },
        path,
    )
}

fn component(file: &PackageFile) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: file.name.clone(),
        version: file.version.clone(),
        kind: String::new(),
        attributes: Value::Object(Default::default()),
    }
}

fn package_kinds() -> [&'static str; 2] {
    [OPAM_KIND, FILE_KIND]
}

async fn packages(db: &impl ConnectionTrait, repository: &str) -> Result<Vec<Asset>, ContentError> {
    let mut found = asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.is_in(package_kinds()))
        .all(db)
        .await?;
    found.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    Ok(found)
}

async fn package_count(db: &impl ConnectionTrait, repository: &str) -> Result<u64, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.is_in(package_kinds()))
        .count(db)
        .await?)
}

fn total_after(summary: &[(String, String, i64)], path: &str, added: usize) -> u64 {
    let mut total = added as u64;
    for (stored, _, size) in summary {
        if stored != path {
            total = total.saturating_add(size.max(&0).unsigned_abs());
        }
    }
    total
}

fn too_large(total: u64) -> Written {
    Written::TooLarge(format!(
        "the package files of this repository would take {total} bytes, and at most {MAX_TOTAL} bytes are kept"
    ))
}

async fn package_summary(
    db: &impl ConnectionTrait,
    repository: &str,
) -> Result<Vec<(String, String, i64)>, ContentError> {
    Ok(asset::Entity::find()
        .select_only()
        .column(asset::Column::Path)
        .column(asset::Column::Sha256)
        .column(asset::Column::Size)
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.is_in(package_kinds()))
        .into_tuple()
        .all(db)
        .await?)
}

fn fingerprint(mut summary: Vec<(String, String, i64)>) -> String {
    summary.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut digest = Sha256::new();
    digest.update(GENERATION.as_bytes());
    for (path, sha256, size) in summary {
        digest.update(format!("\n{path}\t{sha256}\t{size}").as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_fingerprint(found: &Asset) -> Option<&str> {
    found
        .attributes
        .get(ID)
        .and_then(|opam| opam.get("fingerprint"))
        .and_then(Value::as_str)
}

fn file_stream(file: std::fs::File) -> astrum_blob::ContentStream {
    ReaderStream::new(tokio::fs::File::from_std(file)).boxed()
}

async fn rebuild(ctx: &Ctx, locked: &Locked, max_entries: usize) -> Result<Asset, ContentError> {
    let assets = packages(&locked.txn, ctx.name()).await?;
    if assets.len() > files_limit(max_entries) {
        return Err(too_many());
    }
    let total = assets.iter().fold(0u64, |sum, found| {
        sum.saturating_add(found.size.max(0) as u64)
    });
    if total > MAX_TOTAL as u64 {
        return Err(ContentError::Conflict(format!(
            "the package files take {total} bytes, and at most {MAX_TOTAL} bytes can be indexed"
        )));
    }
    let wanted = fingerprint(
        assets
            .iter()
            .map(|found| (found.path.clone(), found.sha256.clone(), found.size))
            .collect(),
    );
    let file = index::build(ctx, &assets).await?;
    let mut put = PutAsset::hosted(ctx, INDEX_PATH)
        .kind(INDEX_KIND)
        .content_type(GZIP_TYPE)
        .attributes(json!({ ID: { "fingerprint": wanted, "entries": assets.len() + 1 } }));
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::Allow);
    store::put(ctx, put, file_stream(file)).await
}

fn stale(previous: Option<Asset>) -> Result<Asset, ContentError> {
    previous.ok_or_else(|| {
        ContentError::Unavailable(
            "the index of this repository cannot be built now; try again later".to_owned(),
        )
    })
}

async fn ensure(ctx: &Ctx, writers: &Writers, max_entries: usize) -> Result<Asset, ContentError> {
    let state = writers.state(ctx.name());
    let current = fingerprint(package_summary(ctx.db(), ctx.name()).await?);
    let previous = store::find(ctx.db(), ctx.name(), INDEX_PATH).await?;
    if let Some(found) = &previous
        && stored_fingerprint(found) == Some(current.as_str())
    {
        return Ok(found.clone());
    }
    if state.recently_failed() {
        return stale(previous);
    }
    let locked = writers.lock(ctx).await?;
    let current = fingerprint(package_summary(&locked.txn, ctx.name()).await?);
    let previous = store::find(&locked.txn, ctx.name(), INDEX_PATH).await?;
    if let Some(found) = &previous
        && stored_fingerprint(found) == Some(current.as_str())
    {
        locked.txn.commit().await?;
        return Ok(found.clone());
    }
    if state.recently_failed() {
        return stale(previous);
    }
    match rebuild(ctx, &locked, max_entries).await {
        Ok(rebuilt) => {
            locked.txn.commit().await?;
            state.clear_failure();
            Ok(rebuilt)
        }
        Err(error) => {
            state.record_failure();
            tracing::warn!(
                repository = ctx.name(),
                error = format!("{error:#}"),
                "the index could not be rebuilt; the previous index is served until the next attempt"
            );
            stale(previous)
        }
    }
}

pub async fn index_asset(
    ctx: &Ctx,
    writers: &Writers,
    max_entries: usize,
) -> Result<Asset, ContentError> {
    ctx.require(View::Read, INDEX_PATH)?;
    ensure(ctx, writers, max_entries).await
}

pub async fn put_package(
    ctx: &Ctx,
    writers: &Writers,
    max_entries: usize,
    file: &PackageFile,
    request: ContentRequest,
) -> Result<Written, ContentError> {
    let path = file.path();
    authorize(ctx, &path).await?;
    let (limit, kind, content_type) = match file.entry {
        Entry::Opam => (MAX_OPAM, OPAM_KIND, TEXT_TYPE),
        Entry::File(_) => (MAX_FILE, FILE_KIND, OCTET_STREAM),
    };
    let body = request.bytes(limit).await.map_err(|error| match error {
        ContentError::BadRequest(message) => {
            ContentError::BadRequest(format!("{path} is at most {limit} bytes long: {message}"))
        }
        other => other,
    })?;
    if file.entry == Entry::Opam && std::str::from_utf8(&body).is_err() {
        return Err(ContentError::BadRequest(format!(
            "the opam file {path} is not UTF-8 text"
        )));
    }
    let locked = writers.lock(ctx).await?;
    let existed = store::find(&locked.txn, ctx.name(), &path).await?.is_some();
    if !existed && package_count(&locked.txn, ctx.name()).await? >= files_limit(max_entries) as u64
    {
        return Err(too_many());
    }
    let summary = package_summary(&locked.txn, ctx.name()).await?;
    let total = total_after(&summary, &path, body.len());
    if total > MAX_TOTAL as u64 {
        return Ok(too_large(total));
    }
    let mut put = PutAsset::hosted(ctx, &path)
        .component(component(file))
        .kind(kind)
        .content_type(content_type);
    put.check_content_type = false;
    store::put(ctx, put, astrum_blob::content_of(body)).await?;
    rebuild(ctx, &locked, max_entries).await?;
    locked.txn.commit().await?;
    locked.state.clear_failure();
    Ok(Written::Status(match existed {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::CREATED,
    }))
}

pub async fn delete_package(
    ctx: &Ctx,
    writers: &Writers,
    max_entries: usize,
    file: &PackageFile,
) -> Result<StatusCode, ContentError> {
    let path = file.path();
    ctx.require(View::Delete, &path)?;
    let locked = writers.lock(ctx).await?;
    let found = store::find(&locked.txn, ctx.name(), &path)
        .await?
        .ok_or_else(|| ContentError::not_found(path.clone()))?;
    store::delete_asset(&ctx.content, found).await?;
    rebuild(ctx, &locked, max_entries).await?;
    locked.txn.commit().await?;
    locked.state.clear_failure();
    Ok(StatusCode::NO_CONTENT)
}

pub async fn put_cache(
    ctx: &Ctx,
    key: &CacheKey,
    request: ContentRequest,
) -> Result<StatusCode, ContentError> {
    let path = key.path();
    authorize(ctx, &path).await?;
    if write_policy(ctx) == WritePolicy::Deny {
        return Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        )));
    }
    let written = store::write_blob(ctx, request.into_stream()).await?;
    let actual = match key.algorithm {
        Algorithm::Sha256 => &written.digests.sha256,
        Algorithm::Sha512 => &written.digests.sha512,
    };
    if *actual != key.hex {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Err(ContentError::BadRequest(format!(
            "the body of {path} does not have the {} digest {}",
            key.algorithm.as_str(),
            key.hex
        )));
    }
    if store::find(ctx.db(), ctx.name(), &path).await?.is_some() {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Ok(StatusCode::NO_CONTENT);
    }
    let mut put = PutAsset::hosted(ctx, &path)
        .kind(CACHE_KIND)
        .content_type(OCTET_STREAM);
    put.check_content_type = false;
    store::adopt(ctx, put, &written).await?;
    Ok(StatusCode::CREATED)
}

pub async fn delete_cache(ctx: &Ctx, key: &CacheKey) -> Result<StatusCode, ContentError> {
    let path = key.path();
    ctx.require(View::Delete, &path)?;
    match store::delete(ctx, &path).await? {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(ContentError::not_found(path)),
    }
}
