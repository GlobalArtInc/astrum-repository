use std::collections::BTreeSet;

use astrum_security::bootstrap::ADMIN_PASSWORD_FILE;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::{PrivilegeKind, SecurityModel, Subject};
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
    axum::Json(UiState {
        version: VERSION,
        revision: REVISION.filter(|revision| !revision.is_empty()),
        user: signed_in.map(|subject| user(&model, subject)),
        anonymous_access: security.anonymous().is_some(),
        permissions: subject.map(permissions).unwrap_or_default(),
        formats: formats(&state),
        base_url,
        session_timeout_minutes: model.session_timeout_minutes,
        admin_password_file,
    })
}

fn user(model: &SecurityModel, subject: &Subject) -> UiUser {
    let stored = model.users.get(&subject.user_id);
    let name = stored
        .map(|user| format!("{} {}", user.first_name.trim(), user.last_name.trim()))
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| subject.user_id.clone());
    UiUser {
        user_id: subject.user_id.clone(),
        name,
        email: stored.map(|user| user.email.clone()).unwrap_or_default(),
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
