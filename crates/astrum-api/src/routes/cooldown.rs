use std::collections::HashMap;

use axum::Router;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, FixedOffset, SecondsFormat};
use sea_orm::{DbBackend, FromQueryResult, Statement};
use serde_json::{Value, json};

use crate::routes::components::visibility;
use crate::{ApiError, AppState, Caller};

pub const WAITS: &str = "/service/rest/v1/security/cooldown";
const PAGE: usize = 50;

pub fn routes() -> Router<AppState> {
    Router::new().route(WAITS, get(list_waits))
}

#[derive(FromQueryResult)]
struct Wait {
    repository: String,
    component_id: i64,
    format: String,
    namespace: String,
    name: String,
    version: String,
    published_at: DateTime<FixedOffset>,
    available_at: DateTime<FixedOffset>,
    blocked: bool,
    refusals: i32,
    first_refused_at: DateTime<FixedOffset>,
}

type Cursor = (DateTime<FixedOffset>, String, i64);

fn invalid() -> ApiError {
    ApiError::BadRequest("the continuation token is not valid".to_owned())
}

fn after(token: Option<&str>) -> Result<Option<Cursor>, ApiError> {
    let Some(token) = token else { return Ok(None) };
    let bytes = URL_SAFE_NO_PAD.decode(token).map_err(|_| invalid())?;
    let (available, repository, component): (String, String, i64) =
        serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let available = DateTime::parse_from_rfc3339(&available).map_err(|_| invalid())?;
    Ok(Some((available, repository, component)))
}

fn token(wait: &Wait) -> String {
    let key = json!([
        wait.available_at
            .to_rfc3339_opts(SecondsFormat::Micros, false),
        wait.repository,
        wait.component_id
    ]);
    URL_SAFE_NO_PAD.encode(key.to_string())
}

fn time(value: DateTime<FixedOffset>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, false)
}

fn item(wait: &Wait) -> Value {
    json!({
        "repository": wait.repository,
        "format": wait.format,
        "namespace": wait.namespace,
        "name": wait.name,
        "version": wait.version,
        "publishedAt": time(wait.published_at),
        "availableAt": time(wait.available_at),
        "blocked": wait.blocked,
        "refusals": wait.refusals,
        "firstRefusedAt": time(wait.first_refused_at),
    })
}

async fn list_waits(
    State(state): State<AppState>,
    caller: Caller,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    if caller.subject().is_none() {
        return Err(ApiError::Unauthorized);
    }
    let repositories = visibility(
        &state,
        &caller,
        params.get("repository").map(String::as_str),
    )
    .await?
    .repositories();
    let cursor = after(params.get("continuationToken").map(String::as_str))?;
    if repositories.is_empty() {
        return Ok(axum::Json(json!({"items": [], "continuationToken": null})).into_response());
    }
    let (at, repository, component) = match cursor {
        Some((at, repository, component)) => (Some(at), Some(repository), Some(component)),
        None => (None, None, None),
    };
    let mut rows = Wait::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT w.repository, w.component_id, c.format, c.namespace, c.name, c.version, \
         w.published_at, w.available_at, w.blocked, w.refusals, w.first_refused_at \
         FROM cooldown_wait w JOIN component c ON c.id = w.component_id \
         WHERE w.repository = ANY($1) \
           AND ($2::timestamptz IS NULL OR (w.available_at, w.repository, w.component_id) > ($2, $3, $4)) \
         ORDER BY w.available_at, w.repository, w.component_id LIMIT $5",
        [
            repositories.into(),
            at.into(),
            repository.into(),
            component.into(),
            ((PAGE + 1) as i64).into(),
        ],
    ))
    .all(state.db().conn())
    .await
    .map_err(ApiError::internal)?;
    let next = if rows.len() > PAGE {
        rows.truncate(PAGE);
        rows.last().map(token)
    } else {
        None
    };
    Ok(axum::Json(json!({
        "items": rows.iter().map(item).collect::<Vec<_>>(),
        "continuationToken": next,
    }))
    .into_response())
}
