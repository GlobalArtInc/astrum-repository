use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use astrum_db::revision;
use astrum_policy::cache::Cache;
use astrum_policy::policy::{self, LicenseRule, Mode, Policy, SignatureRule, VulnerabilityRule};
use astrum_testkit::TestDatabase;
use sea_orm::ConnectionTrait;

fn policy_in(mode: Mode) -> Policy {
    Policy {
        name: "base".to_owned(),
        notes: String::new(),
        vulnerabilities: VulnerabilityRule {
            mode,
            ..VulnerabilityRule::default()
        },
        licenses: LicenseRule::default(),
        signatures: SignatureRule::default(),
        revision: 1,
    }
}

#[tokio::test]
async fn the_cache_follows_the_revision_and_serves_a_held_copy_only_inside_its_age() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &policy_in(Mode::Report))
        .await
        .unwrap();
    revision::bump(conn, revision::POLICIES).await.unwrap();
    let cache = Cache::new(Duration::from_secs(3600));

    let first = cache.get(conn).await.unwrap();
    assert_eq!(first.policies["base"].revision, 1);
    assert_eq!(
        first.policies["base"]
            .policy
            .as_ref()
            .unwrap()
            .vulnerabilities
            .mode,
        Mode::Report
    );

    policy::update(conn, &policy_in(Mode::Block)).await.unwrap();
    revision::bump(conn, revision::POLICIES).await.unwrap();
    let held = cache.get(conn).await.unwrap();
    assert_eq!(held.policies["base"].revision, 1);

    let fresh = cache.fresh(conn).await.unwrap();
    assert_eq!(fresh.policies["base"].revision, 2);
    assert_eq!(
        fresh.policies["base"]
            .policy
            .as_ref()
            .unwrap()
            .vulnerabilities
            .mode,
        Mode::Block
    );
    assert!(fresh.revision > first.revision);
    let again = cache.get(conn).await.unwrap();
    assert_eq!(again.policies["base"].revision, 2);

    let expiring = Cache::new(Duration::ZERO);
    expiring.get(conn).await.unwrap();
    conn.execute_unprepared("UPDATE content_policy SET revision = 9")
        .await
        .unwrap();
    revision::bump(conn, revision::POLICIES).await.unwrap();
    assert_eq!(
        expiring.get(conn).await.unwrap().policies["base"].revision,
        9
    );
}

#[tokio::test]
async fn a_policy_row_that_cannot_be_read_is_kept_as_an_error_not_dropped() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    conn.execute_unprepared(
        "INSERT INTO content_policy (name, vulnerabilities) VALUES ('broken', '{\"mode\": 5}')",
    )
    .await
    .unwrap();
    let snapshot = Cache::default().fresh(conn).await.unwrap();
    assert!(snapshot.policies["broken"].policy.is_err());
}

#[tokio::test]
async fn invalidate_makes_the_next_get_see_the_change_without_waiting_for_the_age() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &policy_in(Mode::Report))
        .await
        .unwrap();
    revision::bump(conn, revision::POLICIES).await.unwrap();
    let cache = Cache::new(Duration::from_secs(3600));
    cache.get(conn).await.unwrap();

    policy::update(conn, &policy_in(Mode::Block)).await.unwrap();
    revision::bump(conn, revision::POLICIES).await.unwrap();
    assert_eq!(cache.get(conn).await.unwrap().policies["base"].revision, 1);

    cache.invalidate();
    assert_eq!(cache.get(conn).await.unwrap().policies["base"].revision, 2);
}

#[tokio::test]
async fn simultaneous_gets_on_an_expired_cache_read_the_revision_once() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let reads = Arc::new(AtomicUsize::new(0));
    let mut conn = db.conn().clone();
    let counter = reads.clone();
    conn.set_metric_callback(move |info| {
        if info.statement.sql.contains("FROM revision WHERE name") {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    });
    let cache = Arc::new(Cache::new(Duration::from_secs(3600)));
    let mut gets = Vec::new();
    for _ in 0..16 {
        let (cache, conn) = (cache.clone(), conn.clone());
        gets.push(tokio::spawn(async move { cache.get(&conn).await.unwrap() }));
    }
    for get in gets {
        get.await.unwrap();
    }
    assert_eq!(reads.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn an_exception_that_cannot_be_read_is_skipped_and_counted_not_fatal() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    conn.execute_unprepared(
        "ALTER TABLE policy_exception DROP CONSTRAINT policy_exception_rule_check; \
         INSERT INTO policy_exception (id, rule, advisory, reason, created_by) VALUES \
           (gen_random_uuid(), 'vulnerability', 'CVE-1', 'fine', 'tester'), \
           (gen_random_uuid(), 'from-the-future', 'CVE-2', 'odd', 'tester')",
    )
    .await
    .unwrap();
    let cache = Cache::default();
    assert_eq!(cache.unreadable_exceptions(), 0);

    let snapshot = cache.fresh(conn).await.unwrap();

    assert_eq!(snapshot.exceptions.len(), 1);
    assert_eq!(snapshot.exceptions[0].reason, "fine");
    assert_eq!(snapshot.unreadable_exceptions, 1);
    assert_eq!(cache.unreadable_exceptions(), 1);
}
