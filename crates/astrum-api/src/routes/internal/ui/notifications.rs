use astrum_db::audit::Event;
use astrum_notify::channel::{self, Channel, ChannelType, Draft, Invalid, Proxy, Validated};
use astrum_notify::{Secrets, store};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::{Value, json};
use uuid::Uuid;

use super::UI;
use crate::error::ApiError;
use crate::routes::repositories::BaseUrl;
use crate::{AppState, Caller};

const READ: &str = "astrum:settings:read";
const UPDATE: &str = "astrum:settings:update";
const DOMAIN: &str = "system.notifications";
const LOCK: i64 = i64::from_be_bytes(*b"astrnchn");

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/notifications"), get(list).post(create))
        .route(&format!("{UI}/notifications/test"), post(test_unsaved))
        .route(
            &format!("{UI}/notifications/{{id}}"),
            put(update).delete(remove),
        )
        .route(&format!("{UI}/notifications/{{id}}/test"), post(test_saved))
}

fn xo(channel: &Channel) -> Value {
    let (proxy_type, proxy_host, proxy_port, proxy_username) = match &channel.proxy {
        Proxy::Custom {
            kind,
            host,
            port,
            username,
        } => (json!(kind.id()), json!(host), json!(port), json!(username)),
        _ => (Value::Null, Value::Null, Value::Null, Value::Null),
    };
    json!({
        "id": channel.id,
        "name": channel.name,
        "type": channel.kind.id(),
        "enabled": channel.enabled,
        "chatId": channel.chat_id,
        "messageThreadId": channel.message_thread_id,
        "recipients": channel.recipients,
        "target": channel.target(),
        "repositories": channel.repositories,
        "events": channel.events.iter().map(|event| event.id()).collect::<Vec<_>>(),
        "language": channel.language,
        "proxyMode": channel.proxy.mode(),
        "proxyType": proxy_type,
        "proxyHost": proxy_host,
        "proxyPort": proxy_port,
        "proxyUsername": proxy_username,
        "hasSecret": channel.sealed.secret.is_some(),
        "hasProxyPassword": channel.sealed.proxy_password.is_some(),
        "hasSigningSecret": channel.sealed.signing_secret.is_some(),
        "hasAuthorization": channel.sealed.authorization.is_some(),
    })
}

fn audit_event(kind: &str, channel: &Channel) -> Event {
    let mut attributes = json!({
        "id": channel.id,
        "name": channel.name,
        "type": channel.kind.id(),
        "enabled": channel.enabled.to_string(),
        "target": channel.target(),
        "repositories": channel.repositories.join(","),
        "events": channel.events.iter().map(|event| event.id()).collect::<Vec<_>>().join(","),
        "language": channel.language,
        "proxyMode": channel.proxy.mode(),
    });
    if let Proxy::Custom {
        kind: proxy_kind,
        host,
        port,
        ..
    } = &channel.proxy
    {
        attributes["proxyType"] = json!(proxy_kind.id());
        attributes["proxyHost"] = json!(host);
        attributes["proxyPort"] = json!(port.to_string());
    }
    Event::new(DOMAIN, kind, &channel.name).with(attributes)
}

fn invalid(error: Invalid) -> ApiError {
    ApiError::invalid(error.field, error.message)
}

fn not_found() -> ApiError {
    ApiError::not_found("Notification channel not found")
}

fn parse_id(id: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|_| not_found())
}

async fn list(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let channels = store::list(state.db().conn()).await?;
    Ok(Json(Value::Array(channels.iter().map(xo).collect())))
}

async fn check_repositories(state: &AppState, channel: &Channel) -> Result<(), ApiError> {
    if channel.repositories.is_empty() {
        return Ok(());
    }
    let rows = state
        .db()
        .conn()
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT name FROM repository WHERE name = ANY($1)",
            [channel.repositories.clone().into()],
        ))
        .await?;
    let known: Vec<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String>("", "name").ok())
        .collect();
    match channel
        .repositories
        .iter()
        .find(|name| !known.contains(name))
    {
        Some(missing) => Err(ApiError::invalid(
            "repositories",
            format!("Repository {missing} does not exist"),
        )),
        None => Ok(()),
    }
}

async fn save(
    state: &AppState,
    caller: &Caller,
    draft: Draft,
    id: Option<Uuid>,
) -> Result<Json<Value>, ApiError> {
    caller.require(UPDATE)?;
    let txn = state.db().conn().begin().await?;
    txn.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({LOCK})"))
        .await?;
    let existing = match id {
        Some(id) => Some(store::find(&txn, id).await?.ok_or_else(not_found)?),
        None => None,
    };
    let Validated {
        mut channel,
        changes,
    } = channel::validate(&draft, existing.as_ref(), false).map_err(invalid)?;
    check_repositories(state, &channel).await?;
    let others = store::list(&txn).await?;
    if others.iter().any(|other| {
        other.name.to_lowercase() == channel.name.to_lowercase() && Some(other.id) != id
    }) {
        return Err(ApiError::invalid(
            "name",
            "A channel with this name already exists",
        ));
    }
    let before = existing
        .as_ref()
        .map(|existing| existing.sealed.clone())
        .unwrap_or_default();
    channel.sealed = store::sealed(&before, &changes, state.secrets());
    match existing {
        Some(_) => store::update(&txn, &channel).await?,
        None => store::insert(&txn, &channel).await?,
    }
    txn.commit().await?;
    let kind = if id.is_some() { "updated" } else { "created" };
    state.audit(caller, audit_event(kind, &channel)).await;
    Ok(Json(xo(&channel)))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    crate::Json(draft): crate::Json<Draft>,
) -> Result<Json<Value>, ApiError> {
    save(&state, &caller, draft, None).await
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    crate::Json(draft): crate::Json<Draft>,
) -> Result<Json<Value>, ApiError> {
    caller.require(UPDATE)?;
    let id = parse_id(&id)?;
    save(&state, &caller, draft, Some(id)).await
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let id = parse_id(&id)?;
    let channel = store::find(state.db().conn(), id)
        .await?
        .ok_or_else(not_found)?;
    if !store::delete(state.db().conn(), id).await? {
        return Err(not_found());
    }
    state.audit(&caller, audit_event("deleted", &channel)).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn base_url(state: &AppState, request: String) -> String {
    state.notifier().base_url().await.unwrap_or(request)
}

async fn send_test(
    state: &AppState,
    channel: &Channel,
    secrets: &Secrets,
    request: String,
) -> Json<Value> {
    let base = base_url(state, request).await;
    match state
        .notifier()
        .send_test(channel, secrets, Some(&base))
        .await
    {
        Ok(()) => Json(json!({"success": true, "message": null})),
        Err(error) => Json(json!({"success": false, "message": error.message})),
    }
}

fn secret_required(channel: &Channel) -> ApiError {
    ApiError::invalid(
        "secret",
        match channel.kind {
            ChannelType::Telegram => "Bot token is required",
            _ => "Webhook URL is required",
        },
    )
}

async fn test_saved(
    State(state): State<AppState>,
    caller: Caller,
    BaseUrl(request): BaseUrl,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(UPDATE)?;
    let id = parse_id(&id)?;
    let channel = store::find(state.db().conn(), id)
        .await?
        .ok_or_else(not_found)?;
    let secrets = channel.open(state.secrets()).map_err(ApiError::internal)?;
    if channel.kind != ChannelType::Email && secrets.secret.is_none() {
        return Err(secret_required(&channel));
    }
    Ok(send_test(&state, &channel, &secrets, request).await)
}

async fn test_unsaved(
    State(state): State<AppState>,
    caller: Caller,
    BaseUrl(request): BaseUrl,
    crate::Json(draft): crate::Json<Draft>,
) -> Result<Json<Value>, ApiError> {
    caller.require(UPDATE)?;
    let existing = match draft.id.as_deref().filter(|id| !id.is_empty()) {
        Some(id) => match Uuid::parse_str(id) {
            Ok(id) => store::find(state.db().conn(), id).await?,
            Err(_) => None,
        },
        None => None,
    };
    let Validated { channel, changes } =
        channel::validate(&draft, existing.as_ref(), true).map_err(invalid)?;
    let saved = match &existing {
        Some(existing) => existing.open(state.secrets()).map_err(ApiError::internal)?,
        None => Secrets::default(),
    };
    let pick = |change: &channel::Change, saved: &Option<String>| match change {
        channel::Change::Keep => saved.clone(),
        channel::Change::Set(value) => Some(value.clone()),
        channel::Change::Clear => None,
    };
    let secrets = Secrets {
        secret: pick(&changes.secret, &saved.secret),
        proxy_password: pick(&changes.proxy_password, &saved.proxy_password),
        signing_secret: pick(&changes.signing_secret, &saved.signing_secret),
        authorization: pick(&changes.authorization, &saved.authorization),
    };
    if channel.kind != ChannelType::Email && secrets.secret.is_none() {
        return Err(secret_required(&channel));
    }
    Ok(send_test(&state, &channel, &secrets, request).await)
}
