mod check;
mod config;
mod receive;
mod status;
pub mod validate;

use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    config::routes()
        .merge(check::routes())
        .merge(receive::routes())
        .merge(status::routes())
}
