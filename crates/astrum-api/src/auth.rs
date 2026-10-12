use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use astrum_security::{AuthError, Subject};

use crate::AppState;
use crate::error::ApiError;

pub const UI_HEADER: &str = "x-astrum-ui";

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

pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let from_ui = is_ui(request.headers());
    let caller = match resolve(&state, request.headers()).await {
        Ok(caller) => caller,
        Err(error) => return without_challenge(error.into_response(), from_ui),
    };
    request.extensions_mut().insert(caller);
    without_challenge(next.run(request).await, from_ui)
}

async fn resolve(state: &AppState, headers: &HeaderMap) -> Result<Caller, ApiError> {
    if let Some((user, password)) = basic_credentials(headers)? {
        return match state.security().authenticate(&user, &password).await {
            Ok(subject) => Ok(Caller(Some(subject))),
            Err(AuthError::BadCredentials) => {
                tracing::info!(user, "a sign-in failed");
                Err(ApiError::Unauthorized)
            }
            Err(AuthError::Internal(error)) => Err(error.into()),
        };
    }
    Ok(Caller(state.security().anonymous()))
}

fn basic_credentials(headers: &HeaderMap) -> Result<Option<(String, String)>, ApiError> {
    let Some(value) = headers.get(header::AUTHORIZATION) else {
        return Ok(None);
    };
    let Ok(value) = value.to_str() else {
        return Err(ApiError::Unauthorized);
    };
    let Some((scheme, encoded)) = value.trim().split_once(' ') else {
        return Err(ApiError::Unauthorized);
    };
    if !scheme.eq_ignore_ascii_case("basic") {
        return Ok(None);
    }
    let decoded = STANDARD
        .decode(encoded.trim())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(ApiError::Unauthorized)?;
    let (user, password) = decoded.split_once(':').ok_or(ApiError::Unauthorized)?;
    Ok(Some((user.to_owned(), password.to_owned())))
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
