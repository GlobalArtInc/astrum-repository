use std::collections::HashMap;

use astrum_api::{AppState, Caller};
use astrum_security::tokens::looks_like_token;
use astrum_security::{AuthError, SecurityModel, Subject};
use axum::Form;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{SecondsFormat, Utc};
use serde_json::json;

use crate::error::RegistryError;

pub const REALM: &str = "DockerToken";
pub const API_VERSION: &str = "registry/2.0";
pub const ANONYMOUS_TOKEN: &str = "anonymous";
pub const LIFETIME_SECONDS: i64 = 1800;

pub fn challenge(security: &SecurityModel, base_url: &str, scope: Option<&str>) -> String {
    if !security.realm_active(REALM) {
        return astrum_content::CHALLENGE.to_owned();
    }
    let service = service(base_url);
    let mut value = format!("Bearer realm=\"{base_url}/v2/token\",service=\"{service}\"");
    if let Some(scope) = scope {
        value.push_str(&format!(",scope=\"{scope}\""));
    }
    value
}

pub fn service(base_url: &str) -> &str {
    base_url
        .split_once("://")
        .map(|(_, host)| host)
        .unwrap_or(base_url)
}

pub fn with_challenge(
    mut response: Response,
    security: &SecurityModel,
    base_url: &str,
    scope: Option<&str>,
) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        "docker-distribution-api-version",
        HeaderValue::from_static(API_VERSION),
    );
    if response.status() == StatusCode::UNAUTHORIZED
        && let Ok(value) = HeaderValue::from_str(&challenge(security, base_url, scope))
    {
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, value);
    }
    response
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.trim().split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then_some(token.trim())
}

fn basic_password(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, encoded) = value.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    let decoded = STANDARD.decode(encoded.trim()).ok()?;
    let text = String::from_utf8(decoded).ok()?;
    text.split_once(':')
        .map(|(_, password)| password.to_owned())
}

pub async fn ping(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let base = astrum_api::routes::repositories::base_url(&headers, &uri);
    let model = state.security().model();
    let admitted = match caller.subject() {
        Some(subject) if !subject.anonymous => true,
        Some(_) => bearer(&headers).is_some(),
        None => false,
    };
    let response = match admitted {
        true => (
            [(header::CONTENT_TYPE, "application/json")],
            "{}".to_owned(),
        )
            .into_response(),
        false => RegistryError::unauthorized().into_response(),
    };
    with_challenge(response, &model, &base, None)
}

fn token_body(token: &str, lifetime: i64, scope: Option<&str>) -> Response {
    let mut body = json!({
        "token": token,
        "access_token": token,
        "expires_in": lifetime,
        "issued_at": Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    });
    if let Some(scope) = scope {
        body["scope"] = scope.into();
    }
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body.to_string(),
    )
        .into_response()
}

async fn issue(state: &AppState, subject: &Subject) -> Result<String, RegistryError> {
    state
        .security()
        .issue_format_token(
            REALM,
            &subject.user_id,
            Some(chrono::Duration::seconds(LIFETIME_SECONDS)),
        )
        .await
        .map_err(|error| astrum_content::ContentError::internal(error).into())
}

fn inactive() -> Response {
    RegistryError::new(
        StatusCode::NOT_FOUND,
        "UNSUPPORTED",
        "the Docker Bearer Token Realm is not active",
    )
    .into_response()
}

pub async fn token(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let base = astrum_api::routes::repositories::base_url(&headers, &uri);
    let model = state.security().model();
    if !model.realm_active(REALM) {
        return with_challenge(inactive(), &model, &base, None);
    }
    let scope = params.get("scope").map(String::as_str);
    let response = match caller.subject() {
        Some(subject) if !subject.anonymous => {
            let reused = basic_password(&headers)
                .filter(|password| looks_like_token(password))
                .or_else(|| {
                    bearer(&headers)
                        .filter(|token| looks_like_token(token))
                        .map(str::to_owned)
                });
            match reused {
                Some(token) => token_body(&token, LIFETIME_SECONDS, scope),
                None => match issue(&state, subject).await {
                    Ok(token) => token_body(&token, LIFETIME_SECONDS, scope),
                    Err(error) => error.into_response(),
                },
            }
        }
        Some(_) => token_body(ANONYMOUS_TOKEN, LIFETIME_SECONDS, scope),
        None => RegistryError::unauthorized().into_response(),
    };
    with_challenge(response, &model, &base, scope)
}

pub async fn oauth(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let base = astrum_api::routes::repositories::base_url(&headers, &uri);
    let model = state.security().model();
    if !model.realm_active(REALM) {
        return with_challenge(inactive(), &model, &base, None);
    }
    let scope = form.get("scope").map(String::as_str);
    if form.get("grant_type").map(String::as_str) != Some("password") {
        return oauth_error(StatusCode::BAD_REQUEST, "unsupported_grant_type");
    }
    let (Some(username), Some(password)) = (form.get("username"), form.get("password")) else {
        return oauth_error(StatusCode::BAD_REQUEST, "invalid_request");
    };
    let security = state.security();
    if looks_like_token(password) {
        return match security.authenticate_token(Some(username), password).await {
            Ok(_) => token_body(password, LIFETIME_SECONDS, scope),
            Err(_) => oauth_error(StatusCode::UNAUTHORIZED, "invalid_grant"),
        };
    }
    match security.authenticate(username, password).await {
        Ok(subject) => match issue(&state, &subject).await {
            Ok(token) => token_body(&token, LIFETIME_SECONDS, scope),
            Err(error) => error.into_response(),
        },
        Err(AuthError::BadCredentials) => {
            tracing::info!(user = %username, "a Docker token request failed");
            oauth_error(StatusCode::UNAUTHORIZED, "invalid_grant")
        }
        Err(AuthError::Internal(error)) => {
            RegistryError::from(astrum_content::ContentError::internal(error)).into_response()
        }
    }
}

fn oauth_error(status: StatusCode, error: &str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        json!({ "error": error }).to_string(),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_are_hosts() {
        assert_eq!(service("https://astrum.example.com"), "astrum.example.com");
        assert_eq!(service("http://127.0.0.1:18104"), "127.0.0.1:18104");
        assert_eq!(service("plain"), "plain");
    }

    #[test]
    fn credentials_are_read_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "Bearer abc".parse().unwrap());
        assert_eq!(bearer(&headers), Some("abc"));
        assert_eq!(basic_password(&headers), None);
        let encoded = STANDARD.encode("user:pa:ss");
        headers.insert(
            header::AUTHORIZATION,
            format!("Basic {encoded}").parse().unwrap(),
        );
        assert_eq!(basic_password(&headers).as_deref(), Some("pa:ss"));
        assert_eq!(bearer(&headers), None);
    }
}
