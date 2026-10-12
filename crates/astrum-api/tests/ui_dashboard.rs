mod support;
mod ui_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use support::{Auth, Fixture};
use ui_support::*;

const DASHBOARD: &str = "/service/rest/internal/ui/dashboard";

async fn busy() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_proxy(&fixture, "remote").await;
    maven_hosted(&fixture, "releases").await;
    put(&fixture, "/repository/files/a/one.txt", "one").await;
    put(&fixture, "/repository/files/b/two.txt", "two-2").await;
    deploy(&fixture, "releases", "org.acme", "lib", "1.0").await;
    for _ in 0..3 {
        let reply = get(&fixture, "/repository/files/b/two.txt", Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::OK);
    }
    let reply = get(
        &fixture,
        "/repository/releases/org/acme/lib/1.0/lib-1.0.jar",
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    fixture.state.content().flush_downloads().await.unwrap();
    fixture
}

fn formats(value: &Value) -> Value {
    value["repositories"]["byFormat"].clone()
}

#[tokio::test]
async fn administrators_see_every_total() {
    let fixture = busy().await;
    let reply = get(&fixture, DASHBOARD, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let board = reply.json();
    assert_eq!(board["repositories"]["total"], 3);
    assert_eq!(
        board["repositories"]["byType"],
        json!({"hosted": 2, "proxy": 1, "group": 0})
    );
    assert_eq!(
        formats(&board),
        json!([
            {"format": "maven2", "total": 1, "hosted": 1, "proxy": 0, "group": 0},
            {"format": "raw", "total": 2, "hosted": 1, "proxy": 1, "group": 0}
        ])
    );
    assert_eq!(board["assets"], 4);
    assert_eq!(board["components"], 3);
    let jar = JAR.len() as i64;
    let pom = pom("org.acme", "lib", "1.0").len() as i64;
    assert_eq!(board["totalSize"], 8 + jar + pom);

    let downloads = &board["downloads"];
    assert_eq!(downloads["total"], 4);
    assert_eq!(downloads["topAssets"][0]["path"], "/b/two.txt");
    assert_eq!(downloads["topAssets"][0]["downloads"], 3);
    assert_eq!(downloads["topAssets"][0]["repository"], "files");
    assert!(downloads["topAssets"][0]["lastDownloaded"].is_string());
    assert_eq!(downloads["topAssets"].as_array().unwrap().len(), 2);
    assert_eq!(downloads["topComponents"][1]["name"], "lib");
    assert_eq!(downloads["topComponents"][1]["group"], "org.acme");
    assert_eq!(downloads["topComponents"][1]["downloads"], 1);

    let recent = board["recentUploads"].as_array().unwrap();
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0]["kind"], "component");
    assert_eq!(recent[0]["name"], "lib");
    assert_eq!(recent[0]["version"], "1.0");
    assert_eq!(recent[0]["assetCount"], 2);
    assert_eq!(recent[0]["uploader"], "admin");
    assert_eq!(recent[2]["path"], "/a/one.txt");

    let stores = board["blobStores"].as_array().unwrap();
    assert_eq!(stores.len(), 1);
    assert_eq!(stores[0]["name"], "default");
    assert_eq!(stores[0]["type"], "File");
    assert_eq!(stores[0]["blobCount"], 4);
}

#[tokio::test]
async fn others_see_their_share_without_blob_stores() {
    let fixture = busy().await;
    user_with(
        &fixture,
        "viewer",
        &[
            "as-repository-view-raw-files-read",
            "as-repository-view-raw-files-browse",
        ],
    )
    .await;
    let board = get(&fixture, DASHBOARD, user("viewer")).await.json();
    assert_eq!(board["repositories"]["total"], 1);
    assert_eq!(board["assets"], 2);
    assert_eq!(board["downloads"]["total"], 3);
    assert_eq!(
        board["downloads"]["topComponents"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(board["recentUploads"].as_array().unwrap().len(), 2);
    assert!(board.get("blobStores").is_none());

    user_with(&fixture, "storekeeper", &["as-blobstores-read"]).await;
    let board = get(&fixture, DASHBOARD, user("storekeeper")).await.json();
    assert_eq!(board["repositories"]["total"], 0);
    assert_eq!(board["assets"], 0);
    assert_eq!(board["downloads"]["topAssets"], json!([]));
    assert_eq!(board["blobStores"].as_array().unwrap().len(), 1);

    assert_eq!(
        get(&fixture, DASHBOARD, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
    anonymous(&fixture, true).await;
    let board = get(&fixture, DASHBOARD, Auth::None).await.json();
    assert_eq!(board["repositories"]["total"], 3);
    assert_eq!(board["assets"], 4);
    assert!(board.get("blobStores").is_none());
}
