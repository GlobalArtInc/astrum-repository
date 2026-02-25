use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use percent_encoding::percent_decode_str;

pub const DEFAULT_DIR: &str = "/usr/share/astrum/web";
pub const RESERVED: [&str; 5] = ["/repository", "/service", "/v2", "/content", "/.well-known"];
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; \
object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'";
pub const NOT_INSTALLED: &str = "The user interface of Astrum Repository is not installed on this server. \
The REST API and the repositories under /repository/ work without it.\n";

const INDEX: &str = "index.html";
const ASSETS: &str = "/assets/";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const REVALIDATE: &str = "no-cache";

pub fn router(dir: PathBuf) -> Router {
    Router::new().fallback(serve).with_state(Arc::new(dir))
}

pub fn installed(dir: &Path) -> bool {
    dir.join(INDEX).is_file()
}

pub fn is_reserved(path: &str) -> bool {
    RESERVED.iter().any(|prefix| {
        path.strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

async fn serve(
    State(dir): State<Arc<PathBuf>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let path = uri.path();
    if is_reserved(path) || !matches!(method, Method::GET | Method::HEAD) {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Some(file) = resolve(&dir, path)
        && let Some(response) = file_response(&file, &headers, cache_control(path)).await
    {
        return response;
    }
    if path.starts_with(ASSETS) {
        return secured(StatusCode::NOT_FOUND.into_response());
    }
    match file_response(&dir.join(INDEX), &headers, REVALIDATE).await {
        Some(response) => response,
        None => not_installed(path),
    }
}

fn cache_control(path: &str) -> &'static str {
    match path.starts_with(ASSETS) {
        true => IMMUTABLE,
        false => REVALIDATE,
    }
}

fn resolve(dir: &Path, path: &str) -> Option<PathBuf> {
    let decoded = percent_decode_str(path).decode_utf8().ok()?;
    let mut resolved = dir.to_owned();
    let mut segments = 0;
    for segment in decoded.split('/').filter(|segment| !segment.is_empty()) {
        if segment.starts_with('.') || segment.contains(['\\', '\0']) {
            return None;
        }
        resolved.push(segment);
        segments += 1;
    }
    (segments > 0).then_some(resolved)
}

async fn file_response(
    file: &Path,
    headers: &HeaderMap,
    cache_control: &'static str,
) -> Option<Response> {
    let metadata = tokio::fs::metadata(file).await.ok()?;
    if !metadata.is_file() {
        return None;
    }
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let etag = format!("\"{:x}-{:x}\"", metadata.len(), modified);
    let matches = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .any(|tag| tag.trim().trim_start_matches("W/") == etag)
        });
    let mut response = match matches {
        true => StatusCode::NOT_MODIFIED.into_response(),
        false => {
            let bytes = tokio::fs::read(file).await.ok()?;
            let mut response = Response::new(Body::from(bytes));
            let content_type = content_type(file);
            if let Ok(value) = HeaderValue::from_str(&content_type) {
                response.headers_mut().insert(header::CONTENT_TYPE, value);
            }
            response
        }
    };
    let response_headers = response.headers_mut();
    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    if let Ok(value) = HeaderValue::from_str(&etag) {
        response_headers.insert(header::ETAG, value);
    }
    Some(secured(response))
}

fn content_type(file: &Path) -> String {
    let guessed = mime_guess::from_path(file).first_or_octet_stream();
    match guessed.type_() {
        mime_guess::mime::TEXT => format!("{guessed}; charset=utf-8"),
        _ if guessed.subtype() == mime_guess::mime::JAVASCRIPT
            || guessed.subtype() == mime_guess::mime::JSON =>
        {
            format!("{guessed}; charset=utf-8")
        }
        _ => guessed.to_string(),
    }
}

fn not_installed(path: &str) -> Response {
    let status = match path {
        "/" => StatusCode::OK,
        _ => StatusCode::NOT_FOUND,
    };
    secured(
        (
            status,
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (header::CACHE_CONTROL, REVALIDATE),
            ],
            NOT_INSTALLED,
        )
            .into_response(),
    )
}

fn secured(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    response
}

#[cfg(test)]
mod tests {
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    struct Reply {
        status: StatusCode,
        headers: HeaderMap,
        body: String,
    }

    fn site() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join(INDEX), "<!doctype html><title>ui</title>").unwrap();
        std::fs::write(dir.path().join("assets/app-1234.js"), "console.log(1)").unwrap();
        std::fs::write(dir.path().join("favicon.svg"), "<svg/>").unwrap();
        std::fs::write(dir.path().join(".secret"), "no").unwrap();
        dir
    }

    async fn get(dir: &Path, method: Method, path: &str, headers: &[(&str, &str)]) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = router(dir.to_owned())
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        Reply {
            status,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }

    fn header(reply: &Reply, name: header::HeaderName) -> &str {
        reply
            .headers
            .get(name)
            .map(|value| value.to_str().unwrap())
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn pages_of_the_ui_fall_back_to_the_index() {
        let dir = site();
        for path in [
            "/",
            "/browse/maven-releases/org/x",
            "/settings",
            "/admin/security/users",
        ] {
            let reply = get(dir.path(), Method::GET, path, &[]).await;
            assert_eq!(reply.status, StatusCode::OK, "{path}");
            assert!(reply.body.contains("<title>ui</title>"), "{path}");
            assert_eq!(
                header(&reply, header::CONTENT_TYPE),
                "text/html; charset=utf-8"
            );
            assert_eq!(header(&reply, header::CACHE_CONTROL), "no-cache");
            assert_eq!(
                header(&reply, header::CONTENT_SECURITY_POLICY),
                CONTENT_SECURITY_POLICY
            );
            assert_eq!(header(&reply, header::X_CONTENT_TYPE_OPTIONS), "nosniff");
            assert_eq!(header(&reply, header::REFERRER_POLICY), "same-origin");
            assert_eq!(header(&reply, header::X_FRAME_OPTIONS), "DENY");
        }
    }

    #[tokio::test]
    async fn hashed_assets_are_cached_for_long_and_other_files_revalidate() {
        let dir = site();
        let asset = get(dir.path(), Method::GET, "/assets/app-1234.js", &[]).await;
        assert_eq!(asset.status, StatusCode::OK);
        assert_eq!(asset.body, "console.log(1)");
        assert_eq!(header(&asset, header::CACHE_CONTROL), IMMUTABLE);
        let content_type = header(&asset, header::CONTENT_TYPE);
        assert!(content_type.contains("javascript"), "{content_type}");
        assert!(content_type.ends_with("; charset=utf-8"), "{content_type}");
        let icon = get(dir.path(), Method::GET, "/favicon.svg", &[]).await;
        assert_eq!(icon.status, StatusCode::OK);
        assert_eq!(header(&icon, header::CONTENT_TYPE), "image/svg+xml");
        assert_eq!(header(&icon, header::CACHE_CONTROL), "no-cache");
        let missing = get(dir.path(), Method::GET, "/assets/old-9999.js", &[]).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND);
        assert!(missing.body.is_empty());
    }

    #[tokio::test]
    async fn an_unchanged_file_answers_not_modified() {
        let dir = site();
        let first = get(dir.path(), Method::GET, "/", &[]).await;
        let etag = header(&first, header::ETAG).to_owned();
        assert!(etag.starts_with('"'), "{etag}");
        let again = get(dir.path(), Method::GET, "/", &[("if-none-match", &etag)]).await;
        assert_eq!(again.status, StatusCode::NOT_MODIFIED);
        assert!(again.body.is_empty());
        assert_eq!(header(&again, header::CACHE_CONTROL), "no-cache");
        let weak = format!("W/{etag}");
        let weakly = get(
            dir.path(),
            Method::GET,
            "/browse",
            &[("if-none-match", &weak)],
        )
        .await;
        assert_eq!(weakly.status, StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn server_paths_and_other_methods_are_not_the_ui() {
        let dir = site();
        for path in [
            "/repository/x/y",
            "/repository",
            "/service/rest/v1/nothing",
            "/v2/",
            "/v2",
            "/content/repositories/x",
            "/.well-known/terraform.json",
        ] {
            let reply = get(dir.path(), Method::GET, path, &[]).await;
            assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
            assert!(reply.body.is_empty(), "{path}");
        }
        assert!(!is_reserved("/repositoryx"));
        assert!(!is_reserved("/v2x/y"));
        let posted = get(dir.path(), Method::POST, "/browse", &[]).await;
        assert_eq!(posted.status, StatusCode::NOT_FOUND);
        let head = get(dir.path(), Method::HEAD, "/browse", &[]).await;
        assert_eq!(head.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn files_outside_the_directory_and_hidden_files_are_never_served() {
        let dir = site();
        for path in [
            "/../etc/passwd",
            "/%2e%2e/%2e%2e/etc/passwd",
            "/.secret",
            "/assets/..%2F.secret",
        ] {
            let reply = get(dir.path(), Method::GET, path, &[]).await;
            assert_ne!(reply.body, "no", "{path}");
            assert!(!reply.body.contains("root:"), "{path}");
        }
        assert!(resolve(dir.path(), "/a/../b").is_none());
        assert!(resolve(dir.path(), "/").is_none());
    }

    #[tokio::test]
    async fn without_the_ui_the_server_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");
        assert!(!installed(&missing));
        let root = get(&missing, Method::GET, "/", &[]).await;
        assert_eq!(root.status, StatusCode::OK);
        assert_eq!(root.body, NOT_INSTALLED);
        assert_eq!(
            header(&root, header::CONTENT_TYPE),
            "text/plain; charset=utf-8"
        );
        let page = get(&missing, Method::GET, "/browse", &[]).await;
        assert_eq!(page.status, StatusCode::NOT_FOUND);
        assert_eq!(page.body, NOT_INSTALLED);
        assert!(installed(site().path()));
    }
}
