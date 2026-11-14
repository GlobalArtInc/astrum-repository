mod cluster;
mod support;
mod ui_support;

use astrum_replication::protocol::component_key;
use axum::http::{Method, StatusCode};
use cluster::Node;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const MCP: &str = "/mcp";
const TOKENS: &str = "/service/rest/internal/access-tokens";
const UNKNOWN: &str = "00000000-0000-0000-0000-000000000001";

async fn token(fixture: &Fixture, auth: Auth<'_>, scope: &str) -> String {
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            auth,
            Some(json!({"name": format!("mcp-{scope}"), "scope": scope, "expiresInDays": 30})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["token"].as_str().unwrap().to_owned()
}

async fn rpc(fixture: &Fixture, token: &str, method: &str, params: Value) -> Value {
    let bearer = format!("Bearer {token}");
    let reply: Reply = fixture
        .send(
            Method::POST,
            MCP,
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

async fn tool(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> Value {
    let reply = rpc(
        fixture,
        token,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
    .await;
    assert!(reply["error"].is_null(), "{reply}");
    reply["result"].clone()
}

async fn ok(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> Value {
    let result = tool(fixture, token, name, arguments).await;
    assert_eq!(result["isError"], false, "{result}");
    result["structuredContent"].clone()
}

async fn failed(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> String {
    let result = tool(fixture, token, name, arguments).await;
    assert_eq!(result["isError"], true, "{result}");
    result["content"][0]["text"].as_str().unwrap().to_owned()
}

fn names(listed: &Value) -> Vec<String> {
    listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect()
}

async fn pair() -> (Node, Node, String) {
    let (sender, receiver) = cluster::pair().await;
    let id = sender
        .text("SELECT id::text AS t FROM replication")
        .await
        .unwrap();
    (sender, receiver, id)
}

async fn dead_item(node: &Node, id: &str, name: &str) {
    node.sql(&format!(
        "INSERT INTO replication_item (replication_id, namespace, name, version, state, reason, \
         attempts, last_error, first_queued_at, leased_until) \
         VALUES ('{id}', 'ns', '{name}', '1.0', 'dead', 'event', 4, 'broken {name}', \
         now() - interval '2 hours', NULL)"
    ))
    .await;
}

async fn state_of(node: &Node, name: &str) -> Option<String> {
    node.text(&format!(
        "SELECT state AS t FROM replication_item WHERE name = '{name}'"
    ))
    .await
}

#[tokio::test]
async fn the_four_tools_carry_their_hints_and_strict_schemas() {
    let fixture = ready().await;
    let writer = token(&fixture, Auth::Admin, "WRITE").await;
    let listed = rpc(&fixture, &writer, "tools/list", json!({})).await;
    let tools = listed["result"]["tools"].as_array().unwrap();
    let find = |name: &str| {
        tools
            .iter()
            .find(|tool| tool["name"] == name)
            .unwrap_or_else(|| panic!("{name} is not listed"))
            .clone()
    };
    for name in [
        "list_replications",
        "get_replication_status",
        "retry_replication_items",
        "run_replication_reconcile",
    ] {
        assert_eq!(find(name)["inputSchema"]["additionalProperties"], false);
    }
    for name in ["list_replications", "get_replication_status"] {
        assert_eq!(find(name)["annotations"]["readOnlyHint"], true, "{name}");
    }
    for name in ["retry_replication_items", "run_replication_reconcile"] {
        let tool = find(name);
        assert_eq!(tool["annotations"]["readOnlyHint"], false, "{name}");
        assert_eq!(tool["annotations"]["destructiveHint"], false, "{name}");
    }
    let retry = find("retry_replication_items");
    assert_eq!(retry["inputSchema"]["required"], json!(["id"]));
    assert_eq!(retry["inputSchema"]["properties"]["ids"]["maxItems"], 100);
}

#[tokio::test]
async fn the_list_names_the_remote_and_never_shows_a_token() {
    let (sender, receiver, id) = pair().await;
    let reader = token(&sender.fixture, Auth::Admin, "READ").await;
    let result = tool(&sender.fixture, &reader, "list_replications", json!({})).await;
    assert_eq!(result["isError"], false, "{result}");
    assert!(!result.to_string().contains(&receiver.token));
    let replication = &result["structuredContent"]["replications"][0];
    assert_eq!(replication["id"], id);
    assert_eq!(replication["remoteRepository"], "mirror");
    assert!(
        replication["remote"]
            .as_str()
            .unwrap()
            .starts_with("remote-")
    );
    assert_eq!(replication["alertState"], "ok");
}

#[tokio::test]
async fn the_reads_need_the_replication_read_right() {
    let (sender, _receiver, _id) = pair().await;
    user_with(&sender.fixture, "dev", &["as-blobstores-read"]).await;
    user_with(&sender.fixture, "reader", &["as-replication-read"]).await;
    let dev = token(&sender.fixture, Auth::Basic("dev", "password-1"), "READ").await;
    let reader = token(&sender.fixture, Auth::Basic("reader", "password-1"), "READ").await;
    let hidden = names(&rpc(&sender.fixture, &dev, "tools/list", json!({})).await);
    assert!(
        !hidden.contains(&"list_replications".to_owned()),
        "{hidden:?}"
    );
    let refused = rpc(
        &sender.fixture,
        &dev,
        "tools/call",
        json!({"name": "list_replications", "arguments": {}}),
    )
    .await;
    assert_eq!(refused["error"]["code"], -32602);
    let shown = names(&rpc(&sender.fixture, &reader, "tools/list", json!({})).await);
    assert!(shown.contains(&"list_replications".to_owned()), "{shown:?}");
    assert!(
        shown.contains(&"get_replication_status".to_owned()),
        "{shown:?}"
    );
}

#[tokio::test]
async fn the_repairs_need_the_replication_update_right() {
    let (sender, _receiver, id) = pair().await;
    dead_item(&sender, &id, "a").await;
    user_with(&sender.fixture, "reader", &["as-replication-read"]).await;
    let reader = token(
        &sender.fixture,
        Auth::Basic("reader", "password-1"),
        "WRITE",
    )
    .await;
    let shown = names(&rpc(&sender.fixture, &reader, "tools/list", json!({})).await);
    assert!(
        !shown.contains(&"retry_replication_items".to_owned()),
        "{shown:?}"
    );
    assert!(
        !shown.contains(&"run_replication_reconcile".to_owned()),
        "{shown:?}"
    );
    let refused = rpc(
        &sender.fixture,
        &reader,
        "tools/call",
        json!({"name": "retry_replication_items", "arguments": {"id": id, "all": true}}),
    )
    .await;
    assert_eq!(refused["error"]["code"], -32602);
    assert_eq!(state_of(&sender, "a").await.as_deref(), Some("dead"));
}

#[tokio::test]
async fn the_status_lists_the_dead_items_with_their_errors() {
    let (sender, _receiver, id) = pair().await;
    let reader = token(&sender.fixture, Auth::Admin, "READ").await;
    let healthy = ok(
        &sender.fixture,
        &reader,
        "get_replication_status",
        json!({"id": id}),
    )
    .await;
    assert_eq!(healthy["dead"], 0);
    assert!(healthy["deadItems"].is_null());

    dead_item(&sender, &id, "a").await;
    dead_item(&sender, &id, "b").await;
    let status = ok(
        &sender.fixture,
        &reader,
        "get_replication_status",
        json!({"id": id}),
    )
    .await;
    assert_eq!(status["dead"], 2);
    let dead = status["deadItems"].as_array().unwrap();
    assert_eq!(dead.len(), 2);
    assert_eq!(dead[0]["lastError"], "broken a");
    assert_eq!(dead[0]["attempts"], 4);
    assert_eq!(dead[0]["key"], component_key("ns", "a", "1.0"));
}

#[tokio::test]
async fn an_unknown_replication_is_a_tool_error_with_the_404() {
    let (sender, _receiver, _id) = pair().await;
    let reader = token(&sender.fixture, Auth::Admin, "READ").await;
    let message = failed(
        &sender.fixture,
        &reader,
        "get_replication_status",
        json!({"id": UNKNOWN}),
    )
    .await;
    assert!(message.starts_with("Astrum answered 404"), "{message}");
}

#[tokio::test]
async fn retry_requeues_the_dead_items_by_id_or_all() {
    let (sender, _receiver, id) = pair().await;
    dead_item(&sender, &id, "a").await;
    dead_item(&sender, &id, "b").await;
    let writer = token(&sender.fixture, Auth::Admin, "WRITE").await;
    let a = sender
        .number("SELECT id AS n FROM replication_item WHERE name = 'a'")
        .await;
    let one = ok(
        &sender.fixture,
        &writer,
        "retry_replication_items",
        json!({"id": id, "ids": [a]}),
    )
    .await;
    assert_eq!(one["retried"], 1);
    assert_eq!(state_of(&sender, "a").await.as_deref(), Some("pending"));
    assert_eq!(state_of(&sender, "b").await.as_deref(), Some("dead"));

    let all = ok(
        &sender.fixture,
        &writer,
        "retry_replication_items",
        json!({"id": id, "all": true}),
    )
    .await;
    assert_eq!(all["retried"], 1);
    assert_eq!(state_of(&sender, "b").await.as_deref(), Some("pending"));
}

#[tokio::test]
async fn retry_takes_exactly_one_of_ids_or_all_up_to_a_hundred_ids() {
    let (sender, _receiver, id) = pair().await;
    let writer = token(&sender.fixture, Auth::Admin, "WRITE").await;
    let many: Vec<i64> = (1..=101).collect();
    let cases = [
        (json!({"id": id}), "give either"),
        (json!({"id": id, "ids": [1], "all": true}), "give either"),
        (json!({"id": id, "all": false}), "give either"),
        (json!({"id": id, "ids": many}), "between 1 and 100"),
    ];
    for (arguments, expected) in cases {
        let message = failed(
            &sender.fixture,
            &writer,
            "retry_replication_items",
            arguments,
        )
        .await;
        assert!(message.contains(expected), "{message}");
    }
}

#[tokio::test]
async fn reconcile_queues_a_run_and_a_second_request_says_it_is_running() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = cluster::connect(&sender, &receiver).await;
    let id = cluster::replicate(
        &sender,
        &remote,
        "source",
        "mirror",
        json!({"enabled": false}),
    )
    .await;
    let writer = token(&sender.fixture, Auth::Admin, "WRITE").await;
    let first = ok(
        &sender.fixture,
        &writer,
        "run_replication_reconcile",
        json!({"id": id}),
    )
    .await;
    assert_eq!(first["started"], true);
    assert!(!first["run"].is_null(), "{first}");
    let second = ok(
        &sender.fixture,
        &writer,
        "run_replication_reconcile",
        json!({"id": id}),
    )
    .await;
    assert_eq!(second["started"], false);
    assert!(
        second["message"]
            .as_str()
            .unwrap()
            .contains("already running"),
        "{second}"
    );
}
