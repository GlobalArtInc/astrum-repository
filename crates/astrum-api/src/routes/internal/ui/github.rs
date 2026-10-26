use axum::Router;
use axum::extract::State;
use axum::routing::get;
use serde::Serialize;

use super::UI;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/github"), get(stars))
}

#[derive(Serialize)]
struct Github {
    stars: Option<u64>,
}

async fn stars(State(state): State<AppState>) -> axum::Json<Github> {
    axum::Json(Github {
        stars: crate::github::stars(&state).await,
    })
}
