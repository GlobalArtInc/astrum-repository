mod roles;
mod users;
mod validate;

use astrum_db::audit::Event;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::{DbErr, SqlErr};
use serde_json::json;

use crate::{ApiError, AppState, Caller};

pub const SECURITY: &str = "/service/rest/v1/security";

pub fn routes() -> Router<AppState> {
    Router::new().merge(users::routes()).merge(roles::routes())
}

async fn changed(state: &AppState, caller: &Caller, event: Event) -> Result<(), ApiError> {
    state.security().reload().await?;
    state.audit(caller, event).await;
    Ok(())
}

fn conflict(message: impl Into<String>) -> Response {
    (
        StatusCode::CONFLICT,
        axum::Json(json!({"id": "*", "message": message.into()})),
    )
        .into_response()
}

fn unique_violation(error: &DbErr) -> bool {
    matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}

fn now() -> chrono::DateTime<chrono::FixedOffset> {
    chrono::Utc::now().fixed_offset()
}

fn dedup(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}
