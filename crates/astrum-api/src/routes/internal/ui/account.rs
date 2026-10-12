use astrum_db::audit::Event;
use astrum_db::entity::security_user;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::{AuthError, Scope, Subject};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use sea_orm::{ActiveValue, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::UI;
use crate::auth::{SESSION_COOKIE, cookie};
use crate::routes::security::users::{check_password, store_password};
use crate::routes::security::validate;
use crate::{ApiError, AppState, Caller, Json, Validation};

const DOMAIN: &str = "security.user";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/user"), get(read).put(update))
        .route(&format!("{UI}/user/password"), put(change_password))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Account {
    user_id: String,
    first_name: String,
    last_name: String,
    email: String,
    source: String,
    realm: String,
    status: &'static str,
    roles: Vec<String>,
    read_only: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountUpdate {
    first_name: Option<String>,
    last_name: Option<String>,
    #[serde(alias = "emailAddress")]
    email: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordChange {
    current_password: Option<String>,
    new_password: Option<String>,
}

fn source(subject: &Subject) -> &str {
    match subject.source.is_empty() {
        true => DEFAULT_SOURCE,
        false => subject.source.as_str(),
    }
}

fn account(state: &AppState, subject: &Subject) -> Account {
    let model = state.security().model();
    let source = source(subject);
    let local = model
        .users
        .get(&subject.user_id)
        .filter(|_| source == DEFAULT_SOURCE);
    Account {
        user_id: subject.user_id.clone(),
        first_name: local
            .map(|user| user.first_name.clone())
            .unwrap_or_default(),
        last_name: local.map(|user| user.last_name.clone()).unwrap_or_default(),
        email: local.map(|user| user.email.clone()).unwrap_or_default(),
        source: source.to_owned(),
        realm: subject.realm.clone(),
        status: local.map(|user| user.status.as_str()).unwrap_or("active"),
        roles: model.roles_of(&subject.user_id, source).to_vec(),
        read_only: local.is_none(),
    }
}

fn writable(caller: &Caller) -> Result<Subject, ApiError> {
    let subject = caller.signed_in()?.clone();
    if subject.scope == Scope::Read {
        return Err(ApiError::Forbidden(
            "a read-only access token cannot change the account".to_owned(),
        ));
    }
    if source(&subject) != DEFAULT_SOURCE {
        return Err(ApiError::BadRequest(format!(
            "the account comes from {} and is managed there",
            source(&subject)
        )));
    }
    Ok(subject)
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Account>, ApiError> {
    let subject = caller.signed_in()?;
    Ok(axum::Json(account(&state, subject)))
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<AccountUpdate>,
) -> Result<axum::Json<Account>, ApiError> {
    let subject = writable(&caller)?;
    let mut validation = Validation::new();
    validation.require("firstName", body.first_name.as_deref());
    validation.require("lastName", body.last_name.as_deref());
    validation.require("email", body.email.as_deref());
    if let Some(email) = body.email.as_deref().filter(|email| !email.is_empty())
        && !validate::email(email)
    {
        validation.add("email", "must be a well-formed email address");
    }
    validation.finish()?;
    if !state
        .security()
        .model()
        .users
        .contains_key(&subject.user_id)
    {
        return Err(ApiError::not_found(format!(
            "User '{}' not found.",
            subject.user_id
        )));
    }
    security_user::Entity::update(security_user::ActiveModel {
        id: ActiveValue::Unchanged(subject.user_id.clone()),
        first_name: ActiveValue::Set(body.first_name.unwrap_or_default().trim().to_owned()),
        last_name: ActiveValue::Set(body.last_name.unwrap_or_default().trim().to_owned()),
        email: ActiveValue::Set(body.email.unwrap_or_default().trim().to_owned()),
        updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    })
    .exec(state.db().conn())
    .await?;
    state.security().reload().await?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &subject.user_id)
                .with(json!({"source": DEFAULT_SOURCE, "account": true})),
        )
        .await;
    Ok(axum::Json(account(&state, &subject)))
}

async fn change_password(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    Json(body): Json<PasswordChange>,
) -> Result<Response, ApiError> {
    let subject = writable(&caller)?;
    let user_id = subject.user_id.clone();
    let security = state.security();
    if security.sign_in_blocked(&user_id) {
        return Ok((
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(
                json!({"id": "*", "message": "too many wrong passwords; wait a few minutes"}),
            ),
        )
            .into_response());
    }
    let current = body.current_password.unwrap_or_default();
    let new = body.new_password.unwrap_or_default();
    let mut validation = Validation::new();
    validation.check(!current.is_empty(), "currentPassword", "must not be empty");
    match new.is_empty() {
        true => validation.add("newPassword", "must not be empty"),
        false => check_password(&mut validation, "newPassword", &new),
    }
    validation.finish()?;
    match security.authenticate(&user_id, &current).await {
        Ok(_) => security.sign_in_succeeded(&user_id),
        Err(AuthError::BadCredentials) => {
            security.sign_in_failed(&user_id);
            tracing::info!(
                user = user_id,
                "a password change gave a wrong current password"
            );
            return Err(ApiError::invalid(
                "currentPassword",
                "the current password is wrong",
            ));
        }
        Err(AuthError::Internal(error)) => return Err(error.into()),
    }
    let model = security.model();
    let user = model
        .users
        .get(&user_id)
        .ok_or_else(|| ApiError::not_found(format!("User '{user_id}' not found.")))?;
    store_password(&state, user, new).await?;
    security.reload().await?;
    let keep = match headers.contains_key(header::AUTHORIZATION) {
        true => None,
        false => cookie(&headers, SESSION_COOKIE),
    };
    security.keep_only_session(&user_id, keep).await?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &user_id)
                .with(json!({"password": "changed", "account": true})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
