use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub const CHALLENGE: &str = "Basic realm=\"Astrum Repository\"";

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("{0}")]
    NotFound(String),
    #[error("authentication is required")]
    Unauthorized,
    #[error("you do not have the permission this needs")]
    Forbidden,
    #[error("{method} is not allowed here")]
    MethodNotAllowed { method: String, allow: &'static str },
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("the range is outside a blob of {0} bytes")]
    RangeNotSatisfiable(u64),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    BadGateway(String),
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

impl ContentError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ContentError::NotFound(message.into())
    }

    pub fn internal(error: impl Into<anyhow::Error>) -> Self {
        ContentError::Internal(error.into())
    }

    pub fn status(&self) -> StatusCode {
        match self {
            ContentError::NotFound(_) => StatusCode::NOT_FOUND,
            ContentError::Unauthorized => StatusCode::UNAUTHORIZED,
            ContentError::Forbidden => StatusCode::FORBIDDEN,
            ContentError::MethodNotAllowed { .. } => StatusCode::METHOD_NOT_ALLOWED,
            ContentError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ContentError::Conflict(_) => StatusCode::CONFLICT,
            ContentError::RangeNotSatisfiable(_) => StatusCode::RANGE_NOT_SATISFIABLE,
            ContentError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ContentError::BadGateway(_) => StatusCode::BAD_GATEWAY,
            ContentError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<astrum_db::Error> for ContentError {
    fn from(error: astrum_db::Error) -> Self {
        ContentError::internal(error)
    }
}

impl From<sea_orm::DbErr> for ContentError {
    fn from(error: sea_orm::DbErr) -> Self {
        ContentError::internal(error)
    }
}

impl From<astrum_blob::Error> for ContentError {
    fn from(error: astrum_blob::Error) -> Self {
        ContentError::internal(error)
    }
}

impl IntoResponse for ContentError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = match &self {
            ContentError::Internal(error) => {
                tracing::error!(error = format!("{error:#}"), "a content request failed");
                "the server failed; see its log".to_owned()
            }
            other => other.to_string(),
        };
        let mut response = Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "text/plain;charset=utf-8")
            .body(Body::from(message))
            .unwrap_or_else(|_| status.into_response());
        let headers = response.headers_mut();
        match self {
            ContentError::Unauthorized => {
                headers.insert(
                    header::WWW_AUTHENTICATE,
                    HeaderValue::from_static(CHALLENGE),
                );
            }
            ContentError::MethodNotAllowed { allow, .. } => {
                headers.insert(header::ALLOW, HeaderValue::from_static(allow));
            }
            ContentError::RangeNotSatisfiable(size) => {
                if let Ok(value) = HeaderValue::from_str(&format!("bytes */{size}")) {
                    headers.insert(header::CONTENT_RANGE, value);
                }
            }
            _ => {}
        }
        response
    }
}
