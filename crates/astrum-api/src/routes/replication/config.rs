use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_security::Subject;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection, refused_remote_destination};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{DbBackend, DbErr, FromQueryResult, SqlErr, Statement, Value};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::validate;
use crate::{ApiError, AppState, Caller, Json, Validation};

const REMOTES: &str = "/service/rest/v1/replication/remotes";
const REPLICATIONS: &str = "/service/rest/v1/replications";
const READ: &str = "astrum:replication:read";
const CREATE: &str = "astrum:replication:create";
pub(super) const UPDATE: &str = "astrum:replication:update";
const DELETE: &str = "astrum:replication:delete";
pub(super) const DOMAIN: &str = "replication";

const DEFAULT_CONCURRENCY: i64 = 2;
const DEFAULT_SETTLE_SECONDS: i64 = 30;
const DEFAULT_RECONCILE_MINUTES: i64 = 360;
const MAX_CONCURRENCY: i64 = 16;
const MAX_SETTLE_SECONDS: i64 = 3600;
const MIN_RECONCILE_MINUTES: i64 = 5;

const REMOTE_COLUMNS: &str = "id, name, url, true AS has_token, use_trust_store, \
     allow_private_network, max_concurrency, max_bytes_per_second, remote_instance_id, \
     remote_version, checked_at, created_at, updated_at";

const REPLICATION_COLUMNS: &str = "id, name, source_repository, remote_id, remote_repository, \
     enabled, name_filter, version_filter, propagate_deletes, settle_seconds, \
     reconcile_minutes, alert_channels, alert_state, last_success_at, last_failure_at, \
     last_error, last_reconcile_at, created_at, updated_at";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(REMOTES, get(list_remotes).post(create_remote))
        .route(
            &format!("{REMOTES}/{{id}}"),
            get(read_remote).put(update_remote).delete(remove_remote),
        )
        .route(
            REPLICATIONS,
            get(list_replications).post(create_replication),
        )
        .route(
            &format!("{REPLICATIONS}/{{id}}"),
            get(read_replication)
                .put(update_replication)
                .delete(remove_replication),
        )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteRequest {
    name: Option<String>,
    url: Option<String>,
    token: Option<String>,
    use_trust_store: Option<bool>,
    allow_private_network: Option<bool>,
    max_concurrency: Option<i64>,
    max_bytes_per_second: Option<i64>,
}

#[derive(Serialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
struct RemoteModel {
    id: Uuid,
    name: String,
    url: String,
    has_token: bool,
    use_trust_store: bool,
    allow_private_network: bool,
    max_concurrency: i32,
    max_bytes_per_second: Option<i64>,
    remote_instance_id: Option<Uuid>,
    remote_version: Option<String>,
    checked_at: Option<DateTimeWithTimeZone>,
    created_at: DateTimeWithTimeZone,
    updated_at: DateTimeWithTimeZone,
}

struct RemoteInput {
    name: String,
    url: String,
    token: Option<String>,
    use_trust_store: bool,
    allow_private_network: bool,
    max_concurrency: i32,
    max_bytes_per_second: Option<i64>,
    parsed: url::Url,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplicationRequest {
    name: Option<String>,
    source_repository: Option<String>,
    remote_id: Option<String>,
    remote_repository: Option<String>,
    enabled: Option<bool>,
    name_filter: Option<String>,
    version_filter: Option<String>,
    propagate_deletes: Option<bool>,
    settle_seconds: Option<i64>,
    reconcile_minutes: Option<i64>,
    alert_channels: Option<Vec<String>>,
}

#[derive(Serialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
struct ReplicationModel {
    id: Uuid,
    name: String,
    source_repository: String,
    remote_id: Uuid,
    remote_repository: String,
    enabled: bool,
    name_filter: Option<String>,
    version_filter: Option<String>,
    propagate_deletes: bool,
    settle_seconds: i32,
    reconcile_minutes: i32,
    alert_channels: Vec<Uuid>,
    alert_state: String,
    last_success_at: Option<DateTimeWithTimeZone>,
    last_failure_at: Option<DateTimeWithTimeZone>,
    last_error: Option<String>,
    last_reconcile_at: Option<DateTimeWithTimeZone>,
    created_at: DateTimeWithTimeZone,
    updated_at: DateTimeWithTimeZone,
}

struct ReplicationInput {
    name: String,
    source_repository: String,
    remote_id: Uuid,
    remote_repository: String,
    enabled: bool,
    name_filter: Option<String>,
    version_filter: Option<String>,
    propagate_deletes: bool,
    settle_seconds: i32,
    reconcile_minutes: i32,
    alert_channels: Vec<Uuid>,
}

#[derive(FromQueryResult)]
struct Named {
    name: String,
}

#[derive(FromQueryResult)]
pub(super) struct Found {
    id: Uuid,
}

#[derive(FromQueryResult)]
struct SourceRepository {
    format: String,
    kind: String,
}

pub(super) fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

fn reply(status: StatusCode, body: serde_json::Value) -> Response {
    (status, axum::Json(body)).into_response()
}

pub(super) fn message(status: StatusCode, text: impl Into<String>) -> Response {
    reply(status, json!({"id": "*", "message": text.into()}))
}

pub(super) fn parse_id(id: &str, what: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id)
        .map_err(|_| ApiError::not_found(format!("There is no {what} with id '{id}'")))
}

pub(super) fn missing(what: &str, id: Uuid) -> ApiError {
    ApiError::not_found(format!("There is no {what} with id '{id}'"))
}

fn constraint_violated(error: &DbErr, name: &str) -> bool {
    match error.sql_err() {
        Some(
            SqlErr::UniqueConstraintViolation(text) | SqlErr::ForeignKeyConstraintViolation(text),
        ) => text.contains(name),
        _ => false,
    }
}

fn is_unique(error: &DbErr) -> bool {
    matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}

fn is_foreign_key(error: &DbErr) -> bool {
    matches!(
        error.sql_err(),
        Some(SqlErr::ForeignKeyConstraintViolation(_))
    )
}

pub(super) async fn ssrf_settings(state: &AppState) -> Result<SsrfProtection, ApiError> {
    Ok(setting::get(state.db().conn(), SSRF_KEY)
        .await?
        .unwrap_or_default())
}

fn remote_input(body: RemoteRequest, validation: &mut Validation) -> Option<RemoteInput> {
    let name = validate::name(body.name.as_deref())
        .map_err(|error| validation.add("name", error))
        .ok();
    let allow_private_network = body.allow_private_network.unwrap_or(false);
    let parsed = validate::remote_url(body.url.as_deref(), allow_private_network)
        .map_err(|error| validation.add("url", error))
        .ok();
    let token = match body.token {
        Some(token) => match validate::token(&token) {
            Ok(()) => Some(token),
            Err(error) => {
                validation.add("token", error);
                None
            }
        },
        None => None,
    };
    let max_concurrency = body.max_concurrency.unwrap_or(DEFAULT_CONCURRENCY);
    if !(1..=MAX_CONCURRENCY).contains(&max_concurrency) {
        validation.add(
            "maxConcurrency",
            format!("must be between 1 and {MAX_CONCURRENCY}"),
        );
    }
    if body.max_bytes_per_second.is_some_and(|limit| limit <= 0) {
        validation.add("maxBytesPerSecond", "must be above 0, or null for no limit");
    }
    let parsed = parsed?;
    Some(RemoteInput {
        name: name?,
        url: validate::normalized(&parsed),
        token,
        use_trust_store: body.use_trust_store.unwrap_or(false),
        allow_private_network,
        max_concurrency: max_concurrency as i32,
        max_bytes_per_second: body.max_bytes_per_second,
        parsed,
    })
}

async fn check_destination(state: &AppState, input: &RemoteInput) -> Result<(), ApiError> {
    let settings = ssrf_settings(state).await?;
    match refused_remote_destination(&settings, &input.parsed, input.allow_private_network).await {
        Some(text) => Err(ApiError::invalid("url", text)),
        None => Ok(()),
    }
}

async fn list_remotes(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<RemoteModel>>, ApiError> {
    caller.require(READ)?;
    let sql = format!("SELECT {REMOTE_COLUMNS} FROM replication_remote ORDER BY lower(name), id");
    Ok(axum::Json(
        RemoteModel::find_by_statement(statement(&sql, Vec::new()))
            .all(state.db().conn())
            .await?,
    ))
}

async fn find_remote(state: &AppState, id: Uuid) -> Result<Option<RemoteModel>, ApiError> {
    let sql = format!("SELECT {REMOTE_COLUMNS} FROM replication_remote WHERE id = $1");
    Ok(
        RemoteModel::find_by_statement(statement(&sql, vec![id.into()]))
            .one(state.db().conn())
            .await?,
    )
}

async fn read_remote(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<axum::Json<RemoteModel>, ApiError> {
    caller.require(READ)?;
    let id = parse_id(&id, "replication remote")?;
    find_remote(&state, id)
        .await?
        .map(axum::Json)
        .ok_or_else(|| missing("replication remote", id))
}

fn remote_conflict(error: &DbErr) -> Option<Response> {
    constraint_violated(error, "replication_remote_name_lower").then(|| {
        message(
            StatusCode::CONFLICT,
            "A replication remote with this name exists already",
        )
    })
}

async fn create_remote(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<RemoteRequest>, ApiError>,
) -> Result<Response, ApiError> {
    caller.require(CREATE)?;
    let Json(body) = body?;
    let mut validation = Validation::new();
    let supplied = body.token.is_some();
    let input = remote_input(body, &mut validation);
    if !supplied {
        validation.add("token", "must not be empty");
    }
    validation.finish()?;
    let input = input.ok_or_else(|| ApiError::internal(anyhow::anyhow!("remote input missing")))?;
    let token = input
        .token
        .as_deref()
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("remote token missing")))?;
    check_destination(&state, &input).await?;
    let id = Uuid::new_v4();
    let sql = format!(
        "INSERT INTO replication_remote (id, name, url, token, use_trust_store, \
         allow_private_network, max_concurrency, max_bytes_per_second) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING {REMOTE_COLUMNS}"
    );
    let inserted = RemoteModel::find_by_statement(statement(
        &sql,
        vec![
            id.into(),
            input.name.clone().into(),
            input.url.clone().into(),
            state.secrets().seal(token).into(),
            input.use_trust_store.into(),
            input.allow_private_network.into(),
            input.max_concurrency.into(),
            input.max_bytes_per_second.into(),
        ],
    ))
    .one(state.db().conn())
    .await;
    let created = match inserted {
        Ok(Some(created)) => created,
        Ok(None) => {
            return Err(ApiError::internal(anyhow::anyhow!(
                "the remote was not returned after it was inserted"
            )));
        }
        Err(error) => {
            return match remote_conflict(&error) {
                Some(response) => Ok(response),
                None => Err(error.into()),
            };
        }
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "remote.created", &created.name).with(json!({
                "id": created.id,
                "name": created.name,
                "url": created.url,
                "useTrustStore": created.use_trust_store,
                "allowPrivateNetwork": created.allow_private_network,
                "maxConcurrency": created.max_concurrency,
                "maxBytesPerSecond": created.max_bytes_per_second,
                "tokenChanged": true,
            })),
        )
        .await;
    Ok((StatusCode::CREATED, axum::Json(created)).into_response())
}

async fn update_remote(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    body: Result<Json<RemoteRequest>, ApiError>,
) -> Result<Response, ApiError> {
    caller.require(UPDATE)?;
    let id = parse_id(&id, "replication remote")?;
    let Json(body) = body?;
    let mut validation = Validation::new();
    let input = remote_input(body, &mut validation);
    validation.finish()?;
    let input = input.ok_or_else(|| ApiError::internal(anyhow::anyhow!("remote input missing")))?;
    let current = find_remote(&state, id)
        .await?
        .ok_or_else(|| missing("replication remote", id))?;
    let address_changed = current.url != input.url;
    if address_changed && input.token.is_none() {
        return Err(ApiError::invalid(
            "token",
            "must be given again when the url changes",
        ));
    }
    if address_changed || current.allow_private_network != input.allow_private_network {
        check_destination(&state, &input).await?;
    }
    let sealed = input
        .token
        .as_deref()
        .map(|token| state.secrets().seal(token));
    let token_changed = sealed.is_some();
    let sql = format!(
        "UPDATE replication_remote SET name = $2, url = $3, token = COALESCE($4, token), \
         use_trust_store = $5, allow_private_network = $6, max_concurrency = $7, \
         max_bytes_per_second = $8, updated_at = now(), \
         remote_instance_id = CASE WHEN url = $3 THEN remote_instance_id END, \
         remote_version = CASE WHEN url = $3 THEN remote_version END, \
         checked_at = CASE WHEN url = $3 THEN checked_at END \
         WHERE id = $1 AND ($4::text IS NOT NULL OR url = $3) RETURNING {REMOTE_COLUMNS}"
    );
    let updated = RemoteModel::find_by_statement(statement(
        &sql,
        vec![
            id.into(),
            input.name.clone().into(),
            input.url.clone().into(),
            sealed.into(),
            input.use_trust_store.into(),
            input.allow_private_network.into(),
            input.max_concurrency.into(),
            input.max_bytes_per_second.into(),
        ],
    ))
    .one(state.db().conn())
    .await;
    let updated = match updated {
        Ok(Some(updated)) => updated,
        Ok(None) => {
            return match find_remote(&state, id).await? {
                Some(_) => Err(ApiError::invalid(
                    "token",
                    "must be given again when the url changes",
                )),
                None => Err(missing("replication remote", id)),
            };
        }
        Err(error) => {
            return match remote_conflict(&error) {
                Some(response) => Ok(response),
                None => Err(error.into()),
            };
        }
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "remote.updated", &updated.name).with(json!({
                "id": updated.id,
                "name": updated.name,
                "previousName": current.name,
                "url": updated.url,
                "useTrustStore": updated.use_trust_store,
                "allowPrivateNetwork": updated.allow_private_network,
                "maxConcurrency": updated.max_concurrency,
                "maxBytesPerSecond": updated.max_bytes_per_second,
                "tokenChanged": token_changed,
            })),
        )
        .await;
    Ok(axum::Json(updated).into_response())
}

async fn remove_remote(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    caller.require(DELETE)?;
    let id = parse_id(&id, "replication remote")?;
    let deleted = Named::find_by_statement(statement(
        "DELETE FROM replication_remote WHERE id = $1 RETURNING name",
        vec![id.into()],
    ))
    .one(state.db().conn())
    .await;
    match deleted {
        Ok(Some(deleted)) => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "remote.deleted", &deleted.name)
                        .with(json!({"id": id, "name": deleted.name})),
                )
                .await;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Ok(None) => Err(missing("replication remote", id)),
        Err(error) => {
            let users = Named::find_by_statement(statement(
                "SELECT name FROM replication WHERE remote_id = $1 ORDER BY lower(name)",
                vec![id.into()],
            ))
            .all(state.db().conn())
            .await?;
            if users.is_empty() {
                return Err(error.into());
            }
            let names: Vec<String> = users.into_iter().map(|user| user.name).collect();
            Ok(message(
                StatusCode::CONFLICT,
                format!(
                    "The remote is used by these replications: {}",
                    names.join(", ")
                ),
            ))
        }
    }
}

async fn list_replications(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<ReplicationModel>>, ApiError> {
    caller.require(READ)?;
    let sql = format!("SELECT {REPLICATION_COLUMNS} FROM replication ORDER BY lower(name), id");
    Ok(axum::Json(
        ReplicationModel::find_by_statement(statement(&sql, Vec::new()))
            .all(state.db().conn())
            .await?,
    ))
}

async fn find_replication(
    state: &AppState,
    id: Uuid,
) -> Result<Option<ReplicationModel>, ApiError> {
    let sql = format!("SELECT {REPLICATION_COLUMNS} FROM replication WHERE id = $1");
    Ok(
        ReplicationModel::find_by_statement(statement(&sql, vec![id.into()]))
            .one(state.db().conn())
            .await?,
    )
}

async fn read_replication(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<axum::Json<ReplicationModel>, ApiError> {
    caller.require(READ)?;
    let id = parse_id(&id, "replication")?;
    find_replication(&state, id)
        .await?
        .map(axum::Json)
        .ok_or_else(|| missing("replication", id))
}

fn optional_filter(
    validation: &mut Validation,
    field: &str,
    value: Option<String>,
) -> Option<String> {
    let value = value.filter(|value| !value.trim().is_empty())?;
    match validate::filter(&value) {
        Ok(_) => Some(value),
        Err(error) => {
            validation.add(field, error);
            None
        }
    }
}

fn ranged(
    validation: &mut Validation,
    field: &str,
    value: Option<i64>,
    default: i64,
    low: i64,
    high: i64,
) -> i32 {
    let value = value.unwrap_or(default);
    if !(low..=high).contains(&value) {
        validation.add(field, format!("must be between {low} and {high}"));
        return default as i32;
    }
    value as i32
}

async fn replication_input(
    state: &AppState,
    subject: &Subject,
    body: ReplicationRequest,
) -> Result<ReplicationInput, ApiError> {
    let mut validation = Validation::new();
    let name = validate::name(body.name.as_deref())
        .map_err(|error| validation.add("name", error))
        .ok();
    let source = body
        .source_repository
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if source.is_empty() {
        validation.add("sourceRepository", "must not be empty");
    }
    let remote_id = match body
        .remote_id
        .as_deref()
        .map(str::trim)
        .map(Uuid::parse_str)
    {
        Some(Ok(id)) => Some(id),
        Some(Err(_)) => {
            validation.add("remoteId", "must be the id of a replication remote");
            None
        }
        None => {
            validation.add("remoteId", "must not be empty");
            None
        }
    };
    let remote_repository = validate::remote_repository(body.remote_repository.as_deref())
        .map_err(|error| validation.add("remoteRepository", error))
        .ok();
    let name_filter = optional_filter(&mut validation, "nameFilter", body.name_filter);
    let version_filter = optional_filter(&mut validation, "versionFilter", body.version_filter);
    let settle_seconds = ranged(
        &mut validation,
        "settleSeconds",
        body.settle_seconds,
        DEFAULT_SETTLE_SECONDS,
        0,
        MAX_SETTLE_SECONDS,
    );
    let reconcile_minutes = ranged(
        &mut validation,
        "reconcileMinutes",
        body.reconcile_minutes,
        DEFAULT_RECONCILE_MINUTES,
        MIN_RECONCILE_MINUTES,
        i32::MAX as i64,
    );
    let mut channels: Vec<Uuid> = Vec::new();
    for raw in body.alert_channels.unwrap_or_default() {
        match Uuid::parse_str(raw.trim()) {
            Ok(id) if !channels.contains(&id) => channels.push(id),
            Ok(_) => {}
            Err(_) => validation.add(
                "alertChannels",
                "must be a list of notification channel ids",
            ),
        }
    }
    validation.finish()?;
    let (Some(name), Some(remote_id), Some(remote_repository)) =
        (name, remote_id, remote_repository)
    else {
        return Err(ApiError::internal(anyhow::anyhow!(
            "replication input missing"
        )));
    };

    let remote = Found::find_by_statement(statement(
        "SELECT id FROM replication_remote WHERE id = $1",
        vec![remote_id.into()],
    ))
    .one(state.db().conn())
    .await?;
    if remote.is_none() {
        return Err(ApiError::invalid(
            "remoteId",
            "there is no replication remote with this id",
        ));
    }
    let repository = SourceRepository::find_by_statement(statement(
        "SELECT format, type AS kind FROM repository WHERE name = $1",
        vec![source.clone().into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| {
        ApiError::invalid("sourceRepository", "there is no repository with this name")
    })?;
    let administered = format!(
        "astrum:repository-admin:{}:{source}:read",
        repository.format
    );
    if !subject.is_permitted(&administered) {
        return Err(ApiError::forbidden());
    }
    if repository.kind != "hosted" {
        return Err(ApiError::invalid(
            "sourceRepository",
            "must be a hosted repository; proxy and group repositories cannot be replicated",
        ));
    }
    if !channels.is_empty() {
        let known = Found::find_by_statement(statement(
            "SELECT id FROM notification_channel WHERE id = ANY($1)",
            vec![channels.clone().into()],
        ))
        .all(state.db().conn())
        .await?;
        let unknown: Vec<String> = channels
            .iter()
            .filter(|id| !known.iter().any(|found| found.id == **id))
            .map(Uuid::to_string)
            .collect();
        if !unknown.is_empty() {
            return Err(ApiError::invalid(
                "alertChannels",
                format!("unknown notification channels: {}", unknown.join(", ")),
            ));
        }
    }
    Ok(ReplicationInput {
        name,
        source_repository: source,
        remote_id,
        remote_repository,
        enabled: body.enabled.unwrap_or(true),
        name_filter,
        version_filter,
        propagate_deletes: body.propagate_deletes.unwrap_or(false),
        settle_seconds,
        reconcile_minutes,
        alert_channels: channels,
    })
}

fn replication_conflict(error: &DbErr) -> Result<Response, ApiError> {
    if constraint_violated(error, "replication_name_lower") {
        return Ok(message(
            StatusCode::CONFLICT,
            "A replication with this name exists already",
        ));
    }
    if is_unique(error) {
        return Ok(message(
            StatusCode::CONFLICT,
            "A replication from this repository to this repository of this remote exists already",
        ));
    }
    if constraint_violated(error, "source_repository") {
        return Err(ApiError::invalid(
            "sourceRepository",
            "there is no repository with this name",
        ));
    }
    if is_foreign_key(error) {
        return Err(ApiError::invalid(
            "remoteId",
            "there is no replication remote with this id",
        ));
    }
    Err(ApiError::internal(anyhow::anyhow!("{error}")))
}

fn replication_values(id: Uuid, input: &ReplicationInput) -> Vec<Value> {
    vec![
        id.into(),
        input.name.clone().into(),
        input.source_repository.clone().into(),
        input.remote_id.into(),
        input.remote_repository.clone().into(),
        input.enabled.into(),
        input.name_filter.clone().into(),
        input.version_filter.clone().into(),
        input.propagate_deletes.into(),
        input.settle_seconds.into(),
        input.reconcile_minutes.into(),
        input.alert_channels.clone().into(),
    ]
}

fn replication_audit(model: &ReplicationModel) -> serde_json::Value {
    json!({
        "id": model.id,
        "name": model.name,
        "sourceRepository": model.source_repository,
        "remoteId": model.remote_id,
        "remoteRepository": model.remote_repository,
        "enabled": model.enabled,
        "nameFilter": model.name_filter,
        "versionFilter": model.version_filter,
        "propagateDeletes": model.propagate_deletes,
        "settleSeconds": model.settle_seconds,
        "reconcileMinutes": model.reconcile_minutes,
        "alertChannels": model.alert_channels,
    })
}

async fn create_replication(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<ReplicationRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let subject = caller.require(CREATE)?;
    let Json(body) = body?;
    let input = replication_input(&state, subject, body).await?;
    let id = Uuid::new_v4();
    let sql = format!(
        "INSERT INTO replication (id, name, source_repository, remote_id, remote_repository, \
         enabled, name_filter, version_filter, propagate_deletes, settle_seconds, \
         reconcile_minutes, alert_channels) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING {REPLICATION_COLUMNS}"
    );
    let inserted =
        ReplicationModel::find_by_statement(statement(&sql, replication_values(id, &input)))
            .one(state.db().conn())
            .await;
    let created = match inserted {
        Ok(Some(created)) => created,
        Ok(None) => {
            return Err(ApiError::internal(anyhow::anyhow!(
                "the replication was not returned after it was inserted"
            )));
        }
        Err(error) => return replication_conflict(&error),
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "replication.created", &created.name)
                .with(replication_audit(&created)),
        )
        .await;
    Ok((StatusCode::CREATED, axum::Json(created)).into_response())
}

async fn require_current_source(
    state: &AppState,
    subject: &Subject,
    id: Uuid,
) -> Result<(), ApiError> {
    #[derive(FromQueryResult)]
    struct Current {
        source_repository: String,
    }
    let current = Current::find_by_statement(statement(
        "SELECT source_repository FROM replication WHERE id = $1",
        vec![id.into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| missing("replication", id))?;
    let format = SourceRepository::find_by_statement(statement(
        "SELECT format, type AS kind FROM repository WHERE name = $1",
        vec![current.source_repository.clone().into()],
    ))
    .one(state.db().conn())
    .await?
    .map_or_else(|| "*".to_owned(), |found| found.format);
    let administered = format!(
        "astrum:repository-admin:{format}:{}:read",
        current.source_repository
    );
    match subject.is_permitted(&administered) {
        true => Ok(()),
        false => Err(ApiError::forbidden()),
    }
}

async fn update_replication(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    body: Result<Json<ReplicationRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let subject = caller.require(UPDATE)?;
    let id = parse_id(&id, "replication")?;
    require_current_source(&state, subject, id).await?;
    let Json(body) = body?;
    let input = replication_input(&state, subject, body).await?;
    let sql = format!(
        "UPDATE replication SET name = $2, source_repository = $3, remote_id = $4, \
         remote_repository = $5, enabled = $6, name_filter = $7, version_filter = $8, \
         propagate_deletes = $9, settle_seconds = $10, reconcile_minutes = $11, \
         alert_channels = $12, updated_at = now() WHERE id = $1 RETURNING {REPLICATION_COLUMNS}"
    );
    let updated =
        ReplicationModel::find_by_statement(statement(&sql, replication_values(id, &input)))
            .one(state.db().conn())
            .await;
    let updated = match updated {
        Ok(Some(updated)) => updated,
        Ok(None) => return Err(missing("replication", id)),
        Err(error) => return replication_conflict(&error),
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "replication.updated", &updated.name)
                .with(replication_audit(&updated)),
        )
        .await;
    Ok(axum::Json(updated).into_response())
}

async fn remove_replication(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let subject = caller.require(DELETE)?;
    let id = parse_id(&id, "replication")?;
    require_current_source(&state, subject, id).await?;
    let deleted = Named::find_by_statement(statement(
        "DELETE FROM replication WHERE id = $1 RETURNING name",
        vec![id.into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| missing("replication", id))?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "replication.deleted", &deleted.name)
                .with(json!({"id": id, "name": deleted.name})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
