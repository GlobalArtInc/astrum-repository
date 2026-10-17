use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use astrum_security::api_keys;
use astrum_security::format_tokens::looks_like_format_token;
use astrum_security::tokens::looks_like_token;
use astrum_security::{AuthError, Subject};
use axum::http::Method;
use subtle::ConstantTimeEq;

use crate::AppState;
use crate::error::ApiError;
use crate::routes::internal::session;

pub const UI_HEADER: &str = "x-astrum-ui";
pub const SESSION_COOKIE: &str = "astrum_session";
pub const CSRF_COOKIE: &str = "astrum_csrf";
pub const CSRF_HEADER: &str = "x-astrum-csrf";

#[derive(Clone, Debug, Default)]
pub struct Caller(pub Option<Subject>);

impl Caller {
    pub fn subject(&self) -> Option<&Subject> {
        self.0.as_ref()
    }

    pub fn require(&self, permission: &str) -> Result<&Subject, ApiError> {
        match &self.0 {
            Some(subject) if subject.is_permitted(permission) => Ok(subject),
            Some(subject) if !subject.anonymous => Err(ApiError::forbidden()),
            _ => Err(ApiError::Unauthorized),
        }
    }

    pub fn signed_in(&self) -> Result<&Subject, ApiError> {
        match &self.0 {
            Some(subject) if !subject.anonymous => Ok(subject),
            _ => Err(ApiError::Unauthorized),
        }
    }

    pub fn initiator(&self) -> String {
        match &self.0 {
            Some(subject) => subject.user_id.clone(),
            None => "*UNKNOWN".to_owned(),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Caller {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<Caller>()
            .cloned()
            .unwrap_or_default())
    }
}

tokio::task_local! {
    static CLIENT: String;
}

pub fn client_address() -> Option<String> {
    CLIENT
        .try_with(Clone::clone)
        .ok()
        .filter(|address| !address.is_empty())
}

fn peer(request: &Request) -> String {
    let forwarded = crate::routes::internal::client_ip(request.headers());
    if !forwarded.is_empty() {
        return forwarded;
    }
    request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_default()
}

pub async fn authenticate(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let address = peer(&request);
    CLIENT.scope(address, admit(state, request, next)).await
}

async fn admit(state: AppState, mut request: Request, next: Next) -> Response {
    let from_ui = is_ui(request.headers());
    let caller = match resolve(
        &state,
        request.method(),
        request.uri().path(),
        request.headers(),
    )
    .await
    {
        Ok(caller) => caller,
        Err(ApiError::Unauthorized) if is_sign_in(request.uri().path()) => {
            Caller(state.security().anonymous())
        }
        Err(error) => return without_challenge(error.into_response(), from_ui),
    };
    request.extensions_mut().insert(caller);
    without_challenge(next.run(request).await, from_ui)
}

enum Credentials {
    Basic(String, String),
    Token(String),
    FormatToken(String),
}

async fn resolve(
    state: &AppState,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
) -> Result<Caller, ApiError> {
    if let Some(credentials) = credentials(headers)? {
        let security = state.security();
        let password_given =
            matches!(&credentials, Credentials::Basic(_, password) if !looks_like_token(password));
        let (user, result) = match credentials {
            Credentials::Basic(user, password) if looks_like_token(&password) => {
                let result = security.authenticate_token(Some(&user), &password).await;
                (user, result)
            }
            Credentials::Basic(user, password) => {
                let mut result = security.authenticate(&user, &password).await;
                if matches!(result, Err(AuthError::BadCredentials))
                    && api_keys::looks_like_api_key(&password)
                {
                    result = security
                        .authenticate_api_key_as(api_keys::NUGET, &user, &password)
                        .await;
                }
                (user, result)
            }
            Credentials::Token(token) => (
                String::new(),
                security.authenticate_token(None, &token).await,
            ),
            Credentials::FormatToken(token)
                if looks_like_format_token(&token) == Some(api_keys::RUBYGEMS) =>
            {
                (
                    String::new(),
                    security
                        .authenticate_api_key(api_keys::RUBYGEMS, &token)
                        .await,
                )
            }
            Credentials::FormatToken(token) => (
                String::new(),
                security.authenticate_format_token(&token).await,
            ),
        };
        return match result {
            Ok(subject) => Ok(Caller(Some(subject))),
            Err(AuthError::BadCredentials) => {
                tracing::info!(user, "a sign-in failed");
                if password_given {
                    session::audit_failed_sign_in(state, &user, &audit_path(path), headers).await;
                }
                Err(ApiError::Unauthorized)
            }
            Err(AuthError::Internal(error)) => Err(error.into()),
        };
    }
    if let Some(key) = headers
        .get(api_keys::NUGET_HEADER)
        .and_then(|value| value.to_str().ok())
    {
        match state
            .security()
            .authenticate_api_key(api_keys::NUGET, key)
            .await
        {
            Ok(subject) => return Ok(Caller(Some(subject))),
            Err(AuthError::BadCredentials) => {
                tracing::debug!("a NuGet API key was not accepted");
            }
            Err(AuthError::Internal(error)) => return Err(error.into()),
        }
    }
    if let Some(token) = cookie(headers, SESSION_COOKIE)
        && let Some(session) = state.security().session(token).await?
    {
        if !is_safe(method) {
            check_csrf(headers, &session.csrf)?;
        }
        return Ok(Caller(Some(session.subject)));
    }
    Ok(Caller(state.security().anonymous()))
}

fn audit_path(path: &str) -> String {
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    match (parts.next(), parts.next()) {
        (Some("service"), Some(second)) => format!("/service/{second}"),
        (Some(first), _) => format!("/{first}"),
        (None, _) => "/".to_owned(),
    }
}

fn is_sign_in(path: &str) -> bool {
    path.contains("/-/user/org.couchdb.user:")
}

fn is_safe(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn check_csrf(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let site = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok());
    if site.is_some_and(|site| site != "same-origin" && site != "none") {
        return Err(ApiError::Forbidden(
            "requests from other sites are refused".to_owned(),
        ));
    }
    let given = headers
        .get(CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    match bool::from(given.as_bytes().ct_eq(expected.as_bytes())) && !given.is_empty() {
        true => Ok(()),
        false => Err(ApiError::Forbidden(
            "the request lacks the anti-forgery token of its session".to_owned(),
        )),
    }
}

pub fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.trim())
}

fn credentials(headers: &HeaderMap) -> Result<Option<Credentials>, ApiError> {
    let Some(value) = headers.get(header::AUTHORIZATION) else {
        return Ok(None);
    };
    let Ok(value) = value.to_str() else {
        return Err(ApiError::Unauthorized);
    };
    let Some((scheme, encoded)) = value.trim().split_once(' ') else {
        if looks_like_format_token(value.trim()).is_some() {
            return Ok(Some(Credentials::FormatToken(value.trim().to_owned())));
        }
        return Err(ApiError::Unauthorized);
    };
    if scheme.eq_ignore_ascii_case("bearer") {
        let token = encoded.trim();
        if looks_like_token(token) {
            return Ok(Some(Credentials::Token(token.to_owned())));
        }
        if looks_like_format_token(token).is_some() {
            return Ok(Some(Credentials::FormatToken(token.to_owned())));
        }
        return Ok(None);
    }
    if !scheme.eq_ignore_ascii_case("basic") {
        return Ok(None);
    }
    let decoded = STANDARD
        .decode(encoded.trim())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(ApiError::Unauthorized)?;
    let (user, password) = decoded.split_once(':').ok_or(ApiError::Unauthorized)?;
    Ok(Some(Credentials::Basic(
        user.to_owned(),
        password.to_owned(),
    )))
}

fn is_ui(headers: &HeaderMap) -> bool {
    headers
        .get(UI_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn without_challenge(mut response: Response, from_ui: bool) -> Response {
    if from_ui {
        response.headers_mut().remove(header::WWW_AUTHENTICATE);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::audit_path;

    #[test]
    fn failed_sign_ins_name_the_part_of_the_server() {
        assert_eq!(audit_path("/repository/raw/docs/a.txt"), "/repository");
        assert_eq!(audit_path("/service/rest/v1/repositories"), "/service/rest");
        assert_eq!(audit_path("/v2/team/app/manifests/1"), "/v2");
        assert_eq!(audit_path("/"), "/");
    }
}
