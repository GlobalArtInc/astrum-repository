use astrum_db::Database;
use astrum_scan::matcher::{self, Change, Config};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

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

const EVERYTHING: &str = r#"[{"type":"SEMVER","events":[{"introduced":"0"}]}]"#;
const NOTIFY_HIGH: &str = r#"{"mode":"report","severity":"critical","notify":"high"}"#;

async fn world(s: &Setup, policy: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO content_policy (name, vulnerabilities) VALUES ('watch', '{policy}'); \
             INSERT INTO repository (name, format, type, attributes) VALUES \
               ('one', 'npm', 'hosted', '{{\"policy\":{{\"name\":\"watch\"}}}}'), \
               ('free', 'npm', 'hosted', '{{}}'); \
             INSERT INTO notification_channel (id, name, type, recipients, events) \
               VALUES (gen_random_uuid(), 'ops', 'email', '{{ops@example.com}}', '{{vulnerability.found}}')"
        ),
    )
    .await;
}

async fn advisory(s: &Setup, id: &str, alias: &str, severity: &str, package: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO advisory (id, aliases, severity, modified_at, document) \
               VALUES ('{id}', '{{{alias}}}', '{severity}', now(), '{{}}'); \
             INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges) \
               VALUES ('{id}', 'npm', '{package}', '{EVERYTHING}')"
        ),
    )
    .await;
}

async fn component(s: &Setup, repository: &str, id: i64, package: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO component (id, repository, format, name, version) \
               VALUES ({id}, '{repository}', 'npm', 'c{id}', '1'); \
             INSERT INTO sca_package (id, ecosystem, name, version, purl) \
               VALUES ({id}, 'npm', '{package}', '1.{id}.0', 'pkg:npm/{package}@1.{id}.0'); \
             INSERT INTO sca_inventory (id, key, source, extractor) VALUES ({id}, 'k{id}', 'metadata', 1); \
             INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES ({id}, {id}); \
             INSERT INTO component_scan (component_id, repository, inventory_id, state) \
               VALUES ({id}, '{repository}', {id}, 'scanned')"
        ),
    )
    .await;
}

async fn notices(s: &Setup) -> i64 {
    scalar(
        s,
        "SELECT count(*) AS n FROM notification_change WHERE action = 'vulnerability.found'",
    )
    .await
}

async fn run(s: &Setup) -> matcher::Batch {
    matcher::match_batch(s.db.conn(), &Config::default())
        .await
        .unwrap()
}

#[tokio::test]
async fn a_new_match_at_the_threshold_queues_one_notice_and_a_rematch_or_a_new_fix_queues_none() {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    component(&s, "one", 1, "lib").await;

    let first = run(&s).await;
    assert_eq!((first.added.len(), first.notices), (1, 1));
    assert_eq!(notices(&s).await, 1);
    let row = text(
        &s,
        "SELECT repository || '/' || name || '/' || (detail->>'policy') || '/' || (detail->>'advisory') \
         || '/' || (detail->>'severity') AS n FROM notification_change WHERE action = 'vulnerability.found'",
    )
    .await;
    assert_eq!(row, "one/c1/watch/GHSA-1/high");

    execute(&s, "UPDATE sca_package SET matched_at = NULL").await;
    let again = run(&s).await;
    assert_eq!((again.matched, again.added.len(), again.notices), (1, 0, 0));

    execute(
        &s,
        "UPDATE sca_match SET fixed = '9.9.9'; UPDATE sca_package SET matched_at = NULL",
    )
    .await;
    let fixed = run(&s).await;
    assert_eq!((fixed.updated.len(), fixed.notices), (1, 0));
    assert_eq!(notices(&s).await, 1);
}

#[tokio::test]
async fn nothing_is_queued_below_the_threshold_without_a_policy_or_for_a_channel_that_does_not_listen()
 {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    advisory(&s, "MED-1", "CVE-2", "medium", "weak").await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    component(&s, "one", 1, "weak").await;
    component(&s, "free", 2, "lib").await;
    let batch = run(&s).await;
    assert_eq!((batch.added.len(), batch.notices), (2, 0));

    execute(&s, "UPDATE notification_channel SET events = '{published}'").await;
    component(&s, "one", 3, "lib").await;
    let deaf = run(&s).await;
    assert_eq!((deaf.added.len(), deaf.notices), (1, 0));
    assert_eq!(notices(&s).await, 0);

    execute(
        &s,
        "UPDATE notification_channel SET events = '{vulnerability.found}'; \
         UPDATE content_policy SET vulnerabilities = '{\"mode\":\"off\",\"notify\":\"low\"}'",
    )
    .await;
    component(&s, "one", 4, "lib").await;
    let off = run(&s).await;
    assert_eq!((off.added.len(), off.notices), (1, 0));
}

#[tokio::test]
async fn a_finding_covered_by_an_active_exception_is_not_announced() {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    execute(
        &s,
        "INSERT INTO policy_exception (id, rule, advisory, name, reason, created_by) \
           VALUES (gen_random_uuid(), 'vulnerability', 'CVE-1', 'c1', 'accepted', 'sec'); \
         INSERT INTO policy_exception (id, rule, advisory, name, reason, created_by, expires_at) \
           VALUES (gen_random_uuid(), 'vulnerability', 'GHSA-1', 'c3', 'lapsed', 'sec', now() - interval '1 hour')",
    )
    .await;
    for id in 1..=3 {
        component(&s, "one", id, "lib").await;
    }
    let batch = run(&s).await;
    assert_eq!((batch.added.len(), batch.notices), (3, 2));
    let named = text(
        &s,
        "SELECT string_agg(name, ',' ORDER BY name) AS n FROM notification_change \
         WHERE action = 'vulnerability.found'",
    )
    .await;
    assert_eq!(named, "c2,c3");
}

#[tokio::test]
async fn a_failed_batch_leaves_neither_matches_nor_notices() {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    component(&s, "one", 1, "lib").await;
    execute(
        &s,
        "CREATE FUNCTION refuse_update() RETURNS trigger LANGUAGE plpgsql AS $$ \
           BEGIN RAISE EXCEPTION 'refused'; END $$; \
         CREATE TRIGGER refuse_update BEFORE UPDATE ON sca_package \
           FOR EACH ROW EXECUTE FUNCTION refuse_update()",
    )
    .await;

    assert!(
        matcher::match_batch(s.db.conn(), &Config::default())
            .await
            .is_err()
    );
    assert_eq!(notices(&s).await, 0);
    assert_eq!(scalar(&s, "SELECT count(*) AS n FROM sca_match").await, 0);

    execute(&s, "DROP TRIGGER refuse_update ON sca_package").await;
    assert_eq!(run(&s).await.notices, 1);
    assert_eq!(notices(&s).await, 1);
}

#[tokio::test]
async fn a_burst_is_cut_at_the_limit_and_says_so() {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    execute(
        &s,
        "INSERT INTO sca_package (id, ecosystem, name, version, purl) \
           VALUES (1, 'npm', 'lib', '1.0.0', 'pkg:npm/lib@1.0.0'); \
         INSERT INTO sca_inventory (id, key, source, extractor) VALUES (1, 'shared', 'metadata', 1); \
         INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES (1, 1); \
         INSERT INTO sca_match (package_id, advisory_id) VALUES (1, 'GHSA-1'); \
         INSERT INTO component (id, repository, format, name, version) \
           SELECT g, 'one', 'npm', 'c' || g, '1' FROM generate_series(1, 5) g; \
         INSERT INTO component_scan (component_id, repository, inventory_id, state) \
           SELECT g, 'one', 1, 'scanned' FROM generate_series(1, 5) g",
    )
    .await;
    let added = [Change {
        package_id: 1,
        advisory_id: "GHSA-1".to_owned(),
    }];
    let txn = s.db.conn().begin().await.unwrap();
    let queued = matcher::queue_notices(&txn, &added, 3).await.unwrap();
    txn.commit().await.unwrap();
    assert_eq!((queued.queued, queued.cut), (3, true));
    assert_eq!(notices(&s).await, 3);

    let txn = s.db.conn().begin().await.unwrap();
    let rest = matcher::queue_notices(&txn, &added, 10).await.unwrap();
    txn.rollback().await.unwrap();
    assert_eq!((rest.queued, rest.cut), (5, false));
    assert_eq!(notices(&s).await, 3);
}

#[tokio::test]
async fn two_packages_of_one_component_hit_by_one_advisory_give_one_notice() {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    execute(
        &s,
        &format!(
            "INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges) \
               VALUES ('GHSA-1', 'npm', 'twin', '{EVERYTHING}')"
        ),
    )
    .await;
    component(&s, "one", 1, "lib").await;
    execute(
        &s,
        "INSERT INTO sca_package (id, ecosystem, name, version, purl) \
           VALUES (2, 'npm', 'twin', '1.0.0', 'pkg:npm/twin@1.0.0'); \
         INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES (1, 2)",
    )
    .await;
    let batch = run(&s).await;
    assert_eq!((batch.added.len(), batch.notices), (2, 1));
    assert_eq!(notices(&s).await, 1);
}

#[tokio::test]
async fn channels_without_a_repository_list_hear_security_events_of_every_repository_type() {
    let s = setup().await;
    world(&s, NOTIFY_HIGH).await;
    execute(
        &s,
        "UPDATE notification_channel SET name = 'open', events = '{vulnerability.found,policy.blocked,published}'; \
         INSERT INTO notification_channel (id, name, type, recipients, repositories, events) \
           VALUES (gen_random_uuid(), 'pinned', 'email', '{ops@example.com}', '{one}', \
                   '{vulnerability.found,policy.blocked,published}'); \
         INSERT INTO repository (name, format, type, attributes) \
           VALUES ('mirror', 'npm', 'proxy', '{\"policy\":{\"name\":\"watch\"}}')",
    )
    .await;
    advisory(&s, "GHSA-1", "CVE-1", "high", "lib").await;
    component(&s, "mirror", 1, "lib").await;

    let batch = run(&s).await;
    assert_eq!(batch.notices, 1);
    execute(
        &s,
        "SELECT notification_capture_events('policy.blocked', ARRAY[1::bigint], ARRAY['{\"policy\":\"watch\"}'])",
    )
    .await;

    let heard = text(
        &s,
        "SELECT COALESCE(string_agg(ch.name || ':' || n.action, ',' ORDER BY ch.name, n.action), '') AS n \
         FROM notification_change n JOIN notification_channel ch ON ch.id = n.channel_id \
         WHERE n.repository = 'mirror'",
    )
    .await;
    assert_eq!(heard, "open:policy.blocked,open:vulnerability.found");
}
