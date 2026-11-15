mod accounts;
mod support;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use accounts::{audit, create_role, create_user};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const REMOTES: &str = "/service/rest/v1/replication/remotes";
const TOKEN: &str = "check-secret-token-0001";
const INSTANCE: &str = "0b8e6f0a-0000-4000-8000-000000000042";
const UNKNOWN: &str = "00000000-0000-0000-0000-000000000001";

#[derive(Clone)]
struct Fake {
    answer: Arc<Mutex<(u16, String)>>,
    authorization: Arc<Mutex<Vec<String>>>,
}

async fn capabilities(State(fake): State<Fake>, headers: HeaderMap) -> Response {
    let sent = headers
        .get(header::AUTHORIZATION)
        .map(|value| value.to_str().unwrap().to_owned())
        .unwrap_or_default();
    fake.authorization.lock().unwrap().push(sent);
    let (status, body) = fake.answer.lock().unwrap().clone();
    (
        StatusCode::from_u16(status).unwrap(),
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}

fn good() -> String {
    json!({"protocol": 1, "instanceId": INSTANCE, "version": "3.0.4", "formats": ["raw"]})
        .to_string()
}

async fn receiver() -> (SocketAddr, Fake) {
    let fake = Fake {
        answer: Arc::new(Mutex::new((200, good()))),
        authorization: Arc::default(),
    };
    let app = Router::new()
        .route(
            "/service/rest/v1/replication/capabilities",
            get(capabilities),
        )
        .with_state(fake.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, fake)
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    let reply = fixture.call(method, path, Auth::Admin, body).await;
    assert!(!reply.text().contains(TOKEN), "a reply carries the token");
    reply
}

async fn ready() -> (Fixture, String, Fake) {
    let fixture = Fixture::new().await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: Vec::new(),
        },
    )
    .await
    .unwrap();
    let (address, fake) = receiver().await;
    let created = admin(
        &fixture,
        Method::POST,
        REMOTES,
        Some(json!({"name": "Mirror", "url": format!("http://{address}"), "token": TOKEN, "allowPrivateNetwork": true})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    (fixture, id, fake)
}

fn check_path(id: &str) -> String {
    format!("{REMOTES}/{id}/check")
}

#[tokio::test]
async fn a_good_remote_is_recorded_with_its_instance_and_version() {
    let (fixture, id, fake) = ready().await;
    let reply = admin(&fixture, Method::POST, &check_path(&id), None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!({"ok": true, "protocol": 1, "instanceId": INSTANCE, "version": "3.0.4", "formats": ["raw"]})
    );
    let stored = admin(&fixture, Method::GET, &format!("{REMOTES}/{id}"), None)
        .await
        .json();
    assert_eq!(stored["remoteInstanceId"], INSTANCE);
    assert_eq!(stored["remoteVersion"], "3.0.4");
    assert!(stored["checkedAt"].is_string());
    assert_eq!(
        *fake.authorization.lock().unwrap(),
        [format!("Bearer {TOKEN}")]
    );
    let events = audit(&fixture, "replication").await;
    let checked: Vec<_> = events
        .iter()
        .filter(|event| event.0 == "remote.checked")
        .collect();
    assert_eq!(checked.len(), 1);
    assert_eq!(checked[0].3["ok"], true);
    assert!(!checked[0].3.to_string().contains(TOKEN));
}

#[tokio::test]
async fn a_failing_remote_answers_ok_false_with_a_kind_and_still_records_the_time() {
    let (fixture, id, fake) = ready().await;
    for (status, body, kind) in [
        (
            401,
            json!({"code": "unauthorized", "message": "bad token"}).to_string(),
            "permanent",
        ),
        (503, "down".to_owned(), "retryable"),
        (
            200,
            json!({"protocol": 2, "instanceId": INSTANCE, "version": "9", "formats": []})
                .to_string(),
            "refused",
        ),
    ] {
        *fake.answer.lock().unwrap() = (status, body);
        let reply = admin(&fixture, Method::POST, &check_path(&id), None).await;
        assert_eq!(reply.status, StatusCode::OK, "{kind}");
        let answer = reply.json();
        assert_eq!(answer["ok"], false, "{kind}");
        assert_eq!(answer["error"]["kind"], kind);
        assert!(
            answer["error"]["message"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
        );
    }
    let stored = admin(&fixture, Method::GET, &format!("{REMOTES}/{id}"), None)
        .await
        .json();
    assert!(stored["checkedAt"].is_string());
    assert!(stored["remoteInstanceId"].is_null());
}

#[tokio::test]
async fn a_remote_that_cannot_be_reached_by_the_guard_is_refused_not_failed() {
    let (fixture, id, fake) = ready().await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection::default(),
    )
    .await
    .unwrap();
    let reply = admin(&fixture, Method::POST, &check_path(&id), None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json()["error"]["kind"], "refused");
    assert!(fake.authorization.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_unknown_remote_is_not_found() {
    let (fixture, _, _) = ready().await;
    for id in [UNKNOWN, "not-a-uuid"] {
        let reply = admin(&fixture, Method::POST, &check_path(id), None).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{id}");
    }
}

#[tokio::test]
async fn checking_needs_the_update_privilege() {
    let (fixture, id, _) = ready().await;
    for (user, privilege) in [
        ("reader", "as-replication-read"),
        ("creator", "as-replication-create"),
        ("receiver", "as-replication-receive"),
        ("updater", "as-replication-update"),
    ] {
        create_role(&fixture, user, &[privilege], &[]).await;
        create_user(&fixture, user, "pw-pw-pw-1", &[user]).await;
    }
    let path = check_path(&id);
    let anonymous = fixture.call(Method::POST, &path, Auth::None, None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    for user in ["reader", "creator", "receiver"] {
        let reply = fixture
            .call(Method::POST, &path, Auth::Basic(user, "pw-pw-pw-1"), None)
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{user}");
    }
    let allowed = fixture
        .call(
            Method::POST,
            &path,
            Auth::Basic("updater", "pw-pw-pw-1"),
            None,
        )
        .await;
    assert_eq!(allowed.status, StatusCode::OK, "{}", allowed.text());
    assert!(!allowed.text().contains(TOKEN));
}
