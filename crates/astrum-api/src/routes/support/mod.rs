pub mod health;
mod logging;
mod logs;
pub mod openapi;
pub mod sysinfo;
mod zip;

use axum::Router;

use crate::AppState;

pub use logging::sync as sync_logging;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(logs::routes())
        .merge(logging::routes())
        .merge(health::routes())
        .merge(sysinfo::routes())
        .merge(zip::routes())
        .merge(openapi::routes())
}
