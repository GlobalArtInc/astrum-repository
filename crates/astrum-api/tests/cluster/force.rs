use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::{Body, Bytes};
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use serde_json::{Value, json};
use tokio::sync::{Notify, watch};

const MARKED_BODY_LIMIT: usize = 8 * 1024 * 1024;

#[derive(Clone)]
pub struct Fail {
    status: u16,
    times: usize,
    suffix: Option<&'static str>,
    marker: Option<&'static str>,
    retry_after: Option<&'static str>,
    body: Option<Value>,
}

impl Fail {
    pub fn new(status: u16, times: usize) -> Self {
        Self {
            status,
            times,
            suffix: None,
            marker: None,
            retry_after: None,
            body: None,
        }
    }

    pub fn reset(times: usize) -> Self {
        Self::new(0, times)
    }

    pub fn on(mut self, suffix: &'static str) -> Self {
        self.suffix = Some(suffix);
        self
    }

    pub fn marked(mut self, marker: &'static str) -> Self {
        self.marker = Some(marker);
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

struct Gate {
    armed: Mutex<Option<(&'static str, bool)>>,
    reached: Notify,
    open: Notify,
    held: AtomicUsize,
    all: watch::Sender<bool>,
}

impl Default for Gate {
    fn default() -> Self {
        Self {
            armed: Mutex::new(None),
            reached: Notify::new(),
            open: Notify::new(),
            held: AtomicUsize::new(0),
            all: watch::channel(false).0,
        }
    }
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
        *self.gate.armed.lock().unwrap() = Some((suffix, false));
    }

    pub fn hold_all(&self, suffix: &'static str) {
        self.gate.all.send_replace(false);
        *self.gate.armed.lock().unwrap() = Some((suffix, true));
    }

    pub async fn reached(&self) {
        self.gate.reached.notified().await;
    }

    pub fn open(&self) {
        self.gate.open.notify_one();
    }

    pub fn open_all(&self) {
        self.gate.armed.lock().unwrap().take();
        self.gate.all.send_replace(true);
    }

    pub fn held(&self) -> usize {
        self.gate.held.load(Ordering::SeqCst)
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

fn dropped_reply() -> Response {
    let broken = stream::once(async { Err::<Bytes, io::Error>(io::Error::other("reset")) });
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_LENGTH, "1000")
        .body(Body::from_stream(broken))
        .unwrap()
}

pub(super) async fn forced(State(force): State<Force>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let method = request.method().clone();
    let gate = {
        let mut armed = force.gate.armed.lock().unwrap();
        match *armed {
            Some((suffix, all)) if path.ends_with(suffix) => {
                if !all {
                    armed.take();
                }
                Some(all)
            }
            _ => None,
        }
    };
    match gate {
        Some(true) => {
            force.gate.held.fetch_add(1, Ordering::SeqCst);
            let mut released = force.gate.all.subscribe();
            let _ = released.wait_for(|open| *open).await;
            force.gate.held.fetch_sub(1, Ordering::SeqCst);
        }
        Some(false) => {
            force.gate.reached.notify_one();
            force.gate.open.notified().await;
        }
        None => {}
    }
    let marked = force
        .plan
        .lock()
        .unwrap()
        .fail
        .as_ref()
        .is_some_and(|fail| {
            fail.marker.is_some() && fail.suffix.is_none_or(|suffix| path.ends_with(suffix))
        });
    let (parts, body) = request.into_parts();
    let (request, text) = match marked {
        true => {
            let bytes = axum::body::to_bytes(body, MARKED_BODY_LIMIT).await.unwrap();
            let text = String::from_utf8_lossy(&bytes).into_owned();
            (Request::from_parts(parts, Body::from(bytes)), text)
        }
        false => (Request::from_parts(parts, body), String::new()),
    };
    let refused = {
        let mut plan = force.plan.lock().unwrap();
        let hit = plan.fail.as_ref().is_some_and(|fail| {
            fail.times > 0
                && fail.suffix.is_none_or(|suffix| path.ends_with(suffix))
                && fail.marker.is_none_or(|marker| text.contains(marker))
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
    if fail.status == 0 {
        return dropped_reply();
    }
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
