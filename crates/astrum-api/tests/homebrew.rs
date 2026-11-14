mod support;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "brew";
const FRESH: &str = "brew-fresh";
const ANONYMOUS_PULL: &str = "Bearer QQ==";
const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";

#[derive(Clone, Debug)]
struct Call {
    path: String,
    query: Option<String>,
    authorization: Option<String>,
    accept: Option<String>,
    conditional: bool,
}

type Files = BTreeMap<String, (String, Vec<u8>)>;

#[derive(Clone, Default)]
struct Stub {
    files: Arc<Mutex<Files>>,
    redirects: Arc<Mutex<BTreeMap<String, String>>>,
    seen: Arc<Mutex<Vec<Call>>>,
}

impl Stub {
    fn publish(&self, path: &str, content_type: &str, body: impl Into<Vec<u8>>) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_owned(), (content_type.to_owned(), body.into()));
    }

    fn redirect(&self, path: &str, target: &str) {
        self.redirects
            .lock()
            .unwrap()
            .insert(path.to_owned(), target.to_owned());
    }

    fn calls(&self, path: &str) -> Vec<Call> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.path == path)
            .cloned()
            .collect()
    }

    fn total(&self) -> usize {
        self.seen.lock().unwrap().len()
    }
}

async fn answer(State(stub): State<Stub>, uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/').to_owned();
    let text = |name: header::HeaderName| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    };
    stub.seen.lock().unwrap().push(Call {
        path: path.clone(),
        query: uri.query().map(str::to_owned),
        authorization: text(header::AUTHORIZATION),
        accept: text(header::ACCEPT),
        conditional: text(header::IF_NONE_MATCH).is_some(),
    });
    let target = stub.redirects.lock().unwrap().get(&path).cloned();
    if let Some(target) = target {
        return (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, target)]).into_response();
    }
    let found = stub.files.lock().unwrap().get(&path).cloned();
    match found {
        Some((content_type, body)) => {
            let etag = format!("\"{}\"", sha(&body));
            if text(header::IF_NONE_MATCH).as_deref() == Some(etag.as_str()) {
                return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
            }
            (
                [(header::CONTENT_TYPE, content_type), (header::ETAG, etag)],
                body,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn serve_stub() -> (Stub, SocketAddr) {
    let stub = Stub::default();
    let app = Router::new()
        .route("/{*path}", routing::get(answer))
        .with_state(stub.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (stub, address)
}

struct Env {
    fixture: Fixture,
    origin: Stub,
    cdn: Stub,
    cdn_url: String,
}

fn sha(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn proxy(name: &str, remote: &str, content_age: u32, metadata_age: u32) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": content_age, "metadataMaxAge": metadata_age},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": true},
    })
}

async fn create(fixture: &Fixture, kind: &str, body: Value) -> Reply {
    fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/homebrew/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn ready() -> Env {
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
    let (origin, origin_address) = serve_stub().await;
    let (cdn, cdn_address) = serve_stub().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_homebrew::Homebrew::with_registry(
            format!("http://{origin_address}"),
        )));
    let remote = format!("http://{origin_address}/api");
    for (name, content_age, metadata_age) in [(REPOSITORY, 1440, 10), (FRESH, 1440, 0)] {
        let reply = create(
            &fixture,
            "proxy",
            proxy(name, &remote, content_age, metadata_age),
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    Env {
        fixture,
        origin,
        cdn,
        cdn_url: format!("http://{cdn_address}"),
    }
}

impl Env {
    async fn get(&self, repository: &str, path: &str) -> Reply {
        self.fixture
            .send(
                Method::GET,
                &format!("/repository/{repository}/{path}"),
                Auth::Admin,
                None,
                &[],
            )
            .await
    }

    async fn stored(&self, repository: &str, path: &str) -> bool {
        astrum_content::store::find(self.fixture.state.db().conn(), repository, path)
            .await
            .unwrap()
            .is_some()
    }
}

fn blob_path(image: &str, data: &[u8]) -> String {
    format!("v2/homebrew/core/{image}/blobs/sha256:{}", sha(data))
}

#[tokio::test]
async fn api_endpoints_are_a_closed_allow_list() {
    let env = ready().await;
    let good = [
        "api/formula.jws.json",
        "api/internal/packages.arm64_sonoma.jws.json",
        "api/cask_tap_migrations.jws.json",
    ];
    for path in good {
        env.origin.publish(path, "application/json", path);
        let reply = env.get(REPOSITORY, path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(reply.body, path.as_bytes());
    }
    assert_eq!(env.origin.total(), good.len());
    for path in [
        "api/formula.json",
        "api/../x.jws.json",
        "api/internal/x/y.jws.json",
        "api/Formula.jws.json",
        "api/formula.jws.json/",
        "api/.jws.json",
    ] {
        env.origin.publish(path, "application/json", "{}");
        let reply = env.get(REPOSITORY, path).await;
        assert!(
            matches!(
                reply.status,
                StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST
            ),
            "{path}: {}",
            reply.status
        );
    }
    assert_eq!(env.origin.total(), good.len());
}

#[tokio::test]
async fn api_files_pass_unchanged_and_are_cached_for_the_metadata_age() {
    let env = ready().await;
    let path = "api/internal/packages.arm64_sonoma.jws.json";
    let body: &[u8] = b"{\"payload\":\"x\",\"signatures\":[\"\xc3\xbf\"]}\n\x00\xff";
    env.origin.publish(path, "application/vnd.test+json", body);
    for _ in 0..2 {
        let reply = env.get(REPOSITORY, path).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(reply.body, body);
        assert_eq!(reply.headers["content-type"], "application/vnd.test+json");
    }
    let calls = env.origin.calls(path);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].authorization, None);

    let other = "api/cask.jws.json";
    env.origin.publish(other, "application/json", "{}");
    let reply = env
        .get(REPOSITORY, &format!("{other}?url=http://127.0.0.1:1/x"))
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let calls = env.origin.calls(other);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].query, None);
}

#[tokio::test]
async fn api_files_are_revalidated_after_the_metadata_age() {
    let env = ready().await;
    let path = "api/formula.jws.json";
    env.origin.publish(path, "application/json", "{\"a\":1}");
    for _ in 0..2 {
        let reply = env.get(FRESH, path).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(reply.body, b"{\"a\":1}");
    }
    let calls = env.origin.calls(path);
    assert_eq!(calls.len(), 2);
    assert!(!calls[0].conditional);
    assert!(calls[1].conditional);
}

#[tokio::test]
async fn blobs_of_one_and_two_segment_images_are_fetched_anonymously_and_cached() {
    let env = ready().await;
    for image in ["jq", "openssl/3"] {
        let data = format!("bottle of {image}").into_bytes();
        let path = blob_path(image, &data);
        env.origin
            .publish(&path, "application/octet-stream", data.clone());
        for _ in 0..2 {
            let reply = env.get(REPOSITORY, &path).await;
            assert_eq!(reply.status, StatusCode::OK, "{image}");
            assert_eq!(reply.body, data);
        }
        let calls = env.origin.calls(&path);
        assert_eq!(calls.len(), 1, "{image}");
        assert_eq!(calls[0].authorization.as_deref(), Some(ANONYMOUS_PULL));
        assert!(env.stored(REPOSITORY, &format!("/{path}")).await);
    }
}

#[tokio::test]
async fn a_blob_that_does_not_match_its_digest_is_not_stored() {
    let env = ready().await;
    let path = blob_path("jq", b"the bytes the path names");
    env.origin
        .publish(&path, "application/octet-stream", "other bytes");
    for round in 1..=2 {
        let reply = env.get(REPOSITORY, &path).await;
        assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
        assert_eq!(env.origin.calls(&path).len(), round);
    }
    assert!(!env.stored(REPOSITORY, &format!("/{path}")).await);
}

#[tokio::test]
async fn the_redirect_to_the_blob_cdn_does_not_carry_the_authorization_header() {
    let env = ready().await;
    let data = b"bottle from the cdn".to_vec();
    let path = blob_path("jq", &data);
    let target = format!("blobs/{}", sha(&data));
    env.cdn
        .publish(&target, "application/octet-stream", data.clone());
    env.origin
        .redirect(&path, &format!("{}/{target}", env.cdn_url));
    let reply = env.get(REPOSITORY, &path).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.body, data);
    let at_registry = env.origin.calls(&path);
    assert_eq!(at_registry.len(), 1);
    assert_eq!(
        at_registry[0].authorization.as_deref(),
        Some(ANONYMOUS_PULL)
    );
    let at_cdn = env.cdn.calls(&target);
    assert_eq!(at_cdn.len(), 1);
    assert_eq!(at_cdn[0].authorization, None);

    let blocked = blob_path("jq", b"never fetched");
    env.origin
        .redirect(&blocked, "http://169.254.169.254/latest/meta-data");
    let reply = env.get(REPOSITORY, &blocked).await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
    assert!(!env.stored(REPOSITORY, &format!("/{blocked}")).await);
}

#[tokio::test]
async fn manifests_forward_the_accept_header_and_follow_the_metadata_age() {
    let env = ready().await;
    for (image, tag) in [("jq", "1.7.1"), ("openssl/3", "3.4.0-1")] {
        let path = format!("v2/homebrew/core/{image}/manifests/{tag}");
        env.origin.publish(&path, OCI_INDEX, "{\"manifests\":[]}");
        for _ in 0..2 {
            let reply = env.get(FRESH, &path).await;
            assert_eq!(reply.status, StatusCode::OK, "{image}");
            assert_eq!(reply.body, b"{\"manifests\":[]}");
            assert_eq!(reply.headers["content-type"], OCI_INDEX);
        }
        let calls = env.origin.calls(&path);
        assert_eq!(calls.len(), 2, "{image}");
        for call in &calls {
            assert_eq!(call.accept.as_deref(), Some(OCI_INDEX));
            assert_eq!(call.authorization.as_deref(), Some(ANONYMOUS_PULL));
        }
        assert!(calls[1].conditional);
    }
}

#[tokio::test]
async fn nothing_outside_the_table_reaches_an_upstream() {
    let env = ready().await;
    let hex = sha(b"x");
    let upper = hex.to_uppercase();
    let refused = [
        "https://example.org/x".to_owned(),
        "http://127.0.0.1:1/x".to_owned(),
        "api/formula.json".to_owned(),
        format!("v2/homebrew/core/a/b/c/blobs/sha256:{hex}"),
        "v2/homebrew/core/a/b/c/manifests/1".to_owned(),
        format!("v2/homebrew/core/jq/blobs/sha256:{upper}"),
        "v2/homebrew/core/jq/manifests/a/b".to_owned(),
        "v2/homebrew/core/jq/manifests/a%2fb".to_owned(),
        "v2/homebrew/core/a%5cb/manifests/1".to_owned(),
        "v2/homebrew/core/%2e%2e/manifests/1".to_owned(),
        "v2/homebrew/other/jq/manifests/1".to_owned(),
        "v2/homebrew/core/jq/tags/list".to_owned(),
        "v2/".to_owned(),
        "robots.txt".to_owned(),
        String::new(),
    ];
    for path in &refused {
        let reply = env.get(REPOSITORY, path).await;
        assert!(
            matches!(
                reply.status,
                StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST
            ),
            "{path}: {}",
            reply.status
        );
    }
    let valid = "api/formula.jws.json";
    for method in [Method::POST, Method::PUT, Method::DELETE] {
        let reply = env
            .fixture
            .send(
                method,
                &format!("/repository/{REPOSITORY}/{valid}"),
                Auth::Admin,
                None,
                &[],
            )
            .await;
        assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
    }
    let anonymous = env
        .fixture
        .send(
            Method::GET,
            &format!("/repository/{REPOSITORY}/{valid}"),
            Auth::None,
            None,
            &[],
        )
        .await;
    assert!(
        matches!(
            anonymous.status,
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ),
        "{}",
        anonymous.status
    );
    assert_eq!(env.origin.total(), 0);
    assert_eq!(env.cdn.total(), 0);
}

#[tokio::test]
async fn hosted_and_group_repositories_are_not_offered() {
    let env = ready().await;
    for kind in ["hosted", "group"] {
        let reply = create(
            &env.fixture,
            kind,
            json!({"name": format!("brew-{kind}"), "online": true}),
        )
        .await;
        assert!(reply.status.is_client_error(), "{kind}: {}", reply.status);
    }
}
