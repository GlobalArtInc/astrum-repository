use axum::Router;
use axum::extract::{Path, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::any;

use astrum_content::{ContentError, ContentRequest, Ctx};

use crate::routes::internal::client_ip;
use crate::routes::repositories::base_url;
use crate::{AppState, Caller};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/repository/{name}", any(root))
        .route("/repository/{name}/", any(root))
        .route("/repository/{name}/{*path}", any(content))
}

async fn root(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    request: Request,
) -> Response {
    dispatch(state, caller, name, String::new(), request)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

async fn content(
    State(state): State<AppState>,
    caller: Caller,
    Path((name, path)): Path<(String, String)>,
    request: Request,
) -> Response {
    dispatch(state, caller, name, path, request)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

pub async fn context(
    state: &AppState,
    caller: Caller,
    name: &str,
    headers: &axum::http::HeaderMap,
    uri: &axum::http::Uri,
) -> Result<(Ctx, std::sync::Arc<dyn astrum_content::Format>), ContentError> {
    let repository = state
        .content()
        .repository(name)
        .await?
        .ok_or_else(|| ContentError::not_found("Repository not found"))?;
    if !repository.online {
        return Err(ContentError::Unavailable(format!(
            "Repository offline: {name}"
        )));
    }
    let format = state
        .content()
        .format(repository.format.id)
        .ok_or_else(|| {
            ContentError::Unavailable(format!(
                "{} repositories are not served by this server yet",
                repository.format.id
            ))
        })?;
    let ctx = state.content().context(
        repository,
        state.security().model(),
        caller.0,
        base_url(headers, uri),
        client_ip(headers),
    );
    Ok((ctx, format))
}

async fn dispatch(
    state: AppState,
    caller: Caller,
    name: String,
    path: String,
    request: Request,
) -> Result<Response, ContentError> {
    let (parts, body) = request.into_parts();
    let (ctx, format) = context(&state, caller, &name, &parts.headers, &parts.uri).await?;
    let request = ContentRequest::new(
        parts.method,
        &path,
        parts.uri.query().map(str::to_owned),
        parts.headers,
        body,
    )?;
    format.handle(ctx, request).await
}
