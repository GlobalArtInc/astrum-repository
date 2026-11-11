mod support;

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{Method, StatusCode};
use axum::routing::get;
use serde_json::{Value, json};
use support::{Auth, Fixture};
use tower::ServiceExt;

const NARROW: [&str; 4] = [
    "token",
    "-/user/org.couchdb.user:someone",
    "v1/users/authenticate",
    "v2/users/authenticate",
];

async fn prepared() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
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
    let state = &fixture.state;
    state
        .content()
        .register(std::sync::Arc::new(astrum_format_raw::Raw));
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
            "/service/rest/v1/routing-rules",
            Auth::Admin,
            Some(json!({"name": name, "description": "", "mode": mode, "matchers": matchers})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

fn narrow_app(fixture: &Fixture) -> Router {
    let routes = astrum_format_conan::routes()
        .merge(astrum_format_npm::routes())
        .merge(astrum_format_terraform::routes());
    astrum_api::router_with(fixture.state.clone(), routes)
}

async fn expect_all(fixture: &Fixture, cases: &[(&str, &[&str], StatusCode)]) {
    let credentials = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("admin:{}", fixture.admin_password),
    );
    let mut wrong = Vec::new();
    for (repository, paths, want) in cases {
        for path in *paths {
            let response = narrow_app(fixture)
                .oneshot(
                    axum::http::Request::get(format!("/repository/{repository}/{path}"))
                        .header("authorization", format!("Basic {credentials}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if response.status() != *want {
                wrong.push(format!(
                    "{repository}/{path}: got {}, want {want}",
                    response.status()
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[tokio::test]
async fn block_rules_apply_on_the_narrow_login_routes() {
    let fixture = prepared().await;
    let remote = upstream().await;
    create_rule(
        &fixture,
        "no-login",
        "BLOCK",
        &[
            "/token",
            "/-/user/.*",
            "/v1/users/authenticate",
            "/v2/users/authenticate",
        ],
    )
    .await;
    create_repository(
        &fixture,
        "proxy",
        proxy("guarded", &remote, Some("no-login")),
    )
    .await;
    create_repository(&fixture, "proxy", proxy("open", &remote, None)).await;
    create_repository(
        &fixture,
        "group",
        group("group-rule", &["open"], Some("no-login")),
    )
    .await;
    create_repository(&fixture, "group", group("member-rule", &["guarded"], None)).await;
    expect_all(
        &fixture,
        &[
            ("open", &NARROW, StatusCode::OK),
            ("guarded", &NARROW, StatusCode::FORBIDDEN),
            ("group-rule", &NARROW, StatusCode::FORBIDDEN),
            ("member-rule", &NARROW, StatusCode::NOT_FOUND),
        ],
    )
    .await;
}

#[tokio::test]
async fn allow_rules_apply_on_the_narrow_login_routes() {
    let fixture = prepared().await;
    let remote = upstream().await;
    create_rule(&fixture, "only-allowed", "ALLOW", &["/allowed/.*"]).await;
    create_rule(&fixture, "only-token", "ALLOW", &["/token"]).await;
    create_repository(
        &fixture,
        "proxy",
        proxy("allowed", &remote, Some("only-allowed")),
    )
    .await;
    create_repository(
        &fixture,
        "proxy",
        proxy("token-only", &remote, Some("only-token")),
    )
    .await;
    expect_all(
        &fixture,
        &[
            ("allowed", &NARROW, StatusCode::FORBIDDEN),
            ("token-only", &NARROW[..1], StatusCode::OK),
            ("token-only", &NARROW[1..], StatusCode::FORBIDDEN),
        ],
    )
    .await;
}
