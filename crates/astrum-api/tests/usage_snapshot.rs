mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use chrono::{Duration, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::json;
use support::{Auth, Fixture};

const TASKS: &str = "/service/rest/v1/tasks";
const UI_TASKS: &str = "/service/rest/internal/ui/tasks";
const NAME: &str = "Record repository usage";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    astrum_tasks::bootstrap(fixture.state.db().conn())
        .await
        .unwrap();
    fixture
}

async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

async fn number(fixture: &Fixture, sql: &str) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

async fn text(fixture: &Fixture, sql: &str) -> String {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<String>("", "t")
        .unwrap()
}

async fn create(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn put(fixture: &Fixture, repository: &str, path: &str, size: usize) {
    let reply = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}/{path}"),
            Auth::Admin,
            Body::from(vec![b'x'; size]),
            &[],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn snapshot(fixture: &Fixture) {
    let listed = fixture
        .call(Method::GET, TASKS, Auth::Admin, None)
        .await
        .json();
    let id = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["name"] == NAME)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let reply = fixture
        .call(
            Method::POST,
            &format!("{TASKS}/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    fixture.state.tasks().run_pending().await.unwrap();
    let runs = fixture
        .call(
            Method::GET,
            &format!("{UI_TASKS}/{id}/runs"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(runs[0]["state"], "done", "{runs}");
}

async fn rows(fixture: &Fixture) -> String {
    let repositories = text(
        fixture,
        "SELECT coalesce(string_agg(repository || '|' || day::text || '|' || components || '|' || assets || '|' || bytes || '|' || downloads, ';' ORDER BY repository, day), '') AS t FROM repository_usage_day",
    )
    .await;
    let stores = text(
        fixture,
        "SELECT coalesce(string_agg(store || '|' || day::text || '|' || blob_count || '|' || total_bytes, ';' ORDER BY store, day), '') AS t FROM blob_store_usage_day",
    )
    .await;
    format!("{repositories}\n{stores}")
}

#[tokio::test]
async fn a_second_run_for_the_same_day_gives_the_same_rows() {
    let fixture = ready().await;
    create(&fixture, "docs").await;
    put(&fixture, "docs", "a/one.bin", 100).await;
    snapshot(&fixture).await;
    let first = rows(&fixture).await;
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*)::bigint AS n FROM repository_usage_day"
        )
        .await,
        1
    );
    snapshot(&fixture).await;
    assert_eq!(rows(&fixture).await, first);
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*)::bigint AS n FROM repository_usage_day"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn a_deleted_repository_loses_its_usage_history() {
    let fixture = ready().await;
    create(&fixture, "docs").await;
    create(&fixture, "gone").await;
    put(&fixture, "gone", "a/one.bin", 100).await;
    snapshot(&fixture).await;
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*)::bigint AS n FROM repository_usage_day WHERE repository = 'gone'"
        )
        .await,
        1
    );
    execute(&fixture, "DELETE FROM repository WHERE name = 'gone'").await;
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*)::bigint AS n FROM repository_usage_day WHERE repository = 'gone'"
        )
        .await,
        0
    );
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*)::bigint AS n FROM repository_usage_day WHERE repository = 'docs'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn rows_older_than_the_retention_are_pruned_and_the_boundary_day_stays() {
    let fixture = ready().await;
    create(&fixture, "docs").await;
    let today = Utc::now().date_naive();
    let kept = (today - Duration::days(730)).to_string();
    let dropped = (today - Duration::days(731)).to_string();
    execute(
        &fixture,
        &format!(
            "INSERT INTO repository_usage_day (repository, day, components, assets, bytes, downloads) \
             VALUES ('docs', '{kept}', 0, 0, 0, 0), ('docs', '{dropped}', 0, 0, 0, 0)"
        ),
    )
    .await;
    execute(
        &fixture,
        &format!(
            "INSERT INTO blob_store_usage_day (store, day, blob_count, total_bytes) \
             VALUES ('default', '{kept}', 0, 0), ('default', '{dropped}', 0, 0)"
        ),
    )
    .await;
    snapshot(&fixture).await;
    for table in ["repository_usage_day", "blob_store_usage_day"] {
        let column = if table == "repository_usage_day" {
            "repository"
        } else {
            "store"
        };
        let days = text(
            &fixture,
            &format!(
                "SELECT coalesce(string_agg(day::text, ',' ORDER BY day), '') AS t FROM {table} WHERE {column} IN ('docs', 'default') AND day < '{}'",
                today - Duration::days(1)
            ),
        )
        .await;
        assert_eq!(days, kept, "{table}");
    }
}

#[tokio::test]
async fn components_assets_bytes_and_downloads_of_yesterday_are_counted() {
    let fixture = ready().await;
    create(&fixture, "docs").await;
    put(&fixture, "docs", "a/one.bin", 100).await;
    put(&fixture, "docs", "a/two.bin", 50).await;
    put(&fixture, "docs", "b/three.bin", 30).await;
    execute(
        &fixture,
        "UPDATE asset SET component_id = (SELECT component_id FROM asset WHERE repository = 'docs' AND path LIKE '%one.bin') \
         WHERE repository = 'docs' AND path LIKE '%three.bin'",
    )
    .await;
    let today = Utc::now().date_naive();
    let yesterday = (today - Duration::days(1)).to_string();
    let older = (today - Duration::days(3)).to_string();
    execute(
        &fixture,
        &format!(
            "INSERT INTO asset_download_day (asset_id, day, downloads) \
             SELECT id, '{yesterday}'::date, 5 FROM asset WHERE repository = 'docs' AND path LIKE '%one.bin' \
             UNION ALL SELECT id, '{yesterday}'::date, 7 FROM asset WHERE repository = 'docs' AND path LIKE '%two.bin' \
             UNION ALL SELECT id, '{older}'::date, 100 FROM asset WHERE repository = 'docs' AND path LIKE '%one.bin'"
        ),
    )
    .await;
    snapshot(&fixture).await;
    assert_eq!(
        text(
            &fixture,
            &format!(
                "SELECT components || '|' || assets || '|' || bytes || '|' || downloads AS t FROM repository_usage_day WHERE repository = 'docs' AND day = '{yesterday}'"
            ),
        )
        .await,
        "2|3|180|12"
    );
}
