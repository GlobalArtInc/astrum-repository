#![allow(dead_code)]

#[path = "../../../astrum-security/tests/support/soft_passkey.rs"]
pub mod soft_passkey;

use astrum_db::setting;
use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use soft_passkey::SoftPasskey;

use crate::support::two_factor::*;
use crate::support::{Auth, Fixture, Reply};

pub const BASE: &str = "https://astrum.example.org";
pub const RP: &str = "astrum.example.org";
pub const OPTIONS: &str = "/service/rest/internal/ui/user/two-factor/passkeys/options";
pub const PASSKEYS: &str = "/service/rest/internal/ui/user/two-factor/passkeys";
pub const SIGN_IN_OPTIONS: &str = "/service/rest/internal/session/second-factor/passkey-options";
pub const POLICY: &str = "/service/rest/v1/security/two-factor";

pub async fn set_base(fixture: &Fixture, url: &str) {
    let base = BaseUrl {
        url: url.to_owned(),
        enabled: true,
        notes: String::new(),
    };
    setting::put(fixture.state.db().conn(), BASE_URL_KEY, &base)
        .await
        .unwrap();
    fixture.state.security().changed().await.unwrap();
}

pub async fn passkey_world() -> Fixture {
    let fixture = world().await;
    set_base(&fixture, BASE).await;
    fixture
}

pub fn key(seed: u8) -> SoftPasskey {
    SoftPasskey::new(seed, RP, BASE)
}

pub async fn act(
    fixture: &Fixture,
    method: Method,
    path: &str,
    login: &Reply,
    body: Value,
) -> Reply {
    with_session(fixture, method, path, login, Some(body)).await
}

pub async fn begin(fixture: &Fixture, login: &Reply, secret: &str, name: &str) -> Reply {
    let body = json!({"currentPassword": secret, "name": name});
    act(fixture, Method::POST, OPTIONS, login, body).await
}

pub async fn register(fixture: &Fixture, login: &Reply, began: &Value, key: &SoftPasskey) -> Reply {
    register_as(fixture, login, began, key, JANE).await
}

pub async fn register_as(
    fixture: &Fixture,
    login: &Reply,
    began: &Value,
    key: &SoftPasskey,
    secret: &str,
) -> Reply {
    let body = json!({
        "ceremony": began["ceremony"],
        "credential": key.register(&began["publicKey"], 0),
        "currentPassword": secret,
    });
    act(fixture, Method::POST, PASSKEYS, login, body).await
}

pub async fn add(fixture: &Fixture, login: &Reply, key: &SoftPasskey, name: &str) -> Value {
    add_as(fixture, login, key, name, JANE).await
}

pub async fn add_as(
    fixture: &Fixture,
    login: &Reply,
    key: &SoftPasskey,
    name: &str,
    secret: &str,
) -> Value {
    let began = begin(fixture, login, secret, name).await;
    assert_eq!(began.status, StatusCode::OK, "{}", began.text());
    let done = register_as(fixture, login, &began.json(), key, secret).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text());
    done.json()
}

pub async fn status(fixture: &Fixture, login: &Reply) -> Value {
    let reply = with_session(fixture, Method::GET, FACTORS, login, None).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

pub fn cookie_of(handle: &str) -> String {
    format!("astrum_signin={handle}")
}

pub async fn options_for(fixture: &Fixture, handle: &str) -> Reply {
    let cookie = cookie_of(handle);
    let body = Some(json!({}));
    let headers = [("cookie", cookie.as_str())];
    fixture
        .send(Method::POST, SIGN_IN_OPTIONS, Auth::None, body, &headers)
        .await
}

pub async fn submit(fixture: &Fixture, handle: &str, ceremony: &Value, credential: Value) -> Reply {
    let cookie = cookie_of(handle);
    let body = json!({"method": "passkey", "ceremony": ceremony, "credential": credential});
    let headers = [("cookie", cookie.as_str())];
    fixture
        .send(Method::POST, SECOND, Auth::None, Some(body), &headers)
        .await
}

pub async fn challenge(fixture: &Fixture, user: &str, secret: &str) -> (String, Value) {
    let handle = start(fixture, user, secret).await;
    let options = options_for(fixture, &handle).await;
    assert_eq!(options.status, StatusCode::OK, "{}", options.text());
    (handle, options.json())
}

pub async fn sign_in(fixture: &Fixture, key: &SoftPasskey, counter: u32) -> Reply {
    let (handle, options) = challenge(fixture, "jane", JANE).await;
    let assertion = key.assert(&options["publicKey"], counter);
    submit(fixture, &handle, &options["ceremony"], assertion).await
}

pub fn field(reply: &Reply, id: &str) {
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert_eq!(reply.json()[0]["id"], id, "{}", reply.text());
}

pub async fn passkey_rows(fixture: &Fixture) -> i64 {
    number(fixture, "SELECT count(*) FROM user_passkey").await
}

pub fn keys(value: &Value) -> Vec<&str> {
    let mut names: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    names
}

pub fn kinds(events: &[Value]) -> Vec<&str> {
    events
        .iter()
        .map(|event| event["type"].as_str().unwrap())
        .collect()
}

pub fn base64_url(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}
