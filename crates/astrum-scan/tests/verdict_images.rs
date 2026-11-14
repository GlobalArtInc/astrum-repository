use std::time::Duration;

use astrum_db::Database;
use astrum_policy::cache::Cache;
use astrum_scan::verdict::{self, Config, SweepState};
use astrum_testkit::TestDatabase;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

struct Setup {
    _test: TestDatabase,
    db: Database,
}

async fn setup() -> Setup {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    execute_on(
        &db,
        "INSERT INTO content_policy (name, vulnerabilities) \
           VALUES ('base', '{\"mode\":\"block\",\"severity\":\"high\"}'); \
         INSERT INTO repository (name, format, type, attributes) VALUES \
           ('one', 'docker', 'hosted', '{\"policy\":{\"name\":\"base\"}}'), \
           ('two', 'docker', 'hosted', '{\"policy\":{\"name\":\"base\"}}'); \
         INSERT INTO advisory (id, aliases, severity, modified_at, document) \
           VALUES ('GHSA-1', '{CVE-1}', 'high', now(), '{}')",
    )
    .await;
    Setup { _test: test, db }
}

async fn execute_on(db: &Database, sql: &str) {
    db.conn().execute_unprepared(sql).await.unwrap();
}

async fn execute(s: &Setup, sql: &str) {
    execute_on(&s.db, sql).await;
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

fn config() -> Config {
    Config {
        idle: Duration::from_millis(20),
        ..Config::default()
    }
}

async fn judge(s: &Setup) {
    verdict::judge_batch(s.db.conn(), &Cache::default(), &config())
        .await
        .unwrap();
}

async fn inventory(s: &Setup, repository: &str, digest: &str, complete: bool, found: bool) {
    let matched = match found {
        true => "INSERT INTO sca_match (package_id, advisory_id) SELECT pkg.id, 'GHSA-1' FROM pkg",
        false => "SELECT 1",
    };
    execute(
        s,
        &format!(
            "WITH inv AS (INSERT INTO sca_inventory (key, source, extractor, complete) \
               VALUES ('oci:{repository}:{digest}', 'sbom', 1, {complete}) RETURNING id), \
             pkg AS (INSERT INTO sca_package (ecosystem, name, version, purl, matched_at) \
               VALUES ('npm', '{repository}-{digest}', '1', 'pkg:npm/x@1', now()) RETURNING id), \
             link AS (INSERT INTO sca_inventory_package (inventory_id, package_id) \
               SELECT inv.id, pkg.id FROM inv, pkg) \
             {matched}"
        ),
    )
    .await;
}

async fn tag(s: &Setup, repository: &str, id: i64, name: &str, version: &str, digest: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO component (id, repository, format, name, version) \
               VALUES ({id}, '{repository}', 'docker', '{name}', '{version}'); \
             INSERT INTO component_scan (component_id, repository, inventory_id, digest, state) \
               SELECT {id}, '{repository}', i.id, '{digest}', 'scanned' FROM sca_inventory i \
               WHERE i.key = 'oci:{repository}:{digest}'; \
             INSERT INTO asset (repository, format, path, kind, component_id, blob_store, blob_id, \
               size, content_type, sha1, sha256, sha512, md5, attributes) \
               VALUES ('{repository}', 'docker', '/v2/{name}/manifests/{version}', 'tag', {id}, \
               'default', gen_random_uuid(), 1, 'x', '', '', '', '', \
               '{{\"docker\":{{\"name\":\"{name}\",\"digest\":\"{digest}\"}}}}')"
        ),
    )
    .await;
}

async fn mark(s: &Setup, repository: &str, subject: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO policy_dirty (repository, subject) VALUES ('{repository}', '{subject}') \
             ON CONFLICT DO NOTHING"
        ),
    )
    .await;
}

const VERDICTS: &str = "SELECT COALESCE(string_agg(repository || '/' || subject || '/' || state \
    || '/' || COALESCE(component_id::text, '-'), ',' ORDER BY repository, subject), '') AS n \
    FROM policy_verdict";

#[tokio::test]
async fn an_oci_verdict_reads_the_inventory_of_its_own_repository_by_key_and_needs_no_tag() {
    let s = setup().await;
    inventory(&s, "one", "sha256:aa", true, true).await;
    inventory(&s, "two", "sha256:aa", true, false).await;
    inventory(&s, "one", "sha256:bb", false, false).await;
    for (repository, digest) in [
        ("one", "sha256:aa"),
        ("two", "sha256:aa"),
        ("one", "sha256:bb"),
        ("one", "sha256:cc"),
    ] {
        mark(&s, repository, &format!("oci:{digest}")).await;
    }
    judge(&s).await;
    assert_eq!(
        text(&s, VERDICTS).await,
        "one/oci:sha256:aa/block/-,one/oci:sha256:bb/unscanned/-,one/oci:sha256:cc/unscanned/-,two/oci:sha256:aa/pass/-"
    );
}

async fn manifest(s: &Setup, repository: &str, digest: &str) {
    execute(
        s,
        &format!(
            "INSERT INTO asset (repository, format, path, kind, blob_store, blob_id, size, \
               content_type, sha1, sha256, sha512, md5, attributes) \
             VALUES ('{repository}', 'docker', '/v2/app/manifests/{digest}', 'manifest', \
               'default', gen_random_uuid(), 1, 'x', '', '', '', '', \
               '{{\"docker\":{{\"name\":\"app\",\"digest\":\"{digest}\"}}}}')"
        ),
    )
    .await;
}

async fn sweep(s: &Setup) {
    verdict::sweep(s.db.conn(), &config(), &mut SweepState::default())
        .await
        .unwrap();
}

async fn subjects(s: &Setup, table: &str, filter: &str) -> String {
    text(
        s,
        &format!(
            "SELECT COALESCE(string_agg(subject, ',' ORDER BY subject), '') AS n FROM {table} \
             WHERE subject LIKE '{filter}'"
        ),
    )
    .await
}

async fn exception(s: &Setup, column: &str, value: &str) {
    execute(
        s,
        &format!(
            "DELETE FROM policy_exception; DELETE FROM policy_verdict; \
             INSERT INTO policy_exception (id, rule, advisory, {column}, reason, created_by) \
             VALUES (gen_random_uuid(), 'vulnerability', 'CVE-1', '{value}', 'accepted', 'tester')"
        ),
    )
    .await;
    for subject in ["c:1", "c:2", "oci:sha256:aa"] {
        mark(s, "one", subject).await;
    }
    judge(s).await;
}

#[tokio::test]
async fn an_exception_gives_a_tag_and_a_digest_pull_the_same_verdict_by_name_or_by_digest() {
    let s = setup().await;
    inventory(&s, "one", "sha256:aa", true, true).await;
    tag(&s, "one", 1, "app", "1.0", "sha256:aa").await;
    tag(&s, "one", 2, "tool", "2.0", "sha256:aa").await;
    let states = "SELECT string_agg(subject || '/' || state, ',' ORDER BY subject) AS n \
                  FROM policy_verdict";
    for (column, value, expected) in [
        ("name", "app", "c:1/pass,c:2/block,oci:sha256:aa/pass"),
        ("name", "tool", "c:1/block,c:2/pass,oci:sha256:aa/pass"),
        ("name", "other", "c:1/block,c:2/block,oci:sha256:aa/block"),
        (
            "digest",
            "sha256:aa",
            "c:1/pass,c:2/pass,oci:sha256:aa/pass",
        ),
        (
            "digest",
            "sha256:zz",
            "c:1/block,c:2/block,oci:sha256:aa/block",
        ),
    ] {
        exception(&s, column, value).await;
        assert_eq!(text(&s, states).await, expected, "{column} {value}");
    }
}

#[tokio::test]
async fn an_oci_verdict_outlives_its_tags_and_goes_only_when_no_manifest_or_tag_holds_the_digest() {
    let s = setup().await;
    inventory(&s, "one", "sha256:aa", true, true).await;
    inventory(&s, "one", "sha256:bb", true, true).await;
    manifest(&s, "one", "sha256:aa").await;
    manifest(&s, "one", "sha256:bb").await;
    tag(&s, "one", 1, "app", "1.0", "sha256:aa").await;
    tag(&s, "one", 2, "app", "stable", "sha256:aa").await;
    for subject in ["c:1", "c:2", "oci:sha256:aa", "oci:sha256:bb"] {
        mark(&s, "one", subject).await;
    }
    judge(&s).await;
    let both = "oci:sha256:aa,oci:sha256:bb";

    execute(&s, "DELETE FROM component WHERE id = 1").await;
    sweep(&s).await;
    mark(&s, "one", "c:2").await;
    judge(&s).await;
    assert_eq!(subjects(&s, "policy_verdict", "oci:%").await, both);
    assert_eq!(subjects(&s, "policy_verdict", "c:%").await, "c:2");

    execute(
        &s,
        "UPDATE component_scan SET digest = 'sha256:bb' WHERE component_id = 2; \
         UPDATE asset SET attributes = jsonb_set(attributes, '{docker,digest}', '\"sha256:bb\"') \
           WHERE component_id = 2",
    )
    .await;
    sweep(&s).await;
    assert_eq!(subjects(&s, "policy_verdict", "oci:%").await, both);

    execute(
        &s,
        "DELETE FROM asset WHERE path = '/v2/app/manifests/sha256:aa'",
    )
    .await;
    sweep(&s).await;
    assert_eq!(
        subjects(&s, "policy_verdict", "oci:%").await,
        "oci:sha256:bb"
    );
}

#[tokio::test]
async fn the_sweeper_marks_images_without_a_verdict_or_with_a_stale_one_by_their_digest() {
    let s = setup().await;
    inventory(&s, "one", "sha256:aa", true, true).await;
    tag(&s, "one", 1, "app", "1.0", "sha256:aa").await;
    manifest(&s, "one", "sha256:dd").await;
    execute(
        &s,
        "INSERT INTO policy_verdict (repository, subject, policy, blocked, state, policy_revision) \
           VALUES ('one', 'oci:sha256:dd', 'base', false, 'pass', 0); \
         DELETE FROM policy_dirty",
    )
    .await;
    sweep(&s).await;
    assert_eq!(
        subjects(&s, "policy_dirty", "%").await,
        "c:1,oci:sha256:aa,oci:sha256:dd"
    );
}
