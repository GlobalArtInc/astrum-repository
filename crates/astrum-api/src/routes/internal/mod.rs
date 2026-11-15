mod access_tokens;
mod audit;
pub(crate) mod session;
mod session_passkey;
pub mod ui;
mod ui_state;

use axum::Router;
use axum::http::HeaderMap;

use crate::AppState;

pub const INTERNAL: &str = "/service/rest/internal";

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(session::routes())
        .merge(session_passkey::routes())
        .merge(access_tokens::routes())
        .merge(audit::routes())
        .merge(ui::routes())
        .merge(ui_state::routes())
}

pub(crate) fn is_https(headers: &HeaderMap) -> bool {
    let header = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    header("x-forwarded-proto").is_some_and(|proto| proto.eq_ignore_ascii_case("https"))
        || header("forwarded")
            .is_some_and(|value| value.to_ascii_lowercase().contains("proto=https"))
}
