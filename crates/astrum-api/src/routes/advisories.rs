use std::collections::HashMap;

use astrum_db::audit::Event;
use astrum_scan::osv::{
    self, Feed, ImportError, Outcome, SourceError, SyncError, TempFile, Upload,
};
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::SecondsFormat;
use serde_json::{Value, json};

use crate::error::ApiError;
use crate::{AppState, Caller};

pub const FEEDS: &str = "/service/rest/v1/security/advisories/feeds";
pub const IMPORT: &str = "/service/rest/v1/security/advisories/import";
pub const DOMAIN: &str = "advisories";
pub const READ: &str = "astrum:policies:read";
pub const UPDATE: &str = "astrum:policies:update";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(FEEDS, get(list))
        .route(IMPORT, post(import))
}

fn document(feed: &Feed) -> Value {
    let time = |at: Option<chrono::DateTime<chrono::Utc>>| {
        at.map(|at| at.to_rfc3339_opts(SecondsFormat::Secs, true))
    };
    json!({
        "ecosystem": feed.ecosystem,
        "records": feed.records,
        "syncedAt": time(feed.synced_at),
        "attemptedAt": time(feed.attempted_at),
        "lastError": feed.last_error,
    })
}

async fn list(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let feeds = osv::feeds(state.db().conn()).await?;
    Ok(Json(Value::Array(feeds.iter().map(document).collect())))
}

fn refusal(status: StatusCode, message: String) -> Response {
    (status, Json(json!({"id": "*", "message": message}))).into_response()
}

async fn import(
    State(state): State<AppState>,
    caller: Caller,
    Query(params): Query<HashMap<String, String>>,
    body: Body,
) -> Result<Response, ApiError> {
    caller.require(UPDATE)?;
    let ecosystem = match params.get("ecosystem").map(String::as_str) {
        Some(name) if osv::valid_ecosystem(name) => name.to_owned(),
        Some(_) => {
            return Err(ApiError::invalid(
                "ecosystem",
                "is not an OSV ecosystem name",
            ));
        }
        None => return Err(ApiError::invalid("ecosystem", "is required")),
    };
    let force = match params.get("force").map(String::as_str) {
        None | Some("false") => false,
        Some("true") => true,
        Some(_) => return Err(ApiError::invalid("force", "must be true or false")),
    };
    let directory = state.data_dir().join("tmp");
    let staged =
        match TempFile::from_stream(&directory, body.into_data_stream(), osv::MAX_ARCHIVE_BYTES)
            .await
        {
            Ok(staged) => staged,
            Err(SourceError::TooLarge) => {
                return Ok(refusal(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!(
                        "the archive is larger than {} bytes",
                        osv::MAX_ARCHIVE_BYTES
                    ),
                ));
            }
            Err(SourceError::Network(message)) => {
                return Err(ApiError::BadRequest(format!(
                    "the upload was interrupted: {message}"
                )));
            }
            Err(error) => return Err(ApiError::internal(error)),
        };
    if staged.size() == 0 {
        return Err(ApiError::BadRequest("the body is empty".to_owned()));
    }
    let db = state.db().conn().clone();
    let target = ecosystem.clone();
    let outcome = tokio::spawn(async move {
        let source = Upload::new(staged);
        osv::sync_with(&db, &source, &target, force, &|| false).await
    })
    .await
    .map_err(ApiError::internal)?;
    let stats = match outcome {
        Ok(Outcome::Imported(stats)) => stats,
        Ok(Outcome::NotModified) => {
            return Err(ApiError::internal(anyhow::anyhow!(
                "an upload cannot be not modified"
            )));
        }
        Err(SyncError::Busy(ecosystem)) => {
            return Ok(refusal(
                StatusCode::CONFLICT,
                format!("an import of the ecosystem {ecosystem} is already running"),
            ));
        }
        Err(SyncError::Import(error @ ImportError::TooMuchData)) => {
            return Ok(refusal(StatusCode::PAYLOAD_TOO_LARGE, error.to_string()));
        }
        Err(SyncError::Import(
            error @ (ImportError::Archive(_)
            | ImportError::TooManyEntries
            | ImportError::Empty
            | ImportError::OutOfScope(_)
            | ImportError::Shrinks { .. }),
        )) => return Err(ApiError::BadRequest(error.to_string())),
        Err(error) => return Err(ApiError::internal(anyhow::anyhow!("{error}"))),
    };
    let result = json!({
        "ecosystem": ecosystem,
        "records": stats.records,
        "changed": stats.changed,
        "unchanged": stats.unchanged,
        "removed": stats.removed,
        "skipped": stats.skipped,
    });
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "imported", &ecosystem).with(result.clone()),
        )
        .await;
    Ok(Json(result).into_response())
}
