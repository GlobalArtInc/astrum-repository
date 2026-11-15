#![allow(dead_code)]

use astrum_db::setting;
use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use axum::http::Method;
use serde_json::{Value, json};

use crate::accounts::create_role;
use crate::support::two_factor::world;
use crate::support::{Auth, Fixture, Reply};

pub const SECURITY: &str = "/service/rest/v1/security";
pub const POLICY: &str = "/service/rest/v1/security/two-factor";
pub const READER: &str = "reader-password-1";
pub const UPDATER: &str = "updater-password-1";
pub const PLAIN: &str = "plain-password-1";

pub fn status_path(user: &str) -> String {
    format!("{SECURITY}/users/{user}/two-factor")
}

pub fn requirement_path(user: &str) -> String {
    format!("{SECURITY}/users/{user}/requirement")
}

pub fn policy(patch: Value) -> Value {
    let mut body = json!({
        "totp": true,
        "passkeys": true,
        "required": {"everyone": false, "administrators": false, "roles": []},
        "graceDays": 7,
        "requiredSince": null,
        "relyingParty": {"id": null, "origins": []},
    });
    for (key, value) in patch.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

pub async fn people() -> Fixture {
    let fixture = world().await;
    create_role(&fixture, "readers", &["as-users-read"], &[]).await;
    create_role(&fixture, "updaters", &["as-users-update"], &[]).await;
    fixture.add_user("reader", READER, &["readers"]).await;
    fixture.add_user("updater", UPDATER, &["updaters"]).await;
    fixture.add_user("plain", PLAIN, &[]).await;
    fixture
}

pub async fn put_policy(fixture: &Fixture, body: Value) -> Reply {
    fixture
        .call(Method::PUT, POLICY, Auth::Admin, Some(body))
        .await
}

pub async fn put_requirement(
    fixture: &Fixture,
    auth: Auth<'_>,
    user: &str,
    requirement: &str,
) -> Reply {
    fixture
        .call(
            Method::PUT,
            &requirement_path(user),
            auth,
            Some(json!({"requirement": requirement})),
        )
        .await
}

pub async fn set_base(fixture: &Fixture, url: &str) {
    let base = BaseUrl {
        url: url.to_owned(),
        enabled: true,
        notes: String::new(),
    };
    setting::put(fixture.state.db().conn(), BASE_URL_KEY, &base)
        .await
        .unwrap();
    fixture.state.forget_base_url();
    fixture.state.security().changed().await.unwrap();
}
