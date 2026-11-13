mod support;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, FixedOffset, SecondsFormat, TimeDelta, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const TOOL: &str = "/repository/npm-proxy/@team/tool";
const TOOL_OLD: &str = "/repository/npm-proxy/@team/tool/-/tool-1.0.0.tgz";
const TOOL_PIN: &str = "/repository/npm-proxy/@team/tool/-/tool-2.0.0.tgz";
const TOOL_PACKUMENT: &str = "/@team/tool";
const TOOL_OLD_FILE: &str = "/@team/tool/-/tool-1.0.0.tgz";
const TOOL_PIN_FILE: &str = "/@team/tool/-/tool-2.0.0.tgz";
const ABBREVIATED: &str = "application/vnd.npm.install-v1+json";

#[derive(Clone)]
struct Remote {
    document: Arc<Mutex<Value>>,
}

impl Remote {
    fn set_time(&self, version: &str, time: String) {
        self.document.lock().unwrap()["time"][version] = json!(time);
    }

    fn time(&self, version: &str) -> String {
        self.document.lock().unwrap()["time"][version]
            .as_str()
            .unwrap()
            .to_owned()
    }
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn document(now: DateTime<Utc>) -> Value {
    json!({
        "_id": "@team/tool",
        "name": "@team/tool",
        "dist-tags": {"latest": "2.0.0", "beta": "2.0.0"},
        "versions": {
            "1.0.0": {"name": "@team/tool", "version": "1.0.0", "dist": {"shasum": "a", "tarball": "/@team/tool/-/tool-1.0.0.tgz"}},
            "2.0.0": {"name": "@team/tool", "version": "2.0.0", "dist": {"shasum": "b", "tarball": "/@team/tool/-/tool-2.0.0.tgz"}}
        },
        "time": {"1.0.0": stamp(now - TimeDelta::days(30)), "2.0.0": stamp(now - TimeDelta::days(2))}
    })
}

async fn remote_file(State(remote): State<Remote>, request: Request) -> Response {
    match request.uri().path() {
        TOOL_PACKUMENT => {
            let body = serde_json::to_vec(&*remote.document.lock().unwrap()).unwrap();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                body,
            )
                .into_response()
        }
        TOOL_OLD_FILE | TOOL_PIN_FILE => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/octet-stream")],
            "upstream tarball",
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn upstream() -> (SocketAddr, Remote) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let remote = Remote {
        document: Arc::new(Mutex::new(document(Utc::now()))),
    };
    let app = Router::new()
        .fallback(remote_file)
        .with_state(remote.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, remote)
}

async fn ready() -> (Fixture, SocketAddr, Remote) {
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
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    let (address, remote) = upstream().await;
    (fixture, address, remote)
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn created(fixture: &Fixture, path: &str, body: Value) {
    let reply = fixture
        .call(Method::POST, path, Auth::Admin, Some(body))
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn policy(fixture: &Fixture, name: &str, mode: &str) {
    created(
        fixture,
        "/service/rest/v1/security/policies",
        json!({"name": name, "cooldown": {"mode": mode, "minAgeDays": 7}}),
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
    created(fixture, "/service/rest/v1/repositories/npm/proxy", body).await;
}

async fn group(fixture: &Fixture, name: &str, members: &[&str]) {
    created(
        fixture,
        "/service/rest/v1/repositories/npm/group",
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": members},
        }),
    )
    .await;
}

async fn move_asset(fixture: &Fixture, column: &str, at: DateTime<Utc>) {
    let sql =
        format!("UPDATE asset SET {column} = $1 WHERE repository = 'npm-proxy' AND path = $2");
    let changed = fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [at.fixed_offset().into(), TOOL_PACKUMENT.into()],
        ))
        .await
        .unwrap();
    assert_eq!(changed.rows_affected(), 1, "{column}");
}

async fn stored_date(fixture: &Fixture, path: &str) -> Option<DateTime<FixedOffset>> {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT upstream_published_at FROM asset WHERE repository = 'npm-proxy' AND path = $1",
            [path.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "upstream_published_at")
        .unwrap()
}

fn day(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d").to_string()
}

#[tokio::test]
async fn hidden_versions_leave_every_document_and_the_tags_are_repointed() {
    let (fixture, address, _) = ready().await;
    policy(&fixture, "fresh", "block").await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;

    let full = get(&fixture, TOOL).await;
    assert_eq!(full.status, StatusCode::OK, "{}", full.text());
    let full = full.json();
    assert_eq!(full["dist-tags"], json!({"latest": "1.0.0"}));
    assert!(full["versions"].get("1.0.0").is_some());
    assert!(full["versions"].get("2.0.0").is_none());
    assert!(full["time"].get("2.0.0").is_none());

    let short = fixture
        .send(
            Method::GET,
            TOOL,
            Auth::Admin,
            None,
            &[("accept", ABBREVIATED)],
        )
        .await
        .json();
    assert_eq!(short["dist-tags"], json!({"latest": "1.0.0"}));
    assert!(short["versions"].get("2.0.0").is_none());

    assert_eq!(
        get(&fixture, &format!("{TOOL}/2.0.0")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, &format!("{TOOL}/latest")).await.json()["version"],
        "1.0.0"
    );
    let tags = get(
        &fixture,
        "/repository/npm-proxy/-/package/@team%2ftool/dist-tags",
    )
    .await;
    assert_eq!(tags.json(), json!({"latest": "1.0.0"}));
}

#[tokio::test]
async fn a_group_does_not_serve_a_hidden_member_version() {
    let (fixture, address, _) = ready().await;
    policy(&fixture, "fresh", "block").await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    group(&fixture, "npm-group", &["npm-proxy"]).await;

    let doc = get(&fixture, "/repository/npm-group/@team/tool")
        .await
        .json();
    assert_eq!(doc["dist-tags"], json!({"latest": "1.0.0"}));
    assert!(doc["versions"].get("2.0.0").is_none());
    assert_eq!(
        get(&fixture, "/repository/npm-group/@team/tool/2.0.0")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_pinned_tarball_of_a_hidden_version_is_refused_with_the_reason() {
    let (fixture, address, _) = ready().await;
    policy(&fixture, "fresh", "block").await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    assert_eq!(get(&fixture, TOOL).await.status, StatusCode::OK);

    let refused = get(&fixture, TOOL_PIN).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    let published = stored_date(&fixture, TOOL_PIN_FILE)
        .await
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(
        refused.json()["error"],
        format!(
            "cooldown: @team/tool 2.0.0 was published {}; available from {}",
            day(published),
            day(published + TimeDelta::days(7))
        )
    );
    assert_eq!(get(&fixture, TOOL_OLD).await.status, StatusCode::OK);
}

#[tokio::test]
async fn report_mode_and_a_repository_without_a_policy_leave_the_documents_alone() {
    let (fixture, address, _) = ready().await;
    policy(&fixture, "watch", "report").await;
    proxy(&fixture, "npm-report", address, Some("watch")).await;
    proxy(&fixture, "npm-plain", address, None).await;

    for path in [
        "/repository/npm-report/@team/tool",
        "/repository/npm-plain/@team/tool",
    ] {
        let doc = get(&fixture, path).await.json();
        assert_eq!(doc["dist-tags"]["latest"], "2.0.0", "{path}");
        assert!(doc["versions"].get("2.0.0").is_some(), "{path}");
    }
}

#[tokio::test]
async fn last_modified_and_etag_move_when_a_version_leaves_the_window() {
    let (fixture, address, remote) = ready().await;
    policy(&fixture, "fresh", "block").await;
    proxy(&fixture, "npm-proxy", address, Some("fresh")).await;
    let hidden = get(&fixture, TOOL).await;
    assert_eq!(hidden.json()["dist-tags"]["latest"], "1.0.0");

    let published = Utc::now() - TimeDelta::days(8);
    remote.set_time("2.0.0", stamp(published));
    move_asset(
        &fixture,
        "last_verified_at",
        Utc::now() - TimeDelta::days(2),
    )
    .await;
    assert_eq!(get(&fixture, TOOL).await.status, StatusCode::OK);
    move_asset(&fixture, "blob_updated_at", Utc::now() - TimeDelta::days(3)).await;

    let unlocked = get(&fixture, TOOL).await;
    assert_eq!(unlocked.json()["dist-tags"]["latest"], "2.0.0");
    assert_eq!(
        unlocked.headers[header::LAST_MODIFIED],
        (published + TimeDelta::days(7))
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string()
    );
    assert_ne!(unlocked.headers[header::ETAG], hidden.headers[header::ETAG]);
}

#[tokio::test]
async fn a_cached_tarball_keeps_its_registry_date_and_an_out_of_range_date_is_unknown() {
    let (fixture, address, remote) = ready().await;
    remote.set_time("2.0.0", "2004-12-31T23:59:59Z".to_owned());
    proxy(&fixture, "npm-proxy", address, None).await;
    assert_eq!(get(&fixture, TOOL).await.status, StatusCode::OK);
    assert_eq!(get(&fixture, TOOL_OLD).await.status, StatusCode::OK);
    assert_eq!(get(&fixture, TOOL_PIN).await.status, StatusCode::OK);

    let expected = DateTime::parse_from_rfc3339(&remote.time("1.0.0")).unwrap();
    assert_eq!(stored_date(&fixture, TOOL_OLD_FILE).await, Some(expected));
    assert_eq!(stored_date(&fixture, TOOL_PIN_FILE).await, None);
}
