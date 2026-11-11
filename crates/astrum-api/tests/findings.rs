mod support;
mod ui_support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_api::AppState;
use astrum_content::ids;
use astrum_db::{Database, DatabaseConfig, DatabaseUrl};
use astrum_security::SecretBox;
use axum::http::{Method, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use ui_support::{anonymous, get, ready, role, selector, selector_privilege, user, user_with};

const POLICIES: &str = "/service/rest/v1/security/policies";
const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";
const FINDINGS: &str = "/service/rest/v1/security/findings";
const SUMMARY: &str = "/service/rest/v1/security/summary";
const ADVISORIES: &str = "/service/rest/v1/security/advisories";

async fn run(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(sql)
        .await
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
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
        .try_get("", "n")
        .unwrap()
}

async fn repository(fixture: &Fixture, name: &str, policy: Option<&str>) {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "ATTACHMENT"}
    });
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
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

async fn component(fixture: &Fixture, repository: &str, name: &str, paths: &[&str]) -> i64 {
    let id = number(
        fixture,
        &format!(
            "INSERT INTO component (repository, format, name, version) \
             VALUES ('{repository}', 'raw', '{name}', '1') RETURNING id AS n"
        ),
    )
    .await;
    for path in paths {
        run(
            fixture,
            &format!(
                "INSERT INTO asset (repository, format, path, component_id, blob_store, blob_id, \
                 size, content_type, sha1, sha256, sha512, md5) \
                 VALUES ('{repository}', 'raw', '{path}', {id}, 'default', gen_random_uuid(), 1, \
                 'text/plain', '', '', '', '')"
            ),
        )
        .await;
    }
    id
}

async fn advisory(
    fixture: &Fixture,
    id: &str,
    aliases: &[&str],
    severity: &str,
    score: &str,
    malicious: bool,
) {
    let aliases = aliases
        .iter()
        .map(|alias| format!("'{alias}'"))
        .collect::<Vec<_>>()
        .join(", ");
    run(
        fixture,
        &format!(
            "INSERT INTO advisory (id, aliases, summary, severity, score, malicious, modified_at, document) \
             VALUES ('{id}', ARRAY[{aliases}]::text[], 'summary of {id}', '{severity}', {score}, {malicious}, now(), '{{}}')"
        ),
    )
    .await;
}

async fn finding(
    fixture: &Fixture,
    repository: &str,
    component: i64,
    advisory: &str,
    fixed: Option<&str>,
) {
    let fixed = fixed.map_or("NULL".to_owned(), |fixed| format!("'{fixed}'"));
    let package = number(
        fixture,
        &format!(
            "INSERT INTO sca_package (ecosystem, name, version, purl, matched_at) \
             VALUES ('npm', 'pkg-{component}', '1.0.0', 'pkg:npm/pkg-{component}@1.0.0', now()) \
             ON CONFLICT (ecosystem, name, version) DO UPDATE SET matched_at = now() RETURNING id AS n"
        ),
    )
    .await;
    let inventory = number(
        fixture,
        &format!(
            "INSERT INTO sca_inventory (key, source, extractor) VALUES ('pkg:c{component}', 'metadata', 1) \
             ON CONFLICT (key) DO UPDATE SET extractor = 1 RETURNING id AS n"
        ),
    )
    .await;
    run(
        fixture,
        &format!(
            "INSERT INTO sca_inventory_package (inventory_id, package_id) VALUES ({inventory}, {package}) \
             ON CONFLICT DO NOTHING; \
             INSERT INTO component_scan (component_id, repository, inventory_id, state) \
             VALUES ({component}, '{repository}', {inventory}, 'scanned') \
             ON CONFLICT (component_id) DO UPDATE SET inventory_id = EXCLUDED.inventory_id; \
             INSERT INTO sca_match (package_id, advisory_id, fixed) VALUES ({package}, '{advisory}', {fixed}) \
             ON CONFLICT (package_id, advisory_id) DO UPDATE SET fixed = EXCLUDED.fixed"
        ),
    )
    .await;
}

async fn judge(fixture: &Fixture) {
    run(
        fixture,
        "INSERT INTO policy_dirty (repository, subject) \
         SELECT c.repository, 'c:' || c.id FROM component c JOIN repository r ON r.name = c.repository \
         WHERE r.attributes #>> '{policy,name}' IS NOT NULL ON CONFLICT DO NOTHING",
    )
    .await;
    let config = astrum_scan::verdict::Config::default();
    loop {
        let judged = astrum_scan::verdict::judge_batch(
            fixture.state.db().conn(),
            fixture.state.policy_cache(),
            &config,
        )
        .await
        .unwrap();
        if judged.taken == 0 {
            break;
        }
    }
}

struct World {
    fixture: Fixture,
    alpha: i64,
    beta: i64,
    gamma: i64,
    delta: i64,
    bare: i64,
}

async fn world() -> World {
    let fixture = ready().await;
    let created = fixture
        .call(
            Method::POST,
            POLICIES,
            Auth::Admin,
            Some(
                json!({"name": "strict", "vulnerabilities": {"mode": "block", "severity": "high"}}),
            ),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    repository(&fixture, "files", Some("strict")).await;
    repository(&fixture, "other", Some("strict")).await;
    repository(&fixture, "bare", None).await;
    advisory(
        &fixture,
        "GHSA-aaaa-0001",
        &["CVE-2024-0001"],
        "critical",
        "9.8",
        false,
    )
    .await;
    advisory(
        &fixture,
        "GHSA-bbbb-0002",
        &["CVE-2024-0002"],
        "high",
        "7.5",
        false,
    )
    .await;
    advisory(&fixture, "MAL-2024-1", &[], "critical", "NULL", true).await;
    advisory(&fixture, "GHSA-cccc-0003", &[], "low", "2.1", false).await;
    let alpha = component(&fixture, "files", "alpha", &["/a/b/one"]).await;
    let beta = component(&fixture, "files", "beta", &["/a/x", "/a/b/two"]).await;
    let gamma = component(&fixture, "files", "gamma", &["/z/three"]).await;
    let delta = component(&fixture, "other", "delta", &["/d/four"]).await;
    let bare = component(&fixture, "bare", "epsilon", &["/e/five"]).await;
    finding(&fixture, "files", alpha, "GHSA-aaaa-0001", Some("1.2.3")).await;
    finding(&fixture, "files", beta, "GHSA-bbbb-0002", None).await;
    finding(&fixture, "files", gamma, "GHSA-bbbb-0002", None).await;
    finding(&fixture, "files", gamma, "MAL-2024-1", None).await;
    finding(&fixture, "other", delta, "GHSA-aaaa-0001", None).await;
    finding(&fixture, "bare", bare, "GHSA-cccc-0003", None).await;
    judge(&fixture).await;
    World {
        fixture,
        alpha,
        beta,
        gamma,
        delta,
        bare,
    }
}

fn component_path(repository: &str, id: i64) -> String {
    format!(
        "/service/rest/v1/components/{}/security",
        ids::encode(repository, id)
    )
}

async fn ask(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    get(fixture, path, auth).await
}

async fn ok(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Value {
    let reply = ask(fixture, path, auth).await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply.json()
}

fn advisories_of(page: &Value) -> Vec<(i64, String)> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            let (_, id) = ids::decode(item["componentId"].as_str().unwrap()).unwrap();
            (id, item["advisory"].as_str().unwrap().to_owned())
        })
        .collect()
}

async fn two_users(fixture: &Fixture) {
    fixture.add_user("bob", "password-1", &[]).await;
    user_with(fixture, "alice", &["as-repository-view-raw-files-browse"]).await;
    selector(fixture, "deep", "path =^ \"/a/b/\"").await;
    selector_privilege(fixture, "files-deep", "deep", "raw", "files", &["BROWSE"]).await;
    user_with(fixture, "deep", &["files-deep"]).await;
    selector(fixture, "nothing", "path =^ \"/nothing/\"").await;
    selector_privilege(
        fixture,
        "files-nothing",
        "nothing",
        "raw",
        "files",
        &["BROWSE"],
    )
    .await;
    user_with(fixture, "hidden", &["files-nothing"]).await;
}

#[tokio::test]
async fn anonymous_callers_get_401_on_every_route() {
    let world = world().await;
    let paths = [
        component_path("files", world.alpha),
        FINDINGS.to_owned(),
        format!("{ADVISORIES}/GHSA-aaaa-0001"),
        SUMMARY.to_owned(),
    ];
    for path in paths {
        let reply = ask(&world.fixture, &path, Auth::None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{path}");
    }
}

#[tokio::test]
async fn a_caller_without_browse_gets_the_answer_for_nothing() {
    let world = world().await;
    let fixture = &world.fixture;
    two_users(fixture).await;

    let real = ask(fixture, &component_path("files", world.alpha), user("bob")).await;
    let missing = ask(fixture, &component_path("files", 987_654), user("bob")).await;
    assert_eq!(real.status, StatusCode::NOT_FOUND);
    assert_eq!((real.status, &real.body), (missing.status, &missing.body));
    let foreign = ask(
        fixture,
        &component_path("other", world.delta),
        user("alice"),
    )
    .await;
    let absent = ask(fixture, &component_path("other", 987_654), user("alice")).await;
    assert_eq!(
        (foreign.status, &foreign.body),
        (absent.status, &absent.body)
    );

    let known = ask(
        fixture,
        &format!("{ADVISORIES}/GHSA-aaaa-0001"),
        user("bob"),
    )
    .await;
    let unknown = ask(
        fixture,
        &format!("{ADVISORIES}/GHSA-zzzz-9999"),
        user("bob"),
    )
    .await;
    assert_eq!(known.status, StatusCode::FORBIDDEN);
    assert_eq!((known.status, &known.body), (unknown.status, &unknown.body));

    let empty = ask(fixture, FINDINGS, user("bob")).await;
    assert_eq!(
        empty.json(),
        json!({"items": [], "continuationToken": null})
    );
    let seen = ask(
        fixture,
        &format!("{FINDINGS}?repository=other"),
        user("alice"),
    )
    .await;
    let ghost = ask(
        fixture,
        &format!("{FINDINGS}?repository=ghost"),
        user("alice"),
    )
    .await;
    assert_eq!((seen.status, &seen.body), (ghost.status, &ghost.body));

    let summary = ok(fixture, SUMMARY, user("bob")).await;
    assert_eq!(summary["repositories"], json!([]));
    assert_eq!(summary["totals"]["componentsScanned"], 0);
    assert_eq!(summary["totals"]["critical"], 0);

    let findings = ok(fixture, FINDINGS, user("alice")).await;
    assert!(
        advisories_of(&findings)
            .iter()
            .all(|(id, _)| [world.alpha, world.beta, world.gamma].contains(id))
    );
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-aaaa-0001"),
        user("alice"),
    )
    .await;
    assert_eq!(advisory["componentCount"], 1);
    assert_eq!(advisory["components"].as_array().unwrap().len(), 1);
    let summary = ok(fixture, SUMMARY, user("alice")).await;
    let names: Vec<&str> = summary["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["repository"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["files"]);
}

#[tokio::test]
async fn a_selector_that_hides_every_asset_hides_the_component_and_its_counts() {
    let world = world().await;
    let fixture = &world.fixture;
    two_users(fixture).await;

    for id in [world.alpha, world.beta, world.gamma] {
        let reply = ask(fixture, &component_path("files", id), user("hidden")).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{id}");
    }
    let page = ok(fixture, FINDINGS, user("hidden")).await;
    assert_eq!(page["items"], json!([]));
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-bbbb-0002"),
        user("hidden"),
    )
    .await;
    assert_eq!(advisory["componentCount"], 0);
    assert_eq!(advisory["components"], json!([]));
    let summary = ok(fixture, SUMMARY, user("hidden")).await;
    assert_eq!(summary["repositories"], json!([]));
    assert_eq!(summary["totals"]["high"], 0);
    assert_eq!(summary["totals"]["componentsScanned"], 0);
}

#[tokio::test]
async fn a_selector_that_shows_one_asset_of_two_shows_the_component() {
    let world = world().await;
    let fixture = &world.fixture;
    two_users(fixture).await;

    let shown = ask(fixture, &component_path("files", world.beta), user("deep")).await;
    assert_eq!(shown.status, StatusCode::OK, "{}", shown.text());
    let hidden = ask(fixture, &component_path("files", world.gamma), user("deep")).await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);

    let page = ok(fixture, FINDINGS, user("deep")).await;
    let seen = advisories_of(&page);
    assert!(seen.contains(&(world.alpha, "GHSA-aaaa-0001".to_owned())));
    assert!(seen.contains(&(world.beta, "GHSA-bbbb-0002".to_owned())));
    assert!(seen.iter().all(|(id, _)| *id != world.gamma));

    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-bbbb-0002"),
        user("deep"),
    )
    .await;
    assert_eq!(advisory["componentCount"], 1);
    assert_eq!(
        advisory["components"][0]["componentId"],
        ids::encode("files", world.beta)
    );
    let summary = ok(fixture, SUMMARY, user("deep")).await;
    assert_eq!(summary["totals"]["componentsScanned"], 2);
    assert_eq!(summary["totals"]["critical"], 1);
    assert_eq!(summary["totals"]["high"], 1);
    assert_eq!(summary["totals"]["componentsVulnerable"], 2);
}

#[tokio::test]
async fn the_security_of_a_component_has_every_field() {
    let world = world().await;
    let fixture = &world.fixture;

    let alpha = ok(fixture, &component_path("files", world.alpha), Auth::Admin).await;
    assert_eq!(alpha["scan"]["state"], "scanned");
    assert_eq!(alpha["scan"]["packages"], 1);
    assert!(alpha["scan"]["scannedAt"].is_string());
    assert_eq!(alpha["policy"]["name"], "strict");
    assert_eq!(alpha["policy"]["revision"], 1);
    assert_eq!(alpha["verdict"]["blocked"], true);
    assert_eq!(alpha["verdict"]["state"], "block");
    assert_eq!(alpha["verdict"]["policy"], "strict");
    assert_eq!(alpha["verdict"]["counts"]["critical"], 1);
    assert_eq!(alpha["verdict"]["maxSeverity"], "critical");
    assert!(alpha["verdict"]["reasons"].is_array());
    assert_eq!(alpha["license"], Value::Null);
    assert!(alpha.get("signatures").is_none());
    assert_eq!(
        alpha["vulnerabilities"],
        json!([{
            "id": "GHSA-aaaa-0001",
            "aliases": ["CVE-2024-0001"],
            "severity": "critical",
            "score": 9.8,
            "summary": "summary of GHSA-aaaa-0001",
            "fixed": "1.2.3",
            "malicious": false,
            "excepted": false,
        }])
    );

    let gamma = ok(fixture, &component_path("files", world.gamma), Auth::Admin).await;
    let found = gamma["vulnerabilities"].as_array().unwrap();
    assert_eq!(found.len(), 2);
    let malicious = found
        .iter()
        .find(|item| item["id"] == "MAL-2024-1")
        .unwrap();
    assert_eq!(malicious["malicious"], true);
    assert_eq!(malicious["score"], Value::Null);
    assert_eq!(malicious["fixed"], Value::Null);

    let bare = ok(fixture, &component_path("bare", world.bare), Auth::Admin).await;
    assert_eq!(bare["policy"], Value::Null);
    assert_eq!(bare["verdict"], Value::Null);
    assert_eq!(bare["vulnerabilities"].as_array().unwrap().len(), 1);

    run(
        fixture,
        &format!(
            "DELETE FROM component_scan WHERE component_id = {}",
            world.beta
        ),
    )
    .await;
    let beta = ok(fixture, &component_path("files", world.beta), Auth::Admin).await;
    assert_eq!(beta["scan"]["state"], "queued");
    assert_eq!(beta["scan"]["scannedAt"], Value::Null);
    assert_eq!(beta["vulnerabilities"], json!([]));
    run(fixture, "DELETE FROM scan_queue").await;
    let beta = ok(fixture, &component_path("files", world.beta), Auth::Admin).await;
    assert_eq!(beta["scan"]["state"], "unscanned");
}

#[tokio::test]
async fn an_exception_marks_the_finding_and_a_newer_policy_hides_the_old_verdict() {
    let world = world().await;
    let fixture = &world.fixture;

    let created = fixture
        .call(
            Method::POST,
            EXCEPTIONS,
            Auth::Admin,
            Some(json!({
                "rule": "vulnerability",
                "advisory": "cve-2024-0001",
                "policy": "strict",
                "reason": "accepted until the next release",
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    judge(fixture).await;

    let alpha = ok(fixture, &component_path("files", world.alpha), Auth::Admin).await;
    assert_eq!(alpha["vulnerabilities"][0]["excepted"], true);
    assert_eq!(alpha["verdict"]["blocked"], false);
    let page = ok(
        fixture,
        &format!("{FINDINGS}?advisory=GHSA-aaaa-0001"),
        Auth::Admin,
    )
    .await;
    let flags: Vec<(i64, bool, bool)> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            let (_, id) = ids::decode(item["componentId"].as_str().unwrap()).unwrap();
            (
                id,
                item["excepted"].as_bool().unwrap(),
                item["blocked"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        flags,
        [(world.alpha, true, false), (world.delta, true, false)]
    );

    let updated = fixture
        .call(
            Method::PUT,
            &format!("{POLICIES}/strict"),
            Auth::Admin,
            Some(json!({"vulnerabilities": {"mode": "block", "severity": "critical"}})),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    let alpha = ok(fixture, &component_path("files", world.alpha), Auth::Admin).await;
    assert_eq!(alpha["verdict"], Value::Null);
    assert_eq!(alpha["policy"]["revision"], 2);
    let page = ok(
        fixture,
        &format!("{FINDINGS}?blocked=false&advisory=GHSA-aaaa-0001"),
        Auth::Admin,
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    judge(fixture).await;
    let alpha = ok(fixture, &component_path("files", world.alpha), Auth::Admin).await;
    assert_eq!(alpha["verdict"]["policy"], "strict");
    assert_eq!(alpha["verdict"]["revision"], 2);
}

#[tokio::test]
async fn findings_filter_by_severity_advisory_blocked_and_repository() {
    let world = world().await;
    let fixture = &world.fixture;
    let list = |query: &str| format!("{FINDINGS}?{query}");

    let critical = ok(fixture, &list("severity=critical"), Auth::Admin).await;
    assert_eq!(
        advisories_of(&critical),
        [
            (world.alpha, "GHSA-aaaa-0001".to_owned()),
            (world.gamma, "MAL-2024-1".to_owned()),
            (world.delta, "GHSA-aaaa-0001".to_owned()),
        ]
    );
    let all = ok(fixture, FINDINGS, Auth::Admin).await;
    assert_eq!(all["items"].as_array().unwrap().len(), 6);
    let severities: Vec<&str> = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["severity"].as_str().unwrap())
        .collect();
    assert_eq!(
        severities,
        ["critical", "critical", "critical", "high", "high", "low"]
    );

    for query in [
        "advisory=ghsa-aaaa-0001",
        "advisory=CVE-2024-0001",
        "advisory=cve-2024-0001",
    ] {
        let page = ok(fixture, &list(query), Auth::Admin).await;
        assert_eq!(
            advisories_of(&page),
            [
                (world.alpha, "GHSA-aaaa-0001".to_owned()),
                (world.delta, "GHSA-aaaa-0001".to_owned())
            ],
            "{query}"
        );
    }
    let none = ok(fixture, &list("advisory=GHSA-aaaa"), Auth::Admin).await;
    assert_eq!(none["items"], json!([]));

    let blocked = ok(fixture, &list("blocked=true"), Auth::Admin).await;
    let blocked_components: Vec<i64> = advisories_of(&blocked).iter().map(|(id, _)| *id).collect();
    assert!(blocked_components.contains(&world.alpha));
    assert!(blocked_components.contains(&world.gamma));
    assert!(!blocked_components.contains(&world.bare));
    let open = ok(fixture, &list("blocked=false"), Auth::Admin).await;
    let open_components: Vec<i64> = advisories_of(&open).iter().map(|(id, _)| *id).collect();
    assert!(open_components.contains(&world.bare));
    assert!(!open_components.contains(&world.alpha));
    assert_eq!(
        blocked["items"].as_array().unwrap().len() + open["items"].as_array().unwrap().len(),
        6
    );

    let files = ok(fixture, &list("repository=files"), Auth::Admin).await;
    assert_eq!(files["items"].as_array().unwrap().len(), 4);
    let item = &files["items"][0];
    assert_eq!(item["repository"], "files");
    assert_eq!(item["format"], "raw");
    assert_eq!(item["namespace"], "");
    assert_eq!(item["version"], "1");
    assert_eq!(item["score"], 9.8);
    assert_eq!(item["fixed"], "1.2.3");
    assert_eq!(item["malicious"], false);

    for query in [
        "severity=urgent",
        "severity=HIGH",
        "blocked=maybe",
        "limit=0",
        "limit=201",
        "repository=",
        "advisory=",
        "continuationToken=%21%21",
    ] {
        let reply = ask(fixture, &list(query), Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{query}");
    }
}

#[tokio::test]
async fn pages_follow_a_keyset_that_survives_new_findings() {
    let fixture = ready().await;
    repository(&fixture, "files", None).await;
    advisory(&fixture, "GHSA-high-0001", &[], "high", "7.0", false).await;
    advisory(&fixture, "GHSA-crit-0001", &[], "critical", "9.5", false).await;
    let mut originals = Vec::new();
    for index in 0..5 {
        let id = component(
            &fixture,
            "files",
            &format!("c{index}"),
            &[&format!("/p/{index}")],
        )
        .await;
        finding(&fixture, "files", id, "GHSA-high-0001", None).await;
        originals.push(id);
    }
    let first = ok(&fixture, &format!("{FINDINGS}?limit=2"), Auth::Admin).await;
    let token = first["continuationToken"].as_str().unwrap().to_owned();
    assert_eq!(first["items"].as_array().unwrap().len(), 2);

    let late = component(&fixture, "files", "late", &["/p/late"]).await;
    finding(&fixture, "files", late, "GHSA-high-0001", None).await;
    finding(&fixture, "files", originals[4], "GHSA-crit-0001", None).await;

    let second = ok(
        &fixture,
        &format!("{FINDINGS}?limit=2&continuationToken={token}"),
        Auth::Admin,
    )
    .await;
    let token = second["continuationToken"].as_str().unwrap().to_owned();
    let third = ok(
        &fixture,
        &format!("{FINDINGS}?limit=2&continuationToken={token}"),
        Auth::Admin,
    )
    .await;
    assert_eq!(third["continuationToken"], Value::Null);

    let seen: Vec<(i64, String)> = [&first, &second, &third]
        .iter()
        .flat_map(|page| advisories_of(page))
        .collect();
    let mut unique = seen.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), seen.len());
    let high: Vec<i64> = seen
        .iter()
        .filter(|(_, advisory)| advisory == "GHSA-high-0001")
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(
        high,
        [
            originals[0],
            originals[1],
            originals[2],
            originals[3],
            originals[4],
            late
        ]
    );

    let exact = ok(&fixture, &format!("{FINDINGS}?limit=7"), Auth::Admin).await;
    assert_eq!(exact["continuationToken"], Value::Null);
    assert_eq!(exact["items"].as_array().unwrap().len(), 7);
    assert_eq!(exact["items"][0]["advisory"], "GHSA-crit-0001");
    let reply = ask(
        &fixture,
        &format!("{FINDINGS}?continuationToken=bm9wZQ"),
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_summary_adds_up_and_flags_a_cut_count() {
    let world = world().await;
    let fixture = &world.fixture;
    let summary = ok(fixture, SUMMARY, Auth::Admin).await;
    assert!(summary["generatedAt"].is_string());
    let totals = &summary["totals"];
    assert_eq!(totals["critical"], 3);
    assert_eq!(totals["high"], 2);
    assert_eq!(totals["low"], 1);
    assert_eq!(totals["medium"], 0);
    assert_eq!(totals["unknown"], 0);
    assert_eq!(totals["blocked"], 4);
    assert_eq!(totals["componentsScanned"], 5);
    assert_eq!(totals["componentsVulnerable"], 5);
    assert_eq!(totals["truncated"], false);
    let rows = summary["repositories"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["repository"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["bare", "files", "other"]
    );
    for key in ["critical", "high", "medium", "low", "unknown", "blocked"] {
        let sum: i64 = rows.iter().map(|row| row[key].as_i64().unwrap()).sum();
        assert_eq!(sum, totals[key].as_i64().unwrap(), "{key}");
    }
    let files = rows
        .iter()
        .find(|row| row["repository"] == "files")
        .unwrap();
    assert_eq!(files["format"], "raw");
    assert_eq!(files["truncated"], false);

    two_users(fixture).await;
    fixture.state.set_selector_row_cap(2);
    let cut = ok(fixture, SUMMARY, user("deep")).await;
    assert_eq!(cut["repositories"][0]["truncated"], true);
    assert_eq!(cut["totals"]["truncated"], true);
    assert_eq!(cut["totals"]["componentsScanned"], 2);
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-bbbb-0002"),
        user("deep"),
    )
    .await;
    assert_eq!(advisory["componentsTruncated"], false);
    fixture.state.set_selector_row_cap(1);
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-bbbb-0002"),
        user("deep"),
    )
    .await;
    assert_eq!(advisory["componentsTruncated"], true);
    fixture.state.set_selector_row_cap(20_000);
    let whole = ok(fixture, SUMMARY, user("deep")).await;
    assert_eq!(whole["totals"]["truncated"], false);
}

#[tokio::test]
async fn an_advisory_lists_its_references_affected_ranges_and_visible_components() {
    let world = world().await;
    let fixture = &world.fixture;
    run(
        fixture,
        "UPDATE advisory SET published_at = '2024-05-01T00:00:00Z', document = jsonb_build_object('references', \
         jsonb_build_array(jsonb_build_object('type', 'ADVISORY', 'url', 'https://example.org/a'), \
         jsonb_build_object('type', 'WEB', 'url', 'javascript:alert(1)'))) WHERE id = 'GHSA-aaaa-0001'; \
         INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges, versions) \
         VALUES ('GHSA-aaaa-0001', 'npm', 'pkg', '[{\"type\":\"SEMVER\",\"events\":[{\"introduced\":\"0\"},{\"fixed\":\"1.2.3\"}]}]', ARRAY['1.0.0'])",
    )
    .await;
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-aaaa-0001"),
        Auth::Admin,
    )
    .await;
    assert_eq!(advisory["id"], "GHSA-aaaa-0001");
    assert_eq!(advisory["aliases"], json!(["CVE-2024-0001"]));
    assert_eq!(advisory["severity"], "critical");
    assert_eq!(advisory["score"], 9.8);
    assert_eq!(advisory["malicious"], false);
    assert_eq!(advisory["published"], "2024-05-01T00:00:00Z");
    assert!(advisory["modified"].is_string());
    assert_eq!(
        advisory["references"],
        json!([{"type": "ADVISORY", "url": "https://example.org/a"}])
    );
    assert_eq!(advisory["affected"][0]["ecosystem"], "npm");
    assert_eq!(advisory["affected"][0]["versions"], json!(["1.0.0"]));
    assert_eq!(advisory["componentCount"], 2);
    let first = &advisory["components"][0];
    assert_eq!(first["componentId"], ids::encode("files", world.alpha));
    assert_eq!(first["repository"], "files");
    assert_eq!(first["format"], "raw");
    assert_eq!(first["name"], "alpha");
    assert_eq!(first["version"], "1");
    assert_eq!(first["fixed"], "1.2.3");
    assert_eq!(first["excepted"], false);
    assert_eq!(first["blocked"], true);

    let missing = ask(
        fixture,
        &format!("{ADVISORIES}/GHSA-zzzz-9999"),
        Auth::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    advisory_many(fixture).await;
    let many = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-many-0001"),
        Auth::Admin,
    )
    .await;
    assert_eq!(many["componentCount"], 105);
    assert_eq!(many["components"].as_array().unwrap().len(), 100);
}

async fn advisory_many(fixture: &Fixture) {
    advisory(fixture, "GHSA-many-0001", &[], "medium", "5.0", false).await;
    for index in 0..105 {
        let id = component(
            fixture,
            "files",
            &format!("many{index}"),
            &[&format!("/many/{index}")],
        )
        .await;
        finding(fixture, "files", id, "GHSA-many-0001", None).await;
    }
}

fn feed(pending: &mut Vec<u8>, started: &mut bool, executes: &AtomicUsize) {
    loop {
        if !*started {
            if pending.len() < 4 {
                return;
            }
            let length = u32::from_be_bytes([pending[0], pending[1], pending[2], pending[3]]);
            if pending.len() < length as usize {
                return;
            }
            pending.drain(..length as usize);
            *started = true;
            continue;
        }
        if pending.len() < 5 {
            return;
        }
        let length = u32::from_be_bytes([pending[1], pending[2], pending[3], pending[4]]) as usize;
        if pending.len() < length + 1 {
            return;
        }
        if pending[0] == b'E' {
            executes.fetch_add(1, Ordering::Relaxed);
        }
        pending.drain(..length + 1);
    }
}

async fn serve(client: TcpStream, upstream: String, executes: Arc<AtomicUsize>) {
    let Ok(server) = TcpStream::connect(&upstream).await else {
        return;
    };
    let (mut from_client, mut to_client) = client.into_split();
    let (mut from_server, mut to_server) = server.into_split();
    let forward = async {
        let mut buffer = vec![0u8; 16 * 1024];
        let mut pending = Vec::new();
        let mut started = false;
        loop {
            let read = from_client.read(&mut buffer).await.unwrap_or(0);
            if read == 0 || to_server.write_all(&buffer[..read]).await.is_err() {
                return;
            }
            pending.extend_from_slice(&buffer[..read]);
            feed(&mut pending, &mut started, &executes);
        }
    };
    tokio::select! {
        _ = forward => {}
        _ = tokio::io::copy(&mut from_server, &mut to_client) => {}
    }
}

async fn counting(base: Fixture) -> (Fixture, Arc<AtomicUsize>) {
    let url = base.test.url().expose().to_owned();
    let start = url
        .find('@')
        .map_or_else(|| url.find("://").unwrap() + 3, |at| at + 1);
    let end = url[start..].find('/').unwrap() + start;
    let upstream = url[start..end].to_owned();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let executes = Arc::new(AtomicUsize::new(0));
    let counted = executes.clone();
    tokio::spawn(async move {
        while let Ok((client, _)) = listener.accept().await {
            tokio::spawn(serve(client, upstream.clone(), counted.clone()));
        }
    });
    let separator = if url.contains('?') { '&' } else { '?' };
    let through = format!(
        "{}{address}{}{separator}sslmode=disable",
        &url[..start],
        &url[end..]
    );
    let config = DatabaseConfig::new(DatabaseUrl::parse(&through).unwrap());
    let db = Database::connect(&config).await.unwrap();
    let state = AppState::load(
        db,
        base.data.path().to_owned(),
        SecretBox::from_key([7; 32]),
    )
    .await
    .unwrap();
    (Fixture { state, ..base }, executes)
}

async fn statements(fixture: &Fixture, executes: &AtomicUsize, path: &str) -> usize {
    ok(fixture, path, Auth::Admin).await;
    let mut least = usize::MAX;
    for _ in 0..2 {
        let before = executes.load(Ordering::Relaxed);
        ok(fixture, path, Auth::Admin).await;
        least = least.min(executes.load(Ordering::Relaxed) - before);
    }
    least
}

#[tokio::test]
async fn the_findings_list_costs_the_same_statements_for_three_and_thirty_components() {
    let base = ready().await;
    repository(&base, "files", None).await;
    let (fixture, executes) = counting(base).await;
    advisory(&fixture, "GHSA-many-0001", &[], "high", "7.0", false).await;
    let path = format!("{FINDINGS}?limit=200");
    for index in 0..30 {
        let id = component(
            &fixture,
            "files",
            &format!("m{index}"),
            &[&format!("/m/{index}")],
        )
        .await;
        finding(&fixture, "files", id, "GHSA-many-0001", None).await;
        if index == 2 {
            let page = ok(&fixture, &path, Auth::Admin).await;
            assert_eq!(page["items"].as_array().unwrap().len(), 3);
        }
    }
    let large = statements(&fixture, &executes, &path).await;
    run(
        &fixture,
        "DELETE FROM component WHERE name NOT IN ('m0', 'm1', 'm2')",
    )
    .await;
    let small = statements(&fixture, &executes, &path).await;
    let page = ok(&fixture, &path, Auth::Admin).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 3);
    assert_eq!(small, large);
    assert!(large <= 8, "{large} statements");
}

fn wrap(sealed: &str) -> String {
    URL_SAFE_NO_PAD.encode(sealed)
}

#[tokio::test]
async fn a_continuation_token_is_sealed_and_anything_else_is_refused() {
    let world = world().await;
    let fixture = &world.fixture;
    let page = ok(fixture, &format!("{FINDINGS}?limit=2"), Auth::Admin).await;
    let token = page["continuationToken"].as_str().unwrap().to_owned();
    let bytes = URL_SAFE_NO_PAD.decode(&token).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(!text.contains("after:") && !text.contains("GHSA") && !text.contains("MAL-"));
    let next = ok(
        fixture,
        &format!("{FINDINGS}?limit=2&continuationToken={token}"),
        Auth::Admin,
    )
    .await;
    assert_eq!(next["items"].as_array().unwrap().len(), 2);

    let plain = URL_SAFE_NO_PAD.encode("after:4:1:GHSA-aaaa-0001");
    let foreign = wrap(&SecretBox::from_key([9; 32]).seal("after:4:1:GHSA-aaaa-0001"));
    let mut tampered = text.clone();
    tampered.pop();
    tampered.push(if text.ends_with('A') { 'B' } else { 'A' });
    let mut body = STANDARD
        .decode(text.strip_prefix("astrum:v1:").unwrap())
        .unwrap();
    let middle = body.len() / 2;
    body[middle] ^= 1;
    let flipped = wrap(&format!("astrum:v1:{}", STANDARD.encode(body)));
    for bad in [
        plain,
        foreign,
        wrap(&tampered),
        flipped,
        wrap("secret:garbage"),
    ] {
        let reply = ask(
            fixture,
            &format!("{FINDINGS}?continuationToken={bad}"),
            Auth::Admin,
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad}");
    }
}

async fn walk(fixture: &Fixture, who: Auth<'_>) -> (Vec<Value>, usize) {
    let mut items = Vec::new();
    let mut token: Option<String> = None;
    let mut seen = Vec::new();
    for hops in 1..=20 {
        let query = match &token {
            Some(token) => format!("{FINDINGS}?continuationToken={token}"),
            None => FINDINGS.to_owned(),
        };
        let who = match &who {
            Auth::Basic(user, password) => Auth::Basic(user, password),
            Auth::Admin => Auth::Admin,
            Auth::None => Auth::None,
        };
        let page = ok(fixture, &query, who).await;
        items.extend(page["items"].as_array().unwrap().clone());
        match page["continuationToken"].as_str() {
            Some(next) => {
                assert!(!seen.contains(&next.to_owned()), "the token repeats");
                seen.push(next.to_owned());
                token = Some(next.to_owned());
            }
            None => return (items, hops),
        }
    }
    panic!("the walk does not end");
}

#[tokio::test]
async fn a_capped_scan_hands_out_a_sealed_token_that_moves_forward() {
    let world = world().await;
    let fixture = &world.fixture;
    two_users(fixture).await;
    fixture.state.set_examine_cap(1);

    let (nothing, hops) = walk(fixture, user("hidden")).await;
    assert!(nothing.is_empty());
    assert!(hops >= 3, "{hops}");
    let (deep, _) = walk(fixture, user("deep")).await;
    fixture
        .state
        .set_examine_cap(astrum_api::routes::findings::EXAMINE_CAP);
    let whole = ok(fixture, FINDINGS, user("deep")).await;
    assert_eq!(deep, whole["items"].as_array().unwrap().clone());
    assert_eq!(deep.len(), 2);
}

#[tokio::test]
async fn a_repository_without_visible_components_is_not_named_when_the_count_is_cut() {
    let world = world().await;
    let fixture = &world.fixture;
    two_users(fixture).await;
    fixture.state.set_selector_row_cap(1);
    let summary = ok(fixture, SUMMARY, user("hidden")).await;
    assert_eq!(summary["repositories"], json!([]));
    assert_eq!(summary["totals"]["truncated"], false);
}

#[tokio::test]
async fn anonymous_access_shows_only_what_the_anonymous_role_may_browse() {
    let world = world().await;
    let fixture = &world.fixture;
    let paths = [
        component_path("files", world.alpha),
        FINDINGS.to_owned(),
        format!("{ADVISORIES}/GHSA-aaaa-0001"),
        SUMMARY.to_owned(),
    ];
    for path in &paths {
        let reply = ask(fixture, path, Auth::None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{path}");
    }
    role(
        fixture,
        "public-files",
        &["as-repository-view-raw-files-browse"],
    )
    .await;
    run(
        fixture,
        "UPDATE security_user_role SET roles = '[\"public-files\"]' WHERE user_id = 'anonymous'",
    )
    .await;
    anonymous(fixture, true).await;
    for path in &paths {
        let reply = ask(fixture, path, Auth::None).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    }
    let page = ok(fixture, FINDINGS, Auth::None).await;
    assert!(
        advisories_of(&page)
            .iter()
            .all(|(id, _)| [world.alpha, world.beta, world.gamma].contains(id))
    );
    let advisory = ok(fixture, &format!("{ADVISORIES}/GHSA-aaaa-0001"), Auth::None).await;
    assert_eq!(advisory["componentCount"], 1);
    let summary = ok(fixture, SUMMARY, Auth::None).await;
    assert_eq!(summary["repositories"].as_array().unwrap().len(), 1);
    let hidden = ask(fixture, &component_path("other", world.delta), Auth::None).await;
    let absent = ask(fixture, &component_path("other", 987_654), Auth::None).await;
    assert_eq!((hidden.status, &hidden.body), (absent.status, &absent.body));
}

#[tokio::test]
async fn a_component_without_assets_is_in_no_answer() {
    let world = world().await;
    let fixture = &world.fixture;
    let hollow = component(fixture, "files", "hollow", &[]).await;
    finding(fixture, "files", hollow, "GHSA-bbbb-0002", None).await;
    let reply = ask(fixture, &component_path("files", hollow), Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let page = ok(
        fixture,
        &format!("{FINDINGS}?advisory=GHSA-bbbb-0002"),
        Auth::Admin,
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert!(advisories_of(&page).iter().all(|(id, _)| *id != hollow));
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-bbbb-0002"),
        Auth::Admin,
    )
    .await;
    assert_eq!(advisory["componentCount"], 2);
    let summary = ok(fixture, SUMMARY, Auth::Admin).await;
    assert_eq!(summary["totals"]["componentsScanned"], 5);
    assert_eq!(summary["totals"]["high"], 2);
}

#[tokio::test]
async fn a_verdict_of_another_revision_or_policy_does_not_count_as_blocked() {
    let world = world().await;
    let fixture = &world.fixture;
    let blocked = format!("{FINDINGS}?blocked=true&repository=files");
    let count = |page: &Value| page["items"].as_array().unwrap().len();
    let before = ok(fixture, &blocked, Auth::Admin).await;
    assert_eq!(count(&before), 4);
    assert!(
        before["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["blocked"] == true)
    );

    run(
        fixture,
        "UPDATE policy_verdict SET policy_revision = policy_revision + 1",
    )
    .await;
    let stale = ok(fixture, &blocked, Auth::Admin).await;
    assert_eq!(count(&stale), 0);
    let open = ok(
        fixture,
        &format!("{FINDINGS}?blocked=false&repository=files"),
        Auth::Admin,
    )
    .await;
    assert_eq!(count(&open), 4);
    assert!(
        open["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["blocked"] == false)
    );

    run(
        fixture,
        "UPDATE policy_verdict SET policy_revision = policy_revision - 1, policy = 'lax'",
    )
    .await;
    assert_eq!(count(&ok(fixture, &blocked, Auth::Admin).await), 0);
    run(fixture, "UPDATE policy_verdict SET policy = 'strict'").await;
    assert_eq!(count(&ok(fixture, &blocked, Auth::Admin).await), 4);
}

#[tokio::test]
async fn a_hide_everything_selector_costs_few_statements_and_the_chain_of_tokens_ends() {
    let base = ready().await;
    repository(&base, "files", None).await;
    selector(&base, "nothing", "path =^ \"/nothing/\"").await;
    selector_privilege(
        &base,
        "files-nothing",
        "nothing",
        "raw",
        "files",
        &["BROWSE"],
    )
    .await;
    user_with(&base, "hidden", &["files-nothing"]).await;
    let (fixture, executes) = counting(base).await;
    advisory(&fixture, "ADV-1", &[], "high", "7.0", false).await;
    run(
        &fixture,
        "INSERT INTO component (repository, format, name, version) \
           SELECT 'files', 'raw', 'h' || g, '1' FROM generate_series(1, 1200) g; \
         INSERT INTO asset (repository, format, path, component_id, blob_store, blob_id, size, \
           content_type, sha1, sha256, sha512, md5) \
           SELECT 'files', 'raw', '/h/' || id, id, 'default', gen_random_uuid(), 1, 't', '', '', '', '' \
           FROM component; \
         INSERT INTO sca_inventory (key, source, extractor) VALUES ('pkg:shared', 'metadata', 1); \
         INSERT INTO sca_package (ecosystem, name, version, purl, matched_at) \
           VALUES ('npm', 'p', '1', 'pkg:npm/p@1', now()); \
         INSERT INTO sca_inventory_package (inventory_id, package_id) \
           SELECT (SELECT id FROM sca_inventory), id FROM sca_package; \
         INSERT INTO sca_match (package_id, advisory_id) SELECT id, 'ADV-1' FROM sca_package; \
         INSERT INTO component_scan (component_id, repository, inventory_id, state) \
           SELECT id, 'files', (SELECT id FROM sca_inventory), 'scanned' FROM component",
    )
    .await;

    fixture.state.set_examine_cap(1000);
    let before = executes.load(Ordering::Relaxed);
    let page = ok(&fixture, &format!("{FINDINGS}?limit=50"), user("hidden")).await;
    let spent = executes.load(Ordering::Relaxed) - before;
    assert!(spent <= 1000 / 500 + 3, "{spent} statements");
    assert_eq!(page["items"], json!([]));
    let token = page["continuationToken"].as_str().unwrap();
    let sealed = String::from_utf8(URL_SAFE_NO_PAD.decode(token).unwrap()).unwrap();
    assert!(sealed.starts_with("astrum:v1:"));

    fixture.state.set_examine_cap(300);
    let (items, hops) = walk(&fixture, user("hidden")).await;
    assert!(items.is_empty());
    assert_eq!(hops, 4);
}

#[tokio::test]
async fn an_advisory_says_when_the_cap_cut_its_list_of_components() {
    let world = world().await;
    let fixture = &world.fixture;
    two_users(fixture).await;
    let path = format!("{ADVISORIES}/GHSA-bbbb-0002");
    let whole = ok(fixture, &path, user("deep")).await;
    assert_eq!(whole["componentsTruncated"], false);
    fixture.state.set_examine_cap(1);
    let cut = ok(fixture, &path, user("deep")).await;
    assert_eq!(cut["componentsTruncated"], true);
    assert_eq!(cut["components"].as_array().unwrap().len(), 1);
    assert_eq!(cut["componentCount"], 1);
}

#[tokio::test]
async fn an_advisory_cuts_long_event_lists_and_drops_unsafe_urls() {
    let world = world().await;
    let fixture = &world.fixture;
    run(
        fixture,
        "INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges) \
           VALUES ('GHSA-aaaa-0001', 'npm', 'pkg', jsonb_build_array(jsonb_build_object('type', 'SEMVER', \
             'events', (SELECT jsonb_agg(jsonb_build_object('introduced', g::text)) FROM generate_series(1, 150) g)))); \
         UPDATE advisory SET document = jsonb_build_object('references', jsonb_build_array( \
           jsonb_build_object('type', 'WEB', 'url', 'https://example.org/a%20b'), \
           jsonb_build_object('type', 'WEB', 'url', 'https://example.org/' || chr(8238) || 'x'), \
           jsonb_build_object('type', 'WEB', 'url', 'https://example.org/' || chr(7) || 'x'), \
           jsonb_build_object('type', 'WEB', 'url', 'https://example.org/' || chr(8203) || 'x'), \
           jsonb_build_object('type', 'WEB', 'url', 'https://ex' || chr(228) || 'mple.org/'))) \
         WHERE id = 'GHSA-aaaa-0001'",
    )
    .await;
    let advisory = ok(
        fixture,
        &format!("{ADVISORIES}/GHSA-aaaa-0001"),
        Auth::Admin,
    )
    .await;
    let events = advisory["affected"][0]["ranges"][0]["events"]
        .as_array()
        .unwrap();
    assert_eq!(events.len(), 100);
    assert_eq!(advisory["truncated"], true);
    assert_eq!(
        advisory["references"],
        json!([{"type": "WEB", "url": "https://example.org/a%20b"}])
    );
}

#[tokio::test]
async fn advisory_ids_and_fixed_versions_are_cleaned_before_they_are_shown() {
    let fixture = ready().await;
    repository(&fixture, "files", None).await;
    advisory(&fixture, "GHSA-x\u{202e}y", &[], "high", "7.0", false).await;
    let id = component(&fixture, "files", "dirty", &["/d/one"]).await;
    finding(&fixture, "files", id, "GHSA-x\u{202e}y", Some("1.0\u{7}1")).await;
    let page = ok(&fixture, FINDINGS, Auth::Admin).await;
    assert_eq!(page["items"][0]["advisory"], "GHSA-xy");
    assert_eq!(page["items"][0]["fixed"], "1.01");
    let security = ok(&fixture, &component_path("files", id), Auth::Admin).await;
    assert_eq!(security["vulnerabilities"][0]["id"], "GHSA-xy");
    assert_eq!(security["vulnerabilities"][0]["fixed"], "1.01");
}
