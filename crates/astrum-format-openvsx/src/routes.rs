use std::collections::HashMap;

use astrum_api::routes::content::{blocked, context};
use astrum_api::{AppState, Caller};
use astrum_content::{ContentError, ContentRequest};
use astrum_security::tokens::looks_like_token;
use astrum_security::{AuthError, Subject};
use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde_json::json;

use crate::ID;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/repository/{name}/api/-/publish", any(with_token))
        .route("/repository/{name}/api/-/namespace/create", any(with_token))
        .route(
            "/repository/{name}/api/{namespace}/verify-pat",
            any(with_token),
        )
        .route(
            "/repository/{name}/api/{namespace}/{extension}/delete",
            any(with_token),
        )
}

fn refused(status: StatusCode, message: &str) -> Box<Response> {
    let mut response = (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        json!({"error": message}).to_string(),
    )
        .into_response();
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    Box::new(response)
}

fn token_of(query: Option<&str>, headers: &HeaderMap) -> Option<String> {
    let given = url::form_urlencoded::parse(query.unwrap_or("").as_bytes())
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.trim().to_owned());
    let bearer = || {
        let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?.trim();
        let (scheme, token) = value.split_once(' ')?;
        scheme
            .eq_ignore_ascii_case("bearer")
            .then(|| token.trim().to_owned())
    };
    given.or_else(bearer).filter(|value| !value.is_empty())
}

async fn authenticate(state: &AppState, token: &str) -> Result<Subject, Box<Response>> {
    let security = state.security();
    if looks_like_token(token) {
        return match security.authenticate_token(None, token).await {
            Ok(subject) => Ok(subject),
            Err(AuthError::BadCredentials) => Err(refused(
                StatusCode::UNAUTHORIZED,
                "the access token is not valid; create one under Access Tokens in Astrum Repository",
            )),
            Err(AuthError::Internal(error)) => {
                Err(Box::new(ContentError::internal(error).into_response()))
            }
        };
    }
    let Some((user, password)) = token.split_once(':') else {
        return Err(refused(
            StatusCode::UNAUTHORIZED,
            "the token must be an access token of Astrum Repository or <user>:<password>",
        ));
    };
    if security.sign_in_blocked(user) {
        return Err(refused(
            StatusCode::TOO_MANY_REQUESTS,
            "too many failed sign-ins; wait a few minutes",
        ));
    }
    match security.authenticate(user, password).await {
        Ok(subject) => {
            security.sign_in_succeeded(user);
            Ok(subject)
        }
        Err(AuthError::BadCredentials) => {
            security.sign_in_failed(user);
            tracing::info!(user, "an Open VSX sign-in failed");
            Err(refused(
                StatusCode::UNAUTHORIZED,
                "the user name or password is wrong",
            ))
        }
        Err(AuthError::Internal(error)) => {
            Err(Box::new(ContentError::internal(error).into_response()))
        }
    }
}

async fn with_token(
    State(state): State<AppState>,
    caller: Caller,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
) -> Response {
    let Some(name) = params.get("name").cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let (parts, body) = request.into_parts();
    let (mut ctx, format) =
        match context(&state, caller.clone(), &name, &parts.headers, &parts.uri).await {
            Ok(found) => found,
            Err(error) => return error.into_response(),
        };
    let anonymous = caller.subject().is_none_or(|subject| subject.anonymous);
    if format.id() == ID
        && anonymous
        && let Some(token) = token_of(parts.uri.query(), &parts.headers)
    {
        match authenticate(&state, &token).await {
            Ok(subject) => ctx.subject = Some(subject),
            Err(response) => return *response,
        }
    }
    let path = parts
        .uri
        .path()
        .strip_prefix(&format!("/repository/{name}"))
        .unwrap_or("/")
        .to_owned();
    let rule_path = astrum_repo::routing::request_path(&path);
    match state
        .content()
        .routing_allows(&ctx.repository, &rule_path)
        .await
    {
        Ok(true) => {}
        Ok(false) => return blocked(),
        Err(error) => return error.into_response(),
    }
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
    format
        .handle(ctx.with_request_path(&rule_path), request)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_come_from_the_query_or_the_bearer_header() {
        let none = HeaderMap::new();
        assert_eq!(
            token_of(Some("allVersions=true&token=astrum_x"), &none).as_deref(),
            Some("astrum_x")
        );
        assert_eq!(token_of(Some("token="), &none), None);
        assert_eq!(token_of(None, &none), None);
        let mut bearer = HeaderMap::new();
        bearer.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer user:secret"),
        );
        assert_eq!(token_of(None, &bearer).as_deref(), Some("user:secret"));
        assert_eq!(
            token_of(Some("token=astrum_y"), &bearer).as_deref(),
            Some("astrum_y")
        );
        let mut basic = HeaderMap::new();
        basic.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic dXNlcg=="),
        );
        assert_eq!(token_of(None, &basic), None);
    }
}
