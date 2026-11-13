mod support;
mod ui_support;

use axum::http::StatusCode;
use chrono::{Days, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture};
use ui_support::*;

const REST: &str = "/service/rest/v1/repositories";
const UI: &str = "/service/rest/internal/ui/usage-history";

async fn seed(fixture: &Fixture, repository: &str, days_ago: u64, bytes: i64) {
    let day = Utc::now()
        .date_naive()
        .checked_sub_days(Days::new(days_ago))
        .unwrap();
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "INSERT INTO repository_usage_day (repository, day, components, assets, bytes, downloads) \
                 VALUES ('{repository}', '{day}', 1, 2, {bytes}, 3)"
            ),
        ))
        .await
        .unwrap();
}

fn bytes_of(points: &Value) -> Vec<i64> {
    points
        .as_array()
        .unwrap()
        .iter()
        .map(|point| point["bytes"].as_i64().unwrap())
        .collect()
}

#[tokio::test]
async fn the_history_lists_the_days_in_order_within_the_window() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    seed(&fixture, "files", 200, 10).await;
    seed(&fixture, "files", 40, 20).await;
    seed(&fixture, "files", 3, 30).await;
    seed(&fixture, "files", 1, 40).await;
    let url = format!("{REST}/files/usage/history");

    let reply = get(&fixture, &url, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(bytes_of(&reply.json()), [20, 30, 40]);
    assert_eq!(
        reply.json()[0]["day"],
        Utc::now()
            .date_naive()
            .checked_sub_days(Days::new(40))
            .unwrap()
            .to_string()
    );
    assert_eq!(
        bytes_of(
            &get(&fixture, &format!("{url}?days=30"), Auth::Admin)
                .await
                .json()
        ),
        [30, 40]
    );
    assert_eq!(
        bytes_of(
            &get(&fixture, &format!("{url}?days=730"), Auth::Admin)
                .await
                .json()
        ),
        [10, 20, 30, 40]
    );
}

#[tokio::test]
async fn days_outside_one_to_730_are_rejected_with_the_field() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    for value in ["0", "731"] {
        let reply = get(
            &fixture,
            &format!("{REST}/files/usage/history?days={value}"),
            Auth::Admin,
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{value}");
        assert_eq!(reply.json()[0]["id"], "days", "{value}");
    }
}

#[tokio::test]
async fn the_history_follows_the_summary_rights() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "other").await;
    user_with(
        &fixture,
        "stranger",
        &["as-repository-admin-raw-other-read"],
    )
    .await;
    user_with(&fixture, "reader", &["as-repository-view-raw-files-browse"]).await;
    let url = format!("{REST}/files/usage/history");

    let anonymous = get(&fixture, &url, Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let stranger = get(&fixture, &url, user("stranger")).await;
    assert_eq!(stranger.status, StatusCode::FORBIDDEN);
    let missing = get(
        &fixture,
        &format!("{REST}/missing/usage/history"),
        Auth::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let reader = get(&fixture, &url, user("reader")).await;
    assert_eq!(reader.status, StatusCode::OK);
}

#[tokio::test]
async fn the_ui_history_has_only_the_repositories_the_caller_may_browse() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "secret").await;
    raw_hosted(&fixture, "empty").await;
    seed(&fixture, "files", 31, 3).await;
    seed(&fixture, "files", 2, 2).await;
    seed(&fixture, "files", 1, 1).await;
    seed(&fixture, "secret", 1, 9).await;
    user_with(&fixture, "viewer", &["as-repository-view-raw-files-browse"]).await;
    user_with(&fixture, "nobody", &[]).await;

    let anonymous = get(&fixture, UI, Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let admin = get(&fixture, UI, Auth::Admin).await;
    assert_eq!(
        admin.json(),
        json!({"empty": [], "files": [2, 1], "secret": [9]})
    );
    let widened = get(&fixture, &format!("{UI}?days=90"), Auth::Admin).await;
    assert_eq!(widened.json()["files"], json!([3, 2, 1]));
    let viewer = get(&fixture, UI, user("viewer")).await;
    assert_eq!(viewer.json(), json!({"files": [2, 1]}));
    let nobody = get(&fixture, UI, user("nobody")).await;
    assert_eq!(nobody.json(), json!({}));
}
