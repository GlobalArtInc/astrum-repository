use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};

use crate::routes::internal::INTERNAL;
use crate::{ApiError, AppState, Caller};

pub const READ: &str = "astrum:logging:read";
pub const MARK: &str = "astrum:logging:create";
const MAX_TAIL: u64 = 10 * 1024 * 1024;
const MAX_MARK: usize = 1000;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{INTERNAL}/logging/logs"), get(list))
        .route(&format!("{INTERNAL}/logging/logs/{{file}}"), get(read))
        .route(&format!("{INTERNAL}/logging/log/mark"), post(mark))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    file_name: String,
    size: u64,
    last_modified: Option<i64>,
    node: String,
    current: bool,
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<Entry>>, ApiError> {
    caller.require(READ)?;
    let logging = state.logging().clone();
    let Some(current) = logging.files().map(|files| files.current().to_owned()) else {
        return Ok(axum::Json(Vec::new()));
    };
    let listed = tokio::task::spawn_blocking(move || match logging.files() {
        Some(files) => files.list(),
        None => Ok(Vec::new()),
    })
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
    Ok(axum::Json(
        listed
            .into_iter()
            .map(|file| Entry {
                current: file.name == current,
                node: node_of(&file.name),
                last_modified: file
                    .modified
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|age| age.as_millis() as i64),
                size: file.size,
                file_name: file.name,
            })
            .collect(),
    ))
}

fn node_of(file: &str) -> String {
    let base = file.trim_end_matches(|c: char| c.is_ascii_digit());
    let base = base.strip_suffix('.').unwrap_or(base);
    base.strip_prefix("astrum-")
        .and_then(|rest| rest.strip_suffix(".log"))
        .unwrap_or(file)
        .to_owned()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadQuery {
    bytes_count: Option<i64>,
    from_byte: Option<u64>,
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(file): Path<String>,
    Query(query): Query<ReadQuery>,
) -> Result<Response, ApiError> {
    caller.require(READ)?;
    let logging = state.logging().clone();
    let missing = || ApiError::not_found(format!("Log file {file} not found"));
    let path = logging
        .files()
        .and_then(|files| files.path_of(&file))
        .ok_or_else(missing)?;
    let disposition = format!("attachment; filename=\"{file}\"");
    let headers = |size: u64, start: u64| {
        [
            (header::CONTENT_TYPE, "text/plain;charset=utf-8".to_owned()),
            (header::CONTENT_DISPOSITION, disposition.clone()),
            (
                header::HeaderName::from_static("x-astrum-log-size"),
                size.to_string(),
            ),
            (
                header::HeaderName::from_static("x-astrum-log-start"),
                start.to_string(),
            ),
        ]
    };
    match (query.bytes_count, query.from_byte) {
        (None, None) => {
            let opened = tokio::fs::File::open(&path).await;
            let handle = match opened {
                Ok(handle) => handle,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(missing());
                }
                Err(error) => return Err(ApiError::internal(error)),
            };
            let size = handle.metadata().await.map_err(ApiError::internal)?.len();
            let body = Body::from_stream(tokio_util::io::ReaderStream::new(handle));
            Ok((StatusCode::OK, headers(size, 0), body).into_response())
        }
        (count, from) => {
            let limit = count.map_or(MAX_TAIL, |count| count.unsigned_abs().min(MAX_TAIL));
            let from = match count {
                Some(count) if count < 0 => None,
                _ => from.or(Some(0)),
            };
            let name = file.clone();
            let chunk = tokio::task::spawn_blocking(move || {
                let files = logging.files().expect("the log files were found above");
                files.read(&name, from, limit)
            })
            .await
            .map_err(ApiError::internal)?;
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(missing());
                }
                Err(error) => return Err(ApiError::internal(error)),
            };
            Ok((
                StatusCode::OK,
                headers(chunk.size, chunk.start),
                Bytes::from(chunk.text),
            )
                .into_response())
        }
    }
}

async fn mark(
    State(state): State<AppState>,
    caller: Caller,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    caller.require(MARK)?;
    let text = String::from_utf8_lossy(&body);
    let text = match serde_json::from_str::<String>(&text) {
        Ok(text) => text,
        Err(_) => text.into_owned(),
    };
    let text = text.trim();
    if text.is_empty() {
        return Err(ApiError::invalid("mark", "Enter the text of the mark"));
    }
    if text.chars().count() > MAX_MARK {
        return Err(ApiError::invalid(
            "mark",
            format!("A mark can hold at most {MAX_MARK} characters"),
        ));
    }
    let text = format!("{text} (by {})", caller.initiator());
    let logging = state.logging().clone();
    let written = tokio::task::spawn_blocking(move || match logging.files() {
        Some(files) => files.mark(&text),
        None => {
            tracing::info!(mark = %text, "log mark");
            Ok(())
        }
    })
    .await
    .map_err(ApiError::internal)?;
    written.map_err(ApiError::internal)?;
    Ok(StatusCode::NO_CONTENT)
}
