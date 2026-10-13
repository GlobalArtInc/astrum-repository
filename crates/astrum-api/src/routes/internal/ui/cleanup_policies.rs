use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use super::UI;
use crate::error::ApiError;
use crate::routes::cleanup_policies::{READ, names_for};
use crate::{AppState, Caller};

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/cleanup-policies/names"), get(names))
}

#[derive(Deserialize)]
struct NamesQuery {
    format: Option<String>,
}

fn administers_repositories(caller: &Caller) -> bool {
    let Some(subject) = caller.subject() else {
        return false;
    };
    let wanted = astrum_security::permission::Permission::parse("astrum:repository-admin")
        .expect("a valid permission");
    subject.permissions().iter().any(|granted| {
        granted.implies(&wanted) || granted.to_string().starts_with("astrum:repository-admin:")
    })
}

async fn names(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<NamesQuery>,
) -> Result<Json<Value>, ApiError> {
    if !administers_repositories(&caller) {
        caller.require(READ)?;
    }
    let policies = names_for(&state).await?;
    Ok(Json(Value::Array(
        policies
            .iter()
            .filter(|policy| {
                query
                    .format
                    .as_deref()
                    .is_none_or(|format| policy.fits(format))
            })
            .map(|policy| json!({"name": policy.name, "format": policy.format, "notes": policy.notes}))
            .collect(),
    )))
}
