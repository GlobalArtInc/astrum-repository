use astrum_db::audit::Event;
use astrum_security::realm;
use astrum_security::two_factor::passkey_store::{
    BeginPasskey, PasskeyEnrollment, PasskeyRecord, PasskeyRemoval, passkey_name,
};
use astrum_security::two_factor::passkeys::RelyingParty;
use astrum_security::two_factor::store::StoreError;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{patch, post};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::two_factor::{
    CEREMONY_MAX_BYTES, DOMAIN, PasswordBody, confirm_password, owner, private, settle, store_error,
};
use super::{UI, time};
use crate::auth::client_address;
use crate::{ApiError, AppState, Caller, Json, Validation};

const CREDENTIAL_MAX_BYTES: usize = 16 * 1024;
const NO_RELYING_PARTY: &str = "passkeys are not available on this server: it has no usable base URL or relying party id; ask an administrator to set the base URL";
const TURNED_OFF: &str = "passkeys are turned off on this server";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{UI}/user/two-factor/passkeys/options"),
            post(begin),
        )
        .route(&format!("{UI}/user/two-factor/passkeys"), post(register))
        .route(
            &format!("{UI}/user/two-factor/passkeys/{{id}}"),
            patch(rename).delete(remove),
        )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BeginBody {
    #[serde(alias = "password")]
    current_password: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterBody {
    ceremony: Option<String>,
    credential: Option<Value>,
    #[serde(alias = "password")]
    current_password: Option<String>,
}

#[derive(Deserialize)]
struct RenameBody {
    name: Option<String>,
}

pub(super) fn passkey_json(passkey: &PasskeyRecord, current_rp_id: Option<&str>) -> Value {
    json!({
        "id": passkey.id,
        "name": passkey.name,
        "createdAt": time(passkey.created_at),
        "lastUsedAt": passkey.last_used_at.map(time),
        "backedUp": passkey.backed_up,
        "usable": current_rp_id == Some(passkey.rp_id.as_str()),
    })
}

fn passkey_id(raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw).map_err(|_| ApiError::not_found("there is no such passkey"))
}

pub(crate) fn resolve_relying_party(state: &AppState) -> Result<RelyingParty, ApiError> {
    state.security().model().relying_party().map_err(|error| {
        tracing::warn!(%error, "a passkey was requested but the relying party cannot be resolved");
        ApiError::Conflict(NO_RELYING_PARTY.to_owned())
    })
}

fn enrolment_relying_party(state: &AppState) -> Result<RelyingParty, ApiError> {
    if !state.security().model().two_factor.passkeys {
        return Err(ApiError::BadRequest(TURNED_OFF.to_owned()));
    }
    resolve_relying_party(state)
}

async fn begin(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<BeginBody>,
) -> Result<Response, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    let name = body.name.unwrap_or_default();
    passkey_name(&name).map_err(store_error)?;
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
    let rp = enrolment_relying_party(&state)?;
    let profile = state.security().profile(subject);
    let display_name = format!("{} {}", profile.first_name, profile.last_name);
    let display_name = match display_name.trim() {
        "" => user_id.as_str(),
        shown => shown,
    };
    let pending = state
        .security()
        .begin_passkey(
            &rp,
            BeginPasskey {
                user_id: &user_id,
                realm: realm::LOCAL,
                display_name,
                name: &name,
                client_ip: &client_address().unwrap_or_default(),
            },
        )
        .await
        .map_err(store_error)?;
    Ok(private(
        axum::Json(json!({
            "ceremony": pending.ceremony,
            "publicKey": pending.public_key,
            "expiresAt": time(pending.expires_at),
        }))
        .into_response(),
    ))
}

async fn register(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<RegisterBody>,
) -> Result<Response, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    let ceremony = body.ceremony.unwrap_or_default();
    let mut validation = Validation::new();
    validation.check(!ceremony.is_empty(), "ceremony", "must not be empty");
    validation.check(
        ceremony.len() <= CEREMONY_MAX_BYTES,
        "ceremony",
        "is not valid",
    );
    match &body.credential {
        Some(credential @ Value::Object(_)) => validation.check(
            credential.to_string().len() <= CREDENTIAL_MAX_BYTES,
            "credential",
            "is too large",
        ),
        _ => validation.add(
            "credential",
            "must be the object the authenticator returned",
        ),
    }
    validation.finish()?;
    let credential = body.credential.unwrap_or(Value::Null);
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
    let rp = enrolment_relying_party(&state)?;
    let outcome = state
        .security()
        .finish_passkey(&rp, &user_id, &ceremony, &credential)
        .await
        .map_err(store_error)?;
    match outcome {
        PasskeyEnrollment::Enrolled {
            passkey,
            recovery_codes,
        } => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "passkey.enrolled", &user_id).with(json!({
                        "userId": user_id,
                        "passkeyId": passkey.id,
                        "backedUp": passkey.backed_up,
                    })),
                )
                .await;
            let summary = passkey_json(&passkey, Some(rp.id()));
            let Some(codes) = recovery_codes else {
                return Ok(axum::Json(json!({"passkey": summary})).into_response());
            };
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "recovery.generated", &user_id)
                        .with(json!({"userId": user_id})),
                )
                .await;
            settle(&state, &user_id, &headers).await;
            Ok(private(
                axum::Json(json!({"passkey": summary, "recoveryCodes": codes.as_slice()}))
                    .into_response(),
            ))
        }
        PasskeyEnrollment::Rejected => Err(ApiError::invalid(
            "credential",
            "the passkey was not accepted; start again",
        )),
        PasskeyEnrollment::Expired => Err(ApiError::invalid(
            "ceremony",
            "the setup expired or is not yours; start again",
        )),
        PasskeyEnrollment::Duplicate => Err(ApiError::invalid(
            "credential",
            "this passkey is already registered",
        )),
        PasskeyEnrollment::LimitReached => Err(store_error(StoreError::TooManyPasskeys)),
    }
}

async fn rename(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Json(body): Json<RenameBody>,
) -> Result<StatusCode, ApiError> {
    let subject = owner(&caller)?;
    let user_id = subject.user_id.clone();
    let id = passkey_id(&id)?;
    let name = body.name.unwrap_or_default();
    let renamed = state
        .security()
        .rename_passkey(&user_id, id, &name)
        .await
        .map_err(store_error)?;
    if !renamed {
        return Err(ApiError::not_found("there is no such passkey"));
    }
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "passkey.renamed", &user_id)
                .with(json!({"userId": user_id, "passkeyId": id})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    uri: Uri,
    headers: HeaderMap,
    Path(id): Path<String>,
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
    let id = passkey_id(&id)?;
    match state
        .security()
        .remove_passkey(&user_id, id)
        .await
        .map_err(store_error)?
    {
        PasskeyRemoval::Removed => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "passkey.removed", &user_id)
                        .with(json!({"userId": user_id, "passkeyId": id})),
                )
                .await;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        PasskeyRemoval::LastFactorRequired => Err(ApiError::BadRequest(
            "a second factor is required for this account; add another before removing this one"
                .to_owned(),
        )),
        PasskeyRemoval::NotFound => Err(ApiError::not_found("there is no such passkey")),
    }
}
