use astrum_api::routes::content::context;
use astrum_api::{AppState, Caller};
use astrum_content::{ContentError, ContentRequest};
use astrum_db::audit::Event;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;

use crate::{ID, REALM, TOKEN_DAYS};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/repository/{name}/v1/users/authenticate",
            any(authenticate),
        )
        .route(
            "/repository/{name}/v2/users/authenticate",
            any(authenticate),
        )
}

fn text(status: StatusCode, body: String) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .unwrap_or_else(|_| status.into_response())
}

async fn authenticate(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let (ctx, format) =
        match context(&state, caller.clone(), &name, &parts.headers, &parts.uri).await {
            Ok(found) => found,
            Err(error) => return error.into_response(),
        };
    if format.id() != ID {
        let path = parts
            .uri
            .path()
            .strip_prefix(&format!("/repository/{name}"))
            .unwrap_or("/")
            .to_owned();
        let request = match ContentRequest::new(
            parts.method,
            &path,
            parts.uri.query().map(str::to_owned),
            parts.headers,
            body,
        ) {
            Ok(request) => request,
            Err(error) => return error.into_response(),
        };
        return format
            .handle(ctx, request)
            .await
            .unwrap_or_else(IntoResponse::into_response);
    }
    let Some(subject) = caller
        .subject()
        .filter(|subject| !subject.anonymous)
        .cloned()
    else {
        return ContentError::Unauthorized.into_response();
    };
    let security = state.security();
    if !security.model().realm_active(REALM) {
        return text(
            StatusCode::FORBIDDEN,
            "The Conan Bearer Token Realm is not active; enable it under Security > Realms"
                .to_owned(),
        );
    }
    match security
        .issue_format_token(
            REALM,
            &subject.user_id,
            Some(chrono::Duration::days(TOKEN_DAYS)),
        )
        .await
    {
        Ok(token) => {
            state
                .audit(
                    &caller,
                    Event::new("security.conantoken", "created", &subject.user_id),
                )
                .await;
            text(StatusCode::OK, token)
        }
        Err(error) => ContentError::internal(error).into_response(),
    }
}
