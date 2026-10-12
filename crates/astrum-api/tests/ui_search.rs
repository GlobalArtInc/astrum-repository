mod support;
mod ui_support;

use axum::http::StatusCode;
use serde_json::Value;
use support::{Auth, Fixture};
use ui_support::*;

const VERSIONS: &str = "/service/rest/internal/ui/search/versions";

async fn catalogue() -> Fixture {
    let fixture = ready().await;
    maven_hosted(&fixture, "releases").await;
    maven_hosted(&fixture, "staging").await;
    for version in ["1.0", "1.9", "2.0-rc1", "1.10", "1.10-beta"] {
        deploy(&fixture, "releases", "org.acme", "lib", version).await;
    }
    deploy(&fixture, "staging", "org.acme", "lib", "1.9").await;
    deploy(&fixture, "releases", "org.acme", "other", "9.0").await;
    fixture
}

fn listed(reply: &Value) -> Vec<String> {
    reply["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|version| version["version"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn versions_come_newest_first_with_their_repositories() {
    let fixture = catalogue().await;
    let reply = get(
        &fixture,
        &format!("{VERSIONS}?format=maven2&group=org.acme&name=lib"),
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let found = reply.json();
    assert_eq!(found["format"], "maven2");
    assert_eq!(found["group"], "org.acme");
    assert_eq!(found["name"], "lib");
    assert_eq!(
        listed(&found),
        ["2.0-rc1", "1.10", "1.10-beta", "1.9", "1.0"]
    );
    let nine = &found["versions"][3];
    assert_eq!(
        nine["repositories"],
        serde_json::json!(["releases", "staging"])
    );
    assert_eq!(nine["componentIds"].as_array().unwrap().len(), 2);
    assert!(nine["lastModified"].is_string());
    let component = get(
        &fixture,
        &format!(
            "/service/rest/v1/components/{}",
            nine["componentIds"][1].as_str().unwrap()
        ),
        Auth::Admin,
    )
    .await;
    assert_eq!(component.json()["repository"], "staging");
    assert_eq!(component.json()["version"], "1.9");

    let by_api_name = get(
        &fixture,
        &format!("{VERSIONS}?format=maven&group=org.acme&name=lib&repository=staging"),
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(by_api_name["format"], "maven2");
    assert_eq!(listed(&by_api_name), ["1.9"]);

    let other_group = get(
        &fixture,
        &format!("{VERSIONS}?format=maven2&group=org.other&name=lib"),
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(listed(&other_group), Vec::<String>::new());
}

#[tokio::test]
async fn versions_show_only_what_the_caller_may_see() {
    let fixture = catalogue().await;
    let path = format!("{VERSIONS}?format=maven2&group=org.acme&name=lib");
    assert_eq!(
        get(&fixture, &path, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
    user_with(
        &fixture,
        "stager",
        &["as-repository-view-maven2-staging-read"],
    )
    .await;
    let found = get(&fixture, &path, user("stager")).await.json();
    assert_eq!(listed(&found), ["1.9"]);
    assert_eq!(
        found["versions"][0]["repositories"],
        serde_json::json!(["staging"])
    );

    selector(&fixture, "old", "path =~ \"/org/acme/lib/1\\\\.[0-9]/.*\"").await;
    selector_privilege(&fixture, "old-lib", "old", "maven2", "releases", &["READ"]).await;
    user_with(&fixture, "archivist", &["old-lib"]).await;
    let found = get(&fixture, &path, user("archivist")).await.json();
    assert_eq!(listed(&found), ["1.9", "1.0"]);
    assert_eq!(
        found["versions"][0]["repositories"],
        serde_json::json!(["releases"])
    );

    anonymous(&fixture, true).await;
    let found = get(&fixture, &path, Auth::None).await.json();
    assert_eq!(listed(&found).len(), 5);
}

#[tokio::test]
async fn versions_need_a_format_and_a_name() {
    let fixture = catalogue().await;
    let reply = get(&fixture, &format!("{VERSIONS}?group=org.acme"), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(names(&reply.json(), "id"), ["format", "name"]);
}
