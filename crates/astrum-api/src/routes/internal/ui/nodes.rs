use std::time::Duration;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, FixedOffset};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, FromQueryResult, Statement};
use serde_json::{Value, json};
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::{UI, time};
use crate::error::ApiError;
use crate::{AppState, Caller};

pub const HEARTBEAT: Duration = Duration::from_secs(15);
const ALIVE_SECONDS: f64 = 60.0;
const FORGET_SECONDS: f64 = 86_400.0;
const READ: &str = "astrum:settings:read";

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/nodes"), get(list))
}

pub fn node_name() -> String {
    std::env::var("ASTRUM_NODE_NAME")
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "astrum".to_owned())
}

pub async fn beat(state: &AppState) -> Result<(), DbErr> {
    let db = state.db().conn();
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO node (id, name, version, started_at, last_seen) VALUES ($1, $2, $3, $4, now()) \
         ON CONFLICT (id) DO UPDATE SET last_seen = now(), name = excluded.name",
        [
            state.node_id().into(),
            node_name().into(),
            env!("CARGO_PKG_VERSION").into(),
            state.started_at().into(),
        ],
    ))
    .await?;
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM node WHERE last_seen < now() - make_interval(secs => $1)",
        [FORGET_SECONDS.into()],
    ))
    .await?;
    Ok(())
}

pub async fn leave(state: &AppState) {
    let _ = state
        .db()
        .conn()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM node WHERE id = $1",
            [state.node_id().into()],
        ))
        .await;
}

pub fn spawn(state: AppState) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Err(error) = beat(&state).await {
                tracing::warn!(%error, "the node heartbeat was not recorded");
            }
            tokio::time::sleep(HEARTBEAT).await;
        }
    })
}

#[derive(FromQueryResult)]
struct Row {
    id: Uuid,
    name: String,
    version: String,
    started_at: DateTime<FixedOffset>,
    last_seen: DateTime<FixedOffset>,
}

async fn list(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    beat(&state).await?;
    let rows = Row::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id, name, version, started_at, last_seen FROM node \
         WHERE last_seen >= now() - make_interval(secs => $1) ORDER BY started_at, id",
        [ALIVE_SECONDS.into()],
    ))
    .all(state.db().conn())
    .await?;
    Ok(Json(json!({
        "nodes": rows
            .iter()
            .map(|row| json!({
                "id": row.id,
                "name": row.name,
                "version": row.version,
                "startedAt": time(row.started_at),
                "lastSeen": time(row.last_seen),
                "current": row.id == state.node_id(),
            }))
            .collect::<Vec<_>>(),
    })))
}
