use astrum_db::audit::Event;
use astrum_security::two_factor::passkey_sign_in::SignInOutcome;
use astrum_security::two_factor::store::CeremonyKind;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use super::session::{
    SIGN_IN_COOKIE, SIGN_IN_LIFETIME, Step, TWO_FACTOR_DOMAIN, expired, refused, store_error,
    verified_subject,
};
use super::ui::two_factor_passkeys::resolve_relying_party;
use crate::auth::{client_address, cookie};
use crate::{ApiError, AppState, Caller};

pub(super) async fn usable(state: &AppState, user_id: &str) -> Result<bool, ApiError> {
    let Ok(rp) = state.security().model().relying_party() else {
        return Ok(false);
    };
    let held = state
        .security()
        .list_passkeys(user_id)
        .await
        .map_err(store_error)?;
    Ok(held.iter().any(|passkey| passkey.rp_id == rp.id()))
}

pub(super) async fn options(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let Some(handle) = cookie(&headers, SIGN_IN_COOKIE).filter(|handle| !handle.is_empty()) else {
        return Ok(expired(&headers));
    };
    let security = state.security();
    let Some(ceremony) = security
        .peek_ceremony(handle, CeremonyKind::SignIn)
        .await
        .map_err(store_error)?
    else {
        return Ok(expired(&headers));
    };
    if verified_subject(&state, &ceremony, handle).await?.is_none() {
        return Ok(expired(&headers));
    }
    if security.sign_in_blocked(&ceremony.user_id) {
        return Ok(refused(
            StatusCode::TOO_MANY_REQUESTS,
            "too many failed sign-ins; wait a few minutes",
        ));
    }
    let rp = resolve_relying_party(&state)?;
    let pending = security
        .begin_passkey_sign_in(
            &rp,
            &ceremony.user_id,
            &ceremony.realm,
            handle,
            &client_address().unwrap_or_default(),
            SIGN_IN_LIFETIME,
        )
        .await
        .map_err(store_error)?;
    let Some(pending) = pending else {
        return Err(ApiError::BadRequest(
            "no passkey of this account can be used to sign in here".to_owned(),
        ));
    };
    let mut response = axum::Json(json!({
        "ceremony": pending.ceremony,
        "publicKey": pending.public_key,
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

pub(super) async fn check(
    state: &AppState,
    user: &str,
    session: &str,
    ceremony: &str,
    credential: &Value,
) -> Result<Step, ApiError> {
    let rp = resolve_relying_party(state)?;
    let outcome = state
        .security()
        .finish_passkey_sign_in(&rp, user, ceremony, session, credential)
        .await
        .map_err(store_error)?;
    Ok(match outcome {
        SignInOutcome::Accepted { .. } => Step::Accepted {
            left: None,
            offset: None,
        },
        SignInOutcome::CounterRegression { passkey } => {
            tracing::warn!(
                user,
                "a passkey presented a signature counter that did not increase"
            );
            state
                .audit(
                    &Caller(None),
                    Event::new(TWO_FACTOR_DOMAIN, "passkey.counter", user)
                        .with(json!({"userId": user, "passkeyId": passkey})),
                )
                .await;
            Step::Wrong
        }
        SignInOutcome::Rejected => Step::Wrong,
    })
}
