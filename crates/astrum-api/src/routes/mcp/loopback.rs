use astrum_mcp::ToolError;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Method, Request, StatusCode, header};
use futures_util::StreamExt;
use serde_json::Value;
use tower::ServiceExt;

use crate::{AppState, Caller};

const FORWARDED: [&str; 7] = [
    "host",
    "x-forwarded-host",
    "x-forwarded-proto",
    "x-forwarded-port",
    "x-forwarded-for",
    "x-real-ip",
    "forwarded",
];
const JSON_LIMIT: usize = 16 * 1024 * 1024;

pub struct Fetched {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub truncated: bool,
}

pub struct Loopback {
    state: AppState,
    caller: Caller,
    forwarded: HeaderMap,
}

impl Loopback {
    pub fn new(state: &AppState, caller: Caller, headers: &HeaderMap) -> Self {
        let mut forwarded = HeaderMap::new();
        for name in FORWARDED {
            if let Some(value) = headers.get(name) {
                forwarded.insert(name, value.clone());
            }
        }
        Self {
            state: state.clone(),
            caller,
            forwarded,
        }
    }

    pub async fn fetch(
        &self,
        method: Method,
        uri: &str,
        body: Option<&Value>,
        limit: usize,
    ) -> Result<Fetched, ToolError> {
        let mut request = Request::builder().method(method).uri(uri);
        for (name, value) in &self.forwarded {
            request = request.header(name, value);
        }
        let body = match body {
            Some(value) => {
                request = request.header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
                Body::from(serde_json::to_vec(value).unwrap_or_default())
            }
            None => Body::empty(),
        };
        let mut request = request
            .body(body)
            .map_err(|error| ToolError::failed(format!("the request cannot be built: {error}")))?;
        request.extensions_mut().insert(self.caller.clone());
        let response = crate::loopback(&self.state)
            .oneshot(request)
            .await
            .map_err(|error| ToolError::failed(format!("the request failed: {error}")))?;
        let (parts, body) = response.into_parts();
        let mut stream = body.into_data_stream();
        let mut bytes = Vec::new();
        let mut truncated = false;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|error| ToolError::failed(format!("the answer broke off: {error}")))?;
            let room = limit.saturating_sub(bytes.len());
            if chunk.len() > room {
                bytes.extend_from_slice(&chunk[..room]);
                truncated = true;
                break;
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Fetched {
            status: parts.status,
            headers: parts.headers,
            body: bytes,
            truncated,
        })
    }

    pub async fn json(
        &self,
        method: Method,
        uri: &str,
        body: Option<&Value>,
    ) -> Result<Value, ToolError> {
        let fetched = self.fetch(method, uri, body, JSON_LIMIT).await?;
        if !fetched.status.is_success() {
            return Err(refusal(&fetched));
        }
        if fetched.body.is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_slice(&fetched.body).map_err(|error| {
            ToolError::failed(format!("the server answered something unexpected: {error}"))
        })
    }

    pub async fn get(&self, uri: &str) -> Result<Value, ToolError> {
        self.json(Method::GET, uri, None).await
    }
}

pub fn refusal(fetched: &Fetched) -> ToolError {
    let detail = match serde_json::from_slice::<Value>(&fetched.body) {
        Ok(Value::Array(problems)) => problems.iter().map(problem).collect::<Vec<_>>().join("; "),
        Ok(value @ Value::Object(_)) => problem(&value),
        _ => String::from_utf8_lossy(&fetched.body).trim().to_owned(),
    };
    let status = fetched.status;
    ToolError::failed(match detail.is_empty() {
        true => format!("Astrum answered {status}"),
        false => format!("Astrum answered {status}: {detail}"),
    })
}

fn problem(value: &Value) -> String {
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string());
    match value.get("id").and_then(Value::as_str) {
        Some(id) if id != "*" => format!("{id}: {message}"),
        _ => message,
    }
}
