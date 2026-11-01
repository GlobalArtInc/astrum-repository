use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::any;

use astrum_content::{ContentError, ContentRequest, Ctx};

use crate::routes::components::{browsable, browse_address, interface_address};
use crate::routes::html::{self, escape};
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
    serve(state, caller, name, String::new(), request).await
}

async fn content(
    State(state): State<AppState>,
    caller: Caller,
    Path((name, path)): Path<(String, String)>,
    request: Request,
) -> Response {
    serve(state, caller, name, path, request).await
}

async fn serve(
    state: AppState,
    caller: Caller,
    name: String,
    path: String,
    request: Request,
) -> Response {
    let directory = path.is_empty() || path.ends_with('/');
    let navigation = directory && html::browser_navigation(request.method(), request.headers());
    let response = dispatch(
        state.clone(),
        caller.clone(),
        name.clone(),
        path.clone(),
        request,
    )
    .await
    .unwrap_or_else(IntoResponse::into_response);
    if !navigation
        || !matches!(
            response.status(),
            StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST
        )
    {
        return response;
    }
    let Ok(Some(repository)) = state.content().repository(&name).await else {
        return response;
    };
    if !browsable(&state, &caller, &repository.name).await {
        return response;
    }
    let prefix = format!("/{}", path.trim_start_matches('/'));
    let body = format!(
        "<p>This is the address the clients of the {} {} repository use; it has no page of its own. Look at its content in the <a href=\"{}\">HTML index</a> or in the <a href=\"{}\">web interface</a>.</p>\n",
        escape(repository.format.id),
        repository.kind,
        browse_address(&name, &prefix),
        interface_address(&name, &prefix),
    );
    (
        response.status(),
        [(header::CACHE_CONTROL, "no-store")],
        Html(html::page(&escape(&name), &body)),
    )
        .into_response()
}

pub async fn context(
    state: &AppState,
    caller: Caller,
    name: &str,
    headers: &axum::http::HeaderMap,
    uri: &axum::http::Uri,
) -> Result<(Ctx, std::sync::Arc<dyn astrum_content::Format>), ContentError> {
    context_at(
        state,
        caller,
        name,
        base_url(headers, uri),
        crate::auth::client_address().unwrap_or_else(|| client_ip(headers)),
    )
    .await
}

pub async fn context_at(
    state: &AppState,
    caller: Caller,
    name: &str,
    base: String,
    address: String,
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
        base,
        address,
    );
    Ok((ctx, format))
}

pub(crate) async fn dispatch(
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
    let rule_path = astrum_repo::routing::request_path(&path);
    if !state
        .content()
        .routing_allows(&ctx.repository, &rule_path)
        .await?
    {
        return Ok(blocked());
    }
    format
        .handle(ctx.with_request_path(&rule_path), request)
        .await
}

pub fn blocked() -> Response {
    (
        axum::http::StatusCode::FORBIDDEN,
        [(axum::http::header::CONTENT_TYPE, "text/plain;charset=utf-8")],
        astrum_content::routing::BLOCKED,
    )
        .into_response()
}
