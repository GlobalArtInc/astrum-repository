use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::atomic::Ordering;

use astrum_content::replicated::{self, Committed, Reason, Refusal as Refused, Removal, instance};
use astrum_content::staging::{self, Counted, MAX_BLOB_BYTES, Staged};
use astrum_content::store;
use astrum_content::transfer::{Bundle, REPLICATION_VIA, Status, fingerprint, origin_of};
use astrum_content::{ContentError, Ctx, Grant, View};
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
use axum::Router;
use axum::body::Body;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{ApiError, AppState, Caller, Json};

pub(super) const RECEIVE: &str = "astrum:replication:receive";
pub(super) const PULL: &str = "astrum:replication:pull";
const INSTANCE_KEY: &str = "replication.instanceId";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUDIT_DOMAIN: &str = "repository.content";

const RECORD_INSTANCE: &str = "INSERT INTO setting (key, value, updated_at) \
     VALUES ($1, to_jsonb($2::text), now()) ON CONFLICT (key) DO NOTHING";

const COMPONENTS_BY_KEY: &str = "SELECT c.* FROM component c \
     JOIN unnest($2::text[], $3::text[], $4::text[]) AS k(namespace, name, version) \
     ON c.namespace = k.namespace AND c.name = k.name AND c.version = k.version \
     WHERE c.repository = $1";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(CAPABILITIES, get(capabilities))
        .route(MISSING, post(missing_blobs))
        .route(BLOB, put(put_blob))
        .route(COMPONENTS, post(commit).delete(tombstone))
        .route(INVENTORY_COMPARE, post(compare))
        .route(INVENTORY, get(inventory))
}

pub(super) enum Refusal {
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

fn status_of(reason: Reason) -> StatusCode {
    match reason {
        Reason::BadRequest => StatusCode::BAD_REQUEST,
        Reason::Conflict => StatusCode::CONFLICT,
        Reason::NotFound => StatusCode::NOT_FOUND,
        Reason::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        Reason::TooMany => StatusCode::TOO_MANY_REQUESTS,
        Reason::Unprocessable => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

impl From<Refused> for Refusal {
    fn from(refused: Refused) -> Self {
        match refused {
            Refused::Content(error) => error.into(),
            Refused::Status {
                reason,
                code,
                message,
            } => Refusal::Status(status_of(reason), code, message),
            Refused::Missing(missing) => Refusal::Missing(missing),
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

pub(super) fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

fn internal(what: &str) -> ApiError {
    ApiError::internal(anyhow::anyhow!("{what}"))
}

pub(super) async fn instance_id(state: &AppState) -> Result<Uuid, ApiError> {
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

pub(super) fn require_any(caller: &Caller, permissions: &[&str]) -> Result<(), ApiError> {
    match caller.subject() {
        Some(subject)
            if permissions
                .iter()
                .any(|wanted| subject.is_permitted(wanted)) =>
        {
            Ok(())
        }
        Some(subject) if !subject.anonymous => Err(ApiError::forbidden()),
        _ => Err(ApiError::Unauthorized),
    }
}

async fn capabilities(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Capabilities>, ApiError> {
    require_any(&caller, &[RECEIVE, PULL])?;
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

async fn open(state: &AppState, caller: Caller, name: &str, view: View) -> Result<Ctx, Refusal> {
    open_as(state, caller, name, view, &[RECEIVE]).await
}

pub(super) async fn open_as(
    state: &AppState,
    caller: Caller,
    name: &str,
    view: View,
    privileges: &[&str],
) -> Result<Ctx, Refusal> {
    require_any(&caller, privileges)?;
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
    let missing = staging::absent(state.db().conn(), ctx.name(), request.sha256).await?;
    Ok(axum::Json(MissingReply { missing }).into_response())
}

async fn put_blob(
    State(state): State<AppState>,
    caller: Caller,
    Path((repository, sha256)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Response {
    match receive_blob(&state, caller, &repository, &sha256, &headers, body).await {
        Ok(status) => status.into_response(),
        Err(refusal) => refusal.into_response(),
    }
}

fn too_large(limit: u64) -> Refusal {
    Refusal::Status(
        StatusCode::PAYLOAD_TOO_LARGE,
        "too_large",
        format!("a replicated blob may hold at most {limit} bytes"),
    )
}

async fn receive_blob(
    state: &AppState,
    caller: Caller,
    repository: &str,
    sha256: &str,
    headers: &HeaderMap,
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
    replicated::format_of(&ctx)?;
    if replicated::read_only(&ctx) {
        return Err(refuse(
            "read_only",
            format!("the repository {repository} is read-only"),
        ));
    }
    let declared = headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if declared.is_some_and(|length| length > MAX_BLOB_BYTES) {
        return Err(too_large(MAX_BLOB_BYTES));
    }
    staging::staged_room(ctx.db(), ctx.name()).await?;
    if !valid_sha256(sha256) {
        return Err(refuse(
            "bad_digest",
            "the digest in the path is not 64 lowercase hexadecimal characters",
        ));
    }
    let (counted, exceeded) = Counted::new(
        body.into_data_stream().map_err(io::Error::other).boxed(),
        MAX_BLOB_BYTES,
    );
    let written = match store::write_blob(&ctx, counted.boxed()).await {
        Ok(written) => written,
        Err(_) if exceeded.load(Ordering::Relaxed) => return Err(too_large(MAX_BLOB_BYTES)),
        Err(error) => return Err(error.into()),
    };
    if written.digests.sha256 != sha256 {
        staging::discard(state.content(), &written).await;
        return Err(Refusal::Status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "digest_mismatch",
            format!(
                "the body has the SHA-256 {}, not the {sha256} of the path",
                written.digests.sha256
            ),
        ));
    }
    let staged = match staging::stage(state.content(), ctx.name(), &written, &uploaded_by).await {
        Ok(staged) => staged,
        Err(refusal) => {
            staging::discard(state.content(), &written).await;
            return Err(refusal.into());
        }
    };
    if staged == Staged::Present {
        staging::discard(state.content(), &written).await;
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

fn sender_of(headers: &HeaderMap) -> Result<Option<String>, Refusal> {
    let Some(value) = headers.get(INSTANCE_HEADER) else {
        return Ok(None);
    };
    let text = value
        .to_str()
        .map_err(|_| refuse("bad_instance", "an instance id is not a UUID"))?;
    Ok(Some(instance(text)?))
}

async fn commit(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    headers: HeaderMap,
    Json(bundle): Json<Bundle>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller.clone(), &repository, View::Add).await?;
    let sender = sender_of(&headers)?;
    let conflict = replicated::conflict_for(&ctx, matches!(ctx.grant(View::Edit), Grant::Whole));
    let component = component_key(&bundle.namespace, &bundle.name, &bundle.version);
    let Committed { outcome, origin } = replicated::commit(&ctx, bundle, sender, conflict).await?;
    if outcome.status == Status::Copied {
        let event = Event::new(AUDIT_DOMAIN, "replicated", ctx.name()).with(json!({
            "repository": ctx.name(),
            "component": component,
            "assets": outcome.assets.len(),
            "bytes": outcome.bytes_copied,
            "origin": origin,
            "via": REPLICATION_VIA,
        }));
        state.audit(&caller, event).await;
    }
    Ok(axum::Json(outcome).into_response())
}

async fn tombstone(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    Json(request): Json<Tombstone>,
) -> Result<Response, Refusal> {
    let ctx = open(&state, caller.clone(), &repository, View::Delete).await?;
    let origin = replicated::delete(
        &ctx,
        &Removal {
            namespace: &request.namespace,
            name: &request.name,
            version: &request.version,
            fingerprint: &request.fingerprint,
            origin: &request.origin,
        },
    )
    .await?;
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
    let ctx = open_as(&state, caller, &repository, View::Read, &[RECEIVE, PULL]).await?;
    let format = replicated::format_of(&ctx)?;
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
    let transfer = replicated::transfer_of(&format)?;
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

const LISTING_FIRST: &str = "SELECT c.* FROM component c WHERE c.repository = $1 \
     ORDER BY c.namespace COLLATE \"C\", c.name COLLATE \"C\", c.version COLLATE \"C\" LIMIT $2";

const LISTING_AFTER: &str = "SELECT c.* FROM component c WHERE c.repository = $1 \
     AND (c.namespace COLLATE \"C\", c.name COLLATE \"C\", c.version COLLATE \"C\") \
         > ($2::text COLLATE \"C\", $3::text COLLATE \"C\", $4::text COLLATE \"C\") \
     ORDER BY c.namespace COLLATE \"C\", c.name COLLATE \"C\", c.version COLLATE \"C\" LIMIT $5";

async fn inventory(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    paging: Result<Query<Paging>, QueryRejection>,
) -> Result<Response, Refusal> {
    let ctx = open_as(&state, caller, &repository, View::Read, &[RECEIVE, PULL]).await?;
    let format = replicated::format_of(&ctx)?;
    let Query(paging) = paging.map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
    let limit = paging.limit.unwrap_or(DEFAULT_PAGE);
    if limit == 0 || limit > MAX_PAGE {
        return Err(ApiError::invalid("limit", format!("must be between 1 and {MAX_PAGE}")).into());
    }
    let ahead = i64::from(limit) + 1;
    let listing = match paging.after.as_deref() {
        None => statement(LISTING_FIRST, vec![ctx.name().into(), ahead.into()]),
        Some(after) => {
            let (namespace, name, version) = parse_component_key(after)
                .ok_or_else(|| refuse("bad_key", "after is not a component key"))?;
            statement(
                LISTING_AFTER,
                vec![
                    ctx.name().into(),
                    namespace.into(),
                    name.into(),
                    version.into(),
                    ahead.into(),
                ],
            )
        }
    };
    let mut rows = component::Model::find_by_statement(listing)
        .all(state.db().conn())
        .await?;
    let more = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    let transfer = replicated::transfer_of(&format)?;
    let mut items = Vec::with_capacity(rows.len());
    for found in &rows {
        let bundle = transfer.bundle(&ctx, found).await?;
        items.push(InventoryEntry {
            key: component_key(&found.namespace, &found.name, &found.version),
            fingerprint: fingerprint(&bundle),
            origin: origin_of(&found.attributes).map(str::to_owned),
        });
    }
    let next = match more {
        true => items.last().map(|item| item.key.clone()),
        false => None,
    };
    Ok(axum::Json(InventoryPage { items, next }).into_response())
}
