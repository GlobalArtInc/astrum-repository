use astrum_blob::config::{Settings, StoreConfig};
use astrum_blob::remote::MIN_TOKEN_LENGTH;
use astrum_blob::{BlobStore, RemoteSettings, SoftQuota, StoreKind};
use astrum_db::audit::Event;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    CHECK_TIMEOUT, CREATE, DOMAIN, QuotaInput, READ, UPDATE, check_name, check_quota,
    ensure_unique, find, insert, internal, refuse_type_change, save,
};
use crate::{ApiError, AppState, Caller, Json, Validation};

const URL: &str = "url";
const TOKEN: &str = "token";
const CERTIFICATE: &str = "certificate";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RemoteRequest {
    name: Option<String>,
    url: Option<String>,
    token: Option<String>,
    certificate: Option<String>,
    soft_quota: Option<QuotaInput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RemoteModel {
    name: String,
    #[serde(rename = "type")]
    kind: &'static str,
    soft_quota: Option<SoftQuota>,
    url: String,
    certificate: Option<String>,
}

pub(super) async fn create(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<RemoteRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(CREATE)?;
    let Json(body) = body?;
    let mut validation = Validation::new();
    let name = check_name(&mut validation, body.name.as_deref());
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    let settings = settings(
        &mut validation,
        &state,
        body.url,
        body.token,
        body.certificate,
        None,
    );
    validation.finish()?;
    ensure_unique(&state, &name).await?;
    check_url(&state, &settings, None).await?;
    let config = StoreConfig {
        name,
        settings: Settings::Remote(settings),
        soft_quota,
    };
    connect(&state, &config).await?;
    insert(&state, &config).await?;
    state.blob_stores().changed().await.map_err(internal)?;
    state.audit(&caller, audit_event("created", &config)).await;
    Ok(StatusCode::CREATED)
}

pub(super) async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<RemoteModel>, ApiError> {
    caller.require(READ)?;
    let config = state
        .blob_stores()
        .config(&name)
        .await
        .map_err(internal)?
        .filter(|config| config.kind() == StoreKind::Remote)
        .ok_or_else(|| {
            ApiError::not_found(format!("There is no remote blob store named '{name}'"))
        })?;
    let remote = config.remote().cloned().unwrap_or_default();
    Ok(axum::Json(RemoteModel {
        name: config.name,
        kind: StoreKind::Remote.as_str(),
        soft_quota: config.soft_quota,
        url: remote.url,
        certificate: remote.certificate,
    }))
}

pub(super) async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<Json<RemoteRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let Json(body) = body?;
    let current = find(&state, &name).await?;
    refuse_type_change(&current, StoreKind::Remote)?;
    let mut validation = Validation::new();
    if let Some(other) = body.name.as_deref().map(str::trim)
        && !other.is_empty()
        && other != current.name
    {
        validation.add("PARAMETER name", "the name of a blob store cannot change");
    }
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    let settings = settings(
        &mut validation,
        &state,
        body.url,
        body.token,
        body.certificate,
        current.remote(),
    );
    validation.finish()?;
    check_url(&state, &settings, current.remote()).await?;
    let config = StoreConfig {
        name: current.name.clone(),
        settings: Settings::Remote(settings),
        soft_quota,
    };
    connect(&state, &config).await?;
    save(&state, &config).await?;
    state.audit(&caller, audit_event("updated", &config)).await;
    Ok(StatusCode::NO_CONTENT)
}

fn settings(
    validation: &mut Validation,
    state: &AppState,
    url: Option<String>,
    token: Option<String>,
    certificate: Option<String>,
    current: Option<&RemoteSettings>,
) -> RemoteSettings {
    let url = optional(url)
        .map(|url| url.trim_end_matches('/').to_owned())
        .unwrap_or_default();
    match url::Url::parse(&url) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") && parsed.host().is_some() => {
            if parsed.path() != "/" || parsed.query().is_some() || !parsed.username().is_empty() {
                validation.add(
                    URL,
                    "must be the address of the agent alone, such as https://storage1:8090",
                );
            }
        }
        _ => validation.add(URL, "must be an http:// or https:// address"),
    }
    let token = match optional(token) {
        Some(token) if token.chars().count() < MIN_TOKEN_LENGTH => {
            validation.add(
                TOKEN,
                format!("must be at least {MIN_TOKEN_LENGTH} characters long"),
            );
            None
        }
        Some(token) => Some(state.secrets().seal(&token)),
        None => {
            let kept = current
                .filter(|current| current.url == url)
                .and_then(|current| current.token.clone());
            if kept.is_none() {
                validation.add(
                    TOKEN,
                    "must not be empty; a new address needs the token again",
                );
            }
            kept
        }
    };
    let certificate = optional(certificate);
    if let Some(pem) = &certificate
        && !pem.contains("-----BEGIN CERTIFICATE-----")
    {
        validation.add(CERTIFICATE, "must be one or more PEM certificates");
    }
    RemoteSettings {
        url,
        token,
        certificate,
    }
}

async fn check_url(
    state: &AppState,
    settings: &RemoteSettings,
    stored: Option<&RemoteSettings>,
) -> Result<(), ApiError> {
    if stored.is_some_and(|stored| stored.url == settings.url) {
        return Ok(());
    }
    match crate::routes::security::refused_destination(state, &settings.url).await? {
        Some(message) => Err(ApiError::invalid(URL, message)),
        None => Ok(()),
    }
}

async fn connect(state: &AppState, config: &StoreConfig) -> Result<(), ApiError> {
    let Some(settings) = config.remote() else {
        return Ok(());
    };
    let store = state
        .blob_stores()
        .open_remote(&config.name, settings)
        .map_err(|error| ApiError::invalid(URL, format!("The agent cannot be used: {error}")))?;
    match tokio::time::timeout(CHECK_TIMEOUT, store.check()).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(ApiError::invalid(
            URL,
            format!("The agent cannot be used: {error}"),
        )),
        Err(_) => Err(ApiError::invalid(URL, "The agent did not answer in time")),
    }
}

fn audit_event(kind: &str, config: &StoreConfig) -> Event {
    let url = config
        .remote()
        .map(|settings| settings.url.clone())
        .unwrap_or_default();
    Event::new(DOMAIN, kind, &config.name).with(json!({"type": "Remote", "url": url}))
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
