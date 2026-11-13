mod support;
mod ui_support;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::{create, ready};

const BASE: &str = "http://astrum.example";

async fn token(fixture: &Fixture, scope: &str) -> String {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Admin,
            Some(json!({"name": format!("quota-{scope}"), "scope": scope, "expiresInDays": 30})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["token"].as_str().unwrap().to_owned()
}

async fn rpc(fixture: &Fixture, token: &str, method: &str, params: Value) -> Value {
    let bearer = format!("Bearer {token}");
    let reply = fixture
        .send(
            Method::POST,
            "/mcp",
            Auth::None,
            Some(json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})),
            &[
                ("authorization", bearer.as_str()),
                ("host", "astrum.example"),
            ],
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

async fn ok(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> Value {
    let reply = rpc(
        fixture,
        token,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
    .await;
    assert!(reply["error"].is_null(), "{reply}");
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    reply["result"]["structuredContent"].clone()
}

async fn failed(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> String {
    let reply = rpc(
        fixture,
        token,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
    .await;
    assert_eq!(reply["result"]["isError"], true, "{reply}");
    reply["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_owned()
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
    let token = token(&fixture, "WRITE").await;
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

#[tokio::test]
async fn usage_is_read_and_the_quota_is_set_through_the_tools() {
    let fixture = ready().await;
    create(
        &fixture,
        "raw",
        "hosted",
        json!({
            "name": "capped",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW", "quota": {"limitBytes": 100, "warnPercent": 80, "enforce": true}},
            "raw": {"contentDisposition": "INLINE"}
        }),
    )
    .await;
    let uploaded = fixture
        .send_body(
            Method::PUT,
            "/repository/capped/a.bin",
            Auth::Admin,
            Body::from(vec![b'x'; 25]),
            &[],
        )
        .await;
    assert_eq!(uploaded.status, StatusCode::CREATED, "{}", uploaded.text());
    let token = token(&fixture, "WRITE").await;

    let usage = ok(
        &fixture,
        &token,
        "get_repository_usage",
        json!({"name": "capped"}),
    )
    .await;
    assert_eq!(usage["bytes"], 25);
    assert_eq!(usage["quota"]["warnPercent"], 80);
    assert_eq!(usage["percentUsed"], 25.0);

    let set = ok(
        &fixture,
        &token,
        "set_repository_quota",
        json!({"name": "capped", "limitBytes": 200, "warnPercent": 50}),
    )
    .await;
    assert_eq!(set["quota"]["limitBytes"], 200);
    assert_eq!(set["quota"]["warnPercent"], 50);
    assert_eq!(set["quota"]["enforce"], true);
    let usage = ok(
        &fixture,
        &token,
        "get_repository_usage",
        json!({"name": "capped"}),
    )
    .await;
    assert_eq!(usage["percentUsed"], 12.5);

    let removed = ok(
        &fixture,
        &token,
        "set_repository_quota",
        json!({"name": "capped", "limitBytes": null}),
    )
    .await;
    assert!(removed["quota"].is_null(), "{removed}");
    let usage = ok(
        &fixture,
        &token,
        "get_repository_usage",
        json!({"name": "capped"}),
    )
    .await;
    assert!(usage["quota"].is_null(), "{usage}");
    assert!(usage["percentUsed"].is_null(), "{usage}");
}

#[tokio::test]
async fn a_read_token_has_no_quota_setter_and_validation_refuses_a_proxy_hard_limit() {
    let fixture = ready().await;
    create(
        &fixture,
        "raw",
        "proxy",
        json!({
            "name": "mirror",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "quota": {"limitBytes": 10}},
            "proxy": {"remoteUrl": "https://example.org/", "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 60},
            "httpClient": {"blocked": false, "autoBlock": true},
            "raw": {"contentDisposition": "ATTACHMENT"}
        }),
    )
    .await;
    let read = token(&fixture, "READ").await;
    let write = token(&fixture, "WRITE").await;

    let listed = rpc(&fixture, &read, "tools/list", json!({})).await;
    let names: Vec<&str> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"get_repository_usage"), "{names:?}");
    assert!(!names.contains(&"set_repository_quota"), "{names:?}");
    let refused = rpc(
        &fixture,
        &read,
        "tools/call",
        json!({"name": "set_repository_quota", "arguments": {"name": "mirror", "limitBytes": 20}}),
    )
    .await;
    assert_eq!(refused["error"]["code"], -32602);

    let problem = failed(
        &fixture,
        &write,
        "set_repository_quota",
        json!({"name": "mirror", "limitBytes": 20, "enforce": true}),
    )
    .await;
    assert!(problem.contains("quota.enforce"), "{problem}");
}
