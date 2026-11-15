mod support;
#[path = "support/namesake.rs"]
mod world;

use astrum_db::entity::access_token;
use astrum_security::{EXTERNAL_TOKENS_OFF, Security, oidc, realm, tokens};
use astrum_testkit::oidc::MockProvider;
use axum::http::{Method, StatusCode, header};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use world::{TOKENS, alice, setup};

const STATE: &str = "/service/rest/internal/ui/state";
const USERS: &str = "/service/rest/v1/security/users";
const HOST: (&str, &str) = ("host", "astrum.test");
const PASSWORD: &str = "local-rita-password";

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

async fn setup_oidc(external_tokens: bool) -> (Fixture, MockProvider) {
    let fixture = Fixture::new().await;
    fixture
        .state
        .security()
        .set_external_tokens(external_tokens);
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

async fn sign_in(fixture: &Fixture, provider: &MockProvider, claims: Value) -> (String, String) {
    provider.sign_in_as(claims);
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
    let callback = fixture
        .send(
            Method::GET,
            &path,
            Auth::None,
            None,
            &[("cookie", &state), HOST],
        )
        .await;
    assert_eq!(location(&callback), "/");
    let cookies = pairs(&callback);
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

async fn within(
    fixture: &Fixture,
    method: Method,
    path: &str,
    session: &(String, String),
    body: Option<Value>,
) -> Reply {
    let headers = [
        ("cookie", session.0.as_str()),
        ("x-astrum-csrf", session.1.as_str()),
    ];
    fixture.send(method, path, Auth::None, body, &headers).await
}

fn draft() -> Value {
    json!({"name": "ci", "scope": "WRITE", "expiresInDays": null})
}

async fn who_am_i(fixture: &Fixture, user: &str, token: &str) -> Value {
    let reply = fixture
        .call(Method::GET, STATE, Auth::Basic(user, token), None)
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["user"].clone()
}

async fn insert_row(security: &Security, source: &str, user_id: &str) -> uuid::Uuid {
    let plain = format!(
        "astrum_{}{}",
        uuid::Uuid::new_v4().simple(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let now = chrono::Utc::now().fixed_offset();
    let id = uuid::Uuid::new_v4();
    access_token::Entity::insert(access_token::ActiveModel {
        id: ActiveValue::Set(id),
        user_id: ActiveValue::Set(user_id.to_owned()),
        source: ActiveValue::Set(source.to_owned()),
        name: ActiveValue::Set("row".to_owned()),
        token_hash: ActiveValue::Set(tokens::hash(&plain)),
        token_hint: ActiveValue::Set(plain[..11].to_owned()),
        scope: ActiveValue::Set("READ".to_owned()),
        created_at: ActiveValue::Set(now),
        expires_at: ActiveValue::Set(None),
        last_used_at: ActiveValue::Set(None),
        customer: ActiveValue::Set(None),
        repositories: ActiveValue::Set(None),
        created_by: ActiveValue::Set(None),
        last_used_ip: ActiveValue::Set(None),
        expiry_notice: ActiveValue::Set(None),
        external_subject: ActiveValue::Set(None),
    })
    .exec(security.db())
    .await
    .unwrap();
    id
}

async fn held(security: &Security, source: &str, user_id: &str) -> usize {
    security.tokens_of(source, user_id).await.unwrap().len()
}

async fn local_token(fixture: &Fixture) {
    fixture.add_user("rita", PASSWORD, &["as-admin"]).await;
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Basic("rita", PASSWORD),
            Some(draft()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn delete_user(fixture: &Fixture, path: &str) {
    let reply = fixture.call(Method::DELETE, path, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

#[tokio::test]
async fn an_oidc_account_lists_creates_and_removes_its_tokens() {
    let (fixture, provider) = setup_oidc(true).await;
    let session = sign_in(
        &fixture,
        &provider,
        json!({"sub": "u-1", "preferred_username": "rita"}),
    )
    .await;
    let created = within(&fixture, Method::POST, TOKENS, &session, Some(draft())).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let created: Value = created.json();
    assert_eq!(created["source"], oidc::SOURCE);
    assert_eq!(created["userId"], "rita");
    let token = created["token"].as_str().unwrap().to_owned();

    let listed = within(&fixture, Method::GET, TOKENS, &session, None).await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let listed: Value = listed.json();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["source"], oidc::SOURCE);
    assert_eq!(listed[0]["id"], created["id"]);

    let user = who_am_i(&fixture, "rita", &token).await;
    assert_eq!(user["userId"], "rita");
    assert_eq!(user["source"], oidc::SOURCE);

    let path = format!("{TOKENS}/{}", created["id"].as_str().unwrap());
    let removed = within(&fixture, Method::DELETE, &path, &session, None).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let listed = within(&fixture, Method::GET, TOKENS, &session, None).await;
    assert_eq!(listed.json(), json!([]));
    let dead = fixture
        .call(Method::GET, STATE, Auth::Basic("rita", &token), None)
        .await;
    assert_eq!(dead.status, StatusCode::UNAUTHORIZED, "{}", dead.text());
}

#[tokio::test]
async fn an_oidc_account_lists_and_removes_only_its_own_source_and_user() {
    let (fixture, provider) = setup_oidc(true).await;
    let security = fixture.state.security();
    let session = sign_in(
        &fixture,
        &provider,
        json!({"sub": "u-5", "preferred_username": "rita"}),
    )
    .await;
    let created = within(&fixture, Method::POST, TOKENS, &session, Some(draft())).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let own = created.json()["id"].as_str().unwrap().to_owned();
    let foreign = [
        insert_row(security, "default", "rita").await,
        insert_row(security, "LDAP", "rita").await,
    ];

    let listed = within(&fixture, Method::GET, TOKENS, &session, None).await;
    let listed: Value = listed.json();
    assert_eq!(listed.as_array().unwrap().len(), 1, "{listed}");
    assert_eq!(listed[0]["id"], own);
    for id in foreign {
        let path = format!("{TOKENS}/{id}");
        let reply = within(&fixture, Method::DELETE, &path, &session, None).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{}", reply.text());
        assert!(security.token(id).await.unwrap().is_some());
    }
    let path = format!("{TOKENS}/{own}");
    let reply = within(&fixture, Method::DELETE, &path, &session, None).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(held(security, oidc::SOURCE, "rita").await, 0);
}

#[tokio::test]
async fn an_external_account_can_still_remove_its_own_token_while_the_option_is_off() {
    let (fixture, provider) = setup_oidc(true).await;
    let security = fixture.state.security();
    let session = sign_in(
        &fixture,
        &provider,
        json!({"sub": "u-6", "preferred_username": "rita"}),
    )
    .await;
    let created = within(&fixture, Method::POST, TOKENS, &session, Some(draft())).await;
    let own = created.json()["id"].as_str().unwrap().to_owned();
    let other = insert_row(security, "LDAP", "rita").await;
    security.set_external_tokens(false);

    let listed = within(&fixture, Method::GET, TOKENS, &session, None).await;
    assert_eq!(listed.status, StatusCode::FORBIDDEN, "{}", listed.text());
    let path = format!("{TOKENS}/{other}");
    let reply = within(&fixture, Method::DELETE, &path, &session, None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND, "{}", reply.text());
    let path = format!("{TOKENS}/{own}");
    let reply = within(&fixture, Method::DELETE, &path, &session, None).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(held(security, oidc::SOURCE, "rita").await, 0);
    assert!(security.token(other).await.unwrap().is_some());
}

#[tokio::test]
async fn r2_a_token_of_an_oidc_account_cannot_create_tokens() {
    let (fixture, provider) = setup_oidc(true).await;
    let session = sign_in(
        &fixture,
        &provider,
        json!({"sub": "u-2", "preferred_username": "rita"}),
    )
    .await;
    let created = within(&fixture, Method::POST, TOKENS, &session, Some(draft())).await;
    let token = created.json()["token"].as_str().unwrap().to_owned();
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Basic("rita", &token),
            Some(draft()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    assert!(
        reply
            .text()
            .contains("access tokens are managed after signing in with a password"),
        "{}",
        reply.text()
    );
}

#[tokio::test]
async fn r8_deleting_a_user_removes_the_tokens_of_its_source_only() {
    let (fixture, provider) = setup_oidc(true).await;
    let security = fixture.state.security();
    let session = sign_in(
        &fixture,
        &provider,
        json!({"sub": "u-3", "preferred_username": "rita"}),
    )
    .await;
    let created = within(&fixture, Method::POST, TOKENS, &session, Some(draft())).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    local_token(&fixture).await;
    assert_eq!(held(security, oidc::SOURCE, "rita").await, 1);
    assert_eq!(held(security, "default", "rita").await, 1);
    delete_user(&fixture, &format!("{USERS}/rita")).await;
    assert_eq!(held(security, oidc::SOURCE, "rita").await, 1);
    assert_eq!(held(security, "default", "rita").await, 0);

    local_token(&fixture).await;
    delete_user(&fixture, &format!("{USERS}/rita?realm={}", realm::OIDC)).await;
    assert_eq!(held(security, oidc::SOURCE, "rita").await, 0);
    assert_eq!(held(security, "default", "rita").await, 1);
}

#[tokio::test]
async fn r13_with_the_option_off_an_oidc_account_gets_an_explicit_reason() {
    let (fixture, provider) = setup_oidc(false).await;
    let session = sign_in(
        &fixture,
        &provider,
        json!({"sub": "u-4", "preferred_username": "rita"}),
    )
    .await;
    let listed = within(&fixture, Method::GET, TOKENS, &session, None).await;
    assert_eq!(listed.status, StatusCode::FORBIDDEN, "{}", listed.text());
    assert!(
        listed.text().contains(EXTERNAL_TOKENS_OFF),
        "{}",
        listed.text()
    );
    let created = within(&fixture, Method::POST, TOKENS, &session, Some(draft())).await;
    assert_eq!(created.status, StatusCode::FORBIDDEN, "{}", created.text());
    assert!(
        created.text().contains(EXTERNAL_TOKENS_OFF),
        "{}",
        created.text()
    );

    fixture.add_user("sam", PASSWORD, &["as-admin"]).await;
    let local = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Basic("sam", PASSWORD),
            Some(draft()),
        )
        .await;
    assert_eq!(local.status, StatusCode::CREATED, "{}", local.text());
    let listed = fixture
        .call(Method::GET, TOKENS, Auth::Basic("sam", PASSWORD), None)
        .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn the_state_tells_the_ui_whether_external_tokens_are_on() {
    let fixture = Fixture::new().await;
    let reply = fixture.call(Method::GET, STATE, Auth::None, None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["externalTokens"], false);
    fixture.state.security().set_external_tokens(true);
    let reply = fixture.call(Method::GET, STATE, Auth::None, None).await;
    assert_eq!(reply.json()["externalTokens"], true);
}

#[tokio::test]
async fn an_ldap_account_gets_a_token_when_the_option_is_on() {
    let world = setup().await;
    world.fixture.state.security().set_external_tokens(true);
    let created = world
        .fixture
        .call(Method::POST, TOKENS, alice(), Some(draft()))
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let created: Value = created.json();
    assert_eq!(created["source"], "LDAP");
    assert_eq!(created["userId"], "alice");
    let token = created["token"].as_str().unwrap().to_owned();
    let user = who_am_i(&world.fixture, "alice", &token).await;
    assert_eq!(user["userId"], "alice");
    assert_eq!(user["source"], "LDAP");
    let listed = world.fixture.call(Method::GET, TOKENS, alice(), None).await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
}
