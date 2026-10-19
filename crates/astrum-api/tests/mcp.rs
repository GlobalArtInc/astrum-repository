mod support;
mod ui_support;

use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const MCP: &str = "/mcp";
const TOKENS: &str = "/service/rest/internal/access-tokens";

async fn token(fixture: &Fixture, scope: &str) -> String {
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Admin,
            Some(json!({"name": format!("mcp-{scope}"), "scope": scope, "expiresInDays": 30})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["token"].as_str().unwrap().to_owned()
}

async fn rpc(fixture: &Fixture, token: &str, method: &str, params: Value) -> Value {
    let reply = post(
        fixture,
        token,
        json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}),
        &[],
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

async fn post(fixture: &Fixture, token: &str, body: Value, headers: &[(&str, &str)]) -> Reply {
    let bearer = format!("Bearer {token}");
    let mut all = vec![
        ("authorization", bearer.as_str()),
        ("host", "astrum.example"),
    ];
    all.extend_from_slice(headers);
    fixture
        .send(Method::POST, MCP, Auth::None, Some(body), &all)
        .await
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

#[tokio::test]
async fn the_endpoint_wants_a_signed_in_caller_from_its_own_origin() {
    let fixture = ready().await;
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "ping"});
    let anonymous = fixture
        .send(Method::POST, MCP, Auth::None, Some(body.clone()), &[])
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(
        anonymous.headers[header::WWW_AUTHENTICATE]
            .to_str()
            .unwrap()
            .starts_with("Bearer")
    );
    let unknown = fixture
        .send(
            Method::POST,
            MCP,
            Auth::None,
            Some(body.clone()),
            &[(
                "authorization",
                "Bearer astrum_000000000000000000000000000000000000000",
            )],
        )
        .await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);

    let token = token(&fixture, "READ").await;
    let foreign = post(
        &fixture,
        &token,
        body.clone(),
        &[("origin", "https://evil.example")],
    )
    .await;
    assert_eq!(foreign.status, StatusCode::FORBIDDEN);
    let own = post(
        &fixture,
        &token,
        body.clone(),
        &[("origin", "https://astrum.example")],
    )
    .await;
    assert_eq!(own.status, StatusCode::OK);
    assert_eq!(own.json()["result"], json!({}));
    let future = post(
        &fixture,
        &token,
        body.clone(),
        &[("mcp-protocol-version", "2099-01-01")],
    )
    .await;
    assert_eq!(future.status, StatusCode::BAD_REQUEST);
    let opening = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}}});
    let unsigned = fixture
        .send(
            Method::POST,
            MCP,
            Auth::None,
            Some(opening.clone()),
            &[("mcp-protocol-version", "2099-01-01")],
        )
        .await;
    assert_eq!(unsigned.status, StatusCode::UNAUTHORIZED);
    assert!(unsigned.headers.contains_key(header::WWW_AUTHENTICATE));
    let negotiated = post(
        &fixture,
        &token,
        opening,
        &[("mcp-protocol-version", "2099-01-01")],
    )
    .await;
    assert_eq!(negotiated.status, StatusCode::OK, "{}", negotiated.text());
    assert_eq!(negotiated.json()["result"]["protocolVersion"], "2025-11-25");
    let notified = post(
        &fixture,
        &token,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        &[],
    )
    .await;
    assert_eq!(notified.status, StatusCode::ACCEPTED);
    let streamed = fixture.call(Method::GET, MCP, Auth::Admin, None).await;
    assert_eq!(streamed.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(streamed.headers[header::ALLOW], "POST");

    let basic = fixture
        .call(Method::POST, MCP, Auth::Admin, Some(body))
        .await;
    assert_eq!(basic.status, StatusCode::OK);
}

#[tokio::test]
async fn a_read_token_sees_only_the_reading_tools() {
    let fixture = ready().await;
    let read = token(&fixture, "READ").await;
    let write = token(&fixture, "WRITE").await;

    let initialized = rpc(
        &fixture,
        &read,
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}}),
    )
    .await;
    assert_eq!(initialized["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(
        initialized["result"]["serverInfo"]["name"],
        "astrum-repository"
    );
    assert_eq!(
        initialized["result"]["serverInfo"]["icons"][0]["src"],
        "http://astrum.example/favicon.svg"
    );
    assert_eq!(
        initialized["result"]["serverInfo"]["websiteUrl"],
        "http://astrum.example"
    );
    assert!(
        initialized["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("search_components")
    );

    let reading = names(&rpc(&fixture, &read, "tools/list", json!({})).await);
    for name in [
        "get_status",
        "search_components",
        "list_versions",
        "read_text_asset",
        "list_docker_tags",
        "list_blob_stores",
        "list_tasks",
    ] {
        assert!(reading.contains(&name.to_owned()), "{name}: {reading:?}");
    }
    for name in [
        "run_task",
        "convert_blob_store_to_group",
        "set_repository_online",
    ] {
        assert!(!reading.contains(&name.to_owned()), "{name}: {reading:?}");
    }
    let refused = rpc(
        &fixture,
        &read,
        "tools/call",
        json!({"name": "run_task", "arguments": {"id": "x"}}),
    )
    .await;
    assert_eq!(refused["error"]["code"], -32602);

    let writing = rpc(&fixture, &write, "tools/list", json!({})).await;
    let all = names(&writing);
    assert!(all.contains(&"run_task".to_owned()), "{all:?}");
    assert!(
        all.contains(&"update_group_blob_store".to_owned()),
        "{all:?}"
    );
    let run = writing["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "run_task")
        .unwrap()
        .clone();
    assert_eq!(run["annotations"]["readOnlyHint"], false);
    assert_eq!(run["inputSchema"]["required"], json!(["id"]));

    user_with(&fixture, "dev", &["as-blobstores-read"]).await;
    let narrow = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Basic("dev", "password-1"),
            Some(json!({"name": "mcp", "scope": "WRITE", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(narrow.status, StatusCode::CREATED, "{}", narrow.text());
    let narrow = narrow.json()["token"].as_str().unwrap().to_owned();
    let narrow_tools = names(&rpc(&fixture, &narrow, "tools/list", json!({})).await);
    assert!(narrow_tools.contains(&"list_blob_stores".to_owned()));
    assert!(!narrow_tools.contains(&"list_tasks".to_owned()));
    assert!(!narrow_tools.contains(&"create_remote_blob_store".to_owned()));
}

#[tokio::test]
async fn artifacts_are_found_listed_and_read() {
    let fixture = ready().await;
    maven_hosted(&fixture, "releases").await;
    for version in ["1.2.0", "1.10.0", "2.0.0-rc.1"] {
        deploy(&fixture, "releases", "org.example", "lib", version).await;
    }
    raw_hosted(&fixture, "files").await;
    put(
        &fixture,
        "/repository/files/docs/readme.txt",
        "hello from astrum\n",
    )
    .await;
    put(
        &fixture,
        "/repository/files/bin/tool.bin",
        vec![0u8, 1, 2, 3],
    )
    .await;
    put(
        &fixture,
        "/repository/files/docs/long.txt",
        "x".repeat(5000),
    )
    .await;
    let token = token(&fixture, "READ").await;

    let status = ok(&fixture, &token, "get_status", json!({})).await;
    assert_eq!(status["readable"], true);
    assert_eq!(status["writable"], true);
    assert_eq!(status["user"], "admin");
    assert_eq!(status["tokenScope"], "read");
    assert_eq!(status["blobStoresNeedingAttention"], json!([]));

    let repositories = ok(
        &fixture,
        &token,
        "list_repositories",
        json!({"format": "maven2"}),
    )
    .await;
    let listed: Vec<&str> = repositories["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|repository| repository["name"].as_str().unwrap())
        .collect();
    assert_eq!(listed, ["releases"]);
    let settings = ok(
        &fixture,
        &token,
        "get_repository",
        json!({"name": "releases"}),
    )
    .await;
    assert_eq!(settings["online"], true);
    assert_eq!(settings["storage"]["blobStoreName"], "default");

    let found = ok(
        &fixture,
        &token,
        "search_components",
        json!({"repository": "releases", "name": "lib"}),
    )
    .await;
    assert_eq!(found["count"], 3);
    let asset = &found["items"][0]["assets"][0];
    assert!(
        asset["downloadUrl"]
            .as_str()
            .unwrap()
            .contains("/repository/releases/")
    );
    assert_eq!(asset["sha256"].as_str().unwrap().len(), 64);
    assert!(asset.get("uploaderIp").is_none());

    let versions = ok(
        &fixture,
        &token,
        "list_versions",
        json!({"name": "lib", "group": "org.example"}),
    )
    .await;
    assert_eq!(versions["latest"], "1.10.0");
    let order: Vec<&str> = versions["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|version| version["version"].as_str().unwrap())
        .collect();
    assert_eq!(order, ["2.0.0-rc.1", "1.10.0", "1.2.0"]);
    assert_eq!(versions["incomplete"], false);

    let assets = ok(
        &fixture,
        &token,
        "search_assets",
        json!({"repository": "files", "q": "readme"}),
    )
    .await;
    assert_eq!(assets["count"], 1);
    assert_eq!(assets["items"][0]["path"], "/docs/readme.txt");

    let text = ok(
        &fixture,
        &token,
        "read_text_asset",
        json!({"repository": "files", "path": "/docs/readme.txt"}),
    )
    .await;
    assert_eq!(text["text"], "hello from astrum\n");
    assert_eq!(text["truncated"], false);
    let cut = ok(
        &fixture,
        &token,
        "read_text_asset",
        json!({"repository": "files", "path": "docs/long.txt", "maxBytes": 100}),
    )
    .await;
    assert_eq!(cut["text"].as_str().unwrap().len(), 100);
    assert_eq!(cut["truncated"], true);
    let binary = failed(
        &fixture,
        &token,
        "read_text_asset",
        json!({"repository": "files", "path": "/bin/tool.bin"}),
    )
    .await;
    assert!(binary.contains("not a text file"), "{binary}");
    let missing = failed(
        &fixture,
        &token,
        "read_text_asset",
        json!({"repository": "files", "path": "/nothing.txt"}),
    )
    .await;
    assert!(missing.contains("404"), "{missing}");
    let no_name = failed(&fixture, &token, "list_versions", json!({})).await;
    assert_eq!(no_name, "the argument name is required");

    let no_tags = ok(
        &fixture,
        &token,
        "list_docker_tags",
        json!({"repository": "files", "image": "app"}),
    )
    .await;
    assert_eq!(no_tags["tags"], json!([]));
}

#[tokio::test]
async fn blob_stores_tasks_and_repositories_are_managed() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    put(&fixture, "/repository/files/a.txt", "a").await;
    let token = token(&fixture, "WRITE").await;

    let stores = ok(&fixture, &token, "list_blob_stores", json!({})).await;
    let default = &stores["blobStores"][0];
    assert_eq!(default["name"], "default");
    assert_eq!(default["type"], "File");
    assert_eq!(default["repositories"], json!(["files"]));
    assert!(default["usedBytes"].as_u64().unwrap() >= 1);

    let group = ok(
        &fixture,
        &token,
        "convert_blob_store_to_group",
        json!({"name": "default", "newNameForOriginal": "default-disk"}),
    )
    .await;
    assert_eq!(group["members"], json!(["default-disk"]));
    let changed = ok(
        &fixture,
        &token,
        "update_group_blob_store",
        json!({"name": "default", "fillPolicy": "mostFreeSpace"}),
    )
    .await;
    assert_eq!(changed["fillPolicy"], "mostFreeSpace");
    assert_eq!(changed["members"], json!(["default-disk"]));
    let wrong = failed(
        &fixture,
        &token,
        "update_group_blob_store",
        json!({"name": "default", "fillPolicy": "biggestFirst"}),
    )
    .await;
    assert!(
        wrong.contains("400") && wrong.contains("mostFreeSpace"),
        "{wrong}"
    );
    let read_back = get(&fixture, "/repository/files/a.txt", Auth::Admin).await;
    assert_eq!(read_back.text(), "a");

    let refused = failed(
        &fixture,
        &token,
        "create_remote_blob_store",
        json!({"name": "far", "url": "ftp://x", "token": "short"}),
    )
    .await;
    assert!(
        refused.contains("url:") && refused.contains("token:"),
        "{refused}"
    );

    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({"type": "blobstore.compact", "name": "Compact default", "frequency": {"schedule": "manual"}, "properties": {"blobstoreName": "default"}})),
        )
        .await;
    assert!(created.status.is_success(), "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let tasks = ok(
        &fixture,
        &token,
        "list_tasks",
        json!({"type": "blobstore.compact"}),
    )
    .await;
    assert_eq!(tasks["items"][0]["id"], id.as_str());
    let task = ok(&fixture, &token, "get_task", json!({"id": id})).await;
    assert_eq!(task["name"], "Compact default");
    let started = ok(&fixture, &token, "run_task", json!({"id": id})).await;
    assert_eq!(started["done"], "run");
    let unknown = failed(&fixture, &token, "get_task", json!({"id": "nothing"})).await;
    assert!(unknown.contains("404"), "{unknown}");

    let offline = ok(
        &fixture,
        &token,
        "set_repository_online",
        json!({"name": "files", "online": false}),
    )
    .await;
    assert_eq!(offline["online"], false);
    let summary = get(
        &fixture,
        "/service/rest/v1/repositories/raw/hosted/files",
        Auth::Admin,
    )
    .await;
    assert_eq!(summary.json()["online"], false);
    let online = ok(
        &fixture,
        &token,
        "set_repository_online",
        json!({"name": "files", "online": true}),
    )
    .await;
    assert_eq!(online["online"], true);
}

#[tokio::test]
async fn prompts_guide_the_usual_jobs() {
    let fixture = ready().await;
    let token = token(&fixture, "READ").await;
    let listed = rpc(&fixture, &token, "prompts/list", json!({})).await;
    let names: Vec<&str> = listed["result"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|prompt| prompt["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["diagnose_space", "find_artifact"]);
    let found = rpc(
        &fixture,
        &token,
        "prompts/get",
        json!({"name": "find_artifact", "arguments": {"name": "phoenix-ui"}}),
    )
    .await;
    let text = found["result"]["messages"][0]["content"]["text"]
        .as_str()
        .unwrap();
    assert!(
        text.contains("\"phoenix-ui\"") && text.contains("list_versions"),
        "{text}"
    );
    let missing = rpc(
        &fixture,
        &token,
        "prompts/get",
        json!({"name": "find_artifact"}),
    )
    .await;
    assert_eq!(missing["error"]["code"], -32602);
}
