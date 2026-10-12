mod access;
mod account;
mod audit;
mod blobstores;
mod browse;
mod browse_asset;
mod dashboard;
pub(crate) mod identity;
mod privileges;
mod repositories;
mod search;
mod selectors;
mod versions;

use axum::Router;
use chrono::{DateTime, FixedOffset, SecondsFormat};

use crate::AppState;

pub const UI: &str = "/service/rest/internal/ui";

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(repositories::routes())
        .merge(browse::routes())
        .merge(browse_asset::routes())
        .merge(search::routes())
        .merge(dashboard::routes())
        .merge(blobstores::routes())
        .merge(selectors::routes())
        .merge(account::routes())
        .merge(privileges::routes())
        .merge(audit::routes())
        .merge(identity::routes())
}

fn time(value: DateTime<FixedOffset>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, false)
}
