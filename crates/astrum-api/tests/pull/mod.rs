#![allow(dead_code)]

use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use astrum_replication::dispatch::Dispatcher;
use astrum_replication::protocol::component_key;
use astrum_replication::reconcile::{self, Summary};
use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use futures_util::stream;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::accounts;
use crate::cluster::{Node, REMOTES, connect, replicate};
use crate::support::Auth;

pub const NAME: &str = "/docs/a.txt";
pub const ALPHA: &[u8] = b"alpha";
pub const STUB_INSTANCE: &str = "5c1e0000-0000-4000-8000-000000000042";

pub struct Pulling {
    pub remote: Node,
    pub local: Node,
    pub replication: String,
}

pub fn pull_body(extra: Value) -> Value {
    let mut body = json!({"direction": "pull"});
    for (key, value) in extra.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

pub async fn pulling(extra: Value) -> Pulling {
    let remote = Node::start(&["source"]).await;
    let local = Node::start(&["mirror"]).await;
    let connected = connect(&local, &remote).await;
    let replication = replicate(&local, &connected, "mirror", "source", pull_body(extra)).await;
    Pulling {
        remote,
        local,
        replication,
    }
}

pub async fn reconcile(node: &Node, replication: &str) -> Summary {
    let id = Uuid::parse_str(replication).unwrap();
    reconcile::run(node.fixture.state.content(), id)
        .await
        .unwrap()
}

pub async fn drain(node: &Node) {
    let dispatcher = Dispatcher::new(node.fixture.state.content().clone());
    for _ in 0..50 {
        if dispatcher.run_once().await.unwrap().leased == 0 {
            return;
        }
    }
    panic!("the queue did not drain");
}

pub async fn column(node: &Node, sql: &str) -> Vec<String> {
    let rows = node
        .fixture
        .state
        .db()
        .conn()
        .query_all(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
    rows.iter()
        .map(|row| row.try_get("", "t").unwrap())
        .collect()
}

pub async fn queued(node: &Node) -> Vec<String> {
    column(node, "SELECT name AS t FROM replication_item ORDER BY name").await
}

pub async fn status(node: &Node, repository: &str, path: &str) -> StatusCode {
    node.get(repository, path).await.status
}

pub async fn user_token(node: &Node, user: &str, privileges: &[&str]) -> String {
    let fixture = &node.fixture;
    accounts::create_role(fixture, user, privileges, &[]).await;
    accounts::create_user(fixture, user, "pw-pw-pw-1", &[user]).await;
    let wanted = json!({"name": "pull", "scope": "WRITE", "expiresInDays": 1});
    let made = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Basic(user, "pw-pw-pw-1"),
            Some(wanted),
        )
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    made.json()["token"].as_str().unwrap().to_owned()
}

pub fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn bundle(format: &str, name: &str, origin: Option<&str>, size: u64, bytes: &[u8]) -> Value {
    json!({
        "format": format, "namespace": "/docs", "name": name, "version": "", "meta": {},
        "origin": origin,
        "assets": [{
            "path": "/docs/a.txt", "kind": "asset", "contentType": "text/plain", "size": size,
            "sha1": "", "sha256": sha(bytes), "sha512": "", "md5": "", "attributes": {}
        }]
    })
}

pub enum Answer {
    Json(Value),
    Raw(Vec<u8>),
}

pub enum Blob {
    Endless,
    Announce(u64),
    Slow(Vec<u8>, Duration),
}

#[derive(Clone)]
pub struct Stub {
    pub bundle: Arc<Mutex<Answer>>,
    pub blob: Arc<Mutex<Blob>>,
    pub blobs: Arc<AtomicUsize>,
}

impl Stub {
    pub fn says(&self, answer: Answer) {
        *self.bundle.lock().unwrap() = answer;
    }

    pub fn serves(&self, blob: Blob) {
        *self.blob.lock().unwrap() = blob;
    }

    pub fn blob_requests(&self) -> usize {
        self.blobs.load(Ordering::SeqCst)
    }
}

fn json_reply(value: Value) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        value.to_string(),
    )
        .into_response()
}

async fn serve(State(stub): State<Stub>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    if path.ends_with("/capabilities") {
        return json_reply(
            json!({"protocol": 1, "instanceId": STUB_INSTANCE, "version": "3.0.4", "formats": ["raw"]}),
        );
    }
    if path.ends_with("/inventory") {
        let key = component_key("/docs", NAME, "");
        return json_reply(json!({"items": [{"key": key, "fingerprint": "f".repeat(64)}]}));
    }
    if path.ends_with("/bundle") {
        return match &*stub.bundle.lock().unwrap() {
            Answer::Json(value) => json_reply(value.clone()),
            Answer::Raw(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                bytes.clone(),
            )
                .into_response(),
        };
    }
    if !path.contains("/blobs/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    stub.blobs.fetch_add(1, Ordering::SeqCst);
    let body = match &*stub.blob.lock().unwrap() {
        Blob::Endless => Body::from_stream(stream::repeat_with(|| {
            Ok::<Bytes, io::Error>(Bytes::from(vec![7u8; 65536]))
        })),
        Blob::Announce(length) => Body::from(vec![1u8; *length as usize]),
        Blob::Slow(bytes, pause) => {
            let (bytes, pause) = (bytes.clone(), *pause);
            let parts = stream::unfold(0usize, move |sent| {
                let (bytes, pause) = (bytes.clone(), pause);
                async move {
                    let byte = *bytes.get(sent)?;
                    tokio::time::sleep(pause).await;
                    Some((Ok::<Bytes, io::Error>(Bytes::from(vec![byte])), sent + 1))
                }
            });
            Body::from_stream(parts)
        }
    };
    let mut response = Response::new(body);
    if let Blob::Slow(bytes, _) = &*stub.blob.lock().unwrap() {
        response
            .headers_mut()
            .insert(header::CONTENT_LENGTH, bytes.len().into());
    }
    response
}

pub struct Hostile {
    pub stub: Stub,
    pub local: Node,
    pub replication: String,
}

impl Hostile {
    pub async fn attempt(&self) {
        reconcile(&self.local, &self.replication).await;
        drain(&self.local).await;
    }

    pub async fn dead(&self) -> Vec<String> {
        let sql = "SELECT last_error AS t FROM replication_item WHERE state = 'dead'";
        column(&self.local, sql).await
    }

    pub async fn wrote_nothing(&self) -> bool {
        let sql = "SELECT (SELECT count(*) FROM component WHERE repository = 'mirror') \
                   + (SELECT count(*) FROM replication_staged_blob) AS n";
        self.local.number(sql).await == 0
    }
}

pub async fn hostile() -> Hostile {
    let stub = Stub {
        bundle: Arc::new(Mutex::new(Answer::Json(bundle(
            "raw",
            NAME,
            None,
            ALPHA.len() as u64,
            ALPHA,
        )))),
        blob: Arc::new(Mutex::new(Blob::Announce(ALPHA.len() as u64))),
        blobs: Arc::new(AtomicUsize::new(0)),
    };
    let app = Router::new().fallback(serve).with_state(stub.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let local = Node::start(&["mirror"]).await;
    let body = json!({
        "name": format!("remote-{}", Uuid::new_v4().simple()),
        "url": format!("http://{address}"),
        "token": "stub-token-0001",
        "allowPrivateNetwork": true,
    });
    let made = local
        .fixture
        .call(Method::POST, REMOTES, Auth::Admin, Some(body))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    let remote = made.json()["id"].as_str().unwrap().to_owned();
    let replication = replicate(&local, &remote, "mirror", "source", pull_body(json!({}))).await;
    Hostile {
        stub,
        local,
        replication,
    }
}
