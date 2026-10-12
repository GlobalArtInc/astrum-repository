use astrum_api::routes::content::context;
use astrum_api::{AppState, Caller};
use astrum_content::{ContentError, ContentRequest};
use astrum_db::audit::Event;
use astrum_security::AuthError;
use astrum_security::format_tokens::looks_like_format_token;
use astrum_security::tokens::looks_like_token;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde_json::{Value, json};

use crate::{ID, REALM, reply};

const MAX_LOGIN: usize = 64 * 1024;
const USER_PREFIX: &str = "org.couchdb.user:";

pub fn routes() -> Router<AppState> {
    Router::new().route("/repository/{name}/-/user/{*rest}", any(user))
}

async fn user(
    State(state): State<AppState>,
    caller: Caller,
    Path((name, rest)): Path<(String, String)>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let (ctx, format) =
        match context(&state, caller.clone(), &name, &parts.headers, &parts.uri).await {
            Ok(found) => found,
            Err(error) => return error.into_response(),
        };
    if format.id() != ID {
        let request = match ContentRequest::new(
            parts.method,
            &format!("-/user/{rest}"),
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
    let answer = match (&parts.method, rest.strip_prefix(USER_PREFIX)) {
        (&Method::PUT, Some(user)) => login(&state, user, body).await,
        (_, Some(_)) => Err(ContentError::MethodNotAllowed {
            method: parts.method.to_string(),
            allow: "PUT",
        }),
        (&Method::DELETE, None) => match rest.strip_prefix("token/") {
            Some(token) => logout(&state, &caller, token).await,
            None => Err(ContentError::not_found("not found")),
        },
        _ => Err(ContentError::not_found("not found")),
    };
    answer.unwrap_or_else(reply::error)
}

fn refused(status: StatusCode, message: &str) -> Response {
    reply::json(status, &json!({"success": false, "error": message}))
}

async fn login(state: &AppState, user: &str, body: Body) -> Result<Response, ContentError> {
    let bytes = axum::body::to_bytes(body, MAX_LOGIN)
        .await
        .map_err(|error| {
            ContentError::BadRequest(format!("the request cannot be read: {error}"))
        })?;
    let doc: Value = serde_json::from_slice(&bytes)
        .map_err(|_| ContentError::BadRequest("the request body is not JSON".to_owned()))?;
    let user = doc
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(user)
        .trim()
        .to_owned();
    let password = doc
        .get("password")
        .and_then(Value::as_str)
        .ok_or_else(|| ContentError::BadRequest("the request carries no password".to_owned()))?;
    let security = state.security();
    if !security.model().realm_active(REALM) {
        return Ok(refused(
            StatusCode::FORBIDDEN,
            "npm sign-in is off: an administrator has to activate the npm Bearer Token realm",
        ));
    }
    if looks_like_token(password) {
        return Ok(refused(
            StatusCode::UNAUTHORIZED,
            "sign in with your password; put access tokens into .npmrc as _authToken instead",
        ));
    }
    if security.sign_in_blocked(&user) {
        return Ok(refused(
            StatusCode::TOO_MANY_REQUESTS,
            "too many failed sign-ins; wait a few minutes",
        ));
    }
    let subject = match security.authenticate(&user, password).await {
        Ok(subject) => subject,
        Err(AuthError::BadCredentials) => {
            security.sign_in_failed(&user);
            tracing::info!(user, "an npm sign-in failed");
            return Ok(refused(
                StatusCode::UNAUTHORIZED,
                "the user name or password is wrong",
            ));
        }
        Err(AuthError::Internal(error)) => return Err(ContentError::internal(error)),
    };
    security.sign_in_succeeded(&user);
    let token = security
        .issue_format_token(REALM, &subject.user_id, None)
        .await
        .map_err(ContentError::internal)?;
    let caller = Caller(Some(subject.clone()));
    state
        .audit(
            &caller,
            Event::new("security.npmtoken", "created", &subject.user_id),
        )
        .await;
    Ok(reply::json(
        StatusCode::CREATED,
        &json!({
            "ok": true,
            "id": format!("{USER_PREFIX}{}", subject.user_id),
            "rev": "_we_dont_use_revs_any_more",
            "token": token,
        }),
    ))
}

async fn logout(state: &AppState, caller: &Caller, token: &str) -> Result<Response, ContentError> {
    if looks_like_format_token(token) == Some(REALM)
        && state
            .security()
            .revoke_format_token(token)
            .await
            .map_err(ContentError::internal)?
    {
        state
            .audit(
                caller,
                Event::new("security.npmtoken", "deleted", caller.initiator()),
            )
            .await;
    }
    Ok(reply::json(StatusCode::OK, &json!({"ok": true})))
}
