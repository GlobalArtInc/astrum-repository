mod access;
mod account;
mod audit;
mod blobstores;
mod browse;
mod browse_asset;
mod cleanup_policies;
mod domains;
pub(crate) mod identity;
pub mod nodes;
mod notifications;
mod nuget_api_key;
mod privileges;
mod repositories;
mod search;
mod selectors;
mod tasks;
mod usage;
mod versions;

use axum::Router;
use chrono::{DateTime, FixedOffset, SecondsFormat};

use crate::AppState;

pub const UI: &str = "/service/rest/internal/ui";

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(repositories::routes())
        .merge(domains::routes())
        .merge(browse::routes())
        .merge(browse_asset::routes())
        .merge(search::routes())
        .merge(usage::routes())
        .merge(blobstores::routes())
        .merge(selectors::routes())
        .merge(account::routes())
        .merge(privileges::routes())
        .merge(audit::routes())
        .merge(identity::routes())
        .merge(nuget_api_key::routes())
        .merge(tasks::routes())
        .merge(cleanup_policies::routes())
        .merge(notifications::routes())
        .merge(nodes::routes())
}

fn time(value: DateTime<FixedOffset>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, false)
}
