use std::time::Duration;

use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};

use super::{QueueError, Retry, Selection, enqueue_repository_with};

async fn scalar(db: &DatabaseConnection, sql: &str) -> i64 {
    db.query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn setup(components: i64) -> (TestDatabase, DatabaseConnection) {
    let test = TestDatabase::create().await;
    let db = test.migrated().await.conn().clone();
    db.execute_unprepared(
        "INSERT INTO repository (name, format, type, attributes) \
         VALUES ('releases', 'maven2', 'hosted', '{}')",
    )
    .await
    .unwrap();
    db.execute_unprepared(&format!(
        "INSERT INTO component (id, repository, format, name, version) \
         SELECT n, 'releases', 'maven2', 'lib' || n, '1' FROM generate_series(1, {components}) n"
    ))
    .await
    .unwrap();
    (test, db)
}

fn retry(lock_timeout_ms: u64, attempts: u32) -> Retry {
    Retry {
        lock_timeout: Duration::from_millis(lock_timeout_ms),
        attempts,
        pause: Duration::from_millis(50),
    }
}

async fn enqueue(
    db: &DatabaseConnection,
    selection: Selection,
    retry: Retry,
) -> Result<super::Enqueued, QueueError> {
    enqueue_repository_with(db, "releases", "test", selection, 10, retry, &|| false).await
}

#[tokio::test]
async fn a_component_deleted_before_its_insert_is_skipped_and_the_chunk_succeeds() {
    let (_test, db) = setup(3).await;
    let deleter = db.begin().await.unwrap();
    deleter
        .execute_unprepared("DELETE FROM component WHERE id = 2")
        .await
        .unwrap();
    let task = {
        let db = db.clone();
        tokio::spawn(async move { enqueue(&db, Selection::Every, retry(5000, 1)).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!task.is_finished());
    deleter.commit().await.unwrap();

    let done = task.await.unwrap().unwrap();
    assert_eq!(done.queued, 2);
    assert_eq!(
        scalar(
            &db,
            "SELECT count(*) AS n FROM scan_queue WHERE component_id <> 2"
        )
        .await,
        2
    );
}

#[tokio::test]
async fn a_chunk_repeats_when_a_row_lock_is_released_after_the_first_attempt() {
    let (_test, db) = setup(3).await;
    db.execute_unprepared("INSERT INTO scan_queue (component_id, reason) VALUES (2, 'old')")
        .await
        .unwrap();
    let holder = db.begin().await.unwrap();
    holder
        .execute_unprepared("UPDATE scan_queue SET reason = 'held' WHERE component_id = 2")
        .await
        .unwrap();
    let release = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(250)).await;
        holder.commit().await.unwrap();
    });

    let done = enqueue(&db, Selection::Every, retry(60, 20)).await.unwrap();
    release.await.unwrap();
    assert_eq!(done.components, 3);
    assert_eq!(scalar(&db, "SELECT count(*) AS n FROM scan_queue").await, 3);
}

#[tokio::test]
async fn a_chunk_whose_lock_is_never_released_fails_instead_of_hanging() {
    let (_test, db) = setup(3).await;
    db.execute_unprepared("INSERT INTO scan_queue (component_id, reason) VALUES (2, 'old')")
        .await
        .unwrap();
    let holder = db.begin().await.unwrap();
    holder
        .execute_unprepared("UPDATE scan_queue SET reason = 'held' WHERE component_id = 2")
        .await
        .unwrap();

    let failed = tokio::time::timeout(
        Duration::from_secs(10),
        enqueue(&db, Selection::Every, retry(50, 3)),
    )
    .await
    .expect("the enqueue must not hang");
    assert!(matches!(failed, Err(QueueError::Database(_))));
    holder.rollback().await.unwrap();
    assert_eq!(scalar(&db, "SELECT count(*) AS n FROM scan_queue").await, 1);
}

#[tokio::test]
async fn a_plain_scheduled_run_leaves_rows_in_backoff_and_in_lease_alone() {
    let (_test, db) = setup(3).await;
    db.execute_unprepared(
        "INSERT INTO scan_queue (component_id, reason, attempts, next_attempt_at) \
         VALUES (1, 'old', 2, now() + interval '30 minutes')",
    )
    .await
    .unwrap();
    let before = scalar(
        &db,
        "SELECT extract(epoch FROM next_attempt_at)::bigint AS n FROM scan_queue WHERE component_id = 1",
    )
    .await;

    let done = enqueue(&db, Selection::Unscanned, Retry::default())
        .await
        .unwrap();
    assert_eq!((done.components, done.queued), (3, 2));
    assert_eq!(
        scalar(
            &db,
            "SELECT extract(epoch FROM next_attempt_at)::bigint AS n FROM scan_queue WHERE component_id = 1"
        )
        .await,
        before
    );
    assert_eq!(
        scalar(
            &db,
            "SELECT attempts::bigint AS n FROM scan_queue WHERE component_id = 1"
        )
        .await,
        2
    );
}
