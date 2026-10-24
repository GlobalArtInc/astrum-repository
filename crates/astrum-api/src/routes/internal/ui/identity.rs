use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_security::model::{DEFAULT_ROLE_KEY, DefaultRole, REALMS_KEY};
use astrum_security::oidc::{self, OidcSettings, SecretChange, View};
use astrum_security::realm;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::UI;
use crate::routes::repositories::BaseUrl;
use crate::routes::security::roles::{RoleXo, Source, external_groups, group_role};
use crate::{ApiError, AppState, Caller, FieldError, Json};

const SETTINGS_READ: &str = "astrum:settings:read";
const SETTINGS_UPDATE: &str = "astrum:settings:update";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/roles"), get(roles))
        .route(&format!("{UI}/roles/sources"), get(sources))
        .route(
            &format!("{UI}/oauth2"),
            get(read_oidc).put(save_oidc).delete(remove_oidc),
        )
        .route(
            &format!("{UI}/default-role"),
            get(read_default_role)
                .put(save_default_role)
                .delete(remove_default_role),
        )
}

#[derive(Debug, Serialize)]
struct SourceXo {
    id: &'static str,
    name: &'static str,
}

async fn sources(caller: Caller) -> Result<axum::Json<Vec<SourceXo>>, ApiError> {
    caller.require("astrum:roles:read")?;
    Ok(axum::Json(vec![
        SourceXo {
            id: astrum_security::ldap::SOURCE,
            name: "LDAP",
        },
        SourceXo {
            id: oidc::SOURCE,
            name: "OpenID Connect",
        },
    ]))
}

#[derive(Debug, Deserialize)]
struct RoleSearch {
    source: Option<String>,
    search: Option<String>,
}

async fn roles(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<RoleSearch>,
) -> Result<axum::Json<Vec<RoleXo>>, ApiError> {
    caller.require("astrum:roles:read")?;
    let source = Source::parse(query.source.as_deref())?;
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|search| !search.is_empty());
    if source != Source::Local {
        return Ok(axum::Json(
            external_groups(&state, source, search)
                .await
                .iter()
                .map(|group| group_role(group, source))
                .collect(),
        ));
    }
    let model = state.security().model();
    let wanted = search.map(str::to_lowercase);
    Ok(axum::Json(
        model
            .roles
            .values()
            .filter(|role| {
                wanted.as_deref().is_none_or(|wanted| {
                    role.id.to_lowercase().contains(wanted)
                        || role.name.to_lowercase().contains(wanted)
                })
            })
            .map(RoleXo::from)
            .collect(),
    ))
}

fn redirect_uri(base_url: &str) -> String {
    format!("{base_url}{}", oidc::CALLBACK_PATH)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OidcXo {
    #[serde(flatten)]
    view: View,
    enabled: bool,
}

async fn read_oidc(
    State(state): State<AppState>,
    caller: Caller,
    BaseUrl(base_url): BaseUrl,
) -> Result<axum::Json<OidcXo>, ApiError> {
    caller.require(SETTINGS_READ)?;
    let model = state.security().model();
    Ok(axum::Json(OidcXo {
        view: View::of(model.oidc.as_ref(), redirect_uri(&base_url)),
        enabled: model.oidc.is_some() && model.realm_active(realm::OIDC),
    }))
}

pub(crate) async fn http(state: &AppState) -> Result<oidc::Http, ApiError> {
    let ssrf: SsrfProtection = setting::get(state.db().conn(), SSRF_KEY)
        .await?
        .unwrap_or_default();
    oidc::Http::new(&ssrf).map_err(|error| ApiError::internal(anyhow::anyhow!(error.message)))
}

async fn activate(
    state: &AppState,
    realm_id: &str,
    on: bool,
) -> Result<Option<Vec<String>>, ApiError> {
    let mut realms = state.security().model().realms.clone();
    let active = realms.iter().any(|id| id == realm_id);
    if active == on {
        return Ok(None);
    }
    match on {
        true => realms.push(realm_id.to_owned()),
        false => realms.retain(|id| id != realm_id),
    }
    setting::put(state.db().conn(), REALMS_KEY, &realms).await?;
    Ok(Some(realms))
}

async fn realm_changed(state: &AppState, caller: &Caller, realms: Option<Vec<String>>) {
    if let Some(realms) = realms {
        state
            .audit(
                caller,
                Event::new("security.realm", "updated", "active").with(json!({"active": realms})),
            )
            .await;
    }
}

async fn save_oidc(
    State(state): State<AppState>,
    caller: Caller,
    Json(draft): Json<Option<oidc::Draft>>,
) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let draft =
        draft.ok_or_else(|| ApiError::BadRequest("The configuration is missing".to_owned()))?;
    let (mut settings, secret) = oidc::config::check(&draft).map_err(|problems| {
        ApiError::Invalid(
            problems
                .into_iter()
                .map(|problem| FieldError {
                    id: problem.field.to_owned(),
                    message: problem.message,
                })
                .collect(),
        )
    })?;
    let http = http(&state).await?;
    if let Err(error) = oidc::OidcProvider::resolve(&settings, &http).await {
        let message = format!(
            "Could not load the identity provider configuration: {}",
            error.message
        );
        return Err(match settings.issuer_url.is_some() {
            true => ApiError::invalid("issuerUrl", message),
            false => ApiError::BadRequest(message),
        });
    }
    let previous = state.security().model().oidc.clone();
    settings.client_secret = match secret {
        SecretChange::Keep => previous
            .as_ref()
            .and_then(|stored| stored.client_secret.clone()),
        SecretChange::Remove => None,
        SecretChange::Set(value) => Some(state.secrets().seal(&value)),
    };
    setting::put(state.db().conn(), oidc::SETTINGS_KEY, &settings).await?;
    let realms = activate(&state, realm::OIDC, true).await?;
    state.security().changed().await?;
    state.security().oidc().reset();
    state
        .audit(
            &caller,
            Event::new(
                "security.oidc",
                match previous {
                    Some(_) => "updated",
                    None => "created",
                },
                "oidc",
            )
            .with(describe(&settings)),
        )
        .await;
    realm_changed(&state, &caller, realms).await;
    Ok(StatusCode::NO_CONTENT)
}

fn describe(settings: &OidcSettings) -> serde_json::Value {
    json!({
        "issuerUrl": settings.issuer_url,
        "clientId": settings.client_id,
        "clientSecret": settings.client_secret.is_some(),
        "groupsClaim": settings.groups_claim,
        "usernameClaim": settings.username_claim,
    })
}

async fn remove_oidc(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    if !setting::delete(state.db().conn(), oidc::SETTINGS_KEY).await? {
        return Err(ApiError::not_found("OpenID Connect is not configured"));
    }
    let realms = activate(&state, realm::OIDC, false).await?;
    state.security().changed().await?;
    state.security().oidc().reset();
    state
        .audit(&caller, Event::new("security.oidc", "deleted", "oidc"))
        .await;
    realm_changed(&state, &caller, realms).await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DefaultRoleXo {
    role: Option<String>,
    enabled: bool,
    missing: bool,
}

async fn read_default_role(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<DefaultRoleXo>, ApiError> {
    caller.require(SETTINGS_READ)?;
    let model = state.security().model();
    Ok(axum::Json(DefaultRoleXo {
        role: model.default_role.clone(),
        enabled: model.default_role.is_some() && model.realm_active(realm::DEFAULT_ROLE),
        missing: model
            .default_role
            .as_ref()
            .is_some_and(|role| !model.roles.contains_key(role)),
    }))
}

#[derive(Debug, Deserialize)]
struct DefaultRoleRequest {
    role: Option<String>,
    enabled: Option<bool>,
}

async fn save_default_role(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<DefaultRoleRequest>,
) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let role = body.role.unwrap_or_default().trim().to_owned();
    if role.is_empty() {
        return Err(ApiError::invalid("role", "must not be empty"));
    }
    let model = state.security().model();
    if !model.roles.contains_key(&role) {
        return Err(ApiError::invalid(
            "role",
            format!("the role {role} does not exist"),
        ));
    }
    let subject = caller.require(SETTINGS_UPDATE)?;
    if !model.can_grant_role(subject, &role) {
        return Err(ApiError::Forbidden(format!(
            "you cannot grant the role {role}: it holds permissions you do not have"
        )));
    }
    setting::put(
        state.db().conn(),
        DEFAULT_ROLE_KEY,
        &DefaultRole { role: role.clone() },
    )
    .await?;
    let enabled = body.enabled.unwrap_or(true);
    let realms = activate(&state, realm::DEFAULT_ROLE, enabled).await?;
    state.security().changed().await?;
    state
        .audit(
            &caller,
            Event::new("security.defaultrole", "updated", &role)
                .with(json!({"role": role, "enabled": enabled})),
        )
        .await;
    realm_changed(&state, &caller, realms).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_default_role(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let previous = state.security().model().default_role.clone();
    setting::delete(state.db().conn(), DEFAULT_ROLE_KEY).await?;
    let realms = activate(&state, realm::DEFAULT_ROLE, false).await?;
    state.security().changed().await?;
    state
        .audit(
            &caller,
            Event::new(
                "security.defaultrole",
                "deleted",
                previous.unwrap_or_default(),
            ),
        )
        .await;
    realm_changed(&state, &caller, realms).await;
    Ok(StatusCode::NO_CONTENT)
}
