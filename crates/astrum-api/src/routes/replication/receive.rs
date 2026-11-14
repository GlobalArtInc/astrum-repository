use std::collections::HashSet;
use std::io;
use std::time::Duration;

use astrum_blob::{BlobId, ContentStream, Written};
use astrum_content::{Content, ContentError, Grant, View, holds, store};
use astrum_db::setting;
use astrum_replication::protocol::{
    BLOB, CAPABILITIES, Capabilities, MAX_MISSING, MISSING, MissingReply, MissingRequest, PROTOCOL,
    valid_sha256,
};
use astrum_repo::RepositoryType;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, TransactionTrait, Value};
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

const STAGE: &str = "INSERT INTO replication_staged_blob \
     (repository, sha256, blob_store, blob_id, size, sha1, sha512, md5, uploaded_by, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now() + make_interval(hours => $10)) \
     ON CONFLICT (repository, sha256) DO NOTHING";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(CAPABILITIES, get(capabilities))
        .route(MISSING, post(missing_blobs))
        .route(BLOB, put(put_blob))
}

enum Refusal {
    Api(ApiError),
    Status(StatusCode, &'static str, String),
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
    let present: HashSet<String> = Digest::find_by_statement(statement(
        PRESENT,
        vec![ctx.name().into(), request.sha256.clone().into()],
    ))
    .all(state.db().conn())
    .await?
    .into_iter()
    .map(|row| row.sha256)
    .collect();
    let mut seen = HashSet::new();
    let missing = request
        .sha256
        .into_iter()
        .filter(|digest| !present.contains(digest) && seen.insert(digest.clone()))
        .collect();
    Ok(axum::Json(MissingReply { missing }).into_response())
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
