use astrum_db::audit::Event;
use astrum_db::entity::{security_oidc_user, security_user, security_user_role};
use astrum_security::bootstrap::ADMIN_PASSWORD_FILE;
use astrum_security::catalogue::{ADMIN_USER, DEFAULT_SOURCE};
use astrum_security::realm::{self, USER_SOURCES};
use astrum_security::{SecurityModel, Subject, UserStatus, ldap, oidc, password};
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::routing::{get, put};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{SECURITY, changed, dedup, now, unique_violation, validate};
use crate::{ApiError, AppState, Caller, FieldError, Json, Validation};

const DOMAIN: &str = "security.user";
const MIN_PASSWORD: usize = 8;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{SECURITY}/users"), get(list).post(create))
        .route(
            &format!("{SECURITY}/users/{{user_id}}"),
            put(update).delete(remove),
        )
        .route(
            &format!("{SECURITY}/users/{{user_id}}/change-password"),
            put(change_password),
        )
        .route(&format!("{SECURITY}/user-sources"), get(sources))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserXo {
    user_id: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email_address: Option<String>,
    source: String,
    status: &'static str,
    read_only: bool,
    roles: Vec<String>,
    external_roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    user_id: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateUser {
    user_id: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email_address: Option<String>,
    password: Option<String>,
    status: Option<String>,
    roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateUser {
    user_id: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email_address: Option<String>,
    source: Option<String>,
    status: Option<String>,
    roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct DeleteQuery {
    realm: Option<String>,
}

#[derive(Debug, Serialize)]
struct SourceXo {
    id: &'static str,
    name: &'static str,
}

fn local(model: &SecurityModel, user: &astrum_security::User) -> UserXo {
    UserXo {
        user_id: user.id.clone(),
        first_name: Some(user.first_name.clone()),
        last_name: Some(user.last_name.clone()),
        email_address: Some(user.email.clone()),
        source: DEFAULT_SOURCE.to_owned(),
        status: user.status.as_str(),
        read_only: false,
        roles: model.roles_of(&user.id, DEFAULT_SOURCE).to_vec(),
        external_roles: Vec::new(),
    }
}

fn split_name(name: Option<&str>) -> (Option<String>, Option<String>) {
    match name.map(str::trim).filter(|name| !name.is_empty()) {
        None => (None, None),
        Some(name) => match name.split_once(' ') {
            Some((first, last)) => (Some(first.to_owned()), Some(last.trim().to_owned())),
            None => (Some(name.to_owned()), None),
        },
    }
}

fn ldap_user(model: &SecurityModel, user: &astrum_security::ldap::LdapUser) -> UserXo {
    let (first_name, last_name) = split_name(user.real_name.as_deref());
    UserXo {
        user_id: user.username.clone(),
        first_name,
        last_name,
        email_address: user.email.clone(),
        source: ldap::SOURCE.to_owned(),
        status: UserStatus::Active.as_str(),
        read_only: true,
        roles: model.roles_of(&user.username, ldap::SOURCE).to_vec(),
        external_roles: user.groups.clone(),
    }
}

fn oidc_user(model: &SecurityModel, user: &astrum_security::oidc::OidcUser) -> UserXo {
    let text = |value: &str| Some(value.to_owned()).filter(|value| !value.is_empty());
    UserXo {
        user_id: user.user_id.clone(),
        first_name: text(&user.first_name),
        last_name: text(&user.last_name),
        email_address: text(&user.email),
        source: oidc::SOURCE.to_owned(),
        status: UserStatus::Active.as_str(),
        read_only: true,
        roles: model.roles_of(&user.user_id, oidc::SOURCE).to_vec(),
        external_roles: user.groups.clone(),
    }
}

async fn external_known(state: &AppState, user_id: &str, source: &str) -> Result<bool, ApiError> {
    let model = state.security().model();
    if model
        .user_roles
        .contains_key(&(user_id.to_owned(), source.to_owned()))
    {
        return Ok(true);
    }
    match source {
        ldap::SOURCE => match state.security().ldap_user(user_id).await {
            Ok(user) => Ok(user.is_some()),
            Err(error) => Err(ApiError::BadRequest(format!(
                "LDAP cannot be asked about the user {user_id}: {error}"
            ))),
        },
        oidc::SOURCE => Ok(model.oidc_users.contains_key(user_id)),
        _ => Ok(false),
    }
}

fn external_source(source: &str) -> bool {
    source != DEFAULT_SOURCE && USER_SOURCES.iter().any(|known| known.id == source)
}

fn user_not_found(user_id: &str) -> ApiError {
    ApiError::not_found(format!("User '{user_id}' not found."))
}

fn parse_status(validation: &mut Validation, value: Option<&str>) -> Option<UserStatus> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    let Some(value) = value else {
        validation.add("status", "must not be empty");
        return None;
    };
    let status = UserStatus::parse(&value.to_ascii_lowercase());
    if status.is_none() {
        validation.add(
            "status",
            "must be one of active, locked, disabled, changepassword",
        );
    }
    status
}

fn check_email(validation: &mut Validation, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty())
        && !validate::email(value)
    {
        validation.add("emailAddress", "must be a well-formed email address");
    }
}

pub(crate) fn check_password(validation: &mut Validation, id: &str, value: &str) {
    if value.chars().count() < MIN_PASSWORD {
        validation.add(id, format!("must be at least {MIN_PASSWORD} characters"));
    }
}

fn check_roles(model: &SecurityModel, roles: &[String]) -> Result<(), ApiError> {
    let unknown: Vec<FieldError> = roles
        .iter()
        .filter(|role| !model.roles.contains_key(*role))
        .map(|role| FieldError {
            id: "roles".to_owned(),
            message: format!("the role {role} does not exist"),
        })
        .collect();
    match unknown.is_empty() {
        true => Ok(()),
        false => Err(ApiError::Invalid(unknown)),
    }
}

pub(super) fn check_grants<'a>(
    model: &SecurityModel,
    subject: &Subject,
    roles: impl IntoIterator<Item = &'a String>,
) -> Result<(), ApiError> {
    for role in roles {
        if !model.can_grant_role(subject, role) {
            return Err(ApiError::Forbidden(format!(
                "you cannot grant the role {role}: it holds permissions you do not have"
            )));
        }
    }
    Ok(())
}

async fn hash(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || password::hash(&password))
        .await
        .map_err(ApiError::internal)
}

async fn put_roles(
    db: &impl ConnectionTrait,
    user_id: &str,
    source: &str,
    roles: &[String],
) -> Result<(), ApiError> {
    security_user_role::Entity::insert(security_user_role::ActiveModel {
        user_id: ActiveValue::Set(user_id.to_owned()),
        source: ActiveValue::Set(source.to_owned()),
        roles: ActiveValue::Set(json!(roles)),
        updated_at: ActiveValue::Set(now()),
    })
    .on_conflict(
        OnConflict::columns([
            security_user_role::Column::UserId,
            security_user_role::Column::Source,
        ])
        .update_columns([
            security_user_role::Column::Roles,
            security_user_role::Column::UpdatedAt,
        ])
        .to_owned(),
    )
    .exec(db)
    .await?;
    Ok(())
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<ListQuery>,
) -> Result<axum::Json<Vec<UserXo>>, ApiError> {
    caller.require("astrum:users:read")?;
    let model = state.security().model();
    let prefix = query
        .user_id
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_lowercase();
    let wanted = |user_id: &str| user_id.to_lowercase().starts_with(&prefix);
    let source = query
        .source
        .as_deref()
        .map(str::trim)
        .filter(|source| !source.is_empty())
        .unwrap_or(DEFAULT_SOURCE);
    let users = if source == DEFAULT_SOURCE {
        model
            .users
            .values()
            .filter(|user| wanted(&user.id))
            .map(|user| local(&model, user))
            .collect()
    } else if source == ldap::SOURCE {
        let prefix = query.user_id.as_deref().map(str::trim);
        state
            .security()
            .ldap()
            .search_users(
                &model.ldap_servers,
                state.security().secrets(),
                prefix,
                ldap::SEARCH_LIMIT,
            )
            .await
            .iter()
            .map(|user| ldap_user(&model, user))
            .collect()
    } else if source == oidc::SOURCE {
        model
            .oidc_users
            .values()
            .filter(|user| wanted(&user.user_id))
            .take(ldap::SEARCH_LIMIT)
            .map(|user| oidc_user(&model, user))
            .collect()
    } else {
        Vec::new()
    };
    Ok(axum::Json(users))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<CreateUser>,
) -> Result<axum::Json<UserXo>, ApiError> {
    let subject = caller.require("astrum:users:create")?.clone();
    let mut validation = Validation::new();
    let user_id = body.user_id.unwrap_or_default();
    if let Some(problem) = validate::identifier_problem(&user_id) {
        validation.add("userId", problem);
    }
    validation.require("firstName", body.first_name.as_deref());
    validation.require("lastName", body.last_name.as_deref());
    validation.require("emailAddress", body.email_address.as_deref());
    validation.require("password", body.password.as_deref());
    let status = parse_status(&mut validation, body.status.as_deref());
    check_email(&mut validation, body.email_address.as_deref());
    let password = body.password.unwrap_or_default();
    if !password.is_empty() {
        check_password(&mut validation, "password", &password);
    }
    validation.finish()?;
    let status = status.unwrap_or(UserStatus::Active);
    let roles = dedup(body.roles.unwrap_or_default());

    let model = state.security().model();
    check_roles(&model, &roles)?;
    if model.users.contains_key(&user_id) {
        return Err(already_exists());
    }
    check_grants(&model, &subject, &roles)?;

    let password_hash = hash(password).await?;
    let txn = state.db().conn().begin().await?;
    let inserted = security_user::Entity::insert(security_user::ActiveModel {
        id: ActiveValue::Set(user_id.clone()),
        first_name: ActiveValue::Set(body.first_name.unwrap_or_default()),
        last_name: ActiveValue::Set(body.last_name.unwrap_or_default()),
        email: ActiveValue::Set(body.email_address.unwrap_or_default()),
        password_hash: ActiveValue::Set(Some(password_hash)),
        status: ActiveValue::Set(status.as_str().to_owned()),
        created_at: ActiveValue::Set(now()),
        updated_at: ActiveValue::Set(now()),
    })
    .exec(&txn)
    .await;
    match inserted {
        Err(error) if unique_violation(&error) => return Err(already_exists()),
        other => {
            other?;
        }
    }
    put_roles(&txn, &user_id, DEFAULT_SOURCE, &roles).await?;
    txn.commit().await?;

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "created", &user_id)
            .with(json!({"status": status.as_str(), "roles": roles})),
    )
    .await?;
    let model = state.security().model();
    let user = model
        .users
        .get(&user_id)
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("the new user is missing")))?;
    Ok(axum::Json(local(&model, user)))
}

fn already_exists() -> ApiError {
    ApiError::invalid("userId", "a user with this id already exists")
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(user_id): Path<String>,
    Json(body): Json<UpdateUser>,
) -> Result<StatusCode, ApiError> {
    let subject = caller.require("astrum:users:update")?.clone();
    let mut validation = Validation::new();
    validation.require("userId", body.user_id.as_deref());
    validation.require("source", body.source.as_deref());
    let status = parse_status(&mut validation, body.status.as_deref());
    check_email(&mut validation, body.email_address.as_deref());
    validation.finish()?;
    let status = status.unwrap_or(UserStatus::Active);
    if body.user_id.as_deref() != Some(user_id.as_str()) {
        return Err(ApiError::BadRequest(
            "the user id in the path does not match the body".to_owned(),
        ));
    }
    let source = body.source.unwrap_or_default();
    let roles = dedup(body.roles.unwrap_or_default());
    let model = state.security().model();
    if source != DEFAULT_SOURCE && !external_source(&source) {
        return Err(ApiError::not_found(format!(
            "the user source {source} does not exist"
        )));
    }
    let known = match source == DEFAULT_SOURCE {
        true => model.users.contains_key(&user_id),
        false => external_known(&state, &user_id, &source).await?,
    };
    if !known {
        return Err(user_not_found(&user_id));
    }
    check_roles(&model, &roles)?;
    let current = model.roles_of(&user_id, &source);
    check_grants(
        &model,
        &subject,
        roles.iter().filter(|role| !current.contains(role)),
    )?;

    let txn = state.db().conn().begin().await?;
    if source == DEFAULT_SOURCE {
        security_user::Entity::update(security_user::ActiveModel {
            id: ActiveValue::Unchanged(user_id.clone()),
            first_name: ActiveValue::Set(body.first_name.unwrap_or_default()),
            last_name: ActiveValue::Set(body.last_name.unwrap_or_default()),
            email: ActiveValue::Set(body.email_address.unwrap_or_default()),
            status: ActiveValue::Set(status.as_str().to_owned()),
            updated_at: ActiveValue::Set(now()),
            ..Default::default()
        })
        .exec(&txn)
        .await?;
    }
    put_roles(&txn, &user_id, &source, &roles).await?;
    txn.commit().await?;

    let mut attributes = json!({"source": source, "roles": roles});
    if source == DEFAULT_SOURCE {
        attributes["status"] = json!(status.as_str());
    }
    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", &user_id).with(attributes),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(user_id): Path<String>,
    Query(query): Query<DeleteQuery>,
) -> Result<StatusCode, ApiError> {
    let subject = caller.require("astrum:users:delete")?;
    let source = match query.realm.as_deref() {
        None => DEFAULT_SOURCE,
        Some(realm_id) => USER_SOURCES
            .iter()
            .find(|source| source.realm == realm_id)
            .map(|source| source.id)
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "the realm '{realm_id}' holds no users; use {}, {} or {}",
                    realm::LOCAL,
                    realm::LDAP,
                    realm::OIDC
                ))
            })?,
    };
    let model = state.security().model();
    let known = match source == DEFAULT_SOURCE {
        true => model.users.contains_key(&user_id),
        false => external_known(&state, &user_id, source).await?,
    };
    if !known {
        return Err(user_not_found(&user_id));
    }
    if subject.user_id == user_id && subject.source == source {
        return Err(ApiError::BadRequest(
            "you cannot delete the user you are signed in as".to_owned(),
        ));
    }
    if source == DEFAULT_SOURCE && model.anonymous.enabled && model.anonymous.user_id == user_id {
        return Err(ApiError::BadRequest(format!(
            "the user {user_id} serves anonymous access; turn anonymous access off or choose another user for it first"
        )));
    }

    let txn = state.db().conn().begin().await?;
    if source == DEFAULT_SOURCE {
        security_user::Entity::delete_by_id(user_id.clone())
            .exec(&txn)
            .await?;
    }
    security_user_role::Entity::delete_by_id((user_id.clone(), source.to_owned()))
        .exec(&txn)
        .await?;
    if source == oidc::SOURCE {
        security_oidc_user::Entity::delete_by_id(user_id.clone())
            .exec(&txn)
            .await?;
    }
    txn.commit().await?;
    if source == oidc::SOURCE {
        state
            .security()
            .close_sessions_in_realm(&user_id, realm::OIDC)
            .await?;
    }
    if source == DEFAULT_SOURCE {
        state.security().close_sessions_of(&user_id).await?;
        state.security().delete_tokens_of(&user_id).await?;
        state.security().revoke_format_tokens_of(&user_id).await?;
    }

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "deleted", &user_id).with(json!({"source": source})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn change_password(
    State(state): State<AppState>,
    caller: Caller,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:userschangepw:create")?;
    if !plain_text(&headers) {
        return Err(ApiError::UnsupportedMediaType("plain text (text/plain)"));
    }
    let model = state.security().model();
    let Some(user) = model.users.get(&user_id) else {
        return Err(user_not_found(&user_id));
    };
    let password = String::from_utf8(body.to_vec())
        .map_err(|_| ApiError::invalid("password", "must be UTF-8 text"))?;
    let mut validation = Validation::new();
    match password.is_empty() {
        true => validation.add("password", "must not be empty"),
        false => check_password(&mut validation, "password", &password),
    }
    validation.finish()?;

    store_password(&state, user, password).await?;

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", &user_id).with(json!({"password": "changed"})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn store_password(
    state: &AppState,
    user: &astrum_security::User,
    password: String,
) -> Result<(), ApiError> {
    let status = match user.status {
        UserStatus::Changepassword => UserStatus::Active,
        other => other,
    };
    let password_hash = hash(password).await?;
    security_user::Entity::update(security_user::ActiveModel {
        id: ActiveValue::Unchanged(user.id.clone()),
        password_hash: ActiveValue::Set(Some(password_hash)),
        status: ActiveValue::Set(status.as_str().to_owned()),
        updated_at: ActiveValue::Set(now()),
        ..Default::default()
    })
    .exec(state.db().conn())
    .await?;
    if user.id == ADMIN_USER {
        let file = state.data_dir().join(ADMIN_PASSWORD_FILE);
        match tokio::fs::remove_file(&file).await {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                tracing::warn!(%error, file = %file.display(), "the first admin password file stays")
            }
            _ => {}
        }
    }
    Ok(())
}

fn plain_text(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/plain"))
}

async fn sources(caller: Caller) -> Result<axum::Json<Vec<SourceXo>>, ApiError> {
    caller.require("astrum:users:read")?;
    Ok(axum::Json(
        USER_SOURCES
            .iter()
            .map(|source| SourceXo {
                id: source.id,
                name: source.realm,
            })
            .collect(),
    ))
}
