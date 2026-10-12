mod support;
mod ui_support;

use axum::http::StatusCode;
use serde_json::Value;
use support::{Auth, Fixture};
use ui_support::*;

const LIST: &str = "/service/rest/internal/ui/repositories";

async fn landscape() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "files2").await;
    raw_proxy(&fixture, "remote").await;
    raw_group(&fixture, "all", &["files", "files2"]).await;
    put(&fixture, "/repository/files/a/one.txt", "one").await;
    put(&fixture, "/repository/files/a/b/two.txt", "two-2").await;
    put(&fixture, "/repository/files/c.txt", "cc").await;
    put(&fixture, "/repository/files2/z.txt", "zzzz").await;
    fixture
}

fn entry<'a>(list: &'a Value, name: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("{name} is missing from {list}"))
}

#[tokio::test]
async fn administrators_see_every_repository_with_its_totals() {
    let fixture = landscape().await;
    let reply = get(&fixture, LIST, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let list = reply.json();
    assert_eq!(names(&list, "name"), ["all", "files", "files2", "remote"]);

    let files = entry(&list, "files");
    assert_eq!(files["format"], "raw");
    assert_eq!(files["type"], "hosted");
    assert_eq!(files["url"], "http://localhost/repository/files");
    assert_eq!(files["online"], true);
    assert_eq!(files["blobStoreName"], "default");
    assert_eq!(files["assetCount"], 3);
    assert_eq!(files["componentCount"], 3);
    assert_eq!(files["totalSize"], 10);
    assert!(files["lastUploaded"].is_string());
    assert_eq!(files["memberOf"], serde_json::json!(["all"]));
    assert!(files.get("members").is_none());
    assert_eq!(files["limited"], false);
    for flag in [
        "browse",
        "read",
        "add",
        "edit",
        "delete",
        "admin",
        "adminEdit",
        "adminDelete",
    ] {
        assert_eq!(files["permissions"][flag], true, "{flag}");
    }

    let group = entry(&list, "all");
    assert_eq!(group["members"], serde_json::json!(["files", "files2"]));
    assert_eq!(group["assetCount"], 0);
    assert_eq!(group["lastUploaded"], Value::Null);
    let remote = entry(&list, "remote");
    assert_eq!(remote["remoteUrl"], "http://127.0.0.1:9/");
    assert_eq!(remote["type"], "proxy");
    assert_eq!(entry(&list, "files2")["totalSize"], 4);
}

#[tokio::test]
async fn users_see_only_what_their_privileges_name() {
    let fixture = landscape().await;
    user_with(
        &fixture,
        "viewer",
        &[
            "as-repository-view-raw-files-browse",
            "as-repository-view-raw-files-read",
        ],
    )
    .await;
    let list = get(&fixture, LIST, user("viewer")).await.json();
    assert_eq!(names(&list, "name"), ["files"]);
    let files = entry(&list, "files");
    assert_eq!(files["memberOf"], serde_json::json!([]));
    assert_eq!(files["assetCount"], 3);
    assert_eq!(files["permissions"]["browse"], true);
    assert_eq!(files["permissions"]["read"], true);
    assert_eq!(files["permissions"]["add"], false);
    assert_eq!(files["permissions"]["delete"], false);
    assert_eq!(files["permissions"]["admin"], false);

    user_with(
        &fixture,
        "configurer",
        &["as-repository-admin-raw-files2-read"],
    )
    .await;
    let list = get(&fixture, LIST, user("configurer")).await.json();
    assert_eq!(names(&list, "name"), ["files2"]);
    let files2 = entry(&list, "files2");
    assert_eq!(files2["assetCount"], Value::Null);
    assert_eq!(files2["totalSize"], Value::Null);
    assert_eq!(files2["permissions"]["admin"], true);
    assert_eq!(files2["permissions"]["adminEdit"], false);
    assert_eq!(files2["permissions"]["browse"], false);

    user_with(&fixture, "uploader", &["as-repository-view-raw-files2-add"]).await;
    let list = get(&fixture, LIST, user("uploader")).await.json();
    assert_eq!(names(&list, "name"), ["files2"]);
    assert_eq!(entry(&list, "files2")["permissions"]["add"], true);
    assert_eq!(entry(&list, "files2")["assetCount"], Value::Null);

    fixture.add_user("nobody", "password-1", &[]).await;
    let list = get(&fixture, LIST, user("nobody")).await.json();
    assert_eq!(list, serde_json::json!([]));
}

#[tokio::test]
async fn content_selectors_limit_the_totals() {
    let fixture = landscape().await;
    selector(&fixture, "a-only", "format == \"raw\" and path =^ \"/a/\"").await;
    selector_privilege(&fixture, "files-a", "a-only", "raw", "files", &["READ"]).await;
    user_with(&fixture, "selected", &["files-a"]).await;
    let list = get(&fixture, LIST, user("selected")).await.json();
    assert_eq!(names(&list, "name"), ["files"]);
    let files = entry(&list, "files");
    assert_eq!(files["limited"], true);
    assert_eq!(files["assetCount"], 2);
    assert_eq!(files["totalSize"], 8);
    assert_eq!(files["permissions"]["read"], true);
    assert_eq!(files["permissions"]["browse"], false);
}

#[tokio::test]
async fn anonymous_callers_follow_the_anonymous_setting() {
    let fixture = landscape().await;
    let reply = get(&fixture, LIST, Auth::None).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);

    anonymous(&fixture, true).await;
    let reply = get(&fixture, LIST, Auth::None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let list = reply.json();
    assert_eq!(names(&list, "name"), ["all", "files", "files2", "remote"]);
    let files = entry(&list, "files");
    assert_eq!(files["permissions"]["read"], true);
    assert_eq!(files["permissions"]["add"], false);
    assert_eq!(files["permissions"]["admin"], false);
    assert_eq!(files["assetCount"], 3);
}

#[tokio::test]
async fn recipes_offer_what_the_server_can_serve() {
    let fixture = ready().await;
    let reply = get(&fixture, &format!("{LIST}/recipes"), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let recipes = reply.json();
    let ids = names(&recipes, "id");
    assert_eq!(
        ids,
        [
            "maven2-hosted",
            "maven2-proxy",
            "maven2-group",
            "raw-hosted",
            "raw-proxy",
            "raw-group"
        ]
    );
    assert_eq!(recipes[0]["apiFormat"], "maven");
    assert_eq!(recipes[0]["format"], "maven2");
    assert_eq!(recipes[0]["type"], "hosted");

    fixture
        .state
        .content()
        .register(std::sync::Arc::new(astrum_format_npm::Npm::default()));
    let recipes = get(&fixture, &format!("{LIST}/recipes"), Auth::Admin)
        .await
        .json();
    assert!(names(&recipes, "id").contains(&"npm-group".to_owned()));
    assert!(!names(&recipes, "id").contains(&"cocoapods-proxy".to_owned()));

    anonymous(&fixture, true).await;
    assert_eq!(
        get(&fixture, &format!("{LIST}/recipes"), Auth::None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
