use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use astrum_content::zip;
use astrum_db::{Database, setting};
use astrum_scan::matcher::{self, Config, Metrics};
use astrum_scan::osv::{self, TempFile, Upload};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::json;
use tokio::sync::Notify;

struct Setup {
    _test: TestDatabase,
    db: Database,
}

async fn setup() -> Setup {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    Setup { _test: test, db }
}

async fn execute(s: &Setup, sql: &str) {
    s.db.conn().execute_unprepared(sql).await.unwrap();
}

async fn scalar(s: &Setup, sql: &str) -> i64 {
    s.db.conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn advisory(s: &Setup, id: &str, ecosystem: &str, name: &str, ranges: &str, versions: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO advisory (id, severity, modified_at, document) VALUES ('{id}', 'high', now(), '{{}}'); \
             INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges, versions) \
             VALUES ('{id}', '{ecosystem}', '{name}', '{ranges}', '{versions}')"
        ),
    )
    .await;
}

const EVERYTHING: &str = r#"[{"type":"SEMVER","events":[{"introduced":"0"}]}]"#;
const BELOW_1_2_3: &str = r#"[{"type":"SEMVER","events":[{"introduced":"0"},{"fixed":"1.2.3"}]}]"#;

async fn package(s: &Setup, ecosystem: &str, name: &str, version: &str) -> i64 {
    scalar(
        s,
        &format!(
            "INSERT INTO sca_package (ecosystem, name, version, purl) \
             VALUES ('{ecosystem}', '{name}', '{version}', 'pkg:{name}@{version}') RETURNING id AS n"
        ),
    )
    .await
}

fn quick() -> Config {
    Config {
        idle: Duration::from_millis(20),
        ..Config::default()
    }
}

fn names(changes: &[matcher::Change]) -> Vec<(i64, &str)> {
    changes
        .iter()
        .map(|change| (change.package_id, change.advisory_id.as_str()))
        .collect()
}

#[tokio::test]
async fn packages_match_by_range_and_list_and_garbage_matches_nothing() {
    let s = setup().await;
    advisory(&s, "NPM-1", "npm", "left-pad", BELOW_1_2_3, "{}").await;
    advisory(
        &s,
        "NPM-2",
        "npm",
        "left-pad",
        r#"[{"type":"SEMVER","events":[{"introduced":"2.0.0"}]}]"#,
        "{}",
    )
    .await;
    advisory(
        &s,
        "NPM-BAD",
        "npm",
        "left-pad",
        r#"[{"type":"SEMVER","events":[{"introduced":5}]}]"#,
        "{}",
    )
    .await;
    advisory(&s, "MVN-1", "Maven", "org.x:lib", "[]", "{3.1,3.2}").await;
    advisory(&s, "NPM-OLD", "npm", "left-pad", EVERYTHING, "{}").await;
    execute(
        &s,
        "UPDATE advisory SET withdrawn_at = now() WHERE id = 'NPM-OLD'",
    )
    .await;
    let hit = package(&s, "npm", "left-pad", "1.0.0").await;
    let fixed = package(&s, "npm", "left-pad", "1.2.3").await;
    let listed = package(&s, "Maven", "org.x:lib", "3.1").await;
    let unlisted = package(&s, "Maven", "org.x:lib", "3.3").await;
    let garbage = package(&s, "npm", "left-pad", "not a version!").await;
    let foreign = package(&s, "RubyGems", "rake", "1.0").await;

    let batch = matcher::match_batch(s.db.conn(), &Config::default())
        .await
        .unwrap();

    assert_eq!(batch.matched, 6);
    assert_eq!(names(&batch.added), [(hit, "NPM-1"), (listed, "MVN-1")]);
    assert!(batch.removed.is_empty());
    assert_eq!(batch.skipped_advisories, 2);
    assert_eq!(batch.unresolved_packages, 2);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_match").await, 2);
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE matched_at IS NULL"
        )
        .await,
        0
    );
    for id in [fixed, unlisted, garbage, foreign] {
        let count = scalar(
            &s,
            &format!("SELECT count(*) AS n FROM sca_match WHERE package_id = {id}"),
        )
        .await;
        assert_eq!(count, 0, "package {id}");
    }
    assert_eq!(
        scalar(
            &s,
            &format!(
                "SELECT count(*) AS n FROM sca_match WHERE package_id = {hit} AND fixed = '1.2.3'"
            )
        )
        .await,
        1
    );
}

#[tokio::test]
async fn a_name_with_more_advisories_than_the_cap_is_cut_and_counted() {
    let s = setup().await;
    for id in ["A", "B", "C"] {
        advisory(&s, id, "npm", "busy", EVERYTHING, "{}").await;
    }
    package(&s, "npm", "busy", "1.0.0").await;
    let config = Config {
        cap: 2,
        ..Config::default()
    };
    let batch = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(batch.truncated_packages, 1);
    assert_eq!(names(&batch.added), [(1, "A"), (1, "B")]);
}

async fn scanned(
    s: &Setup,
    repository: &str,
    component: i64,
    inventory: i64,
    package: Option<i64>,
) {
    execute(
        s,
        &format!(
            "INSERT INTO component (id, repository, format, name, version) \
             VALUES ({component}, '{repository}', 'npm', 'c{component}', '1'); \
             INSERT INTO sca_inventory (id, key, source, extractor) VALUES ({inventory}, 'k{inventory}', 'metadata', 1) \
             ON CONFLICT DO NOTHING; \
             INSERT INTO component_scan (component_id, repository, inventory_id, state) \
             VALUES ({component}, '{repository}', {inventory}, 'scanned')"
        ),
    )
    .await;
    if let Some(package) = package {
        execute(
            s,
            &format!(
                "INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES ({inventory}, {package}) \
                 ON CONFLICT DO NOTHING"
            ),
        )
        .await;
    }
}

async fn repositories(s: &Setup) {
    execute(
        s,
        "INSERT INTO repository (name, format, type, attributes) VALUES \
         ('one', 'npm', 'hosted', '{}'), ('two', 'npm', 'hosted', '{}')",
    )
    .await;
}

#[tokio::test]
async fn a_changed_match_queues_the_components_and_the_set_is_replaced_not_appended() {
    let s = setup().await;
    repositories(&s).await;
    advisory(&s, "A", "npm", "lib", BELOW_1_2_3, "{}").await;
    let lib = package(&s, "npm", "lib", "1.0.0").await;
    scanned(&s, "one", 10, 1, Some(lib)).await;
    let config = Config::default();

    let first = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(names(&first.added), [(lib, "A")]);
    assert_eq!(first.marked.len(), 1);
    matcher::fan_out(s.db.conn(), &config, &first.marked).await;
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE repository = 'one' AND subject = 'c:10'"
        )
        .await,
        1
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE rejudge_at IS NOT NULL"
        )
        .await,
        0
    );

    execute(
        &s,
        "UPDATE sca_match SET first_seen_at = now() - interval '2 days'",
    )
    .await;
    advisory(&s, "B", "npm", "lib", EVERYTHING, "{}").await;
    execute(&s, "UPDATE advisory_affected SET ranges = '[{\"type\":\"SEMVER\",\"events\":[{\"introduced\":\"0\"},{\"fixed\":\"1.5.0\"}]}]' WHERE advisory_id = 'A'").await;
    execute(&s, "UPDATE sca_package SET matched_at = NULL").await;
    let second = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(names(&second.added), [(lib, "B")]);
    assert_eq!(names(&second.updated), [(lib, "A")]);
    assert!(second.removed.is_empty());
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM sca_match WHERE advisory_id = 'A' AND fixed = '1.5.0' AND first_seen_at < now() - interval '1 day'").await,
        1
    );

    execute(&s, "DELETE FROM advisory WHERE id IN ('A', 'B')").await;
    execute(&s, "UPDATE sca_package SET matched_at = NULL").await;
    execute(&s, "DELETE FROM policy_dirty").await;
    let third = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert!(third.added.is_empty());
    assert!(third.removed.is_empty());
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_match").await, 0);
    matcher::fan_out(s.db.conn(), &config, &third.marked).await;
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        1
    );
}

#[tokio::test]
async fn a_package_in_many_components_and_repositories_is_queued_in_chunks() {
    let s = setup().await;
    repositories(&s).await;
    advisory(&s, "A", "npm", "lib", EVERYTHING, "{}").await;
    let lib = package(&s, "npm", "lib", "1.0.0").await;
    let other = package(&s, "npm", "other", "1.0.0").await;
    for component in 1..=7 {
        let repository = if component % 2 == 0 { "one" } else { "two" };
        scanned(&s, repository, component, 1 + component % 3, Some(lib)).await;
    }
    scanned(&s, "one", 20, 30, Some(other)).await;
    scanned(&s, "one", 21, 31, None).await;
    execute(&s, "INSERT INTO component_scan (component_id, repository, inventory_id, state) SELECT 22, 'ghost', 1, 'scanned' WHERE false").await;
    let config = Config {
        chunk: 2,
        ..Config::default()
    };
    let batch = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(batch.marked.len(), 2);
    let marked: Vec<matcher::Marked> = batch
        .marked
        .iter()
        .copied()
        .filter(|item| item.package_id == lib)
        .collect();
    let done = matcher::fan_out(s.db.conn(), &config, &marked).await;
    assert_eq!((done.packages, done.subjects, done.failed), (1, 7, 0));
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        7
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE subject IN ('c:20', 'c:21')"
        )
        .await,
        0
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE repository = 'one'"
        )
        .await,
        3
    );
    let again = matcher::fan_out(s.db.conn(), &config, &marked).await;
    assert_eq!(again.failed, 0);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        7
    );
}

#[tokio::test]
async fn a_crash_after_the_match_commit_loses_no_mark() {
    let s = setup().await;
    repositories(&s).await;
    advisory(&s, "A", "npm", "lib", EVERYTHING, "{}").await;
    let lib = package(&s, "npm", "lib", "1.0.0").await;
    scanned(&s, "one", 10, 1, Some(lib)).await;
    let config = Config::default();

    matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        0
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE rejudge_at IS NOT NULL"
        )
        .await,
        1
    );
    let early = matcher::sweep(s.db.conn(), &config).await.unwrap();
    assert_eq!(early.packages, 0);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        0
    );

    let overdue = Config {
        grace: Duration::ZERO,
        ..Config::default()
    };
    let swept = matcher::sweep(s.db.conn(), &overdue).await.unwrap();
    assert_eq!((swept.packages, swept.subjects), (1, 1));
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE subject = 'c:10'"
        )
        .await,
        1
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE rejudge_at IS NOT NULL"
        )
        .await,
        0
    );

    execute(&s, "DELETE FROM policy_dirty; \
         UPDATE advisory_affected SET ranges = '[{\"type\":\"SEMVER\",\"events\":[{\"introduced\":\"5.0.0\"}]}]'; \
         UPDATE sca_package SET matched_at = NULL").await;
    let removal = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(names(&removal.removed), [(lib, "A")]);
    matcher::sweep(s.db.conn(), &overdue).await.unwrap();
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE subject = 'c:10'"
        )
        .await,
        1
    );
}

fn advisory_file(id: &str, modified: &str, fixed: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": id,
        "modified": modified,
        "summary": "a flaw",
        "database_specific": {"severity": "HIGH"},
        "affected": [{
            "package": {"ecosystem": "npm", "name": "lib"},
            "ranges": [{"type": "SEMVER", "events": [{"introduced": "0"}, {"fixed": fixed}]}]
        }]
    }))
    .unwrap()
}

async fn import(
    db: &sea_orm::DatabaseConnection,
    dir: &std::path::Path,
    modified: &str,
    fixed: &str,
) {
    let name = "A.json".to_owned();
    let file = advisory_file("A", modified, fixed);
    let archive = zip::write(&[(name.as_str(), file.as_slice())]);
    let staged = TempFile::from_stream(
        dir,
        futures_util::stream::iter(vec![Ok::<_, Infallible>(archive)]),
        osv::MAX_ARCHIVE_BYTES,
    )
    .await
    .unwrap();
    let source = Upload::new(staged);
    osv::sync_with(db, &source, "npm", true, &|| false)
        .await
        .unwrap();
}

const WAITING_ADVISORY_LOCKS: &str = "SELECT count(*) AS n FROM pg_locks WHERE locktype = 'advisory' \
     AND NOT granted AND database = (SELECT oid FROM pg_database WHERE datname = current_database())";

#[tokio::test]
async fn an_import_waits_for_a_matcher_that_is_midway_and_then_unmatches_the_package() {
    let s = setup().await;
    let dir = tempfile::tempdir().unwrap();
    import(s.db.conn(), dir.path(), "2026-01-01T00:00:00Z", "1.2.3").await;
    let lib = package(&s, "npm", "lib", "1.0.0").await;
    let reached = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());

    let matching = {
        let (db, reached, resume) = (s.db.conn().clone(), reached.clone(), resume.clone());
        tokio::spawn(async move {
            matcher::match_batch_with(&db, &Config::default(), || async move {
                reached.notify_one();
                resume.notified().await;
            })
            .await
        })
    };
    reached.notified().await;
    let importing = {
        let (db, path) = (s.db.conn().clone(), dir.path().to_owned());
        tokio::spawn(async move { import(&db, &path, "2026-02-01T00:00:00Z", "0.5.0").await })
    };
    tokio::time::timeout(Duration::from_secs(30), async {
        while scalar(&s, WAITING_ADVISORY_LOCKS).await == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the import never queued for the advisory lock");
    assert!(
        !importing.is_finished(),
        "the import ran while a matcher held its read"
    );

    resume.notify_one();
    let batch = matching.await.unwrap().unwrap();
    importing.await.unwrap();

    assert_eq!(names(&batch.added), [(lib, "A")]);
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE matched_at IS NULL"
        )
        .await,
        1
    );
    let next = matcher::match_batch(s.db.conn(), &Config::default())
        .await
        .unwrap();
    assert_eq!(names(&next.removed), [(lib, "A")]);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_match").await, 0);
}

async fn unfinished_markers(s: &Setup) -> i64 {
    scalar(
        s,
        "SELECT count(*) AS n FROM sca_package WHERE rejudge_at IS NOT NULL",
    )
    .await
}

#[tokio::test]
async fn clearing_the_markers_does_not_wait_for_a_package_another_transaction_holds() {
    let s = setup().await;
    repositories(&s).await;
    advisory(&s, "A", "npm", "lib", EVERYTHING, "{}").await;
    let held = package(&s, "npm", "lib", "1.0.0").await;
    let free = package(&s, "npm", "lib", "1.1.0").await;
    scanned(&s, "one", 10, 1, Some(held)).await;
    scanned(&s, "one", 11, 2, Some(free)).await;
    let config = Config::default();
    let batch = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_eq!(batch.marked.len(), 2);

    let holder = s.db.conn().begin().await.unwrap();
    holder
        .execute_unprepared(&format!(
            "SELECT id FROM sca_package WHERE id = {held} FOR UPDATE"
        ))
        .await
        .unwrap();
    let done = tokio::time::timeout(
        Duration::from_secs(10),
        matcher::fan_out(s.db.conn(), &config, &batch.marked),
    )
    .await
    .expect("the fan-out waited for a package row another transaction holds");
    assert_eq!((done.packages, done.failed), (2, 0));
    assert_eq!(unfinished_markers(&s).await, 1);
    assert_eq!(
        scalar(
            &s,
            &format!(
                "SELECT count(*) AS n FROM sca_package WHERE id = {held} AND rejudge_at IS NOT NULL"
            )
        )
        .await,
        1
    );
    holder.commit().await.unwrap();

    let overdue = Config {
        grace: Duration::ZERO,
        ..Config::default()
    };
    let swept = matcher::sweep(s.db.conn(), &overdue).await.unwrap();
    assert_eq!(swept.packages, 1);
    assert_eq!(unfinished_markers(&s).await, 0);
}

#[tokio::test]
async fn a_marker_survives_the_clearing_by_the_token_of_an_older_match() {
    let s = setup().await;
    repositories(&s).await;
    advisory(&s, "A", "npm", "lib", EVERYTHING, "{}").await;
    let lib = package(&s, "npm", "lib", "1.0.0").await;
    scanned(&s, "one", 10, 1, Some(lib)).await;
    let config = Config::default();
    let first = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    execute(&s, "UPDATE sca_package SET matched_at = NULL").await;
    let second = matcher::match_batch(s.db.conn(), &config).await.unwrap();
    assert_ne!(first.marked[0].token, second.marked[0].token);

    matcher::fan_out(s.db.conn(), &config, &first.marked).await;
    assert_eq!(unfinished_markers(&s).await, 1);
    matcher::fan_out(s.db.conn(), &config, &second.marked).await;
    assert_eq!(unfinished_markers(&s).await, 0);
}

#[tokio::test]
async fn every_step_sweeps_an_overdue_marker_even_while_packages_wait_to_be_matched() {
    let s = setup().await;
    repositories(&s).await;
    advisory(&s, "A", "npm", "lib", EVERYTHING, "{}").await;
    let overdue = package(&s, "npm", "lib", "1.0.0").await;
    scanned(&s, "one", 10, 1, Some(overdue)).await;
    execute(
        &s,
        "UPDATE sca_package SET matched_at = now(), rejudge_at = now() - interval '1 hour'",
    )
    .await;
    package(&s, "npm", "lib", "1.1.0").await;
    package(&s, "npm", "lib", "1.2.0").await;
    let config = Config {
        batch: 1,
        ..Config::default()
    };
    let metrics = Metrics::default();

    assert!(matcher::step(s.db.conn(), &config, &metrics).await);
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE matched_at IS NULL"
        )
        .await,
        1
    );
    assert_eq!(unfinished_markers(&s).await, 0);
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE subject = 'c:10'"
        )
        .await,
        1
    );

    assert!(matcher::step(s.db.conn(), &config, &metrics).await);
    assert!(!matcher::step(s.db.conn(), &config, &metrics).await);
    assert_eq!(metrics.snapshot().count, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_matchers_split_two_thousand_packages_without_overlap_or_deadlock() {
    let s = setup().await;
    execute(
        &s,
        "INSERT INTO advisory (id, severity, modified_at, document) \
         SELECT 'ADV-' || n, 'high', now(), '{}' FROM generate_series(0, 49) n; \
         INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges, versions) \
         SELECT 'ADV-' || n, 'npm', 'lib' || n, '[{\"type\":\"SEMVER\",\"events\":[{\"introduced\":\"0\"}]}]', '{}' \
         FROM generate_series(0, 49) n; \
         INSERT INTO sca_package (ecosystem, name, version, purl) \
         SELECT 'npm', 'lib' || (n % 50), '1.' || n || '.0', 'p' || n FROM generate_series(1, 2000) n",
    )
    .await;
    let config = Config {
        batch: 100,
        ..Config::default()
    };
    let mut tasks = Vec::new();
    for _ in 0..3 {
        let (db, config) = (s.db.conn().clone(), config.clone());
        tasks.push(tokio::spawn(async move {
            let mut matched = 0;
            let mut added = 0;
            loop {
                let batch = matcher::match_batch(&db, &config).await.unwrap();
                if batch.matched == 0 {
                    return (matched, added);
                }
                matched += batch.matched;
                added += batch.added.len();
            }
        }));
    }
    let mut total = (0, 0);
    for task in tasks {
        let (matched, added) = tokio::time::timeout(Duration::from_secs(120), task)
            .await
            .expect("the matchers deadlocked")
            .unwrap();
        total.0 += matched;
        total.1 += added;
    }
    assert_eq!(total, (2000, 2000));
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM sca_match").await,
        2000
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM sca_package WHERE matched_at IS NULL"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn zero_matchers_run_nothing_and_the_setting_is_read_and_capped() {
    let s = setup().await;
    assert_eq!(Config::load(s.db.conn()).await.matchers, 1);
    setting::put(s.db.conn(), matcher::MATCHERS_KEY, &9)
        .await
        .unwrap();
    assert_eq!(
        Config::load(s.db.conn()).await.matchers,
        matcher::MAX_MATCHERS
    );
    setting::put(s.db.conn(), matcher::MATCHERS_KEY, &0)
        .await
        .unwrap();
    let off = Config::load(s.db.conn()).await;
    assert_eq!(off.matchers, 0);

    advisory(&s, "A", "npm", "lib", EVERYTHING, "{}").await;
    package(&s, "npm", "lib", "1.0.0").await;
    let metrics = Arc::new(Metrics::default());
    let idle = matcher::spawn(
        s.db.conn().clone(),
        Config {
            idle: Duration::from_millis(20),
            ..off
        },
        metrics.clone(),
    );
    assert_eq!(idle.count(), 0);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_match").await, 0);

    let running = matcher::spawn(s.db.conn().clone(), quick(), metrics.clone());
    assert_eq!(running.count(), 1);
    for _ in 0..100 {
        if scalar(&s, "SELECT count(*) AS n FROM sca_match").await == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    running.abort();
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_match").await, 1);
    assert_eq!(metrics.snapshot().added, 1);
}
