use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::Value;

pub const JSON: &str = "application/json";
pub const XML: &str = "application/xml;charset=utf-8";
pub const FEED: &str = "application/atom+xml;type=feed;charset=utf-8";
pub const ENTRY: &str = "application/atom+xml;type=entry;charset=utf-8";
pub const TEXT: &str = "text/plain";

pub fn body(method: &Method, content_type: &str, text: String) -> Response {
    let length = text.len();
    let body = match *method == Method::HEAD {
        true => Body::empty(),
        false => Body::from(text),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .body(body)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub fn json(method: &Method, value: &Value) -> Response {
    body(method, JSON, value.to_string())
}

pub fn status(status: StatusCode) -> Response {
    status.into_response()
}
