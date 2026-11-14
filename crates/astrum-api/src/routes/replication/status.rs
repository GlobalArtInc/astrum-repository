use astrum_db::audit::Event;
use astrum_replication::lifecycle::{self, Snapshot};
use astrum_replication::protocol::component_key;
use astrum_security::Subject;
use astrum_tasks::store::Requested;
use axum::Router;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{ConnectionTrait, FromQueryResult, TransactionTrait, Value};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::config::{
    DOMAIN, READ, UPDATE, message, missing, parse_id, require_current_source, statement,
};
use crate::{ApiError, AppState, Caller, Json};

const BASE: &str = "/service/rest/v1/replications/{id}";
const DEFAULT_PAGE: i64 = 50;
const MAX_PAGE: i64 = 200;
const MAX_IDS: usize = 1000;
const DISABLED_NOTE: &str = "the replication is disabled: the run will be skipped";
const STATES: [&str; 3] = ["dead", "pending", "leased"];

const STATUS: &str = "SELECT r.enabled, r.alert_state, r.last_success_at, r.last_failure_at, \
     r.last_error, r.last_reconcile_at, \
     (SELECT count(*) FROM replication_item i \
         WHERE i.replication_id = r.id AND i.state = 'pending') AS pending, \
     (SELECT count(*) FROM replication_item i \
         WHERE i.replication_id = r.id AND i.state = 'leased') AS leased, \
     (SELECT count(*) FROM replication_item i \
         WHERE i.replication_id = r.id AND i.state = 'dead') AS dead, \
     extract(epoch FROM now() - \
         (SELECT min(i.first_queued_at) FROM replication_item i \
             WHERE i.replication_id = r.id AND i.state IN ('pending', 'leased')))::bigint AS lag_seconds, \
     CASE WHEN m.suspended_until > now() THEN m.suspended_until END AS suspended_until, \
     m.failures, m.last_error AS remote_error \
     FROM replication r JOIN replication_remote m ON m.id = r.remote_id \
     WHERE r.id = $1";

const ITEMS: &str = "SELECT id, namespace, name, version, state, reason, attempts, not_before, \
     first_queued_at, last_error FROM replication_item \
     WHERE replication_id = $1 AND state = $2 AND id > $3 ORDER BY id LIMIT $4";

const RETRY: &str = "UPDATE replication_item SET state = 'pending', attempts = 0, \
     reason = 'retry', not_before = now(), first_queued_at = now() \
     WHERE replication_id = $1 AND state = 'dead' AND ($2::bigint[] IS NULL OR id = ANY($2))";

const DISCARD: &str = "DELETE FROM replication_item \
     WHERE replication_id = $1 AND state = 'dead' AND ($2::bigint[] IS NULL OR id = ANY($2))";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{BASE}/status"), get(read_status))
        .route(&format!("{BASE}/items"), get(list_items))
        .route(&format!("{BASE}/items/retry"), post(retry_items))
        .route(&format!("{BASE}/items/discard"), post(discard_items))
        .route(&format!("{BASE}/reconcile"), post(reconcile))
}

#[derive(FromQueryResult)]
struct Counted {
    enabled: bool,
    alert_state: String,
    last_success_at: Option<DateTimeWithTimeZone>,
    last_failure_at: Option<DateTimeWithTimeZone>,
    last_error: Option<String>,
    last_reconcile_at: Option<DateTimeWithTimeZone>,
    pending: i64,
    leased: i64,
    dead: i64,
    lag_seconds: Option<i64>,
    suspended_until: Option<DateTimeWithTimeZone>,
    failures: i32,
    remote_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoteStatus {
    suspended_until: Option<DateTimeWithTimeZone>,
    failures: i32,
    last_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    enabled: bool,
    pending: i64,
    leased: i64,
    dead: i64,
    lag_seconds: Option<i64>,
    last_success_at: Option<DateTimeWithTimeZone>,
    last_failure_at: Option<DateTimeWithTimeZone>,
    last_error: Option<String>,
    last_reconcile_at: Option<DateTimeWithTimeZone>,
    alert_state: String,
    remote: RemoteStatus,
}

#[derive(Deserialize)]
struct ItemsQuery {
    state: Option<String>,
    after: Option<i64>,
    limit: Option<i64>,
}

#[derive(FromQueryResult)]
struct ItemRow {
    id: i64,
    namespace: String,
    name: String,
    version: String,
    state: String,
    reason: String,
    attempts: i32,
    not_before: DateTimeWithTimeZone,
    first_queued_at: DateTimeWithTimeZone,
    last_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemModel {
    id: i64,
    key: String,
    state: String,
    reason: String,
    attempts: i32,
    not_before: DateTimeWithTimeZone,
    first_queued_at: DateTimeWithTimeZone,
    last_error: Option<String>,
}

#[derive(Serialize)]
struct ItemPage {
    items: Vec<ItemModel>,
    next: Option<i64>,
}

#[derive(Deserialize)]
struct ItemsRequest {
    ids: Option<Vec<i64>>,
    all: Option<bool>,
}

enum Selection {
    Ids(Vec<i64>),
    All,
}

impl Selection {
    fn parse(body: ItemsRequest) -> Result<Self, ApiError> {
        match (body.ids, body.all) {
            (Some(ids), None) if ids.is_empty() => {
                Err(ApiError::invalid("ids", "must not be empty"))
            }
            (Some(ids), None) if ids.len() > MAX_IDS => Err(ApiError::invalid(
                "ids",
                format!("must hold at most {MAX_IDS} ids"),
            )),
            (Some(ids), None) => Ok(Selection::Ids(ids)),
            (None, Some(true)) => Ok(Selection::All),
            (None, Some(false)) => Err(ApiError::invalid("all", "must be true when it is given")),
            _ => Err(ApiError::invalid("ids", "give exactly one of ids and all")),
        }
    }

    fn value(&self) -> Value {
        match self {
            Selection::Ids(ids) => ids.clone().into(),
            Selection::All => Value::Array(sea_orm::sea_query::ArrayType::BigInt, None),
        }
    }

    fn all(&self) -> bool {
        matches!(self, Selection::All)
    }
}

async fn require_source_view(
    state: &AppState,
    subject: &Subject,
    id: Uuid,
) -> Result<(), ApiError> {
    #[derive(FromQueryResult)]
    struct Source {
        source_repository: String,
        format: Option<String>,
    }
    let source = Source::find_by_statement(statement(
        "SELECT r.source_repository, p.format FROM replication r \
         LEFT JOIN repository p ON p.name = r.source_repository WHERE r.id = $1",
        vec![id.into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| missing("replication", id))?;
    let format = source.format.as_deref().unwrap_or("*");
    let name = &source.source_repository;
    let viewed = ["browse", "read"].iter().any(|action| {
        subject.is_permitted(&format!("astrum:repository-view:{format}:{name}:{action}"))
    });
    match viewed {
        true => Ok(()),
        false => Err(ApiError::forbidden()),
    }
}

async fn read_status(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<axum::Json<Status>, ApiError> {
    caller.require(READ)?;
    let id = parse_id(&id, "replication")?;
    let counted = Counted::find_by_statement(statement(STATUS, vec![id.into()]))
        .one(state.db().conn())
        .await?
        .ok_or_else(|| missing("replication", id))?;
    Ok(axum::Json(Status {
        enabled: counted.enabled,
        pending: counted.pending,
        leased: counted.leased,
        dead: counted.dead,
        lag_seconds: counted.lag_seconds,
        last_success_at: counted.last_success_at,
        last_failure_at: counted.last_failure_at,
        last_error: counted.last_error,
        last_reconcile_at: counted.last_reconcile_at,
        alert_state: counted.alert_state,
        remote: RemoteStatus {
            suspended_until: counted.suspended_until,
            failures: counted.failures,
            last_error: counted.remote_error,
        },
    }))
}

async fn list_items(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    query: Result<Query<ItemsQuery>, QueryRejection>,
) -> Result<axum::Json<ItemPage>, ApiError> {
    let subject = caller.require(READ)?;
    let id = parse_id(&id, "replication")?;
    require_source_view(&state, subject, id).await?;
    let Query(query) = query.map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
    let wanted = query
        .state
        .as_deref()
        .filter(|wanted| STATES.contains(wanted))
        .ok_or_else(|| ApiError::invalid("state", "must be one of dead, pending or leased"))?;
    let limit = query.limit.unwrap_or(DEFAULT_PAGE);
    if !(1..=MAX_PAGE).contains(&limit) {
        return Err(ApiError::invalid(
            "limit",
            format!("must be between 1 and {MAX_PAGE}"),
        ));
    }
    let after = query.after.unwrap_or(0);
    let mut rows = ItemRow::find_by_statement(statement(
        ITEMS,
        vec![
            id.into(),
            wanted.to_owned().into(),
            after.into(),
            (limit + 1).into(),
        ],
    ))
    .all(state.db().conn())
    .await?;
    let more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let next = rows.last().map(|row| row.id).filter(|_| more);
    let items = rows
        .into_iter()
        .map(|row| ItemModel {
            key: component_key(&row.namespace, &row.name, &row.version),
            id: row.id,
            state: row.state,
            reason: row.reason,
            attempts: row.attempts,
            not_before: row.not_before,
            first_queued_at: row.first_queued_at,
            last_error: row.last_error,
        })
        .collect();
    Ok(axum::Json(ItemPage { items, next }))
}

async fn change_items(
    state: &AppState,
    caller: &Caller,
    id: &str,
    body: Result<Json<ItemsRequest>, ApiError>,
    sql: &str,
    event: &str,
) -> Result<u64, ApiError> {
    let subject = caller.require(UPDATE)?;
    let id = parse_id(id, "replication")?;
    require_current_source(state, subject, id).await?;
    let Json(body) = body?;
    let selection = Selection::parse(body)?;
    let changed = state
        .db()
        .conn()
        .execute(statement(sql, vec![id.into(), selection.value()]))
        .await?
        .rows_affected();
    #[derive(FromQueryResult)]
    struct Named {
        name: String,
    }
    let named = Named::find_by_statement(statement(
        "SELECT name FROM replication WHERE id = $1",
        vec![id.into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| missing("replication", id))?;
    state
        .audit(
            caller,
            Event::new(DOMAIN, event, &named.name).with(json!({
                "id": id,
                "name": named.name,
                "all": selection.all(),
                "count": changed,
            })),
        )
        .await;
    Ok(changed)
}

async fn retry_items(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    body: Result<Json<ItemsRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let retried = change_items(
        &state,
        &caller,
        &id,
        body,
        RETRY,
        "replication.items.retried",
    )
    .await?;
    Ok(axum::Json(json!({"retried": retried})).into_response())
}

async fn discard_items(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    body: Result<Json<ItemsRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let discarded = change_items(
        &state,
        &caller,
        &id,
        body,
        DISCARD,
        "replication.items.discarded",
    )
    .await?;
    Ok(axum::Json(json!({"discarded": discarded})).into_response())
}

async fn reconcile(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let subject = caller.require(UPDATE)?;
    let id = parse_id(&id, "replication")?;
    require_current_source(&state, subject, id).await?;
    let txn = state.db().conn().begin().await?;
    let snapshot = Snapshot::locked(&txn, id)
        .await?
        .ok_or_else(|| missing("replication", id))?;
    let requested = lifecycle::run_now(&txn, &snapshot, &caller.initiator()).await?;
    txn.commit().await?;
    match requested {
        Requested::Queued(run) => {
            state.tasks().wake();
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "replication.reconcile.requested", &snapshot.name)
                        .with(json!({"id": id, "name": snapshot.name, "run": run})),
                )
                .await;
            let answer = match snapshot.enabled {
                true => json!({"run": run}),
                false => json!({"run": run, "note": DISABLED_NOTE}),
            };
            Ok((StatusCode::ACCEPTED, axum::Json(answer)).into_response())
        }
        Requested::AlreadyActive => Ok(message(
            StatusCode::CONFLICT,
            format!(
                "The reconcile of the replication \"{}\" is already running or about to run",
                snapshot.name
            ),
        )),
    }
}
