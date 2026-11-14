mod support;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode};
use axum::routing;
use axum::{Json, Router};
use chrono::{DateTime, SecondsFormat, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const PIP_JSON: &str = "application/vnd.pypi.simple.v1+json";
const POLICIES: &str = "/service/rest/v1/security/policies";
const REPOSITORIES: &str = "/service/rest/v1/repositories/pypi";
const PAGE: &str = "/repository/pypi-proxy/simple/demo/";
const WHEEL: &str = "demo-1.0-py3-none-any.whl";
const SDIST: &str = "demo-0.9.tar.gz";

#[derive(Clone)]
struct Upstream {
    page: Arc<Mutex<Value>>,
}

struct Remote {
    address: SocketAddr,
    page: Arc<Mutex<Value>>,
}

fn contents(filename: &str) -> Vec<u8> {
    format!("bytes of {filename}").into_bytes()
}

async fn upstream_page(State(upstream): State<Upstream>) -> Json<Value> {
    Json(upstream.page.lock().unwrap().clone())
}

async fn upstream_file(Path(name): Path<String>) -> Vec<u8> {
    contents(&name)
}

async fn serve_upstream() -> Remote {
    let page = Arc::new(Mutex::new(Value::Null));
    let app = Router::new()
        .route("/simple/demo/", routing::get(upstream_page))
        .route("/files/{name}", routing::get(upstream_file))
        .with_state(Upstream { page: page.clone() });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Remote { address, page }
}

async fn ready() -> (Fixture, Remote) {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_pypi::PyPi));
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    let remote = serve_upstream().await;
    (fixture, remote)
}

fn days_ago(days: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(Utc::now().timestamp() - days * 86_400, 0).unwrap()
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn entry(remote: &Remote, filename: &str, upload_time: Option<String>) -> Value {
    let mut entry = json!({
        "filename": filename,
        "url": format!("http://{}/files/{filename}", remote.address),
        "hashes": {"sha256": hex::encode(Sha256::digest(contents(filename)))},
        "size": contents(filename).len(),
    });
    if let Some(time) = upload_time {
        entry["upload-time"] = json!(time);
    }
    entry
}

fn publish(remote: &Remote, files: Vec<Value>) {
    *remote.page.lock().unwrap() = json!({
        "meta": {"api-version": "1.1"},
        "name": "demo",
        "versions": ["0.9", "1.0"],
        "files": files,
    });
}

fn names(document: &Value) -> Vec<&str> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["filename"].as_str().unwrap())
        .collect()
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    fixture.call(method, path, Auth::Admin, body).await
}

async fn get(fixture: &Fixture, path: &str, accept: Option<&str>) -> Reply {
    let headers: Vec<(&str, &str)> = accept
        .map(|accept| ("accept", accept))
        .into_iter()
        .collect();
    fixture
        .send(Method::GET, path, Auth::Admin, None, &headers)
        .await
}

fn created(reply: Reply) {
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn policy(fixture: &Fixture, name: &str, mode: &str) {
    created(
        admin(
            fixture,
            Method::POST,
            POLICIES,
            Some(json!({"name": name, "cooldown": {"mode": mode, "minAgeDays": 7}})),
        )
        .await,
    );
}

async fn proxy(fixture: &Fixture, name: &str, remote: &Remote, policy: Option<&str>) {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {
            "remoteUrl": format!("http://{}/", remote.address),
            "contentMaxAge": 1440,
            "metadataMaxAge": 0
        },
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {"blocked": false, "autoBlock": false},
    });
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
    }
    created(
        admin(
            fixture,
            Method::POST,
            &format!("{REPOSITORIES}/proxy"),
            Some(body),
        )
        .await,
    );
}

async fn group(fixture: &Fixture, name: &str, members: &[&str]) {
    created(
        admin(
            fixture,
            Method::POST,
            &format!("{REPOSITORIES}/group"),
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": members},
            })),
        )
        .await,
    );
}

async fn published_at(fixture: &Fixture, path: &str) -> Option<i64> {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT EXTRACT(EPOCH FROM upstream_published_at)::bigint AS at FROM asset WHERE repository = $1 AND path = $2",
            ["pypi-proxy".into(), path.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "at")
        .unwrap()
}

async fn fresh_wheel_and_old_sdist(fixture: &Fixture, remote: &Remote) {
    policy(fixture, "fresh", "block").await;
    proxy(fixture, "pypi-proxy", remote, Some("fresh")).await;
    publish(
        remote,
        vec![
            entry(remote, WHEEL, Some(stamp(days_ago(2)))),
            entry(remote, SDIST, Some(stamp(days_ago(30)))),
        ],
    );
}

#[tokio::test]
async fn json_and_html_pages_hide_a_fresh_wheel_and_keep_the_older_sdist() {
    let (fixture, remote) = ready().await;
    fresh_wheel_and_old_sdist(&fixture, &remote).await;

    let document = get(&fixture, PAGE, Some(PIP_JSON)).await.json();
    assert_eq!(names(&document), [SDIST]);
    assert_eq!(document["versions"], json!(["0.9"]));
    let html = get(&fixture, PAGE, None).await.text();
    assert!(html.contains(SDIST), "{html}");
    assert!(!html.contains(WHEEL), "{html}");
}

#[tokio::test]
async fn a_direct_request_for_the_hidden_wheel_is_refused_with_the_reason() {
    let (fixture, remote) = ready().await;
    fresh_wheel_and_old_sdist(&fixture, &remote).await;

    let refused = get(
        &fixture,
        "/repository/pypi-proxy/packages/demo/1.0/demo-1.0-py3-none-any.whl",
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    assert!(
        refused
            .text()
            .starts_with("cooldown: demo 1.0 was published "),
        "{}",
        refused.text()
    );
    let sdist = get(
        &fixture,
        "/repository/pypi-proxy/packages/demo/0.9/demo-0.9.tar.gz",
        None,
    )
    .await;
    assert_eq!(sdist.status, StatusCode::OK, "{}", sdist.text());
}

#[tokio::test]
async fn a_group_page_does_not_show_the_hidden_file_of_its_member() {
    let (fixture, remote) = ready().await;
    fresh_wheel_and_old_sdist(&fixture, &remote).await;
    group(&fixture, "pypi-group", &["pypi-proxy"]).await;

    let document = get(
        &fixture,
        "/repository/pypi-group/simple/demo/",
        Some(PIP_JSON),
    )
    .await
    .json();
    assert_eq!(names(&document), [SDIST]);
}

#[tokio::test]
async fn a_page_without_or_with_an_unparseable_upload_time_lists_the_file() {
    let (fixture, remote) = ready().await;
    policy(&fixture, "fresh", "block").await;
    proxy(&fixture, "pypi-proxy", &remote, Some("fresh")).await;
    publish(
        &remote,
        vec![
            entry(&remote, WHEEL, None),
            entry(&remote, SDIST, Some("not a date".to_owned())),
        ],
    );

    let document = get(&fixture, PAGE, Some(PIP_JSON)).await.json();
    assert_eq!(names(&document), [WHEEL, SDIST]);
}

#[tokio::test]
async fn report_mode_and_a_repository_without_a_policy_hide_nothing() {
    let (fixture, remote) = ready().await;
    policy(&fixture, "watch", "report").await;
    proxy(&fixture, "pypi-report", &remote, Some("watch")).await;
    proxy(&fixture, "pypi-plain", &remote, None).await;
    publish(
        &remote,
        vec![
            entry(&remote, WHEEL, Some(stamp(days_ago(2)))),
            entry(&remote, SDIST, Some(stamp(days_ago(30)))),
        ],
    );

    for repository in ["pypi-report", "pypi-plain"] {
        let document = get(
            &fixture,
            &format!("/repository/{repository}/simple/demo/"),
            Some(PIP_JSON),
        )
        .await
        .json();
        assert_eq!(names(&document), [WHEEL, SDIST], "{repository}");
    }
    let wheel = get(
        &fixture,
        "/repository/pypi-report/packages/demo/1.0/demo-1.0-py3-none-any.whl",
        None,
    )
    .await;
    assert_eq!(wheel.status, StatusCode::OK, "{}", wheel.text());
}

#[tokio::test]
async fn the_upload_time_becomes_the_publish_date_of_the_cached_file_and_an_out_of_range_one_is_unknown()
 {
    let (fixture, remote) = ready().await;
    policy(&fixture, "fresh", "block").await;
    proxy(&fixture, "pypi-proxy", &remote, Some("fresh")).await;
    let old = days_ago(30);
    publish(
        &remote,
        vec![
            entry(&remote, SDIST, Some(stamp(old))),
            entry(
                &remote,
                "demo-0.8.tar.gz",
                Some("1990-01-01T00:00:00Z".to_owned()),
            ),
        ],
    );

    for (filename, version) in [(SDIST, "0.9"), ("demo-0.8.tar.gz", "0.8")] {
        let reply = get(
            &fixture,
            &format!("/repository/pypi-proxy/packages/demo/{version}/{filename}"),
            None,
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    }
    assert_eq!(
        published_at(&fixture, "/packages/demo/0.9/demo-0.9.tar.gz").await,
        Some(old.timestamp())
    );
    assert_eq!(
        published_at(&fixture, "/packages/demo/0.8/demo-0.8.tar.gz").await,
        None
    );
}

#[tokio::test]
async fn a_file_is_listed_again_once_its_upload_time_leaves_the_window() {
    let (fixture, remote) = ready().await;
    fresh_wheel_and_old_sdist(&fixture, &remote).await;
    let document = get(&fixture, PAGE, Some(PIP_JSON)).await.json();
    assert_eq!(names(&document), [SDIST]);

    publish(
        &remote,
        vec![
            entry(&remote, WHEEL, Some(stamp(days_ago(30)))),
            entry(&remote, SDIST, Some(stamp(days_ago(30)))),
        ],
    );
    let document = get(&fixture, PAGE, Some(PIP_JSON)).await.json();
    assert_eq!(names(&document), [WHEEL, SDIST]);
}
