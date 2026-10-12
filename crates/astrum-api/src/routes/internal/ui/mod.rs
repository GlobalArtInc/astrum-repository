mod account;

use axum::Router;

use crate::AppState;

pub const UI: &str = "/service/rest/internal/ui";

pub fn routes() -> Router<AppState> {
    Router::new().merge(account::routes())
}
