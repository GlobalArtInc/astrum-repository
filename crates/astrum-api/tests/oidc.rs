mod support;

use astrum_testkit::oidc::MockProvider;
use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const SETTINGS: &str = "/service/rest/internal/ui/oauth2";
const STATE: &str = "/service/rest/internal/ui/state";
const USERS: &str = "/service/rest/v1/security/users";

async fn allow_loopback(fixture: &Fixture) {
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/ssrf-protection",
            Auth::Admin,
            Some(json!({"enabled": true, "allowedIPs": ["127.0.0.1"], "allowedDomains": []})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

async fn configure(fixture: &Fixture, provider: &MockProvider, extra: Value) -> Reply {
    let mut body = json!({
        "issuerUrl": provider.url,
        "clientId": "astrum",
        "clientSecret": "s3cret",
        "groupsClaim": "groups",
    });
    for (name, value) in extra.as_object().unwrap() {
        body[name] = value.clone();
    }
    fixture
        .call(Method::PUT, SETTINGS, Auth::Admin, Some(body))
        .await
}

async fn setup() -> (Fixture, MockProvider) {
    let fixture = Fixture::new().await;
    let provider = MockProvider::start("astrum", Some("s3cret")).await;
    allow_loopback(&fixture).await;
    let reply = configure(&fixture, &provider, json!({})).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    (fixture, provider)
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

fn cookie_pairs(reply: &Reply) -> Vec<(String, String)> {
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

struct Login {
    callback: Reply,
}

impl Login {
    fn cookies(&self) -> String {
        cookie_pairs(&self.callback)
            .into_iter()
            .filter(|(name, value)| name.starts_with("astrum_") && !value.is_empty())
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

async fn follow(url: &str) -> String {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .unwrap();
    let response = client.get(url).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FOUND);
    response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn path_of(url: &str) -> String {
    let parsed = url::Url::parse(url).unwrap();
    match parsed.query() {
        Some(query) => format!("{}?{query}", parsed.path()),
        None => parsed.path().to_owned(),
    }
}

async fn sign_in(fixture: &Fixture, return_to: Option<&str>) -> Login {
    let path = match return_to {
        Some(target) => format!("/oidc/login?hash={}", STANDARD.encode(target)),
        None => "/oidc/login".to_owned(),
    };
    let start = fixture
        .send(
            Method::GET,
            &path,
            Auth::None,
            None,
            &[("host", "astrum.test")],
        )
        .await;
    assert_eq!(start.status, StatusCode::FOUND, "{}", start.text());
    let state_cookie = cookie_pairs(&start)
        .into_iter()
        .find(|(name, _)| name == "astrum_oidc")
        .map(|(name, value)| format!("{name}={value}"))
        .unwrap();
    let back = follow(&location(&start)).await;
    assert!(
        back.starts_with("http://astrum.test/oidc/callback?"),
        "{back}"
    );
    let callback = fixture
        .send(
            Method::GET,
            &path_of(&back),
            Auth::None,
            None,
            &[("cookie", &state_cookie), ("host", "astrum.test")],
        )
        .await;
    Login { callback }
}

#[tokio::test]
async fn the_settings_are_read_saved_and_removed() {
    let fixture = Fixture::new().await;
    let empty = fixture
        .send(
            Method::GET,
            SETTINGS,
            Auth::Admin,
            None,
            &[("host", "astrum.test")],
        )
        .await
        .json();
    assert_eq!(empty["usernameClaim"], "preferred_username");
    assert_eq!(empty["redirectUri"], "http://astrum.test/oidc/callback");
    assert_eq!(empty["configured"], false);
    assert_eq!(empty["enabled"], false);

    let provider = MockProvider::start("astrum", Some("s3cret")).await;
    let blocked = configure(&fixture, &provider, json!({})).await;
    assert_eq!(blocked.status, StatusCode::BAD_REQUEST);
    assert_eq!(blocked.json()[0]["id"], "issuerUrl");
    assert!(
        blocked.json()[0]["message"]
            .as_str()
            .unwrap()
            .contains("SSRF"),
        "{}",
        blocked.text()
    );
    allow_loopback(&fixture).await;
    let missing = fixture
        .call(
            Method::PUT,
            SETTINGS,
            Auth::Admin,
            Some(json!({"issuerUrl": provider.url})),
        )
        .await;
    assert_eq!(missing.json()[0]["id"], "clientId");
    assert_eq!(
        configure(&fixture, &provider, json!({})).await.status,
        StatusCode::NO_CONTENT
    );
    let saved = fixture
        .call(Method::GET, SETTINGS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(saved["clientId"], "astrum");
    assert_eq!(saved["clientSecret"], Value::Null);
    assert_eq!(saved["clientSecretSet"], true);
    assert_eq!(saved["enabled"], true);
    let stored = fixture.state.security().model().oidc.clone().unwrap();
    assert!(!stored.client_secret.unwrap().contains("s3cret"));
    let realms = fixture
        .call(
            Method::GET,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert!(realms.as_array().unwrap().contains(&json!("OAuth2Realm")));
    let state = fixture
        .call(Method::GET, STATE, Auth::None, None)
        .await
        .json();
    assert_eq!(state["oidcEnabled"], true);

    let kept = fixture
        .call(
            Method::PUT,
            SETTINGS,
            Auth::Admin,
            Some(json!({"issuerUrl": provider.url, "clientId": "astrum", "groupsClaim": "roles"})),
        )
        .await;
    assert_eq!(kept.status, StatusCode::NO_CONTENT);
    let model = fixture.state.security().model();
    assert!(model.oidc.as_ref().unwrap().client_secret.is_some());
    assert_eq!(
        model.oidc.as_ref().unwrap().groups_claim.as_deref(),
        Some("roles")
    );

    let removed = fixture
        .call(Method::DELETE, SETTINGS, Auth::Admin, None)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let state = fixture
        .call(Method::GET, STATE, Auth::None, None)
        .await
        .json();
    assert_eq!(state["oidcEnabled"], false);
    let login = fixture
        .call(Method::GET, "/oidc/login", Auth::None, None)
        .await;
    assert_eq!(login.status, StatusCode::NOT_FOUND);

    fixture.add_user("viewer", "viewer-password", &[]).await;
    let forbidden = fixture
        .call(
            Method::GET,
            SETTINGS,
            Auth::Basic("viewer", "viewer-password"),
            None,
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_provider_sign_in_opens_a_session_with_the_groups_as_roles() {
    let (fixture, provider) = setup().await;
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(
                json!({"id": "devs", "name": "Developers", "source": "OAuth2",
                        "privileges": ["as-users-read"]}),
            ),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    provider.sign_in_as(json!({
        "sub": "u-1", "preferred_username": "rita", "given_name": "Rita", "family_name": "Hayworth",
        "groups": ["devs", "ops"]
    }));
    provider.user_info(json!({"email": "rita@example.org"}));

    let login = sign_in(&fixture, Some("/admin/security/users")).await;
    assert_eq!(
        login.callback.status,
        StatusCode::FOUND,
        "{}",
        login.callback.text()
    );
    assert_eq!(location(&login.callback), "/admin/security/users");
    let authorize = provider.last_authorize();
    assert_eq!(authorize["code_challenge_method"], "S256");
    assert_eq!(authorize["scope"], "openid profile email");
    assert_eq!(
        authorize["redirect_uri"],
        "http://astrum.test/oidc/callback"
    );

    let cookies = login.cookies();
    assert!(cookies.contains("astrum_session=") && cookies.contains("astrum_csrf="));
    let state = fixture
        .send(
            Method::GET,
            STATE,
            Auth::None,
            None,
            &[("cookie", &cookies)],
        )
        .await
        .json();
    assert_eq!(state["user"]["userId"], "rita");
    assert_eq!(state["user"]["source"], "OAuth2");
    assert_eq!(state["user"]["name"], "Rita Hayworth");
    assert_eq!(state["user"]["email"], "rita@example.org");
    let users = fixture
        .send(
            Method::GET,
            USERS,
            Auth::None,
            None,
            &[("cookie", &cookies)],
        )
        .await;
    assert_eq!(
        users.status,
        StatusCode::OK,
        "the devs group grants users:read"
    );

    let listed = fixture
        .call(
            Method::GET,
            &format!("{USERS}?source=OAuth2"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(listed[0]["userId"], "rita");
    assert_eq!(listed[0]["externalRoles"], json!(["devs", "ops"]));
    assert_eq!(listed[0]["readOnly"], true);
    let groups = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/roles?source=OAuth2&search=o",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(groups[0]["id"], "ops");

    provider.sign_in_as(json!({"sub": "u-1", "preferred_username": "rita", "groups": []}));
    let again = sign_in(&fixture, None).await;
    assert_eq!(location(&again.callback), "/");
    let cookies = again.cookies();
    let users = fixture
        .send(
            Method::GET,
            USERS,
            Auth::None,
            None,
            &[("cookie", &cookies)],
        )
        .await;
    assert_eq!(
        users.status,
        StatusCode::FORBIDDEN,
        "group changes follow the provider"
    );

    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("{USERS}/rita?realm=OAuth2Realm"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let state = fixture
        .send(
            Method::GET,
            STATE,
            Auth::None,
            None,
            &[("cookie", &cookies)],
        )
        .await
        .json();
    assert_eq!(
        state["user"],
        Value::Null,
        "deleting the user ends the session"
    );
}

async fn failed_with(fixture: &Fixture, code: &str) {
    let login = sign_in(fixture, None).await;
    assert_eq!(location(&login.callback), format!("/login?sso={code}"));
    assert!(!login.cookies().contains("astrum_session="));
}

#[tokio::test]
async fn refused_sign_ins_return_to_the_login_page() {
    let (fixture, provider) = setup().await;
    provider.deny(Some("access_denied"));
    failed_with(&fixture, "provider").await;
    provider.deny(None);

    provider.sign_in_as(json!({"sub": "a", "preferred_username": "anonymous"}));
    failed_with(&fixture, "denied").await;

    let reply = configure(
        &fixture,
        &provider,
        json!({"exactMatchClaims": {"hd": "example.com"}}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    provider.sign_in_as(json!({"sub": "b", "preferred_username": "bea", "hd": "other.org"}));
    failed_with(&fixture, "denied").await;
    provider.sign_in_as(json!({"sub": "b", "preferred_username": "bea", "hd": "example.com"}));
    let allowed = sign_in(&fixture, None).await;
    assert_eq!(location(&allowed.callback), "/");

    let start = fixture
        .call(Method::GET, "/oidc/login", Auth::None, None)
        .await;
    let back = follow(&location(&start)).await;
    let forged = fixture
        .send(
            Method::GET,
            &path_of(&back),
            Auth::None,
            None,
            &[("cookie", "astrum_oidc=forged")],
        )
        .await;
    assert_eq!(location(&forged), "/login?sso=expired");
    let unsafe_return = fixture
        .call(
            Method::GET,
            &format!("/oidc/login?hash={}", STANDARD.encode("//evil.example/x")),
            Auth::None,
            None,
        )
        .await;
    assert_eq!(unsafe_return.status, StatusCode::FOUND);

    let wrong_secret = configure(&fixture, &provider, json!({"clientSecret": "other"})).await;
    assert_eq!(wrong_secret.status, StatusCode::NO_CONTENT);
    let before = provider.token_requests();
    failed_with(&fixture, "failed").await;
    assert_eq!(provider.token_requests(), before + 1);
}
