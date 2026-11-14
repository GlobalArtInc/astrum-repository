mod support;

use std::net::SocketAddr;
use std::sync::Arc;

use astrum_api::STATUS;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const GATEWAY: &str = "10.0.0.1";
const FIRST: [(&str, &str); 1] = [("x-forwarded-for", "198.51.100.1")];
const SECOND: [(&str, &str); 1] = [("x-forwarded-for", "198.51.100.2")];

async fn behind_gateway() -> Fixture {
    let mut fixture = Fixture::new().await;
    fixture.trust(&["10.0.0.0/8"]);
    fixture.peer = Some(GATEWAY.parse().unwrap());
    fixture
}

async fn status_as(fixture: &Fixture, user: &str, password: &str, from: &[(&str, &str)]) -> Reply {
    fixture
        .send(Method::GET, STATUS, Auth::Basic(user, password), None, from)
        .await
}

async fn fail(fixture: &Fixture, user: &str, times: usize, from: &[(&str, &str)]) {
    for _ in 0..times {
        let reply = status_as(fixture, user, "wrong", from).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn wrong_passwords_block_the_pair_even_for_the_right_password() {
    let fixture = behind_gateway().await;
    fail(&fixture, "admin", 10, &FIRST).await;
    let blocked = status_as(&fixture, "admin", "wrong", &FIRST).await;
    assert_eq!(blocked.status, StatusCode::TOO_MANY_REQUESTS);
    let wait: u64 = blocked.headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=300).contains(&wait), "{wait}");
    assert_eq!(blocked.json()["id"], "*");

    let right = status_as(&fixture, "admin", &fixture.admin_password, &FIRST).await;
    assert_eq!(right.status, StatusCode::TOO_MANY_REQUESTS);

    fail(&fixture, "nobody", 10, &FIRST).await;
    let unknown = status_as(&fixture, "nobody", "wrong", &FIRST).await;
    assert_eq!(unknown.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(unknown.json(), blocked.json());
}

#[tokio::test]
async fn other_addresses_and_other_users_are_not_blocked() {
    let fixture = behind_gateway().await;
    fixture.add_user("puller", "puller-password-1", &[]).await;
    fail(&fixture, "admin", 10, &FIRST).await;

    let elsewhere = status_as(&fixture, "admin", &fixture.admin_password, &SECOND).await;
    assert_eq!(elsewhere.status, StatusCode::OK);
    let other_user = status_as(&fixture, "puller", "puller-password-1", &FIRST).await;
    assert_eq!(other_user.status, StatusCode::OK);
    let spelled = status_as(&fixture, " ADMIN", "wrong", &FIRST).await;
    assert_eq!(spelled.status, StatusCode::TOO_MANY_REQUESTS);

    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let event = audit
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["domain"] == "security.auth.ratelimit" && event["type"] == "blocked")
        .expect("a rate limit event");
    assert_eq!(event["attributes"]["sourceIp"], "198.51.100.1");
    assert_eq!(event["attributes"]["username"], "admin");
}

#[tokio::test]
async fn tokens_of_a_blocked_pair_still_work() {
    let fixture = behind_gateway().await;
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Admin,
            Some(json!({"name": "ci", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await
        .json();
    let token = created["token"].as_str().unwrap();
    fail(&fixture, "admin", 10, &FIRST).await;
    assert_eq!(
        status_as(&fixture, "admin", &fixture.admin_password, &FIRST)
            .await
            .status,
        StatusCode::TOO_MANY_REQUESTS
    );

    let bearer = format!("Bearer {token}");
    let by_bearer = fixture
        .send(
            Method::GET,
            STATUS,
            Auth::None,
            None,
            &[("authorization", &bearer), FIRST[0]],
        )
        .await;
    assert_eq!(by_bearer.status, StatusCode::OK);
    let as_password = status_as(&fixture, "admin", token, &FIRST).await;
    assert_eq!(as_password.status, StatusCode::OK);
}

#[tokio::test]
async fn parallel_wrong_passwords_cannot_exceed_the_allowance() {
    let fixture = behind_gateway().await;
    let replies = futures_util::future::join_all(
        (0..30).map(|_| status_as(&fixture, "admin", "wrong", &FIRST)),
    )
    .await;
    let refused = replies
        .iter()
        .filter(|reply| reply.status == StatusCode::UNAUTHORIZED)
        .count();
    let paused = replies
        .iter()
        .filter(|reply| reply.status == StatusCode::TOO_MANY_REQUESTS)
        .count();
    assert_eq!((refused, paused), (10, 20));
}

#[tokio::test]
async fn a_success_clears_the_counter() {
    let fixture = behind_gateway().await;
    fail(&fixture, "admin", 9, &FIRST).await;
    let right = status_as(&fixture, "admin", &fixture.admin_password, &FIRST).await;
    assert_eq!(right.status, StatusCode::OK);
    fail(&fixture, "admin", 9, &FIRST).await;
    let right = status_as(&fixture, "admin", &fixture.admin_password, &FIRST).await;
    assert_eq!(right.status, StatusCode::OK);
}

async fn registry_call(
    app: &axum::Router,
    method: Method,
    path: &str,
    authorization: Option<String>,
    form: Option<String>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", "198.51.100.1")
        .extension(ConnectInfo(SocketAddr::new(
            GATEWAY.parse().unwrap(),
            40000,
        )));
    if let Some(authorization) = authorization {
        request = request.header(header::AUTHORIZATION, authorization);
    }
    let body = match form {
        Some(form) => {
            request = request.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
            Body::from(form)
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, json)
}

#[tokio::test]
async fn the_registry_answers_toomanyrequests() {
    let fixture = behind_gateway().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_docker::Docker::default()));
    astrum_db::setting::put(
        fixture.state.db().conn(),
        astrum_security::model::REALMS_KEY,
        &["AstrumAuthenticatingRealm", "DockerToken"],
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_docker::routes());
    let wrong = format!("Basic {}", STANDARD.encode("admin:wrong"));
    for _ in 0..10 {
        let (status, _, _) =
            registry_call(&app, Method::GET, "/v2/token", Some(wrong.clone()), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, headers, body) =
        registry_call(&app, Method::GET, "/v2/token", Some(wrong), None).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["errors"][0]["code"], "TOOMANYREQUESTS");
    assert!(headers.contains_key(header::RETRY_AFTER));

    let form = format!(
        "grant_type=password&username=admin&password={}",
        fixture.admin_password
    );
    let (status, _, body) =
        registry_call(&app, Method::POST, "/v2/token", None, Some(form.clone())).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["errors"][0]["code"], "TOOMANYREQUESTS");

    let rest = status_as(&fixture, "admin", &fixture.admin_password, &SECOND).await;
    assert_eq!(rest.status, StatusCode::OK);
}

#[tokio::test]
async fn a_forged_forwarded_header_from_an_untrusted_peer_gets_no_fresh_counter() {
    let mut fixture = Fixture::new().await;
    fixture.trust(&["10.0.0.0/8"]);
    fixture.peer = Some("192.0.2.7".parse().unwrap());
    for round in 0..10 {
        let forged = format!("198.51.100.{round}");
        let reply = status_as(&fixture, "admin", "wrong", &[("x-forwarded-for", &forged)]).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }
    let again = status_as(
        &fixture,
        "admin",
        "wrong",
        &[("x-forwarded-for", "203.0.113.99")],
    )
    .await;
    assert_eq!(again.status, StatusCode::TOO_MANY_REQUESTS);
    let right = status_as(&fixture, "admin", &fixture.admin_password, &SECOND).await;
    assert_eq!(right.status, StatusCode::TOO_MANY_REQUESTS);

    fixture.peer = Some("192.0.2.8".parse().unwrap());
    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let event = audit
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["domain"] == "security.auth.ratelimit")
        .expect("a rate limit event");
    assert_eq!(event["attributes"]["sourceIp"], "192.0.2.7");
}

#[tokio::test]
async fn requests_without_a_known_peer_share_one_counter() {
    let fixture = Fixture::new().await;
    for round in 0..10 {
        let forged = format!("198.51.100.{round}");
        let reply = status_as(&fixture, "admin", "wrong", &[("x-forwarded-for", &forged)]).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }
    let again = status_as(&fixture, "admin", "wrong", &SECOND).await;
    assert_eq!(again.status, StatusCode::TOO_MANY_REQUESTS);
    let other_user = status_as(&fixture, "nobody", "wrong", &SECOND).await;
    assert_eq!(other_user.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn spellings_of_one_address_and_one_ipv6_prefix_share_a_counter() {
    let fixture = behind_gateway().await;
    let spellings = [
        "198.51.100.1",
        "::ffff:198.51.100.1",
        "198.51.100.1:5000",
        "[::ffff:198.51.100.1]:5000",
        "198.51.100.1",
        "::ffff:198.51.100.1",
        "198.51.100.1:6000",
        "198.51.100.1",
        "::ffff:198.51.100.1",
        "198.51.100.1",
    ];
    for spelling in spellings {
        let reply = status_as(&fixture, "admin", "wrong", &[("x-forwarded-for", spelling)]).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{spelling}");
    }
    assert_eq!(
        status_as(&fixture, "admin", "wrong", &FIRST).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );

    for round in 0..10 {
        let rotated = format!("2001:db8:1:2:{round:x}::{round:x}");
        let reply = status_as(&fixture, "admin", "wrong", &[("x-forwarded-for", &rotated)]).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{rotated}");
    }
    let same_prefix = status_as(
        &fixture,
        "admin",
        "wrong",
        &[("x-forwarded-for", "2001:db8:1:2:ffff::1")],
    )
    .await;
    assert_eq!(same_prefix.status, StatusCode::TOO_MANY_REQUESTS);
    let next_prefix = status_as(
        &fixture,
        "admin",
        &fixture.admin_password,
        &[("x-forwarded-for", "2001:db8:1:3::1")],
    )
    .await;
    assert_eq!(next_prefix.status, StatusCode::OK);
}
