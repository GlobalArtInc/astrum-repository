mod support;
mod ui_support;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::{create, ready};

const BASE: &str = "http://astrum.example";

async fn token(fixture: &Fixture) -> String {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Admin,
            Some(json!({"name": "quota", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["token"].as_str().unwrap().to_owned()
}

async fn ticket(fixture: &Fixture, token: &str, file_name: &str) -> String {
    let bearer = format!("Bearer {token}");
    let reply = fixture
        .send(
            Method::POST,
            "/mcp",
            Auth::None,
            Some(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": "request_upload", "arguments": {"repository": "capped", "fileName": file_name, "fields": {"directory": "/d"}}}
            })),
            &[("authorization", bearer.as_str()), ("host", "astrum.example")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let issued: Value = reply.json()["result"]["structuredContent"].clone();
    issued["uploadUrl"]
        .as_str()
        .unwrap()
        .strip_prefix(BASE)
        .unwrap()
        .to_owned()
}

async fn upload(fixture: &Fixture, path: &str, size: usize) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            Auth::None,
            Body::from(vec![b'x'; size]),
            &[("host", "astrum.example")],
        )
        .await
}

#[tokio::test]
async fn an_mcp_upload_over_the_limit_is_refused_and_a_smaller_one_passes() {
    let fixture = ready().await;
    create(
        &fixture,
        "raw",
        "hosted",
        json!({
            "name": "capped",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW", "quota": {"limitBytes": 20, "enforce": true}},
            "raw": {"contentDisposition": "INLINE"}
        }),
    )
    .await;
    let token = token(&fixture).await;
    let first = ticket(&fixture, &token, "big.bin").await;
    let refused = upload(&fixture, &first, 21).await;
    assert_eq!(
        refused.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        refused.text()
    );
    assert!(refused.text().contains("capped"), "{}", refused.text());
    let second = ticket(&fixture, &token, "small.bin").await;
    let accepted = upload(&fixture, &second, 20).await;
    assert_eq!(accepted.status, StatusCode::CREATED, "{}", accepted.text());
}
