mod loopback;
mod tools;

use astrum_mcp::{Icon, Reply, ServerInfo, VERSION_HEADER};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Router};
use serde_json::json;

use crate::auth::AgentClient;
use crate::limits;
use crate::{AppState, Caller};

pub const PATH: &str = "/mcp";
pub const RESOURCE_METADATA: &str = "/.well-known/oauth-protected-resource/mcp";

tokio::task_local! {
    static CLIENT: String;
}

pub fn client() -> Option<String> {
    CLIENT.try_with(Clone::clone).ok()
}
const INSTRUCTIONS: &str = "Astrum Repository keeps build artifacts: Maven, npm, Docker, Helm, PyPI, NuGet and other formats. \
Find artifacts with search_components and list_versions, Docker tags with list_docker_tags, and read small text files such as \
pom.xml or Chart.yaml with read_text_asset. list_blob_stores shows where blobs are kept and how much room is left; tasks run \
cleanups and compactions. Writing tools change the server for everyone: say what you are about to change before you call them.";

pub fn routes() -> Router<AppState> {
    Router::new().route(PATH, post(serve).get(not_allowed).delete(not_allowed))
}

fn server_info(base: &str) -> ServerInfo {
    let icon = |file: &str, mime_type: &str, size: &str| Icon {
        src: format!("{base}/{file}"),
        mime_type: mime_type.to_owned(),
        sizes: vec![size.to_owned()],
    };
    ServerInfo {
        name: "astrum-repository".to_owned(),
        title: "Astrum Repository".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        instructions: INSTRUCTIONS.to_owned(),
        website_url: Some(base.to_owned()),
        icons: vec![
            icon("favicon.svg", "image/svg+xml", "any"),
            icon("icon-512.png", "image/png", "512x512"),
            icon("icon-192.png", "image/png", "192x192"),
        ],
    }
}

async fn serve(
    State(state): State<AppState>,
    caller: Caller,
    agent: Option<Extension<AgentClient>>,
    headers: HeaderMap,
    uri: Uri,
    body: Bytes,
) -> Response {
    if !origin_allowed(&headers) {
        return refuse(
            StatusCode::FORBIDDEN,
            "requests from another origin are refused",
        );
    }
    let Ok(subject) = caller.signed_in() else {
        return challenge(&headers, &uri, false);
    };
    if let Some(version) = headers
        .get(VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        && !astrum_mcp::supported(version)
        && !astrum_mcp::opens_session(&body)
    {
        return refuse(
            StatusCode::BAD_REQUEST,
            &format!("the protocol version {version} is not supported"),
        );
    }
    if let Err(wait) = state
        .limits()
        .take(&format!("mcp:{}", subject.user_id), limits::AGENT_CALLS)
    {
        let mut response = refuse(
            StatusCode::TOO_MANY_REQUESTS,
            "too many requests; slow down and try again later",
        );
        if let Ok(value) = HeaderValue::from_str(&wait.as_secs().to_string()) {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        return response;
    }
    let client = match agent {
        Some(Extension(AgentClient(name))) => name,
        None => "access token".to_owned(),
    };
    let handler = tools::AstrumTools::new(
        loopback::Loopback::new(&state, caller.clone(), &headers),
        subject.clone(),
    );
    let info = server_info(&crate::routes::repositories::base_url(&headers, &uri));
    let reply = CLIENT
        .scope(client, astrum_mcp::handle_body(&info, &handler, &body))
        .await;
    match reply {
        Reply::Message(message) => axum::Json(message).into_response(),
        Reply::Accepted => StatusCode::ACCEPTED.into_response(),
    }
}

pub fn challenge(headers: &HeaderMap, uri: &Uri, rejected: bool) -> Response {
    let base = crate::routes::repositories::base_url(headers, uri);
    let mut value = format!(
        "Bearer realm=\"Astrum Repository\", resource_metadata=\"{base}{RESOURCE_METADATA}\", scope=\"read write\""
    );
    if rejected {
        value.push_str(", error=\"invalid_token\"");
    }
    let mut response = refuse(
        StatusCode::UNAUTHORIZED,
        "sign in through OAuth or with an access token: Authorization: Bearer astrum_...",
    );
    if let Ok(value) = HeaderValue::from_str(&value) {
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, value);
    }
    response
}

async fn not_allowed() -> Response {
    let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
    response
        .headers_mut()
        .insert(header::ALLOW, HeaderValue::from_static("POST"));
    response
}

fn refuse(status: StatusCode, message: &str) -> Response {
    (
        status,
        axum::Json(json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": { "code": -32000, "message": message },
        })),
    )
        .into_response()
}

fn origin_allowed(headers: &HeaderMap) -> bool {
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return true;
    };
    let Ok(origin) = url::Url::parse(origin) else {
        return false;
    };
    let Some(origin_host) = origin.host_str() else {
        return false;
    };
    let origin_authority = match origin.port() {
        Some(port) => format!("{origin_host}:{port}"),
        None => origin_host.to_owned(),
    };
    [header::HOST.as_str(), "x-forwarded-host"]
        .iter()
        .filter_map(|name| headers.get(*name).and_then(|value| value.to_str().ok()))
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .any(|host| {
            host.eq_ignore_ascii_case(&origin_authority) || host.eq_ignore_ascii_case(origin_host)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, HeaderValue::from_static(value));
        }
        headers
    }

    #[test]
    fn only_the_own_origin_is_let_in() {
        assert!(origin_allowed(&headers(&[("host", "astrum.example")])));
        assert!(origin_allowed(&headers(&[
            ("host", "astrum.example"),
            ("origin", "https://astrum.example")
        ])));
        assert!(origin_allowed(&headers(&[
            ("host", "10.0.0.1:8082"),
            ("x-forwarded-host", "astrum.example"),
            ("origin", "https://astrum.example")
        ])));
        assert!(!origin_allowed(&headers(&[
            ("host", "astrum.example"),
            ("origin", "https://evil.example")
        ])));
        assert!(!origin_allowed(&headers(&[
            ("host", "astrum.example"),
            ("origin", "null")
        ])));
    }
}
