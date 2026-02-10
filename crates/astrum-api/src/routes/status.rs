use std::io::ErrorKind;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use astrum_db::read_only;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use tokio::time::timeout;

use crate::AppState;

pub const STATUS: &str = "/service/rest/v1/status";
pub const STATUS_WRITABLE: &str = "/service/rest/v1/status/writable";

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

static PROBE: AtomicU64 = AtomicU64::new(0);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(STATUS, get(readable))
        .route(STATUS_WRITABLE, get(writable))
}

async fn readable(State(state): State<AppState>) -> StatusCode {
    match timeout(PROBE_TIMEOUT, state.db().ping()).await {
        Ok(Ok(())) => StatusCode::OK,
        Ok(Err(error)) => unavailable("the database does not answer", &error),
        Err(_) => unavailable("the database does not answer", &"timed out"),
    }
}

async fn writable(State(state): State<AppState>) -> StatusCode {
    match timeout(PROBE_TIMEOUT, probe_writes(&state)).await {
        Ok(Ok(())) => StatusCode::OK,
        Ok(Err(reason)) => unavailable("writes are not possible", &reason),
        Err(_) => unavailable("writes are not possible", &"timed out"),
    }
}

async fn probe_writes(state: &AppState) -> Result<(), String> {
    let db = state.db();
    if !db
        .accepts_writes()
        .await
        .map_err(|error| error.to_string())?
    {
        return Err("the database is read-only".to_owned());
    }
    let mode = read_only::state(db.conn())
        .await
        .map_err(|error| error.to_string())?;
    if mode.frozen {
        return Err(format!(
            "read-only mode is on: {}",
            mode.reason.as_deref().unwrap_or("no reason given")
        ));
    }
    probe_data_dir(state).await
}

async fn probe_data_dir(state: &AppState) -> Result<(), String> {
    let probe = state.data_dir().join("tmp").join(format!(
        ".writable-{}-{}",
        std::process::id(),
        PROBE.fetch_add(1, Ordering::Relaxed)
    ));
    let failed = |error: std::io::Error| format!("cannot write {}: {error}", probe.display());
    tokio::fs::write(&probe, b"").await.map_err(failed)?;
    match tokio::fs::remove_file(&probe).await {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(failed(error)),
        _ => Ok(()),
    }
}

fn unavailable(what: &str, reason: &dyn std::fmt::Display) -> StatusCode {
    tracing::warn!(%reason, "{what}");
    StatusCode::SERVICE_UNAVAILABLE
}
