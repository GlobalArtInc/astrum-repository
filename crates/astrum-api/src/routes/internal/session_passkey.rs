use astrum_db::audit::Event;
use astrum_security::two_factor::passkey_sign_in::SignInOutcome;
use astrum_security::two_factor::store::{Ceremony, CeremonyKind};
use astrum_security::{Gate, Pending, Subject};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::Deserialize;
use serde_json::{Value, json};

use super::INTERNAL;
use super::session::{
    CODE_WRONG, SIGN_IN_COOKIE, SIGN_IN_LIFETIME, Step, TWO_FACTOR_DOMAIN, expired, refused,
    store_error, subject_of,
};
use super::ui::two_factor_passkeys::resolve_relying_party;
use crate::auth::{client_address, cookie};
use crate::{ApiError, AppState, Caller};

const CEREMONY_MAX_BYTES: usize = 128;
const CREDENTIAL_MAX_BYTES: usize = 16 * 1024;
pub(super) const PASSKEY: &str = "passkey";
const UNUSABLE: &str = "passkeys-unusable";

pub(super) fn routes() -> Router<AppState> {
    Router::new().route(
        &format!("{INTERNAL}/session/second-factor/passkey-options"),
        post(options),
    )
}

pub(super) fn unusable(user: &str) -> Response {
    tracing::warn!(
        user,
        "a signed-in user has no second factor method that works"
    );
    (
        StatusCode::FORBIDDEN,
        axum::Json(json!({
            "id": "*",
            "message": "none of your passkeys works on this server any more and you have no recovery codes left; ask an administrator to reset your second factor",
            "code": UNUSABLE,
        })),
    )
        .into_response()
}

pub(super) fn wrong() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "id": "*",
            "message": "the passkey was not accepted or was already used",
            "code": CODE_WRONG,
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub(super) struct Fields {
    #[serde(default)]
    ceremony: String,
    #[serde(default)]
    credential: Option<Value>,
}

impl Fields {
    pub(super) fn validate(&self) -> Result<(), ApiError> {
        if self.ceremony.is_empty() || self.ceremony.len() > CEREMONY_MAX_BYTES {
            return Err(ApiError::invalid("ceremony", "is not valid"));
        }
        match &self.credential {
            Some(credential @ Value::Object(_))
                if credential.to_string().len() <= CREDENTIAL_MAX_BYTES =>
            {
                Ok(())
            }
            _ => Err(ApiError::invalid(
                "credential",
                "must be the object the authenticator returned",
            )),
        }
    }

    pub(super) async fn check(
        &self,
        state: &AppState,
        user: &str,
        session: &str,
    ) -> Result<Step, ApiError> {
        let rp = resolve_relying_party(state)?;
        let credential = self.credential.as_ref().unwrap_or(&Value::Null);
        let outcome = state
            .security()
            .finish_passkey_sign_in(&rp, user, &self.ceremony, session, credential)
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

async fn options(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, ApiError> {
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
    security.sign_in_failed(&ceremony.user_id);
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
