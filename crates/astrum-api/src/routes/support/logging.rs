use std::collections::BTreeMap;

use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_logging::{Level, Logger, ROOT};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::Deserialize;

use crate::routes::internal::ui::UI;
use crate::{ApiError, AppState, Caller, Json};

pub const READ: &str = "astrum:logging:read";
pub const UPDATE: &str = "astrum:logging:update";
pub const SETTING: &str = "logging.levels";
const DOMAIN: &str = "logging";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/loggingConfiguration"), get(list))
        .route(&format!("{UI}/loggingConfiguration/reset"), post(reset_all))
        .route(
            &format!("{UI}/loggingConfiguration/{{name}}"),
            get(read).put(update).delete(reset),
        )
        .route(
            &format!("{UI}/loggingConfiguration/{{name}}/reset"),
            post(reset),
        )
}

pub async fn sync(state: &AppState) -> Result<(), ApiError> {
    let stored: BTreeMap<String, Level> = setting::get(state.db().conn(), SETTING)
        .await
        .map_err(ApiError::internal)?
        .unwrap_or_default();
    state.logging().replace(stored).map_err(ApiError::internal)
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<Logger>>, ApiError> {
    caller.require(READ)?;
    sync(&state).await?;
    Ok(axum::Json(state.logging().loggers()))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<Logger>, ApiError> {
    caller.require(READ)?;
    sync(&state).await?;
    Ok(axum::Json(state.logging().logger(&name)))
}

#[derive(Deserialize)]
struct Change {
    name: Option<String>,
    level: String,
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    Json(change): Json<Change>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let name = name.trim().to_owned();
    if change
        .name
        .as_deref()
        .is_some_and(|other| other.trim() != name)
    {
        return Err(ApiError::invalid(
            "name",
            "The logger name in the body differs from the one in the address",
        ));
    }
    if !astrum_logging::valid_name(&name) {
        return Err(ApiError::invalid(
            "name",
            "Use ROOT or a Rust module path such as astrum_api::routes",
        ));
    }
    let level: Level = change
        .level
        .parse()
        .map_err(|error: astrum_logging::UnknownLevel| {
            ApiError::invalid("level", error.to_string())
        })?;
    store(&state, |levels| {
        levels.insert(name.clone(), level);
    })
    .await?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &name)
                .with(serde_json::json!({ "level": level.as_str() })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn reset(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let mut removed = false;
    store(&state, |levels| removed = levels.remove(&name).is_some()).await?;
    if removed {
        state
            .audit(&caller, Event::new(DOMAIN, "reset", &name))
            .await;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn reset_all(State(state): State<AppState>, caller: Caller) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    store(&state, BTreeMap::clear).await?;
    state
        .audit(&caller, Event::new(DOMAIN, "reset", ROOT))
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn store(
    state: &AppState,
    change: impl FnOnce(&mut BTreeMap<String, Level>),
) -> Result<(), ApiError> {
    let db = state.db().conn();
    let mut levels: BTreeMap<String, Level> = setting::get(db, SETTING)
        .await
        .map_err(ApiError::internal)?
        .unwrap_or_default();
    change(&mut levels);
    state
        .logging()
        .replace(levels.clone())
        .map_err(ApiError::internal)?;
    setting::put(db, SETTING, &levels)
        .await
        .map_err(ApiError::internal)
}
