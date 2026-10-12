use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_security::model::{ANONYMOUS_KEY, AnonymousSettings, REALMS_KEY};
use astrum_security::realm::{self, AVAILABLE};
use astrum_security::ssrf::{Network, SSRF_KEY, SsrfProtection, valid_domain};
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{SECURITY, changed, dedup};
use crate::{ApiError, AppState, Caller, Json, Validation};

const SETTINGS_READ: &str = "astrum:settings:read";
const SETTINGS_UPDATE: &str = "astrum:settings:update";
const ADMINISTRATOR: &str = "astrum:*";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{SECURITY}/realms/available"), get(available))
        .route(
            &format!("{SECURITY}/realms/active"),
            get(active).put(activate),
        )
        .route(
            &format!("{SECURITY}/anonymous"),
            get(anonymous).put(set_anonymous),
        )
        .route(
            &format!("{SECURITY}/ssrf-protection"),
            get(ssrf).put(set_ssrf),
        )
}

#[derive(Debug, Serialize)]
struct RealmXo {
    id: &'static str,
    name: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnonymousRequest {
    enabled: Option<bool>,
    user_id: Option<String>,
    realm_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SsrfRequest {
    enabled: Option<bool>,
    #[serde(rename = "allowedIPs")]
    allowed_ips: Option<Vec<String>>,
    allowed_domains: Option<Vec<String>>,
}

async fn available(caller: Caller) -> Result<axum::Json<Vec<RealmXo>>, ApiError> {
    caller.require(SETTINGS_READ)?;
    Ok(axum::Json(
        AVAILABLE
            .iter()
            .map(|realm| RealmXo {
                id: realm.id,
                name: realm.name,
            })
            .collect(),
    ))
}

async fn active(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<String>>, ApiError> {
    caller.require(SETTINGS_READ)?;
    Ok(axum::Json(state.security().model().realms.clone()))
}

async fn activate(
    State(state): State<AppState>,
    caller: Caller,
    Json(ids): Json<Vec<String>>,
) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let unknown: Vec<&str> = ids
        .iter()
        .filter(|id| !realm::is_known(id))
        .map(String::as_str)
        .collect();
    if !unknown.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "unknown realm ids: {}",
            unknown.join(", ")
        )));
    }
    let ids = dedup(ids);
    setting::put(state.db().conn(), REALMS_KEY, &ids).await?;
    changed(
        &state,
        &caller,
        Event::new("security.realm", "updated", "active").with(json!({"active": ids})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn anonymous(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<AnonymousSettings>, ApiError> {
    caller.require(SETTINGS_READ)?;
    Ok(axum::Json(state.security().model().anonymous.clone()))
}

async fn set_anonymous(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<AnonymousRequest>,
) -> Result<axum::Json<AnonymousSettings>, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let mut validation = Validation::new();
    validation.require("userId", body.user_id.as_deref());
    validation.require("realmName", body.realm_name.as_deref());
    let realm_name = body.realm_name.unwrap_or_default();
    if !realm_name.trim().is_empty()
        && realm_name != realm::AUTHORIZING
        && !realm::is_known(&realm_name)
    {
        validation.add(
            "realmName",
            format!("the realm {realm_name} does not exist"),
        );
    }
    validation.finish()?;
    let settings = AnonymousSettings {
        enabled: body.enabled.unwrap_or(false),
        user_id: body.user_id.unwrap_or_default(),
        realm_name,
    };
    setting::put(state.db().conn(), ANONYMOUS_KEY, &settings).await?;
    changed(
        &state,
        &caller,
        Event::new("security.anonymous", "updated", "anonymous").with(json!(settings)),
    )
    .await?;
    Ok(axum::Json(settings))
}

async fn ssrf(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<SsrfProtection>, ApiError> {
    caller.require(ADMINISTRATOR)?;
    let settings = setting::get(state.db().conn(), SSRF_KEY)
        .await?
        .unwrap_or_default();
    Ok(axum::Json(settings))
}

async fn set_ssrf(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<SsrfRequest>,
) -> Result<axum::Json<SsrfProtection>, ApiError> {
    caller.require(ADMINISTRATOR)?;
    let mut validation = Validation::new();
    if body.enabled.is_none() {
        validation.add("enabled", "must not be empty");
    }
    let ips: Vec<String> = body
        .allowed_ips
        .unwrap_or_default()
        .iter()
        .map(|ip| ip.trim().to_owned())
        .collect();
    for ip in &ips {
        if Network::parse(ip).is_none() {
            validation.add(
                "allowedIPs",
                format!("{ip} is not an IP address or a network such as 10.0.0.0/8"),
            );
        }
    }
    let domains: Vec<String> = body
        .allowed_domains
        .unwrap_or_default()
        .iter()
        .map(|domain| domain.trim().to_lowercase())
        .collect();
    for domain in &domains {
        if !valid_domain(domain) {
            validation.add(
                "allowedDomains",
                format!("{domain} is not a domain name such as registry.local or *.corp.example"),
            );
        }
    }
    validation.finish()?;
    let settings = SsrfProtection {
        enabled: body.enabled.unwrap_or(true),
        allowed_ips: dedup(ips),
        allowed_domains: dedup(domains),
    };
    setting::put(state.db().conn(), SSRF_KEY, &settings).await?;
    state
        .audit(
            &caller,
            Event::new("security.ssrf", "updated", "ssrf-protection").with(json!(settings)),
        )
        .await;
    Ok(axum::Json(settings))
}
