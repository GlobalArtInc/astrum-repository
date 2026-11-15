mod support;

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use astrum_api::gate::PolicyGate;
use astrum_content::Gate;
use axum::Router;
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, TimeDelta, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const POLICIES: &str = "/service/rest/v1/security/policies";
const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";
const REPOSITORIES: &str = "/service/rest/v1/repositories";
const TINY_PATH: &str = "/tiny/-/tiny-1.0.0.tgz";
const TINY: &str = "/repository/npm-proxy/tiny/-/tiny-1.0.0.tgz";
const BODY: &str = "upstream tarball";

async fn upstream() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new().fallback(|request: Request| async move {
        match request.uri().path() {
            "/tiny/-/tiny-1.0.0.tgz" | "/@globalart/ui/-/ui-1.0.0.tgz" => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/octet-stream")],
                BODY.as_bytes().to_vec(),
            )
                .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    });
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
}

async fn ready() -> (Fixture, SocketAddr) {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_npm::Npm::default()));
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
    let address = upstream().await;
    (fixture, address)
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    fixture.call(method, path, Auth::Admin, body).await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    admin(fixture, Method::GET, path, None).await
}

async fn created(reply: Reply) {
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn policy(fixture: &Fixture, cooldown: Value) {
    created(
        admin(
            fixture,
            Method::POST,
            POLICIES,
            Some(json!({"name": "fresh", "cooldown": cooldown})),
        )
        .await,
    )
    .await;
}

async fn block(fixture: &Fixture) {
    policy(
        fixture,
        json!({"mode": "block", "minAgeDays": 7, "exclude": ["@globalart/*"]}),
    )
    .await;
}

async fn proxy(fixture: &Fixture, name: &str, remote: SocketAddr, policy: Option<&str>) {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": format!("http://{remote}/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {"blocked": false, "autoBlock": false},
        "npm": {"removeQuarantined": false, "removeNonCataloged": false},
    });
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
    }
    created(
        admin(
            fixture,
            Method::POST,
            &format!("{REPOSITORIES}/npm/proxy"),
            Some(body),
        )
        .await,
    )
    .await;
}

async fn hosted(fixture: &Fixture, name: &str, policy: &str) {
    created(
        admin(
            fixture,
            Method::POST,
            &format!("{REPOSITORIES}/npm/hosted"),
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
                "policy": {"name": policy},
            })),
        )
        .await,
    )
    .await;
    let data = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x03package lib 1.0.0".to_vec();
    let reply = admin(
        fixture,
        Method::PUT,
        &format!("/repository/{name}/lib"),
        Some(json!({
            "_id": "lib", "name": "lib", "dist-tags": {"latest": "1.0.0"},
            "versions": {"1.0.0": {"name": "lib", "version": "1.0.0",
                "dist": {"tarball": "http://client.example/lib/-/lib-1.0.0.tgz"}}},
            "_attachments": {"lib-1.0.0.tgz": {"content_type": "application/octet-stream",
                "data": STANDARD.encode(&data), "length": data.len()}}
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

async fn group(fixture: &Fixture, members: &[&str]) {
    created(
        admin(
            fixture,
            Method::POST,
            &format!("{REPOSITORIES}/npm/group"),
            Some(json!({
                "name": "npm-group",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": members},
            })),
        )
        .await,
    )
    .await;
}

async fn except(fixture: &Fixture, namespace: Option<&str>, name: &str, version: &str) {
    created(
        admin(
            fixture,
            Method::POST,
            EXCEPTIONS,
            Some(json!({"rule": "cooldown", "namespace": namespace, "name": name, "version": version, "reason": "audited"})),
        )
        .await,
    )
    .await;
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

async fn published(fixture: &Fixture, repository: &str, path: &str, at: Option<DateTime<Utc>>) {
    let changed = fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE asset SET upstream_published_at = $1 WHERE repository = $2 AND path = $3",
            [
                at.map(|at| at.fixed_offset()).into(),
                repository.into(),
                path.into(),
            ],
        ))
        .await
        .unwrap();
    assert_eq!(changed.rows_affected(), 1, "{repository} {path}");
}

fn days_ago(days: i64) -> DateTime<Utc> {
    Utc::now() - TimeDelta::days(days)
}

async fn warm(fixture: &Fixture, path: &str) {
    let reply = get(fixture, path).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.text(), BODY);
}

async fn waits(fixture: &Fixture) -> Vec<(String, bool, i32)> {
    fixture
        .state
        .db()
        .conn()
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT repository, blocked, refusals FROM cooldown_wait ORDER BY repository"
                .to_owned(),
        ))
        .await
        .unwrap()
        .iter()
        .map(|row| {
            (
                row.try_get("", "repository").unwrap(),
                row.try_get("", "blocked").unwrap(),
                row.try_get("", "refusals").unwrap(),
            )
        })
        .collect()
}

fn reason(reply: &Reply) -> String {
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    assert!(!reply.text().contains(BODY));
    reply.json()["error"].as_str().unwrap().to_owned()
}

fn day(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d").to_string()
}

#[tokio::test]
async fn a_package_inside_the_window_is_refused_with_its_dates_and_served_once_it_is_old_enough() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    warm(&fixture, TINY).await;

    let at = days_ago(2);
    published(&fixture, "npm-proxy", TINY_PATH, Some(at)).await;
    let refused = get(&fixture, TINY).await;
    assert_eq!(
        reason(&refused),
        format!(
            "cooldown: tiny 1.0.0 was published {}; available from {}",
            day(at),
            day(at + TimeDelta::days(7))
        )
    );
    assert_eq!(
        admin(&fixture, Method::HEAD, TINY, None).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        waits(&fixture).await,
        vec![("npm-proxy".to_owned(), true, 1)]
    );

    published(&fixture, "npm-proxy", TINY_PATH, Some(days_ago(8))).await;
    warm(&fixture, TINY).await;
}

#[tokio::test]
async fn report_mode_serves_the_package_and_records_what_it_would_have_held_back() {
    let (fixture, address) = ready().await;
    policy(&fixture, json!({"mode": "report", "minAgeDays": 7})).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    warm(&fixture, TINY).await;
    published(&fixture, "npm-proxy", TINY_PATH, Some(days_ago(1))).await;

    warm(&fixture, TINY).await;

    assert_eq!(
        waits(&fixture).await,
        vec![("npm-proxy".to_owned(), false, 1)]
    );
}

#[tokio::test]
async fn an_exception_lifts_the_refusal_of_its_version_only() {
    let (fixture, address) = ready().await;
    policy(&fixture, json!({"mode": "block", "minAgeDays": 7})).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    let scoped = "/repository/npm-proxy/@globalart/ui/-/ui-1.0.0.tgz";
    warm(&fixture, TINY).await;
    warm(&fixture, scoped).await;
    published(&fixture, "npm-proxy", TINY_PATH, Some(days_ago(1))).await;
    published(
        &fixture,
        "npm-proxy",
        "/@globalart/ui/-/ui-1.0.0.tgz",
        Some(days_ago(1)),
    )
    .await;
    except(&fixture, None, "tiny", "9.9.9").await;
    except(&fixture, Some("globalart"), "ui", "1.0.0").await;

    reason(&get(&fixture, TINY).await);
    warm(&fixture, scoped).await;

    assert_eq!(
        waits(&fixture).await,
        vec![("npm-proxy".to_owned(), true, 1)]
    );
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) AS n FROM cooldown_wait w JOIN component c ON c.id = w.component_id WHERE c.name = 'tiny'"
        )
        .await,
        1
    );
}

#[tokio::test]
async fn an_excluded_name_is_served_inside_the_window_and_others_are_not() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    let scoped = "/repository/npm-proxy/@globalart/ui/-/ui-1.0.0.tgz";
    warm(&fixture, scoped).await;
    warm(&fixture, TINY).await;
    published(
        &fixture,
        "npm-proxy",
        "/@globalart/ui/-/ui-1.0.0.tgz",
        Some(days_ago(1)),
    )
    .await;
    published(&fixture, "npm-proxy", TINY_PATH, Some(days_ago(1))).await;

    warm(&fixture, scoped).await;
    reason(&get(&fixture, TINY).await);
    assert_eq!(
        waits(&fixture).await,
        vec![("npm-proxy".to_owned(), true, 1)]
    );
    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) AS n FROM cooldown_wait w JOIN component c ON c.id = w.component_id WHERE c.name = 'ui'"
        )
        .await,
        0
    );
}

#[tokio::test]
async fn a_group_applies_the_policy_of_the_member_that_holds_the_package_and_asks_no_other() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-a", address, Some("fresh")).await;
    proxy(&fixture, "npm-b", address, None).await;
    group(&fixture, &["npm-a", "npm-b"]).await;
    let through = "/repository/npm-group/tiny/-/tiny-1.0.0.tgz";
    warm(&fixture, through).await;
    published(&fixture, "npm-a", TINY_PATH, Some(days_ago(1))).await;

    reason(&get(&fixture, through).await);

    assert_eq!(
        number(
            &fixture,
            "SELECT count(*) AS n FROM asset WHERE repository = 'npm-b'"
        )
        .await,
        0
    );
    assert_eq!(waits(&fixture).await, vec![("npm-a".to_owned(), true, 1)]);
}

#[tokio::test]
async fn a_hosted_repository_with_the_policy_is_never_cooled() {
    let (fixture, _) = ready().await;
    block(&fixture).await;
    hosted(&fixture, "npm-hosted", "fresh").await;
    let path = "/repository/npm-hosted/lib/-/lib-1.0.0.tgz";
    assert_eq!(get(&fixture, path).await.status, StatusCode::OK);
    published(
        &fixture,
        "npm-hosted",
        "/lib/-/lib-1.0.0.tgz",
        Some(days_ago(1)),
    )
    .await;

    let reply = get(&fixture, path).await;

    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert!(waits(&fixture).await.is_empty());
}

#[tokio::test]
async fn an_asset_without_a_publish_date_is_served_and_leaves_no_wait() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    warm(&fixture, TINY).await;

    warm(&fixture, TINY).await;

    assert!(waits(&fixture).await.is_empty());
}

#[tokio::test]
async fn ten_refusals_write_one_wait_and_the_next_hour_writes_again() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    warm(&fixture, TINY).await;
    published(&fixture, "npm-proxy", TINY_PATH, Some(days_ago(1))).await;
    let base = Instant::now();
    let offset = Arc::new(AtomicU64::new(0));
    let seen = offset.clone();
    let gate = PolicyGate::new(
        fixture.state.policy_cache().clone(),
        fixture.state.gate_metrics().clone(),
    )
    .with_clock(move || base + Duration::from_secs(seen.load(Ordering::SeqCst)));
    fixture.state.content().register_gate(Arc::new(gate));

    for _ in 0..10 {
        reason(&get(&fixture, TINY).await);
    }
    assert_eq!(
        waits(&fixture).await,
        vec![("npm-proxy".to_owned(), true, 1)]
    );

    offset.store(3599, Ordering::SeqCst);
    reason(&get(&fixture, TINY).await);
    assert_eq!(waits(&fixture).await[0].2, 1);

    offset.store(3601, Ordering::SeqCst);
    for _ in 0..3 {
        reason(&get(&fixture, TINY).await);
    }
    assert_eq!(waits(&fixture).await[0].2, 2);
}

#[tokio::test]
async fn a_wait_that_cannot_be_written_never_changes_the_answer() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    warm(&fixture, TINY).await;
    let at = days_ago(1);
    published(&fixture, "npm-proxy", TINY_PATH, Some(at)).await;
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared("DROP TABLE cooldown_wait")
        .await
        .unwrap();

    let refused = get(&fixture, TINY).await;

    assert!(
        reason(&refused).starts_with("cooldown: tiny 1.0.0 was published"),
        "{}",
        refused.text()
    );
}

#[tokio::test]
async fn the_gate_offers_a_cooldown_to_proxies_with_an_active_block_only() {
    let (fixture, address) = ready().await;
    block(&fixture).await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    proxy(&fixture, "npm-plain", address, None).await;
    hosted(&fixture, "npm-hosted", "fresh").await;
    except(&fixture, None, "tiny", "1.0.0").await;
    let content = fixture.state.content();
    let gate = PolicyGate::new(
        fixture.state.policy_cache().clone(),
        fixture.state.gate_metrics().clone(),
    );
    let offered = |name: &'static str| {
        let content = content.clone();
        let gate = &gate;
        async move {
            let repository = content.repository(name).await.unwrap().unwrap();
            let ctx = content.context(
                repository,
                Arc::new(astrum_security::SecurityModel::default()),
                None,
                "http://h",
                "127.0.0.1",
            );
            gate.cooldown(&ctx).await.unwrap()
        }
    };

    let before = Utc::now();
    let cooldown = offered("npm-proxy").await.unwrap();
    let after = Utc::now();

    assert!(cooldown.block);
    assert_eq!(cooldown.min_age, TimeDelta::days(7));
    assert!(cooldown.cutoff >= before - TimeDelta::days(7));
    assert!(cooldown.cutoff <= after - TimeDelta::days(7));
    assert_eq!(cooldown.exclude, vec!["@globalart/*".to_owned()]);
    assert_eq!(cooldown.excepted.len(), 1);
    assert!(offered("npm-plain").await.is_none());
    assert!(offered("npm-hosted").await.is_none());
}
