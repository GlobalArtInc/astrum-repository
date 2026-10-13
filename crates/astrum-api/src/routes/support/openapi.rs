use std::sync::LazyLock;

use axum::Router;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::AppState;

pub const PATH: &str = "/service/rest/swagger.json";

static DOCUMENT: LazyLock<String> = LazyLock::new(|| {
    let mut document: serde_json::Value =
        serde_json::from_str(include_str!("openapi.json")).expect("openapi.json is valid JSON");
    document["info"]["version"] = env!("CARGO_PKG_VERSION").into();
    serde_json::to_string(&document).expect("a JSON value serializes")
});

pub fn routes() -> Router<AppState> {
    Router::new().route(PATH, get(document))
}

async fn document() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        DOCUMENT.as_str(),
    )
}
