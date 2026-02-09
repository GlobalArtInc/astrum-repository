use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

pub const CHALLENGE: &str = "Basic realm=\"Astrum Repository\"";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub id: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("the request is not valid")]
    Invalid(Vec<FieldError>),
    #[error("{0}")]
    BadRequest(String),
    #[error("authentication is required")]
    Unauthorized,
    #[error("{0}")]
    Forbidden(String),
    #[error("the request body must be {0}")]
    UnsupportedMediaType(&'static str),
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError::NotFound(message.into())
    }

    pub fn invalid(id: impl Into<String>, message: impl Into<String>) -> Self {
        ApiError::Invalid(vec![FieldError {
            id: id.into(),
            message: message.into(),
        }])
    }

    pub fn forbidden() -> Self {
        ApiError::Forbidden("you do not have the permission this needs".to_owned())
    }

    pub fn internal(error: impl Into<anyhow::Error>) -> Self {
        ApiError::Internal(error.into())
    }
}

impl From<astrum_db::Error> for ApiError {
    fn from(error: astrum_db::Error) -> Self {
        ApiError::internal(error)
    }
}

impl From<astrum_security::Error> for ApiError {
    fn from(error: astrum_security::Error) -> Self {
        ApiError::internal(error)
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        ApiError::internal(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound(message) => {
                (StatusCode::NOT_FOUND, message_body(message)).into_response()
            }
            ApiError::Invalid(errors) => (StatusCode::BAD_REQUEST, Json(errors)).into_response(),
            ApiError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(vec![FieldError {
                    id: "*".to_owned(),
                    message,
                }]),
            )
                .into_response(),
            ApiError::Unauthorized => {
                let mut response = (
                    StatusCode::UNAUTHORIZED,
                    message_body("authentication is required"),
                )
                    .into_response();
                response.headers_mut().insert(
                    header::WWW_AUTHENTICATE,
                    HeaderValue::from_static(CHALLENGE),
                );
                response
            }
            ApiError::Forbidden(message) => {
                (StatusCode::FORBIDDEN, message_body(message)).into_response()
            }
            ApiError::UnsupportedMediaType(expected) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                message_body(format!("the request body must be {expected}")),
            )
                .into_response(),
            ApiError::Internal(error) => {
                let reference = reference();
                tracing::error!(reference, error = format!("{error:#}"), "a request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    message_body(format!("the server failed; the log names it {reference}")),
                )
                    .into_response()
            }
        }
    }
}

fn message_body(message: impl Into<String>) -> Json<serde_json::Value> {
    Json(json!({"id": "*", "message": message.into()}))
}

fn reference() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or_default();
    format!("{:x}-{}", nanos, NEXT.fetch_add(1, Ordering::Relaxed))
}

#[derive(Debug, Default)]
pub struct Validation(Vec<FieldError>);

impl Validation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.0.push(FieldError {
            id: id.into(),
            message: message.into(),
        });
    }

    pub fn require(&mut self, id: &str, value: Option<&str>) {
        if value.is_none_or(|value| value.trim().is_empty()) {
            self.add(id, "must not be empty");
        }
    }

    pub fn check(&mut self, ok: bool, id: &str, message: &str) {
        if !ok {
            self.add(id, message);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn finish(self) -> Result<(), ApiError> {
        match self.0.is_empty() {
            true => Ok(()),
            false => Err(ApiError::Invalid(self.0)),
        }
    }
}
