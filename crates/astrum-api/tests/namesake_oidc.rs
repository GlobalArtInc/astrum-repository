mod support;

use astrum_testkit::oidc::MockProvider;
use axum::http::{Method, StatusCode, header};
use serde_json::json;
use support::{Auth, Fixture, Reply};

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
