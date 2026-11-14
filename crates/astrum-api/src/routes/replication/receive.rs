use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_blob::{BlobId, ContentStream, Digests, Written};
use astrum_content::store::{self, PutAsset};
use astrum_content::transfer::{
    BlobSource, Bundle, BundleAsset, Conflict, MAX_ASSETS, Outcome, REPLICATED, REPLICATION_VIA,
    Sink, Status, Terms, fingerprint, origin_of,
};
use astrum_content::{Content, ContentError, Ctx, Format, Grant, View, holds};
use astrum_db::audit::Event;
use astrum_db::entity::component;
use astrum_db::setting;
use astrum_replication::protocol::{
    BLOB, BLOBS_MISSING, CAPABILITIES, COMPONENTS, Capabilities, DEFAULT_PAGE, INSTANCE_HEADER,
    INVENTORY, INVENTORY_COMPARE, InventoryCompare, InventoryCompareReply, InventoryEntry,
    InventoryPage, MAX_COMPARE, MAX_MISSING, MAX_PAGE, MISSING, MissingReply, MissingRequest,
    PROTOCOL, Tombstone, component_key, parse_component_key, valid_sha256,
};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, QueryFilter, Statement,
    TransactionTrait, Value,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{ApiError, AppState, Caller, Json};

const RECEIVE: &str = "astrum:replication:receive";
const INSTANCE_KEY: &str = "replication.instanceId";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const STAGED_HOURS: i32 = 24;
const HOLD_MARGIN_HOURS: u64 = 24;

const RECORD_INSTANCE: &str = "INSERT INTO setting (key, value, updated_at) \
     VALUES ($1, to_jsonb($2::text), now()) ON CONFLICT (key) DO NOTHING";

const PRESENT: &str = "SELECT sha256 FROM asset WHERE repository = $1 AND sha256 = ANY($2::text[]) \
     UNION \
     SELECT sha256 FROM replication_staged_blob \
     WHERE repository = $1 AND sha256 = ANY($2::text[]) AND expires_at > now()";

const DROP_EXPIRED: &str = "DELETE FROM replication_staged_blob \
     WHERE repository = $1 AND sha256 = $2 AND expires_at <= now() \
     RETURNING blob_store, blob_id";

const ASSET_EXISTS: &str =
    "SELECT 1 AS present FROM asset WHERE repository = $1 AND sha256 = $2 LIMIT 1";

const AUDIT_DOMAIN: &str = "repository.content";

const ASSET_BLOB: &str = "SELECT blob_store, blob_id, sha1, sha256, sha512, md5 FROM asset \
     WHERE repository = $1 AND sha256 = $2 ORDER BY id LIMIT 1";

const STAGED_BLOB: &str = "SELECT blob_store, blob_id, sha1, sha256, sha512, md5 \
     FROM replication_staged_blob \
     WHERE repository = $1 AND sha256 = $2 AND expires_at > now()";

const RELEASE_STAGED: &str = "DELETE FROM replication_staged_blob s \
     WHERE s.repository = $1 AND s.sha256 = ANY($2::text[]) \
     AND EXISTS (SELECT 1 FROM asset a WHERE a.repository = s.repository AND a.sha256 = s.sha256) \
     RETURNING s.blob_store, s.blob_id";

const SET_CONTEXT: &str =
    "SELECT set_config('astrum.action', $1, true), set_config('astrum.initiator', $2, true)";

const LOCK_COMPONENT: &str = "SELECT id, attributes FROM component \
     WHERE repository = $1 AND namespace = $2 AND name = $3 AND version = $4 FOR UPDATE";

const COMPONENT_BLOBS: &str = "SELECT blob_store, blob_id FROM asset WHERE component_id = $1";

const DELETE_COMPONENT: &str = "DELETE FROM component WHERE id = $1";

const COMPONENTS_BY_KEY: &str = "SELECT c.* FROM component c \
     JOIN unnest($2::text[], $3::text[], $4::text[]) AS k(namespace, name, version) \
     ON c.namespace = k.namespace AND c.name = k.name AND c.version = k.version \
     WHERE c.repository = $1";

const ESCAPED: &str = "replace(replace(replace({}, '%', '%25'), '/', '%2F'), '@', '%40')";

const STAGE: &str = "INSERT INTO replication_staged_blob \
     (repository, sha256, blob_store, blob_id, size, sha1, sha512, md5, uploaded_by, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now() + make_interval(hours => $10)) \
     ON CONFLICT (repository, sha256) DO NOTHING";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(CAPABILITIES, get(capabilities))
        .route(MISSING, post(missing_blobs))
        .route(BLOB, put(put_blob))
        .route(COMPONENTS, post(commit).delete(tombstone))
        .route(INVENTORY_COMPARE, post(compare))
        .route(INVENTORY, get(inventory))
}

enum Refusal {
    Api(ApiError),
    Status(StatusCode, &'static str, String),
    Missing(Vec<String>),
}

impl From<ApiError> for Refusal {
    fn from(error: ApiError) -> Self {
        Refusal::Api(error)
    }
}

impl From<sea_orm::DbErr> for Refusal {
    fn from(error: sea_orm::DbErr) -> Self {
        Refusal::Api(error.into())
    }
}

impl From<ContentError> for Refusal {
    fn from(error: ContentError) -> Self {
        match error {
            ContentError::Internal(error) => Refusal::Api(ApiError::Internal(error)),
            ContentError::Conflict(message) => {
                Refusal::Status(StatusCode::CONFLICT, "conflict", message)
            }
            other => Refusal::Status(other.status(), "refused", other.to_string()),
        }
    }
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        match self {
            Refusal::Api(error) => error.into_response(),
            Refusal::Status(status, code, message) => (
                status,
                axum::Json(json!({"id": "*", "code": code, "message": message})),
            )
                .into_response(),
            Refusal::Missing(missing) => (
                StatusCode::CONFLICT,
                axum::Json(json!({
                    "id": "*",
                    "code": BLOBS_MISSING,
                    "message": format!("{} blob(s) of the component are not on this server", missing.len()),
                    "missing": missing,
                })),
            )
                .into_response(),
        }
    }
}

fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

fn internal(what: &str) -> ApiError {
    ApiError::internal(anyhow::anyhow!("{what}"))
}

async fn instance_id(state: &AppState) -> Result<Uuid, ApiError> {
    let db = state.db().conn();
    db.execute(statement(
        RECORD_INSTANCE,
        vec![INSTANCE_KEY.into(), Uuid::new_v4().to_string().into()],
    ))
    .await?;
    let stored: String = setting::get(db, INSTANCE_KEY)
        .await?
        .ok_or_else(|| internal("the setting replication.instanceId was not stored"))?;
    Uuid::parse_str(&stored)
        .map_err(|_| internal("the setting replication.instanceId is not a UUID"))
}

async fn capabilities(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Capabilities>, ApiError> {
    caller.require(RECEIVE)?;
    let instance_id = instance_id(&state).await?.to_string();
    let formats = state
        .content()
        .formats()
        .iter()
        .filter(|format| format.transfer().is_some())
        .map(|format| format.id().to_owned())
        .collect();
    Ok(axum::Json(Capabilities {
        protocol: PROTOCOL,
        instance_id,
        version: VERSION.to_owned(),
        formats,
    }))
}

async fn open(
    state: &AppState,
    caller: Caller,
    name: &str,
    view: View,
) -> Result<astrum_content::Ctx, Refusal> {
    caller.require(RECEIVE)?;
    let hidden = || ApiError::not_found(format!("Repository not found: {name}"));
    let repository = state
        .content()
        .repository(name)
        .await
        .map_err(ApiError::internal)?
        .filter(|found| found.name == name)
        .ok_or_else(hidden)?;
    let online = repository.online;
    let ctx = state
        .content()
        .context(repository, state.security().model(), caller.0, "", "");
    if !ctx.grant(View::Read).any() {
        return Err(hidden().into());
    }
    if !matches!(ctx.grant(view), Grant::Whole) {
        return Err(ApiError::forbidden().into());
    }
    if !online {
        return Err(ContentError::Unavailable(format!("Repository offline: {name}")).into());
    }
    Ok(ctx)
}

fn refuse(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal::Status(StatusCode::BAD_REQUEST, code, message.into())
}

#[derive(FromQueryResult)]
struct Digest {
    sha256: String,
}

async fn missing_blobs(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    Json(request): Json<MissingRequest>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller, &repository, View::Read).await?;
    if request.sha256.is_empty() {
        return Err(ApiError::invalid("sha256", "must name at least one digest").into());
    }
    if request.sha256.len() > MAX_MISSING {
        return Err(ApiError::invalid(
            "sha256",
            format!(
                "must name at most {MAX_MISSING} digests, not {}",
                request.sha256.len()
            ),
        )
        .into());
    }
    if let Some(position) = request
        .sha256
        .iter()
        .position(|digest| !valid_sha256(digest))
    {
        return Err(ApiError::invalid(
            "sha256",
            format!("the digest at position {position} is not 64 lowercase hexadecimal characters"),
        )
        .into());
    }
    let missing = absent(&state, ctx.name(), request.sha256).await?;
    Ok(axum::Json(MissingReply { missing }).into_response())
}

async fn absent(
    state: &AppState,
    repository: &str,
    digests: Vec<String>,
) -> Result<Vec<String>, sea_orm::DbErr> {
    let present: HashSet<String> = Digest::find_by_statement(statement(
        PRESENT,
        vec![repository.into(), digests.clone().into()],
    ))
    .all(state.db().conn())
    .await?
    .into_iter()
    .map(|row| row.sha256)
    .collect();
    let mut seen = HashSet::new();
    Ok(digests
        .into_iter()
        .filter(|digest| !present.contains(digest) && seen.insert(digest.clone()))
        .collect())
}

#[derive(FromQueryResult)]
struct Stale {
    blob_store: String,
    blob_id: Uuid,
}

#[derive(Debug, PartialEq, Eq)]
enum Staged {
    Created,
    Present,
}

fn hold_ttl() -> Duration {
    Duration::from_secs((STAGED_HOURS as u64 + HOLD_MARGIN_HOURS) * 3600)
}

async fn stage(
    state: &AppState,
    repository: &str,
    written: &Written,
    uploaded_by: &str,
) -> Result<Staged, Refusal> {
    let size = i64::try_from(written.size).map_err(|_| internal("a blob is larger than i64"))?;
    let txn = state.db().conn().begin().await?;
    let stale = Stale::find_by_statement(statement(
        DROP_EXPIRED,
        vec![repository.into(), written.digests.sha256.clone().into()],
    ))
    .all(&txn)
    .await?;
    let stale: Vec<(String, BlobId)> = stale
        .into_iter()
        .map(|row| (row.blob_store, row.blob_id.into()))
        .collect();
    holds::lapse(&txn, &stale).await?;
    let named = txn
        .query_one(statement(
            ASSET_EXISTS,
            vec![repository.into(), written.digests.sha256.clone().into()],
        ))
        .await?
        .is_some();
    let inserted = match named {
        true => 0,
        false => txn
            .execute(statement(
                STAGE,
                vec![
                    repository.into(),
                    written.digests.sha256.clone().into(),
                    written.store.clone().into(),
                    written.id.uuid().into(),
                    size.into(),
                    written.digests.sha1.clone().into(),
                    written.digests.sha512.clone().into(),
                    written.digests.md5.clone().into(),
                    uploaded_by.into(),
                    STAGED_HOURS.into(),
                ],
            ))
            .await?
            .rows_affected(),
    };
    if inserted > 0 {
        holds::hold(&txn, &written.store, written.id, hold_ttl())
            .await
            .map_err(ContentError::from)?;
    }
    txn.commit().await?;
    Ok(match inserted {
        0 => Staged::Present,
        _ => Staged::Created,
    })
}

async fn discard(content: &Content, written: &Written) {
    let deleted = match content.blob_stores().get(&written.store).await {
        Ok(store) => store.delete(written.id).await.map(|_| ()),
        Err(error) => Err(error),
    };
    if let Err(error) = deleted {
        tracing::warn!(%error, store = written.store, blob = %written.id, "a replicated blob that was not kept stays in its store until compaction collects it");
    }
}

async fn put_blob(
    State(state): State<AppState>,
    caller: Caller,
    Path((repository, sha256)): Path<(String, String)>,
    body: Body,
) -> Response {
    match receive_blob(&state, caller, &repository, &sha256, body).await {
        Ok(status) => status.into_response(),
        Err(refusal) => refusal.into_response(),
    }
}

async fn receive_blob(
    state: &AppState,
    caller: Caller,
    repository: &str,
    sha256: &str,
    body: Body,
) -> Result<StatusCode, Refusal> {
    let uploaded_by = caller.initiator();
    let ctx = open(state, caller, repository, View::Add).await?;
    if ctx.kind() != RepositoryType::Hosted {
        return Err(refuse(
            "not_hosted",
            format!("{repository} is not a hosted repository and takes no replicated blobs"),
        ));
    }
    if !valid_sha256(sha256) {
        return Err(refuse(
            "bad_digest",
            "the digest in the path is not 64 lowercase hexadecimal characters",
        ));
    }
    let stream: ContentStream = body.into_data_stream().map_err(io::Error::other).boxed();
    let written = store::write_blob(&ctx, stream).await?;
    if written.digests.sha256 != sha256 {
        discard(state.content(), &written).await;
        return Err(Refusal::Status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "digest_mismatch",
            format!(
                "the body has the SHA-256 {}, not the {sha256} of the path",
                written.digests.sha256
            ),
        ));
    }
    let staged = match stage(state, ctx.name(), &written, &uploaded_by).await {
        Ok(staged) => staged,
        Err(refusal) => {
            discard(state.content(), &written).await;
            return Err(refusal);
        }
    };
    if staged == Staged::Present {
        discard(state.content(), &written).await;
    }
    tracing::debug!(
        repository = ctx.name(),
        sha256,
        size = written.size,
        created = staged == Staged::Created,
        "a replicated blob arrived"
    );
    Ok(match staged {
        Staged::Created => StatusCode::CREATED,
        Staged::Present => StatusCode::OK,
    })
}

fn conflict(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal::Status(StatusCode::CONFLICT, code, message.into())
}

fn hosted(ctx: &Ctx) -> Result<(), Refusal> {
    match ctx.kind() {
        RepositoryType::Hosted => Ok(()),
        _ => Err(refuse(
            "not_hosted",
            format!(
                "{} is not a hosted repository and takes no replicated components",
                ctx.name()
            ),
        )),
    }
}

fn format_of(state: &AppState, ctx: &Ctx) -> Result<Arc<dyn Format>, Refusal> {
    state
        .content()
        .format(ctx.format_id())
        .filter(|format| format.transfer().is_some())
        .ok_or_else(|| {
            refuse(
                "unsupported_format",
                format!(
                    "the format {} takes no part in replication",
                    ctx.format_id()
                ),
            )
        })
}

fn transfer_of(
    format: &Arc<dyn Format>,
) -> Result<&dyn astrum_content::transfer::Transfer, ContentError> {
    format.transfer().ok_or_else(|| {
        ContentError::internal(anyhow::anyhow!(
            "the format {} lost its transfer",
            format.id()
        ))
    })
}

fn instance(text: &str) -> Result<String, Refusal> {
    Uuid::parse_str(text)
        .map(|id| id.to_string())
        .map_err(|_| refuse("bad_instance", "an instance id is not a UUID"))
}

fn sender_of(headers: &HeaderMap) -> Result<Option<String>, Refusal> {
    let Some(value) = headers.get(INSTANCE_HEADER) else {
        return Ok(None);
    };
    let text = value
        .to_str()
        .map_err(|_| refuse("bad_instance", "an instance id is not a UUID"))?;
    instance(text).map(Some)
}

fn conflict_for(ctx: &Ctx) -> Conflict {
    let policy = PutAsset::hosted(ctx, "").write_policy;
    match (policy, matches!(ctx.grant(View::Edit), Grant::Whole)) {
        (Some(WritePolicy::Allow), true) => Conflict::Replace,
        _ => Conflict::Fail,
    }
}

#[derive(FromQueryResult)]
struct Stored {
    blob_store: String,
    blob_id: Uuid,
    sha1: String,
    sha256: String,
    sha512: String,
    md5: String,
}

struct StagedSource {
    content: Content,
    repository: String,
    lacking: Mutex<Vec<String>>,
}

impl StagedSource {
    fn new(content: Content, repository: &str) -> Self {
        Self {
            content,
            repository: repository.to_owned(),
            lacking: Mutex::new(Vec::new()),
        }
    }

    fn lack(&self, sha256: &str) -> ContentError {
        self.lacking
            .lock()
            .expect("the lacking lock is never poisoned")
            .push(sha256.to_owned());
        ContentError::Conflict(format!("the blob {sha256} is not on this server"))
    }

    fn lacked(&self) -> Vec<String> {
        self.lacking
            .lock()
            .expect("the lacking lock is never poisoned")
            .clone()
    }

    async fn find(&self, sql: &str, sha256: &str) -> Result<Option<Stored>, ContentError> {
        Ok(Stored::find_by_statement(statement(
            sql,
            vec![self.repository.clone().into(), sha256.into()],
        ))
        .one(self.content.db())
        .await?)
    }
}

#[async_trait]
impl BlobSource for StagedSource {
    async fn write(&self, sink: &Sink, asset: &BundleAsset) -> Result<Written, ContentError> {
        let sha256 = asset.sha256.to_ascii_lowercase();
        let found = match self.find(ASSET_BLOB, &sha256).await? {
            Some(found) => found,
            None => self
                .find(STAGED_BLOB, &sha256)
                .await?
                .ok_or_else(|| self.lack(&sha256))?,
        };
        let digests = Digests {
            sha1: found.sha1,
            sha256: found.sha256,
            sha512: found.sha512,
            md5: found.md5,
        };
        let id = BlobId::from(found.blob_id);
        if found.blob_store == sink.name {
            return match sink.store.duplicate(id, &digests).await {
                Ok(written) => Ok(written),
                Err(astrum_blob::Error::Missing { .. }) => Err(self.lack(&sha256)),
                Err(error) => Err(error.into()),
            };
        }
        let origin = self.content.blob_stores().get(&found.blob_store).await?;
        let content = origin.read(id).await?.ok_or_else(|| self.lack(&sha256))?;
        Ok(sink.store.put(content.stream).await?)
    }
}

async fn release_staged(ctx: &Ctx, digests: &[String]) {
    let released = async {
        let txn = ctx.db().begin().await?;
        let freed: Vec<(String, BlobId)> = Stale::find_by_statement(statement(
            RELEASE_STAGED,
            vec![ctx.name().into(), digests.to_vec().into()],
        ))
        .all(&txn)
        .await?
        .into_iter()
        .map(|row| (row.blob_store, row.blob_id.into()))
        .collect();
        holds::lapse(&txn, &freed).await?;
        txn.commit().await?;
        Ok::<(), ContentError>(())
    }
    .await;
    if let Err(error) = released {
        tracing::warn!(%error, repository = ctx.name(), "staged blobs of a committed component stay until they expire");
    }
}

fn check_bundle_shape(bundle: &Bundle) -> Result<(), Refusal> {
    if bundle.assets.is_empty() || bundle.assets.len() > MAX_ASSETS {
        return Err(refuse(
            "bad_bundle",
            format!("a bundle holds between 1 and {MAX_ASSETS} assets"),
        ));
    }
    match bundle
        .assets
        .iter()
        .position(|asset| !valid_sha256(&asset.sha256))
    {
        Some(position) => Err(refuse(
            "bad_digest",
            format!(
                "the SHA-256 of the asset at position {position} is not 64 lowercase hexadecimal characters"
            ),
        )),
        None => Ok(()),
    }
}

async fn commit(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    headers: HeaderMap,
    Json(mut bundle): Json<Bundle>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller.clone(), &repository, View::Add).await?;
    hosted(&ctx)?;
    let format = format_of(&state, &ctx)?;
    check_bundle_shape(&bundle)?;
    let origin = match (bundle.origin.as_deref(), sender_of(&headers)?) {
        (Some(origin), _) => instance(origin)?,
        (None, Some(sender)) => sender,
        (None, None) => {
            return Err(refuse(
                "origin_missing",
                "the bundle names no origin and the request names no sender instance",
            ));
        }
    };
    bundle.origin = Some(origin.clone());
    let digests: Vec<String> = bundle
        .assets
        .iter()
        .map(|asset| asset.sha256.clone())
        .collect();
    let lacking = absent(&state, ctx.name(), digests.clone()).await?;
    if !lacking.is_empty() {
        return Err(Refusal::Missing(lacking));
    }
    let conflict = conflict_for(&ctx);
    let source = Arc::new(StagedSource::new(state.content().clone(), ctx.name()));
    let task = {
        let (ctx, bundle, source) = (ctx.clone(), bundle.clone(), source.clone());
        tokio::spawn(async move {
            let transfer = transfer_of(&format)?;
            let terms = Terms::new(conflict, REPLICATED);
            let outcome = transfer
                .receive(&ctx, &bundle, source.as_ref(), terms)
                .await?;
            release_staged(&ctx, &digests).await;
            Ok::<Outcome, ContentError>(outcome)
        })
    };
    let outcome = match task.await.map_err(ApiError::internal)? {
        Ok(outcome) => outcome,
        Err(error) => {
            let lacked = source.lacked();
            return Err(match lacked.is_empty() {
                true => error.into(),
                false => Refusal::Missing(lacked),
            });
        }
    };
    if outcome.status == Status::Copied {
        let event = Event::new(AUDIT_DOMAIN, "replicated", ctx.name()).with(json!({
            "repository": ctx.name(),
            "component": component_key(&bundle.namespace, &bundle.name, &bundle.version),
            "assets": outcome.assets.len(),
            "bytes": outcome.bytes_copied,
            "origin": origin,
            "via": REPLICATION_VIA,
        }));
        state.audit(&caller, event).await;
    }
    Ok(axum::Json(outcome).into_response())
}

#[derive(FromQueryResult)]
struct Locked {
    id: i64,
    attributes: serde_json::Value,
}

async fn tombstone(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    Json(request): Json<Tombstone>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller.clone(), &repository, View::Delete).await?;
    hosted(&ctx)?;
    let format = format_of(&state, &ctx)?;
    let origin = instance(&request.origin)?;
    if !valid_sha256(&request.fingerprint) {
        return Err(refuse(
            "bad_digest",
            "the fingerprint is not 64 lowercase hexadecimal characters",
        ));
    }
    let txn = ctx.db().begin().await?;
    txn.query_one(statement(
        SET_CONTEXT,
        vec![REPLICATED.into(), ctx.uploader().into()],
    ))
    .await?;
    let locked = Locked::find_by_statement(statement(
        LOCK_COMPONENT,
        vec![
            ctx.name().into(),
            request.namespace.clone().into(),
            request.name.clone().into(),
            request.version.clone().into(),
        ],
    ))
    .one(&txn)
    .await?
    .ok_or_else(|| {
        Refusal::Status(
            StatusCode::NOT_FOUND,
            "component_not_found",
            "the component is not in the repository".to_owned(),
        )
    })?;
    if origin_of(&locked.attributes) != Some(origin.as_str()) {
        return Err(conflict(
            "origin_mismatch",
            "the component was not replicated from the sender, so a tombstone does not apply to it",
        ));
    }
    let found = store::component_by_id(ctx.db(), locked.id)
        .await?
        .ok_or_else(|| internal("a locked component vanished"))?;
    let bundle = transfer_of(&format)?.bundle(&ctx, &found).await?;
    if fingerprint(&bundle) != request.fingerprint {
        return Err(conflict(
            "fingerprint_mismatch",
            "the component changed after the sender last saw it",
        ));
    }
    let blobs = Stale::find_by_statement(statement(COMPONENT_BLOBS, vec![locked.id.into()]))
        .all(&txn)
        .await?;
    txn.execute(statement(DELETE_COMPONENT, vec![locked.id.into()]))
        .await?;
    txn.commit().await?;
    for blob in blobs {
        store::discard(ctx.content.clone(), &blob.blob_store, blob.blob_id);
    }
    let event = Event::new(AUDIT_DOMAIN, "deleted", ctx.name()).with(json!({
        "repository": ctx.name(),
        "component": component_key(&request.namespace, &request.name, &request.version),
        "origin": origin,
        "via": REPLICATION_VIA,
    }));
    state.audit(&caller, event).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

fn key_failure(position: usize) -> Refusal {
    refuse(
        "bad_key",
        format!("the key at position {position} is not a component key"),
    )
}

async fn compare(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    Json(request): Json<InventoryCompare>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller, &repository, View::Read).await?;
    let format = format_of(&state, &ctx)?;
    if request.items.len() > MAX_COMPARE {
        return Err(ApiError::invalid(
            "items",
            format!(
                "must name at most {MAX_COMPARE} items, not {}",
                request.items.len()
            ),
        )
        .into());
    }
    let mut parsed = Vec::with_capacity(request.items.len());
    for (position, item) in request.items.iter().enumerate() {
        let key = parse_component_key(&item.key).ok_or_else(|| key_failure(position))?;
        if !valid_sha256(&item.fingerprint) {
            return Err(refuse(
                "bad_digest",
                format!(
                    "the fingerprint at position {position} is not 64 lowercase hexadecimal characters"
                ),
            ));
        }
        parsed.push(key);
    }
    let column = |pick: fn(&(String, String, String)) -> &String| -> Vec<String> {
        parsed.iter().map(|key| pick(key).clone()).collect()
    };
    let stored = component::Model::find_by_statement(statement(
        COMPONENTS_BY_KEY,
        vec![
            ctx.name().into(),
            column(|key| &key.0).into(),
            column(|key| &key.1).into(),
            column(|key| &key.2).into(),
        ],
    ))
    .all(state.db().conn())
    .await?;
    let known: HashMap<(String, String, String), component::Model> = stored
        .into_iter()
        .map(|found| {
            (
                (
                    found.namespace.clone(),
                    found.name.clone(),
                    found.version.clone(),
                ),
                found,
            )
        })
        .collect();
    let transfer = transfer_of(&format)?;
    let mut seen = HashSet::new();
    let mut differing = Vec::new();
    for (item, key) in request.items.iter().zip(parsed) {
        let same = match known.get(&key) {
            Some(found) => {
                let bundle = transfer.bundle(&ctx, found).await?;
                fingerprint(&bundle) == item.fingerprint
            }
            None => false,
        };
        if !same && seen.insert(item.key.clone()) {
            differing.push(item.key.clone());
        }
    }
    Ok(axum::Json(InventoryCompareReply { differing }).into_response())
}

#[derive(Deserialize)]
struct Paging {
    after: Option<String>,
    limit: Option<u32>,
}

#[derive(FromQueryResult)]
struct Keyed {
    id: i64,
    item_key: String,
}

fn keyed_listing() -> String {
    let escaped = |column: &str| ESCAPED.replace("{}", column);
    let key = format!(
        "((CASE WHEN c.namespace = '' THEN '' ELSE {} || '/' END) || {} || '@' || {}) COLLATE \"C\"",
        escaped("c.namespace"),
        escaped("c.name"),
        escaped("c.version"),
    );
    format!(
        "SELECT c.id, {key} AS item_key FROM component c \
         WHERE c.repository = $1 AND ($2::text IS NULL OR {key} > $2) \
         ORDER BY item_key LIMIT $3"
    )
}

async fn inventory(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    paging: Result<Query<Paging>, QueryRejection>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller, &repository, View::Read).await?;
    let format = format_of(&state, &ctx)?;
    let Query(paging) = paging.map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
    let limit = paging.limit.unwrap_or(DEFAULT_PAGE);
    if limit == 0 || limit > MAX_PAGE {
        return Err(ApiError::invalid("limit", format!("must be between 1 and {MAX_PAGE}")).into());
    }
    if let Some(after) = &paging.after
        && parse_component_key(after).is_none()
    {
        return Err(refuse("bad_key", "after is not a component key"));
    }
    let mut rows = Keyed::find_by_statement(statement(
        &keyed_listing(),
        vec![
            ctx.name().into(),
            paging.after.into(),
            (i64::from(limit) + 1).into(),
        ],
    ))
    .all(state.db().conn())
    .await?;
    let more = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    let components: HashMap<i64, component::Model> = component::Entity::find()
        .filter(component::Column::Id.is_in(rows.iter().map(|row| row.id)))
        .all(state.db().conn())
        .await?
        .into_iter()
        .map(|found| (found.id, found))
        .collect();
    let transfer = transfer_of(&format)?;
    let next = match more {
        true => rows.last().map(|row| row.item_key.clone()),
        false => None,
    };
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let Some(found) = components.get(&row.id) else {
            continue;
        };
        let bundle = transfer.bundle(&ctx, found).await?;
        items.push(InventoryEntry {
            key: row.item_key,
            fingerprint: fingerprint(&bundle),
            origin: origin_of(&found.attributes).map(str::to_owned),
        });
    }
    Ok(axum::Json(InventoryPage { items, next }).into_response())
}
