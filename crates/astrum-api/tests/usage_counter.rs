mod support;

use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_content::{store, usage};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::json;
use support::{Auth, Fixture};

const POLICIES: &str = "/service/rest/internal/cleanup-policies";
const TASKS: &str = "/service/rest/v1/tasks";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
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

async fn create(fixture: &Fixture, name: &str, cleanup: Option<&str>) {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    });
    if let Some(policy) = cleanup {
        body["cleanup"] = json!({"policyNames": [policy]});
    }
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(body),
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

async fn actual(fixture: &Fixture, repository: &str) -> (i64, i64) {
    let row = fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*)::bigint AS assets, coalesce(sum(size), 0)::bigint AS bytes FROM asset WHERE repository = $1",
            [repository.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    (
        row.try_get("", "assets").unwrap(),
        row.try_get("", "bytes").unwrap(),
    )
}

async fn counter(fixture: &Fixture, repository: &str) -> Option<(i64, i64)> {
    usage::current(fixture.state.db().conn(), repository)
        .await
        .unwrap()
        .map(|usage| (usage.assets, usage.bytes))
}

async fn agrees(fixture: &Fixture, repository: &str) -> (i64, i64) {
    let expected = actual(fixture, repository).await;
    assert_eq!(counter(fixture, repository).await, Some(expected));
    expected
}

#[tokio::test]
async fn uploads_replacements_and_deletes_keep_the_counter_exact() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    assert_eq!(agrees(&fixture, "docs").await, (0, 0));

    put(&fixture, "docs", "a.txt", 100).await;
    put(&fixture, "docs", "b.txt", 40).await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 140));

    put(&fixture, "docs", "a.txt", 250).await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 290));
    put(&fixture, "docs", "a.txt", 30).await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 70));
    put(&fixture, "docs", "a.txt", 30).await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 70));
    put(&fixture, "docs", "a.txt", 0).await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 40));

    let content = fixture.state.content();
    let found = store::find(content.db(), "docs", "/b.txt")
        .await
        .unwrap()
        .unwrap();
    store::delete_asset(content, found).await.unwrap();
    assert_eq!(agrees(&fixture, "docs").await, (1, 0));

    put(&fixture, "docs", "c/d.txt", 500).await;
    put(&fixture, "docs", "c/e.txt", 5).await;
    assert_eq!(agrees(&fixture, "docs").await, (3, 505));
    let component = number(
        &fixture,
        "SELECT component_id AS n FROM asset WHERE repository = 'docs' AND path = '/c/d.txt'",
    )
    .await;
    assert!(store::delete_component(content, component).await.unwrap());
    assert_eq!(agrees(&fixture, "docs").await, (2, 5));
}

#[tokio::test]
async fn repositories_start_at_zero_rename_and_delete_with_their_assets() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    create(&fixture, "other", None).await;
    put(&fixture, "docs", "a.txt", 100).await;
    put(&fixture, "docs", "b.txt", 11).await;
    put(&fixture, "other", "a.txt", 7).await;
    put(&fixture, "other", "z.bin", 9).await;

    execute(
        &fixture,
        "UPDATE repository SET name = 'papers' WHERE name = 'docs'",
    )
    .await;
    assert_eq!(counter(&fixture, "docs").await, None);
    assert_eq!(agrees(&fixture, "papers").await, (2, 111));
    assert_eq!(agrees(&fixture, "other").await, (2, 16));
    put(&fixture, "papers", "c.txt", 3).await;
    assert_eq!(agrees(&fixture, "papers").await, (3, 114));

    execute(&fixture, "DELETE FROM asset WHERE path LIKE '/a.%'").await;
    assert_eq!(agrees(&fixture, "papers").await, (2, 14));
    assert_eq!(agrees(&fixture, "other").await, (1, 9));

    let reply = fixture
        .call(
            Method::DELETE,
            "/service/rest/v1/repositories/papers",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(counter(&fixture, "papers").await, None);
    assert_eq!(actual(&fixture, "papers").await, (0, 0));
    assert_eq!(agrees(&fixture, "other").await, (1, 9));
    assert_eq!(
        number(&fixture, "SELECT count(*) AS n FROM repository_usage").await,
        1
    );
}

#[tokio::test]
async fn deleting_a_repository_in_sql_does_not_fail_or_recreate_the_row() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    put(&fixture, "docs", "a.txt", 100).await;
    execute(&fixture, "DELETE FROM repository WHERE name = 'docs'").await;
    assert_eq!(counter(&fixture, "docs").await, None);
    assert_eq!(
        number(&fixture, "SELECT count(*) AS n FROM repository_usage").await,
        0
    );
}

#[tokio::test]
async fn cleanup_task_deleting_many_components_is_one_exact_correction() {
    let fixture = ready().await;
    let reply = fixture
        .call(
            Method::POST,
            POLICIES,
            Auth::Admin,
            Some(json!({"name": "old-raw", "format": "raw", "criteriaLastBlobUpdated": 7})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    create(&fixture, "docs", Some("old-raw")).await;
    for index in 0..30 {
        put(&fixture, "docs", &format!("old/{index}.txt"), 10 + index).await;
    }
    put(&fixture, "docs", "new.txt", 77).await;
    execute(
        &fixture,
        "UPDATE asset SET blob_updated_at = now() - interval '10 days' WHERE path LIKE '/old/%'",
    )
    .await;
    assert_eq!(agrees(&fixture, "docs").await, (31, 30 * 10 + 435 + 77));

    let created = fixture
        .call(
            Method::POST,
            TASKS,
            Auth::Admin,
            Some(json!({"type": "repository.cleanup", "name": "Cleanup"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let run = fixture
        .call(
            Method::POST,
            &format!("{TASKS}/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(run.status, StatusCode::NO_CONTENT, "{}", run.text());
    fixture.state.tasks().run_pending().await.unwrap();
    assert_eq!(agrees(&fixture, "docs").await, (1, 77));
}

#[tokio::test]
async fn sizes_changed_in_place_move_the_counter() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    create(&fixture, "other", None).await;
    put(&fixture, "docs", "a.txt", 100).await;
    put(&fixture, "docs", "b.txt", 50).await;
    put(&fixture, "other", "a.txt", 10).await;

    execute(
        &fixture,
        "UPDATE asset SET size = 400 WHERE repository = 'docs' AND path = '/a.txt'",
    )
    .await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 450));
    execute(&fixture, "UPDATE asset SET size = size + 1").await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 452));
    assert_eq!(agrees(&fixture, "other").await, (1, 11));
    execute(&fixture, "UPDATE asset SET size = 1").await;
    assert_eq!(agrees(&fixture, "docs").await, (2, 2));
    assert_eq!(agrees(&fixture, "other").await, (1, 1));
}

#[tokio::test]
async fn flushing_downloads_does_not_touch_the_counter_row() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    put(&fixture, "docs", "a.txt", 100).await;
    let version = "SELECT xmin::text AS t FROM repository_usage WHERE repository = 'docs'";
    let before = text(&fixture, version).await;
    let stamp = text(
        &fixture,
        "SELECT updated_at::text AS t FROM repository_usage WHERE repository = 'docs'",
    )
    .await;

    let content = fixture.state.content();
    let id = number(
        &fixture,
        "SELECT id AS n FROM asset WHERE repository = 'docs'",
    )
    .await;
    content.note_download(id);
    content.note_download(id);
    content.flush_downloads().await.unwrap();
    execute(
        &fixture,
        "UPDATE asset SET last_verified_at = now(), attributes = '{\"k\": 1}'",
    )
    .await;

    assert_eq!(
        number(
            &fixture,
            "SELECT download_count AS n FROM asset WHERE id > 0"
        )
        .await,
        2
    );
    assert_eq!(text(&fixture, version).await, before);
    assert_eq!(
        text(
            &fixture,
            "SELECT updated_at::text AS t FROM repository_usage WHERE repository = 'docs'"
        )
        .await,
        stamp
    );
    assert_eq!(agrees(&fixture, "docs").await, (1, 100));
}

#[tokio::test]
async fn twenty_concurrent_uploads_give_the_exact_sum() {
    let fixture = Arc::new(ready().await);
    create(&fixture, "docs", None).await;
    let mut uploads = Vec::new();
    for index in 0..20usize {
        let fixture = Arc::clone(&fixture);
        uploads.push(tokio::spawn(async move {
            put(
                &fixture,
                "docs",
                &format!("f/{index}.bin"),
                (index + 1) * 100,
            )
            .await;
        }));
    }
    for upload in uploads {
        upload.await.unwrap();
    }
    assert_eq!(agrees(&fixture, "docs").await, (20, 21_000));
}

#[tokio::test]
async fn concurrent_deletes_across_repositories_keep_the_sums_exact() {
    let fixture = Arc::new(ready().await);
    create(&fixture, "docs", None).await;
    create(&fixture, "other", None).await;
    for repository in ["docs", "other"] {
        execute(
            &fixture,
            &format!(
                "INSERT INTO asset (repository, format, path, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5) \
                 SELECT '{repository}', 'raw', '/' || n, 'default', gen_random_uuid(), n, 'x', '', '', '', '' \
                 FROM generate_series(1, 200) AS n"
            ),
        )
        .await;
    }
    assert_eq!(agrees(&fixture, "docs").await, (200, 20_100));
    let mut deletes = Vec::new();
    for remainder in 0..3 {
        let fixture = Arc::clone(&fixture);
        deletes.push(tokio::spawn(async move {
            execute(
                &fixture,
                &format!("DELETE FROM asset WHERE id % 4 = {remainder}"),
            )
            .await;
        }));
    }
    for delete in deletes {
        delete.await.unwrap();
    }
    assert_eq!(agrees(&fixture, "docs").await.0, 50);
    assert_eq!(agrees(&fixture, "other").await.0, 50);
}

#[tokio::test]
async fn deleting_the_last_asset_and_uploading_into_its_component_cannot_deadlock() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    put(&fixture, "docs", "a.txt", 100).await;
    let component = number(
        &fixture,
        "SELECT component_id AS n FROM asset WHERE repository = 'docs' AND path = '/a.txt'",
    )
    .await;
    let content = fixture.state.content().clone();
    let found = store::find(content.db(), "docs", "/a.txt")
        .await
        .unwrap()
        .unwrap();

    let uploader = fixture.state.db().conn().begin().await.unwrap();
    uploader
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!("UPDATE component SET updated_at = now() WHERE id = {component}"),
        ))
        .await
        .unwrap();

    let deleter = tokio::spawn(async move { store::delete_asset(&content, found).await });
    let deadline = Instant::now() + Duration::from_secs(10);
    while number(
        &fixture,
        "SELECT count(*) AS n FROM pg_stat_activity WHERE datname = current_database() AND wait_event_type = 'Lock'",
    )
    .await
        == 0
    {
        assert!(Instant::now() < deadline, "the deleter never waited");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    uploader
        .execute(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "INSERT INTO asset (repository, format, path, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5, component_id) \
                 VALUES ('docs', 'raw', '/b.txt', 'default', gen_random_uuid(), 700, 'x', '', '', '', '', {component})"
            ),
        ))
        .await
        .unwrap();
    uploader.commit().await.unwrap();
    deleter.await.unwrap().unwrap();

    assert_eq!(agrees(&fixture, "docs").await, (1, 700));
    assert_eq!(
        number(
            &fixture,
            &format!("SELECT count(*) AS n FROM component WHERE id = {component}")
        )
        .await,
        1
    );
}

#[tokio::test]
async fn reconcile_corrects_drift_and_waits_for_open_uploads() {
    let fixture = ready().await;
    create(&fixture, "docs", None).await;
    put(&fixture, "docs", "a.txt", 100).await;
    put(&fixture, "docs", "b.txt", 50).await;
    let db = fixture.state.db().conn();

    let clean = usage::reconcile(db, "docs").await.unwrap().unwrap();
    assert!(!clean.corrected());
    assert_eq!(
        clean.usage,
        usage::Usage {
            assets: 2,
            bytes: 150
        }
    );
    assert!(usage::reconcile(db, "nowhere").await.unwrap().is_none());

    execute(
        &fixture,
        "UPDATE repository_usage SET assets = assets + 3, bytes = bytes - 70 WHERE repository = 'docs'",
    )
    .await;
    assert_eq!(counter(&fixture, "docs").await, Some((5, 80)));
    let fixed = usage::reconcile(db, "docs").await.unwrap().unwrap();
    assert_eq!((fixed.assets_drift, fixed.bytes_drift), (-3, 70));
    assert_eq!(agrees(&fixture, "docs").await, (2, 150));

    let open = db.begin().await.unwrap();
    open.execute(Statement::from_string(
        DbBackend::Postgres,
        "INSERT INTO asset (repository, format, path, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5) \
         VALUES ('docs', 'raw', '/held.bin', 'default', gen_random_uuid(), 700, 'x', '', '', '', '')",
    ))
    .await
    .unwrap();
    let waiting = {
        let db = db.clone();
        tokio::spawn(async move { usage::reconcile(&db, "docs").await })
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!waiting.is_finished());
    open.commit().await.unwrap();
    let after = waiting.await.unwrap().unwrap().unwrap();
    assert!(!after.corrected());
    assert_eq!(
        after.usage,
        usage::Usage {
            assets: 3,
            bytes: 850
        }
    );
    assert_eq!(agrees(&fixture, "docs").await, (3, 850));
}
