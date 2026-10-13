use std::collections::BTreeSet;

use astrum_security::bootstrap::ADMIN_PASSWORD_FILE;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::{PrivilegeKind, Subject, realm};
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use serde::Serialize;

use super::INTERNAL;
use crate::routes::repositories::BaseUrl;
use crate::{AppState, Caller};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REVISION: Option<&str> = option_env!("ASTRUM_GIT_REVISION");
const EVERYTHING: &str = "astrum:*";

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{INTERNAL}/ui/state"), get(ui_state))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UiState {
    version: &'static str,
    revision: Option<&'static str>,
    user: Option<UiUser>,
    anonymous_access: bool,
    permissions: Vec<String>,
    formats: Vec<UiFormat>,
    base_url: String,
    session_timeout_minutes: u32,
    admin_password_file: Option<String>,
    oidc_enabled: bool,
    default_role: Option<UiRole>,
    nuget_api_key_realm: bool,
    ui_settings: UiSettingsXo,
    branding: BrandingXo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UiSettingsXo {
    title: String,
    debug_allowed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrandingXo {
    header_html: Option<String>,
    footer_html: Option<String>,
}

async fn look(state: &AppState, base_url: &str) -> (UiSettingsXo, BrandingXo) {
    use astrum_db::setting;
    use astrum_db::system::{BRANDING_KEY, Branding, UI_SETTINGS_KEY, UiSettings};
    let db = state.db().conn();
    let ui = setting::get::<UiSettings>(db, UI_SETTINGS_KEY)
        .await
        .ok()
        .flatten()
        .filter(|ui| ui.enabled)
        .unwrap_or_default();
    let branding = setting::get::<Branding>(db, BRANDING_KEY)
        .await
        .ok()
        .flatten()
        .filter(|branding| branding.enabled);
    let snippet = |shown: bool, html: &str| {
        (shown && !html.trim().is_empty()).then(|| html.replace("$baseUrl", base_url))
    };
    (
        UiSettingsXo {
            title: ui.title,
            debug_allowed: ui.debug_allowed,
        },
        BrandingXo {
            header_html: branding
                .as_ref()
                .and_then(|branding| snippet(branding.header_enabled, &branding.header_html)),
            footer_html: branding
                .as_ref()
                .and_then(|branding| snippet(branding.footer_enabled, &branding.footer_html)),
        },
    )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UiRole {
    id: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UiUser {
    user_id: String,
    name: String,
    email: String,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UiFormat {
    id: &'static str,
    api_name: &'static str,
    types: Vec<&'static str>,
}

async fn ui_state(
    State(state): State<AppState>,
    caller: Caller,
    BaseUrl(base_url): BaseUrl,
) -> axum::Json<UiState> {
    let security = state.security();
    let model = security.model();
    let subject = caller.subject();
    let signed_in = subject.filter(|subject| !subject.anonymous);
    let password_file = state.data_dir().join(ADMIN_PASSWORD_FILE);
    let may_see_password_file = signed_in.is_none_or(|subject| subject.is_permitted(EVERYTHING));
    let admin_password_file = (may_see_password_file
        && tokio::fs::try_exists(&password_file).await.unwrap_or(false))
    .then(|| password_file.display().to_string());
    let (ui_settings, branding) = look(&state, &base_url).await;
    axum::Json(UiState {
        version: VERSION,
        revision: REVISION.filter(|revision| !revision.is_empty()),
        user: signed_in.map(|subject| user(security, subject)),
        anonymous_access: security.anonymous().is_some(),
        permissions: subject.map(permissions).unwrap_or_default(),
        formats: formats(&state),
        base_url,
        session_timeout_minutes: model.session_timeout_minutes,
        admin_password_file,
        oidc_enabled: model.oidc.is_some() && model.realm_active(realm::OIDC),
        default_role: signed_in
            .and_then(|_| model.effective_default_role())
            .and_then(|id| model.roles.get(id))
            .map(|role| UiRole {
                id: role.id.clone(),
                name: role.name.clone(),
            }),
        nuget_api_key_realm: model.realm_active(astrum_security::api_keys::NUGET),
        ui_settings,
        branding,
    })
}

fn user(security: &astrum_security::Security, subject: &Subject) -> UiUser {
    let profile = security.profile(subject);
    let name = format!("{} {}", profile.first_name.trim(), profile.last_name.trim())
        .trim()
        .to_owned();
    let name = match name.is_empty() {
        true => subject.user_id.clone(),
        false => name,
    };
    UiUser {
        user_id: subject.user_id.clone(),
        name,
        email: profile.email,
        source: match subject.source.is_empty() {
            true => DEFAULT_SOURCE.to_owned(),
            false => subject.source.clone(),
        },
    }
}

fn permissions(subject: &Subject) -> Vec<String> {
    let granted = subject.permissions().iter().map(ToString::to_string);
    let selectors = subject.selectors().iter().filter_map(|grant| {
        PrivilegeKind::RepositoryContentSelector {
            content_selector: grant.selector.clone(),
            format: grant.format.clone(),
            repository: grant.repository.clone(),
            actions: grant.actions.clone(),
        }
        .effective_permission()
        .map(|permission| permission.to_string())
    });
    granted
        .chain(selectors)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn formats(state: &AppState) -> Vec<UiFormat> {
    state
        .content()
        .formats()
        .iter()
        .filter_map(|served| {
            astrum_repo::FORMATS
                .iter()
                .find(|known| known.id == served.id())
        })
        .map(|format| UiFormat {
            id: format.id,
            api_name: format.api_name,
            types: format.types.iter().map(|kind| kind.as_str()).collect(),
        })
        .collect()
}
