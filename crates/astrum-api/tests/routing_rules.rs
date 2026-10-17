mod support;

use std::sync::Arc;

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{Method, StatusCode};
use axum::routing::get;
use serde_json::{Value, json};
use support::{Auth, Fixture};

const RULES: &str = "/service/rest/v1/routing-rules";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
    fixture
}

async fn upstream() -> String {
    let app = Router::new().route(
        "/{*path}",
        get(|Path(path): Path<String>| async move { format!("remote {path}") }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{address}/")
}

fn proxy(name: &str, remote: &str, rule: Option<&str>) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {"blocked": false, "autoBlock": false},
        "routingRuleName": rule,
        "raw": {"contentDisposition": "ATTACHMENT"}
    })
}

fn group(name: &str, members: &[&str], rule: Option<&str>) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
        "group": {"memberNames": members},
        "routingRuleName": rule,
        "raw": {"contentDisposition": "ATTACHMENT"}
    })
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "ATTACHMENT"}
    })
}

async fn create_repository(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/raw/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn create_rule(fixture: &Fixture, name: &str, mode: &str, matchers: &[&str]) {
    let reply = fixture
        .call(
            Method::POST,
            RULES,
            Auth::Admin,
            Some(json!({"name": name, "description": "", "mode": mode, "matchers": matchers})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

async fn fetch(fixture: &Fixture, path: &str) -> (StatusCode, String) {
    let reply = fixture.call(Method::GET, path, Auth::Admin, None).await;
    (reply.status, reply.text())
}

#[tokio::test]
async fn rules_are_created_read_renamed_and_deleted() {
    let fixture = ready().await;
    create_rule(&fixture, "block-sources", "BLOCK", &[".*-sources\\.jar"]).await;
    let listed = fixture.call(Method::GET, RULES, Auth::Admin, None).await;
    assert_eq!(
        listed.json(),
        json!([{"name": "block-sources", "description": "", "mode": "BLOCK", "matchers": [".*-sources\\.jar"]}])
    );
    let duplicate = fixture
        .call(
            Method::POST,
            RULES,
            Auth::Admin,
            Some(json!({"name": "BLOCK-sources", "mode": "ALLOW", "matchers": ["x"]})),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::BAD_REQUEST);
    assert_eq!(duplicate.json()[0]["id"], "PARAMETER name");

    let invalid = fixture
        .call(
            Method::POST,
            RULES,
            Auth::Admin,
            Some(json!({"name": "_x", "mode": "DENY", "matchers": ["(", "ok"]})),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    let ids: Vec<String> = invalid
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|problem| problem["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        ids,
        ["PARAMETER name", "PARAMETER mode", "PARAMETER matchers[0]"]
    );

    let missing = fixture
        .call(Method::GET, &format!("{RULES}/nope"), Auth::Admin, None)
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing.json()["message"],
        "Did not find a routing rule with the name 'nope'"
    );

    let remote = upstream().await;
    create_repository(
        &fixture,
        "proxy",
        proxy("mirror", &remote, Some("block-sources")),
    )
    .await;
    let renamed = fixture
        .call(
            Method::PUT,
            &format!("{RULES}/block-sources"),
            Auth::Admin,
            Some(json!({"name": "no-sources", "description": "d", "mode": "BLOCK", "matchers": [".*-sources\\.jar"]})),
        )
        .await;
    assert_eq!(renamed.status, StatusCode::NO_CONTENT, "{}", renamed.text());
    let mirror = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/raw/proxy/mirror",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(mirror.json()["routingRuleName"], "no-sources");

    let internal = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/routing-rules/no-sources",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(internal.json()["assignedRepositoryCount"], 1);
    assert_eq!(
        internal.json()["assignedRepositoryNames"],
        json!(["mirror"])
    );

    let in_use = fixture
        .call(
            Method::DELETE,
            &format!("{RULES}/no-sources"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(in_use.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        in_use.json()["message"],
        "Routing rule is still in use by 1 repositories."
    );
    let mut unassigned = proxy("mirror", &remote, None);
    unassigned["routingRuleName"] = Value::Null;
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/raw/proxy/mirror",
            Auth::Admin,
            Some(unassigned),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("{RULES}/no-sources"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);

    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=RoutingRule",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(audit.json().as_array().map(Vec::len), Some(3));
}

#[tokio::test]
async fn unknown_rules_cannot_be_assigned() {
    let fixture = ready().await;
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/proxy",
            Auth::Admin,
            Some(proxy("mirror", "https://example.org/", Some("missing"))),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json()[0]["id"], "PARAMETER routingRuleName");
}

#[tokio::test]
async fn proxies_and_groups_answer_blocked_paths_with_403() {
    let fixture = ready().await;
    let remote = upstream().await;
    create_rule(&fixture, "only-com", "ALLOW", &["/com/.*"]).await;
    create_rule(&fixture, "no-secrets", "BLOCK", &[".*/secret/.*"]).await;
    create_repository(
        &fixture,
        "proxy",
        proxy("mirror", &remote, Some("only-com")),
    )
    .await;
    create_repository(&fixture, "proxy", proxy("plain", &remote, None)).await;
    create_repository(&fixture, "hosted", hosted("local")).await;

    assert_eq!(
        fetch(&fixture, "/repository/mirror/com/a.txt").await,
        (StatusCode::OK, "remote com/a.txt".to_owned())
    );
    assert_eq!(
        fetch(&fixture, "/repository/mirror/org/a.txt").await,
        (
            StatusCode::FORBIDDEN,
            "Routing rules block the requested item from this repository".to_owned()
        )
    );

    create_repository(&fixture, "group", group("all", &["mirror", "local"], None)).await;
    assert_eq!(
        fetch(&fixture, "/repository/all/com/b.txt").await.0,
        StatusCode::OK
    );
    assert_eq!(
        fetch(&fixture, "/repository/all/org/b.txt").await.0,
        StatusCode::NOT_FOUND,
        "a group skips members whose rule blocks the path"
    );

    create_repository(
        &fixture,
        "group",
        group("guarded", &["plain"], Some("no-secrets")),
    )
    .await;
    assert_eq!(
        fetch(&fixture, "/repository/guarded/a/secret/b.txt")
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fetch(&fixture, "/repository/guarded/a/public/b.txt").await,
        (StatusCode::OK, "remote a/public/b.txt".to_owned())
    );
    let document = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/raw/group/guarded",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(document["routingRuleName"], "no-secrets");
    let plain_group = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/raw/group/all",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert!(
        plain_group.get("routingRuleName").is_none(),
        "groups without a rule keep the reference shape"
    );

    let preview = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/routing-rules/preview?path=/org/x.txt&filter=all",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let children = preview["children"].as_array().unwrap();
    let names: Vec<&str> = children
        .iter()
        .map(|node| node["repository"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["all", "guarded", "mirror", "plain"]);
    assert_eq!(children[0]["type"], "group");
    assert_eq!(children[0]["children"][0]["repository"], "mirror");
    assert_eq!(children[0]["children"][0]["allowed"], false);
    assert_eq!(children[0]["children"][0]["rule"], "only-com");
    assert_eq!(children[0]["children"][1]["allowed"], true);
    assert_eq!(children[2]["allowed"], false);
    let proxies = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/routing-rules/preview?path=org/x.txt&filter=proxies",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(proxies["children"].as_array().map(Vec::len), Some(2));
    let empty = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/routing-rules/preview?path=",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_test_endpoint_reports_each_matcher() {
    let fixture = ready().await;
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/internal/ui/routing-rules/test",
            Auth::Admin,
            Some(json!({"mode": "BLOCK", "matchers": ["com/.*", "/com/.*", "("], "path": "com/a.jar"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["allowed"], false);
    assert_eq!(body["matchers"][0]["matches"], false);
    assert_eq!(body["matchers"][1]["matches"], true);
    assert!(body["matchers"][2]["error"].is_string());
}

#[tokio::test]
async fn routing_rules_need_an_administrator() {
    let fixture = ready().await;
    fixture.add_user("jane", "secret-1", &[]).await;
    for (method, path) in [
        (Method::GET, RULES),
        (Method::GET, "/service/rest/internal/ui/routing-rules"),
    ] {
        let reply = fixture
            .call(method, path, Auth::Basic("jane", "secret-1"), None)
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
    }
    let anonymous = fixture
        .send_body(Method::GET, RULES, Auth::None, Body::empty(), &[])
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn docker_path_routing_applies_the_rule_too() {
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let fixture = ready().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_docker::Docker::default()));
    create_rule(&fixture, "no-library", "BLOCK", &["/v2/library/.*"]).await;
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/docker/proxy",
            Auth::Admin,
            Some(json!({
                "name": "hub",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": upstream().await, "contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": false, "timeToLive": 1},
                "httpClient": {"blocked": false, "autoBlock": false},
                "routingRuleName": "no-library",
                "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
                "dockerProxy": {"indexType": "REGISTRY"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_docker::routes());
    let credentials = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("admin:{}", fixture.admin_password),
    );
    for (path, blocked) in [
        ("/v2/hub/library/alpine/manifests/latest", true),
        ("/repository/hub/v2/library/alpine/manifests/latest", true),
        ("/v2/hub/team/app/manifests/latest", false),
    ] {
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::get(path)
                    .header("authorization", format!("Basic {credentials}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let text = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(status == StatusCode::FORBIDDEN, blocked, "{path}: {text}");
        assert_eq!(
            text == astrum_content::routing::BLOCKED,
            blocked,
            "{path}: {text}"
        );
    }
}
