mod support;
mod ui_support;

use astrum_db::entity::audit_event;
use axum::http::{Method, StatusCode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const BROWSE: &str = "/service/rest/internal/ui/browse";

async fn files() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    put(&fixture, "/repository/files/a/one.txt", "one").await;
    put(&fixture, "/repository/files/a/b/two.txt", "two-2").await;
    put(&fixture, "/repository/files/a/b/three.txt", "three").await;
    put(&fixture, "/repository/files/c.txt", "cc").await;
    fixture
}

async fn browse(fixture: &Fixture, repository: &str, path: &str, auth: Auth<'_>) -> Reply {
    get(fixture, &format!("{BROWSE}/{repository}?path={path}"), auth).await
}

fn kinds_and_names(tree: &Value) -> Vec<(String, String)> {
    tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap().to_owned(),
                node["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(kind, name)| ((*kind).to_owned(), (*name).to_owned()))
        .collect()
}

#[tokio::test]
async fn files_with_components_come_before_folders_with_their_details() {
    let fixture = files().await;
    let root = browse(&fixture, "files", "", Auth::Admin).await;
    assert_eq!(root.status, StatusCode::OK, "{}", root.text());
    let root = root.json();
    assert_eq!(root["repository"], "files");
    assert_eq!(root["format"], "raw");
    assert_eq!(root["type"], "hosted");
    assert_eq!(root["path"], "/");
    assert_eq!(root["truncated"], false);
    assert_eq!(
        kinds_and_names(&root),
        pairs(&[("asset", "c.txt"), ("folder", "a")])
    );
    assert_eq!(root["children"][1]["path"], "/a/");
    assert_eq!(root["children"][1]["childCount"], 2);

    let folder = browse(&fixture, "files", "a", Auth::Admin).await.json();
    assert_eq!(folder["path"], "/a/");
    assert_eq!(
        kinds_and_names(&folder),
        pairs(&[("asset", "one.txt"), ("folder", "b")])
    );
    let one = &folder["children"][0];
    assert_eq!(one["assetPath"], "/a/one.txt");
    assert_eq!(one["path"], "/a/one.txt");
    assert_eq!(one["size"], 3);
    assert_eq!(one["contentType"], "text/plain");
    assert_eq!(one["repository"], "files");
    assert!(one["lastModified"].is_string());
    assert_eq!(
        one["downloadUrl"],
        "http://localhost/repository/files/a/one.txt"
    );
    let asset = get(
        &fixture,
        &format!(
            "/service/rest/v1/assets/{}",
            one["assetId"].as_str().unwrap()
        ),
        Auth::Admin,
    )
    .await;
    assert_eq!(asset.status, StatusCode::OK);
    assert_eq!(asset.json()["path"], "/a/one.txt");
    let component = get(
        &fixture,
        &format!(
            "/service/rest/v1/components/{}",
            one["componentId"].as_str().unwrap()
        ),
        Auth::Admin,
    )
    .await;
    assert_eq!(component.status, StatusCode::OK);

    let deeper = browse(&fixture, "files", "/a/b/", Auth::Admin).await.json();
    assert_eq!(
        kinds_and_names(&deeper),
        pairs(&[("asset", "three.txt"), ("asset", "two.txt")])
    );
    let limited = get(
        &fixture,
        &format!("{BROWSE}/files?path=a/b&limit=1"),
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(limited["truncated"], true);
    assert_eq!(limited["children"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn missing_folders_and_repositories_answer_404() {
    let fixture = files().await;
    for path in ["nope", "a/one.txt", "a/b/c"] {
        assert_eq!(
            browse(&fixture, "files", path, Auth::Admin).await.status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    assert_eq!(
        browse(&fixture, "files", "a/../c", Auth::Admin)
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        browse(&fixture, "missing", "", Auth::Admin).await.status,
        StatusCode::NOT_FOUND
    );
    raw_hosted(&fixture, "empty").await;
    let empty = browse(&fixture, "empty", "", Auth::Admin).await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.json()["children"], json!([]));
}

#[tokio::test]
async fn maven_versions_show_as_components() {
    let fixture = ready().await;
    maven_hosted(&fixture, "releases").await;
    for version in ["1.0", "1.10", "1.9"] {
        deploy(&fixture, "releases", "org.acme", "lib", version).await;
    }
    let org = browse(&fixture, "releases", "org", Auth::Admin)
        .await
        .json();
    assert_eq!(kinds_and_names(&org), pairs(&[("folder", "acme")]));
    let lib = browse(&fixture, "releases", "org/acme/lib", Auth::Admin)
        .await
        .json();
    assert_eq!(
        kinds_and_names(&lib),
        pairs(&[
            ("component", "1.0"),
            ("component", "1.9"),
            ("component", "1.10")
        ])
    );
    let node = &lib["children"][0];
    assert_eq!(
        node["component"],
        json!({"group": "org.acme", "name": "lib", "version": "1.0"})
    );
    assert_eq!(node["childCount"], 2);
    assert_eq!(node["path"], "/org/acme/lib/1.0/");
    let version = browse(&fixture, "releases", "org/acme/lib/1.0", Auth::Admin)
        .await
        .json();
    assert_eq!(
        kinds_and_names(&version),
        pairs(&[("asset", "lib-1.0.jar"), ("asset", "lib-1.0.pom")])
    );
    assert_eq!(version["children"][0]["componentId"], node["componentId"]);
    assert_eq!(version["children"][1]["component"]["version"], "1.0");
}

#[tokio::test]
async fn groups_show_the_union_of_their_members() {
    let fixture = ready().await;
    raw_hosted(&fixture, "first").await;
    raw_hosted(&fixture, "second").await;
    put(&fixture, "/repository/first/x/same.txt", "first").await;
    put(&fixture, "/repository/first/x/only-first.txt", "1").await;
    put(&fixture, "/repository/second/x/same.txt", "second-longer").await;
    put(&fixture, "/repository/second/y/only-second.txt", "2").await;
    raw_group(&fixture, "both", &["first", "second"]).await;
    raw_group(&fixture, "outer", &["both"]).await;

    for group in ["both", "outer"] {
        let root = browse(&fixture, group, "", Auth::Admin).await.json();
        assert_eq!(
            kinds_and_names(&root),
            pairs(&[("folder", "x"), ("folder", "y")]),
            "{group}"
        );
        assert_eq!(root["children"][0]["childCount"], 2);
        let x = browse(&fixture, group, "x", Auth::Admin).await.json();
        assert_eq!(
            kinds_and_names(&x),
            pairs(&[("asset", "only-first.txt"), ("asset", "same.txt")]),
            "{group}"
        );
        let same = &x["children"][1];
        assert_eq!(same["repository"], "first");
        assert_eq!(same["size"], 5);
        assert_eq!(
            same["downloadUrl"],
            format!("http://localhost/repository/{group}/x/same.txt")
        );
        let y = browse(&fixture, group, "y", Auth::Admin).await.json();
        assert_eq!(y["children"][0]["repository"], "second");
    }
}

#[tokio::test]
async fn browsing_follows_views_and_content_selectors() {
    let fixture = files().await;
    assert_eq!(
        browse(&fixture, "files", "", Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
    fixture.add_user("nobody", "password-1", &[]).await;
    assert_eq!(
        browse(&fixture, "files", "", user("nobody")).await.status,
        StatusCode::NOT_FOUND
    );

    selector(&fixture, "deep", "path =^ \"/a/b/\"").await;
    selector_privilege(&fixture, "files-deep", "deep", "raw", "files", &["BROWSE"]).await;
    user_with(&fixture, "selected", &["files-deep"]).await;
    let root = browse(&fixture, "files", "", user("selected")).await;
    assert_eq!(root.status, StatusCode::OK, "{}", root.text());
    assert_eq!(kinds_and_names(&root.json()), pairs(&[("folder", "a")]));
    assert_eq!(root.json()["children"][0]["childCount"], 1);
    let a = browse(&fixture, "files", "a", user("selected"))
        .await
        .json();
    assert_eq!(kinds_and_names(&a), pairs(&[("folder", "b")]));
    assert_eq!(
        browse(&fixture, "files", "c.txt", user("selected"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    anonymous(&fixture, true).await;
    let anonymous = browse(&fixture, "files", "", Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::OK);
    assert_eq!(
        kinds_and_names(&anonymous.json()),
        pairs(&[("asset", "c.txt"), ("folder", "a")])
    );
}

async fn remove(fixture: &Fixture, repository: &str, path: &str, auth: Auth<'_>) -> Reply {
    fixture
        .call(
            Method::DELETE,
            &format!("{BROWSE}/{repository}?path={path}"),
            auth,
            None,
        )
        .await
}

#[tokio::test]
async fn deleting_a_folder_removes_what_the_caller_may_delete() {
    let fixture = files().await;
    user_with(
        &fixture,
        "reader",
        &[
            "as-repository-view-raw-files-browse",
            "as-repository-view-raw-files-read",
        ],
    )
    .await;
    assert_eq!(
        remove(&fixture, "files", "a", user("reader")).await.status,
        StatusCode::FORBIDDEN
    );

    selector(&fixture, "deep", "path =^ \"/a/b/\"").await;
    selector_privilege(
        &fixture,
        "files-deep-delete",
        "deep",
        "raw",
        "files",
        &["DELETE"],
    )
    .await;
    user_with(
        &fixture,
        "pruner",
        &[
            "as-repository-view-raw-files-browse",
            "as-repository-view-raw-files-read",
            "files-deep-delete",
        ],
    )
    .await;
    let reply = remove(&fixture, "files", "a", user("pruner")).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!({"deletedAssets": 2, "deletedComponents": 2, "skippedAssets": 1})
    );
    let root = browse(&fixture, "files", "a", Auth::Admin).await.json();
    assert_eq!(kinds_and_names(&root), pairs(&[("asset", "one.txt")]));
    assert_eq!(
        get(&fixture, "/repository/files/a/b/two.txt", Auth::Admin)
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let events = audit_event::Entity::find()
        .filter(audit_event::Column::Domain.is_in(["asset", "component"]))
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    let summary: Vec<(String, String, String)> = events
        .iter()
        .map(|event| {
            (
                event.domain.clone(),
                event.context.clone(),
                event.initiator.clone(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                "asset".to_owned(),
                "/a/b/three.txt".to_owned(),
                "pruner".to_owned()
            ),
            (
                "asset".to_owned(),
                "/a/b/two.txt".to_owned(),
                "pruner".to_owned()
            ),
            (
                "component".to_owned(),
                "/a/b:/a/b/three.txt:".to_owned(),
                "pruner".to_owned()
            ),
            (
                "component".to_owned(),
                "/a/b:/a/b/two.txt:".to_owned(),
                "pruner".to_owned()
            ),
        ]
    );
    assert_eq!(events[0].attributes["repository"], "files");
    assert_eq!(events[0].attributes["folder"], "/a/");

    let reply = remove(&fixture, "files", "a/", Auth::Admin).await;
    assert_eq!(
        reply.json(),
        json!({"deletedAssets": 1, "deletedComponents": 1, "skippedAssets": 0})
    );
    assert_eq!(
        remove(&fixture, "files", "a", Auth::Admin).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        remove(&fixture, "files", "", Auth::Admin).await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        remove(&fixture, "files", "c.txt", Auth::Admin).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        remove(&fixture, "missing", "a", Auth::Admin).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        remove(&fixture, "files", "a", Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn groups_refuse_folder_deletes_and_proxies_take_them() {
    let fixture = files().await;
    raw_group(&fixture, "all", &["files"]).await;
    raw_proxy(&fixture, "remote").await;
    let refused = remove(&fixture, "all", "a", Auth::Admin).await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert_eq!(
        remove(&fixture, "remote", "a", Auth::Admin).await.status,
        StatusCode::NOT_FOUND
    );

    maven_hosted(&fixture, "releases").await;
    deploy(&fixture, "releases", "org.acme", "lib", "1.0").await;
    deploy(&fixture, "releases", "org.acme", "lib", "2.0").await;
    let reply = remove(&fixture, "releases", "org/acme/lib/1.0", Auth::Admin).await;
    assert_eq!(
        reply.json(),
        json!({"deletedAssets": 2, "deletedComponents": 1, "skippedAssets": 0})
    );
    let lib = browse(&fixture, "releases", "org/acme/lib", Auth::Admin)
        .await
        .json();
    assert_eq!(kinds_and_names(&lib), pairs(&[("component", "2.0")]));
}
