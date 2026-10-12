mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const PREVIEW: &str = "/service/rest/internal/ui/content-selectors/preview";

async fn content() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "secret").await;
    maven_hosted(&fixture, "releases").await;
    put(&fixture, "/repository/files/a/one.txt", "one").await;
    put(&fixture, "/repository/files/a/b/two.txt", "two").await;
    put(&fixture, "/repository/files/c.txt", "c").await;
    put(&fixture, "/repository/secret/a/hidden.txt", "h").await;
    deploy(&fixture, "releases", "org.acme", "lib", "1.0").await;
    fixture
}

async fn preview(fixture: &Fixture, auth: Auth<'_>, body: Value) -> Reply {
    fixture.call(Method::POST, PREVIEW, auth, Some(body)).await
}

fn paths(reply: &Reply) -> Vec<String> {
    reply.json()["items"]
        .as_array()
        .unwrap_or_else(|| panic!("{}", reply.text()))
        .iter()
        .map(|item| {
            format!(
                "{}{}",
                item["repository"].as_str().unwrap(),
                item["path"].as_str().unwrap()
            )
        })
        .collect()
}

#[tokio::test]
async fn previews_list_the_matching_assets() {
    let fixture = content().await;
    let reply = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "path =^ \"/a/\""}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        paths(&reply),
        [
            "files/a/b/two.txt",
            "files/a/one.txt",
            "secret/a/hidden.txt"
        ]
    );
    assert_eq!(reply.json()["more"], false);
    let item = &reply.json()["items"][0];
    assert_eq!(item["format"], "raw");
    assert!(item["assetId"].is_string());

    let one_repository = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "path =^ \"/a/\"", "repository": "files"}),
    )
    .await;
    assert_eq!(
        paths(&one_repository),
        ["files/a/b/two.txt", "files/a/one.txt"]
    );
    let everywhere = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "format == \"maven2\" and path =~ \".*\\\\.jar\"", "repository": "*"}),
    )
    .await;
    assert_eq!(
        paths(&everywhere),
        ["releases/org/acme/lib/1.0/lib-1.0.jar"]
    );
    let by_format = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "path =~ \".*\"", "format": "maven2"}),
    )
    .await;
    assert_eq!(by_format.json()["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn previews_report_invalid_expressions() {
    let fixture = content().await;
    for expression in ["", "path ==", "colour == \"red\""] {
        let reply = preview(&fixture, Auth::Admin, json!({"expression": expression})).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{expression}");
        assert_eq!(names(&reply.json(), "id"), ["expression"], "{expression}");
    }
    let reply = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "path =~ \"(\""}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let missing = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "format == \"raw\"", "repository": "missing"}),
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn previews_stop_after_a_hundred_assets() {
    let fixture = ready().await;
    raw_hosted(&fixture, "many").await;
    for index in 0..101 {
        put(&fixture, &format!("/repository/many/f{index:03}.txt"), "x").await;
    }
    let reply = preview(
        &fixture,
        Auth::Admin,
        json!({"expression": "format == \"raw\""}),
    )
    .await;
    assert_eq!(reply.json()["items"].as_array().unwrap().len(), 100);
    assert_eq!(reply.json()["more"], true);
    assert_eq!(reply.json()["items"][99]["path"], "/f099.txt");
}

#[tokio::test]
async fn previews_show_only_what_the_caller_may_read() {
    let fixture = content().await;
    user_with(
        &fixture,
        "editor",
        &["as-selectors-read", "as-repository-view-raw-files-read"],
    )
    .await;
    let reply = preview(
        &fixture,
        user("editor"),
        json!({"expression": "path =^ \"/a/\""}),
    )
    .await;
    assert_eq!(paths(&reply), ["files/a/b/two.txt", "files/a/one.txt"]);
    let hidden = preview(
        &fixture,
        user("editor"),
        json!({"expression": "path =^ \"/a/\"", "repository": "secret"}),
    )
    .await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);

    user_with(&fixture, "reader", &["as-repository-view-raw-files-read"]).await;
    assert_eq!(
        preview(
            &fixture,
            user("reader"),
            json!({"expression": "format == \"raw\""})
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        preview(
            &fixture,
            Auth::None,
            json!({"expression": "format == \"raw\""})
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
}
