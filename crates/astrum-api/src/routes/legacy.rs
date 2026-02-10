use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::any;

use astrum_content::ContentError;

use crate::routes::content::dispatch;
use crate::{AppState, Caller};

const RAW: &str = "raw";
const NO_PATH: &str = "Repository path must not be null or empty";
const NO_SLASH: &str = "Repository path must have another '/' after initial '/'";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Any,
    Sites,
}

fn kind(prefix: &str, segment: &str) -> Option<Kind> {
    match (prefix, segment) {
        ("content", "repositories" | "groups") => Some(Kind::Any),
        ("content", "sites") => Some(Kind::Sites),
        ("service", "repositories" | "repo_groups") => Some(Kind::Any),
        _ => None,
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/content/{segment}",
            any(|Path(segment): Path<String>| async move { empty("content", &segment) }),
        )
        .route(
            "/content/{segment}/",
            any(|Path(segment): Path<String>| async move { incomplete("content", &segment) }),
        )
        .route(
            "/content/{segment}/{*rest}",
            any(
                |state: State<AppState>,
                 caller: Caller,
                 Path((segment, rest)): Path<(String, String)>,
                 request: Request| async move {
                    content("content", state, caller, segment, rest, request).await
                },
            ),
        )
        .route(
            "/service/local/{segment}",
            any(|Path(segment): Path<String>| async move { empty("service", &segment) }),
        )
        .route(
            "/service/local/{segment}/",
            any(|Path(segment): Path<String>| async move { incomplete("service", &segment) }),
        )
        .route(
            "/service/local/{segment}/{*rest}",
            any(
                |state: State<AppState>,
                 caller: Caller,
                 Path((segment, rest)): Path<(String, String)>,
                 request: Request| async move {
                    content("service", state, caller, segment, rest, request).await
                },
            ),
        )
}

fn empty(prefix: &str, segment: &str) -> Response {
    match kind(prefix, segment) {
        Some(_) => ContentError::BadRequest(NO_PATH.to_owned()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn incomplete(prefix: &str, segment: &str) -> Response {
    match kind(prefix, segment) {
        Some(_) => ContentError::BadRequest(NO_SLASH.to_owned()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn content(
    prefix: &str,
    State(state): State<AppState>,
    caller: Caller,
    segment: String,
    rest: String,
    request: Request,
) -> Response {
    let Some(kind) = kind(prefix, &segment) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let (name, path) = match rest.split_once('/') {
        Some((name, path)) if !name.is_empty() => (name.to_owned(), path.to_owned()),
        _ => return ContentError::BadRequest(NO_SLASH.to_owned()).into_response(),
    };
    if kind == Kind::Sites {
        match state.content().repository(&name).await {
            Ok(Some(repository)) if repository.format.id == RAW => {}
            Ok(_) => return ContentError::not_found("Repository not found").into_response(),
            Err(error) => return error.into_response(),
        }
    }
    dispatch(state, caller, name, path, request)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}
