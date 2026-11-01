mod support;
mod ui_support;

use std::sync::Arc;

use astrum_db::entity::{audit_event, mcp_upload_ticket};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const MCP: &str = "/mcp";
const TOKENS: &str = "/service/rest/internal/access-tokens";
const BASE: &str = "http://astrum.example";

async fn ready_for_uploads() -> Fixture {
    let fixture = ready().await;
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    fixture
}

async fn token_of(fixture: &Fixture, auth: Auth<'_>, scope: &str) -> String {
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            auth,
            Some(json!({"name": format!("up-{scope}"), "scope": scope, "expiresInDays": 30})),
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

async fn call(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> Value {
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
    let result = call(fixture, token, name, arguments).await;
    assert_eq!(result["isError"], false, "{result}");
    result["structuredContent"].clone()
}

async fn failed(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> String {
    let result = call(fixture, token, name, arguments).await;
    assert_eq!(result["isError"], true, "{result}");
    result["content"][0]["text"].as_str().unwrap().to_owned()
}

async fn tool_names(fixture: &Fixture, token: &str) -> Vec<String> {
    rpc(fixture, token, "tools/list", json!({})).await["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect()
}

async fn admin_token(fixture: &Fixture) -> String {
    token_of(fixture, Auth::Admin, "WRITE").await
}

async fn user_token(fixture: &Fixture, user: &str) -> String {
    token_of(fixture, Auth::Basic(user, "password-1"), "WRITE").await
}

async fn issue(fixture: &Fixture, token: &str, arguments: Value) -> (String, Value) {
    let issued = ok(fixture, token, "request_upload", arguments).await;
    let url = issued["uploadUrl"].as_str().unwrap();
    let path = url.strip_prefix(BASE).unwrap().to_owned();
    (path, issued)
}

async fn put_file(
    fixture: &Fixture,
    path: &str,
    auth: Auth<'_>,
    body: impl Into<Vec<u8>>,
    headers: &[(&str, &str)],
) -> Reply {
    let mut all = vec![("host", "astrum.example")];
    all.extend_from_slice(headers);
    fixture
        .send_body(Method::PUT, path, auth, Body::from(body.into()), &all)
        .await
}

async fn download(fixture: &Fixture, path: &str) -> Reply {
    get(fixture, path, Auth::Admin).await
}

async fn tickets_held(fixture: &Fixture) -> u64 {
    mcp_upload_ticket::Entity::find()
        .count(fixture.state.db().conn())
        .await
        .unwrap()
}

async fn upload_events(fixture: &Fixture) -> Vec<audit_event::Model> {
    audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("repository.upload"))
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
}

async fn expire_tickets(fixture: &Fixture) {
    mcp_upload_ticket::Entity::update_many()
        .col_expr(
            mcp_upload_ticket::Column::ExpiresAt,
            Expr::cust("now() - interval '1 minute'"),
        )
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
}

fn staged_files(fixture: &Fixture) -> usize {
    std::fs::read_dir(fixture.data.path().join("tmp"))
        .unwrap()
        .count()
}

async fn raw_with_policy(fixture: &Fixture, name: &str, policy: &str) {
    create(
        fixture,
        "raw",
        "hosted",
        json!({
            "name": name,
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": policy},
            "raw": {"contentDisposition": "INLINE"}
        }),
    )
    .await;
}

fn tgz(name: &str, version: &str) -> Vec<u8> {
    let manifest = serde_json::to_vec(&json!({"name": name, "version": version})).unwrap();
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, "package/package.json", manifest.as_slice())
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

#[tokio::test]
async fn only_those_who_may_add_see_the_upload_tools() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    user_with(
        &fixture,
        "reader",
        &[
            "as-repository-view-raw-files-read",
            "as-repository-view-raw-files-browse",
        ],
    )
    .await;
    user_with(&fixture, "adder", &["as-repository-view-raw-files-add"]).await;

    let admin = admin_token(&fixture).await;
    let admin_tools = tool_names(&fixture, &admin).await;
    for name in ["request_upload", "upload_component"] {
        assert!(admin_tools.contains(&name.to_owned()), "{name}");
    }
    let read_only = token_of(&fixture, Auth::Admin, "READ").await;
    let read_tools = tool_names(&fixture, &read_only).await;
    assert!(!read_tools.contains(&"request_upload".to_owned()));
    assert!(!read_tools.contains(&"upload_component".to_owned()));
    let refused = rpc(
        &fixture,
        &read_only,
        "tools/call",
        json!({"name": "request_upload", "arguments": {"repository": "files", "fileName": "a"}}),
    )
    .await;
    assert_eq!(refused["error"]["code"], -32602);
    assert_eq!(
        refused["error"]["message"],
        "there is no tool named request_upload"
    );

    let reader = user_token(&fixture, "reader").await;
    let reader_tools = tool_names(&fixture, &reader).await;
    assert!(!reader_tools.contains(&"request_upload".to_owned()));
    assert!(!reader_tools.contains(&"upload_component".to_owned()));
    let adder = user_token(&fixture, "adder").await;
    let adder_tools = tool_names(&fixture, &adder).await;
    assert!(adder_tools.contains(&"request_upload".to_owned()));
    assert!(adder_tools.contains(&"upload_component".to_owned()));

    let anonymous = fixture
        .call(Method::POST, MCP, Auth::None, Some(json!({})))
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    for path in [
        "/mcp/upload/garbage",
        "/mcp/upload/",
        "/mcp/upload/astrum_up_",
    ] {
        let reply = put_file(&fixture, path, Auth::None, b"x".to_vec(), &[]).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
    }
    assert_eq!(tickets_held(&fixture).await, 0);
}

#[tokio::test]
async fn a_raw_file_goes_in_through_a_ticket_and_the_audit_says_via_mcp() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;

    let (path, issued) = issue(
        &fixture,
        &token,
        json!({"repository": "files", "fileName": "hello.txt", "fields": {"directory": "/docs"}}),
    )
    .await;
    assert_eq!(issued["method"], "PUT");
    assert_eq!(issued["singleUse"], true);
    assert_eq!(issued["expiresInSeconds"], 300);
    assert_eq!(issued["maxBytes"], 512 * 1024 * 1024);
    assert_eq!(issued["format"], "raw");
    assert_eq!(
        issued["fields"],
        json!({"raw.directory": "/docs", "raw.asset1.filename": "hello.txt"})
    );
    assert_eq!(issued["fieldsNotSet"], json!([]));
    assert!(!issued["next"].as_str().unwrap().contains("also asks for"));
    assert!(path.starts_with("/mcp/upload/astrum_up_"), "{path}");
    assert!(issued["curl"].as_str().unwrap().starts_with(
        "curl -sS -w '\\nHTTP %{http_code}\\n' -T <path-to-file> 'http://astrum.example/mcp/upload/"
    ));

    let stored = put_file(&fixture, &path, Auth::None, "hello from the ticket\n", &[]).await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(
        stored.json(),
        json!({"repository": "files", "format": "raw", "fileName": "hello.txt", "size": 22})
    );
    let read = download(&fixture, "/repository/files/docs/hello.txt").await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(read.text(), "hello from the ticket\n");
    let found = ok(
        &fixture,
        &token,
        "search_assets",
        json!({"repository": "files", "q": "hello"}),
    )
    .await;
    assert_eq!(found["items"][0]["path"], "/docs/hello.txt");

    let events = upload_events(&fixture).await;
    let kinds: Vec<&str> = events.iter().map(|event| event.kind.as_str()).collect();
    assert_eq!(kinds, ["ticket-issued", "uploaded"]);
    for event in &events {
        assert_eq!(event.context, "files");
        assert_eq!(event.attributes["via"], "mcp");
        assert_eq!(event.attributes["client"], "access token");
        assert!(event.initiator.starts_with("admin"), "{}", event.initiator);
        assert!(!event.attributes.to_string().contains("astrum_up_"));
    }
    assert_eq!(events[1].attributes["size"], 22);
    assert_eq!(staged_files(&fixture), 0);
}

#[tokio::test]
async fn npm_packages_plain_and_scoped_go_in_through_tickets() {
    let fixture = ready_for_uploads().await;
    create(
        &fixture,
        "npm",
        "hosted",
        json!({
            "name": "npm-hosted",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"}
        }),
    )
    .await;
    let token = admin_token(&fixture).await;

    for (name, file) in [
        ("left-padder", "left-padder-1.0.0.tgz"),
        ("@acme/widgets", "acme-widgets-1.0.0.tgz"),
    ] {
        let (path, issued) = issue(
            &fixture,
            &token,
            json!({"repository": "npm-hosted", "fileName": file}),
        )
        .await;
        assert_eq!(issued["fields"], json!({}));
        let stored = put_file(&fixture, &path, Auth::None, tgz(name, "1.0.0"), &[]).await;
        assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    }
    let plain = ok(
        &fixture,
        &token,
        "list_versions",
        json!({"name": "left-padder", "repository": "npm-hosted"}),
    )
    .await;
    assert_eq!(plain["latest"], "1.0.0");
    let scoped = ok(
        &fixture,
        &token,
        "list_versions",
        json!({"name": "widgets", "group": "acme", "repository": "npm-hosted"}),
    )
    .await;
    assert_eq!(scoped["latest"], "1.0.0");

    let (path, _) = issue(
        &fixture,
        &token,
        json!({"repository": "npm-hosted", "fileName": "broken.tgz"}),
    )
    .await;
    let broken = put_file(&fixture, &path, Auth::None, "not a tarball", &[]).await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    assert!(broken.text().contains("gzip"), "{}", broken.text());
    assert_eq!(staged_files(&fixture), 0);
}

#[tokio::test]
async fn a_ticket_expires_and_works_once() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;

    let (late, _) = issue(
        &fixture,
        &token,
        json!({"repository": "files", "fileName": "late.txt", "fields": {"directory": "/t"}}),
    )
    .await;
    expire_tickets(&fixture).await;
    let expired = put_file(&fixture, &late, Auth::None, "late", &[]).await;
    assert_eq!(expired.status, StatusCode::NOT_FOUND);
    assert_eq!(
        download(&fixture, "/repository/files/t/late.txt")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let (once, _) = issue(
        &fixture,
        &token,
        json!({"repository": "files", "fileName": "once.txt", "fields": {"directory": "/t"}}),
    )
    .await;
    let first = put_file(&fixture, &once, Auth::None, "first", &[]).await;
    assert_eq!(first.status, StatusCode::CREATED);
    let second = put_file(&fixture, &once, Auth::None, "second", &[]).await;
    assert_eq!(second.status, StatusCode::NOT_FOUND);
    assert_eq!(
        second.json()["message"],
        "The upload ticket is unknown, expired or already used."
    );
    assert_eq!(
        download(&fixture, "/repository/files/t/once.txt")
            .await
            .text(),
        "first"
    );
}

#[tokio::test]
async fn a_ticket_is_bound_to_its_repository_its_user_and_its_rights() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "other").await;
    user_with(&fixture, "adder", &["as-repository-view-raw-files-add"]).await;
    let adder = user_token(&fixture, "adder").await;

    let (path, _) = issue(
        &fixture,
        &adder,
        json!({"repository": "files", "fileName": "bound.txt", "fields": {"directory": "/only-here"}}),
    )
    .await;
    let ticket = path.rsplit('/').next().unwrap();
    let mut altered = ticket.to_owned();
    altered.pop();
    altered.push(if ticket.ends_with('a') { 'b' } else { 'a' });
    let wrong = put_file(
        &fixture,
        &format!("/mcp/upload/{altered}"),
        Auth::None,
        "x",
        &[],
    )
    .await;
    assert_eq!(wrong.status, StatusCode::NOT_FOUND);
    let longer = put_file(&fixture, &format!("{path}/more"), Auth::None, "x", &[]).await;
    assert_eq!(longer.status, StatusCode::NOT_FOUND);
    assert_eq!(tickets_held(&fixture).await, 1);

    let stored = put_file(&fixture, &path, Auth::Admin, "bound", &[]).await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(
        download(&fixture, "/repository/files/only-here/bound.txt")
            .await
            .text(),
        "bound"
    );
    assert_eq!(
        download(&fixture, "/repository/other/only-here/bound.txt")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let assets = get(
        &fixture,
        "/service/rest/v1/search/assets?repository=files&q=bound",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(assets["items"][0]["uploader"], "adder");
    let uploaded = upload_events(&fixture)
        .await
        .into_iter()
        .find(|event| event.kind == "uploaded")
        .unwrap();
    assert!(
        uploaded.initiator.starts_with("adder"),
        "{}",
        uploaded.initiator
    );

    let (revoked, _) = issue(
        &fixture,
        &adder,
        json!({"repository": "files", "fileName": "late.txt", "fields": {"directory": "/only-here"}}),
    )
    .await;
    let revocation = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/roles/adder-role",
            Auth::Admin,
            Some(json!({"id": "adder-role", "name": "adder-role", "description": "", "privileges": [], "roles": []})),
        )
        .await;
    assert!(revocation.status.is_success(), "{}", revocation.text());
    let refused = put_file(&fixture, &revoked, Auth::Admin, "late", &[]).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    assert_eq!(
        download(&fixture, "/repository/files/only-here/late.txt")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(staged_files(&fixture), 0);
}

#[tokio::test]
async fn a_ticket_holds_its_size_and_is_spent_by_a_refusal() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;

    for size in [json!(512 * 1024 * 1024 + 1), json!(0), json!(1.5)] {
        let text = failed(
            &fixture,
            &token,
            "request_upload",
            json!({"repository": "files", "fileName": "a.bin", "sizeBytes": size}),
        )
        .await;
        assert!(text.contains("sizeBytes"), "{text}");
    }
    assert_eq!(tickets_held(&fixture).await, 0);

    let arguments = json!({"repository": "files", "fileName": "ten.bin", "sizeBytes": 10, "fields": {"directory": "/s"}});
    let (streamed, issued) = issue(&fixture, &token, arguments.clone()).await;
    assert_eq!(issued["maxBytes"], 10);
    let too_big = put_file(&fixture, &streamed, Auth::None, vec![7u8; 11], &[]).await;
    assert_eq!(too_big.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        download(&fixture, "/repository/files/s/ten.bin")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let spent = put_file(&fixture, &streamed, Auth::None, vec![7u8; 5], &[]).await;
    assert_eq!(spent.status, StatusCode::NOT_FOUND);
    assert_eq!(staged_files(&fixture), 0);

    let (declared, _) = issue(&fixture, &token, arguments.clone()).await;
    let announced = put_file(
        &fixture,
        &declared,
        Auth::None,
        vec![7u8; 11],
        &[("content-length", "11")],
    )
    .await;
    assert_eq!(announced.status, StatusCode::PAYLOAD_TOO_LARGE);

    let (empty, _) = issue(&fixture, &token, arguments.clone()).await;
    let nothing = put_file(&fixture, &empty, Auth::None, Vec::new(), &[]).await;
    assert_eq!(nothing.status, StatusCode::BAD_REQUEST);
    assert_eq!(staged_files(&fixture), 0);
    assert_eq!(
        put_file(&fixture, &empty, Auth::None, vec![7u8; 5], &[])
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(staged_files(&fixture), 0);

    let (exact, _) = issue(&fixture, &token, arguments).await;
    let fits = put_file(&fixture, &exact, Auth::None, vec![7u8; 10], &[]).await;
    assert_eq!(fits.status, StatusCode::CREATED, "{}", fits.text());
}

#[tokio::test]
async fn the_write_policy_of_the_repository_holds() {
    let fixture = ready_for_uploads().await;
    raw_with_policy(&fixture, "once", "ALLOW_ONCE").await;
    let token = admin_token(&fixture).await;
    let arguments =
        json!({"repository": "once", "fileName": "x.txt", "fields": {"directory": "/p"}});

    let (first, _) = issue(&fixture, &token, arguments.clone()).await;
    assert_eq!(
        put_file(&fixture, &first, Auth::None, "one", &[])
            .await
            .status,
        StatusCode::CREATED
    );
    let (second, _) = issue(&fixture, &token, arguments).await;
    let again = put_file(&fixture, &second, Auth::None, "two", &[]).await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST, "{}", again.text());
    assert_eq!(
        download(&fixture, "/repository/once/p/x.txt").await.text(),
        "one"
    );
    assert_eq!(staged_files(&fixture), 0);
}

#[tokio::test]
async fn tickets_are_refused_where_nothing_can_be_added() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    raw_hosted(&fixture, "other").await;
    raw_proxy(&fixture, "mirror").await;
    raw_group(&fixture, "all", &["files"]).await;
    create(
        &fixture,
        "docker",
        "hosted",
        json!({
            "name": "images",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW", "latestPolicy": true},
            "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
        }),
    )
    .await;
    user_with(&fixture, "adder", &["as-repository-view-raw-files-add"]).await;
    let admin = admin_token(&fixture).await;
    let adder = user_token(&fixture, "adder").await;

    for (repository, wanted) in [
        ("mirror", "only hosted repositories take uploads"),
        ("all", "only hosted repositories take uploads"),
        ("images", "docker repositories do not take uploads"),
        ("missing", "Repository not found"),
    ] {
        let text = failed(
            &fixture,
            &admin,
            "request_upload",
            json!({"repository": repository, "fileName": "a.txt"}),
        )
        .await;
        assert!(text.contains(wanted), "{repository}: {text}");
    }
    let denied = failed(
        &fixture,
        &adder,
        "request_upload",
        json!({"repository": "other", "fileName": "a.txt"}),
    )
    .await;
    assert!(
        denied.contains("you may not add content to other"),
        "{denied}"
    );

    let wrong_field = failed(
        &fixture,
        &admin,
        "request_upload",
        json!({"repository": "files", "fileName": "a.txt", "fields": {"folder": "/x"}}),
    )
    .await;
    assert!(wrong_field.contains("directory, filename"), "{wrong_field}");
    let bad_name = failed(
        &fixture,
        &admin,
        "request_upload",
        json!({"repository": "files", "fileName": "../a.txt"}),
    )
    .await;
    assert!(bad_name.contains("fileName"), "{bad_name}");
    assert_eq!(tickets_held(&fixture).await, 0);
}

#[tokio::test]
async fn small_files_go_in_as_base64() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;
    let upload = |content: String| json!({"repository": "files", "fileName": "small.txt", "contentBase64": content, "fields": {"directory": "/inline"}});

    let stored = ok(
        &fixture,
        &token,
        "upload_component",
        upload(STANDARD.encode("small and sweet")),
    )
    .await;
    assert_eq!(
        stored,
        json!({"repository": "files", "format": "raw", "fileName": "small.txt", "size": 15})
    );
    assert_eq!(
        download(&fixture, "/repository/files/inline/small.txt")
            .await
            .text(),
        "small and sweet"
    );
    let events = upload_events(&fixture).await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "uploaded");
    assert_eq!(events[0].attributes["via"], "mcp");

    let big = failed(
        &fixture,
        &token,
        "upload_component",
        upload(STANDARD.encode(vec![1u8; 64 * 1024 + 1])),
    )
    .await;
    assert!(big.contains("request_upload"), "{big}");
    let edge = ok(
        &fixture,
        &token,
        "upload_component",
        upload(STANDARD.encode(vec![1u8; 64 * 1024])),
    )
    .await;
    assert_eq!(edge["size"], 64 * 1024);
    for bad in ["not base64!", "c21hbGw=\n", ""] {
        let text = failed(&fixture, &token, "upload_component", upload(bad.to_owned())).await;
        assert!(text.contains("contentBase64"), "{bad:?}: {text}");
    }
    assert_eq!(staged_files(&fixture), 0);
}

#[tokio::test]
async fn open_tickets_and_attempts_are_limited() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;
    let arguments = json!({"repository": "files", "fileName": "a.txt"});

    for _ in 0..20 {
        issue(&fixture, &token, arguments.clone()).await;
    }
    let text = failed(&fixture, &token, "request_upload", arguments.clone()).await;
    assert!(text.contains("20 open upload tickets"), "{text}");
    assert_eq!(tickets_held(&fixture).await, 20);
    expire_tickets(&fixture).await;
    issue(&fixture, &token, arguments).await;
    assert_eq!(tickets_held(&fixture).await, 1);

    let mut replies = Vec::new();
    for _ in 0..60 {
        replies.push(put_file(&fixture, "/mcp/upload/garbage", Auth::None, "x", &[]).await);
    }
    let limited: Vec<&Reply> = replies
        .iter()
        .filter(|reply| reply.status == StatusCode::TOO_MANY_REQUESTS)
        .collect();
    assert!(!limited.is_empty());
    for reply in &limited {
        assert!(reply.headers.contains_key(header::RETRY_AFTER));
    }
    assert!(
        replies
            .iter()
            .filter(|reply| reply.status != StatusCode::TOO_MANY_REQUESTS)
            .all(|reply| reply.status == StatusCode::NOT_FOUND)
    );
}

#[tokio::test]
async fn request_upload_names_the_required_fields_left_unset() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;

    let (path, issued) = issue(
        &fixture,
        &token,
        json!({"repository": "files", "fileName": "bare.txt"}),
    )
    .await;
    assert_eq!(issued["fieldsNotSet"], json!(["directory"]));
    let next = issued["next"].as_str().unwrap();
    assert!(next.contains("also asks for: directory."), "{next}");
    assert!(next.contains("HTTP 201"), "{next}");
    assert_eq!(tickets_held(&fixture).await, 1);
    let stored = put_file(&fixture, &path, Auth::None, "bare", &[]).await;
    assert_ne!(stored.status, StatusCode::NOT_FOUND, "{}", stored.text());
}

#[tokio::test]
async fn two_calls_with_one_ticket_store_exactly_one_file() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;
    let (path, _) = issue(
        &fixture,
        &token,
        json!({"repository": "files", "fileName": "race.txt", "fields": {"directory": "/r"}}),
    )
    .await;

    let (first, second) = tokio::join!(
        put_file(&fixture, &path, Auth::None, "first body", &[]),
        put_file(&fixture, &path, Auth::None, "second body", &[]),
    );
    let mut statuses = [first.status, second.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::CREATED, StatusCode::NOT_FOUND]);
    let winner = if first.status == StatusCode::CREATED {
        "first body"
    } else {
        "second body"
    };
    let read = download(&fixture, "/repository/files/r/race.txt").await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(read.text(), winner);
    assert_eq!(tickets_held(&fixture).await, 0);
    assert_eq!(staged_files(&fixture), 0);
}

#[tokio::test]
async fn a_broken_authorization_header_does_not_spend_the_ticket() {
    let fixture = ready_for_uploads().await;
    raw_hosted(&fixture, "files").await;
    let token = admin_token(&fixture).await;
    let (path, _) = issue(
        &fixture,
        &token,
        json!({"repository": "files", "fileName": "auth.txt", "fields": {"directory": "/a"}}),
    )
    .await;

    let refused = put_file(
        &fixture,
        &path,
        Auth::None,
        "nope",
        &[(
            "authorization",
            "Bearer astrum_0000000000000000000000000000000000000000",
        )],
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        refused.text()
    );
    assert_eq!(tickets_held(&fixture).await, 1);
    let stored = put_file(&fixture, &path, Auth::None, "fine", &[]).await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(
        download(&fixture, "/repository/files/a/auth.txt")
            .await
            .text(),
        "fine"
    );
}
