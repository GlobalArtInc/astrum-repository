mod accounts;
mod support;

use std::net::SocketAddr;
use std::time::Duration;

use accounts::{audit, create_role, create_user, error_ids};
use astrum_api::{AppState, revisions, router};
use astrum_security::SecretBox;
use astrum_security::ip_allow::IP_ALLOW_KEY;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const IP_ALLOW: &str = "/service/rest/v1/security/ip-allow";
const REPOSITORIES: &str = "/service/rest/v1/repositories";
const OFFICE: &str = "10.1.1.1";
const OUTSIDE: &str = "203.0.113.5";

async fn put(fixture: &mut Fixture, peer: Option<&str>, body: Value) -> Reply {
    fixture.peer = peer.map(|peer| peer.parse().unwrap());
    fixture
        .call(Method::PUT, IP_ALLOW, Auth::Admin, Some(body))
        .await
}

async fn get(fixture: &mut Fixture, peer: &str, headers: &[(&str, &str)]) -> Reply {
    fixture.peer = Some(peer.parse().unwrap());
    fixture
        .send(Method::GET, IP_ALLOW, Auth::Admin, None, headers)
        .await
}

fn lists(management: &[&str], content: &[&str], agents: &[&str]) -> Value {
    json!({"management": management, "content": content, "agents": agents})
}

async fn on(state: &AppState, peer: &str, fixture: &Fixture, path: &str) -> StatusCode {
    let encoded = STANDARD.encode(format!("admin:{}", fixture.admin_password));
    let mut request = Request::builder()
        .uri(path)
        .header(header::AUTHORIZATION, format!("Basic {encoded}"))
        .body(Body::empty())
        .unwrap();
    let peer: std::net::IpAddr = peer.parse().unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(peer, 40000)));
    let response = router(state.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    response.into_body().collect().await.unwrap();
    status
}

#[tokio::test]
async fn a_list_is_saved_read_back_and_audited_with_what_the_server_sees() {
    let mut fixture = Fixture::new().await;
    let empty = get(&mut fixture, OFFICE, &[]).await;
    assert_eq!(empty.status, StatusCode::OK, "{}", empty.text());
    assert_eq!(
        empty.json(),
        json!({
            "management": [], "content": [], "agents": [],
            "yourAddress": OFFICE, "peer": OFFICE,
            "trustedProxies": [], "forwardedIgnored": false,
        })
    );

    let forged = [("x-forwarded-for", "198.51.100.1")];
    let ignored = get(&mut fixture, OFFICE, &forged).await.json();
    assert_eq!(ignored["forwardedIgnored"], true);
    assert_eq!(ignored["yourAddress"], OFFICE);
    fixture.trust(&["10.0.0.0/8"]);
    let believed = get(&mut fixture, OFFICE, &forged).await.json();
    assert_eq!(believed["forwardedIgnored"], false);
    assert_eq!(believed["yourAddress"], "198.51.100.1");
    assert_eq!(believed["peer"], OFFICE);
    assert_eq!(believed["trustedProxies"], json!(["10.0.0.0/8"]));

    let saved = put(
        &mut fixture,
        Some(OFFICE),
        lists(
            &["10.0.0.0/8", " 10.0.0.0/8", "::ffff:10.0.0.0/104"],
            &["192.0.2.7"],
            &[],
        ),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.text());
    assert_eq!(
        saved.json()["management"],
        json!(["10.0.0.0/8", "::ffff:10.0.0.0/104"])
    );
    let read = get(&mut fixture, OFFICE, &[]).await;
    assert_eq!(read.json()["content"], json!(["192.0.2.7"]));
    let events = audit(&fixture, "security.ipAllow").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "updated");
    assert_eq!(events[0].3["before"]["management"], json!([]));
    assert_eq!(events[0].3["after"]["content"], json!(["192.0.2.7"]));
}

#[tokio::test]
async fn a_value_of_the_wrong_shape_is_replaced_by_a_valid_list_and_audited_as_it_was() {
    let mut fixture = Fixture::new().await;
    let corrupt = json!({"management": "oops"});
    astrum_db::setting::put(fixture.state.db().conn(), IP_ALLOW_KEY, &corrupt)
        .await
        .unwrap();
    let saved = put(&mut fixture, Some(OFFICE), lists(&["10.0.0.0/8"], &[], &[])).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.text());
    let read = get(&mut fixture, OFFICE, &[]).await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    assert_eq!(read.json()["management"], json!(["10.0.0.0/8"]));
    let events = audit(&fixture, "security.ipAllow").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].3["before"], corrupt);
}

#[tokio::test]
async fn a_list_that_would_lock_the_caller_out_is_refused() {
    let mut fixture = Fixture::new().await;
    let refused = put(
        &mut fixture,
        Some(OFFICE),
        lists(&["192.0.2.0/24"], &[], &[]),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert_eq!(error_ids(&refused), ["management"]);
    assert!(refused.text().contains(OFFICE), "{}", refused.text());
    let unknown = put(&mut fixture, None, lists(&["192.0.2.0/24"], &[], &[])).await;
    assert_eq!(
        unknown.status,
        StatusCode::BAD_REQUEST,
        "{}",
        unknown.text()
    );
    assert_eq!(error_ids(&unknown), ["management"]);
    assert!(audit(&fixture, "security.ipAllow").await.is_empty());
    assert_eq!(
        get(&mut fixture, OFFICE, &[]).await.json()["management"],
        json!([])
    );

    let content_only = put(&mut fixture, None, lists(&[], &["192.0.2.0/24"], &[])).await;
    assert_eq!(
        content_only.status,
        StatusCode::OK,
        "{}",
        content_only.text()
    );
    let own = put(&mut fixture, Some(OFFICE), lists(&["10.0.0.0/8"], &[], &[])).await;
    assert_eq!(own.status, StatusCode::OK, "{}", own.text());
    let inside = fixture
        .call(Method::GET, REPOSITORIES, Auth::Admin, None)
        .await;
    assert_eq!(inside.status, StatusCode::OK, "the caller still gets in");
}

#[tokio::test]
async fn invalid_entries_long_lists_and_missing_fields_are_refused_per_field() {
    let mut fixture = Fixture::new().await;
    let invalid = put(
        &mut fixture,
        Some(OFFICE),
        lists(&["10.0.0.0/33"], &["gateway"], &["::ffff:10.0.0.0/64"]),
    )
    .await;
    assert_eq!(
        invalid.status,
        StatusCode::BAD_REQUEST,
        "{}",
        invalid.text()
    );
    assert_eq!(error_ids(&invalid), ["management", "content", "agents"]);

    let whole = put(
        &mut fixture,
        Some(OFFICE),
        lists(&["0.0.0.0/0"], &["::/0"], &["::ffff:0.0.0.0/96"]),
    )
    .await;
    assert_eq!(whole.status, StatusCode::BAD_REQUEST, "{}", whole.text());
    assert_eq!(error_ids(&whole), ["management", "content", "agents"]);
    assert!(whole.text().contains(
        "matches a whole address family is not allowed; clear the list to allow everybody"
    ));

    let entries: Vec<String> = (0..=256)
        .map(|index| format!("192.0.{}.{}", index / 200, index % 200))
        .collect();
    let refs: Vec<&str> = entries.iter().map(String::as_str).collect();
    let too_long = put(&mut fixture, Some(OFFICE), lists(&[], &refs, &[])).await;
    assert_eq!(
        too_long.status,
        StatusCode::BAD_REQUEST,
        "{}",
        too_long.text()
    );
    assert_eq!(error_ids(&too_long), ["content"]);
    let at_limit = put(&mut fixture, Some(OFFICE), lists(&[], &refs[..256], &[])).await;
    assert_eq!(at_limit.status, StatusCode::OK, "{}", at_limit.text());

    let missing = put(&mut fixture, Some(OFFICE), json!({"management": []})).await;
    assert_eq!(
        missing.status,
        StatusCode::BAD_REQUEST,
        "{}",
        missing.text()
    );
    assert_eq!(error_ids(&missing), ["content", "agents"]);
    assert_eq!(
        get(&mut fixture, OFFICE, &[]).await.json()["content"]
            .as_array()
            .unwrap()
            .len(),
        256
    );
}

#[tokio::test]
async fn only_an_administrator_reads_or_changes_the_lists() {
    let mut fixture = Fixture::new().await;
    create_role(&fixture, "settings-admins", &["as-settings-all"], &[]).await;
    create_user(&fixture, "keeper", "keeper password", &["settings-admins"]).await;
    for method in [Method::GET, Method::PUT] {
        let body = (method == Method::PUT).then(|| lists(&[], &[], &[]));
        let keeper = Auth::Basic("keeper", "keeper password");
        let reply = fixture
            .call(method.clone(), IP_ALLOW, keeper, body.clone())
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method}");
        let reply = fixture
            .call(method.clone(), IP_ALLOW, Auth::None, body)
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method}");
    }
    let reply = put(&mut fixture, Some(OFFICE), lists(&[], &[], &[])).await;
    assert_eq!(reply.status, StatusCode::OK);
}

#[tokio::test]
async fn another_server_follows_a_change_within_one_catch_up() {
    let mut fixture = Fixture::new().await;
    let second = AppState::load(
        fixture.test.connect().await,
        fixture.data.path().to_owned(),
        SecretBox::from_key([7; 32]),
    )
    .await
    .unwrap();
    assert_eq!(
        on(&second, OUTSIDE, &fixture, REPOSITORIES).await,
        StatusCode::OK
    );

    let saved = put(&mut fixture, Some(OFFICE), lists(&["10.0.0.0/8"], &[], &[])).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.text());
    assert_eq!(
        on(&second, OUTSIDE, &fixture, REPOSITORIES).await,
        StatusCode::OK,
        "the second server still holds the old list until it catches up"
    );
    tokio::time::timeout(Duration::from_secs(10), revisions::catch_up(&second))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        on(&second, OUTSIDE, &fixture, REPOSITORIES).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        on(&second, OFFICE, &fixture, REPOSITORIES).await,
        StatusCode::OK
    );

    astrum_security::ip_allow::reset(fixture.state.db().conn())
        .await
        .unwrap();
    revisions::catch_up(&second).await.unwrap();
    revisions::catch_up(&fixture.state).await.unwrap();
    for state in [&second, &fixture.state] {
        assert_eq!(
            on(state, OUTSIDE, &fixture, REPOSITORIES).await,
            StatusCode::OK
        );
    }
    let events = audit(&fixture, "security.ipAllow").await;
    assert_eq!(events.last().unwrap().2, "*CLI");
}
