mod support;
mod ui_support;

use axum::http::StatusCode;
use chrono::{Days, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture};
use ui_support::*;

const USAGE: &str = "/service/rest/internal/ui/usage-dashboard";

async fn sql(fixture: &Fixture, statement: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(DbBackend::Postgres, statement))
        .await
        .unwrap();
}

async fn download(fixture: &Fixture, path: &str, times: usize) {
    for _ in 0..times {
        let reply = get(fixture, path, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
    }
}

async fn busy() -> Fixture {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    raw_proxy(&fixture, "remote").await;
    maven_hosted(&fixture, "releases").await;
    raw_group(&fixture, "everything", &["files", "remote"]).await;
    put(&fixture, "/repository/files/a/one.txt", "one").await;
    put(&fixture, "/repository/files/b/two.txt", "two-2").await;
    deploy(&fixture, "releases", "org.acme", "lib", "1.0").await;
    download(&fixture, "/repository/files/b/two.txt", 3).await;
    download(
        &fixture,
        "/repository/releases/org/acme/lib/1.0/lib-1.0.jar",
        1,
    )
    .await;
    fixture.state.content().flush_downloads().await.unwrap();
    fixture
}

fn day(offset: u64) -> String {
    Utc::now()
        .date_naive()
        .checked_sub_days(Days::new(offset))
        .unwrap()
        .to_string()
}

fn downloads_on(board: &Value, day: &str) -> i64 {
    board["downloadsPerDay"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["day"] == day)
        .map(|entry| entry["downloads"].as_i64().unwrap())
        .unwrap_or(-1)
}

#[tokio::test]
async fn administrators_see_every_figure() {
    let fixture = busy().await;
    let reply = get(&fixture, USAGE, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let board = reply.json();
    assert_eq!(board["days"], 30);
    assert_eq!(board["to"], day(0));
    assert_eq!(board["from"], day(29));
    assert_eq!(board["unusedDays"], 90);
    assert_eq!(board["repository"], Value::Null);
    assert_eq!(
        names(&board["repositories"], "name"),
        ["everything", "files", "releases", "remote"]
    );
    assert_eq!(board["repositories"][0]["type"], "group");

    let totals = &board["totals"];
    let jar = JAR.len() as i64;
    let pom = pom("org.acme", "lib", "1.0").len() as i64;
    assert_eq!(totals["repositories"], 4);
    assert_eq!(
        [&totals["hosted"], &totals["proxy"], &totals["group"]],
        [2, 1, 1]
    );
    assert_eq!(totals["components"], 3);
    assert_eq!(totals["assets"], 4);
    assert_eq!(totals["storageBytes"], 8 + jar + pom);
    assert_eq!(totals["proxyStorageBytes"], 0);
    assert_eq!(totals["downloads"], 4);
    assert_eq!(totals["downloadsToday"], 4);
    assert_eq!(totals["downloadsInPeriod"], 4);
    assert_eq!(totals["componentsDownloadedInPeriod"], 2);

    let per_day = board["downloadsPerDay"].as_array().unwrap();
    assert_eq!(per_day.len(), 30);
    assert_eq!(per_day[29], json!({"day": day(0), "downloads": 4}));
    assert_eq!(per_day[0]["downloads"], 0);

    let top = &board["topComponents"];
    assert_eq!(names(top, "name"), ["/b/two.txt", "lib"]);
    assert_eq!(top[0]["downloads"], 3);
    assert_eq!(top[0]["repository"], "files");
    assert_eq!(top[0]["group"], "/b");
    assert_eq!(top[0]["browsePath"], "b/two.txt");
    assert_eq!(top[0]["createdBy"], "admin");
    assert_eq!(top[1]["group"], "org.acme");
    assert_eq!(top[1]["version"], "1.0");
    assert_eq!(top[1]["browsePath"], "org/acme/lib/1.0/");
    assert!(top[1]["id"].is_string());
    assert_eq!(board["topComponentsAllTime"], board["topComponents"]);

    assert_eq!(
        names(&board["topRepositoriesByDownloads"], "name"),
        ["files", "releases"]
    );
    assert_eq!(board["topRepositoriesByDownloads"][0]["downloads"], 3);
    assert_eq!(board["topRepositoriesByDownloads"][0]["type"], "hosted");
    assert_eq!(
        names(&board["topRepositoriesByStorage"], "name"),
        ["releases", "files"]
    );
    assert_eq!(
        board["topRepositoriesByStorage"][0]["storageBytes"],
        jar + pom
    );
    assert_eq!(board["topRepositoriesByStorage"][0]["components"], 1);

    assert_eq!(
        names(&board["recentComponents"], "name"),
        ["lib", "/b/two.txt", "/a/one.txt"]
    );
    assert_eq!(board["recentComponents"][0]["downloads"], 0);
    assert!(board["recentComponents"][0]["created"].is_string());

    let unused = &board["unused"];
    assert_eq!(unused["components"], 0);
    assert_eq!(unused["bytes"], 0);
    assert_eq!(unused["neverDownloadedComponents"], 1);
    assert_eq!(unused["neverDownloadedBytes"], 3);
    assert_eq!(unused["repositories"], json!([]));
}

#[tokio::test]
async fn the_period_and_the_repository_narrow_the_figures() {
    let fixture = busy().await;
    sql(
        &fixture,
        &format!(
            "INSERT INTO asset_download_day (asset_id, day, downloads) \
             SELECT id, '{}'::date, 5 FROM asset WHERE path = '/a/one.txt' \
             UNION ALL SELECT id, '{}'::date, 7 FROM asset WHERE path = '/a/one.txt'",
            day(10),
            day(60)
        ),
    )
    .await;

    let week = get(&fixture, &format!("{USAGE}?days=7"), Auth::Admin)
        .await
        .json();
    assert_eq!(week["days"], 7);
    assert_eq!(week["downloadsPerDay"].as_array().unwrap().len(), 7);
    assert_eq!(week["totals"]["downloadsInPeriod"], 4);
    assert_eq!(names(&week["topComponents"], "name"), ["/b/two.txt", "lib"]);

    let month = get(&fixture, USAGE, Auth::Admin).await.json();
    assert_eq!(month["totals"]["downloadsInPeriod"], 9);
    assert_eq!(downloads_on(&month, &day(10)), 5);
    assert_eq!(month["totals"]["componentsDownloadedInPeriod"], 3);
    assert_eq!(month["topComponents"][0]["name"], "/a/one.txt");
    assert_eq!(month["topComponents"][0]["downloads"], 5);
    assert_eq!(month["topRepositoriesByDownloads"][0]["downloads"], 8);

    let quarter = get(&fixture, &format!("{USAGE}?days=90"), Auth::Admin)
        .await
        .json();
    assert_eq!(quarter["downloadsPerDay"].as_array().unwrap().len(), 90);
    assert_eq!(quarter["totals"]["downloadsInPeriod"], 16);
    assert_eq!(downloads_on(&quarter, &day(60)), 7);
    assert_eq!(quarter["totals"]["downloadsToday"], 4);

    let clamped = get(&fixture, &format!("{USAGE}?days=100000"), Auth::Admin)
        .await
        .json();
    assert_eq!(clamped["days"], 180);

    let one = get(
        &fixture,
        &format!("{USAGE}?repository=releases"),
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(one["repository"], "releases");
    assert_eq!(one["repositories"].as_array().unwrap().len(), 4);
    assert_eq!(one["totals"]["repositories"], 1);
    assert_eq!(one["totals"]["components"], 1);
    assert_eq!(one["totals"]["downloadsInPeriod"], 1);
    assert_eq!(names(&one["recentComponents"], "name"), ["lib"]);

    let missing = get(&fixture, &format!("{USAGE}?repository=nope"), Auth::Admin).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let wrong = get(&fixture, &format!("{USAGE}?days=many"), Auth::Admin).await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn idle_components_and_proxy_caches_are_measured() {
    let fixture = busy().await;
    sql(
        &fixture,
        "UPDATE asset SET created_at = now() - interval '120 days', \
           last_downloaded_at = CASE WHEN last_downloaded_at IS NULL THEN NULL \
             ELSE now() - interval '100 days' END \
         WHERE repository = 'releases'",
    )
    .await;
    sql(
        &fixture,
        "UPDATE component SET repository = 'remote' WHERE name = '/a/one.txt'",
    )
    .await;
    sql(
        &fixture,
        "UPDATE asset SET repository = 'remote' WHERE path = '/a/one.txt'",
    )
    .await;

    let board = get(&fixture, USAGE, Auth::Admin).await.json();
    let jar = JAR.len() as i64;
    let pom = pom("org.acme", "lib", "1.0").len() as i64;
    assert_eq!(board["totals"]["proxyStorageBytes"], 3);
    let unused = &board["unused"];
    assert_eq!(unused["components"], 1);
    assert_eq!(unused["bytes"], jar + pom);
    assert_eq!(
        unused["repositories"],
        json!([{"name": "releases", "format": "maven2", "components": 1, "bytes": jar + pom}])
    );
}

#[tokio::test]
async fn others_see_only_their_share() {
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
    let board = get(&fixture, USAGE, user("viewer")).await.json();
    assert_eq!(names(&board["repositories"], "name"), ["files"]);
    assert_eq!(board["totals"]["repositories"], 1);
    assert_eq!(board["totals"]["assets"], 2);
    assert_eq!(board["totals"]["downloadsInPeriod"], 3);
    assert_eq!(names(&board["topComponents"], "name"), ["/b/two.txt"]);
    assert_eq!(
        get(
            &fixture,
            &format!("{USAGE}?repository=releases"),
            user("viewer")
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    selector(&fixture, "only-a", "format == \"raw\" and path =^ \"/a/\"").await;
    selector_privilege(&fixture, "files-a", "only-a", "raw", "files", &["read"]).await;
    user_with(&fixture, "picker", &["files-a"]).await;
    let board = get(&fixture, USAGE, user("picker")).await.json();
    assert_eq!(names(&board["repositories"], "name"), ["files"]);
    assert_eq!(board["totals"]["assets"], 1);
    assert_eq!(board["totals"]["storageBytes"], 3);
    assert_eq!(board["totals"]["downloadsInPeriod"], 0);
    assert_eq!(board["topComponents"], json!([]));
    assert_eq!(names(&board["recentComponents"], "name"), ["/a/one.txt"]);

    user_with(&fixture, "storekeeper", &["as-blobstores-read"]).await;
    let board = get(&fixture, USAGE, user("storekeeper")).await.json();
    assert_eq!(board["repositories"], json!([]));
    assert_eq!(board["totals"]["repositories"], 0);
    assert_eq!(board["downloadsPerDay"].as_array().unwrap().len(), 30);

    assert_eq!(
        get(&fixture, USAGE, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
    anonymous(&fixture, true).await;
    let board = get(&fixture, USAGE, Auth::None).await.json();
    assert_eq!(board["totals"]["repositories"], 4);
    assert_eq!(board["totals"]["downloads"], 4);
}

#[tokio::test]
async fn download_days_older_than_the_retention_are_pruned() {
    let fixture = busy().await;
    get(&fixture, USAGE, Auth::Admin).await;
    sql(
        &fixture,
        &format!(
            "INSERT INTO asset_download_day (asset_id, day, downloads) \
             SELECT id, '{}'::date, 2 FROM asset WHERE path = '/a/one.txt'",
            day(200)
        ),
    )
    .await;
    let pruned = fixture.state.content().prune_download_days().await.unwrap();
    assert_eq!(pruned, 1);
    sql(&fixture, "DELETE FROM asset WHERE path = '/b/two.txt'").await;
    let board = get(&fixture, USAGE, Auth::Admin).await.json();
    assert_eq!(board["totals"]["downloadsToday"], 1);
}
