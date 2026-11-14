use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use astrum_security::api_keys;
use astrum_security::format_tokens::looks_like_format_token;
use astrum_security::oauth::looks_like_access_token;
use astrum_security::sessions::BasicDenied;
use astrum_security::tokens::looks_like_token;
use astrum_security::{AuthError, Gate, Subject};
use axum::http::Method;
use subtle::ConstantTimeEq;

use crate::AppState;
use crate::error::{ApiError, CONCURRENT_MESSAGE};
use crate::routes::internal::session;

pub const UI_HEADER: &str = "x-astrum-ui";
pub const SESSION_COOKIE: &str = "astrum_session";
pub const CSRF_COOKIE: &str = "astrum_csrf";
pub const CSRF_HEADER: &str = "x-astrum-csrf";

#[derive(Clone, Debug, Default)]
pub struct Caller(pub Option<Subject>);

#[derive(Clone, Debug)]
pub struct AgentClient(pub String);

#[derive(Clone, Copy, Debug)]
pub struct RestrictedSession;

const RESTRICTED_SESSION_PATH: &str = "/service/rest/internal/session";
const RESTRICTED_ACCOUNT_PATH: &str = "/service/rest/internal/ui/user";
const RESTRICTED_STATE_PATH: &str = "/service/rest/internal/ui/state";
const RESTRICTED_FACTORS_PATH: &str = "/service/rest/internal/ui/user/two-factor";

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

pub fn client_address() -> Option<String> {
    crate::client::current().map(|address| address.to_string())
}

pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let from_ui = is_ui(request.headers());
    if crate::routes::oauth::is_endpoint(request.uri().path()) {
        request.extensions_mut().insert(Caller(None));
        return next.run(request).await;
    }
    let for_agents = request.uri().path() == crate::routes::mcp::PATH;
    let (caller, client, restricted) = match resolve(
        &state,
        request.method(),
        request.uri().path(),
        request.headers(),
    )
    .await
    {
        Ok(caller) => caller,
        Err(ApiError::Unauthorized) if is_sign_in(request.uri().path()) => {
            (Caller(state.security().anonymous()), None, false)
        }
        Err(ApiError::Unauthorized) if for_agents => {
            return crate::routes::mcp::challenge(request.headers(), request.uri(), true);
        }
        Err(ApiError::TooManyAttempts(seconds)) if is_registry(request.uri().path()) => {
            return registry_throttled(seconds);
        }
        Err(ApiError::TooManyConcurrent) if is_registry(request.uri().path()) => {
            return registry_busy();
        }
        Err(error) => return without_challenge(error.into_response(), from_ui),
    };
    if let Some(client) = client {
        request.extensions_mut().insert(AgentClient(client));
    }
    if restricted {
        request.extensions_mut().insert(RestrictedSession);
    }
    request.extensions_mut().insert(caller);
    without_challenge(next.run(request).await, from_ui)
}

enum Credentials {
    Basic(String, String),
    Token(String),
    FormatToken(String),
    OAuth(String),
}

async fn resolve(
    state: &AppState,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
) -> Result<(Caller, Option<String>, bool), ApiError> {
    if let Some(credentials) = credentials(headers)? {
        if let Credentials::OAuth(token) = &credentials {
            if path != crate::routes::mcp::PATH {
                return Err(ApiError::Unauthorized);
            }
            return match state.security().authenticate_oauth(token).await? {
                Some(found) => Ok((Caller(Some(found.subject)), Some(found.client_name), false)),
                None => Err(ApiError::Unauthorized),
            };
        }
        let security = state.security();
        let password_given =
            matches!(&credentials, Credentials::Basic(_, password) if !looks_like_token(password));
        let mut refused_gate = None;
        let (user, result) = match credentials {
            Credentials::Basic(user, password) if looks_like_token(&password) => {
                let result = security.authenticate_token(Some(&user), &password).await;
                (user, result)
            }
            Credentials::Basic(user, password) => {
                let result = match check_password(state, &user, &password, "Basic", true).await {
                    Ok(subject) => Ok(subject),
                    Err(PasswordRefusal::Wrong(gate)) => {
                        refused_gate = gate;
                        Err(AuthError::BadCredentials)
                    }
                    Err(PasswordRefusal::Throttled(seconds)) => {
                        return Err(ApiError::TooManyAttempts(seconds));
                    }
                    Err(PasswordRefusal::Busy) => return Err(ApiError::TooManyConcurrent),
                    Err(PasswordRefusal::Internal(error)) => Err(AuthError::Internal(error)),
                };
                (user, result)
            }
            Credentials::Token(token) => (
                String::new(),
                security.authenticate_token(None, &token).await,
            ),
            Credentials::FormatToken(token)
                if looks_like_format_token(&token)
                    .is_some_and(|realm| api_keys::KEPT_REALMS.contains(&realm)) =>
            {
                let realm = looks_like_format_token(&token)
                    .unwrap_or(api_keys::RUBYGEMS)
                    .to_owned();
                (
                    String::new(),
                    security.authenticate_api_key(&realm, &token).await,
                )
            }
            Credentials::FormatToken(token) => (
                String::new(),
                security.authenticate_format_token(&token).await,
            ),
            Credentials::OAuth(_) => (String::new(), Err(AuthError::BadCredentials)),
        };
        return match result {
            Ok(subject) => Ok((Caller(Some(subject)), None, false)),
            Err(AuthError::BadCredentials) => {
                tracing::info!(user, "a sign-in failed");
                if password_given {
                    let reason = refused_gate.unwrap_or(Gate::Open).failure_reason();
                    session::audit_failed_sign_in(state, &user, &audit_path(path), headers, reason)
                        .await;
                }
                Err(ApiError::Unauthorized)
            }
            Err(AuthError::SecondFactor(pending)) => {
                tracing::error!(
                    user = pending.user_id,
                    "a credential other than a password raised a second factor demand; refused"
                );
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
            Ok(subject) => return Ok((Caller(Some(subject)), None, false)),
            Err(AuthError::BadCredentials) => {
                tracing::debug!("a NuGet API key was not accepted");
            }
            Err(AuthError::SecondFactor(pending)) => {
                tracing::error!(
                    user = pending.user_id,
                    "a NuGet API key raised a second factor demand; refused"
                );
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
        if session.restricted && !restricted_allows(method, path) {
            return Err(ApiError::EnrollmentRequired);
        }
        return Ok((Caller(Some(session.subject)), None, session.restricted));
    }
    Ok((Caller(state.security().anonymous()), None, false))
}

fn restricted_allows(method: &Method, path: &str) -> bool {
    match path {
        RESTRICTED_SESSION_PATH => matches!(*method, Method::GET | Method::DELETE),
        RESTRICTED_ACCOUNT_PATH | RESTRICTED_STATE_PATH | crate::routes::status::STATUS => {
            *method == Method::GET
        }
        _ => path
            .strip_prefix(RESTRICTED_FACTORS_PATH)
            .is_some_and(|rest| {
                (rest.is_empty() || rest.starts_with('/'))
                    && !rest.contains('%')
                    && !rest.split('/').any(|part| part == "." || part == "..")
            }),
    }
}

#[derive(Debug)]
pub enum PasswordRefusal {
    Wrong(Option<Gate>),
    Throttled(u64),
    Busy,
    Internal(astrum_security::Error),
}

pub async fn check_password(
    state: &AppState,
    user: &str,
    password: &str,
    method: &str,
    nuget_key_allowed: bool,
) -> Result<Subject, PasswordRefusal> {
    let security = state.security();
    let address = crate::client::current();
    let by_key = nuget_key_allowed && api_keys::looks_like_api_key(password);
    let slot = match security.basic_begin(user, address) {
        Ok(slot) => slot,
        Err(denied) => {
            if by_key {
                match security
                    .authenticate_api_key_as(api_keys::NUGET, user, password)
                    .await
                {
                    Ok(subject) => return Ok(subject),
                    Err(AuthError::Internal(error)) => {
                        return Err(PasswordRefusal::Internal(error));
                    }
                    Err(AuthError::SecondFactor(_)) | Err(AuthError::BadCredentials) => {}
                }
            }
            return Err(match denied {
                BasicDenied::Blocked(wait) => {
                    PasswordRefusal::Throttled(wait.as_secs_f64().ceil().max(1.0) as u64)
                }
                BasicDenied::Busy => PasswordRefusal::Busy,
            });
        }
    };
    let mut refused_gate = None;
    match security.authenticate(user, password).await {
        Ok(subject) => {
            slot.succeeded();
            return Ok(subject);
        }
        Err(AuthError::Internal(error)) => return Err(PasswordRefusal::Internal(error)),
        Err(AuthError::SecondFactor(pending)) => refused_gate = Some(pending.gate),
        Err(AuthError::BadCredentials) => {}
    }
    if by_key {
        match security
            .authenticate_api_key_as(api_keys::NUGET, user, password)
            .await
        {
            Ok(subject) => return Ok(subject),
            Err(AuthError::Internal(error)) => return Err(PasswordRefusal::Internal(error)),
            Err(AuthError::SecondFactor(_)) | Err(AuthError::BadCredentials) => {}
        }
    }
    if slot.failed() && security.model().users.contains_key(user) {
        session::audit_rate_limited(state, user, address, method).await;
    }
    Err(PasswordRefusal::Wrong(refused_gate))
}

pub async fn audit_refused_password(
    state: &AppState,
    user: &str,
    path: &str,
    headers: &HeaderMap,
    gate: Option<Gate>,
) {
    let reason = gate.unwrap_or(Gate::Open).failure_reason();
    session::audit_failed_sign_in(state, user, path, headers, reason).await;
}

fn is_registry(path: &str) -> bool {
    path == "/v2" || path.starts_with("/v2/")
}

fn registry_refusal(message: &str, seconds: u64) -> Response {
    let body = serde_json::json!({
        "errors": [{
            "code": "TOOMANYREQUESTS",
            "message": message,
        }]
    });
    let mut response = (
        axum::http::StatusCode::TOO_MANY_REQUESTS,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, axum::http::HeaderValue::from(seconds));
    response
}

pub fn registry_throttled(seconds: u64) -> Response {
    registry_refusal("too many failed sign-ins; wait a few minutes", seconds)
}

pub fn registry_busy() -> Response {
    registry_refusal(CONCURRENT_MESSAGE, 1)
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
        if looks_like_token(value.trim()) {
            return Ok(Some(Credentials::Token(value.trim().to_owned())));
        }
        if looks_like_format_token(value.trim()).is_some() {
            return Ok(Some(Credentials::FormatToken(value.trim().to_owned())));
        }
        return Err(ApiError::Unauthorized);
    };
    if scheme.eq_ignore_ascii_case("bearer") || scheme.eq_ignore_ascii_case("token") {
        let token = encoded.trim();
        if looks_like_access_token(token) {
            return Ok(Some(Credentials::OAuth(token.to_owned())));
        }
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
