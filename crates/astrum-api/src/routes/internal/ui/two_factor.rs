use astrum_db::audit::Event;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::realm;
use astrum_security::two_factor::FAILURE_WRONG_PASSWORD;
use astrum_security::two_factor::store::{Confirmation, RemoveOutcome, StoreError};
use astrum_security::{AuthError, Subject};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

use super::{UI, time};
use crate::auth::{RestrictedSession, SESSION_COOKIE, client_address, cookie};
use crate::routes::internal::session;
use crate::{ApiError, AppState, Caller, Json, Validation};

const DOMAIN: &str = "security.twofactor";
const CODE_MAX_BYTES: usize = 64;
const CEREMONY_MAX_BYTES: usize = 128;
const PRODUCT: &str = "Astrum";
const TOTP: &str = "totp";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/user/two-factor"), get(status))
        .route(
            &format!("{UI}/user/two-factor/totp"),
            post(begin_totp).delete(remove_totp),
        )
        .route(
            &format!("{UI}/user/two-factor/totp/confirm"),
            post(confirm_totp),
        )
        .route(
            &format!("{UI}/user/two-factor/recovery-codes"),
            post(regenerate_codes),
        )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordBody {
    #[serde(alias = "password")]
    current_password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfirmBody {
    ceremony: Option<String>,
    code: Option<String>,
    #[serde(alias = "password")]
    current_password: Option<String>,
}

fn owner(caller: &Caller) -> Result<&Subject, ApiError> {
    let subject = caller.signed_in()?;
    let source = match subject.source.is_empty() {
        true => DEFAULT_SOURCE,
        false => subject.source.as_str(),
    };
    if subject.realm == realm::LOCAL && source == DEFAULT_SOURCE {
        return Ok(subject);
    }
    if subject.realm == realm::LDAP || subject.realm == realm::OIDC || source != DEFAULT_SOURCE {
        return Err(ApiError::BadRequest(format!(
            "the account comes from {source}; its second factor is managed there"
        )));
    }
    Err(ApiError::Forbidden(
        "access tokens and API keys cannot manage a second factor; sign in to the web interface"
            .to_owned(),
    ))
}

fn store_error(error: StoreError) -> ApiError {
    match error {
        StoreError::AlreadyEnrolled => ApiError::BadRequest(
            "an authenticator app is already set up; remove it before adding another".to_owned(),
        ),
        StoreError::NotEnrolled => {
            ApiError::BadRequest("no second factor is set up for this account".to_owned())
        }
        other => ApiError::internal(other),
    }
}

fn throttled() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(json!({"id": "*", "message": "too many wrong passwords; wait a few minutes"})),
    )
        .into_response()
}

async fn confirm_password(
    state: &AppState,
    user_id: &str,
    password: Option<String>,
    path: &str,
    headers: &HeaderMap,
) -> Result<Option<Response>, ApiError> {
    let security = state.security();
    if security.sign_in_blocked(user_id) {
        return Ok(Some(throttled()));
    }
    let password = password.unwrap_or_default();
    let mut validation = Validation::new();
    validation.check(!password.is_empty(), "currentPassword", "must not be empty");
    validation.finish()?;
    let right = match security.authenticate(user_id, &password).await {
        Ok(subject) => subject.realm == realm::LOCAL,
        Err(AuthError::SecondFactor(_)) => true,
        Err(AuthError::BadCredentials) => false,
        Err(AuthError::Internal(error)) => return Err(error.into()),
    };
    if right {
        security.sign_in_succeeded(user_id);
        return Ok(None);
    }
    security.sign_in_failed(user_id);
    tracing::info!(
        user = user_id,
        "a second factor change gave a wrong current password"
    );
    session::audit_failed_sign_in(state, user_id, path, headers, FAILURE_WRONG_PASSWORD).await;
    if security.sign_in_blocked(user_id) {
        session::audit_rate_limited(state, user_id, crate::client::current(), "UI").await;
    }
    Err(ApiError::invalid(
        "currentPassword",
        "the current password is wrong",
    ))
}

fn private(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn issuer(state: &AppState) -> String {
    let host = state
        .configured_base_url()
        .await
        .and_then(|base| url::Url::parse(&base).ok())
        .and_then(|base| base.host_str().map(str::to_owned));
    match host {
        Some(host) => format!("{PRODUCT} ({host})"),
        None => PRODUCT.to_owned(),
    }
}

async fn status(
    State(state): State<AppState>,
    caller: Caller,
    restricted: Option<axum::Extension<RestrictedSession>>,
) -> Result<axum::Json<Value>, ApiError> {
    let subject = owner(&caller)?;
    let security = state.security();
    let overview = security
        .summary(&subject.user_id)
        .await
        .map_err(store_error)?;
    let model = security.model();
    let standing = model.second_factor_standing(&subject.user_id);
    let now = Utc::now().fixed_offset();
    let tokens = security
        .tokens(Some(&subject.user_id))
        .await?
        .iter()
        .filter(|token| token.expires_at.is_none_or(|expires| expires > now))
        .count();
    let mut methods = Vec::new();
    if model.two_factor.totp {
        methods.push(TOTP);
    }
    Ok(axum::Json(json!({
        "enrolled": overview.enrolled(),
        "totp": overview.totp,
        "required": standing.required,
        "requirementSource": overview.requirement.as_str(),
        "enrollBy": standing
            .enroll_by
            .filter(|_| !overview.enrolled())
            .map(|deadline| time(deadline.fixed_offset())),
        "restricted": restricted.is_some(),
        "recoveryCodesLeft": overview.recovery_codes_left,
        "lockedUntil": overview.locked_until.map(time),
        "accessTokens": tokens,
        "methods": methods,
    })))
}

async fn begin_totp(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<PasswordBody>,
) -> Result<Response, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    if let Some(refusal) = confirm_password(
        &state,
        &user_id,
        body.current_password,
        uri.path(),
        &headers,
    )
    .await?
    {
        return Ok(refusal);
    }
    if !state.security().model().two_factor.totp {
        return Err(ApiError::BadRequest(
            "authenticator apps are turned off on this server".to_owned(),
        ));
    }
    let pending = state
        .security()
        .begin_totp(
            &user_id,
            realm::LOCAL,
            &issuer(&state).await,
            &user_id,
            &client_address().unwrap_or_default(),
        )
        .await
        .map_err(store_error)?;
    Ok(private(
        axum::Json(json!({
            "ceremony": pending.ceremony,
            "secret": pending.secret,
            "otpauthUri": pending.otpauth_uri,
            "qrSvg": pending.qr_svg,
            "expiresAt": time(pending.expires_at),
        }))
        .into_response(),
    ))
}

fn drift_message(seconds_off: i64) -> String {
    let minutes = seconds_off.unsigned_abs().div_ceil(60).max(1);
    format!(
        "the clock of your device is about {minutes} minute{} off; set it to automatic time and enter a new code",
        if minutes == 1 { "" } else { "s" }
    )
}

async fn confirm_totp(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<ConfirmBody>,
) -> Result<Response, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    let ceremony = body.ceremony.unwrap_or_default();
    let code = body.code.unwrap_or_default();
    let mut validation = Validation::new();
    validation.check(!ceremony.is_empty(), "ceremony", "must not be empty");
    validation.check(
        ceremony.len() <= CEREMONY_MAX_BYTES,
        "ceremony",
        "is not valid",
    );
    validation.check(!code.trim().is_empty(), "code", "must not be empty");
    validation.check(code.len() <= CODE_MAX_BYTES, "code", "is too long");
    validation.finish()?;
    if let Some(refusal) = confirm_password(
        &state,
        &user_id,
        body.current_password,
        uri.path(),
        &headers,
    )
    .await?
    {
        return Ok(refusal);
    }
    let security = state.security();
    if !security.model().two_factor.totp {
        return Err(ApiError::BadRequest(
            "authenticator apps are turned off on this server".to_owned(),
        ));
    }
    let outcome = security
        .confirm_totp(&user_id, &ceremony, &code, Utc::now())
        .await
        .map_err(store_error)?;
    match outcome {
        Confirmation::Enrolled {
            recovery_codes,
            offset_steps,
        } => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "totp.enrolled", &user_id)
                        .with(json!({"userId": user_id, "offsetSteps": offset_steps})),
                )
                .await;
            let Some(codes) = recovery_codes else {
                return Ok(axum::Json(json!({})).into_response());
            };
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "recovery.generated", &user_id)
                        .with(json!({"userId": user_id})),
                )
                .await;
            let keep = match headers.contains_key(header::AUTHORIZATION) {
                true => None,
                false => cookie(&headers, SESSION_COOKIE),
            };
            if let Err(error) = security.settle_after_enrollment(&user_id, keep).await {
                tracing::error!(%error, user = %user_id, "closing the other sessions after enrolment failed");
            }
            Ok(private(
                axum::Json(json!({"recoveryCodes": codes.as_slice()})).into_response(),
            ))
        }
        Confirmation::ClockDrift { seconds_off } => {
            Err(ApiError::invalid("code", drift_message(seconds_off)))
        }
        Confirmation::Rejected => Err(ApiError::invalid("code", "the code is wrong")),
        Confirmation::Expired => Err(ApiError::invalid(
            "ceremony",
            "the setup expired or is not yours; start again",
        )),
    }
}

async fn remove_totp(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<PasswordBody>,
) -> Result<Response, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    if let Some(refusal) = confirm_password(
        &state,
        &user_id,
        body.current_password,
        uri.path(),
        &headers,
    )
    .await?
    {
        return Ok(refusal);
    }
    match state
        .security()
        .remove_totp(&user_id)
        .await
        .map_err(store_error)?
    {
        RemoveOutcome::Removed => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "totp.removed", &user_id).with(json!({"userId": user_id})),
                )
                .await;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        RemoveOutcome::LastFactorRequired => Err(ApiError::BadRequest(
            "a second factor is required for this account; add another before removing this one"
                .to_owned(),
        )),
        RemoveOutcome::NotEnrolled => Err(ApiError::BadRequest(
            "no authenticator app is set up for this account".to_owned(),
        )),
    }
}

async fn regenerate_codes(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<PasswordBody>,
) -> Result<Response, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    if let Some(refusal) = confirm_password(
        &state,
        &user_id,
        body.current_password,
        uri.path(),
        &headers,
    )
    .await?
    {
        return Ok(refusal);
    }
    let codes = state
        .security()
        .regenerate_recovery_codes(&user_id)
        .await
        .map_err(store_error)?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "recovery.regenerated", &user_id).with(json!({"userId": user_id})),
        )
        .await;
    Ok(private(
        axum::Json(json!({"recoveryCodes": codes.as_slice()})).into_response(),
    ))
}
