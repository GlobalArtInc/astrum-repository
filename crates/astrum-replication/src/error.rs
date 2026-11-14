use std::error::Error as StdError;
use std::time::{Duration, SystemTime};

use reqwest::StatusCode;

use crate::protocol::{BLOBS_MISSING, ErrorBody, valid_sha256};

pub const MAX_RETRY_AFTER: Duration = Duration::from_secs(3600);
pub const MIN_RETRY_AFTER: Duration = Duration::from_secs(1);
pub const ERROR_BODY_LIMIT: usize = 64 * 1024;
pub const MESSAGE_LIMIT: usize = 500;
pub const BACKOFF_BASE: Duration = Duration::from_secs(30);
pub const BACKOFF_CAP: Duration = Duration::from_secs(3600);
pub const BACKOFF_JITTER: f64 = 0.2;
const MAX_DOUBLINGS: u32 = 16;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("temporary failure: {reason}")]
    Retryable {
        reason: String,
        retry_after: Option<Duration>,
    },
    #[error("the remote refused with status {status} ({code}): {message}")]
    Permanent {
        status: u16,
        code: String,
        message: String,
    },
    #[error("the remote lacks {} blob(s) the component needs", .0.len())]
    BlobsMissing(Vec<String>),
    #[error("{0}")]
    Refused(String),
}

pub fn clean(text: &str) -> String {
    let spaced: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    spaced.trim().chars().take(MESSAGE_LIMIT).collect()
}

pub fn parse_retry_after(value: Option<&str>) -> Option<Duration> {
    parse_retry_after_at(value, SystemTime::now())
}

fn parse_retry_after_at(value: Option<&str>, now: SystemTime) -> Option<Duration> {
    let text = value?.trim();
    let wanted = match text.parse::<u64>() {
        Ok(seconds) => Duration::from_secs(seconds),
        Err(_) => httpdate::parse_http_date(text)
            .ok()
            .map(|at| at.duration_since(now).unwrap_or(Duration::ZERO))?,
    };
    Some(wanted.clamp(MIN_RETRY_AFTER, MAX_RETRY_AFTER))
}

pub fn backoff(attempts: u32) -> Duration {
    backoff_with(attempts, rand::random::<f64>)
}

pub fn backoff_with(attempts: u32, random: impl FnOnce() -> f64) -> Duration {
    let doubled = BACKOFF_BASE.saturating_mul(1 << attempts.min(MAX_DOUBLINGS));
    let capped = doubled.min(BACKOFF_CAP);
    let unit = random().clamp(0.0, 1.0);
    let factor = 1.0 - BACKOFF_JITTER + 2.0 * BACKOFF_JITTER * unit;
    capped.mul_f64(factor).min(BACKOFF_CAP)
}

impl Error {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::Retryable { .. })
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::Retryable { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Error::Retryable { .. } => "retryable",
            Error::Permanent { .. } | Error::BlobsMissing(_) => "permanent",
            Error::Refused(_) => "refused",
        }
    }

    pub fn redacted(self, secret: &str) -> Self {
        if secret.is_empty() {
            return self;
        }
        let hide = |text: String| text.replace(secret, "[redacted]");
        match self {
            Error::Retryable {
                reason,
                retry_after,
            } => Error::Retryable {
                reason: hide(reason),
                retry_after,
            },
            Error::Permanent {
                status,
                code,
                message,
            } => Error::Permanent {
                status,
                code: hide(code),
                message: hide(message),
            },
            Error::Refused(text) => Error::Refused(hide(text)),
            blobs @ Error::BlobsMissing(_) => blobs,
        }
    }

    pub fn from_status(status: StatusCode, retry_after: Option<Duration>, body: &[u8]) -> Self {
        if status.is_redirection() {
            return Error::Refused("the remote redirected".to_owned());
        }
        let parsed: Option<ErrorBody> = serde_json::from_slice(body).ok();
        let (code, message) = match &parsed {
            Some(parsed) => (clean(&parsed.code), clean(&parsed.message)),
            None => (String::new(), clean(&String::from_utf8_lossy(body))),
        };
        if status.is_server_error()
            || status == StatusCode::TOO_MANY_REQUESTS
            || status == StatusCode::REQUEST_TIMEOUT
        {
            let reason = match message.is_empty() {
                true => format!("the remote answered {}", status.as_u16()),
                false => format!("the remote answered {}: {message}", status.as_u16()),
            };
            return Error::Retryable {
                reason,
                retry_after,
            };
        }
        if status == StatusCode::CONFLICT
            && code == BLOBS_MISSING
            && let Some(listed) = parsed.and_then(|parsed| parsed.missing)
        {
            let digests: Vec<String> = listed
                .into_iter()
                .filter(|digest| valid_sha256(digest))
                .collect();
            if !digests.is_empty() {
                return Error::BlobsMissing(digests);
            }
        }
        Error::Permanent {
            status: status.as_u16(),
            code: if code.is_empty() {
                "unknown".to_owned()
            } else {
                code
            },
            message,
        }
    }

    pub(crate) fn transport(error: &reqwest::Error) -> Self {
        let source: &(dyn StdError + 'static) = error;
        if astrum_security::ssrf::is_blocked(source) {
            return Error::Refused(
                "the address of the remote is blocked by the SSRF protection".to_owned(),
            );
        }
        if error.is_redirect() {
            return Error::Refused("the remote redirected".to_owned());
        }
        if error.is_builder() {
            return Error::Refused("the request to the remote could not be built".to_owned());
        }
        let detail = detail(error);
        let reason = if error.is_timeout() {
            "the request to the remote timed out".to_owned()
        } else if error.is_connect() {
            format!("the connection to the remote failed: {detail}")
        } else {
            format!("the exchange with the remote failed: {detail}")
        };
        Error::Retryable {
            reason,
            retry_after: None,
        }
    }
}

fn detail(error: &reqwest::Error) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut next = StdError::source(error);
    while let Some(cause) = next {
        parts.push(cause.to_string());
        next = cause.source();
    }
    clean(&parts.join(": "))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn status(code: u16) -> StatusCode {
        StatusCode::from_u16(code).unwrap()
    }

    fn body(code: &str, message: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"code": code, "message": message})).unwrap()
    }

    #[test]
    fn server_errors_and_throttling_are_retryable_and_the_hint_is_kept() {
        for code in [500, 502, 503, 504, 429, 408] {
            let error = Error::from_status(status(code), Some(Duration::from_secs(7)), b"busy");
            assert!(error.is_retryable(), "{code}");
            assert_eq!(error.retry_after(), Some(Duration::from_secs(7)), "{code}");
            assert_eq!(error.kind(), "retryable");
        }
    }

    #[test]
    fn client_errors_are_permanent_and_carry_the_receiver_code() {
        for code in [400, 401, 403, 404, 409, 413, 422, 418] {
            let error = Error::from_status(status(code), None, &body("nope", "not like this"));
            assert!(!error.is_retryable(), "{code}");
            match error {
                Error::Permanent {
                    status,
                    code: reported,
                    message,
                } => {
                    assert_eq!(status, code);
                    assert_eq!(reported, "nope");
                    assert_eq!(message, "not like this");
                }
                other => panic!("{code}: {other:?}"),
            }
        }
        match Error::from_status(status(403), None, b"<html>denied</html>") {
            Error::Permanent { code, message, .. } => {
                assert_eq!(code, "unknown");
                assert_eq!(message, "<html>denied</html>");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_conflict_naming_blobs_is_an_actionable_answer() {
        let digest = "ab".repeat(32);
        let wire = serde_json::to_vec(&json!({
            "code": BLOBS_MISSING, "message": "m", "missing": [digest, "not a digest"]
        }))
        .unwrap();
        match Error::from_status(status(409), None, &wire) {
            Error::BlobsMissing(list) => assert_eq!(list, vec!["ab".repeat(32)]),
            other => panic!("{other:?}"),
        }
        let empty = serde_json::to_vec(&json!({"code": BLOBS_MISSING, "missing": []})).unwrap();
        assert!(matches!(
            Error::from_status(status(409), None, &empty),
            Error::Permanent { status: 409, .. }
        ));
        assert!(matches!(
            Error::from_status(status(409), None, &body("conflict", "exists")),
            Error::Permanent { status: 409, .. }
        ));
    }

    #[test]
    fn a_redirect_is_a_local_refusal() {
        assert!(matches!(
            Error::from_status(status(302), None, b""),
            Error::Refused(text) if text == "the remote redirected"
        ));
    }

    #[test]
    fn the_hint_is_read_in_seconds_and_capped_at_one_hour() {
        assert_eq!(
            parse_retry_after(Some(" 12 ")),
            Some(Duration::from_secs(12))
        );
        assert_eq!(parse_retry_after(Some("999999")), Some(MAX_RETRY_AFTER));
        assert_eq!(parse_retry_after(Some("0")), Some(MIN_RETRY_AFTER));
        let date = Some("Wed, 21 Oct 2026 07:28:00 GMT");
        let at = |text: &str| httpdate::parse_http_date(text).unwrap();
        assert_eq!(
            parse_retry_after_at(date, at("Wed, 21 Oct 2026 07:26:30 GMT")),
            Some(Duration::from_secs(90))
        );
        assert_eq!(
            parse_retry_after_at(date, at("Wed, 21 Oct 2026 05:00:00 GMT")),
            Some(MAX_RETRY_AFTER)
        );
        assert_eq!(
            parse_retry_after_at(date, at("Wed, 21 Oct 2026 09:00:00 GMT")),
            Some(MIN_RETRY_AFTER)
        );
        assert_eq!(parse_retry_after(Some("tomorrow")), None);
        assert_eq!(parse_retry_after(Some("-5")), None);
        assert_eq!(parse_retry_after(None), None);
    }

    #[test]
    fn answers_are_cleaned_and_cut_before_they_are_shown() {
        let noisy = format!("line\none\u{1b}[31m{}", "x".repeat(2000));
        let shown = clean(&noisy);
        assert!(!shown.chars().any(char::is_control));
        assert_eq!(shown.chars().count(), MESSAGE_LIMIT);
        assert!(shown.starts_with("line one [31m"));
    }

    #[test]
    fn a_secret_echoed_by_the_remote_is_hidden() {
        let error = Error::from_status(
            status(401),
            None,
            &body("bad", "token hunter2-xyz is wrong"),
        )
        .redacted("hunter2-xyz");
        assert!(!format!("{error} {error:?}").contains("hunter2-xyz"));
    }

    #[test]
    fn backoff_doubles_from_thirty_seconds_and_stops_at_one_hour() {
        let middle = || 0.5;
        let seconds = |attempts| backoff_with(attempts, middle).as_secs();
        assert_eq!(seconds(0), 30);
        assert_eq!(seconds(1), 60);
        assert_eq!(seconds(2), 120);
        assert_eq!(seconds(6), 1920);
        assert_eq!(seconds(7), 3600);
        assert_eq!(seconds(11), 3600);
        assert_eq!(seconds(u32::MAX), 3600);
    }

    #[test]
    fn jitter_stays_within_twenty_percent_and_never_passes_the_cap() {
        assert_eq!(backoff_with(0, || 0.0), Duration::from_secs(24));
        assert_eq!(backoff_with(0, || 1.0), Duration::from_secs(36));
        assert_eq!(backoff_with(3, || 0.0), Duration::from_secs(192));
        assert_eq!(backoff_with(3, || 1.0), Duration::from_secs(288));
        assert_eq!(backoff_with(9, || 0.0), Duration::from_secs(2880));
        assert_eq!(backoff_with(9, || 1.0), BACKOFF_CAP);
        assert_eq!(backoff_with(0, || 7.0), Duration::from_secs(36));
        assert_eq!(backoff_with(0, || -3.0), Duration::from_secs(24));
        for attempts in 0..20 {
            let delay = backoff(attempts);
            assert!(delay <= BACKOFF_CAP, "{attempts}: {delay:?}");
            assert!(delay >= Duration::from_secs(24), "{attempts}: {delay:?}");
        }
    }

    #[tokio::test]
    async fn a_timeout_and_a_refused_connection_are_retryable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let silent = listener.local_addr().unwrap();
        let holder = tokio::spawn(async move {
            let _held = listener.accept().await;
            tokio::time::sleep(Duration::from_secs(30)).await;
        });
        let quick = reqwest::Client::builder()
            .timeout(Duration::from_millis(150))
            .build()
            .unwrap();
        let timed_out = quick
            .get(format!("http://{silent}/"))
            .send()
            .await
            .unwrap_err();
        holder.abort();
        match Error::transport(&timed_out) {
            Error::Retryable { reason, .. } => assert!(reason.contains("timed out"), "{reason}"),
            other => panic!("{other:?}"),
        }
        let closed = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let gone = closed.local_addr().unwrap();
        drop(closed);
        let refused = quick
            .get(format!("http://{gone}/"))
            .send()
            .await
            .unwrap_err();
        match Error::transport(&refused) {
            Error::Retryable { reason, .. } => assert!(reason.contains("failed"), "{reason}"),
            other => panic!("{other:?}"),
        }
    }
}
