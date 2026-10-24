#![allow(dead_code)]

use std::sync::Arc;

use astrum_security::AnonymousSettings;
use astrum_security::model::ANONYMOUS_KEY;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::support::{Auth, Fixture, Reply};

pub const UI: &str = "/service/rest/internal/ui";
pub const JAR: &[u8] = b"PK\x03\x04\x14\x00\x08\x00\x08\x00jar-content";

pub async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_maven::Maven));
    fixture
}

pub async fn create(fixture: &Fixture, format: &str, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/{format}/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

pub async fn raw_hosted(fixture: &Fixture, name: &str) {
    raw_hosted_in(fixture, name, "default").await;
}

pub async fn raw_hosted_in(fixture: &Fixture, name: &str, store: &str) {
    create(
        fixture,
        "raw",
        "hosted",
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": store, "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
            "raw": {"contentDisposition": "INLINE"}
        }),
    )
    .await;
}

pub async fn raw_proxy(fixture: &Fixture, name: &str) {
    create(
        fixture,
        "raw",
        "proxy",
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "proxy": {"remoteUrl": "http://upstream.invalid/", "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": false, "timeToLive": 1},
            "httpClient": {"blocked": true, "autoBlock": false},
            "raw": {"contentDisposition": "INLINE"}
        }),
    )
    .await;
}

pub async fn raw_group(fixture: &Fixture, name: &str, members: &[&str]) {
    create(
        fixture,
        "raw",
        "group",
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "group": {"memberNames": members}
        }),
    )
    .await;
}

pub async fn maven_hosted(fixture: &Fixture, name: &str) {
    create(
        fixture,
        "maven",
        "hosted",
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
            "maven": {"versionPolicy": "MIXED", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"}
        }),
    )
    .await;
}

pub async fn put(fixture: &Fixture, path: &str, body: impl Into<Vec<u8>>) {
    let reply = fixture
        .send_body(Method::PUT, path, Auth::Admin, Body::from(body.into()), &[])
        .await;
    assert!(
        reply.status.is_success(),
        "{path}: {} {}",
        reply.status,
        reply.text()
    );
}

pub fn pom(group: &str, artifact: &str, version: &str) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project xmlns=\"http://maven.apache.org/POM/4.0.0\">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>{group}</groupId>\n  <artifactId>{artifact}</artifactId>\n  <version>{version}</version>\n  <packaging>jar</packaging>\n</project>\n"
    )
    .into_bytes()
}

pub async fn deploy(
    fixture: &Fixture,
    repository: &str,
    group: &str,
    artifact: &str,
    version: &str,
) {
    let base = format!(
        "/repository/{repository}/{}/{artifact}/{version}/{artifact}-{version}",
        group.replace('.', "/")
    );
    put(fixture, &format!("{base}.jar"), JAR).await;
    put(
        fixture,
        &format!("{base}.pom"),
        pom(group, artifact, version),
    )
    .await;
}

pub async fn get(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    fixture.call(Method::GET, path, auth, None).await
}

pub async fn role(fixture: &Fixture, id: &str, privileges: &[&str]) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": id, "name": id, "description": "", "privileges": privileges, "roles": []})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

pub async fn user_with(fixture: &Fixture, user: &str, privileges: &[&str]) {
    let role_id = format!("{user}-role");
    role(fixture, &role_id, privileges).await;
    fixture.add_user(user, "password-1", &[&role_id]).await;
}

pub async fn selector(fixture: &Fixture, name: &str, expression: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/content-selectors",
            Auth::Admin,
            Some(json!({"name": name, "description": "", "expression": expression})),
        )
        .await;
    assert!(reply.status.is_success(), "{}", reply.text());
}

pub async fn selector_privilege(
    fixture: &Fixture,
    name: &str,
    selector: &str,
    format: &str,
    repository: &str,
    actions: &[&str],
) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/privileges/repository-content-selector",
            Auth::Admin,
            Some(json!({"name": name, "description": "", "actions": actions, "format": format, "repository": repository, "contentSelector": selector})),
        )
        .await;
    assert!(reply.status.is_success(), "{}", reply.text());
}

pub async fn anonymous(fixture: &Fixture, enabled: bool) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        ANONYMOUS_KEY,
        &AnonymousSettings {
            enabled,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
}

pub fn user(name: &str) -> Auth<'_> {
    Auth::Basic(name, "password-1")
}

pub fn names(value: &Value, field: &str) -> Vec<String> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("not an array: {value}"))
        .iter()
        .map(|item| item[field].as_str().unwrap_or_default().to_owned())
        .collect()
}
