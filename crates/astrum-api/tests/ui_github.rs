mod support;
mod ui_support;

use axum::http::StatusCode;
use serde_json::json;
use support::Auth;
use ui_support::*;

const GITHUB: &str = "/service/rest/internal/ui/github";

#[tokio::test]
async fn the_star_count_comes_from_the_cache_and_needs_no_sign_in() {
    let fixture = ready().await;
    fixture.state.stars().remember(321);
    let reply = get(&fixture, GITHUB, Auth::None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json(), json!({"stars": 321}));
    let signed_in = get(&fixture, GITHUB, Auth::Admin).await;
    assert_eq!(signed_in.json(), json!({"stars": 321}));
}

#[tokio::test]
async fn a_failed_lookup_leaves_the_number_out() {
    let fixture = ready().await;
    fixture.state.stars().failed();
    let reply = get(&fixture, GITHUB, Auth::None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json(), json!({"stars": null}));
}
