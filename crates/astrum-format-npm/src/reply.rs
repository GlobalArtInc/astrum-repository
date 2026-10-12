use astrum_content::{ContentError, ContentRequest};
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, FixedOffset, Utc};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};

pub const JSON: &str = "application/json";
pub const ABBREVIATED: &str = "application/vnd.npm.install-v1+json";

pub fn json(status: StatusCode, value: &Value) -> Response {
    let body = serde_json::to_vec(value).unwrap_or_default();
    (status, [(header::CONTENT_TYPE, JSON)], body).into_response()
}

pub fn success() -> Response {
    json(StatusCode::OK, &json!({"success": true}))
}

pub fn error(error: ContentError) -> Response {
    let message = match &error {
        ContentError::Internal(_) => "the server failed; see its log".to_owned(),
        other => other.to_string(),
    };
    let mut response = error.into_response();
    *response.body_mut() = Body::from(
        serde_json::to_vec(&json!({"success": false, "error": message})).unwrap_or_default(),
    );
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(JSON));
    response
}

pub fn wants_abbreviated(request: &ContentRequest) -> bool {
    let Some(accept) = request.header("accept") else {
        return false;
    };
    let mut abbreviated = None;
    let mut full = None;
    for entry in accept.split(',') {
        let mut parts = entry.split(';');
        let kind = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
        let quality = parts
            .filter_map(|part| part.trim().strip_prefix("q="))
            .find_map(|value| value.trim().parse::<f32>().ok())
            .unwrap_or(1.0);
        match kind.as_str() {
            ABBREVIATED => abbreviated = Some(quality),
            JSON => full = Some(quality),
            _ => {}
        }
    }
    match abbreviated {
        Some(quality) if quality > 0.0 => full.is_none_or(|full| quality >= full),
        _ => false,
    }
}

pub fn document(
    request: &ContentRequest,
    value: &Value,
    content_type: &'static str,
    modified: Option<DateTime<FixedOffset>>,
) -> Result<Response, ContentError> {
    let body = serde_json::to_vec(value).map_err(ContentError::internal)?;
    let etag = format!("\"{}\"", hex::encode(Sha1::digest(&body)));
    let mut builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::VARY, "Accept")
        .header(header::CACHE_CONTROL, "no-cache");
    if let Some(modified) = modified {
        builder = builder.header(
            header::LAST_MODIFIED,
            httpdate::fmt_http_date(modified.with_timezone(&Utc).into()),
        );
    }
    let matches = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == etag || tag == "*")
    });
    if matches {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    builder = builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len());
    let body = match request.method {
        Method::HEAD => Body::empty(),
        _ => Body::from(body),
    };
    builder.body(body).map_err(ContentError::internal)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use super::*;

    fn request(accept: &str) -> ContentRequest {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, accept.parse().unwrap());
        ContentRequest::new(Method::GET, "/tool", None, headers, Body::empty()).unwrap()
    }

    #[test]
    fn installs_get_the_abbreviated_document() {
        assert!(wants_abbreviated(&request(
            "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*"
        )));
        assert!(wants_abbreviated(&request(
            "application/vnd.npm.install-v1+json"
        )));
        assert!(!wants_abbreviated(&request("application/json")));
        assert!(!wants_abbreviated(&request("*/*")));
        assert!(!wants_abbreviated(&request(
            "application/vnd.npm.install-v1+json; q=0.5, application/json"
        )));
    }
}
