use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use astrum_db::{Database, setting};
use astrum_policy::cache::Cache;
use astrum_scan::verdict::{self, Config, Metrics, SweepState, Swept};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
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

async fn text(s: &Setup, sql: &str) -> String {
    s.db.conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

const BLOCK_HIGH: &str = r#"{"mode":"block","severity":"high"}"#;

async fn world(s: &Setup, policy_json: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO content_policy (name, vulnerabilities) VALUES ('base', '{policy_json}'); \
             INSERT INTO repository (name, format, type, attributes) VALUES \
               ('one', 'npm', 'hosted', '{{\"policy\":{{\"name\":\"base\"}}}}'), \
               ('free', 'npm', 'hosted', '{{}}'); \
             INSERT INTO advisory (id, aliases, severity, modified_at, document) \
               VALUES ('GHSA-1', '{{CVE-1}}', 'high', now(), '{{}}'), ('LOW-1', '{{}}', 'low', now(), '{{}}')"
        ),
    )
    .await;
}

async fn component(s: &Setup, repository: &str, id: i64, advisory: Option<&str>) {
    execute(
        s,
        &format!(
            "INSERT INTO component (id, repository, format, name, version) \
               VALUES ({id}, '{repository}', 'npm', 'c{id}', '1'); \
             INSERT INTO sca_package (id, ecosystem, name, version, purl, matched_at) \
               VALUES ({id}, 'npm', 'c{id}', '1', 'pkg:npm/c{id}@1', now()); \
             INSERT INTO sca_inventory (id, key, source, extractor) VALUES ({id}, 'k{id}', 'metadata', 1); \
             INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES ({id}, {id}); \
             INSERT INTO component_scan (component_id, repository, inventory_id, state) \
               VALUES ({id}, '{repository}', {id}, 'scanned')"
        ),
    )
    .await;
    if let Some(advisory) = advisory {
        execute(
            s,
            &format!("INSERT INTO sca_match (package_id, advisory_id) VALUES ({id}, '{advisory}')"),
        )
        .await;
    }
    mark(s, repository, id).await;
}

async fn mark(s: &Setup, repository: &str, id: i64) {
    execute(
        s,
        &format!(
            "INSERT INTO policy_dirty (repository, subject) VALUES ('{repository}', 'c:{id}') \
             ON CONFLICT DO NOTHING"
        ),
    )
    .await;
}

fn config() -> Config {
    Config {
        idle: Duration::from_millis(20),
        ..Config::default()
    }
}

async fn judge(s: &Setup, cache: &Cache) -> verdict::Judged {
    verdict::judge_batch(s.db.conn(), cache, &config())
        .await
        .unwrap()
}

#[tokio::test]
async fn components_get_verdicts_and_marks_for_free_or_vanished_subjects_are_dropped() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    component(&s, "one", 2, Some("LOW-1")).await;
    component(&s, "one", 3, None).await;
    component(&s, "free", 4, Some("GHSA-1")).await;
    execute(
        &s,
        "INSERT INTO policy_verdict (repository, subject, policy, component_id, blocked, state, policy_revision) \
         VALUES ('free', 'c:4', 'base', 4, true, 'block', 1)",
    )
    .await;
    mark(&s, "one", 99).await;
    let cache = Cache::default();

    let judged = judge(&s, &cache).await;

    assert_eq!(judged.taken, 5);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        0
    );
    assert_eq!(
        text(
            &s,
            "SELECT string_agg(subject || ':' || state || ':' || blocked::text || ':' || high || ':' || low, ',' ORDER BY subject) AS n FROM policy_verdict"
        )
        .await,
        "c:1:block:true:1:0,c:2:pass:false:0:1,c:3:pass:false:0:0"
    );
    assert_eq!(
        text(
            &s,
            "SELECT max_severity AS n FROM policy_verdict WHERE subject = 'c:1'"
        )
        .await,
        "high"
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_verdict WHERE subject = 'c:3' AND max_severity IS NULL AND policy = 'base' AND policy_revision = 1"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn a_package_still_waiting_for_its_match_counts_as_unscanned_and_off_removes_the_verdict() {
    let s = setup().await;
    world(
        &s,
        r#"{"mode":"block","severity":"high","unscanned":"block"}"#,
    )
    .await;
    component(&s, "one", 1, None).await;
    execute(&s, "UPDATE sca_package SET matched_at = NULL").await;
    let cache = Cache::default();

    judge(&s, &cache).await;
    assert_eq!(
        text(
            &s,
            "SELECT state || ':' || blocked::text AS n FROM policy_verdict"
        )
        .await,
        "unscanned:true"
    );

    execute(&s, "UPDATE sca_package SET matched_at = now()").await;
    mark(&s, "one", 1).await;
    judge(&s, &cache).await;
    assert_eq!(
        text(
            &s,
            "SELECT state || ':' || blocked::text AS n FROM policy_verdict"
        )
        .await,
        "pass:false"
    );

    execute(
        &s,
        "UPDATE content_policy SET vulnerabilities = '{\"mode\":\"off\"}'",
    )
    .await;
    astrum_db::revision::bump(s.db.conn(), astrum_db::revision::POLICIES)
        .await
        .unwrap();
    mark(&s, "one", 1).await;
    let judged = judge(&s, &cache).await;
    assert_eq!(judged.dropped, 1);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_verdict").await,
        0
    );
}

#[tokio::test]
async fn a_policy_that_cannot_be_read_or_is_missing_blocks_with_the_error_state() {
    let s = setup().await;
    world(&s, r#"{"mode":5}"#).await;
    execute(
        &s,
        "INSERT INTO repository (name, format, type, attributes) \
         VALUES ('ghost', 'npm', 'hosted', '{\"policy\":{\"name\":\"nobody\"}}')",
    )
    .await;
    component(&s, "one", 1, None).await;
    component(&s, "ghost", 2, None).await;

    judge(&s, &Cache::default()).await;

    assert_eq!(
        text(
            &s,
            "SELECT string_agg(subject || ':' || state || ':' || blocked::text, ',' ORDER BY subject) AS n FROM policy_verdict"
        )
        .await,
        "c:1:error:true,c:2:error:true"
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_verdict WHERE reasons -> 0 ->> 'kind' = 'policy-error'"
        )
        .await,
        2
    );
}

#[tokio::test]
async fn a_mark_inserted_while_a_verdict_is_computed_survives_the_commit() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    let cache = Cache::default();
    let reached = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());

    let judging = {
        let (db, reached, resume) = (s.db.conn().clone(), reached.clone(), resume.clone());
        tokio::spawn(async move {
            verdict::judge_batch_with(&db, &cache, &config(), || async move {
                reached.notify_one();
                resume.notified().await;
            })
            .await
        })
    };
    reached.notified().await;
    let inserting = {
        let db = s.db.conn().clone();
        tokio::spawn(async move {
            db.execute_unprepared(
                "INSERT INTO policy_dirty (repository, subject) VALUES ('one', 'c:1') ON CONFLICT DO NOTHING",
            )
            .await
        })
    };
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(
        !inserting.is_finished(),
        "the insert did not wait for the drain"
    );

    resume.notify_one();
    judging.await.unwrap().unwrap();
    inserting.await.unwrap().unwrap();

    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE repository = 'one' AND subject = 'c:1'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn three_workers_judge_every_subject_once_and_never_deadlock() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    for id in 1..=300 {
        component(&s, "one", id, (id % 2 == 0).then_some("GHSA-1")).await;
    }
    let taken = Arc::new(AtomicU64::new(0));
    let cache = Arc::new(Cache::default());
    let mut workers = Vec::new();
    for _ in 0..3 {
        let (db, taken, cache) = (s.db.conn().clone(), taken.clone(), cache.clone());
        workers.push(tokio::spawn(async move {
            let config = Config {
                batch: 20,
                ..config()
            };
            loop {
                let judged = verdict::judge_batch(&db, &cache, &config).await.unwrap();
                if judged.taken == 0 {
                    if judged.busy {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                    return;
                }
                taken.fetch_add(judged.taken, Ordering::Relaxed);
            }
        }));
    }
    for worker in workers {
        worker.await.unwrap();
    }

    assert_eq!(taken.load(Ordering::Relaxed), 300);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_verdict").await,
        300
    );
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_verdict WHERE blocked").await,
        150
    );
}

#[tokio::test]
async fn a_repository_delete_arriving_mid_verdict_waits_and_does_not_deadlock() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    let holder = s.db.conn().begin().await.unwrap();
    holder
        .execute_unprepared("SELECT 1 FROM component WHERE id = 1 FOR UPDATE")
        .await
        .unwrap();

    let judging = {
        let db = s.db.conn().clone();
        tokio::spawn(async move { verdict::judge_batch(&db, &Cache::default(), &config()).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    let deleting = {
        let db = s.db.conn().clone();
        tokio::spawn(async move {
            db.execute_unprepared("DELETE FROM repository WHERE name = 'one'")
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!judging.is_finished());
    assert!(!deleting.is_finished());
    holder.commit().await.unwrap();

    tokio::time::timeout(Duration::from_secs(20), async {
        judging.await.unwrap().unwrap();
        deleting.await.unwrap().unwrap();
    })
    .await
    .expect("the verdict and the repository delete deadlocked");
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM repository WHERE name = 'one'"
        )
        .await,
        0
    );
    for table in ["component", "policy_verdict", "policy_dirty"] {
        assert_eq!(
            scalar(&s, &format!("SELECT count(*) AS n FROM {table}")).await,
            0,
            "{table}"
        );
    }
}

async fn exception(s: &Setup, extra: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO policy_exception (id, rule, advisory, reason, created_by, expires_at) \
             VALUES (gen_random_uuid(), 'vulnerability', 'CVE-1', 'accepted', 'tester', {extra})"
        ),
    )
    .await;
}

#[tokio::test]
async fn an_exception_unblocks_and_the_sweeper_re_blocks_after_it_expires() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    exception(&s, "now() + interval '2 seconds'").await;
    let cache = Cache::default();

    judge(&s, &cache).await;
    assert_eq!(
        text(
            &s,
            "SELECT state || ':' || blocked::text AS n FROM policy_verdict"
        )
        .await,
        "pass:false"
    );
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_verdict WHERE reasons @> '[{\"kind\":\"exception\",\"reason\":\"accepted\"}]'"
        )
        .await,
        1
    );

    let mut state = SweepState::default();
    verdict::sweep(s.db.conn(), &config(), &mut state)
        .await
        .unwrap();
    execute(&s, "DELETE FROM policy_dirty").await;
    tokio::time::sleep(Duration::from_millis(2300)).await;
    let swept = verdict::sweep(s.db.conn(), &config(), &mut state)
        .await
        .unwrap();
    assert_eq!(swept, Swept::Done { marked: 1 });
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE subject = 'c:1'"
        )
        .await,
        1
    );

    judge(&s, &cache).await;
    assert_eq!(
        text(
            &s,
            "SELECT state || ':' || blocked::text AS n FROM policy_verdict"
        )
        .await,
        "block:true"
    );
    let again = verdict::sweep(s.db.conn(), &config(), &mut state)
        .await
        .unwrap();
    assert_eq!(again, Swept::Done { marked: 0 });
}

#[tokio::test]
async fn a_new_policy_revision_and_a_changed_exception_mark_the_subjects_even_after_a_restart() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    component(&s, "one", 2, None).await;
    let cache = Cache::default();
    judge(&s, &cache).await;
    let mut state = SweepState::default();
    let first = verdict::sweep(s.db.conn(), &config(), &mut state)
        .await
        .unwrap();
    assert_eq!(first, Swept::Done { marked: 1 });
    judge(&s, &cache).await;
    let quiet = verdict::sweep(s.db.conn(), &config(), &mut state)
        .await
        .unwrap();
    assert_eq!(quiet, Swept::Done { marked: 0 });

    execute(&s, "UPDATE content_policy SET revision = revision + 1").await;
    astrum_db::revision::bump(s.db.conn(), astrum_db::revision::POLICIES)
        .await
        .unwrap();
    let mut restarted = SweepState::default();
    let swept = verdict::sweep(s.db.conn(), &config(), &mut restarted)
        .await
        .unwrap();
    assert_eq!(swept, Swept::Done { marked: 2 });
    judge(&s, &cache).await;
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_verdict WHERE policy_revision = 2"
        )
        .await,
        2
    );

    exception(&s, "NULL").await;
    astrum_db::revision::bump(s.db.conn(), astrum_db::revision::POLICIES)
        .await
        .unwrap();
    let after_exception = verdict::sweep(s.db.conn(), &config(), &mut state)
        .await
        .unwrap();
    assert_eq!(after_exception, Swept::Done { marked: 1 });
    assert_eq!(
        scalar(
            &s,
            "SELECT count(*) AS n FROM policy_dirty WHERE subject = 'c:1'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn only_one_node_sweeps_at_a_time() {
    let s = setup().await;
    let holder = s.db.conn().begin().await.unwrap();
    holder
        .execute_unprepared("SELECT pg_advisory_xact_lock(hashtext('policy.sweep'))")
        .await
        .unwrap();
    let busy = verdict::sweep(s.db.conn(), &config(), &mut SweepState::default())
        .await
        .unwrap();
    assert_eq!(busy, Swept::Busy);
    holder.commit().await.unwrap();
    let free = verdict::sweep(s.db.conn(), &config(), &mut SweepState::default())
        .await
        .unwrap();
    assert!(matches!(free, Swept::Done { .. }));
}

#[tokio::test]
async fn zero_verdict_workers_run_nothing_and_the_setting_is_read_and_capped() {
    let s = setup().await;
    assert_eq!(Config::load(s.db.conn()).await.workers, 1);
    setting::put(s.db.conn(), verdict::WORKERS_KEY, &9)
        .await
        .unwrap();
    assert_eq!(
        Config::load(s.db.conn()).await.workers,
        verdict::MAX_WORKERS
    );
    setting::put(s.db.conn(), verdict::WORKERS_KEY, &0)
        .await
        .unwrap();
    let off = Config::load(s.db.conn()).await;
    assert_eq!(off.workers, 0);

    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    let running = verdict::spawn(
        s.db.conn().clone(),
        Config {
            idle: Duration::from_millis(20),
            ..off
        },
        Arc::default(),
        Arc::new(Metrics::default()),
    );
    assert_eq!(running.count(), 0);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_verdict").await,
        0
    );
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
        1
    );
}

const WAITING_TRANSACTIONS: &str = "SELECT count(*) AS n FROM pg_locks l \
     JOIN pg_stat_activity a ON a.pid = l.pid \
     WHERE l.locktype = 'transactionid' AND NOT l.granted AND a.datname = current_database()";

#[tokio::test]
async fn a_worker_whose_oldest_marks_are_held_by_another_is_told_to_retry_soon() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    execute(
        &s,
        "INSERT INTO repository (name, format, type, attributes) \
         VALUES ('two', 'npm', 'hosted', '{\"policy\":{\"name\":\"base\"}}')",
    )
    .await;
    component(&s, "one", 1, None).await;
    component(&s, "one", 2, None).await;
    component(&s, "two", 3, None).await;
    component(&s, "two", 4, None).await;
    let holder = s.db.conn().begin().await.unwrap();
    holder
        .execute_unprepared("SELECT 1 FROM policy_dirty WHERE repository = 'one' FOR UPDATE")
        .await
        .unwrap();
    let config = Config {
        batch: 2,
        ..config()
    };
    let cache = Cache::default();

    let blocked = verdict::judge_batch(s.db.conn(), &cache, &config)
        .await
        .unwrap();
    assert_eq!((blocked.taken, blocked.busy), (0, true));

    holder.rollback().await.unwrap();
    let freed = verdict::judge_batch(s.db.conn(), &cache, &config)
        .await
        .unwrap();
    assert_eq!((freed.taken, freed.busy), (2, false));
    let idle = Config {
        batch: 10,
        ..config
    };
    verdict::judge_batch(s.db.conn(), &cache, &idle)
        .await
        .unwrap();
    let empty = verdict::judge_batch(s.db.conn(), &cache, &idle)
        .await
        .unwrap();
    assert_eq!((empty.taken, empty.busy), (0, false));
}

#[tokio::test]
async fn a_policy_change_between_the_calculation_and_the_commit_marks_the_batch_again() {
    let s = setup().await;
    world(&s, BLOCK_HIGH).await;
    component(&s, "one", 1, Some("GHSA-1")).await;
    component(&s, "one", 2, None).await;
    let cache = Cache::default();
    let db = s.db.conn().clone();

    let judged = verdict::judge_batch_with(s.db.conn(), &cache, &config(), || async move {
        astrum_db::revision::bump(&db, astrum_db::revision::POLICIES)
            .await
            .unwrap();
    })
    .await
    .unwrap();

    assert_eq!(judged.taken, 2);
    assert_eq!(
        scalar(&s, "SELECT count(*) AS n FROM policy_verdict").await,
        2
    );
    assert_eq!(
        text(
            &s,
            "SELECT string_agg(subject, ',' ORDER BY subject) AS n FROM policy_dirty"
        )
        .await,
        "c:1,c:2"
    );
}

async fn crossing_world(s: &Setup) {
    execute(
        s,
        "INSERT INTO content_policy (name, vulnerabilities) VALUES ('base', '{\"mode\":\"block\",\"severity\":\"high\"}'); \
         INSERT INTO repository (name, format, type, attributes) VALUES \
           ('a', 'npm', 'hosted', '{\"policy\":{\"name\":\"base\"}}'), \
           ('b', 'npm', 'hosted', '{\"policy\":{\"name\":\"base\"}}'); \
         INSERT INTO advisory (id, aliases, severity, modified_at, document) \
           VALUES ('GHSA-1', '{}', 'high', now(), '{}')",
    )
    .await;
    component(s, "a", 20, Some("GHSA-1")).await;
    component(s, "b", 10, Some("GHSA-1")).await;
    execute(s, "DELETE FROM policy_dirty").await;
}

const CROSSING_VERDICTS: &str = "INSERT INTO policy_verdict (repository, subject, policy, component_id, blocked, state, max_severity, policy_revision, computed_at, reasons) \
     VALUES ('a', 'c:20', 'base', 20, true, 'block', 'high', 1, now() - interval '1 hour', \
               '[{\"kind\":\"exception\",\"id\":\"00000000-0000-0000-0000-000000000001\"}]'), \
            ('b', 'c:10', 'base', 10, true, 'block', 'high', 1, now() - interval '1 hour', \
               '[{\"kind\":\"exception\",\"id\":\"00000000-0000-0000-0000-000000000001\"}]')";

const CROSSING_EXCEPTION: &str = "INSERT INTO policy_exception (id, rule, advisory, reason, created_by, expires_at) \
     VALUES ('00000000-0000-0000-0000-000000000001', 'vulnerability', 'CVE-1', 'gone', 'tester', now() - interval '1 minute')";

#[tokio::test]
async fn every_sweeper_page_marks_in_component_order_so_it_cannot_deadlock_with_the_matcher() {
    for (label, prepare) in [
        ("stale", ""),
        ("findings", CROSSING_VERDICTS),
        (
            "expired",
            &format!("{CROSSING_EXCEPTION}; {CROSSING_VERDICTS}"),
        ),
    ] {
        let s = setup().await;
        crossing_world(&s).await;
        if !prepare.is_empty() {
            execute(&s, prepare).await;
        }
        let holder = s.db.conn().begin().await.unwrap();
        holder
            .execute_unprepared(
                "INSERT INTO policy_dirty (repository, subject) VALUES ('b', 'c:10')",
            )
            .await
            .unwrap();
        let sweeping = {
            let db = s.db.conn().clone();
            tokio::spawn(
                async move { verdict::sweep(&db, &config(), &mut SweepState::default()).await },
            )
        };
        tokio::time::timeout(Duration::from_secs(30), async {
            while scalar(&s, WAITING_TRANSACTIONS).await == 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("{label}: the sweeper never waited on the open mark"));

        holder
            .execute_unprepared(
                "INSERT INTO policy_dirty (repository, subject) VALUES ('a', 'c:20')",
            )
            .await
            .unwrap_or_else(|error| panic!("{label}: the second mark failed: {error}"));
        holder.commit().await.unwrap();
        tokio::time::timeout(Duration::from_secs(30), sweeping)
            .await
            .unwrap_or_else(|_| panic!("{label}: the sweep hung"))
            .unwrap()
            .unwrap_or_else(|error| panic!("{label}: the sweep failed: {error}"));
        assert_eq!(
            scalar(&s, "SELECT count(*) AS n FROM policy_dirty").await,
            2,
            "{label}"
        );
    }
}
