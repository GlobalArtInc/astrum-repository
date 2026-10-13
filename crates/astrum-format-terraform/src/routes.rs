use astrum_api::routes::content::context;
use astrum_api::{AppState, Caller};
use astrum_content::{ContentError, ContentRequest};
use astrum_db::audit::Event;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use serde_json::{Value, json};

use crate::{ID, REALM};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(crate::paths::DISCOVERY, any(discovery))
        .route("/repository/{name}/token", any(token))
}

fn json_response(method: &Method, status: StatusCode, value: &Value) -> Response {
    let body = value.to_string();
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(match *method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .unwrap_or_else(|_| status.into_response())
}

fn host_of(headers: &HeaderMap) -> Option<(String, Option<u16>)> {
    let raw = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|value| value.to_str().ok())?
        .split(',')
        .next()?
        .trim()
        .to_ascii_lowercase();
    match raw.rsplit_once(':') {
        Some((host, port)) if !host.ends_with(']') || raw.starts_with('[') => {
            match port.parse::<u16>() {
                Ok(port) => Some((host.to_owned(), Some(port))),
                Err(_) => Some((raw.clone(), None)),
            }
        }
        _ => Some((raw, None)),
    }
}

fn default_port(headers: &HeaderMap) -> u16 {
    match headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
    {
        Some(proto) if proto.eq_ignore_ascii_case("http") => 80,
        _ => 443,
    }
}

pub fn matches_host(registry_host: &str, host: &str, port: u16) -> bool {
    let registry_host = registry_host.trim().to_ascii_lowercase();
    match registry_host.rsplit_once(':') {
        Some((name, wanted)) if wanted.bytes().all(|c| c.is_ascii_digit()) => {
            name == host && wanted.parse::<u16>().ok() == Some(port)
        }
        _ => registry_host == host,
    }
}

async fn discovery(State(state): State<AppState>, method: Method, headers: HeaderMap) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return ContentError::MethodNotAllowed {
            method: method.to_string(),
            allow: "GET,HEAD",
        }
        .into_response();
    }
    let Some((host, port)) = host_of(&headers) else {
        return json_response(
            &method,
            StatusCode::NOT_FOUND,
            &json!({"errors": ["The request names no host"]}),
        );
    };
    let port = port.unwrap_or_else(|| default_port(&headers));
    let repositories = match astrum_repo::store::list(state.content().db()).await {
        Ok(repositories) => repositories,
        Err(error) => return ContentError::internal(error).into_response(),
    };
    let found = repositories
        .into_iter()
        .filter(|repository| repository.format.id == ID && repository.online)
        .filter(|repository| {
            repository
                .attributes
                .terraform
                .as_ref()
                .and_then(|terraform| terraform.registry_host.as_deref())
                .is_some_and(|registry_host| matches_host(registry_host, &host, port))
        })
        .min_by(|a, b| a.name.cmp(&b.name));
    match found {
        Some(repository) => {
            let base = format!("/repository/{}", repository.name);
            json_response(
                &method,
                StatusCode::OK,
                &json!({
                    "modules.v1": format!("{base}/v1/modules/"),
                    "providers.v1": format!("{base}/v1/providers/"),
                }),
            )
        }
        None => json_response(
            &method,
            StatusCode::NOT_FOUND,
            &json!({"errors": [format!("No Terraform repository answers for host {host}; set the registry host of a Terraform repository to it")]}),
        ),
    }
}

async fn token(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let (ctx, format) =
        match context(&state, caller.clone(), &name, &parts.headers, &parts.uri).await {
            Ok(found) => found,
            Err(error) => return error.into_response(),
        };
    if format.id() != ID {
        let request = match ContentRequest::new(
            parts.method,
            "token",
            parts.uri.query().map(str::to_owned),
            parts.headers,
            body,
        ) {
            Ok(request) => request,
            Err(error) => return error.into_response(),
        };
        return format
            .handle(ctx, request)
            .await
            .unwrap_or_else(IntoResponse::into_response);
    }
    let method = parts.method;
    let Some(subject) = caller
        .subject()
        .filter(|subject| !subject.anonymous)
        .cloned()
    else {
        return ContentError::Unauthorized.into_response();
    };
    let security = state.security();
    match method {
        Method::DELETE => match security
            .revoke_format_tokens_of_realm(&subject.user_id, REALM)
            .await
        {
            Ok(0) => json_response(
                &method,
                StatusCode::NOT_FOUND,
                &json!({"errors": ["There is no Terraform token to revoke"]}),
            ),
            Ok(_) => {
                state
                    .audit(
                        &caller,
                        Event::new("security.terraformtoken", "deleted", &subject.user_id),
                    )
                    .await;
                StatusCode::NO_CONTENT.into_response()
            }
            Err(error) => ContentError::internal(error).into_response(),
        },
        Method::GET | Method::HEAD => {
            if !security.model().realm_active(REALM) {
                return json_response(
                    &method,
                    StatusCode::CONFLICT,
                    &json!({"errors": ["The Terraform Bearer Token Realm is not active; ask an administrator to activate it in Security > Realms"]}),
                );
            }
            match security
                .issue_format_token(REALM, &subject.user_id, None)
                .await
            {
                Ok(token) => {
                    state
                        .audit(
                            &caller,
                            Event::new("security.terraformtoken", "created", &subject.user_id),
                        )
                        .await;
                    json_response(&method, StatusCode::OK, &json!({ "token": token }))
                }
                Err(error) => ContentError::internal(error).into_response(),
            }
        }
        other => ContentError::MethodNotAllowed {
            method: other.to_string(),
            allow: "GET,HEAD,DELETE",
        }
        .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_hosts_match_with_or_without_a_port() {
        assert!(matches_host(
            "Astrum.Example.com",
            "astrum.example.com",
            443
        ));
        assert!(matches_host(
            "astrum.example.com",
            "astrum.example.com",
            8443
        ));
        assert!(matches_host(
            "astrum.example.com:8443",
            "astrum.example.com",
            8443
        ));
        assert!(!matches_host(
            "astrum.example.com:8443",
            "astrum.example.com",
            443
        ));
        assert!(!matches_host(
            "other.example.com",
            "astrum.example.com",
            443
        ));
    }
}
