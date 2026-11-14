use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_outbound::Snapshot;
use astrum_replication::{Client, Remote};
use astrum_security::SecretBox;
use astrum_security::ssrf::{Guard, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

pub const TOKEN: &str = "tok-5f1c9a0d77e24b";
pub const INSTANCE: &str = "0b8e6f0a-0000-4000-8000-000000000001";

#[derive(Clone, Debug)]
pub struct Logged {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub authorization: Option<String>,
    pub content_length: Option<String>,
    pub chunked: bool,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub struct Forced {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
    pub echo_authorization: bool,
}

#[derive(Clone, Default)]
pub struct Receiver {
    pub seen: Arc<Mutex<Vec<Logged>>>,
    forced: Arc<Mutex<Option<Forced>>>,
}

impl Receiver {
    pub fn force(&self, status: u16, body: impl Into<Vec<u8>>) {
        *self.forced.lock().unwrap() = Some(Forced {
            status,
            headers: Vec::new(),
            body: body.into(),
            echo_authorization: false,
        });
    }

    pub fn force_with(&self, forced: Forced) {
        *self.forced.lock().unwrap() = Some(forced);
    }

    pub fn log(&self) -> Vec<Logged> {
        self.seen.lock().unwrap().clone()
    }
}

fn json_reply(status: StatusCode, value: Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        value.to_string(),
    )
        .into_response()
}

pub fn outcome() -> Value {
    json!({"status": "copied", "componentId": 7, "componentIds": [7],
        "assets": [{"path": "a.txt", "status": "new", "size": 6}],
        "bytesCopied": 6, "bytesReused": 0})
}

async fn serve(State(receiver): State<Receiver>, request: Request<Body>) -> Response {
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap()
        .to_vec();
    let text = |name: header::HeaderName| {
        parts
            .headers
            .get(name)
            .map(|value| value.to_str().unwrap().to_owned())
    };
    let authorization = text(header::AUTHORIZATION);
    receiver.seen.lock().unwrap().push(Logged {
        method: parts.method.to_string(),
        path: parts.uri.path().to_owned(),
        query: parts.uri.query().map(str::to_owned),
        authorization: authorization.clone(),
        content_length: text(header::CONTENT_LENGTH),
        chunked: text(header::TRANSFER_ENCODING).is_some(),
        body: body.clone(),
    });
    let forced = receiver.forced.lock().unwrap().clone();
    if let Some(forced) = forced {
        let mut response = Response::builder().status(forced.status);
        for (name, value) in &forced.headers {
            response = response.header(*name, value);
        }
        let body = match forced.echo_authorization {
            true => json!({"code": "echo", "message": authorization.unwrap_or_default()})
                .to_string()
                .into_bytes(),
            false => forced.body,
        };
        return response.body(Body::from(body)).unwrap();
    }
    let path = parts.uri.path();
    let request: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    match parts.method.as_str() {
        "GET" if path.ends_with("/capabilities") => json_reply(
            StatusCode::OK,
            json!({"protocol": 1, "instanceId": INSTANCE, "version": "3.0.4", "formats": ["raw", "maven2"]}),
        ),
        "POST" if path.ends_with("/blobs/missing") => {
            json_reply(StatusCode::OK, json!({"missing": request["sha256"]}))
        }
        "PUT" if path.contains("/blobs/") => StatusCode::CREATED.into_response(),
        "POST" if path.ends_with("/components") => json_reply(StatusCode::OK, outcome()),
        "DELETE" if path.ends_with("/components") => StatusCode::NO_CONTENT.into_response(),
        "POST" if path.ends_with("/inventory/compare") => json_reply(
            StatusCode::OK,
            json!({"differing": request["items"].as_array().map(|items| items.iter().map(|item| item["key"].clone()).collect::<Vec<_>>())}),
        ),
        "GET" if path.ends_with("/inventory") => json_reply(
            StatusCode::OK,
            json!({"items": [{"key": "app/1", "fingerprint": "f", "origin": null}], "next": "app/1"}),
        ),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn start() -> (SocketAddr, Receiver) {
    let receiver = Receiver::default();
    let app = Router::new().fallback(serve).with_state(receiver.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, receiver)
}

pub fn guard_off() -> Guard {
    Guard::new(&SsrfProtection {
        enabled: false,
        ..SsrfProtection::default()
    })
}

pub fn remote(url: &str) -> Remote {
    Remote {
        url: url.to_owned(),
        use_trust_store: false,
        allow_private_network: true,
        max_concurrency: 3,
        max_bytes_per_second: None,
    }
}

pub fn build(remote: &Remote, guard: &Guard) -> Result<Client, astrum_replication::Error> {
    Client::new(
        remote,
        TOKEN,
        &Snapshot::default(),
        &SecretBox::from_key([7; 32]),
        guard,
    )
}

pub async fn connected() -> (Client, Receiver, SocketAddr) {
    let (address, receiver) = start().await;
    let client = build(&remote(&format!("http://{address}")), &guard_off()).unwrap();
    (client, receiver, address)
}
