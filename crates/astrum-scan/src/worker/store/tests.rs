use std::time::Duration;

use astrum_content::inventory::{Inventory, PackageRef};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};

use super::{claim, give_up, record_inventory, record_unsupported, retry_later};
use crate::PACKAGE_LOCK;
use crate::osv::purge;
use crate::worker::State;

const LEASE: Duration = Duration::from_secs(60);

async fn scalar(db: &DatabaseConnection, sql: &str) -> i64 {
    db.query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn setup() -> (TestDatabase, DatabaseConnection, i64) {
    let test = TestDatabase::create().await;
    let db = test.migrated().await.conn().clone();
    db.execute_unprepared(
        "INSERT INTO repository (name, format, type, attributes) \
         VALUES ('releases', 'maven2', 'hosted', '{}')",
    )
    .await
    .unwrap();
    let id = scalar(
        &db,
        "WITH made AS (INSERT INTO component (repository, format, name, version) \
         VALUES ('releases', 'maven2', 'lib', '1') RETURNING id) \
         SELECT id AS n FROM made",
    )
    .await;
    db.execute_unprepared(&format!(
        "INSERT INTO scan_queue (component_id, reason) VALUES ({id}, 'test')"
    ))
    .await
    .unwrap();
    (test, db, id)
}

fn inventory() -> Inventory {
    Inventory {
        key: "pkg:maven/lib@1".to_owned(),
        packages: vec![PackageRef {
            ecosystem: "Maven".to_owned(),
            name: "lib".to_owned(),
            version: "1".to_owned(),
            purl: "pkg:maven/lib@1".to_owned(),
        }],
        license: None,
        complete: true,
    }
}

#[tokio::test]
async fn a_worker_that_lost_its_lease_cannot_give_up_a_row_another_worker_holds() {
    let (_test, db, id) = setup().await;
    let stale = claim(&db, 10, LEASE).await.unwrap().remove(0);
    db.execute_unprepared("UPDATE scan_queue SET next_attempt_at = now()")
        .await
        .unwrap();
    let current = claim(&db, 10, LEASE).await.unwrap().remove(0);
    assert_eq!((stale.attempts, current.attempts), (1, 2));

    give_up(&db, stale, "too late").await.unwrap();
    assert_eq!(
        scalar(&db, "SELECT count(*) AS n FROM component_scan").await,
        0
    );
    assert_eq!(scalar(&db, "SELECT count(*) AS n FROM scan_queue").await, 1);

    give_up(&db, current, "failed").await.unwrap();
    assert_eq!(
        scalar(
            &db,
            &format!("SELECT count(*) AS n FROM component_scan WHERE component_id = {id} AND state = 'failed'")
        )
        .await,
        1
    );
    assert_eq!(scalar(&db, "SELECT count(*) AS n FROM scan_queue").await, 0);
}

#[tokio::test]
async fn recording_an_inventory_waits_while_the_purge_holds_the_package_lock() {
    let (_test, db, id) = setup().await;
    let item = claim(&db, 10, LEASE).await.unwrap().remove(0);
    let component = astrum_content::store::component_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    let purge_txn = db.begin().await.unwrap();
    purge_txn
        .execute_unprepared(&format!("SELECT pg_advisory_xact_lock({PACKAGE_LOCK})"))
        .await
        .unwrap();

    let writer = {
        let db = db.clone();
        tokio::spawn(async move {
            record_inventory(&db, item, &component, &inventory(), State::Scanned).await
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!writer.is_finished());
    purge_txn.commit().await.unwrap();
    assert!(writer.await.unwrap().unwrap());
    assert_eq!(
        scalar(&db, "SELECT count(*) AS n FROM sca_inventory_package").await,
        1
    );
}

#[tokio::test]
async fn the_purge_waits_for_a_writer_and_keeps_the_package_it_linked() {
    let (_test, db, _id) = setup().await;
    let writer = db.begin().await.unwrap();
    writer
        .execute_unprepared(&format!(
            "SELECT pg_advisory_xact_lock_shared({PACKAGE_LOCK})"
        ))
        .await
        .unwrap();
    writer
        .execute_unprepared(
            "INSERT INTO sca_package (id, ecosystem, name, version, purl) \
             VALUES (1, 'Maven', 'lib', '1', 'p'), (2, 'Maven', 'orphan', '1', 'q')",
        )
        .await
        .unwrap();
    let purger = {
        let db = db.clone();
        tokio::spawn(async move { purge::run(&db).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!purger.is_finished());
    writer
        .execute_unprepared(
            "INSERT INTO sca_inventory (id, key, source, extractor) VALUES (1, 'k', 'metadata', 1); \
             INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES (1, 1)",
        )
        .await
        .unwrap();
    writer.commit().await.unwrap();

    let purged = purger.await.unwrap().unwrap();
    assert_eq!(purged.packages, 1);
    assert_eq!(
        scalar(
            &db,
            "SELECT count(*) AS n FROM sca_package WHERE name = 'lib'"
        )
        .await,
        1
    );
    assert_eq!(
        scalar(&db, "SELECT count(*) AS n FROM sca_inventory_package").await,
        1
    );
}

async fn scan_state(db: &DatabaseConnection, id: i64) -> (i64, i64) {
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "SELECT count(*) AS n, count(*) FILTER (WHERE state = 'scanned' \
                 AND inventory_id = (SELECT id FROM sca_inventory WHERE key = 'pkg:maven/new@1')) AS fresh \
                 FROM component_scan WHERE component_id = {id}"
            ),
        ))
        .await
        .unwrap()
        .unwrap();
    (
        row.try_get("", "n").unwrap(),
        row.try_get("", "fresh").unwrap(),
    )
}

fn newer_inventory() -> Inventory {
    Inventory {
        key: "pkg:maven/new@1".to_owned(),
        packages: vec![PackageRef {
            ecosystem: "Maven".to_owned(),
            name: "new".to_owned(),
            version: "1".to_owned(),
            purl: "pkg:maven/new@1".to_owned(),
        }],
        license: None,
        complete: true,
    }
}

#[tokio::test]
async fn a_worker_that_lost_the_row_writes_no_result_over_the_newer_one() {
    let (_test, db, id) = setup().await;
    let component = astrum_content::store::component_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    let stale = claim(&db, 10, LEASE).await.unwrap().remove(0);
    db.execute_unprepared("UPDATE scan_queue SET next_attempt_at = now()")
        .await
        .unwrap();
    let current = claim(&db, 10, LEASE).await.unwrap().remove(0);

    assert!(
        !record_inventory(&db, stale, &component, &inventory(), State::Scanned)
            .await
            .unwrap()
    );
    assert!(!record_unsupported(&db, stale, &component).await.unwrap());
    assert_eq!(scan_state(&db, id).await, (0, 0));
    assert_eq!(scalar(&db, "SELECT count(*) AS n FROM scan_queue").await, 1);

    assert!(
        record_inventory(&db, current, &component, &newer_inventory(), State::Scanned)
            .await
            .unwrap()
    );
    assert_eq!(scan_state(&db, id).await, (1, 1));

    assert!(
        !record_inventory(&db, stale, &component, &inventory(), State::Scanned)
            .await
            .unwrap()
    );
    assert!(!record_unsupported(&db, stale, &component).await.unwrap());
    assert_eq!(scan_state(&db, id).await, (1, 1));
}

#[tokio::test]
async fn after_a_requeue_reset_a_worker_with_the_same_attempt_but_an_older_enqueue_time_writes_nothing()
 {
    let (_test, db, id) = setup().await;
    let component = astrum_content::store::component_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    let zombie = claim(&db, 10, LEASE).await.unwrap().remove(0);
    db.execute_unprepared("UPDATE scan_queue SET queued_at = now()")
        .await
        .unwrap();
    retry_later(&db, zombie, Duration::from_secs(60))
        .await
        .unwrap();
    assert_eq!(
        scalar(&db, "SELECT attempts::bigint AS n FROM scan_queue").await,
        0
    );
    let owner = claim(&db, 10, LEASE).await.unwrap().remove(0);
    assert_eq!(owner.attempts, zombie.attempts);
    assert_ne!(owner.queued_at, zombie.queued_at);
    assert_eq!(scan_state(&db, id).await, (0, 0));

    assert!(
        !record_inventory(&db, zombie, &component, &inventory(), State::Scanned)
            .await
            .unwrap()
    );
    give_up(&db, zombie, "too late").await.unwrap();
    assert_eq!(scan_state(&db, id).await, (0, 0));
}

#[tokio::test]
async fn the_purge_drops_layer_facts_before_it_waits_for_the_package_lock() {
    let (_test, db, _id) = setup().await;
    db.execute_unprepared(
        "INSERT INTO oci_layer_facts (digest, extractor, facts, extracted_at) \
         VALUES ('sha256:aa', 1, '{}', now() - interval '2 hours')",
    )
    .await
    .unwrap();
    let holder = db.begin().await.unwrap();
    holder
        .execute_unprepared(&format!(
            "SELECT pg_advisory_xact_lock_shared({PACKAGE_LOCK})"
        ))
        .await
        .unwrap();
    let purger = {
        let db = db.clone();
        tokio::spawn(async move { purge::run(&db).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!purger.is_finished());
    assert_eq!(
        scalar(&db, "SELECT count(*) AS n FROM oci_layer_facts").await,
        0
    );
    holder.commit().await.unwrap();
    assert_eq!(purger.await.unwrap().unwrap().layer_facts, 1);
}
