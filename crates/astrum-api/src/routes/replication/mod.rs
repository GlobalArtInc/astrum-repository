mod check;
mod config;
pub mod validate;

use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    config::routes().merge(check::routes())
}
