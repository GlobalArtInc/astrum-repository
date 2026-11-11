mod support;

use astrum_testkit::oidc::MockProvider;
use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const TOKENS: &str = "/service/rest/internal/access-tokens";
const HOST: (&str, &str) = ("host", "astrum.test");

fn pairs(reply: &Reply) -> Vec<(String, String)> {
    reply
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| {
            let pair = value.to_str().unwrap().split(';').next().unwrap();
            let (name, value) = pair.split_once('=').unwrap();
            (name.to_owned(), value.to_owned())
        })
        .collect()
}

fn location(reply: &Reply) -> String {
    reply
        .headers
        .get(header::LOCATION)
        .unwrap_or_else(|| panic!("no Location in {} {}", reply.status, reply.text()))
        .to_str()
        .unwrap()
        .to_owned()
}

async fn setup() -> (Fixture, MockProvider) {
    let fixture = Fixture::new().await;
    let provider = MockProvider::start("astrum", Some("s3cret")).await;
    let ssrf = json!({"enabled": true, "allowedIPs": ["127.0.0.1"], "allowedDomains": []});
    let path = "/service/rest/v1/security/ssrf-protection";
    let reply = fixture
        .call(Method::PUT, path, Auth::Admin, Some(ssrf))
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let settings = json!({"issuerUrl": provider.url, "clientId": "astrum",
        "clientSecret": "s3cret", "groupsClaim": "groups"});
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/internal/ui/oauth2",
            Auth::Admin,
            Some(settings),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    (fixture, provider)
}

async fn sign_in(fixture: &Fixture) -> Reply {
    let start = fixture
        .send(Method::GET, "/oidc/login", Auth::None, None, &[HOST])
        .await;
    assert_eq!(start.status, StatusCode::FOUND, "{}", start.text());
    let state = pairs(&start)
        .into_iter()
        .find(|(name, _)| name == "astrum_oidc")
        .map(|(name, value)| format!("{name}={value}"))
        .unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .unwrap();
    let response = client.get(location(&start)).send().await.unwrap();
    let back = url::Url::parse(response.headers()["location"].to_str().unwrap()).unwrap();
    let path = format!("{}?{}", back.path(), back.query().unwrap());
    fixture
        .send(
            Method::GET,
            &path,
            Auth::None,
            None,
            &[("cookie", &state), HOST],
        )
        .await
}

fn session(callback: &Reply) -> (String, String) {
    let cookies = pairs(callback);
    let value = |wanted: &str| {
        cookies
            .iter()
            .find(|(name, value)| name == wanted && !value.is_empty())
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| panic!("no {wanted} cookie after {}", callback.status))
    };
    let csrf = value("astrum_csrf");
    (
        format!(
            "astrum_session={}; astrum_csrf={csrf}",
            value("astrum_session")
        ),
        csrf,
    )
}

async fn make_token(fixture: &Fixture, callback: &Reply) -> Reply {
    let (cookies, csrf) = session(callback);
    let body = json!({"name": "mine", "scope": "WRITE", "expiresInDays": null});
    fixture
        .send(
            Method::POST,
            TOKENS,
            Auth::None,
            Some(body),
            &[("cookie", &cookies), ("x-astrum-csrf", &csrf)],
        )
        .await
}

#[tokio::test]
async fn an_oidc_namesake_of_a_local_user_is_refused() {
    let (fixture, provider) = setup().await;
    fixture
        .add_user("bob", "local-bob-password", &["as-admin"])
        .await;
    provider.sign_in_as(json!({"sub": "ns-1", "preferred_username": "bob"}));
    let callback = sign_in(&fixture).await;
    assert_eq!(location(&callback), "/login?sso=denied");
    assert!(
        !pairs(&callback)
            .iter()
            .any(|(name, value)| name == "astrum_session" && !value.is_empty())
    );
    provider.sign_in_as(json!({"sub": "ns-2", "preferred_username": "BOB"}));
    let callback = sign_in(&fixture).await;
    assert_eq!(location(&callback), "/login?sso=denied");
}

#[tokio::test]
async fn an_oidc_user_without_a_local_namesake_signs_in_but_keeps_no_credentials() {
    let (fixture, provider) = setup().await;
    provider.sign_in_as(json!({"sub": "u-1", "preferred_username": "rita"}));
    let callback = sign_in(&fixture).await;
    assert_eq!(location(&callback), "/");
    let reply = make_token(&fixture, &callback).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    let body: Value = reply.json();
    assert!(
        body.to_string().contains("managed outside Astrum"),
        "{body}"
    );
}

async fn grants_with(fixture: &Fixture, callback: &Reply) -> Reply {
    let (cookies, csrf) = session(callback);
    fixture
        .send(
            Method::GET,
            "/service/rest/internal/ui/oauth/grants",
            Auth::None,
            None,
            &[("cookie", &cookies), ("x-astrum-csrf", &csrf)],
        )
        .await
}

#[tokio::test]
async fn an_oidc_session_ends_when_a_local_namesake_appears() {
    let (fixture, provider) = setup().await;
    provider.sign_in_as(json!({"sub": "u-2", "preferred_username": "rita"}));
    let callback = sign_in(&fixture).await;
    assert_eq!(
        grants_with(&fixture, &callback).await.status,
        StatusCode::OK
    );
    fixture
        .add_user("rita", "local-rita-password", &["as-admin"])
        .await;
    let reply = grants_with(&fixture, &callback).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{}", reply.text());
}

#[tokio::test]
async fn an_oidc_session_without_a_namesake_lives_on() {
    let (fixture, provider) = setup().await;
    provider.sign_in_as(json!({"sub": "u-3", "preferred_username": "sam"}));
    let callback = sign_in(&fixture).await;
    assert_eq!(
        grants_with(&fixture, &callback).await.status,
        StatusCode::OK
    );
    fixture
        .add_user("dora", "local-dora-password", &["as-admin"])
        .await;
    assert_eq!(
        grants_with(&fixture, &callback).await.status,
        StatusCode::OK
    );
    assert_eq!(
        grants_with(&fixture, &callback).await.status,
        StatusCode::OK
    );
}
