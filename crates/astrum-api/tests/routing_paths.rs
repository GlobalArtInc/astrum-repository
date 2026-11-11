mod support;

use std::sync::{Arc, Mutex};

use astrum_content::{ContentError, ContentRequest, Ctx, Format};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use async_trait::async_trait;
use axum::Router;
use axum::extract::Path;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use serde_json::{Value, json};
use support::{Auth, Fixture};

type Seen = Arc<Mutex<Vec<(Option<String>, String)>>>;

struct Recording {
    inner: astrum_format_raw::Raw,
    seen: Seen,
}

#[async_trait]
impl Format for Recording {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        self.seen
            .lock()
            .unwrap()
            .push((ctx.request_path().map(str::to_owned), request.path.clone()));
        self.inner.handle(ctx, request).await
    }

    async fn get(
        &self,
        ctx: &Ctx,
        path: &str,
    ) -> Result<Option<astrum_content::Asset>, ContentError> {
        self.inner.get(ctx, path).await
    }
}

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
    fixture
}

async fn ready() -> (Fixture, Seen) {
    let fixture = prepared().await;
    let seen = Seen::default();
    fixture.state.content().register(Arc::new(Recording {
        inner: astrum_format_raw::Raw,
        seen: seen.clone(),
    }));
    (fixture, seen)
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
    create_in(fixture, "raw", kind, body).await;
}

async fn create_in(fixture: &Fixture, format: &str, kind: &str, body: Value) {
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

async fn expect(fixture: &Fixture, repository: &str, cases: &[(&str, StatusCode)]) {
    let mut wrong = Vec::new();
    for (path, want) in cases {
        let reply = fixture
            .call(
                Method::GET,
                &format!("/repository/{repository}/{path}"),
                Auth::Admin,
                None,
            )
            .await;
        if reply.status != *want {
            wrong.push(format!(
                "{repository}/{path}: got {} {:?}, want {want}",
                reply.status,
                reply.text()
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[tokio::test]
async fn block_rules_judge_the_normalised_path() {
    let (fixture, _) = ready().await;
    let remote = upstream().await;
    create_rule(
        &fixture,
        "no-secrets",
        "BLOCK",
        &["/secret/.*", r"/exact/key\.txt"],
    )
    .await;
    create_repository(
        &fixture,
        "proxy",
        proxy("blocker", &remote, Some("no-secrets")),
    )
    .await;
    let blocked = StatusCode::FORBIDDEN;
    expect(
        &fixture,
        "blocker",
        &[
            ("secret/a.txt", blocked),
            ("/secret/a.txt", blocked),
            ("//secret/a.txt", blocked),
            ("secret//a.txt", blocked),
            ("secret///a.txt", blocked),
            ("secret//", blocked),
            ("secret%2fa.txt", blocked),
            ("exact/key.txt", blocked),
            ("/exact/key.txt", blocked),
            ("//exact/key.txt", blocked),
            ("exact//key.txt", blocked),
            ("exact///key.txt", blocked),
            ("./secret/a.txt", StatusCode::BAD_REQUEST),
            ("secret/%2e%2e/open/a.txt", StatusCode::BAD_REQUEST),
            ("open/a.txt", StatusCode::OK),
            ("open//a.txt", StatusCode::OK),
            ("//open/a.txt", StatusCode::OK),
        ],
    )
    .await;
}

#[tokio::test]
async fn allow_rules_judge_the_normalised_path() {
    let (fixture, _) = ready().await;
    let remote = upstream().await;
    create_rule(&fixture, "only-allowed", "ALLOW", &["/allowed/.*"]).await;
    create_repository(
        &fixture,
        "proxy",
        proxy("allower", &remote, Some("only-allowed")),
    )
    .await;
    let blocked = StatusCode::FORBIDDEN;
    expect(
        &fixture,
        "allower",
        &[
            ("allowed/a.txt", StatusCode::OK),
            ("/allowed/a.txt", StatusCode::OK),
            ("//allowed/a.txt", StatusCode::OK),
            ("allowed//a.txt", StatusCode::OK),
            ("allowed///a.txt", StatusCode::OK),
            ("other//allowed/a.txt", blocked),
            ("other/a.txt", blocked),
            ("allowed/%2e%2e/other/a.txt", StatusCode::BAD_REQUEST),
        ],
    )
    .await;
}

async fn guarded_setup() -> Fixture {
    let (fixture, _) = ready().await;
    let remote = upstream().await;
    create_rule(
        &fixture,
        "no-secrets",
        "BLOCK",
        &["/secret/.*", r"/exact/key\.txt"],
    )
    .await;
    create_repository(
        &fixture,
        "proxy",
        proxy("blocker", &remote, Some("no-secrets")),
    )
    .await;
    create_repository(&fixture, "proxy", proxy("plain", &remote, None)).await;
    create_repository(&fixture, "hosted", hosted("local")).await;
    create_repository(
        &fixture,
        "group",
        group("guarded", &["plain"], Some("no-secrets")),
    )
    .await;
    create_repository(
        &fixture,
        "group",
        group("members", &["blocker", "local"], None),
    )
    .await;
    fixture
}

#[tokio::test]
async fn group_rules_judge_the_normalised_path() {
    let fixture = guarded_setup().await;
    let blocked = StatusCode::FORBIDDEN;
    expect(
        &fixture,
        "guarded",
        &[
            ("secret/a.txt", blocked),
            ("//secret/a.txt", blocked),
            ("secret//a.txt", blocked),
            ("exact//key.txt", blocked),
            ("open//a.txt", StatusCode::OK),
        ],
    )
    .await;
}

#[tokio::test]
async fn member_rules_judge_the_normalised_path() {
    let fixture = guarded_setup().await;
    let missing = StatusCode::NOT_FOUND;
    expect(
        &fixture,
        "members",
        &[
            ("secret/a.txt", missing),
            ("//secret/a.txt", missing),
            ("secret//a.txt", missing),
            ("exact//key.txt", missing),
            ("//exact/key.txt", missing),
            ("open//a.txt", StatusCode::OK),
        ],
    )
    .await;
}

#[tokio::test]
async fn formats_see_the_normalised_request_path() {
    let (fixture, seen) = ready().await;
    let remote = upstream().await;
    create_repository(&fixture, "proxy", proxy("plain", &remote, None)).await;
    expect(
        &fixture,
        "plain",
        &[
            ("x//y///z.txt", StatusCode::OK),
            ("//x/y/z.txt", StatusCode::OK),
            ("x/y/z.txt", StatusCode::OK),
            ("dir//sub//", StatusCode::NOT_FOUND),
        ],
    )
    .await;
    let seen = seen.lock().unwrap().clone();
    let expected = [
        (Some("/x/y/z.txt".to_owned()), "/x/y/z.txt".to_owned()),
        (Some("/x/y/z.txt".to_owned()), "/x/y/z.txt".to_owned()),
        (Some("/x/y/z.txt".to_owned()), "/x/y/z.txt".to_owned()),
        (Some("/dir/sub/".to_owned()), "/dir/sub/".to_owned()),
    ];
    assert_eq!(seen, expected);
}

async fn blocked_by_the_rule(fixture: &Fixture, app: &Router, path: &str) -> bool {
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let credentials = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("admin:{}", fixture.admin_password),
    );
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::get(path)
                .header("authorization", format!("Basic {credentials}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    status == StatusCode::FORBIDDEN && body == astrum_content::routing::BLOCKED.as_bytes()
}

async fn check_blocking(fixture: &Fixture, app: &Router, cases: &[(&str, bool)]) {
    let mut wrong = Vec::new();
    for (path, blocked) in cases {
        if blocked_by_the_rule(fixture, app, path).await != *blocked {
            wrong.push(format!("{path}: blocked should be {blocked}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[tokio::test]
async fn open_vsx_rules_judge_the_decoded_normalised_path() {
    let fixture = prepared().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_openvsx::OpenVsx));
    create_rule(&fixture, "no-secret", "BLOCK", &["/api/secret/.*"]).await;
    let remote = upstream().await;
    create_in(
        &fixture,
        "openvsx",
        "proxy",
        json!({
            "name": "vsx-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
            "routingRuleName": "no-secret",
        }),
    )
    .await;
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_openvsx::routes());
    check_blocking(
        &fixture,
        &app,
        &[
            ("/repository/vsx-proxy/api/secret/verify-pat", true),
            ("/repository/vsx-proxy//api/secret/verify-pat", true),
            ("/repository/vsx-proxy/api/%73ecret/verify-pat", true),
            ("/repository/vsx-proxy/api/%2fsecret/verify-pat", true),
            ("/repository/vsx%2Dproxy/api/secret/verify-pat", true),
            ("/repository/vsx-proxy/api/other/verify-pat", false),
            ("/repository/vsx%2Dproxy/api/other/verify-pat", false),
        ],
    )
    .await;
}

#[tokio::test]
async fn docker_rules_judge_the_v2_form_of_the_normalised_path() {
    let fixture = prepared().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_docker::Docker::default()));
    create_rule(
        &fixture,
        "no-library",
        "BLOCK",
        &["/v2/library/.*", "/v2/exact/manifests/latest"],
    )
    .await;
    let remote = upstream().await;
    create_in(
        &fixture,
        "docker",
        "proxy",
        json!({
            "name": "hub",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": false, "timeToLive": 1},
            "httpClient": {"blocked": false, "autoBlock": false},
            "routingRuleName": "no-library",
            "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true},
            "dockerProxy": {"indexType": "REGISTRY"}
        }),
    )
    .await;
    let app = astrum_api::router_with(fixture.state.clone(), astrum_format_docker::routes());
    check_blocking(
        &fixture,
        &app,
        &[
            ("/v2/hub/library/alpine/manifests/latest", true),
            ("/v2/hub//library/alpine/manifests/latest", true),
            ("/v2/hub///library/alpine/manifests/latest", true),
            ("/v2/hub/library//alpine/manifests/latest", true),
            ("/v2/hub/exact/manifests/latest", true),
            ("/v2/hub//exact/manifests/latest", true),
            ("/v2/hub/exact//manifests/latest", true),
            ("/v2/hub/team/app/manifests/latest", false),
            ("/v2/hub//team//app/manifests/latest", false),
        ],
    )
    .await;
}
