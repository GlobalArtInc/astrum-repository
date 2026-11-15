mod accounts;
mod support;
mod ui_support;

use std::net::SocketAddr;

use accounts::audit;
use astrum_api::{STATUS, STATUS_WRITABLE};
use astrum_db::setting;
use astrum_format_docker::Connector;
use astrum_security::ip_allow::{IP_ALLOW_KEY, IpAllowSettings};
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;
use ui_support::{create, raw_hosted, ready};

const MESSAGE: &str = "Access from this address is not allowed";
const REPOSITORIES: &str = "/service/rest/v1/repositories";
const TOKENS: &str = "/service/rest/internal/access-tokens";
const OFFICE: &str = "10.1.1.1";
const CI: &str = "192.0.2.9";
const CLOUD: &str = "198.51.100.7";
const OUTSIDE: &str = "203.0.113.5";

fn owned(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|entry| (*entry).to_owned()).collect()
}

async fn store(fixture: &Fixture, management: &[&str], content: &[&str], agents: &[&str]) {
    let settings = IpAllowSettings {
        management: owned(management),
        content: owned(content),
        agents: owned(agents),
    };
    setting::put(fixture.state.db().conn(), IP_ALLOW_KEY, &settings)
        .await
        .unwrap();
    fixture.state.security().changed().await.unwrap();
}

async fn get(fixture: &mut Fixture, peer: &str, path: &str, auth: Auth<'_>) -> Reply {
    sent(fixture, peer, Method::GET, path, auth, &[]).await
}

async fn sent(
    fixture: &mut Fixture,
    peer: &str,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    headers: &[(&str, &str)],
) -> Reply {
    fixture.peer = Some(peer.parse().unwrap());
    fixture.send(method, path, auth, None, headers).await
}

async fn put_file(fixture: &mut Fixture, peer: &str, path: &str) -> Reply {
    fixture.peer = Some(peer.parse().unwrap());
    fixture
        .send_body(Method::PUT, path, Auth::Admin, Body::from("hello"), &[])
        .await
}

async fn agent_token(fixture: &Fixture) -> String {
    let reply = fixture
        .call(
            Method::POST,
            TOKENS,
            Auth::Admin,
            Some(json!({"name": "agent", "scope": "READ", "expiresInDays": 30})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["token"].as_str().unwrap().to_owned()
}

async fn mcp(fixture: &mut Fixture, peer: &str, token: &str, body: Value) -> Reply {
    fixture.peer = Some(peer.parse().unwrap());
    let bearer = format!("Bearer {token}");
    fixture
        .send(
            Method::POST,
            "/mcp",
            Auth::None,
            Some(body),
            &[("authorization", &bearer), ("host", "astrum.example")],
        )
        .await
}

fn refused(reply: &Reply) {
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    assert_eq!(reply.json(), json!({"message": MESSAGE}));
    assert!(!reply.headers.contains_key(header::WWW_AUTHENTICATE));
}

#[tokio::test]
async fn each_surface_admits_its_own_addresses_and_refuses_the_rest() {
    let mut fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    let token = agent_token(&fixture).await;
    for peer in [OFFICE, OUTSIDE] {
        let open = get(&mut fixture, peer, REPOSITORIES, Auth::Admin).await;
        assert_eq!(open.status, StatusCode::OK, "an empty list allows {peer}");
    }
    store(
        &fixture,
        &["10.0.0.0/8"],
        &["192.0.2.0/24"],
        &["198.51.100.0/24"],
    )
    .await;

    let allowed = get(&mut fixture, OFFICE, REPOSITORIES, Auth::Admin).await;
    assert_eq!(allowed.status, StatusCode::OK);
    for peer in [CI, CLOUD, OUTSIDE] {
        refused(&get(&mut fixture, peer, REPOSITORIES, Auth::Admin).await);
    }
    let reply = get(&mut fixture, OUTSIDE, REPOSITORIES, Auth::Admin).await;
    assert!(!reply.text().contains("10.0.0.0"), "{}", reply.text());
    assert_eq!(
        get(&mut fixture, OFFICE, REPOSITORIES, Auth::None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    refused(&get(&mut fixture, OUTSIDE, REPOSITORIES, Auth::None).await);
    let wrong = || Auth::Basic("admin", "not the password");
    let unauthorized = get(&mut fixture, OFFICE, REPOSITORIES, wrong()).await;
    assert_eq!(unauthorized.status, StatusCode::UNAUTHORIZED);
    refused(&get(&mut fixture, OUTSIDE, REPOSITORIES, wrong()).await);
    let unknown = [(
        "authorization",
        "Bearer astrum_000000000000000000000000000000000000000",
    )];
    let from_office = sent(
        &mut fixture,
        OFFICE,
        Method::GET,
        REPOSITORIES,
        Auth::None,
        &unknown,
    )
    .await;
    assert_eq!(from_office.status, StatusCode::UNAUTHORIZED);
    let from_outside = sent(
        &mut fixture,
        OUTSIDE,
        Method::GET,
        REPOSITORIES,
        Auth::None,
        &unknown,
    )
    .await;
    refused(&from_outside);

    let stored = put_file(&mut fixture, CI, "/repository/files/a.txt").await;
    assert!(stored.status.is_success(), "{}", stored.text());
    refused(&put_file(&mut fixture, OFFICE, "/repository/files/b.txt").await);
    let download = get(&mut fixture, CI, "/repository/files/a.txt", Auth::Admin).await;
    assert_eq!(download.text(), "hello");
    refused(&get(&mut fixture, OFFICE, "/repository/files/a.txt", Auth::Admin).await);
    let legacy = get(
        &mut fixture,
        CI,
        "/content/repositories/files/a.txt",
        Auth::Admin,
    )
    .await;
    assert_eq!(legacy.text(), "hello");
    refused(
        &get(
            &mut fixture,
            OFFICE,
            "/content/repositories/files/a.txt",
            Auth::Admin,
        )
        .await,
    );
    let upload = "/service/rest/v1/components?repository=files";
    let from_ci = sent(&mut fixture, CI, Method::POST, upload, Auth::Admin, &[]).await;
    assert_ne!(from_ci.status, StatusCode::FORBIDDEN);
    refused(&sent(&mut fixture, OFFICE, Method::POST, upload, Auth::Admin, &[]).await);

    let ping = json!({"jsonrpc": "2.0", "id": 1, "method": "ping"});
    let from_cloud = mcp(&mut fixture, CLOUD, &token, ping.clone()).await;
    assert_eq!(from_cloud.status, StatusCode::OK, "{}", from_cloud.text());
    for peer in [OFFICE, CI, OUTSIDE] {
        refused(&mcp(&mut fixture, peer, &token, ping.clone()).await);
    }
    let metadata = "/.well-known/oauth-authorization-server";
    let host = [("host", "astrum.example")];
    let known = sent(
        &mut fixture,
        CLOUD,
        Method::GET,
        metadata,
        Auth::None,
        &host,
    )
    .await;
    assert_eq!(known.status, StatusCode::OK, "{}", known.text());
    refused(
        &sent(
            &mut fixture,
            OFFICE,
            Method::GET,
            metadata,
            Auth::None,
            &host,
        )
        .await,
    );
}

#[tokio::test]
async fn a_percent_encoded_legacy_path_is_judged_as_the_content_the_router_serves() {
    let mut fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    store(&fixture, &["10.0.0.0/8"], &["192.0.2.0/24"], &[]).await;
    let stored = put_file(&mut fixture, CI, "/repository/files/a.txt").await;
    assert!(stored.status.is_success(), "{}", stored.text());
    let encoded = "/service/local/%72epositories/files/a.txt";
    refused(&get(&mut fixture, OFFICE, encoded, Auth::Admin).await);
    refused(
        &put_file(
            &mut fixture,
            OFFICE,
            "/service/local/%72epositories/files/b.txt",
        )
        .await,
    );
    let served = get(&mut fixture, CI, encoded, Auth::Admin).await;
    assert_eq!(served.text(), "hello");
    let created = put_file(
        &mut fixture,
        CI,
        "/service/local/%72epositories/files/b.txt",
    )
    .await;
    assert!(created.status.is_success(), "{}", created.text());
}

#[tokio::test]
async fn the_status_endpoints_answer_everybody() {
    let mut fixture = Fixture::new().await;
    store(&fixture, &[CI], &[CI], &[CI]).await;
    assert_eq!(STATUS, astrum_security::ip_allow::STATUS);
    assert_eq!(STATUS_WRITABLE, astrum_security::ip_allow::STATUS_WRITABLE);
    for path in [STATUS, STATUS_WRITABLE] {
        let reply = get(&mut fixture, OUTSIDE, path, Auth::None).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
    }
    refused(&get(&mut fixture, OUTSIDE, REPOSITORIES, Auth::None).await);
}

#[tokio::test]
async fn a_domain_routed_repository_and_a_docker_connector_port_count_as_content() {
    let mut fixture = ready().await;
    create(
        &fixture,
        "raw",
        "hosted",
        json!({
            "name": "site",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
            "raw": {"contentDisposition": "INLINE"},
            "domain": {"host": "files.example.org"},
        }),
    )
    .await;
    let stored = put_file(&mut fixture, OFFICE, "/repository/site/a.txt").await;
    assert!(stored.status.is_success(), "{}", stored.text());
    store(&fixture, &[OFFICE], &[CI], &[]).await;

    let host = [("host", "files.example.org")];
    let from_ci = sent(&mut fixture, CI, Method::GET, "/a.txt", Auth::Admin, &host).await;
    assert_eq!(from_ci.text(), "hello");
    for peer in [OFFICE, OUTSIDE] {
        refused(
            &sent(
                &mut fixture,
                peer,
                Method::GET,
                "/a.txt",
                Auth::Admin,
                &host,
            )
            .await,
        );
    }

    fixture
        .state
        .content()
        .register(std::sync::Arc::new(astrum_format_docker::Docker::default()));
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_docker::routes())
        .layer(axum::Extension(Connector(18511)));
    for (peer, expected) in [(CI, false), (OFFICE, true), (OUTSIDE, true)] {
        let mut request = Request::builder().uri("/v2/").body(Body::empty()).unwrap();
        let peer: std::net::IpAddr = peer.parse().unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(peer, 40000)));
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        match expected {
            true => {
                assert_eq!(status, StatusCode::FORBIDDEN, "{peer}");
                assert_eq!(
                    serde_json::from_slice::<Value>(&body).unwrap(),
                    json!({"message": MESSAGE})
                );
            }
            false => assert_ne!(status, StatusCode::FORBIDDEN, "{peer}"),
        }
    }
}

#[tokio::test]
async fn the_judged_address_is_the_one_a_trusted_proxy_vouches_for() {
    let mut fixture = Fixture::new().await;
    fixture.trust(&["10.0.0.0/8"]);
    store(&fixture, &["203.0.113.7"], &[], &[]).await;
    let status =
        |peer: &'static str, forwarded: &'static str| (peer, [("x-forwarded-for", forwarded)]);
    for (peer, forwarded, allowed) in [
        ("198.51.100.9", "203.0.113.7", false),
        ("198.51.100.9", "", false),
        ("203.0.113.7", "198.51.100.1", true),
        ("10.0.0.1", "203.0.113.7", true),
        ("10.0.0.1", "198.51.100.1, 203.0.113.7", true),
        ("10.0.0.1", "203.0.113.7, 198.51.100.1", false),
        ("10.0.0.1", "", false),
    ] {
        let (peer, headers) = status(peer, forwarded);
        let headers: &[(&str, &str)] = match forwarded.is_empty() {
            true => &[],
            false => &headers,
        };
        let reply = sent(
            &mut fixture,
            peer,
            Method::GET,
            REPOSITORIES,
            Auth::Admin,
            headers,
        )
        .await;
        match allowed {
            true => assert_eq!(reply.status, StatusCode::OK, "{peer} {forwarded}"),
            false => refused(&reply),
        }
    }
    store(&fixture, &["10.0.0.0/8"], &[], &[]).await;
    let forged = [("x-forwarded-for", "198.51.100.1")];
    let reply = sent(
        &mut fixture,
        "10.0.0.1",
        Method::GET,
        REPOSITORIES,
        Auth::Admin,
        &forged,
    )
    .await;
    refused(&reply);
}

#[tokio::test]
async fn a_denial_is_audited_once_per_window_and_counted() {
    let mut fixture = Fixture::new().await;
    store(&fixture, &[OFFICE], &[OFFICE], &[]).await;
    for _ in 0..3 {
        refused(&get(&mut fixture, OUTSIDE, REPOSITORIES, Auth::Admin).await);
    }
    let events = audit(&fixture, "security.ipAllow").await;
    assert_eq!(events.len(), 1, "{events:?}");
    let (kind, context, initiator, attributes) = &events[0];
    assert_eq!((kind.as_str(), context.as_str()), ("denied", "management"));
    assert_eq!(initiator, &format!("*UNKNOWN/{OUTSIDE}"));
    assert_eq!(
        attributes,
        &json!({"address": OUTSIDE, "surface": "management", "path": "/service/rest"})
    );
    refused(&get(&mut fixture, OUTSIDE, "/repository/a/b", Auth::None).await);
    refused(&get(&mut fixture, CI, REPOSITORIES, Auth::None).await);
    assert_eq!(audit(&fixture, "security.ipAllow").await.len(), 3);
    let counted = astrum_api::metrics::render(&fixture.state, &Default::default()).await;
    assert!(
        counted.contains("astrum_ip_allow_denied_total{surface=\"management\"} 4"),
        "{counted}"
    );
    assert!(counted.contains("astrum_ip_allow_denied_total{surface=\"content\"} 1"));
    assert!(counted.contains("astrum_ip_allow_denied_total{surface=\"agents\"} 0"));
}

#[tokio::test]
async fn an_agent_call_may_use_management_routes_when_only_agents_admits_it() {
    let mut fixture = ready().await;
    raw_hosted(&fixture, "files").await;
    let token = agent_token(&fixture).await;
    store(&fixture, &["192.0.2.1"], &[], &[CLOUD]).await;
    refused(&get(&mut fixture, CLOUD, REPOSITORIES, Auth::Admin).await);
    let call = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": "list_repositories", "arguments": {}},
    });
    let reply = mcp(&mut fixture, CLOUD, &token, call).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let result = &reply.json()["result"];
    assert_eq!(result["isError"], false, "{result}");
    assert!(result.to_string().contains("files"), "{result}");
}
