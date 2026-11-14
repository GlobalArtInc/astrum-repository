use std::sync::{Arc, Mutex};

use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tokio::sync::Notify;

#[derive(Clone)]
pub struct Fail {
    status: u16,
    times: usize,
    suffix: Option<&'static str>,
    retry_after: Option<&'static str>,
    body: Option<Value>,
}

impl Fail {
    pub fn new(status: u16, times: usize) -> Self {
        Self {
            status,
            times,
            suffix: None,
            retry_after: None,
            body: None,
        }
    }

    pub fn on(mut self, suffix: &'static str) -> Self {
        self.suffix = Some(suffix);
        self
    }

    pub fn retry_after(mut self, seconds: &'static str) -> Self {
        self.retry_after = Some(seconds);
        self
    }

    pub fn body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }
}

#[derive(Default)]
struct Plan {
    fail: Option<Fail>,
    commits: usize,
    uploads: usize,
    tombstones: usize,
}

#[derive(Default)]
struct Gate {
    armed: Mutex<Option<&'static str>>,
    reached: Notify,
    open: Notify,
}

#[derive(Clone, Default)]
pub struct Force {
    plan: Arc<Mutex<Plan>>,
    gate: Arc<Gate>,
}

impl Force {
    pub fn fail(&self, fail: Fail) {
        self.plan.lock().unwrap().fail = Some(fail);
    }

    pub fn hold(&self, suffix: &'static str) {
        *self.gate.armed.lock().unwrap() = Some(suffix);
    }

    pub async fn reached(&self) {
        self.gate.reached.notified().await;
    }

    pub fn open(&self) {
        self.gate.open.notify_one();
    }

    pub fn commits(&self) -> usize {
        self.plan.lock().unwrap().commits
    }

    pub fn uploads(&self) -> usize {
        self.plan.lock().unwrap().uploads
    }

    pub fn tombstones(&self) -> usize {
        self.plan.lock().unwrap().tombstones
    }
}

pub(super) async fn forced(State(force): State<Force>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let method = request.method().clone();
    let held = {
        let mut armed = force.gate.armed.lock().unwrap();
        let hit = armed.is_some_and(|suffix| path.ends_with(suffix));
        if hit {
            armed.take();
        }
        hit
    };
    if held {
        force.gate.reached.notify_one();
        force.gate.open.notified().await;
    }
    let refused = {
        let mut plan = force.plan.lock().unwrap();
        let hit = plan.fail.as_ref().is_some_and(|fail| {
            fail.times > 0 && fail.suffix.is_none_or(|suffix| path.ends_with(suffix))
        });
        match hit {
            true => {
                let fail = plan.fail.as_mut().unwrap();
                fail.times -= 1;
                Some(fail.clone())
            }
            false => {
                if method == Method::POST && path.ends_with("/components") {
                    plan.commits += 1;
                }
                if method == Method::PUT && path.contains("/blobs/") {
                    plan.uploads += 1;
                }
                if method == Method::DELETE && path.ends_with("/components") {
                    plan.tombstones += 1;
                }
                None
            }
        }
    };
    let Some(fail) = refused else {
        return next.run(request).await;
    };
    let body = fail
        .body
        .clone()
        .unwrap_or_else(|| json!({"code": "forced", "message": "forced by the test"}));
    let mut response = (
        StatusCode::from_u16(fail.status).unwrap(),
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response();
    if let Some(seconds) = fail.retry_after {
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static(seconds));
    }
    response
}
