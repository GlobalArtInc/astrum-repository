use std::net::IpAddr;

use astrum_db::audit::Event;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::sessions::{KEY_ADDRESS_LENGTH, KEY_USER_LENGTH};
use astrum_security::two_factor::store::{
    CEREMONY_ATTEMPTS, Ceremony, CeremonyKind, NewCeremony, RecoveryOutcome, StoreError,
    Verification,
};
use astrum_security::{AuthError, Gate, Pending, Subject};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{INTERNAL, is_https, session_passkey};
use crate::auth::{CSRF_COOKIE, RestrictedSession, SESSION_COOKIE, client_address, cookie};
use crate::{ApiError, AppState, Caller, Json};

const COOKIE_AGE: u64 = 400 * 24 * 60 * 60;
const SIGN_IN_PATH: &str = "/service/rest/internal/session";
pub(super) const SIGN_IN_COOKIE: &str = "astrum_signin";
pub(super) const SIGN_IN_LIFETIME: Duration = Duration::minutes(5);
const SIGN_IN_EXPIRED: &str = "sign-in-expired";
const CODE_WRONG: &str = "second-factor-invalid";
const CODE_MAX_BYTES: usize = 64;
const CEREMONY_MAX_BYTES: usize = 128;
const CREDENTIAL_MAX_BYTES: usize = 16 * 1024;
const TOTP: &str = "totp";
const RECOVERY: &str = "recovery";
const PASSKEY: &str = "passkey";
pub(super) const TWO_FACTOR_DOMAIN: &str = "security.twofactor";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{INTERNAL}/session"),
            get(current).post(sign_in).delete(sign_out),
        )
        .route(
            &format!("{INTERNAL}/session/second-factor"),
            post(second_factor),
        )
        .route(
            &format!("{INTERNAL}/session/second-factor/passkey-options"),
            post(session_passkey::options),
        )
}

#[derive(Deserialize)]
struct SignIn {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct SecondStep {
    method: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    ceremony: String,
    #[serde(default)]
    credential: Option<Value>,
}

#[derive(Default)]
struct Opening<'a> {
    restricted: bool,
    second_factor: Option<&'a str>,
    offset_steps: Option<i8>,
}

async fn current(
    State(state): State<AppState>,
    caller: Caller,
    restricted: Option<axum::Extension<RestrictedSession>>,
) -> axum::Json<Value> {
    axum::Json(describe(&state, caller.subject(), restricted.is_some()))
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
    match security.authenticate(&user, &body.password).await {
        Ok(subject) => {
            security.sign_in_succeeded(&user);
            let mut response = axum::Json(describe(&state, Some(&subject), false)).into_response();
            begin(&state, &headers, &subject, &mut response).await?;
            Ok(response)
        }
        Err(AuthError::BadCredentials) => {
            refuse_password(&state, &user, &headers, Gate::Open).await
        }
        Err(AuthError::SecondFactor(pending)) => match pending.gate {
            Gate::Verify => start_second_step(&state, &user, &headers, pending).await,
            Gate::Enroll => open_restricted(&state, &user, &headers, pending).await,
            Gate::Open => {
                tracing::error!(user, "the second factor gate demanded nothing; refused");
                refuse_password(&state, &user, &headers, Gate::Open).await
            }
        },
        Err(AuthError::Internal(error)) => Err(error.into()),
    }
}

async fn refuse_password(
    state: &AppState,
    user: &str,
    headers: &HeaderMap,
    gate: Gate,
) -> Result<Response, ApiError> {
    let security = state.security();
    security.sign_in_failed(user);
    tracing::info!(user, "a sign-in to the user interface failed");
    audit_failed_sign_in(state, user, SIGN_IN_PATH, headers, gate.failure_reason()).await;
    if security.sign_in_blocked(user) {
        audit_rate_limited(state, user, crate::client::current(), "UI").await;
    }
    Ok(refused(
        StatusCode::UNAUTHORIZED,
        "the user name or password is wrong",
    ))
}

fn subject_of(state: &AppState, pending: &Pending) -> Option<Subject> {
    state
        .security()
        .model()
        .local_subject(&pending.user_id, &pending.realm)
}

async fn open_restricted(
    state: &AppState,
    user: &str,
    headers: &HeaderMap,
    pending: Pending,
) -> Result<Response, ApiError> {
    let Some(subject) = subject_of(state, &pending) else {
        return refuse_password(state, user, headers, pending.gate).await;
    };
    state.security().sign_in_succeeded(user);
    let mut response = axum::Json(describe(state, Some(&subject), true)).into_response();
    let opening = Opening {
        restricted: true,
        ..Opening::default()
    };
    open(state, headers, &subject, &mut response, opening).await?;
    state
        .audit(
            &Caller(Some(subject.clone())),
            Event::new(TWO_FACTOR_DOMAIN, "restricted.signin", &subject.user_id)
                .with(json!({"userId": subject.user_id})),
        )
        .await;
    Ok(response)
}

async fn start_second_step(
    state: &AppState,
    user: &str,
    headers: &HeaderMap,
    pending: Pending,
) -> Result<Response, ApiError> {
    let security = state.security();
    let Some(subject) = subject_of(state, &pending) else {
        return refuse_password(state, user, headers, pending.gate).await;
    };
    let overview = security
        .summary(&pending.user_id)
        .await
        .map_err(store_error)?;
    let mut methods = Vec::new();
    if overview.totp {
        methods.push(TOTP);
    }
    if session_passkey::usable(state, &pending.user_id).await? {
        methods.push(PASSKEY);
    }
    if overview.recovery_codes_left > 0 {
        methods.push(RECOVERY);
    }
    let handle = security
        .replace_ceremony(NewCeremony {
            kind: CeremonyKind::SignIn,
            user_id: &pending.user_id,
            realm: &pending.realm,
            state: Some(json!({"stamp": security.sign_in_stamp(&subject)})),
            client_ip: &client_address().unwrap_or_default(),
            lifetime: SIGN_IN_LIFETIME,
        })
        .await
        .map_err(store_error)?;
    let expires_at = Utc::now() + SIGN_IN_LIFETIME;
    let mut response = axum::Json(json!({
        "authenticated": false,
        "anonymous": false,
        "user": null,
        "permissions": [],
        "secondFactor": {"methods": methods, "expiresAt": expires_at.to_rfc3339()},
    }))
    .into_response();
    set_ceremony_cookie(
        &mut response,
        &handle,
        is_https(headers),
        u64::try_from(SIGN_IN_LIFETIME.num_seconds()).unwrap_or(0),
    );
    Ok(response)
}

pub(super) fn store_error(error: StoreError) -> ApiError {
    ApiError::internal(error)
}

pub(super) fn expired(headers: &HeaderMap) -> Response {
    let mut response = (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "id": "*",
            "message": "the sign-in expired; start again with the password",
            "code": SIGN_IN_EXPIRED,
        })),
    )
        .into_response();
    set_ceremony_cookie(&mut response, "", is_https(headers), 0);
    response
}

fn wrong_code(method: &str) -> Response {
    let message = match method {
        PASSKEY => "the passkey was not accepted or was already used",
        _ => "the code is wrong or was already used",
    };
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "id": "*",
            "message": message,
            "code": CODE_WRONG,
        })),
    )
        .into_response()
}

pub(super) async fn verified_subject(
    state: &AppState,
    ceremony: &Ceremony,
    handle: &str,
) -> Result<Option<Subject>, ApiError> {
    let security = state.security();
    let pending = Pending {
        user_id: ceremony.user_id.clone(),
        realm: ceremony.realm.clone(),
        gate: Gate::Verify,
    };
    let stamp = ceremony
        .state
        .as_ref()
        .and_then(|state| state.get("stamp"))
        .and_then(Value::as_str);
    match subject_of(state, &pending) {
        Some(subject) if stamp == Some(security.sign_in_stamp(&subject).as_str()) => {
            Ok(Some(subject))
        }
        _ => {
            security
                .take_ceremony(handle, CeremonyKind::SignIn)
                .await
                .map_err(store_error)?;
            Ok(None)
        }
    }
}

pub(super) enum Step {
    Accepted {
        left: Option<u32>,
        offset: Option<i8>,
    },
    Wrong,
    Locked(u64),
}

async fn second_factor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SecondStep>,
) -> Result<Response, ApiError> {
    let method = match body.method.as_str() {
        TOTP => TOTP,
        RECOVERY => RECOVERY,
        PASSKEY => PASSKEY,
        _ => {
            return Err(ApiError::invalid(
                "method",
                "the method must be totp, recovery or passkey",
            ));
        }
    };
    if body.code.len() > CODE_MAX_BYTES {
        return Err(ApiError::invalid("code", "the code is too long"));
    }
    if method == PASSKEY {
        if body.ceremony.is_empty() || body.ceremony.len() > CEREMONY_MAX_BYTES {
            return Err(ApiError::invalid("ceremony", "is not valid"));
        }
        match &body.credential {
            Some(credential @ Value::Object(_))
                if credential.to_string().len() <= CREDENTIAL_MAX_BYTES => {}
            _ => {
                return Err(ApiError::invalid(
                    "credential",
                    "must be the object the authenticator returned",
                ));
            }
        }
    }
    let Some(handle) = cookie(&headers, SIGN_IN_COOKIE).filter(|handle| !handle.is_empty()) else {
        return Ok(expired(&headers));
    };
    let security = state.security();
    let Some(ceremony) = security
        .attempt_ceremony(handle, CeremonyKind::SignIn, None)
        .await
        .map_err(store_error)?
    else {
        return Ok(expired(&headers));
    };
    let user = ceremony.user_id.clone();
    let Some(subject) = verified_subject(&state, &ceremony, handle).await? else {
        return Ok(expired(&headers));
    };
    if security.sign_in_blocked(&user) {
        return Ok(refused(
            StatusCode::TOO_MANY_REQUESTS,
            "too many failed sign-ins; wait a few minutes",
        ));
    }
    let step = if method == TOTP {
        check_totp(&state, &user, &body.code).await?
    } else if method == PASSKEY {
        let credential = body.credential.unwrap_or(Value::Null);
        session_passkey::check(&state, &user, handle, &body.ceremony, &credential).await?
    } else {
        match security
            .consume_recovery_code(&user, &body.code)
            .await
            .map_err(store_error)?
        {
            RecoveryOutcome::Accepted { left } => Step::Accepted {
                left: Some(left),
                offset: None,
            },
            RecoveryOutcome::Rejected => Step::Wrong,
        }
    };
    match step {
        Step::Accepted { left, offset } => {
            let taken = security
                .take_ceremony(handle, CeremonyKind::SignIn)
                .await
                .map_err(store_error)?;
            if taken.is_none() {
                tracing::warn!(
                    user,
                    "a sign-in ceremony was consumed while its code was accepted; refused"
                );
                return Ok(expired(&headers));
            }
            security.sign_in_succeeded(&user);
            if let Some(left) = left {
                state
                    .audit(
                        &Caller(Some(subject.clone())),
                        Event::new(TWO_FACTOR_DOMAIN, "recovery.used", &user)
                            .with(json!({"userId": user, "left": left})),
                    )
                    .await;
            }
            let mut response = axum::Json(describe(&state, Some(&subject), false)).into_response();
            let opening = Opening {
                restricted: false,
                second_factor: Some(method),
                offset_steps: offset,
            };
            open(&state, &headers, &subject, &mut response, opening).await?;
            set_ceremony_cookie(&mut response, "", is_https(&headers), 0);
            Ok(response)
        }
        Step::Wrong | Step::Locked(_) => {
            security.sign_in_failed(&user);
            tracing::info!(user, method, "a second step of a sign-in failed");
            state
                .audit(
                    &Caller(None),
                    Event::new(TWO_FACTOR_DOMAIN, "verify.failed", &user)
                        .with(json!({"userId": user, "method": method})),
                )
                .await;
            if security.sign_in_blocked(&user) {
                audit_rate_limited(&state, &user, crate::client::current(), "UI").await;
            }
            if ceremony.attempts >= CEREMONY_ATTEMPTS {
                security
                    .take_ceremony(handle, CeremonyKind::SignIn)
                    .await
                    .map_err(store_error)?;
                return Ok(expired(&headers));
            }
            match step {
                Step::Locked(seconds) => Err(ApiError::TooManyAttempts(seconds)),
                _ => Ok(wrong_code(method)),
            }
        }
    }
}

async fn check_totp(state: &AppState, user: &str, code: &str) -> Result<Step, ApiError> {
    let report = state
        .security()
        .verify_totp(user, code, Utc::now())
        .await
        .map_err(store_error)?;
    if report.detail.unreadable_secret {
        state
            .audit(
                &Caller(None),
                Event::new(TWO_FACTOR_DOMAIN, "totp.unreadable", user)
                    .with(json!({"userId": user})),
            )
            .await;
    }
    Ok(match report.result {
        Verification::Accepted => Step::Accepted {
            left: None,
            offset: report.detail.offset_steps,
        },
        Verification::Rejected => Step::Wrong,
        Verification::Locked { until } => {
            if report.detail.fresh_lock {
                state
                    .audit(
                        &Caller(None),
                        Event::new(TWO_FACTOR_DOMAIN, "locked", user)
                            .with(json!({"userId": user, "until": until.to_rfc3339()})),
                    )
                    .await;
            }
            let seconds = (until.to_utc() - Utc::now()).num_seconds().max(0) + 1;
            Step::Locked(u64::try_from(seconds).unwrap_or(1))
        }
    })
}

pub(crate) async fn begin(
    state: &AppState,
    headers: &HeaderMap,
    subject: &Subject,
    response: &mut Response,
) -> Result<(), ApiError> {
    open(state, headers, subject, response, Opening::default()).await
}

async fn open(
    state: &AppState,
    headers: &HeaderMap,
    subject: &Subject,
    response: &mut Response,
    opening: Opening<'_>,
) -> Result<(), ApiError> {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let session = state
        .security()
        .open_session(
            subject,
            &client_address().unwrap_or_default(),
            user_agent,
            opening.restricted,
        )
        .await?;
    let caller = Caller(Some(subject.clone()));
    let mut attributes = json!({"principal": subject.user_id});
    if let Some(method) = opening.second_factor {
        attributes["secondFactor"] = json!(method);
    }
    if let Some(offset) = opening.offset_steps {
        attributes["offsetSteps"] = json!(offset);
    }
    state
        .audit(
            &caller,
            Event::new(&subject.realm, "login", "").with(attributes),
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

fn describe(state: &AppState, subject: Option<&Subject>, restricted: bool) -> Value {
    let Some(subject) = subject else {
        return json!({
            "authenticated": false,
            "anonymous": false,
            "user": null,
            "permissions": [],
            "secondFactor": null,
        });
    };
    let model = state.security().model();
    let user = model.users.get(&subject.user_id);
    let profile = state.security().profile(subject);
    let permissions: Vec<String> = subject
        .permissions()
        .iter()
        .map(ToString::to_string)
        .collect();
    json!({
        "authenticated": !subject.anonymous,
        "anonymous": subject.anonymous,
        "user": {
            "userId": subject.user_id,
            "firstName": profile.first_name,
            "lastName": profile.last_name,
            "emailAddress": profile.email,
            "status": user.filter(|_| subject.source == DEFAULT_SOURCE).map(|user| user.status.as_str()).unwrap_or("active"),
            "source": if subject.source.is_empty() { DEFAULT_SOURCE } else { subject.source.as_str() },
            "realm": subject.realm,
        },
        "permissions": permissions,
        "secondFactor": if subject.anonymous { Value::Null } else { json!({"restricted": restricted}) },
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

pub(super) fn refused(status: StatusCode, message: &str) -> Response {
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
    append_cookie(
        response,
        &CookieSpec {
            name,
            value,
            path: "/",
            same_site: "Lax",
            http_only,
            secure,
            max_age,
        },
    );
}

fn set_ceremony_cookie(response: &mut Response, value: &str, secure: bool, max_age: u64) {
    append_cookie(
        response,
        &CookieSpec {
            name: SIGN_IN_COOKIE,
            value,
            path: SIGN_IN_PATH,
            same_site: "Strict",
            http_only: true,
            secure,
            max_age,
        },
    );
}

struct CookieSpec<'a> {
    name: &'a str,
    value: &'a str,
    path: &'a str,
    same_site: &'a str,
    http_only: bool,
    secure: bool,
    max_age: u64,
}

fn append_cookie(response: &mut Response, spec: &CookieSpec<'_>) {
    let mut cookie = format!(
        "{}={}; Path={}; SameSite={}; Max-Age={}",
        spec.name, spec.value, spec.path, spec.same_site, spec.max_age
    );
    if spec.http_only {
        cookie.push_str("; HttpOnly");
    }
    if spec.secure {
        cookie.push_str("; Secure");
    }
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}
