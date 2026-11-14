use astrum_content::{CHALLENGE, ContentError};
use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const JSON_TYPE: &str = "application/json";

pub fn json(status: StatusCode, value: &Value) -> Response {
    let body = serde_json::to_vec(value).expect("a JSON value serialises");
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .body(Body::from(body))
        .expect("a JSON response is valid")
}

pub fn failure(status: StatusCode, code: &str, message: &str) -> Response {
    json(status, &json!({"code": code, "message": message}))
}

pub fn etag_of(body: &[u8]) -> String {
    format!("\"{}\"", hex::encode(&Sha256::digest(body)[..16]))
}

pub fn from_error(error: ContentError) -> Response {
    match error {
        ContentError::Unauthorized => {
            let mut response = failure(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication is required",
            );
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(CHALLENGE),
            );
            response
        }
        ContentError::Forbidden => failure(
            StatusCode::FORBIDDEN,
            "forbidden",
            "you do not have the permission this needs",
        ),
        ContentError::BadRequest(message) => {
            failure(StatusCode::BAD_REQUEST, "badRequest", &message)
        }
        ContentError::NotFound(message) => failure(StatusCode::NOT_FOUND, "notFound", &message),
        other => other.into_response(),
    }
}
