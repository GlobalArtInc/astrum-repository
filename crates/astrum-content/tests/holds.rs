use std::collections::HashSet;
use std::time::{Duration, Instant};

use astrum_blob::BlobId;
use astrum_content::holds::{self, Expired, HoldError};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use uuid::Uuid;

async fn count(conn: &impl ConnectionTrait, sql: &str) -> i64 {
    conn.query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

async fn held(conn: &impl ConnectionTrait, store: &str, ids: &[Uuid]) -> HashSet<Uuid> {
    let wanted: Vec<String> = ids.iter().map(|id| format!("'{id}'")).collect();
    conn.query_all(Statement::from_string(
        DbBackend::Postgres,
        format!(
            "SELECT blob_id FROM blob_hold WHERE blob_store = '{store}' AND blob_id IN ({})",
            wanted.join(", ")
        ),
    ))
    .await
    .unwrap()
    .iter()
    .map(|row| row.try_get::<Uuid>("", "blob_id").unwrap())
    .collect()
}

async fn lapse(conn: &impl ConnectionTrait, id: BlobId) {
    conn.execute_unprepared(&format!(
        "UPDATE blob_hold SET held_until = now() - interval '1 minute' WHERE blob_id = '{}'",
        id.uuid()
    ))
    .await
    .unwrap();
}

#[tokio::test]
async fn a_second_hold_on_the_same_blob_is_refused() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let id = BlobId::new();
    holds::hold(conn, "files", id, Duration::from_secs(3600))
        .await
        .unwrap();

    let error = holds::hold(conn, "files", id, Duration::from_secs(60))
        .await
        .unwrap_err();
    assert!(matches!(error, HoldError::Exists { .. }), "{error}");
    holds::hold(conn, "elsewhere", id, Duration::from_secs(60))
        .await
        .unwrap();
    assert_eq!(count(conn, "SELECT count(*) AS n FROM blob_hold").await, 2);

    let other = BlobId::new();
    let ids = [id.uuid(), other.uuid()];
    assert_eq!(
        held(conn, "files", &ids).await,
        [id.uuid()].into_iter().collect()
    );
    assert!(
        held(conn, "nowhere", &ids).await.is_empty(),
        "a hold belongs to the store it names"
    );
}

#[tokio::test]
async fn extending_a_hold_never_shortens_it_and_never_revives_a_lapsed_one() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let id = BlobId::new();
    assert!(
        !holds::extend(conn, "files", id, Duration::from_secs(60))
            .await
            .unwrap(),
        "there is nothing to extend"
    );
    holds::hold(conn, "files", id, Duration::from_secs(600))
        .await
        .unwrap();

    assert!(
        holds::extend(conn, "files", id, Duration::from_secs(3600))
            .await
            .unwrap()
    );
    assert!(
        holds::extend(conn, "files", id, Duration::from_secs(60))
            .await
            .unwrap()
    );
    assert_eq!(
        count(
            conn,
            "SELECT count(*) AS n FROM blob_hold WHERE held_until > now() + interval '50 minutes'"
        )
        .await,
        1,
        "a shorter extension keeps the longer hold"
    );

    lapse(conn, id).await;
    assert!(
        !holds::extend(conn, "files", id, Duration::from_secs(3600))
            .await
            .unwrap(),
        "a lapsed hold stays lapsed"
    );
    assert_eq!(holds::take_expired(conn, 10).await.unwrap().len(), 1);
    assert!(
        !holds::extend(conn, "files", id, Duration::from_secs(3600))
            .await
            .unwrap(),
        "a swept hold is gone"
    );
    assert_eq!(count(conn, "SELECT count(*) AS n FROM blob_hold").await, 0);
}

#[tokio::test]
async fn a_released_hold_is_gone_once() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let id = BlobId::new();
    holds::hold(conn, "files", id, Duration::from_secs(600))
        .await
        .unwrap();

    assert!(holds::release(conn, "files", id).await.unwrap());
    assert!(!holds::release(conn, "files", id).await.unwrap());
    assert!(held(conn, "files", &[id.uuid()]).await.is_empty());
}

#[tokio::test]
async fn a_lapsed_hold_still_protects_until_the_sweep_takes_it() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let (old, live) = (BlobId::new(), BlobId::new());
    holds::hold(conn, "files", old, Duration::from_secs(600))
        .await
        .unwrap();
    holds::hold(conn, "files", live, Duration::from_secs(600))
        .await
        .unwrap();
    lapse(conn, old).await;

    let ids = [old.uuid(), live.uuid()];
    assert_eq!(held(conn, "files", &ids).await, ids.into_iter().collect());

    let taken = holds::take_expired(conn, 10).await.unwrap();
    assert_eq!(
        taken,
        vec![Expired {
            store: "files".to_owned(),
            id: old,
            named: false
        }]
    );
    assert!(holds::take_expired(conn, 10).await.unwrap().is_empty());
    assert_eq!(
        held(conn, "files", &ids).await,
        [live.uuid()].into_iter().collect()
    );
    holds::hold(conn, "files", old, Duration::from_secs(60))
        .await
        .unwrap();
}

#[tokio::test]
async fn expired_holds_are_taken_a_batch_at_a_time() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    for _ in 0..5 {
        holds::hold(conn, "files", BlobId::new(), Duration::from_secs(60))
            .await
            .unwrap();
    }
    conn.execute_unprepared("UPDATE blob_hold SET held_until = now() - interval '1 minute'")
        .await
        .unwrap();

    assert_eq!(holds::take_expired(conn, 2).await.unwrap().len(), 2);
    assert_eq!(holds::take_expired(conn, 2).await.unwrap().len(), 2);
    assert_eq!(holds::take_expired(conn, 2).await.unwrap().len(), 1);
    assert_eq!(count(conn, "SELECT count(*) AS n FROM blob_hold").await, 0);
}

async fn until(mut done: impl AsyncFnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !done().await {
        assert!(Instant::now() < deadline, "the condition never held");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn a_keeper_extends_its_holds_and_notices_one_that_is_gone() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let (kept, gone) = (BlobId::new(), BlobId::new());
    holds::hold(conn, "files", kept, Duration::from_secs(5))
        .await
        .unwrap();
    let keeper = holds::Keeper::start(
        conn.clone(),
        Duration::from_secs(3600),
        Duration::from_millis(20),
    );
    keeper.add("files", kept);
    until(async || {
        count(
            conn,
            "SELECT count(*) AS n FROM blob_hold WHERE held_until > now() + interval '30 minutes'",
        )
        .await
            == 1
    })
    .await;
    keeper.check().unwrap();

    keeper.add("files", gone);
    until(async || keeper.check().is_err()).await;
    drop(keeper);
}
