use std::future::Future;
use std::io::{Read, Write};

use astrum_content::{ContentError, ContentRequest};
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use prost::Message;
use serde_json::json;

pub const PROTO: &str = "application/proto";
pub const MAX_REQUEST: usize = 64 * 1024 * 1024;
const GZIP_FROM: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    InvalidArgument,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    FailedPrecondition,
    Unimplemented,
    Internal,
    Unavailable,
    Unauthenticated,
}

impl Code {
    pub fn name(self) -> &'static str {
        match self {
            Code::InvalidArgument => "invalid_argument",
            Code::NotFound => "not_found",
            Code::AlreadyExists => "already_exists",
            Code::PermissionDenied => "permission_denied",
            Code::FailedPrecondition => "failed_precondition",
            Code::Unimplemented => "unimplemented",
            Code::Internal => "internal",
            Code::Unavailable => "unavailable",
            Code::Unauthenticated => "unauthenticated",
        }
    }

    pub fn status(self) -> StatusCode {
        match self {
            Code::InvalidArgument | Code::FailedPrecondition => StatusCode::BAD_REQUEST,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::AlreadyExists => StatusCode::CONFLICT,
            Code::PermissionDenied => StatusCode::FORBIDDEN,
            Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub code: Code,
    pub message: String,
}

impl Status {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(Code::InvalidArgument, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(Code::NotFound, message)
    }

    pub fn exists(message: impl Into<String>) -> Self {
        Self::new(Code::AlreadyExists, message)
    }

    pub fn precondition(message: impl Into<String>) -> Self {
        Self::new(Code::FailedPrecondition, message)
    }

    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "a protobuf registry request failed");
        Self::new(Code::Internal, "the server failed; see its log")
    }
}

impl From<ContentError> for Status {
    fn from(error: ContentError) -> Self {
        match error {
            ContentError::NotFound(message) => Status::not_found(message),
            ContentError::Unauthorized => {
                Status::new(Code::Unauthenticated, "authentication is required")
            }
            ContentError::Forbidden => Status::new(
                Code::PermissionDenied,
                "you do not have the permission this needs",
            ),
            ContentError::BadRequest(message) => Status::invalid(message),
            ContentError::Conflict(message) => Status::exists(message),
            ContentError::MethodNotAllowed { method, .. } => {
                Status::invalid(format!("{method} is not allowed here"))
            }
            ContentError::RangeNotSatisfiable(_) | ContentError::PreconditionFailed => {
                Status::precondition("a precondition of the request does not hold")
            }
            ContentError::Unavailable(message) | ContentError::BadGateway(message) => {
                Status::new(Code::Unavailable, message)
            }
            ContentError::Internal(error) => Status::internal(format!("{error:#}")),
        }
    }
}

impl From<sea_orm::DbErr> for Status {
    fn from(error: sea_orm::DbErr) -> Self {
        Status::internal(error)
    }
}

pub type Reply<T> = Result<T, Status>;

pub fn failure(status: &Status) -> Response {
    let body = json!({"code": status.code.name(), "message": status.message}).to_string();
    Response::builder()
        .status(status.code.status())
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(body))
        .expect("a Connect error is a valid response")
}

fn plain(status: StatusCode, allow: Option<&'static str>, message: &str) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain;charset=utf-8");
    if let Some(allow) = allow {
        builder = builder.header(header::ALLOW, allow);
    }
    builder
        .body(Body::from(message.to_owned()))
        .expect("a plain answer is valid")
}

fn media_type(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn gunzip(bytes: &[u8]) -> Reply<Vec<u8>> {
    let mut decoded = Vec::new();
    GzDecoder::new(bytes)
        .take(MAX_REQUEST as u64 + 1)
        .read_to_end(&mut decoded)
        .map_err(|error| Status::invalid(format!("the request is not valid gzip: {error}")))?;
    if decoded.len() > MAX_REQUEST {
        return Err(Status::invalid("the request is too large"));
    }
    Ok(decoded)
}

pub fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(bytes)
        .expect("writing to memory cannot fail");
    encoder.finish().expect("writing to memory cannot fail")
}

fn decompress(encoding: &str, bytes: Vec<u8>) -> Reply<Vec<u8>> {
    match encoding.trim().to_ascii_lowercase().as_str() {
        "" | "identity" => Ok(bytes),
        "gzip" => gunzip(&bytes),
        other => Err(Status::new(
            Code::Unimplemented,
            format!("the compression {other} is not supported; use gzip or none"),
        )),
    }
}

pub fn unpack_query(query: &str) -> Reply<Vec<u8>> {
    let mut message = None;
    let mut encoding = None;
    let mut compression = String::new();
    let mut base64 = false;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "message" => message = Some(value.into_owned()),
            "encoding" => encoding = Some(value.into_owned()),
            "compression" => compression = value.into_owned(),
            "base64" => base64 = value == "1",
            _ => {}
        }
    }
    if encoding.as_deref() != Some("proto") {
        return Err(Status::invalid("the query must set encoding=proto"));
    }
    let message = message.unwrap_or_default();
    let bytes = match base64 {
        true => URL_SAFE_NO_PAD
            .decode(message.trim_end_matches('='))
            .map_err(|_| Status::invalid("the message is not valid base64"))?,
        false => message.into_bytes(),
    };
    decompress(&compression, bytes)
}

pub async fn read(request: &ContentRequest) -> Result<Vec<u8>, Box<Response>> {
    match request.method {
        Method::GET => {
            let query = request.query.clone().unwrap_or_default();
            unpack_query(&query).map_err(|status| Box::new(failure(&status)))
        }
        Method::POST => {
            let kind = request.header("content-type").map(media_type);
            if kind.as_deref() != Some(PROTO) {
                return Err(Box::new(plain(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    None,
                    "this registry speaks the Connect protocol with application/proto messages",
                )));
            }
            let body = request
                .bytes(MAX_REQUEST)
                .await
                .map_err(|error| Box::new(failure(&Status::from(error))))?;
            let encoding = request.header("content-encoding").unwrap_or_default();
            decompress(encoding, body.to_vec()).map_err(|status| Box::new(failure(&status)))
        }
        _ => Err(Box::new(plain(
            StatusCode::METHOD_NOT_ALLOWED,
            Some("GET,POST"),
            "Connect calls use POST, or GET when the method has no side effects",
        ))),
    }
}

fn accepts_gzip(request: &ContentRequest) -> bool {
    request
        .header("accept-encoding")
        .is_some_and(|value| value.split(',').any(|part| part.trim().starts_with("gzip")))
}

pub fn success<M: Message>(request: &ContentRequest, message: &M) -> Response {
    let mut body = message.encode_to_vec();
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, PROTO)
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::VARY, "Accept-Encoding");
    if body.len() >= GZIP_FROM && accepts_gzip(request) {
        body = gzip(&body);
        builder = builder.header(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
    }
    builder
        .header(header::CONTENT_LENGTH, body.len())
        .body(Body::from(body))
        .expect("a Connect answer is a valid response")
}

pub async fn unary<Q, R, F, Fut>(
    request: &ContentRequest,
    side_effects: bool,
    handler: F,
) -> Response
where
    Q: Message + Default,
    R: Message,
    F: FnOnce(Q) -> Fut,
    Fut: Future<Output = Reply<R>>,
{
    if side_effects && request.method != Method::POST {
        return plain(
            StatusCode::METHOD_NOT_ALLOWED,
            Some("POST"),
            "this method changes data and takes POST only",
        );
    }
    let bytes = match read(request).await {
        Ok(bytes) => bytes,
        Err(response) => return *response,
    };
    let message = match Q::decode(bytes.as_slice()) {
        Ok(message) => message,
        Err(error) => {
            return failure(&Status::invalid(format!(
                "the request message cannot be decoded: {error}"
            )));
        }
    };
    match handler(message).await {
        Ok(reply) => success(request, &reply),
        Err(status) => failure(&status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_map_to_the_http_statuses_of_the_protocol() {
        assert_eq!(Code::NotFound.status(), StatusCode::NOT_FOUND);
        assert_eq!(Code::Unauthenticated.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(Code::PermissionDenied.status(), StatusCode::FORBIDDEN);
        assert_eq!(Code::AlreadyExists.status(), StatusCode::CONFLICT);
        assert_eq!(Code::InvalidArgument.status(), StatusCode::BAD_REQUEST);
        assert_eq!(Code::FailedPrecondition.status(), StatusCode::BAD_REQUEST);
        assert_eq!(Code::Unimplemented.status(), StatusCode::NOT_IMPLEMENTED);
        assert_eq!(Code::Unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(Code::Internal.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(Code::FailedPrecondition.name(), "failed_precondition");
    }

    #[test]
    fn an_error_is_a_json_body_with_the_code_and_the_message() {
        let response = failure(&Status::not_found("no such module"));
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn content_errors_become_connect_codes() {
        assert_eq!(
            Status::from(ContentError::Unauthorized).code,
            Code::Unauthenticated
        );
        assert_eq!(
            Status::from(ContentError::Forbidden).code,
            Code::PermissionDenied
        );
        assert_eq!(
            Status::from(ContentError::not_found("x")).code,
            Code::NotFound
        );
        assert_eq!(
            Status::from(ContentError::BadRequest("x".to_owned())).code,
            Code::InvalidArgument
        );
        assert_eq!(
            Status::from(ContentError::Conflict("x".to_owned())).code,
            Code::AlreadyExists
        );
    }

    #[test]
    fn a_get_request_carries_the_message_in_the_query() {
        let message = [10u8, 3, b'a', b'b', b'c'];
        let encoded = URL_SAFE_NO_PAD.encode(message);
        let query = format!("connect=v1&encoding=proto&base64=1&message={encoded}");
        assert_eq!(unpack_query(&query).unwrap(), message);
        let padded = format!("encoding=proto&base64=1&message={encoded}%3D%3D");
        assert_eq!(unpack_query(&padded).unwrap(), message);
    }

    #[test]
    fn a_get_request_may_be_compressed() {
        let message = vec![7u8; 200];
        let packed = gzip(&message);
        let encoded = URL_SAFE_NO_PAD.encode(packed);
        let query = format!("encoding=proto&base64=1&compression=gzip&message={encoded}");
        assert_eq!(unpack_query(&query).unwrap(), message);
    }

    #[test]
    fn a_get_request_needs_the_proto_encoding() {
        assert_eq!(
            unpack_query("encoding=json&message=e30").unwrap_err().code,
            Code::InvalidArgument
        );
        assert_eq!(
            unpack_query("message=AA").unwrap_err().code,
            Code::InvalidArgument
        );
        assert_eq!(
            unpack_query("encoding=proto&base64=1&message=***")
                .unwrap_err()
                .code,
            Code::InvalidArgument
        );
    }

    #[test]
    fn unknown_compressions_are_refused_and_gzip_round_trips() {
        assert_eq!(
            decompress("br", vec![1]).unwrap_err().code,
            Code::Unimplemented
        );
        assert_eq!(decompress("identity", vec![1, 2]).unwrap(), vec![1, 2]);
        let packed = gzip(b"hello hello hello");
        assert_eq!(decompress("gzip", packed).unwrap(), b"hello hello hello");
        assert_eq!(
            decompress("gzip", vec![1, 2, 3]).unwrap_err().code,
            Code::InvalidArgument
        );
    }

    #[test]
    fn media_types_are_compared_without_parameters() {
        assert_eq!(media_type("Application/Proto; charset=utf-8"), PROTO);
        assert_eq!(media_type("application/json"), "application/json");
    }
}
