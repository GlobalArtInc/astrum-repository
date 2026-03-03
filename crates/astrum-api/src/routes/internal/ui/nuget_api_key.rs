use astrum_db::audit::Event;
use astrum_security::api_keys::{ApiKey, NUGET};
use astrum_security::{AuthError, Subject};
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{UI, time};
use crate::routes::internal::INTERNAL;
use crate::{ApiError, AppState, Caller, Json};

const DOMAIN: &str = "security.nugetapikey";
const READ: &str = "astrum:apikey:read";
const DELETE: &str = "astrum:apikey:delete";
const TICKET_SECONDS: i64 = 120;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{UI}/user/nuget-api-key"),
            get(status).post(reveal),
        )
        .route(&format!("{UI}/user/nuget-api-key/reset"), post(reset))
        .route(
            &format!("{INTERNAL}/nuget-api-key"),
            get(ticket_reveal).delete(ticket_reset),
        )
        .route("/service/rest/wonderland/authenticate", post(authenticate))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    realm_active: bool,
    created_at: Option<String>,
    last_used_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Revealed {
    api_key: String,
    created_at: String,
}

#[derive(Deserialize)]
struct Confirmation {
    password: Option<String>,
}

fn holder<'a>(caller: &'a Caller, permission: &str) -> Result<&'a Subject, ApiError> {
    caller.signed_in()?;
    caller.require(permission)
}

async fn status(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Status>, ApiError> {
    let subject = holder(&caller, READ)?;
    let security = state.security();
    let info = security.api_key_info(NUGET, &subject.user_id).await?;
    Ok(axum::Json(Status {
        realm_active: security.model().realm_active(NUGET),
        created_at: info.as_ref().map(|info| time(info.created_at)),
        last_used_at: info.and_then(|info| info.last_used_at).map(time),
    }))
}

fn too_many() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(json!({"id": "*", "message": "too many wrong passwords; wait a few minutes"})),
    )
        .into_response()
}

async fn confirm(state: &AppState, user_id: &str, password: &str) -> Result<bool, ApiError> {
    let security = state.security();
    match security.authenticate(user_id, password).await {
        Ok(_) => {
            security.sign_in_succeeded(user_id);
            Ok(true)
        }
        Err(AuthError::BadCredentials) => {
            security.sign_in_failed(user_id);
            tracing::info!(
                user = user_id,
                "a NuGet API key request gave a wrong password"
            );
            Ok(false)
        }
        Err(AuthError::Internal(error)) => Err(error.into()),
    }
}

async fn issued(state: &AppState, caller: &Caller, key: ApiKey) -> Revealed {
    if key.created {
        state
            .audit(caller, Event::new(DOMAIN, "created", caller.initiator()))
            .await;
    }
    Revealed {
        api_key: key.key,
        created_at: time(key.created_at),
    }
}

async fn removed(state: &AppState, caller: &Caller, user_id: &str) -> Result<(), ApiError> {
    if state.security().delete_api_key(NUGET, user_id).await? {
        state
            .audit(caller, Event::new(DOMAIN, "deleted", user_id))
            .await;
    }
    Ok(())
}

async fn confirmed(
    state: &AppState,
    caller: &Caller,
    permission: &str,
    body: Confirmation,
) -> Result<Result<Subject, Response>, ApiError> {
    let subject = holder(caller, permission)?.clone();
    if state.security().sign_in_blocked(&subject.user_id) {
        return Ok(Err(too_many()));
    }
    let password = body.password.unwrap_or_default();
    if password.is_empty() {
        return Err(ApiError::invalid("password", "must not be empty"));
    }
    if !confirm(state, &subject.user_id, &password).await? {
        return Err(ApiError::invalid("password", "the password is wrong"));
    }
    Ok(Ok(subject))
}

async fn reveal(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<Confirmation>,
) -> Result<Response, ApiError> {
    let subject = match confirmed(&state, &caller, READ, body).await? {
        Ok(subject) => subject,
        Err(response) => return Ok(response),
    };
    let key = state
        .security()
        .api_key(state.secrets(), NUGET, &subject.user_id)
        .await?;
    Ok(axum::Json(issued(&state, &caller, key).await).into_response())
}

async fn reset(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<Confirmation>,
) -> Result<Response, ApiError> {
    let subject = match confirmed(&state, &caller, DELETE, body).await? {
        Ok(subject) => subject,
        Err(response) => return Ok(response),
    };
    removed(&state, &caller, &subject.user_id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
struct Credentials {
    u: Option<String>,
    p: Option<String>,
}

fn decode(value: Option<&str>) -> Option<String> {
    STANDARD
        .decode(value?.trim())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

async fn authenticate(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    Json(body): Json<Credentials>,
) -> Result<Response, ApiError> {
    let subject = caller.signed_in()?.clone();
    let (Some(user), Some(password)) = (decode(body.u.as_deref()), decode(body.p.as_deref()))
    else {
        return Err(ApiError::BadRequest(
            "the user name and password must be base64 encoded".to_owned(),
        ));
    };
    if user != subject.user_id {
        return Err(ApiError::Forbidden(
            "a ticket is issued only for the signed-in user".to_owned(),
        ));
    }
    if state.security().sign_in_blocked(&user) {
        return Ok(too_many());
    }
    if !confirm(&state, &user, &password).await? {
        return Err(ApiError::Forbidden("the password is wrong".to_owned()));
    }
    let expires = Utc::now().timestamp() + TICKET_SECONDS;
    let ticket = state
        .secrets()
        .seal(&json!({"user": user, "expires": expires}).to_string());
    let wants_json = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("json"));
    Ok(match wants_json {
        true => axum::Json(json!({"t": ticket})).into_response(),
        false => (
            [(header::CONTENT_TYPE, "application/xml;charset=UTF-8")],
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><authTicket><t>{ticket}</t></authTicket>"#
            ),
        )
            .into_response(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TicketQuery {
    auth_token: Option<String>,
}

fn ticket_holder(state: &AppState, caller: &Caller, query: &TicketQuery) -> Result<(), ApiError> {
    let subject = caller.signed_in()?;
    let token = query
        .auth_token
        .as_deref()
        .filter(|token| !token.is_empty())
        .ok_or_else(|| ApiError::Forbidden("Missing authentication ticket".to_owned()))?;
    let invalid = || ApiError::Forbidden("Invalid authentication ticket".to_owned());
    let sealed = decode(Some(token)).ok_or_else(invalid)?;
    let opened = state.secrets().open(&sealed).map_err(|_| invalid())?;
    let ticket: serde_json::Value = serde_json::from_str(&opened).map_err(|_| invalid())?;
    let user = ticket.get("user").and_then(|value| value.as_str());
    let expires = ticket.get("expires").and_then(|value| value.as_i64());
    match (user, expires) {
        (Some(user), Some(expires))
            if user == subject.user_id && expires > Utc::now().timestamp() =>
        {
            Ok(())
        }
        _ => Err(invalid()),
    }
}

async fn ticket_reveal(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<TicketQuery>,
) -> Result<Response, ApiError> {
    ticket_holder(&state, &caller, &query)?;
    let subject = caller.require(READ)?;
    let key = state
        .security()
        .api_key(state.secrets(), NUGET, &subject.user_id)
        .await?;
    let revealed = issued(&state, &caller, key).await;
    Ok(axum::Json(json!({"apiKey": revealed.api_key})).into_response())
}

async fn ticket_reset(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<TicketQuery>,
) -> Result<StatusCode, ApiError> {
    ticket_holder(&state, &caller, &query)?;
    let subject = caller.require(DELETE)?;
    removed(&state, &caller, &subject.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
