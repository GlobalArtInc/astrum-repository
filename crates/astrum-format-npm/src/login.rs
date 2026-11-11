use astrum_api::routes::content::context;
use astrum_api::second_factor::{self, Method as Factor, Step};
use astrum_api::{AppState, Caller};
use astrum_content::{ContentError, ContentRequest};
use astrum_db::audit::Event;
use astrum_security::format_tokens::looks_like_format_token;
use astrum_security::tokens::looks_like_token;
use astrum_security::{AuthError, Gate, Pending};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde_json::{Value, json};

use crate::{ID, REALM, reply};

const MAX_LOGIN: usize = 64 * 1024;
const USER_PREFIX: &str = "org.couchdb.user:";
const OTP_HEADER: &str = "npm-otp";
const OTP_CHALLENGE: &str = "OTP";
const TOTP_DIGITS: usize = 6;
const DIRECTORY_ACCOUNT: &str =
    "this account signs in through a directory; use an access token instead";

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
        (&Method::PUT, Some(user)) => login(&state, user, &parts.headers, body).await,
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

fn challenge(status: StatusCode, message: &str) -> Response {
    let mut response = refused(status, message);
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(OTP_CHALLENGE),
    );
    response
}

fn one_time_password(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(OTP_HEADER)?;
    let code = String::from_utf8_lossy(value.as_bytes()).trim().to_owned();
    (!code.is_empty()).then_some(code)
}

fn method_of(code: &str) -> Factor {
    let squeezed: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    if squeezed.len() == TOTP_DIGITS && squeezed.chars().all(|c| c.is_ascii_digit()) {
        Factor::Totp
    } else {
        Factor::Recovery
    }
}

async fn login(
    state: &AppState,
    user: &str,
    headers: &HeaderMap,
    body: Body,
) -> Result<Response, ContentError> {
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
    let mut verified = None;
    let subject = match security.authenticate(&user, password).await {
        Ok(subject) => subject,
        Err(AuthError::BadCredentials) => return Ok(wrong_password(state, &user)),
        Err(AuthError::SecondFactor(pending)) => match pending.gate {
            Gate::Verify => match second_step(state, &user, headers, &pending).await? {
                Second::Done(response) => return Ok(response),
                Second::Accepted(subject, method) => {
                    verified = Some(method);
                    subject
                }
            },
            Gate::Enroll => {
                return Ok(refused(
                    StatusCode::UNAUTHORIZED,
                    "this account must set up a second factor first; sign in to the web interface",
                ));
            }
            Gate::Open => {
                tracing::error!(user, "the second factor gate demanded nothing; refused");
                return Ok(wrong_password(state, &user));
            }
        },
        Err(AuthError::Internal(error)) => return Err(ContentError::internal(error)),
    };
    security.sign_in_succeeded(&user);
    let token = match security.issue_format_token(REALM, &subject, None).await {
        Ok(token) => token,
        Err(astrum_security::Error::NotLocal) => {
            tracing::info!(user, "an npm sign-in came from an account of a directory");
            return Ok(refused(StatusCode::UNAUTHORIZED, DIRECTORY_ACCOUNT));
        }
        Err(error) => return Err(ContentError::internal(error)),
    };
    let caller = Caller(Some(subject.clone()));
    let mut event = Event::new("security.npmtoken", "created", &subject.user_id);
    if let Some(method) = verified {
        event = event.with(json!({"secondFactor": method.as_str()}));
    }
    state.audit(&caller, event).await;
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

enum Second {
    Done(Response),
    Accepted(astrum_security::Subject, Factor),
}

fn wrong_password(state: &AppState, user: &str) -> Response {
    state.security().sign_in_failed(user);
    tracing::info!(user, "an npm sign-in failed");
    refused(
        StatusCode::UNAUTHORIZED,
        "the user name or password is wrong",
    )
}

async fn second_step(
    state: &AppState,
    user: &str,
    headers: &HeaderMap,
    pending: &Pending,
) -> Result<Second, ContentError> {
    let security = state.security();
    let overview = security
        .summary(&pending.user_id)
        .await
        .map_err(ContentError::internal)?;
    if !overview.totp && overview.recovery_codes_left == 0 {
        return Ok(Second::Done(refused(
            StatusCode::UNAUTHORIZED,
            "this account has passkeys only; sign in with an access token: put it into .npmrc as _authToken",
        )));
    }
    let Some(code) = one_time_password(headers) else {
        return Ok(Second::Done(challenge(
            StatusCode::UNAUTHORIZED,
            "a one-time password is required",
        )));
    };
    let Some(subject) = second_factor::subject_of(state, pending) else {
        return Ok(Second::Done(wrong_password(state, user)));
    };
    let method = method_of(&code);
    let step = second_factor::check(state, &pending.user_id, method, &code)
        .await
        .map_err(ContentError::internal)?;
    match step {
        Step::Accepted { left, .. } => {
            if let Some(left) = left {
                second_factor::audit_recovery_used(state, &subject, left).await;
            }
            Ok(Second::Accepted(subject, method))
        }
        Step::Wrong => {
            second_factor::record_failure(state, user, method, "npm").await;
            Ok(Second::Done(challenge(
                StatusCode::UNAUTHORIZED,
                "the one-time password is wrong or was already used",
            )))
        }
        Step::Locked(seconds) => {
            second_factor::record_failure(state, user, method, "npm").await;
            let mut response = refused(
                StatusCode::TOO_MANY_REQUESTS,
                "too many wrong codes; wait and try again",
            );
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
            Ok(Second::Done(response))
        }
    }
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
