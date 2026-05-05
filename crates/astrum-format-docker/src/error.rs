use astrum_content::ContentError;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

#[derive(Debug)]
pub struct RegistryError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    detail: Value,
    headers: Vec<(HeaderName, String)>,
}

impl RegistryError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            detail: Value::Null,
            headers: Vec::new(),
        }
    }

    pub fn detail(mut self, detail: Value) -> Self {
        self.detail = detail;
        self
    }

    pub fn header(mut self, name: HeaderName, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    pub fn blob_unknown(digest: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "BLOB_UNKNOWN",
            "blob unknown to registry",
        )
        .detail(json!({"digest": digest}))
    }

    pub fn manifest_unknown(name: &str, reference: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "MANIFEST_UNKNOWN",
            "manifest unknown",
        )
        .detail(json!({"name": name, "reference": reference}))
    }

    pub fn name_unknown(name: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "NAME_UNKNOWN",
            "repository name not known to registry",
        )
        .detail(json!({"name": name}))
    }

    pub fn name_invalid(name: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "NAME_INVALID",
            "invalid repository name",
        )
        .detail(json!({"name": name}))
    }

    pub fn digest_invalid(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "DIGEST_INVALID", message)
    }

    pub fn manifest_invalid(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "MANIFEST_INVALID", message)
    }

    pub fn manifest_blob_unknown(digest: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "MANIFEST_BLOB_UNKNOWN",
            "manifest references a manifest or blob unknown to registry",
        )
        .detail(json!({"digest": digest}))
    }

    pub fn upload_unknown() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "BLOB_UPLOAD_UNKNOWN",
            "blob upload unknown to registry",
        )
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(StatusCode::METHOD_NOT_ALLOWED, "UNSUPPORTED", message)
    }

    pub fn denied(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "DENIED", message)
    }

    pub fn refused(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "DENIED", message)
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            "authentication required",
        )
    }

    pub fn too_large(limit: usize) -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "MANIFEST_INVALID",
            format!("a manifest may hold at most {limit} bytes"),
        )
    }
}

impl From<ContentError> for RegistryError {
    fn from(error: ContentError) -> Self {
        match error {
            ContentError::NotFound(message) => {
                Self::new(StatusCode::NOT_FOUND, "NAME_UNKNOWN", message)
            }
            ContentError::Unauthorized => Self::unauthorized(),
            ContentError::Forbidden => Self::denied("requested access to the resource is denied"),
            ContentError::MethodNotAllowed { method, allow } => {
                Self::unsupported(format!("{method} is not allowed here"))
                    .header(header::ALLOW, allow)
            }
            ContentError::BadRequest(message) | ContentError::Conflict(message) => {
                Self::refused(message)
            }
            ContentError::RangeNotSatisfiable(size) => Self::new(
                StatusCode::RANGE_NOT_SATISFIABLE,
                "UNSUPPORTED",
                format!("the range is outside a blob of {size} bytes"),
            )
            .header(header::CONTENT_RANGE, format!("bytes */{size}")),
            ContentError::PreconditionFailed => Self::new(
                StatusCode::PRECONDITION_FAILED,
                "UNSUPPORTED",
                ContentError::PreconditionFailed.to_string(),
            ),
            ContentError::Unavailable(message) => {
                Self::new(StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE", message)
            }
            ContentError::BadGateway(message) => {
                Self::new(StatusCode::BAD_GATEWAY, "UNAVAILABLE", message)
            }
            ContentError::Internal(error) => {
                tracing::error!(error = format!("{error:#}"), "a registry request failed");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "UNKNOWN",
                    "the server failed; see its log",
                )
            }
        }
    }
}

impl From<sea_orm::DbErr> for RegistryError {
    fn from(error: sea_orm::DbErr) -> Self {
        ContentError::from(error).into()
    }
}

impl From<astrum_blob::Error> for RegistryError {
    fn from(error: astrum_blob::Error) -> Self {
        ContentError::from(error).into()
    }
}

impl IntoResponse for RegistryError {
    fn into_response(self) -> Response {
        let mut entry = json!({"code": self.code, "message": self.message});
        if !self.detail.is_null() {
            entry["detail"] = self.detail;
        }
        let body = json!({"errors": [entry]});
        let mut response = Response::builder()
            .status(self.status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap_or_else(|_| self.status.into_response());
        for (name, value) in self.headers {
            if let Ok(value) = HeaderValue::from_str(&value) {
                response.headers_mut().append(name, value);
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_errors_become_registry_errors() {
        let unauthorized = RegistryError::from(ContentError::Unauthorized);
        assert_eq!(unauthorized.status, StatusCode::UNAUTHORIZED);
        assert_eq!(unauthorized.code, "UNAUTHORIZED");
        let forbidden = RegistryError::from(ContentError::Forbidden);
        assert_eq!(
            (forbidden.status, forbidden.code),
            (StatusCode::FORBIDDEN, "DENIED")
        );
        let range = RegistryError::from(ContentError::RangeNotSatisfiable(12)).into_response();
        assert_eq!(range.status(), StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(range.headers()[header::CONTENT_RANGE], "bytes */12");
        let upstream = RegistryError::from(ContentError::BadGateway("down".to_owned()));
        assert_eq!(upstream.status, StatusCode::BAD_GATEWAY);
    }
}
