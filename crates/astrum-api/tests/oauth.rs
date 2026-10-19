mod support;
mod ui_support;

use astrum_db::entity::audit_event;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sea_orm::{EntityTrait, QueryOrder};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const HOST: &str = "astrum.example";
const BASE: &str = "http://astrum.example";
const CALLBACK: &str = "http://127.0.0.1:33418/callback";
const VERIFIER: &str = "a-code-verifier-that-is-long-enough-for-pkce-0123456789";
const UI: &str = "/service/rest/internal/ui/oauth";

fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

async fn anonymous(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    fixture
        .send(method, path, Auth::None, body, &[("host", HOST)])
        .await
}

async fn form(fixture: &Fixture, path: &str, pairs: &[(&str, &str)]) -> Reply {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    fixture
        .send_body(
            Method::POST,
            path,
            Auth::None,
            Body::from(body),
            &[
                ("host", HOST),
                ("content-type", "application/x-www-form-urlencoded"),
            ],
        )
        .await
}

async fn register(fixture: &Fixture, name: &str) -> String {
    let reply = anonymous(
        fixture,
        Method::POST,
        "/oauth/register",
        Some(json!({"client_name": name, "redirect_uris": [CALLBACK], "token_endpoint_auth_method": "none"})),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["client_id"].as_str().unwrap().to_owned()
}

fn location(reply: &Reply) -> String {
    assert!(
        reply.status.is_redirection(),
        "{} {}",
        reply.status,
        reply.text()
    );
    reply.headers[header::LOCATION].to_str().unwrap().to_owned()
}

fn query_of(location: &str) -> Vec<(String, String)> {
    url::Url::parse(location)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

fn param(location: &str, name: &str) -> Option<String> {
    query_of(location)
        .into_iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
}

async fn begin(fixture: &Fixture, client: &str, scope: &str) -> String {
    let uri = format!(
        "/oauth/authorize?response_type=code&client_id={client}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state=xyz&scope={}&resource={}",
        urlencoding(CALLBACK),
        challenge(VERIFIER),
        urlencoding(scope),
        urlencoding(&format!("{BASE}/mcp")),
    );
    let reply = anonymous(fixture, Method::GET, &uri, None).await;
    let target = location(&reply);
    assert!(
        target.starts_with(&format!("{BASE}/user/authorize?request=")),
        "{target}"
    );
    param(&target, "request").unwrap()
}

fn urlencoding(text: &str) -> String {
    url::form_urlencoded::byte_serialize(text.as_bytes()).collect()
}

async fn consent(fixture: &Fixture, request: &str, write: bool) -> String {
    let reply = fixture
        .call(
            Method::POST,
            &format!("{UI}/requests/{request}/approve"),
            Auth::Admin,
            Some(json!({"write": write})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let redirect = reply.json()["redirect"].as_str().unwrap().to_owned();
    assert!(redirect.starts_with(CALLBACK), "{redirect}");
    assert_eq!(param(&redirect, "state").as_deref(), Some("xyz"));
    param(&redirect, "code").unwrap()
}

async fn exchange(fixture: &Fixture, client: &str, code: &str) -> Value {
    let reply = form(
        fixture,
        "/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", CALLBACK),
            ("client_id", client),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.headers[header::CACHE_CONTROL], "no-store");
    reply.json()
}

async fn connect(fixture: &Fixture, client: &str, write: bool) -> Value {
    let request = begin(fixture, client, "read write").await;
    let code = consent(fixture, &request, write).await;
    exchange(fixture, client, &code).await
}

async fn mcp(fixture: &Fixture, token: &str, method: &str, params: Value) -> Reply {
    fixture
        .send(
            Method::POST,
            "/mcp",
            Auth::None,
            Some(json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})),
            &[
                ("host", HOST),
                ("authorization", &format!("Bearer {token}")),
            ],
        )
        .await
}

fn tool_names(reply: &Reply) -> Vec<String> {
    reply.json()["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn the_server_describes_itself_for_oauth_clients() {
    let fixture = ready().await;
    let resource = anonymous(
        &fixture,
        Method::GET,
        "/.well-known/oauth-protected-resource/mcp",
        None,
    )
    .await;
    assert_eq!(resource.status, StatusCode::OK);
    assert_eq!(resource.json()["resource"], format!("{BASE}/mcp"));
    assert_eq!(resource.json()["authorization_servers"], json!([BASE]));
    let server = anonymous(
        &fixture,
        Method::GET,
        "/.well-known/oauth-authorization-server",
        None,
    )
    .await
    .json();
    assert_eq!(server["issuer"], BASE);
    assert_eq!(server["token_endpoint"], format!("{BASE}/oauth/token"));
    assert_eq!(
        server["registration_endpoint"],
        format!("{BASE}/oauth/register")
    );
    assert_eq!(server["code_challenge_methods_supported"], json!(["S256"]));
    assert_eq!(
        server["token_endpoint_auth_methods_supported"],
        json!(["none"])
    );

    let challenged = anonymous(
        &fixture,
        Method::POST,
        "/mcp",
        Some(json!({"jsonrpc": "2.0", "id": 1, "method": "ping"})),
    )
    .await;
    assert_eq!(challenged.status, StatusCode::UNAUTHORIZED);
    let value = challenged.headers[header::WWW_AUTHENTICATE]
        .to_str()
        .unwrap();
    assert!(
        value.contains(&format!(
            "resource_metadata=\"{BASE}/.well-known/oauth-protected-resource/mcp\""
        )),
        "{value}"
    );
    assert!(!value.contains("invalid_token"), "{value}");
}

#[tokio::test]
async fn clients_register_with_safe_redirects_only() {
    let fixture = ready().await;
    let with_basic = fixture
        .send(
            Method::POST,
            "/oauth/register",
            Auth::Basic("someone", "wrong"),
            Some(json!({"redirect_uris": ["https://claude.ai/api/mcp/auth_callback"], "client_name": "Claude"})),
            &[("host", HOST)],
        )
        .await;
    assert_eq!(
        with_basic.status,
        StatusCode::CREATED,
        "{}",
        with_basic.text()
    );
    let created = with_basic.json();
    assert!(
        created["client_id"]
            .as_str()
            .unwrap()
            .starts_with("astrum-mcp-")
    );
    assert_eq!(created["token_endpoint_auth_method"], "none");
    assert_eq!(created["client_name"], "Claude");

    for (body, code) in [
        (json!({"client_name": "x"}), "invalid_redirect_uri"),
        (
            json!({"redirect_uris": ["http://evil.example/cb"]}),
            "invalid_redirect_uri",
        ),
        (
            json!({"redirect_uris": ["javascript:alert(1)"]}),
            "invalid_redirect_uri",
        ),
        (
            json!({"redirect_uris": [CALLBACK], "grant_types": ["client_credentials"]}),
            "invalid_client_metadata",
        ),
        (
            json!({"redirect_uris": [CALLBACK], "client_name": "x".repeat(101)}),
            "invalid_client_metadata",
        ),
    ] {
        let reply = anonymous(&fixture, Method::POST, "/oauth/register", Some(body)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        assert_eq!(reply.json()["error"], code, "{}", reply.text());
    }
}

#[tokio::test]
async fn an_agent_connects_works_and_is_cut_off() {
    let fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    let client = register(&fixture, "Claude Code").await;

    let request = begin(&fixture, &client, "read write").await;
    let shown = fixture
        .call(
            Method::GET,
            &format!("{UI}/requests/{request}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(shown.status, StatusCode::OK, "{}", shown.text());
    let shown = shown.json();
    assert_eq!(shown["clientName"], "Claude Code");
    assert_eq!(shown["redirectHost"], "127.0.0.1");
    assert_eq!(shown["writeRequested"], true);
    assert_eq!(shown["user"], "admin");
    let unsigned = fixture
        .call(
            Method::GET,
            &format!("{UI}/requests/{request}"),
            Auth::None,
            None,
        )
        .await;
    assert_eq!(unsigned.status, StatusCode::UNAUTHORIZED);

    let code = consent(&fixture, &request, true).await;
    let gone = fixture
        .call(
            Method::GET,
            &format!("{UI}/requests/{request}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let wrong_verifier = form(
        &fixture,
        "/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", CALLBACK),
            ("client_id", &client),
            (
                "code_verifier",
                "another-verifier-that-is-long-enough-for-pkce-012345",
            ),
        ],
    )
    .await;
    assert_eq!(wrong_verifier.status, StatusCode::BAD_REQUEST);
    assert_eq!(wrong_verifier.json()["error"], "invalid_grant");
    let reused = form(
        &fixture,
        "/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", CALLBACK),
            ("client_id", &client),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(reused.json()["error"], "invalid_grant", "{}", reused.text());

    let tokens = connect(&fixture, &client, true).await;
    assert_eq!(tokens["token_type"], "Bearer");
    assert_eq!(tokens["expires_in"], 3600);
    assert_eq!(tokens["scope"], "read write");
    let access = tokens["access_token"].as_str().unwrap().to_owned();
    let refresh = tokens["refresh_token"].as_str().unwrap().to_owned();
    assert!(access.starts_with("astrum_at_"), "{access}");

    let listed = mcp(&fixture, &access, "tools/list", json!({})).await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    assert!(tool_names(&listed).contains(&"set_repository_online".to_owned()));
    let offline = mcp(
        &fixture,
        &access,
        "tools/call",
        json!({"name": "set_repository_online", "arguments": {"name": "files", "online": false}}),
    )
    .await;
    assert_eq!(
        offline.json()["result"]["isError"],
        false,
        "{}",
        offline.text()
    );
    let events = audit_event::Entity::find()
        .order_by_desc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    let changed = events
        .iter()
        .find(|event| event.domain == "repository")
        .expect("the change is audited");
    assert_eq!(changed.attributes["via"], "mcp", "{:?}", changed.attributes);
    assert_eq!(changed.attributes["client"], "Claude Code");
    assert!(
        changed.initiator.starts_with("admin"),
        "{}",
        changed.initiator
    );
    let authorized = events
        .iter()
        .find(|event| event.domain == "oauth" && event.kind == "authorized")
        .expect("the consent is audited");
    assert_eq!(authorized.context, "Claude Code");

    let rest = fixture
        .send(
            Method::GET,
            "/service/rest/v1/repositories",
            Auth::None,
            None,
            &[
                ("host", HOST),
                ("authorization", &format!("Bearer {access}")),
            ],
        )
        .await;
    assert_eq!(rest.status, StatusCode::UNAUTHORIZED);

    let refreshed = form(
        &fixture,
        "/oauth/token",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
            ("client_id", &client),
        ],
    )
    .await;
    assert_eq!(refreshed.status, StatusCode::OK, "{}", refreshed.text());
    let fresh = refreshed.json()["access_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let stale = mcp(&fixture, &access, "ping", json!({})).await;
    assert_eq!(stale.status, StatusCode::UNAUTHORIZED);
    assert!(
        stale.headers[header::WWW_AUTHENTICATE]
            .to_str()
            .unwrap()
            .contains("error=\"invalid_token\"")
    );
    assert_eq!(
        mcp(&fixture, &fresh, "ping", json!({})).await.status,
        StatusCode::OK
    );
    let replayed = form(
        &fixture,
        "/oauth/token",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
            ("client_id", &client),
        ],
    )
    .await;
    assert_eq!(replayed.json()["error"], "invalid_grant");

    let grants = fixture
        .call(Method::GET, &format!("{UI}/grants"), Auth::Admin, None)
        .await
        .json();
    let grants = grants.as_array().unwrap();
    assert_eq!(grants.len(), 1, "{grants:?}");
    assert_eq!(grants[0]["clientName"], "Claude Code");
    assert_eq!(grants[0]["scope"], "WRITE");
    let id = grants[0]["id"].as_str().unwrap();
    let revoked = fixture
        .call(
            Method::DELETE,
            &format!("{UI}/grants/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    assert_eq!(
        mcp(&fixture, &fresh, "ping", json!({})).await.status,
        StatusCode::UNAUTHORIZED
    );

    let another = connect(&fixture, &client, false).await;
    let token = another["access_token"].as_str().unwrap().to_owned();
    assert_eq!(another["scope"], "read");
    let reading = tool_names(&mcp(&fixture, &token, "tools/list", json!({})).await);
    assert!(
        !reading.contains(&"set_repository_online".to_owned()),
        "{reading:?}"
    );
    let out = form(&fixture, "/oauth/revoke", &[("token", &token)]).await;
    assert_eq!(out.status, StatusCode::OK);
    assert_eq!(
        mcp(&fixture, &token, "ping", json!({})).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn broken_authorization_requests_go_to_the_right_place() {
    let fixture = ready().await;
    let client = register(&fixture, "Claude Code").await;
    let unknown = anonymous(
        &fixture,
        Method::GET,
        "/oauth/authorize?response_type=code&client_id=nobody",
        None,
    )
    .await;
    assert_eq!(
        location(&unknown),
        format!("{BASE}/user/authorize?problem=client")
    );
    let elsewhere = anonymous(
        &fixture,
        Method::GET,
        &format!(
            "/oauth/authorize?response_type=code&client_id={client}&redirect_uri={}",
            urlencoding("https://evil.example/cb")
        ),
        None,
    )
    .await;
    assert_eq!(
        location(&elsewhere),
        format!("{BASE}/user/authorize?problem=redirect")
    );
    let other_port = format!(
        "/oauth/authorize?response_type=code&client_id={client}&redirect_uri={}&state=s",
        urlencoding("http://127.0.0.1:5555/callback")
    );
    let no_pkce = location(&anonymous(&fixture, Method::GET, &other_port, None).await);
    assert!(
        no_pkce.starts_with("http://127.0.0.1:5555/callback?"),
        "{no_pkce}"
    );
    assert_eq!(param(&no_pkce, "error").as_deref(), Some("invalid_request"));
    assert_eq!(param(&no_pkce, "state").as_deref(), Some("s"));
    let plain = location(
        &anonymous(
            &fixture,
            Method::GET,
            &format!("{other_port}&code_challenge=abc&code_challenge_method=plain"),
            None,
        )
        .await,
    );
    assert_eq!(param(&plain, "error").as_deref(), Some("invalid_request"));
    let scope = location(
        &anonymous(
            &fixture,
            Method::GET,
            &format!("{other_port}&code_challenge=abc&code_challenge_method=S256&scope=admin"),
            None,
        )
        .await,
    );
    assert_eq!(param(&scope, "error").as_deref(), Some("invalid_scope"));
    let target = location(
        &anonymous(
            &fixture,
            Method::GET,
            &format!(
                "{other_port}&code_challenge=abc&code_challenge_method=S256&resource={}",
                urlencoding("https://other.example/mcp")
            ),
            None,
        )
        .await,
    );
    assert_eq!(param(&target, "error").as_deref(), Some("invalid_target"));

    let request = begin(&fixture, &client, "read").await;
    let denied = fixture
        .call(
            Method::POST,
            &format!("{UI}/requests/{request}/deny"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(denied.status, StatusCode::OK, "{}", denied.text());
    let redirect = denied.json()["redirect"].as_str().unwrap().to_owned();
    assert_eq!(param(&redirect, "error").as_deref(), Some("access_denied"));

    let no_client = form(
        &fixture,
        "/oauth/token",
        &[("grant_type", "authorization_code")],
    )
    .await;
    assert_eq!(no_client.status, StatusCode::UNAUTHORIZED);
    assert_eq!(no_client.json()["error"], "invalid_client");
    let grant = form(
        &fixture,
        "/oauth/token",
        &[("grant_type", "password"), ("client_id", &client)],
    )
    .await;
    assert_eq!(grant.json()["error"], "unsupported_grant_type");
}

#[tokio::test]
async fn agents_and_registrations_are_rate_limited() {
    let fixture = ready().await;
    let mut limited = false;
    for index in 0..12 {
        let reply = anonymous(
            &fixture,
            Method::POST,
            "/oauth/register",
            Some(json!({"client_name": format!("c{index}"), "redirect_uris": [CALLBACK]})),
        )
        .await;
        if reply.status == StatusCode::TOO_MANY_REQUESTS {
            assert_eq!(reply.json()["error"], "slow_down");
            limited = true;
            break;
        }
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    assert!(limited, "registrations were never limited");

    let mut refused = None;
    for _ in 0..200 {
        let reply = fixture
            .send(
                Method::POST,
                "/mcp",
                Auth::Admin,
                Some(json!({"jsonrpc": "2.0", "id": 1, "method": "ping"})),
                &[("host", HOST)],
            )
            .await;
        if reply.status == StatusCode::TOO_MANY_REQUESTS {
            refused = Some(reply);
            break;
        }
        assert_eq!(reply.status, StatusCode::OK);
    }
    let refused = refused.expect("agent calls were never limited");
    assert!(refused.headers.contains_key(header::RETRY_AFTER));
}
