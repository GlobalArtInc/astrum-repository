use std::time::SystemTime;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::Response;

use crate::store::Asset;
use crate::{ContentError, Ctx};

#[derive(Clone, Debug, Default)]
pub struct Serve {
    pub disposition: Option<&'static str>,
    pub content_type: Option<String>,
    pub count_download: bool,
    pub etag: Option<String>,
}

impl Serve {
    pub fn download() -> Self {
        Self {
            count_download: true,
            ..Default::default()
        }
    }

    pub fn disposition(mut self, disposition: &'static str) -> Self {
        self.disposition = Some(disposition);
        self
    }

    pub fn etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Wanted {
    Whole,
    Part(u64, u64),
    Unsatisfiable,
}

fn wanted(headers: &HeaderMap, size: u64, etag: &str, last_modified: SystemTime) -> Wanted {
    let Some(range) = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
    else {
        return Wanted::Whole;
    };
    if let Some(condition) = headers
        .get(header::IF_RANGE)
        .and_then(|value| value.to_str().ok())
    {
        let still = match httpdate::parse_http_date(condition) {
            Ok(date) => date >= last_modified,
            Err(_) => condition == etag,
        };
        if !still {
            return Wanted::Whole;
        }
    }
    let Some(spec) = range.trim().strip_prefix("bytes=") else {
        return Wanted::Whole;
    };
    if spec.contains(',') {
        return Wanted::Whole;
    }
    let Some((start, end)) = spec.trim().split_once('-') else {
        return Wanted::Whole;
    };
    let (start, end) = (start.trim(), end.trim());
    let parsed = match (start.is_empty(), end.is_empty()) {
        (true, true) => return Wanted::Whole,
        (true, false) => match end.parse::<u64>() {
            Ok(0) => return Wanted::Unsatisfiable,
            Ok(suffix) => (size.saturating_sub(suffix), size.saturating_sub(1)),
            Err(_) => return Wanted::Whole,
        },
        (false, _) => {
            let Ok(first) = start.parse::<u64>() else {
                return Wanted::Whole;
            };
            let last = match end.is_empty() {
                true => size.saturating_sub(1),
                false => match end.parse::<u64>() {
                    Ok(last) if last >= first => last.min(size.saturating_sub(1)),
                    _ => return Wanted::Whole,
                },
            };
            (first, last)
        }
    };
    if size == 0 || parsed.0 >= size {
        return Wanted::Unsatisfiable;
    }
    Wanted::Part(parsed.0, parsed.1)
}

fn precondition_fails(headers: &HeaderMap, etag: &str, last_modified: SystemTime) -> bool {
    if let Some(tags) = headers
        .get(header::IF_MATCH)
        .and_then(|value| value.to_str().ok())
    {
        return !tags
            .split(',')
            .map(str::trim)
            .any(|tag| tag == "*" || tag == etag);
    }
    headers
        .get(header::IF_UNMODIFIED_SINCE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| httpdate::parse_http_date(value).ok())
        .is_some_and(|since| last_modified > since)
}

fn not_modified(headers: &HeaderMap, etag: &str, last_modified: SystemTime) -> bool {
    if let Some(tags) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
    {
        return tags
            .split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag);
    }
    headers
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| httpdate::parse_http_date(value).ok())
        .is_some_and(|since| last_modified <= since)
}

fn truncate_to_second(time: SystemTime) -> SystemTime {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(seconds)
}

pub async fn serve(
    ctx: &Ctx,
    asset: &Asset,
    method: &Method,
    headers: &HeaderMap,
    options: Serve,
) -> Result<Response, ContentError> {
    let size = asset.size.max(0) as u64;
    let etag = format!(
        "\"{}\"",
        options.etag.as_deref().unwrap_or(asset.sha1.as_str())
    );
    let last_modified =
        truncate_to_second(asset.blob_updated_at.with_timezone(&chrono::Utc).into());
    let mut builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(
            header::LAST_MODIFIED,
            httpdate::fmt_http_date(last_modified),
        )
        .header(header::ACCEPT_RANGES, "bytes");
    if let Some(disposition) = options.disposition {
        builder = builder.header(header::CONTENT_DISPOSITION, disposition);
    }
    if precondition_fails(headers, &etag, last_modified) {
        return Err(ContentError::PreconditionFailed);
    }
    if not_modified(headers, &etag, last_modified) {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    let content_type = options
        .content_type
        .clone()
        .unwrap_or_else(|| asset.content_type.clone());
    builder = builder.header(header::CONTENT_TYPE, content_type);
    let wanted = wanted(headers, size, &etag, last_modified);
    let (status, offset, length) = match wanted {
        Wanted::Unsatisfiable => return Err(ContentError::RangeNotSatisfiable(size)),
        Wanted::Whole => (StatusCode::OK, 0, size),
        Wanted::Part(first, last) => {
            builder = builder.header(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&format!("bytes {first}-{last}/{size}"))
                    .map_err(ContentError::internal)?,
            );
            (StatusCode::PARTIAL_CONTENT, first, last - first + 1)
        }
    };
    builder = builder
        .status(status)
        .header(header::CONTENT_LENGTH, length);
    if *method == Method::HEAD {
        return builder.body(Body::empty()).map_err(ContentError::internal);
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let content = match status {
        StatusCode::PARTIAL_CONTENT => {
            store
                .read_range(asset.blob_id.into(), offset, length)
                .await?
        }
        _ => store.read(asset.blob_id.into()).await?,
    };
    let Some(content) = content else {
        tracing::error!(repository = %asset.repository, path = %asset.path, blob = %asset.blob_id, "an asset's blob is missing from its store");
        return Err(ContentError::not_found(asset.path.clone()));
    };
    if options.count_download && offset == 0 {
        ctx.content.note_download(asset.id);
        ctx.content.audit_download(asset, &ctx.uploader()).await;
    }
    builder
        .body(Body::from_stream(content.stream))
        .map_err(ContentError::internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(value: &str, size: u64) -> Wanted {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, value.parse().unwrap());
        wanted(&headers, size, "\"x\"", SystemTime::UNIX_EPOCH)
    }

    #[test]
    fn single_ranges_are_cut() {
        assert_eq!(range("bytes=0-4", 12), Wanted::Part(0, 4));
        assert_eq!(range("bytes=6-", 12), Wanted::Part(6, 11));
        assert_eq!(range("bytes=-3", 12), Wanted::Part(9, 11));
        assert_eq!(range("bytes=5-100", 12), Wanted::Part(5, 11));
        assert_eq!(range("bytes=-100", 12), Wanted::Part(0, 11));
    }

    #[test]
    fn odd_ranges_fall_back_or_fail() {
        assert_eq!(range("bytes=100-", 12), Wanted::Unsatisfiable);
        assert_eq!(range("bytes=-0", 12), Wanted::Unsatisfiable);
        assert_eq!(range("bytes=0-1,3-4", 12), Wanted::Whole);
        assert_eq!(range("bytes=4-2", 12), Wanted::Whole);
        assert_eq!(range("items=0-1", 12), Wanted::Whole);
        assert_eq!(range("bytes=0-", 0), Wanted::Unsatisfiable);
    }

    #[test]
    fn conditions_match_tags_and_dates() {
        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, "\"a\", W/\"b\"".parse().unwrap());
        assert!(not_modified(&headers, "\"b\"", SystemTime::UNIX_EPOCH));
        assert!(!not_modified(&headers, "\"c\"", SystemTime::UNIX_EPOCH));
        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_MODIFIED_SINCE,
            "Thu, 01 Jan 1970 00:00:10 GMT".parse().unwrap(),
        );
        assert!(not_modified(&headers, "\"x\"", SystemTime::UNIX_EPOCH));
        assert!(!not_modified(
            &headers,
            "\"x\"",
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(11)
        ));
    }

    #[test]
    fn preconditions_need_a_matching_tag_or_an_unchanged_date() {
        let at = |seconds| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(seconds);
        let mut headers = HeaderMap::new();
        assert!(!precondition_fails(&headers, "\"a\"", at(0)));
        headers.insert(header::IF_MATCH, "\"b\", \"a\"".parse().unwrap());
        assert!(!precondition_fails(&headers, "\"a\"", at(0)));
        assert!(precondition_fails(&headers, "\"c\"", at(0)));
        headers.insert(header::IF_MATCH, "W/\"a\"".parse().unwrap());
        assert!(precondition_fails(&headers, "\"a\"", at(0)));
        headers.insert(header::IF_MATCH, "*".parse().unwrap());
        assert!(!precondition_fails(&headers, "\"c\"", at(0)));
        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_UNMODIFIED_SINCE,
            "Thu, 01 Jan 1970 00:00:10 GMT".parse().unwrap(),
        );
        assert!(!precondition_fails(&headers, "\"x\"", at(10)));
        assert!(precondition_fails(&headers, "\"x\"", at(11)));
        headers.insert(header::IF_MATCH, "\"x\"".parse().unwrap());
        assert!(!precondition_fails(&headers, "\"x\"", at(11)));
    }
}
