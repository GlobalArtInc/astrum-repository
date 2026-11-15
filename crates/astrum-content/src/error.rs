use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

pub const CHALLENGE: &str = "Basic realm=\"Astrum Repository\"";
pub const BLOCKED_BY: &str = "x-astrum-blocked-by";
pub const MAX_BLOCKED_TEXT: usize = 512;
const BLOCKED_PREFIX: &str = "blocked by policy ";

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
    #[error(
        "the repository {repository} would exceed its quota of {limit_bytes} bytes ({used_bytes} bytes are used)"
    )]
    QuotaExceeded {
        repository: String,
        limit_bytes: i64,
        used_bytes: i64,
    },
    #[error("{0}")]
    Protected(String),
    #[error("{0}")]
    Blocked(String),
    #[error("the range is outside a blob of {0} bytes")]
    RangeNotSatisfiable(u64),
    #[error("a precondition of the request does not hold")]
    PreconditionFailed,
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

    pub fn blocked_by(policy: &str, details: &str) -> Self {
        let text = format!("{BLOCKED_PREFIX}{policy}: {details}");
        let text = match text.chars().count() > MAX_BLOCKED_TEXT {
            true => {
                let mut cut: String = text.chars().take(MAX_BLOCKED_TEXT - 1).collect();
                cut.push('\u{2026}');
                cut
            }
            false => text,
        };
        ContentError::Blocked(text)
    }

    pub fn blocking_policy(&self) -> Option<&str> {
        match self {
            ContentError::Blocked(text) => text
                .strip_prefix(BLOCKED_PREFIX)
                .and_then(|rest| rest.split_once(": "))
                .map(|(policy, _)| policy)
                .filter(|policy| !policy.is_empty()),
            _ => None,
        }
    }

    pub fn mark(&self, response: &mut Response) {
        if let Some(policy) = self.blocking_policy()
            && let Ok(value) = HeaderValue::from_str(policy)
        {
            response.headers_mut().insert(BLOCKED_BY, value);
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            ContentError::NotFound(_) => StatusCode::NOT_FOUND,
            ContentError::Unauthorized => StatusCode::UNAUTHORIZED,
            ContentError::Forbidden | ContentError::Blocked(_) => StatusCode::FORBIDDEN,
            ContentError::MethodNotAllowed { .. } => StatusCode::METHOD_NOT_ALLOWED,
            ContentError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ContentError::Conflict(_) | ContentError::Protected(_) => StatusCode::CONFLICT,
            ContentError::QuotaExceeded { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            ContentError::RangeNotSatisfiable(_) => StatusCode::RANGE_NOT_SATISFIABLE,
            ContentError::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
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
        self.mark(&mut response);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_answers_forbidden_with_its_reason() {
        let reason = "blocked by policy strict: critical findings";
        let error = ContentError::Blocked(reason.to_owned());
        assert_eq!(error.status(), StatusCode::FORBIDDEN);
        assert_eq!(error.to_string(), reason);
        assert_eq!(error.into_response().status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn a_refusal_by_a_policy_names_it_in_a_header_and_other_refusals_do_not() {
        let error = ContentError::blocked_by("strict", "GHSA-1: high in npm/lib@1: fixed");
        assert_eq!(error.blocking_policy(), Some("strict"));
        assert_eq!(
            error.to_string(),
            "blocked by policy strict: GHSA-1: high in npm/lib@1: fixed"
        );
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(response.headers()[BLOCKED_BY], "strict");
        let long = ContentError::blocked_by("strict", &"x".repeat(2000));
        assert_eq!(long.to_string().chars().count(), MAX_BLOCKED_TEXT);
        assert!(long.to_string().ends_with('\u{2026}'));
        assert_eq!(long.blocking_policy(), Some("strict"));
        let exact = ContentError::blocked_by(
            "p",
            &"y".repeat(MAX_BLOCKED_TEXT - "blocked by policy p: ".len()),
        );
        assert_eq!(exact.to_string().chars().count(), MAX_BLOCKED_TEXT);
        assert!(!exact.to_string().ends_with('\u{2026}'));
        for other in [
            ContentError::Blocked("refused".to_owned()),
            ContentError::Blocked("blocked by policy ".to_owned()),
            ContentError::Blocked("blocked by policy : nothing".to_owned()),
            ContentError::Forbidden,
        ] {
            assert_eq!(other.blocking_policy(), None, "{other}");
            assert!(other.into_response().headers().get(BLOCKED_BY).is_none());
        }
    }
}
