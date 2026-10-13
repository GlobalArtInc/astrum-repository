mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::json;
use sha2::{Digest, Sha256};
use support::{Auth, Fixture};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const CONFIG: &[u8] = b"{\"model_type\": \"bert\"}\n";
const WEIGHTS: &[u8] = b"pretend these are model weights, stored in LFS";

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

#[derive(Clone, Default)]
struct Hub {
    files: Arc<AtomicUsize>,
}

async fn revision(Path((owner, name, revision)): Path<(String, String, String)>) -> Response {
    if format!("{owner}/{name}") != "acme/tiny" || !(revision == "main" || revision == COMMIT) {
        return (
            StatusCode::NOT_FOUND,
            [("x-error-code", "RepoNotFound")],
            "{}",
        )
            .into_response();
    }
    axum::Json(json!({
        "id": "acme/tiny",
        "sha": COMMIT,
        "siblings": [
            {"rfilename": "config.json", "blobId": "1111111111111111111111111111111111111111", "size": CONFIG.len()},
            {"rfilename": "weights/model.bin", "blobId": "2222222222222222222222222222222222222222", "size": 134,
             "lfs": {"sha256": sha256(WEIGHTS), "size": WEIGHTS.len(), "pointerSize": 134}}
        ]
    }))
    .into_response()
}

async fn resolve(
    State(hub): State<Hub>,
    Path((owner, name, rest)): Path<(String, String, String)>,
) -> Response {
    hub.files.fetch_add(1, Ordering::SeqCst);
    match (format!("{owner}/{name}").as_str(), rest.as_str()) {
        ("acme/tiny", rest) if rest == format!("{COMMIT}/config.json") => (
            [(header::ETAG, "\"1111111111111111111111111111111111111111\"")],
            CONFIG,
        )
            .into_response(),
        ("acme/tiny", rest) if rest == format!("{COMMIT}/weights/model.bin") => (
            StatusCode::FOUND,
            [(header::LOCATION, "/lfs-storage/weights")],
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            [("x-error-code", "EntryNotFound")],
            "",
        )
            .into_response(),
    }
}

async fn lfs() -> Response {
    Body::from(WEIGHTS).into_response()
}

async fn hub() -> (String, Hub) {
    let state = Hub::default();
    let app = Router::new()
        .route(
            "/api/models/{owner}/{name}/revision/{revision}",
            routing::get(revision),
        )
        .route("/{owner}/{name}/resolve/{*rest}", routing::get(resolve))
        .route("/lfs-storage/weights", routing::get(lfs))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), state)
}

async fn ready() -> (Fixture, Hub) {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_huggingface::HuggingFace::default()));
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
    let (remote, state) = hub().await;
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/huggingface/proxy",
            Auth::Admin,
            Some(json!({
                "name": "hf",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": true, "timeToLive": 60},
                "httpClient": {"blocked": false, "autoBlock": false}
            })),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    (fixture, state)
}

#[tokio::test]
async fn revisions_trees_and_paths_come_from_the_hub() {
    let (fixture, _) = ready().await;
    let info = fixture
        .call(
            Method::GET,
            "/repository/hf/api/models/acme/tiny",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(info.status, StatusCode::OK, "{}", info.text());
    assert_eq!(info.json()["sha"], COMMIT);
    let at_commit = fixture
        .call(
            Method::GET,
            &format!("/repository/hf/api/models/acme/tiny/revision/{COMMIT}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(at_commit.json()["siblings"].as_array().unwrap().len(), 2);
    let tree = fixture
        .call(
            Method::GET,
            "/repository/hf/api/models/acme/tiny/tree/main?recursive=true",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let paths: Vec<&str> = tree
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, ["weights", "config.json", "weights/model.bin"]);
    assert_eq!(tree[2]["lfs"]["oid"], sha256(WEIGHTS));
    let paths_info = fixture
        .send_body(
            Method::POST,
            "/repository/hf/api/models/acme/tiny/paths-info/main",
            Auth::Admin,
            Body::from("paths=config.json&paths=weights&paths=missing"),
            &[("content-type", "application/x-www-form-urlencoded")],
        )
        .await;
    assert_eq!(paths_info.status, StatusCode::OK, "{}", paths_info.text());
    let listed: Vec<String> = paths_info
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(listed, ["config.json", "weights"]);
    let missing = fixture
        .call(
            Method::GET,
            "/repository/hf/api/models/acme/other",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.headers["x-error-code"], "RepoNotFound");
}

#[tokio::test]
async fn files_are_cached_per_commit_with_hub_headers() {
    let (fixture, hub) = ready().await;
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/hf/acme/tiny/resolve/main/weights/model.bin",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert_eq!(head.headers["x-repo-commit"], COMMIT);
    assert_eq!(
        head.headers["x-linked-etag"],
        format!("\"{}\"", sha256(WEIGHTS)).as_str()
    );
    assert_eq!(
        head.headers["x-linked-size"],
        WEIGHTS.len().to_string().as_str()
    );
    assert_eq!(hub.files.load(Ordering::SeqCst), 0);

    let weights = fixture
        .call(
            Method::GET,
            "/repository/hf/acme/tiny/resolve/main/weights/model.bin",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(weights.status, StatusCode::OK);
    assert_eq!(weights.body, WEIGHTS);
    assert_eq!(
        weights.headers[header::ETAG],
        format!("\"{}\"", sha256(WEIGHTS)).as_str()
    );
    let config = fixture
        .call(
            Method::GET,
            "/repository/hf/acme/tiny/resolve/main/config.json",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(config.body, CONFIG);
    assert_eq!(
        config.headers["x-linked-etag"],
        "\"1111111111111111111111111111111111111111\""
    );
    let again = fixture
        .call(
            Method::GET,
            &format!("/repository/hf/acme/tiny/resolve/{COMMIT}/config.json"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(again.body, CONFIG);
    assert_eq!(hub.files.load(Ordering::SeqCst), 2);

    let stored = fixture
        .call(
            Method::GET,
            &format!("/repository/hf/models/acme/tiny/{COMMIT}/config.json"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.body, CONFIG);
    let missing = fixture
        .call(
            Method::GET,
            "/repository/hf/acme/tiny/resolve/main/nothing.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.headers["x-error-code"], "EntryNotFound");

    let search = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=hf&huggingface.revision=main",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let items = search["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "{search}");
    assert_eq!(items[0]["group"], "models");
    assert_eq!(items[0]["name"], "acme/tiny");
    assert_eq!(items[0]["version"], COMMIT);
    let anonymous = fixture
        .call(
            Method::GET,
            "/repository/hf/api/models/acme/tiny",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}
