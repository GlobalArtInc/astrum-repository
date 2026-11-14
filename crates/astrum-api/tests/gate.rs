mod support;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_api::gate::{Outcome, PolicyGate};
use astrum_db::entity::audit_event;
use astrum_scan::PACKAGE_LOCK;
use astrum_scan::inline::{self, InlineError};
use astrum_scan::verdict::{self, SweepState};
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Statement, TransactionTrait,
};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const POLICIES: &str = "/service/rest/v1/security/policies";
const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";
const REPOSITORIES: &str = "/service/rest/v1/repositories";
const BLOCKED_BY: &str = "x-astrum-blocked-by";
const JAR: &[u8] = b"PK\x03\x04\x14\x00\x08\x00\x08\x00jar-content";

const COUNT_SCANS: &str = "CREATE TABLE scan_writes (id serial PRIMARY KEY); \
     CREATE FUNCTION count_scan_write() RETURNS trigger LANGUAGE plpgsql AS $$ \
       BEGIN INSERT INTO scan_writes DEFAULT VALUES; RETURN NULL; END $$; \
     CREATE TRIGGER count_scan_write AFTER INSERT OR UPDATE ON component_scan \
       FOR EACH ROW EXECUTE FUNCTION count_scan_write()";
const COUNT_MARKS: &str = "CREATE TABLE mark_writes (id serial PRIMARY KEY); \
     CREATE FUNCTION count_mark_write() RETURNS trigger LANGUAGE plpgsql AS $$ \
       BEGIN INSERT INTO mark_writes DEFAULT VALUES; RETURN NULL; END $$; \
     CREATE TRIGGER count_mark_write BEFORE INSERT ON policy_dirty \
       FOR EACH STATEMENT EXECUTE FUNCTION count_mark_write()";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_go::Go));
    fixture
}

async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(sql)
        .await
        .unwrap();
}

async fn number(fixture: &Fixture, sql: &str) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    fixture.call(method, path, Auth::Admin, body).await
}

async fn policy(fixture: &Fixture, name: &str, rule: Value) {
    let reply = admin(
        fixture,
        Method::POST,
        POLICIES,
        Some(json!({"name": name, "vulnerabilities": rule})),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn strict(fixture: &Fixture) {
    policy(
        fixture,
        "strict",
        json!({"mode": "block", "severity": "high"}),
    )
    .await;
}

async fn repository(fixture: &Fixture, format: &str, kind: &str, name: &str, policy: Option<&str>) {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
    });
    match format {
        "maven" => {
            body["maven"] = json!({"versionPolicy": "RELEASE", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"});
        }
        "go" => body["storage"]["writePolicy"] = json!("ALLOW_ONCE"),
        _ => {}
    }
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
    }
    let reply = admin(
        fixture,
        Method::POST,
        &format!("{REPOSITORIES}/{format}/{kind}"),
        Some(body),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn group(fixture: &Fixture, format: &str, name: &str, members: &[&str]) {
    let reply = admin(
        fixture,
        Method::POST,
        &format!("{REPOSITORIES}/{format}/group"),
        Some(json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": members},
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

struct Package {
    path: String,
    name: &'static str,
}

fn tarball(name: &str, version: &str) -> Vec<u8> {
    let mut data = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x03".to_vec();
    data.extend_from_slice(format!("package {name} {version}").as_bytes());
    data
}

async fn npm_path(fixture: &Fixture, repository: &str, name: &str) -> String {
    let data = tarball(name, "1.0.0");
    let reply = admin(
        fixture,
        Method::PUT,
        &format!("/repository/{repository}/{name}"),
        Some(json!({
            "_id": name, "name": name, "dist-tags": {"latest": "1.0.0"},
            "versions": {"1.0.0": {"name": name, "version": "1.0.0",
                "dist": {"tarball": format!("http://client.example/{name}/-/{name}-1.0.0.tgz")}}},
            "_attachments": {format!("{name}-1.0.0.tgz"): {"content_type": "application/octet-stream",
                "data": STANDARD.encode(&data), "length": data.len()}}
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    format!("/repository/{repository}/{name}/-/{name}-1.0.0.tgz")
}

async fn npm(fixture: &Fixture, repository: &str) -> Package {
    Package {
        path: npm_path(fixture, repository, "lib").await,
        name: "lib",
    }
}

async fn put(fixture: &Fixture, path: &str, body: Vec<u8>) {
    let reply = fixture
        .send_body(
            Method::PUT,
            path,
            Auth::Admin,
            Body::from(body),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn maven(fixture: &Fixture, repository: &str) -> Package {
    let directory = format!("/repository/{repository}/org/example/lib/1.0");
    put(fixture, &format!("{directory}/lib-1.0.jar"), JAR.to_vec()).await;
    let pom = "<?xml version=\"1.0\"?><project xmlns=\"http://maven.apache.org/POM/4.0.0\"><modelVersion>4.0.0</modelVersion><groupId>org.example</groupId><artifactId>lib</artifactId><version>1.0</version><packaging>jar</packaging></project>";
    put(
        fixture,
        &format!("{directory}/lib-1.0.pom"),
        pom.as_bytes().to_vec(),
    )
    .await;
    Package {
        path: format!("{directory}/lib-1.0.jar"),
        name: "lib",
    }
}

async fn golang(fixture: &Fixture, repository: &str) -> Package {
    let go_mod = "module example.com/lib\n\ngo 1.22\n";
    let files: Vec<(&str, &[u8])> = vec![
        ("example.com/lib@v1.0.0/go.mod", go_mod.as_bytes()),
        ("example.com/lib@v1.0.0/lib.go", b"package lib\n"),
    ];
    put(
        fixture,
        &format!("/repository/{repository}/example.com/lib/@v/v1.0.0.zip"),
        astrum_content::zip::write(&files),
    )
    .await;
    Package {
        path: format!("/repository/{repository}/example.com/lib/@v/v1.0.0.zip"),
        name: "example.com/lib",
    }
}

async fn component(fixture: &Fixture, repository: &str, name: &str) -> i64 {
    number(
        fixture,
        &format!(
            "SELECT id AS n FROM component WHERE repository = '{repository}' AND name = '{name}'"
        ),
    )
    .await
}

async fn revision_of(fixture: &Fixture, policy: &str) -> i64 {
    number(
        fixture,
        &format!("SELECT revision AS n FROM content_policy WHERE name = '{policy}'"),
    )
    .await
}

async fn seed(
    fixture: &Fixture,
    repository: &str,
    component: i64,
    state: &str,
    blocked: bool,
    reasons: Value,
) {
    let revision = revision_of(fixture, "strict").await;
    seed_as(
        fixture, repository, component, "strict", revision, state, blocked, reasons,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn seed_as(
    fixture: &Fixture,
    repository: &str,
    component: i64,
    policy: &str,
    revision: i64,
    state: &str,
    blocked: bool,
    reasons: Value,
) {
    execute(
        fixture,
        &format!(
            "INSERT INTO policy_verdict (repository, subject, policy, component_id, blocked, state, \
               reasons, policy_revision, max_severity, high) \
             VALUES ('{repository}', 'c:{component}', '{policy}', {component}, {blocked}, '{state}', \
               '{reasons}'::jsonb, {revision}, 'high', 1) \
             ON CONFLICT (repository, subject) DO UPDATE SET policy = EXCLUDED.policy, \
               blocked = EXCLUDED.blocked, state = EXCLUDED.state, reasons = EXCLUDED.reasons, \
               policy_revision = EXCLUDED.policy_revision, computed_at = now()"
        ),
    )
    .await;
}

fn finding(advisory: &str) -> Value {
    json!([{"kind": "vulnerability", "advisory": advisory, "aliases": [], "severity": "high",
        "malicious": false, "package": "npm/lib@1.0.0", "fixed": "1.0.1", "blocking": true}])
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    admin(fixture, Method::GET, path, None).await
}

fn refused(reply: &Reply) {
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    assert_eq!(reply.headers[BLOCKED_BY], "strict");
}

fn install_gate(fixture: &Fixture, budget_ms: u64) -> Arc<PolicyGate> {
    let gate = Arc::new(
        PolicyGate::new(
            fixture.state.policy_cache().clone(),
            fixture.state.gate_metrics().clone(),
        )
        .with_budget(Duration::from_millis(budget_ms)),
    );
    fixture.state.content().register_gate(gate.clone());
    gate
}

fn install_small_gate(fixture: &Fixture, budget_ms: u64, permits: usize) -> Arc<PolicyGate> {
    let gate = Arc::new(
        PolicyGate::new(
            fixture.state.policy_cache().clone(),
            fixture.state.gate_metrics().clone(),
        )
        .with_budget(Duration::from_millis(budget_ms))
        .with_permits(permits),
    );
    fixture.state.content().register_gate(gate.clone());
    gate
}

async fn settled(gate: &PolicyGate, permits: usize) {
    for _ in 0..200 {
        if gate.available_permits() == permits {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!(
        "the scans did not end: {} of {permits} slots are free",
        gate.available_permits()
    );
}

struct Held {
    transaction: sea_orm::DatabaseTransaction,
    _database: astrum_db::Database,
}

impl Held {
    async fn release(self) {
        self.transaction.rollback().await.unwrap();
    }
}

async fn hold(fixture: &Fixture, sql: &str) -> Held {
    let database = fixture.test.connect().await;
    let transaction = database.conn().begin().await.unwrap();
    transaction.execute_unprepared(sql).await.unwrap();
    Held {
        transaction,
        _database: database,
    }
}

fn package_lock() -> String {
    format!("SELECT pg_advisory_xact_lock({PACKAGE_LOCK})")
}

async fn waiting_for_advisory_locks(fixture: &Fixture) -> i64 {
    number(
        fixture,
        "SELECT count(*) AS n FROM pg_stat_activity \
         WHERE datname = current_database() AND wait_event_type = 'Lock' AND wait_event = 'advisory'",
    )
    .await
}

async fn waiting_settles_at(fixture: &Fixture, wanted: i64) {
    for _ in 0..100 {
        if waiting_for_advisory_locks(fixture).await == wanted {
            tokio::time::sleep(Duration::from_millis(300)).await;
            assert_eq!(waiting_for_advisory_locks(fixture).await, wanted);
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!(
        "{} scans wait for the lock, not {wanted}",
        waiting_for_advisory_locks(fixture).await
    );
}

async fn marks(fixture: &Fixture) -> i64 {
    number(fixture, "SELECT count(*) AS n FROM policy_dirty").await
}

#[tokio::test]
async fn a_blocked_package_is_refused_with_its_reason_in_npm_maven_and_go() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    repository(&fixture, "maven", "hosted", "maven-hosted", Some("strict")).await;
    repository(&fixture, "go", "hosted", "go-hosted", Some("strict")).await;
    let packages = [
        ("npm-hosted", npm(&fixture, "npm-hosted").await),
        ("maven-hosted", maven(&fixture, "maven-hosted").await),
        ("go-hosted", golang(&fixture, "go-hosted").await),
    ];
    for (repository, package) in packages {
        let id = component(&fixture, repository, package.name).await;
        let open = get(&fixture, &package.path).await;
        assert_eq!(open.status, StatusCode::OK, "{repository}: {}", open.text());
        let etag = open.headers[header::ETAG].to_str().unwrap().to_owned();

        seed(&fixture, repository, id, "block", true, finding("GHSA-1")).await;
        let reply = get(&fixture, &package.path).await;
        refused(&reply);
        assert!(
            reply.text().contains(
                "blocked by policy strict: GHSA-1: high in npm/lib@1.0.0, fixed in 1.0.1"
            ),
            "{repository}: {}",
            reply.text()
        );
        let head = admin(&fixture, Method::HEAD, &package.path, None).await;
        refused(&head);
        let conditional = fixture
            .send(
                Method::GET,
                &package.path,
                Auth::Admin,
                None,
                &[("if-none-match", &etag)],
            )
            .await;
        refused(&conditional);
        let ranged = fixture
            .send(
                Method::GET,
                &package.path,
                Auth::Admin,
                None,
                &[("range", "bytes=0-3")],
            )
            .await;
        refused(&ranged);
    }
    let pom = get(
        &fixture,
        "/repository/maven-hosted/org/example/lib/1.0/lib-1.0.pom",
    )
    .await;
    refused(&pom);
}

fn results(fixture: &Fixture) -> HashMap<&'static str, u64> {
    let snapshot = fixture.state.gate_metrics().snapshot();
    Outcome::ALL
        .iter()
        .zip(snapshot.results)
        .filter(|(_, count)| *count > 0)
        .map(|(outcome, count)| (outcome.label(), count))
        .collect()
}

async fn verdict_state(fixture: &Fixture, component: i64) -> Option<String> {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT state FROM policy_verdict WHERE subject = 'c:{component}'"),
        ))
        .await
        .unwrap()
        .map(|row| row.try_get("", "state").unwrap())
}

async fn stored_verdict(fixture: &Fixture, component: i64) -> (String, i64, String, bool) {
    let row = fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!(
                "SELECT policy, policy_revision, state, blocked FROM policy_verdict WHERE subject = 'c:{component}'"
            ),
        ))
        .await
        .unwrap()
        .unwrap();
    (
        row.try_get("", "policy").unwrap(),
        row.try_get("", "policy_revision").unwrap(),
        row.try_get("", "state").unwrap(),
        row.try_get("", "blocked").unwrap(),
    )
}

#[tokio::test]
async fn a_verdict_of_another_policy_or_revision_is_ignored_and_no_policy_means_no_gate() {
    let fixture = ready().await;
    strict(&fixture).await;
    policy(
        &fixture,
        "other",
        json!({"mode": "block", "severity": "high"}),
    )
    .await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    repository(&fixture, "npm", "hosted", "npm-free", None).await;
    let package = npm(&fixture, "npm-hosted").await;
    let id = component(&fixture, "npm-hosted", package.name).await;
    let revision = revision_of(&fixture, "strict").await;

    for (name, wrong_revision) in [("strict", 99), ("other", revision)] {
        seed_as(
            &fixture,
            "npm-hosted",
            id,
            name,
            wrong_revision,
            "block",
            true,
            finding("GHSA-1"),
        )
        .await;
        let reply = get(&fixture, &package.path).await;
        assert_eq!(reply.status, StatusCode::OK, "{name}: {}", reply.text());
        assert_eq!(
            stored_verdict(&fixture, id).await,
            ("strict".to_owned(), revision, "pass".to_owned(), false),
            "the scan at download replaced the verdict of {name}"
        );
    }
    assert_eq!(results(&fixture), HashMap::from([("inline_scan_pass", 2)]));

    let free = npm(&fixture, "npm-free").await;
    let free_id = component(&fixture, "npm-free", free.name).await;
    seed(
        &fixture,
        "npm-free",
        free_id,
        "block",
        true,
        finding("GHSA-1"),
    )
    .await;
    let reply = get(&fixture, &free.path).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(results(&fixture), HashMap::from([("inline_scan_pass", 2)]));
}

#[tokio::test]
async fn the_state_of_a_matching_verdict_decides_and_unknown_states_block() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    let id = component(&fixture, "npm-hosted", package.name).await;
    let broken = json!([{"kind": "policy-error", "message": "the content policy \"strict\" cannot be read"}]);
    let waiting = json!([{"kind": "unscanned", "scan": "failed"}]);
    let table = [
        ("error", true, true, broken.clone()),
        ("error", false, true, broken),
        ("from-the-future", false, true, json!([])),
        ("unscanned", true, true, waiting.clone()),
        ("unscanned", false, false, waiting),
        ("block", true, true, finding("GHSA-1")),
        ("pass", false, false, json!([])),
        ("report", false, false, json!([])),
    ];
    for (state, blocked, refuses, reasons) in table {
        seed(&fixture, "npm-hosted", id, state, blocked, reasons).await;
        let reply = get(&fixture, &package.path).await;
        match refuses {
            true => {
                refused(&reply);
                assert!(
                    reply.text().contains("blocked by policy strict: "),
                    "{state}/{blocked}: {}",
                    reply.text()
                );
            }
            false => assert_eq!(
                reply.status,
                StatusCode::OK,
                "{state}/{blocked}: {}",
                reply.text()
            ),
        }
    }
    let results = results(&fixture);
    assert_eq!((results["blocked"], results["pass"]), (5, 3));
}

#[tokio::test]
async fn report_and_off_modes_serve_and_report_counts_what_it_would_block() {
    let fixture = ready().await;
    policy(
        &fixture,
        "watch",
        json!({"mode": "report", "severity": "high"}),
    )
    .await;
    policy(&fixture, "quiet", json!({"mode": "off"})).await;
    repository(&fixture, "npm", "hosted", "npm-watch", Some("watch")).await;
    repository(&fixture, "npm", "hosted", "npm-quiet", Some("quiet")).await;
    let watched = npm(&fixture, "npm-watch").await;
    let quiet = npm(&fixture, "npm-quiet").await;
    let id = component(&fixture, "npm-watch", watched.name).await;
    let revision = revision_of(&fixture, "watch").await;
    seed_as(
        &fixture,
        "npm-watch",
        id,
        "watch",
        revision,
        "report",
        false,
        finding("GHSA-1"),
    )
    .await;
    let quiet_id = component(&fixture, "npm-quiet", quiet.name).await;
    let quiet_revision = revision_of(&fixture, "quiet").await;
    seed_as(
        &fixture,
        "npm-quiet",
        quiet_id,
        "quiet",
        quiet_revision,
        "block",
        true,
        finding("GHSA-1"),
    )
    .await;

    for path in [&watched.path, &quiet.path] {
        let reply = get(&fixture, path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    }
    assert_eq!(
        results(&fixture),
        HashMap::from([("would_block", 1), ("pass", 1)])
    );
}

#[tokio::test]
async fn a_scan_that_does_not_finish_in_time_follows_the_unscanned_setting_and_queues_a_mark() {
    let fixture = ready().await;
    policy(
        &fixture,
        "open",
        json!({"mode": "block", "severity": "high"}),
    )
    .await;
    policy(
        &fixture,
        "closed",
        json!({"mode": "block", "severity": "high", "unscanned": "block"}),
    )
    .await;
    repository(&fixture, "npm", "hosted", "npm-open", Some("open")).await;
    repository(&fixture, "npm", "hosted", "npm-closed", Some("closed")).await;
    let open = npm(&fixture, "npm-open").await;
    let closed = npm(&fixture, "npm-closed").await;
    let gate = install_gate(&fixture, 300);
    let holder = hold(&fixture, "LOCK TABLE sca_package IN ACCESS EXCLUSIVE MODE").await;

    let served = get(&fixture, &open.path).await;
    assert_eq!(served.status, StatusCode::OK, "{}", served.text());
    let refused_reply = get(&fixture, &closed.path).await;
    assert_eq!(refused_reply.status, StatusCode::FORBIDDEN);
    assert_eq!(refused_reply.headers[BLOCKED_BY], "closed");
    assert!(
        refused_reply
            .text()
            .contains("blocked by policy closed: the component has not been scanned yet"),
        "{}",
        refused_reply.text()
    );

    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_timeout", 2)])
    );
    for (repository, package) in [("npm-open", &open), ("npm-closed", &closed)] {
        let id = component(&fixture, repository, package.name).await;
        assert_eq!(
            number(
                &fixture,
                &format!("SELECT count(*) AS n FROM policy_dirty WHERE repository = '{repository}' AND subject = 'c:{id}'")
            )
            .await,
            1,
            "{repository}"
        );
    }
    holder.release().await;
    settled(&gate, 8).await;
}

fn advisory_for(package: &str, version: &str) -> String {
    format!(
        "INSERT INTO advisory (id, aliases, severity, modified_at, document) \
           VALUES ('GHSA-T1', '{{CVE-T1}}', 'high', now(), '{{}}'); \
         INSERT INTO advisory_affected (advisory_id, ecosystem, name, versions) \
           VALUES ('GHSA-T1', 'npm', '{package}', '{{{version}}}')"
    )
}

#[tokio::test]
async fn a_download_without_a_verdict_is_judged_at_once_and_an_exception_lifts_and_returns_it() {
    let fixture = ready().await;
    strict(&fixture).await;
    execute(&fixture, &advisory_for("lib", "1.0.0")).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;

    let first = get(&fixture, &package.path).await;
    refused(&first);
    assert!(first.text().contains("GHSA-T1"), "{}", first.text());
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_blocked", 1)])
    );
    assert_eq!(
        get(&fixture, &package.path).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(results(&fixture)["blocked"], 1);

    let expires = (chrono::Utc::now() + chrono::Duration::seconds(3)).to_rfc3339();
    let made = admin(
        &fixture,
        Method::POST,
        EXCEPTIONS,
        Some(json!({"rule": "vulnerability", "advisory": "CVE-T1", "reason": "accepted", "expiresAt": expires})),
    )
    .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    let db = fixture.state.db().conn();
    let cache = fixture.state.policy_cache();
    let config = verdict::Config::default();
    let mut sweeper = SweepState::default();

    verdict::sweep(db, &config, &mut sweeper).await.unwrap();
    verdict::judge_batch(db, cache, &config).await.unwrap();
    let lifted = get(&fixture, &package.path).await;
    assert_eq!(lifted.status, StatusCode::OK, "{}", lifted.text());

    tokio::time::sleep(Duration::from_millis(3300)).await;
    verdict::sweep(db, &config, &mut sweeper).await.unwrap();
    verdict::judge_batch(db, cache, &config).await.unwrap();
    refused(&get(&fixture, &package.path).await);
}

#[tokio::test]
async fn a_group_refuses_a_blocked_member_asset_and_does_not_fall_through_to_another() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-a", Some("strict")).await;
    repository(&fixture, "npm", "hosted", "npm-b", None).await;
    group(&fixture, "npm", "npm-group", &["npm-a", "npm-b"]).await;
    let blocked = npm(&fixture, "npm-a").await;
    npm(&fixture, "npm-b").await;
    let id = component(&fixture, "npm-a", blocked.name).await;
    let path = "/repository/npm-group/lib/-/lib-1.0.0.tgz";
    assert_eq!(get(&fixture, path).await.status, StatusCode::OK);

    seed(&fixture, "npm-a", id, "block", true, finding("GHSA-1")).await;
    refused(&get(&fixture, path).await);
    refused(&admin(&fixture, Method::HEAD, path, None).await);

    let content = fixture.state.content();
    let group_repository = content.repository("npm-group").await.unwrap().unwrap();
    let ctx = content.context(
        group_repository,
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    for (error, falls_through) in [
        (
            astrum_content::ContentError::blocked_by("strict", "x"),
            false,
        ),
        (astrum_content::ContentError::Forbidden, true),
        (astrum_content::ContentError::not_found("x"), true),
    ] {
        let error = Arc::new(Mutex::new(Some(error)));
        let found = astrum_content::group::first(&ctx, |member| {
            let error = error.clone();
            async move {
                match member.name() {
                    "npm-a" => Err(error.lock().unwrap().take().unwrap()),
                    _ => Ok(Some("from b")),
                }
            }
        })
        .await;
        match (falls_through, found) {
            (true, Ok(Some((member, "from b")))) => assert_eq!(member.name(), "npm-b"),
            (false, Err(astrum_content::ContentError::Blocked(_))) => {}
            (falls_through, other) => panic!(
                "falls through {falls_through}: unexpected outcome, error {}",
                other
                    .err()
                    .map_or("none".to_owned(), |error| error.to_string())
            ),
        }
    }
}

async fn start_upstream() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new().fallback(|request: Request| async move {
        match request.uri().path() {
            "/lodash/-/lodash-4.17.20.tgz" | "/tiny/-/tiny-1.0.0.tgz" => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/octet-stream")],
                b"upstream tarball".to_vec(),
            )
                .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    });
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

#[tokio::test]
async fn the_first_fetch_of_a_proxy_is_judged_before_it_is_served() {
    let fixture = ready().await;
    strict(&fixture).await;
    execute(&fixture, &advisory_for("lodash", "4.17.20")).await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::ssrf::SSRF_KEY,
        &astrum_security::ssrf::SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    let address = start_upstream().await;
    let created = admin(
        &fixture,
        Method::POST,
        &format!("{REPOSITORIES}/npm/proxy"),
        Some(json!({
            "name": "npm-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 60},
            "httpClient": {"blocked": false, "autoBlock": false},
            "npm": {"removeQuarantined": false, "removeNonCataloged": false},
            "policy": {"name": "strict"},
        })),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    let bad = get(
        &fixture,
        "/repository/npm-proxy/lodash/-/lodash-4.17.20.tgz",
    )
    .await;
    refused(&bad);
    assert!(bad.text().contains("GHSA-T1"), "{}", bad.text());
    let good = get(&fixture, "/repository/npm-proxy/tiny/-/tiny-1.0.0.tgz").await;
    assert_eq!(good.status, StatusCode::OK, "{}", good.text());
    assert_eq!(good.text(), "upstream tarball");
    refused(
        &get(
            &fixture,
            "/repository/npm-proxy/lodash/-/lodash-4.17.20.tgz",
        )
        .await,
    );
    assert_eq!(
        results(&fixture),
        HashMap::from([
            ("inline_scan_blocked", 1),
            ("inline_scan_pass", 1),
            ("blocked", 1)
        ])
    );
}

#[tokio::test]
async fn a_refusal_is_audited_once_per_subject_without_its_reasons() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    repository(&fixture, "maven", "hosted", "maven-hosted", Some("strict")).await;
    let npm_package = npm(&fixture, "npm-hosted").await;
    let maven_package = maven(&fixture, "maven-hosted").await;
    let npm_id = component(&fixture, "npm-hosted", npm_package.name).await;
    let maven_id = component(&fixture, "maven-hosted", maven_package.name).await;
    seed(
        &fixture,
        "npm-hosted",
        npm_id,
        "block",
        true,
        finding("GHSA-1"),
    )
    .await;
    seed(
        &fixture,
        "maven-hosted",
        maven_id,
        "block",
        true,
        finding("GHSA-1"),
    )
    .await;

    for path in [
        &npm_package.path,
        &npm_package.path,
        &maven_package.path,
        &npm_package.path,
    ] {
        refused(&get(&fixture, path).await);
    }

    let rows = audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("contentPolicy"))
        .filter(audit_event::Column::Kind.eq("blocked"))
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    let row = rows
        .iter()
        .find(|row| row.attributes["repository"] == "npm-hosted")
        .unwrap();
    assert_eq!(
        row.attributes,
        json!({"repository": "npm-hosted", "componentId": npm_id, "policy": "strict", "state": "block"})
    );
    assert_eq!(row.context, "/lib/-/lib-1.0.0.tgz");
    assert!(row.initiator.starts_with("admin"), "{}", row.initiator);
    assert!(!row.attributes.to_string().contains("GHSA-1"));
}

fn counted(fixture: &Fixture) -> (astrum_content::Content, Arc<std::sync::atomic::AtomicUsize>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let mut conn = fixture.state.db().conn().clone();
    let counter = Arc::new(AtomicUsize::new(0));
    let seen = counter.clone();
    conn.set_metric_callback(move |_| {
        seen.fetch_add(1, Ordering::Relaxed);
    });
    let secrets = astrum_security::SecretBox::from_key([7; 32]);
    let blobs = Arc::new(astrum_blob::BlobStores::new(
        conn.clone(),
        fixture.data.path(),
        secrets.clone(),
    ));
    (astrum_content::Content::new(conn, blobs, secrets), counter)
}

async fn checked(
    content: &astrum_content::Content,
    gate: &PolicyGate,
    repository: &str,
    path: &str,
) -> Result<(), astrum_content::ContentError> {
    use astrum_content::Gate;
    let repository = content.repository(repository).await.unwrap().unwrap();
    let ctx = content.context(
        repository,
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    let asset = astrum_content::store::find(ctx.db(), ctx.name(), path)
        .await
        .unwrap()
        .unwrap();
    gate.check(&ctx, &asset).await
}

#[tokio::test]
async fn the_hot_path_is_one_statement_and_free_repositories_cost_none() {
    use std::sync::atomic::Ordering;
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    repository(&fixture, "npm", "hosted", "npm-free", None).await;
    let package = npm(&fixture, "npm-hosted").await;
    npm(&fixture, "npm-free").await;
    let id = component(&fixture, "npm-hosted", package.name).await;
    seed(&fixture, "npm-hosted", id, "block", true, finding("GHSA-1")).await;
    let (content, counter) = counted(&fixture);
    let gate = PolicyGate::new(
        Arc::new(astrum_policy::cache::Cache::new(Duration::from_secs(3600))),
        Arc::default(),
    );
    let path = "/lib/-/lib-1.0.0.tgz";

    let warm = checked(&content, &gate, "npm-hosted", path).await;
    assert!(matches!(
        warm,
        Err(astrum_content::ContentError::Blocked(_))
    ));
    notices_written(&gate).await;
    let repository = content.repository("npm-hosted").await.unwrap().unwrap();
    let ctx = content.context(
        repository,
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    let asset = astrum_content::store::find(ctx.db(), ctx.name(), path)
        .await
        .unwrap()
        .unwrap();
    use astrum_content::Gate;
    counter.store(0, Ordering::Relaxed);
    assert!(gate.check(&ctx, &asset).await.is_err());
    assert_eq!(counter.load(Ordering::Relaxed), 1, "a refusal");
    notices_written(&gate).await;
    assert_eq!(
        counter.load(Ordering::Relaxed),
        1,
        "a refusal of a subject announced today costs no more"
    );

    seed(&fixture, "npm-hosted", id, "pass", false, json!([])).await;
    counter.store(0, Ordering::Relaxed);
    gate.check(&ctx, &asset).await.unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 1, "a pass");

    let free = content.repository("npm-free").await.unwrap().unwrap();
    let free_ctx = content.context(
        free,
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    let free_asset = astrum_content::store::find(free_ctx.db(), free_ctx.name(), path)
        .await
        .unwrap()
        .unwrap();
    counter.store(0, Ordering::Relaxed);
    gate.check(&free_ctx, &free_asset).await.unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 0, "no policy");
}

#[tokio::test]
async fn a_database_error_while_reading_the_verdict_refuses_with_503_and_counts() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    let id = component(&fixture, "npm-hosted", package.name).await;
    seed(&fixture, "npm-hosted", id, "pass", false, json!([])).await;
    assert_eq!(get(&fixture, &package.path).await.status, StatusCode::OK);

    execute(
        &fixture,
        "ALTER TABLE policy_verdict RENAME TO policy_verdict_away",
    )
    .await;
    let reply = get(&fixture, &package.path).await;
    execute(
        &fixture,
        "ALTER TABLE policy_verdict_away RENAME TO policy_verdict",
    )
    .await;

    assert_eq!(
        reply.status,
        StatusCode::SERVICE_UNAVAILABLE,
        "{}",
        reply.text()
    );
    assert!(!reply.text().contains("policy_verdict"), "{}", reply.text());
    assert_eq!(get(&fixture, &package.path).await.status, StatusCode::OK);
    assert_eq!(
        results(&fixture),
        HashMap::from([("pass", 2), ("error", 1)])
    );
}

#[tokio::test]
async fn metadata_assets_without_a_component_and_docker_are_not_gated() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    repository(&fixture, "maven", "hosted", "maven-hosted", Some("strict")).await;
    repository(&fixture, "go", "hosted", "go-hosted", Some("strict")).await;
    let created = admin(
        &fixture,
        Method::POST,
        &format!("{REPOSITORIES}/docker/hosted"),
        Some(json!({
            "name": "docker-hosted", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW", "latestPolicy": true},
            "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
            "policy": {"name": "strict"},
        })),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let packages = [
        ("npm-hosted", npm(&fixture, "npm-hosted").await),
        ("maven-hosted", maven(&fixture, "maven-hosted").await),
        ("go-hosted", golang(&fixture, "go-hosted").await),
    ];
    for (repository, package) in &packages {
        let id = component(&fixture, repository, package.name).await;
        seed(&fixture, repository, id, "block", true, finding("GHSA-1")).await;
        refused(&get(&fixture, &package.path).await);
    }

    for path in [
        "/repository/npm-hosted/lib",
        "/repository/maven-hosted/org/example/lib/maven-metadata.xml",
        "/repository/go-hosted/example.com/lib/@v/list",
    ] {
        let reply = get(&fixture, path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    }

    let (content, counter) = counted(&fixture);
    let gate = PolicyGate::new(Arc::default(), Arc::default());
    let npm_id = component(&fixture, "npm-hosted", "lib").await;
    let asset = astrum_content::store::find(
        fixture.state.db().conn(),
        "npm-hosted",
        "/lib/-/lib-1.0.0.tgz",
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(asset.component_id, Some(npm_id));
    let ctx = content.context(
        content.repository("docker-hosted").await.unwrap().unwrap(),
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    let npm_ctx = content.context(
        content.repository("npm-hosted").await.unwrap().unwrap(),
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    let loose = astrum_content::Asset {
        component_id: None,
        ..asset.clone()
    };
    use astrum_content::Gate;
    counter.store(0, std::sync::atomic::Ordering::Relaxed);
    gate.check(&ctx, &asset).await.unwrap();
    gate.check(&ctx, &loose).await.unwrap();
    gate.check(&npm_ctx, &loose).await.unwrap();
    assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert!(gate.check(&npm_ctx, &asset).await.is_err());
}

fn pub_form(archive: &[u8]) -> Vec<u8> {
    let boundary = "dart-http-boundary-0123456789";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"package.tar.gz\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(archive);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

async fn pub_get(fixture: &Fixture, path: String) -> Reply {
    fixture
        .send(
            Method::GET,
            &path,
            Auth::Admin,
            None,
            &[("accept", "application/vnd.pub.v2+json")],
        )
        .await
}

async fn publish_hello(fixture: &Fixture, repository: &str) -> String {
    let archive = astrum_format_pub::testing::archive(
        "name: hello\nversion: 1.0.0\ndescription: The hello package for tests.\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n",
        &[("lib/main.dart", b"// hello\n")],
    );
    let new = pub_get(
        fixture,
        format!("/repository/{repository}/api/packages/versions/new"),
    )
    .await;
    assert_eq!(new.status, StatusCode::OK, "{}", new.text());
    let upload = fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/api/packages/versions/upload"),
            Auth::Admin,
            Body::from(pub_form(&archive)),
            &[(
                "content-type",
                "multipart/form-data; boundary=dart-http-boundary-0123456789",
            )],
        )
        .await;
    let finalize = upload.headers[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let finalize = finalize
        .strip_prefix("http://localhost")
        .unwrap_or(&finalize);
    assert_eq!(
        pub_get(fixture, finalize.to_owned()).await.status,
        StatusCode::OK
    );
    format!("/repository/{repository}/packages/hello/versions/1.0.0.tar.gz")
}

#[tokio::test]
async fn a_format_with_its_own_error_documents_still_answers_forbidden_with_the_policy_text() {
    let fixture = ready().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_pub::Pub));
    strict(&fixture).await;
    repository(&fixture, "pub", "hosted", "pub-hosted", Some("strict")).await;
    let path = publish_hello(&fixture, "pub-hosted").await;
    assert_eq!(get(&fixture, &path).await.status, StatusCode::OK);
    let id = component(&fixture, "pub-hosted", "hello").await;
    seed(&fixture, "pub-hosted", id, "block", true, finding("GHSA-1")).await;

    let reply = pub_get(&fixture, path).await;

    refused(&reply);
    assert!(reply.headers.get(header::WWW_AUTHENTICATE).is_none());
    let document = reply.json();
    assert_eq!(document["error"]["code"], "InsufficientPermissions");
    assert!(
        document["error"]["message"]
            .as_str()
            .unwrap()
            .starts_with("blocked by policy strict: GHSA-1"),
        "{document}"
    );
}

#[tokio::test]
async fn a_component_a_worker_holds_right_now_is_waited_for_briefly_and_then_follows_the_setting() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    let id = component(&fixture, "npm-hosted", package.name).await;
    let revision = revision_of(&fixture, "strict").await;
    let held = hold(
        &fixture,
        &format!("SELECT 1 FROM scan_queue WHERE component_id = {id} FOR UPDATE"),
    )
    .await;
    let arriving = format!(
        "INSERT INTO policy_verdict (repository, subject, policy, component_id, blocked, state, reasons, policy_revision) \
         VALUES ('npm-hosted', 'c:{id}', 'strict', {id}, true, 'block', '{}'::jsonb, {revision})",
        finding("GHSA-1")
    );
    let conn = fixture.state.db().conn().clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        conn.execute_unprepared(&arriving).await.unwrap();
    });
    refused(&get(&fixture, &package.path).await);
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_blocked", 1)])
    );

    execute(&fixture, "DELETE FROM policy_verdict").await;
    let started = std::time::Instant::now();
    let reply = get(&fixture, &package.path).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert!(
        started.elapsed() >= Duration::from_millis(400)
            && started.elapsed() < Duration::from_millis(1500),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(results(&fixture)["inline_scan_error"], 1);
    assert_eq!(
        number(
            &fixture,
            &format!("SELECT count(*) AS n FROM policy_dirty WHERE subject = 'c:{id}'")
        )
        .await,
        1
    );
    held.release().await;
}

#[tokio::test]
async fn a_queue_row_that_is_not_due_follows_the_setting_at_once_and_does_not_poll() {
    use std::sync::atomic::Ordering;
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    let id = component(&fixture, "npm-hosted", package.name).await;
    execute(
        &fixture,
        &format!("UPDATE scan_queue SET next_attempt_at = now() + interval '1 hour' WHERE component_id = {id}"),
    )
    .await;
    let (content, counter) = counted(&fixture);
    let gate = PolicyGate::new(
        Arc::new(astrum_policy::cache::Cache::new(Duration::from_secs(3600))),
        Arc::default(),
    );
    let path = "/lib/-/lib-1.0.0.tgz";
    checked(&content, &gate, "npm-hosted", path).await.unwrap();

    counter.store(0, Ordering::Relaxed);
    let started = std::time::Instant::now();
    checked(&content, &gate, "npm-hosted", path).await.unwrap();

    assert!(
        started.elapsed() < Duration::from_millis(300),
        "{:?}",
        started.elapsed()
    );
    let statements = counter.load(Ordering::Relaxed);
    assert!(statements <= 10, "{statements} statements");
    assert_eq!(marks(&fixture).await, 1);
}

#[tokio::test]
async fn cold_downloads_behind_a_held_lock_follow_the_setting_within_the_budget_and_leave_the_pool_free()
 {
    let fixture = ready().await;
    policy(
        &fixture,
        "open",
        json!({"mode": "block", "severity": "high"}),
    )
    .await;
    policy(
        &fixture,
        "closed",
        json!({"mode": "block", "severity": "high", "unscanned": "block"}),
    )
    .await;
    repository(&fixture, "npm", "hosted", "npm-open", Some("open")).await;
    repository(&fixture, "npm", "hosted", "npm-closed", Some("closed")).await;
    repository(&fixture, "npm", "hosted", "npm-free", None).await;
    let mut paths = Vec::new();
    for index in 0..12 {
        for repository in ["npm-open", "npm-closed"] {
            let path = npm_path(&fixture, repository, &format!("lib{index}")).await;
            paths.push((repository, path));
        }
    }
    let free = npm_path(&fixture, "npm-free", "free").await;
    let gate = install_gate(&fixture, 300);
    let held = hold(&fixture, &package_lock()).await;

    let downloads = futures_util::future::join_all(paths.iter().map(|(repository, path)| {
        let fixture = &fixture;
        async move {
            let started = std::time::Instant::now();
            let reply = get(fixture, path).await;
            (*repository, reply, started.elapsed())
        }
    }));
    let unrelated = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let started = std::time::Instant::now();
        let reply = get(&fixture, &free).await;
        (reply, started.elapsed())
    };
    let (downloads, (free_reply, free_time)) = tokio::join!(downloads, unrelated);

    for (repository, reply, elapsed) in &downloads {
        match *repository {
            "npm-open" => assert_eq!(reply.status, StatusCode::OK, "{}", reply.text()),
            _ => assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text()),
        }
        assert!(
            *elapsed < Duration::from_secs(3),
            "{repository}: {elapsed:?}"
        );
    }
    assert_eq!(free_reply.status, StatusCode::OK, "{}", free_reply.text());
    assert!(free_time < Duration::from_secs(3), "{free_time:?}");
    waiting_settles_at(&fixture, 8).await;
    assert_eq!(gate.available_permits(), 0);
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_timeout", 24)])
    );
    assert_eq!(marks(&fixture).await, 24);

    held.release().await;
    settled(&gate, 8).await;
    let started = std::time::Instant::now();
    let after = get(&fixture, &paths[0].1).await;
    assert_eq!(after.status, StatusCode::OK, "{}", after.text());
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[tokio::test]
async fn a_scan_that_outlives_its_caller_keeps_its_slot_until_it_ends() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let first = npm_path(&fixture, "npm-hosted", "first").await;
    let second = npm_path(&fixture, "npm-hosted", "second").await;
    let gate = install_small_gate(&fixture, 300, 1);
    let held = hold(&fixture, &package_lock()).await;

    let reply = get(&fixture, &first).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(gate.available_permits(), 0);

    let started = std::time::Instant::now();
    let reply = get(&fixture, &second).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    waiting_settles_at(&fixture, 1).await;
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_timeout", 2)])
    );

    held.release().await;
    settled(&gate, 1).await;
    let first_id = component(&fixture, "npm-hosted", "first").await;
    let mut state = None;
    for _ in 0..200 {
        state = verdict_state(&fixture, first_id).await;
        if state.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        state.as_deref(),
        Some("pass"),
        "the scan went on after its caller had left"
    );
    for _ in 0..20 {
        let again = get(&fixture, &second).await;
        assert_eq!(again.status, StatusCode::OK, "{}", again.text());
        if results(&fixture).get("inline_scan_pass") == Some(&1) {
            break;
        }
        settled(&gate, 1).await;
    }
    assert_eq!(results(&fixture)["inline_scan_pass"], 1);
}

#[tokio::test]
async fn a_scan_at_download_whose_verdict_never_comes_gives_up_by_itself() {
    let fixture = ready().await;
    repository(&fixture, "npm", "hosted", "npm-free", None).await;
    let package = npm(&fixture, "npm-free").await;
    let id = component(&fixture, "npm-free", package.name).await;
    let security = Arc::new(astrum_security::SecurityModel::default());
    let started = std::time::Instant::now();

    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        inline::judge_component(
            fixture.state.content(),
            &security,
            fixture.state.policy_cache(),
            "npm-free",
            id,
        ),
    )
    .await
    .expect("the wait for the verdict has a bound of its own");

    assert!(
        matches!(outcome, Err(InlineError::NoVerdict)),
        "{outcome:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[tokio::test]
async fn the_preview_of_a_blocked_component_is_refused_like_its_download() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    repository(&fixture, "maven", "hosted", "maven-hosted", Some("strict")).await;
    group(&fixture, "npm", "npm-group", &["npm-hosted"]).await;
    let npm_package = npm(&fixture, "npm-hosted").await;
    let maven_package = maven(&fixture, "maven-hosted").await;
    let previews = [
        (
            "npm-hosted",
            npm_package.name,
            "npm-hosted",
            "/lib/-/lib-1.0.0.tgz",
        ),
        (
            "npm-group",
            npm_package.name,
            "npm-hosted",
            "/lib/-/lib-1.0.0.tgz",
        ),
        (
            "maven-hosted",
            maven_package.name,
            "maven-hosted",
            "/org/example/lib/1.0/lib-1.0.jar",
        ),
    ];
    let preview = |repository: &'static str, path: &'static str| {
        let fixture = &fixture;
        async move {
            let uri = format!("/service/rest/internal/ui/browse/{repository}/content?path={path}");
            get(fixture, &uri).await
        }
    };
    for (browsed, name, owner, path) in previews {
        execute(&fixture, "DELETE FROM policy_verdict").await;
        let open = preview(browsed, path).await;
        assert_eq!(open.status, StatusCode::OK, "{browsed}: {}", open.text());
        assert!(!open.body.is_empty());

        let id = component(&fixture, owner, name).await;
        seed(&fixture, owner, id, "block", true, finding("GHSA-1")).await;
        let blocked = preview(browsed, path).await;
        assert_eq!(
            blocked.status,
            StatusCode::FORBIDDEN,
            "{browsed}: {}",
            blocked.text()
        );
        assert!(
            blocked.text().contains("blocked by policy strict: GHSA-1"),
            "{browsed}: {}",
            blocked.text()
        );
    }
}

#[tokio::test]
async fn a_repository_deleted_while_the_gate_reads_answers_not_found_not_unavailable() {
    use astrum_content::Gate;
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    let content = fixture.state.content();
    let ctx = content.context(
        content.repository("npm-hosted").await.unwrap().unwrap(),
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    let asset = astrum_content::store::find(ctx.db(), ctx.name(), "/lib/-/lib-1.0.0.tgz")
        .await
        .unwrap()
        .unwrap();
    let removed = admin(
        &fixture,
        Method::DELETE,
        &format!("{REPOSITORIES}/npm-hosted"),
        None,
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let metrics: Arc<astrum_api::gate::Metrics> = Arc::default();
    let gate = PolicyGate::new(Arc::default(), metrics.clone());

    let error = gate.check(&ctx, &asset).await.unwrap_err();

    assert!(
        matches!(error, astrum_content::ContentError::NotFound(_)),
        "{error:?}"
    );
    assert_eq!(
        get(&fixture, &package.path).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(metrics.snapshot(), astrum_api::gate::Snapshot::default());
}

#[tokio::test]
async fn members_without_a_policy_are_counted_like_the_timing_histogram_counts_them() {
    let fixture = ready().await;
    repository(&fixture, "npm", "hosted", "npm-free", None).await;
    group(&fixture, "npm", "npm-group", &["npm-free"]).await;
    npm(&fixture, "npm-free").await;

    let reply = get(&fixture, "/repository/npm-group/lib/-/lib-1.0.0.tgz").await;

    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(results(&fixture), HashMap::from([("unguarded", 1)]));
    let snapshot = fixture.state.gate_metrics().snapshot();
    assert_eq!(snapshot.count, snapshot.results.iter().sum::<u64>());
}

#[tokio::test]
async fn a_scan_at_download_that_leaves_the_component_unscanned_is_counted_apart_from_a_pass() {
    let fixture = ready().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_pub::Pub));
    strict(&fixture).await;
    repository(&fixture, "pub", "hosted", "pub-hosted", Some("strict")).await;
    let path = publish_hello(&fixture, "pub-hosted").await;

    let reply = pub_get(&fixture, path.clone()).await;

    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_unscanned", 1)])
    );
    let again = pub_get(&fixture, path).await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.text());
    assert_eq!(results(&fixture)["pass"], 1);
}

#[tokio::test]
async fn many_downloads_of_one_cold_blocked_component_share_one_scan_and_all_get_the_refusal() {
    let fixture = ready().await;
    strict(&fixture).await;
    execute(&fixture, &advisory_for("lib", "1.0.0")).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    execute(&fixture, COUNT_SCANS).await;
    let gate = install_gate(&fixture, 2000);

    let replies =
        futures_util::future::join_all((0..20).map(|_| get(&fixture, &package.path))).await;

    for reply in &replies {
        refused(reply);
        assert!(reply.text().contains("GHSA-T1"), "{}", reply.text());
    }
    assert_eq!(
        number(&fixture, "SELECT count(*) AS n FROM scan_writes").await,
        1
    );
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_blocked", 20)])
    );
    assert_eq!(gate.flights_open(), 0);
}

#[tokio::test]
async fn a_download_that_follows_a_scan_gives_up_with_its_own_budget_and_leaves_the_mark_to_the_leader()
 {
    let fixture = ready().await;
    policy(
        &fixture,
        "closed",
        json!({"mode": "block", "severity": "high", "unscanned": "block"}),
    )
    .await;
    repository(&fixture, "npm", "hosted", "npm-closed", Some("closed")).await;
    let package = npm(&fixture, "npm-closed").await;
    execute(&fixture, COUNT_MARKS).await;
    let gate = install_gate(&fixture, 300);
    let held = hold(&fixture, &package_lock()).await;

    let replies =
        futures_util::future::join_all((0..5).map(|_| get(&fixture, &package.path))).await;

    for reply in &replies {
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
        assert!(
            reply
                .text()
                .contains("blocked by policy closed: the component has not been scanned yet"),
            "{}",
            reply.text()
        );
    }
    assert_eq!(
        number(&fixture, "SELECT count(*) AS n FROM mark_writes").await,
        1
    );
    assert_eq!(
        results(&fixture),
        HashMap::from([("inline_scan_timeout", 5)])
    );
    assert_eq!(gate.flights_open(), 1);
    held.release().await;
    settled(&gate, 8).await;
}

#[tokio::test]
async fn the_flight_map_is_empty_after_a_scan_ends_fails_or_its_leader_gives_up() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let done = npm_path(&fixture, "npm-hosted", "done").await;
    let stuck = npm_path(&fixture, "npm-hosted", "stuck").await;
    let first = npm_path(&fixture, "npm-hosted", "first").await;
    let second = npm_path(&fixture, "npm-hosted", "second").await;
    let stuck_id = component(&fixture, "npm-hosted", "stuck").await;
    execute(
        &fixture,
        &format!("UPDATE scan_queue SET next_attempt_at = now() + interval '1 hour' WHERE component_id = {stuck_id}"),
    )
    .await;
    let gate = install_small_gate(&fixture, 300, 1);

    assert_eq!(get(&fixture, &done).await.status, StatusCode::OK);
    assert_eq!(gate.flights_open(), 0, "after a scan that ended");
    assert_eq!(get(&fixture, &stuck).await.status, StatusCode::OK);
    assert_eq!(gate.flights_open(), 0, "after a scan that was refused");

    let held = hold(&fixture, &package_lock()).await;
    assert_eq!(get(&fixture, &first).await.status, StatusCode::OK);
    assert_eq!(
        gate.flights_open(),
        1,
        "the scan of a leader that left goes on"
    );
    assert_eq!(get(&fixture, &second).await.status, StatusCode::OK);
    assert_eq!(
        gate.flights_open(),
        1,
        "a leader that found no free slot leaves nothing behind"
    );
    held.release().await;
    settled(&gate, 1).await;
    assert_eq!(
        gate.flights_open(),
        0,
        "after the scan of a leader that left"
    );
}

#[tokio::test]
async fn scans_of_different_components_run_side_by_side() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let x = npm_path(&fixture, "npm-hosted", "x").await;
    let y = npm_path(&fixture, "npm-hosted", "y").await;
    let gate = install_small_gate(&fixture, 300, 2);
    let held = hold(&fixture, &package_lock()).await;

    let replies =
        futures_util::future::join_all([&x, &x, &y, &y].map(|path| get(&fixture, path))).await;

    for reply in &replies {
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    }
    assert_eq!(gate.flights_open(), 2);
    assert_eq!(gate.available_permits(), 0);
    waiting_settles_at(&fixture, 2).await;
    held.release().await;
    settled(&gate, 2).await;
    assert_eq!(gate.flights_open(), 0);
}

#[tokio::test]
async fn followers_of_one_scan_take_no_slot_so_another_component_still_gets_one() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let x = npm_path(&fixture, "npm-hosted", "x").await;
    let y = npm_path(&fixture, "npm-hosted", "y").await;
    let gate = install_small_gate(&fixture, 1500, 2);
    let held = hold(&fixture, &package_lock()).await;

    let same = futures_util::future::join_all((0..4).map(|_| get(&fixture, &x)));
    let other = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        get(&fixture, &y).await
    };
    let observer = async {
        tokio::time::sleep(Duration::from_millis(400)).await;
        let waiting = waiting_for_advisory_locks(&fixture).await;
        let open = gate.flights_open();
        held.release().await;
        (waiting, open)
    };
    let (same, other, (waiting, open)) = tokio::join!(same, other, observer);

    assert_eq!(waiting, 2, "one scan for x and one for y wait for the lock");
    assert_eq!(open, 2);
    for reply in same.iter().chain([&other]) {
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    }
    assert_eq!(results(&fixture), HashMap::from([("inline_scan_pass", 5)]));
    settled(&gate, 2).await;
}

#[tokio::test]
async fn the_scrape_names_every_gate_result_in_its_help_and_lists_it() {
    let fixture = ready().await;
    let enabled = admin(
        &fixture,
        Method::POST,
        "/service/rest/v1/capabilities",
        Some(json!({"type": "metrics.prometheus", "enabled": true, "properties": {}})),
    )
    .await;
    assert_eq!(enabled.status, StatusCode::CREATED, "{}", enabled.text());

    let text = get(&fixture, "/service/metrics/prometheus").await.text();

    let help = text
        .lines()
        .find(|line| line.starts_with("# HELP astrum_policy_gate_decisions_total "))
        .expect("the gate decisions have a help line");
    for outcome in Outcome::ALL {
        let label = outcome.label();
        assert!(help.contains(label), "{label} is not in: {help}");
        let sample = format!("astrum_policy_gate_decisions_total{{result=\"{label}\"}} 0");
        assert!(text.lines().any(|line| line == sample), "{sample}\n{text}");
    }
}

#[tokio::test]
async fn a_policy_the_cache_has_not_seen_yet_is_loaded_before_the_gate_judges_it() {
    let fixture = ready().await;
    strict(&fixture).await;
    repository(&fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let package = npm(&fixture, "npm-hosted").await;
    fixture
        .state
        .policy_cache()
        .get(fixture.state.db().conn())
        .await
        .unwrap();
    execute(
        &fixture,
        "INSERT INTO content_policy (name, vulnerabilities) VALUES ('late', '{\"mode\":\"block\",\"severity\":\"high\"}'); \
         UPDATE repository SET attributes = jsonb_set(attributes, '{policy,name}', '\"late\"') WHERE name = 'npm-hosted'",
    )
    .await;
    astrum_db::revision::bump(fixture.state.db().conn(), astrum_db::revision::POLICIES)
        .await
        .unwrap();

    let reply = get(&fixture, &package.path).await;

    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(results(&fixture), HashMap::from([("inline_scan_pass", 1)]));
}

async fn notices_written(gate: &PolicyGate) {
    for _ in 0..400 {
        if gate.notices_in_flight() == 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("{} notices still being written", gate.notices_in_flight());
}

async fn listening(fixture: &Fixture, events: &str) {
    execute(
        fixture,
        &format!(
            "INSERT INTO notification_channel (id, name, type, recipients, events) \
             VALUES (gen_random_uuid(), 'ops', 'email', '{{ops@example.com}}', '{{{events}}}')"
        ),
    )
    .await;
}

async fn blocked_notices(fixture: &Fixture) -> i64 {
    number(
        fixture,
        "SELECT count(*) AS n FROM notification_change WHERE action = 'policy.blocked'",
    )
    .await
}

async fn notices_reach(fixture: &Fixture, wanted: i64) {
    for _ in 0..200 {
        if blocked_notices(fixture).await >= wanted {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!(
        "{} notices after 5 s, wanted {wanted}",
        blocked_notices(fixture).await
    );
}

async fn blocked_world(fixture: &Fixture) -> (String, String) {
    strict(fixture).await;
    repository(fixture, "npm", "hosted", "npm-hosted", Some("strict")).await;
    let first = npm_path(fixture, "npm-hosted", "lib").await;
    let second = npm_path(fixture, "npm-hosted", "other").await;
    for name in ["lib", "other"] {
        let id = component(fixture, "npm-hosted", name).await;
        seed(fixture, "npm-hosted", id, "block", true, finding("GHSA-1")).await;
    }
    listening(fixture, "policy.blocked").await;
    (first, second)
}

#[tokio::test]
async fn a_subject_refused_again_the_same_day_queues_one_notice_and_another_subject_its_own() {
    let fixture = ready().await;
    let (lib, other) = blocked_world(&fixture).await;

    refused(&get(&fixture, &lib).await);
    notices_reach(&fixture, 1).await;
    refused(&get(&fixture, &lib).await);
    refused(&get(&fixture, &lib).await);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(blocked_notices(&fixture).await, 1);

    refused(&get(&fixture, &other).await);
    notices_reach(&fixture, 2).await;
    let id = component(&fixture, "npm-hosted", "lib").await;
    assert_eq!(
        number(
            &fixture,
            &format!(
                "SELECT count(*) AS n FROM notification_change WHERE action = 'policy.blocked' \
                 AND component_id = {id} AND repository = 'npm-hosted' \
                 AND detail = '{{\"policy\": \"strict\"}}'::jsonb"
            )
        )
        .await,
        1
    );
    assert!(!blocked_detail(&fixture).await.contains("GHSA-1"));
}

async fn blocked_detail(fixture: &Fixture) -> String {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT string_agg(detail::text, ' ') AS n FROM notification_change WHERE action = 'policy.blocked'"
                .to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

#[tokio::test]
async fn another_node_does_not_repeat_a_notice_of_the_same_day_but_does_on_the_next_one() {
    let fixture = ready().await;
    let (lib, _) = blocked_world(&fixture).await;

    refused(&get(&fixture, &lib).await);
    notices_reach(&fixture, 1).await;

    install_gate(&fixture, 2000);
    refused(&get(&fixture, &lib).await);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(blocked_notices(&fixture).await, 1);

    execute(
        &fixture,
        "UPDATE policy_blocked_notice SET noticed_at = now() - interval '25 hours'",
    )
    .await;
    install_gate(&fixture, 2000);
    refused(&get(&fixture, &lib).await);
    notices_reach(&fixture, 2).await;
}

#[tokio::test]
async fn a_notice_that_cannot_be_queued_changes_nothing_for_the_download_and_is_counted() {
    let fixture = ready().await;
    let (lib, _) = blocked_world(&fixture).await;
    execute(
        &fixture,
        "ALTER TABLE policy_blocked_notice RENAME TO policy_blocked_notice_away",
    )
    .await;

    let reply = get(&fixture, &lib).await;
    refused(&reply);
    for _ in 0..200 {
        if results(&fixture).get("notice_failed") == Some(&1) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert_eq!(results(&fixture).get("notice_failed"), Some(&1));
    assert_eq!(results(&fixture).get("blocked"), Some(&1));
    assert_eq!(blocked_notices(&fixture).await, 0);

    execute(
        &fixture,
        "ALTER TABLE policy_blocked_notice_away RENAME TO policy_blocked_notice",
    )
    .await;
    refused(&get(&fixture, &lib).await);
    notices_reach(&fixture, 1).await;
}

#[tokio::test]
async fn a_refusal_does_not_wait_for_the_notice_and_the_notice_follows_when_the_table_is_free() {
    let fixture = ready().await;
    let (lib, _) = blocked_world(&fixture).await;
    let locker = fixture.state.db().conn().begin().await.unwrap();
    locker
        .execute_unprepared("LOCK TABLE policy_blocked_notice IN ACCESS EXCLUSIVE MODE")
        .await
        .unwrap();

    let started = std::time::Instant::now();
    let reply = get(&fixture, &lib).await;
    let elapsed = started.elapsed();
    refused(&reply);
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
    assert_eq!(blocked_notices(&fixture).await, 0);

    locker.commit().await.unwrap();
    notices_reach(&fixture, 1).await;
}
