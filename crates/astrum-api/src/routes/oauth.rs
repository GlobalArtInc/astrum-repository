use std::collections::HashMap;

use astrum_db::audit::Event;
use astrum_security::oauth::{
    self, NewAuthorization, OAuthError, READ_SCOPE, WRITE_SCOPE, parse_scope, redirect_matches,
    with_query,
};
use astrum_security::{Scope, realm};
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{delete, get, post};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::limits;
use crate::routes::internal::ui::UI;
use crate::routes::mcp;
use crate::routes::repositories::base_url;
use crate::{ApiError, AppState, Caller};

const PROTECTED_RESOURCE: &str = "/.well-known/oauth-protected-resource";
const AUTHORIZATION_SERVER: &str = "/.well-known/oauth-authorization-server";
const REGISTER: &str = "/oauth/register";
const AUTHORIZE: &str = "/oauth/authorize";
const TOKEN: &str = "/oauth/token";
const REVOKE: &str = "/oauth/revoke";
const CONSENT_PAGE: &str = "/user/authorize";
const DOMAIN: &str = "oauth";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(PROTECTED_RESOURCE, get(protected_resource))
        .route(mcp::RESOURCE_METADATA, get(protected_resource))
        .route(AUTHORIZATION_SERVER, get(authorization_server))
        .route(REGISTER, post(register))
        .route(AUTHORIZE, get(authorize))
        .route(TOKEN, post(token))
        .route(REVOKE, post(revoke))
        .route(&format!("{UI}/oauth/requests/{{id}}"), get(pending))
        .route(
            &format!("{UI}/oauth/requests/{{id}}/approve"),
            post(approve),
        )
        .route(&format!("{UI}/oauth/requests/{{id}}/deny"), post(deny))
        .route(&format!("{UI}/oauth/grants"), get(grants))
        .route(&format!("{UI}/oauth/grants/{{id}}"), delete(revoke_grant))
}

pub fn is_endpoint(path: &str) -> bool {
    path.starts_with("/oauth/") || path.starts_with("/.well-known/oauth-")
}

fn client_key(prefix: &str) -> String {
    format!(
        "{prefix}:{}",
        crate::auth::client_address().unwrap_or_default()
    )
}

fn no_store(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response
}

fn error(status: StatusCode, code: &str, description: &str) -> Response {
    no_store(
        (
            status,
            axum::Json(json!({ "error": code, "error_description": description })),
        )
            .into_response(),
    )
}

fn failure(problem: OAuthError) -> Response {
    let status = match &problem {
        OAuthError::InvalidClient => StatusCode::UNAUTHORIZED,
        OAuthError::Internal(error) => {
            tracing::error!(%error, "an OAuth request failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        _ => StatusCode::BAD_REQUEST,
    };
    let description = match &problem {
        OAuthError::Internal(_) => "the server failed".to_owned(),
        other => other.to_string(),
    };
    error(status, problem.code(), &description)
}

fn too_many() -> Response {
    error(
        StatusCode::TOO_MANY_REQUESTS,
        "slow_down",
        "too many requests; try again later",
    )
}

async fn protected_resource(headers: HeaderMap, uri: Uri) -> Response {
    let base = base_url(&headers, &uri);
    axum::Json(json!({
        "resource": format!("{base}{}", mcp::PATH),
        "authorization_servers": [base],
        "scopes_supported": [READ_SCOPE, WRITE_SCOPE],
        "bearer_methods_supported": ["header"],
        "resource_name": "Astrum Repository",
    }))
    .into_response()
}

async fn authorization_server(headers: HeaderMap, uri: Uri) -> Response {
    let base = base_url(&headers, &uri);
    axum::Json(json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}{AUTHORIZE}"),
        "token_endpoint": format!("{base}{TOKEN}"),
        "registration_endpoint": format!("{base}{REGISTER}"),
        "revocation_endpoint": format!("{base}{REVOKE}"),
        "response_types_supported": ["code"],
        "response_modes_supported": ["query"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
        "revocation_endpoint_auth_methods_supported": ["none"],
        "scopes_supported": [READ_SCOPE, WRITE_SCOPE],
    }))
    .into_response()
}

#[derive(Default, Deserialize)]
struct Registration {
    redirect_uris: Option<Vec<String>>,
    client_name: Option<String>,
    grant_types: Option<Vec<String>>,
    response_types: Option<Vec<String>>,
}

async fn register(State(state): State<AppState>, body: Bytes) -> Response {
    if state
        .limits()
        .take(&client_key("register"), limits::CLIENT_REGISTRATIONS)
        .is_err()
    {
        return too_many();
    }
    let registration: Registration = match serde_json::from_slice(&body) {
        Ok(registration) => registration,
        Err(problem) => {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_client_metadata",
                &format!("the body is not client metadata: {problem}"),
            );
        }
    };
    if let Some(types) = &registration.grant_types
        && let Some(other) = types
            .iter()
            .find(|kind| !matches!(kind.as_str(), "authorization_code" | "refresh_token"))
    {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_client_metadata",
            &format!("the grant type {other} is not supported"),
        );
    }
    if let Some(types) = &registration.response_types
        && types.iter().any(|kind| kind != "code")
    {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_client_metadata",
            "only the response type code is supported",
        );
    }
    let uris = registration.redirect_uris.unwrap_or_default();
    let client = match state
        .security()
        .register_oauth_client(registration.client_name.as_deref(), &uris)
        .await
    {
        Ok(client) => client,
        Err(problem) => return failure(problem),
    };
    tracing::info!(client = %client.id, name = %client.name, "an OAuth client registered");
    no_store(
        (
            StatusCode::CREATED,
            axum::Json(json!({
                "client_id": client.id,
                "client_id_issued_at": client.created_at.timestamp(),
                "client_name": client.name,
                "redirect_uris": client.redirect_uris,
                "grant_types": ["authorization_code", "refresh_token"],
                "response_types": ["code"],
                "token_endpoint_auth_method": "none",
            })),
        )
            .into_response(),
    )
}

fn problem_page(base: &str, problem: &str) -> Response {
    Redirect::to(&format!("{base}{CONSENT_PAGE}?problem={problem}")).into_response()
}

fn back_to(redirect_uri: &str, state: Option<&str>, code: &str, description: &str) -> Response {
    let mut pairs = vec![("error", code), ("error_description", description)];
    if let Some(state) = state {
        pairs.push(("state", state));
    }
    Redirect::to(&with_query(redirect_uri, &pairs)).into_response()
}

fn same_resource(base: &str, resource: &str) -> bool {
    let resource = resource.trim_end_matches('/');
    resource == base || resource == format!("{base}{}", mcp::PATH)
}

async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let base = base_url(&headers, &uri);
    if state
        .limits()
        .take(&client_key("oauth"), limits::OAUTH_REQUESTS)
        .is_err()
    {
        return too_many();
    }
    let param = |name: &str| {
        params
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    };
    let Some(client_id) = param("client_id") else {
        return problem_page(&base, "client");
    };
    let client = match state.security().oauth_client(client_id).await {
        Ok(Some(client)) => client,
        Ok(None) => return problem_page(&base, "client"),
        Err(problem) => return failure(OAuthError::Internal(problem)),
    };
    let redirect_uri = match param("redirect_uri") {
        Some(presented) => match client
            .redirect_uris
            .iter()
            .any(|registered| redirect_matches(registered, presented))
        {
            true => presented.to_owned(),
            false => return problem_page(&base, "redirect"),
        },
        None => match client.redirect_uris.as_slice() {
            [only] => only.clone(),
            _ => return problem_page(&base, "redirect"),
        },
    };
    let state_value = param("state");
    if param("response_type") != Some("code") {
        return back_to(
            &redirect_uri,
            state_value,
            "unsupported_response_type",
            "only the response type code is supported",
        );
    }
    let Some(challenge) = param("code_challenge") else {
        return back_to(
            &redirect_uri,
            state_value,
            "invalid_request",
            "a PKCE code_challenge is required",
        );
    };
    if param("code_challenge_method") != Some("S256") {
        return back_to(
            &redirect_uri,
            state_value,
            "invalid_request",
            "code_challenge_method must be S256",
        );
    }
    let scope = match parse_scope(param("scope")) {
        Ok(scope) => scope,
        Err(problem) => {
            return back_to(
                &redirect_uri,
                state_value,
                problem.code(),
                &problem.to_string(),
            );
        }
    };
    let resource = param("resource");
    if let Some(resource) = resource
        && !same_resource(&base, resource)
    {
        return back_to(
            &redirect_uri,
            state_value,
            "invalid_target",
            "the resource is not this server",
        );
    }
    match state
        .security()
        .begin_oauth_authorization(NewAuthorization {
            client_id: &client.id,
            redirect_uri: &redirect_uri,
            code_challenge: challenge,
            scope,
            state: state_value,
            resource,
        })
        .await
    {
        Ok(id) => Redirect::to(&format!("{base}{CONSENT_PAGE}?request={id}")).into_response(),
        Err(problem) => failure(problem),
    }
}

fn form(headers: &HeaderMap, body: &[u8]) -> HashMap<String, String> {
    let json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"));
    if json {
        return serde_json::from_slice::<HashMap<String, Value>>(body)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(key, value)| match value {
                Value::String(text) => Some((key, text)),
                _ => None,
            })
            .collect();
    }
    url::form_urlencoded::parse(body).into_owned().collect()
}

async fn token(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    if state
        .limits()
        .take(&client_key("oauth"), limits::OAUTH_REQUESTS)
        .is_err()
    {
        return too_many();
    }
    let fields = form(&headers, &body);
    let field = |name: &str| {
        fields
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    };
    let Some(client_id) = field("client_id") else {
        return failure(OAuthError::InvalidClient);
    };
    match state.security().oauth_client(client_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return failure(OAuthError::InvalidClient),
        Err(problem) => return failure(OAuthError::Internal(problem)),
    }
    let missing = |name: &str| {
        failure(OAuthError::InvalidRequest(format!(
            "the field {name} is required"
        )))
    };
    let issued = match field("grant_type") {
        Some("authorization_code") => {
            let (Some(code), Some(redirect_uri), Some(verifier)) =
                (field("code"), field("redirect_uri"), field("code_verifier"))
            else {
                return missing("code, redirect_uri or code_verifier");
            };
            state
                .security()
                .exchange_oauth_code(client_id, code, redirect_uri, verifier)
                .await
                .map(|(tokens, _)| tokens)
        }
        Some("refresh_token") => {
            let Some(refresh) = field("refresh_token") else {
                return missing("refresh_token");
            };
            state
                .security()
                .refresh_oauth_tokens(client_id, refresh)
                .await
        }
        Some(other) => {
            return error(
                StatusCode::BAD_REQUEST,
                "unsupported_grant_type",
                &format!("the grant type {other} is not supported"),
            );
        }
        None => return missing("grant_type"),
    };
    match issued {
        Ok(tokens) => no_store(
            axum::Json(json!({
                "access_token": tokens.access_token,
                "token_type": "Bearer",
                "expires_in": tokens.expires_in,
                "refresh_token": tokens.refresh_token,
                "scope": tokens.scope,
            }))
            .into_response(),
        ),
        Err(problem) => failure(problem),
    }
}

async fn revoke(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    if state
        .limits()
        .take(&client_key("oauth"), limits::OAUTH_REQUESTS)
        .is_err()
    {
        return too_many();
    }
    let fields = form(&headers, &body);
    if let Some(token) = fields.get("token")
        && let Err(problem) = state.security().revoke_oauth_token(token).await
    {
        return failure(OAuthError::Internal(problem));
    }
    no_store(StatusCode::OK.into_response())
}

fn person(caller: &Caller) -> Result<&astrum_security::Subject, ApiError> {
    let subject = caller.signed_in()?;
    if subject.realm == realm::ACCESS_TOKEN {
        return Err(ApiError::Forbidden(
            "sign in to the web interface to connect an agent".to_owned(),
        ));
    }
    Ok(subject)
}

fn redirect_host(uri: &str) -> String {
    match url::Url::parse(uri) {
        Ok(url) => match url.host_str() {
            Some(host) => host.to_owned(),
            None => format!("{}:", url.scheme()),
        },
        Err(_) => uri.to_owned(),
    }
}

async fn pending(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let subject = person(&caller)?;
    let Some(request) = state.security().pending_oauth_authorization(&id).await? else {
        return Err(ApiError::not_found(
            "the sign-in request is not known or has expired; start again from the agent",
        ));
    };
    Ok(axum::Json(json!({
        "id": request.id,
        "clientName": request.client.name,
        "clientId": request.client.id,
        "redirectHost": redirect_host(&request.redirect_uri),
        "requestedScope": request.requested_scope,
        "writeRequested": request.requested_scope.iter().any(|scope| scope == WRITE_SCOPE),
        "user": subject.user_id,
        "expiresAt": request.expires_at.to_rfc3339(),
    }))
    .into_response())
}

#[derive(Default, Deserialize)]
struct Approval {
    #[serde(default)]
    write: bool,
}

async fn approve(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    body: Option<axum::Json<Approval>>,
) -> Result<Response, ApiError> {
    let subject = person(&caller)?;
    let write = body.map(|axum::Json(body)| body.write).unwrap_or(false);
    let Some(request) = state.security().pending_oauth_authorization(&id).await? else {
        return Err(ApiError::not_found(
            "the sign-in request is not known or has expired; start again from the agent",
        ));
    };
    let scope = match write {
        true => Scope::Write,
        false => Scope::Read,
    };
    let Some(redirect) = state
        .security()
        .approve_oauth_authorization(&id, subject, scope)
        .await?
    else {
        return Err(ApiError::not_found(
            "the sign-in request is not known or has expired; start again from the agent",
        ));
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "authorized", &request.client.name).with(json!({
                "clientId": request.client.id,
                "scope": oauth::scope_text(scope),
            })),
        )
        .await;
    Ok(axum::Json(json!({ "redirect": redirect })).into_response())
}

async fn deny(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    person(&caller)?;
    match state.security().deny_oauth_authorization(&id).await? {
        Some(redirect) => Ok(axum::Json(json!({ "redirect": redirect })).into_response()),
        None => Err(ApiError::not_found(
            "the sign-in request is not known or has expired",
        )),
    }
}

async fn grants(State(state): State<AppState>, caller: Caller) -> Result<Response, ApiError> {
    let subject = caller.signed_in()?;
    let grants = state.security().oauth_grants(&subject.user_id).await?;
    Ok(axum::Json(
        grants
            .into_iter()
            .map(|grant| {
                json!({
                    "id": grant.id,
                    "clientName": grant.client_name,
                    "scope": match grant.scope {
                        Scope::Read => "READ",
                        Scope::Write => "WRITE",
                    },
                    "createdAt": grant.created_at.to_rfc3339(),
                    "lastUsedAt": grant.last_used_at.map(|time| time.to_rfc3339()),
                    "expiresAt": grant.refresh_expires_at.to_rfc3339(),
                })
            })
            .collect::<Vec<_>>(),
    )
    .into_response())
}

async fn revoke_grant(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let subject = caller.signed_in()?;
    let Ok(id) = id.parse::<Uuid>() else {
        return Err(ApiError::not_found("there is no such agent access"));
    };
    match state
        .security()
        .delete_oauth_grant(id, &subject.user_id)
        .await?
    {
        Some(grant) => {
            state
                .audit(
                    &caller,
                    Event::new(DOMAIN, "revoked", &grant.client_name)
                        .with(json!({ "clientId": grant.client_id })),
                )
                .await;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(ApiError::not_found("there is no such agent access")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_this_server_is_a_resource() {
        let base = "https://astrum.example";
        assert!(same_resource(base, "https://astrum.example/mcp"));
        assert!(same_resource(base, "https://astrum.example/mcp/"));
        assert!(same_resource(base, "https://astrum.example"));
        assert!(!same_resource(base, "https://other.example/mcp"));
        assert!(is_endpoint("/oauth/token"));
        assert!(is_endpoint("/.well-known/oauth-authorization-server"));
        assert!(!is_endpoint("/service/rest/internal/ui/oauth/grants"));
        assert_eq!(redirect_host("https://claude.ai/api/cb"), "claude.ai");
        assert_eq!(redirect_host("vscode://x/cb"), "x");
    }
}
