use std::net::IpAddr;

use astrum_db::audit::Event;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::sessions::{KEY_ADDRESS_LENGTH, KEY_USER_LENGTH};
use astrum_security::{AuthError, Gate, Subject};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use serde_json::{Value, json};

use super::{INTERNAL, is_https};
use crate::auth::{CSRF_COOKIE, SESSION_COOKIE, client_address, cookie};
use crate::{ApiError, AppState, Caller, Json};

const COOKIE_AGE: u64 = 400 * 24 * 60 * 60;
const SIGN_IN_PATH: &str = "/service/rest/internal/session";

pub fn routes() -> Router<AppState> {
    Router::new().route(
        &format!("{INTERNAL}/session"),
        get(current).post(sign_in).delete(sign_out),
    )
}

#[derive(Deserialize)]
struct SignIn {
    username: String,
    password: String,
}

async fn current(State(state): State<AppState>, caller: Caller) -> axum::Json<Value> {
    axum::Json(describe(&state, caller.subject()))
}

async fn sign_in(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SignIn>,
) -> Result<Response, ApiError> {
    let security = state.security();
    let user = body.username.trim().to_owned();
    if security.sign_in_blocked(&user) {
        return Ok(refused(
            StatusCode::TOO_MANY_REQUESTS,
            "too many failed sign-ins; wait a few minutes",
        ));
    }
    let outcome = match security.authenticate(&user, &body.password).await {
        Ok(subject) => Ok(subject),
        Err(AuthError::BadCredentials) => Err(Gate::Open),
        Err(AuthError::SecondFactor(pending)) => Err(pending.gate),
        Err(AuthError::Internal(error)) => return Err(error.into()),
    };
    let subject = match outcome {
        Ok(subject) => subject,
        Err(gate) => {
            security.sign_in_failed(&user);
            tracing::info!(user, "a sign-in to the user interface failed");
            audit_failed_sign_in(&state, &user, SIGN_IN_PATH, &headers, gate.failure_reason())
                .await;
            if security.sign_in_blocked(&user) {
                audit_rate_limited(&state, &user, crate::client::current(), "UI").await;
            }
            return Ok(refused(
                StatusCode::UNAUTHORIZED,
                "the user name or password is wrong",
            ));
        }
    };
    security.sign_in_succeeded(&user);
    let mut response = axum::Json(describe(&state, Some(&subject))).into_response();
    begin(&state, &headers, &subject, &mut response).await?;
    Ok(response)
}

pub(crate) async fn begin(
    state: &AppState,
    headers: &HeaderMap,
    subject: &Subject,
    response: &mut Response,
) -> Result<(), ApiError> {
    subject
        .require_unnarrowed()
        .map_err(|_| ApiError::forbidden())?;
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let session = state
        .security()
        .open_session(subject, &client_address().unwrap_or_default(), user_agent)
        .await?;
    let caller = Caller(Some(subject.clone()));
    state
        .audit(
            &caller,
            Event::new(&subject.realm, "login", "").with(json!({"principal": subject.user_id})),
        )
        .await;
    let secure = is_https(headers);
    set_cookie(
        response,
        SESSION_COOKIE,
        &session.token,
        true,
        secure,
        COOKIE_AGE,
    );
    set_cookie(
        response,
        CSRF_COOKIE,
        &session.csrf,
        false,
        secure,
        COOKIE_AGE,
    );
    Ok(())
}

async fn sign_out(
    State(state): State<AppState>,
    headers: HeaderMap,
    caller: Caller,
) -> Result<Response, ApiError> {
    if let Some(token) = cookie(&headers, SESSION_COOKIE)
        && state.security().close_session(token).await?
        && let Some(subject) = caller.subject()
    {
        state
            .audit(
                &caller,
                Event::new(&subject.realm, "logout", "")
                    .with(json!({"principal": subject.user_id})),
            )
            .await;
    }
    let secure = is_https(&headers);
    let mut response = StatusCode::NO_CONTENT.into_response();
    set_cookie(&mut response, SESSION_COOKIE, "", true, secure, 0);
    set_cookie(&mut response, CSRF_COOKIE, "", false, secure, 0);
    Ok(response)
}

fn describe(state: &AppState, subject: Option<&Subject>) -> Value {
    let Some(subject) = subject else {
        return json!({"authenticated": false, "anonymous": false, "user": null, "permissions": []});
    };
    let model = state.security().model();
    let identity = subject.public_identity();
    let user = model
        .users
        .get(&subject.user_id)
        .filter(|_| !identity.hidden);
    let profile = match identity.hidden {
        true => astrum_security::Profile::default(),
        false => state.security().profile(subject),
    };
    let permissions: Vec<String> = subject
        .permissions()
        .iter()
        .map(ToString::to_string)
        .collect();
    json!({
        "authenticated": !subject.anonymous,
        "anonymous": subject.anonymous,
        "user": {
            "userId": identity.id,
            "firstName": profile.first_name,
            "lastName": profile.last_name,
            "emailAddress": profile.email,
            "status": user.filter(|_| subject.source == DEFAULT_SOURCE).map(|user| user.status.as_str()).unwrap_or("active"),
            "source": if subject.source.is_empty() { DEFAULT_SOURCE } else { subject.source.as_str() },
            "realm": subject.realm,
        },
        "permissions": permissions,
    })
}

pub(crate) async fn audit_failed_sign_in(
    state: &AppState,
    user: &str,
    path: &str,
    headers: &HeaderMap,
    reason: &str,
) {
    if !state.security().model().users.contains_key(user) {
        return;
    }
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    state
        .audit(
            &Caller(None),
            Event::new("security.user", "authentication", "").with(json!({
                "path": path,
                "userId": user,
                "remoteIp": client_address().unwrap_or_default(),
                "userAgent": user_agent,
                "wasSuccessful": false,
                "failureReasons": [reason],
            })),
        )
        .await;
}

pub(crate) async fn audit_rate_limited(
    state: &AppState,
    user: &str,
    address: Option<IpAddr>,
    method: &str,
) {
    let user: String = user.chars().take(KEY_USER_LENGTH).collect();
    let address: String = address
        .map(|address| address.to_string())
        .unwrap_or_default()
        .chars()
        .take(KEY_ADDRESS_LENGTH)
        .collect();
    state
        .audit(
            &Caller(None),
            Event::new("security.auth.ratelimit", "blocked", &user).with(json!({
                "sourceIp": address,
                "username": user,
                "authMethod": method,
            })),
        )
        .await;
}

fn refused(status: StatusCode, message: &str) -> Response {
    (status, axum::Json(json!({"id": "*", "message": message}))).into_response()
}

pub(crate) fn set_cookie(
    response: &mut Response,
    name: &str,
    value: &str,
    http_only: bool,
    secure: bool,
    max_age: u64,
) {
    let mut cookie = format!("{name}={value}; Path=/; SameSite=Lax; Max-Age={max_age}");
    if http_only {
        cookie.push_str("; HttpOnly");
    }
    if secure {
        cookie.push_str("; Secure");
    }
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}
