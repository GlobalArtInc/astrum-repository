use astrum_security::oidc::{self, Callback, Failure, OidcError, PendingLogin};
use astrum_security::realm;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

use crate::auth::cookie;
use crate::routes::internal::ui::identity::http;
use crate::routes::internal::{is_https, session};
use crate::routes::repositories::BaseUrl;
use crate::{ApiError, AppState};

pub const STATE_COOKIE: &str = "astrum_oidc";
const COOKIE_PATH: &str = "/oidc";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(oidc::LOGIN_PATH, get(login))
        .route(oidc::CALLBACK_PATH, get(callback))
}

#[derive(Debug, Deserialize)]
struct LoginQuery {
    hash: Option<String>,
}

fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn redirect(location: &str) -> Response {
    let mut response = StatusCode::FOUND.into_response();
    if let Ok(value) = HeaderValue::from_str(location) {
        response.headers_mut().insert(header::LOCATION, value);
    }
    no_store(response)
}

fn state_cookie(response: &mut Response, value: &str, secure: bool, max_age: i64) {
    let mut cookie = format!(
        "{STATE_COOKIE}={value}; Path={COOKIE_PATH}; SameSite=Lax; HttpOnly; Max-Age={max_age}"
    );
    if secure {
        cookie.push_str("; Secure");
    }
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

fn not_enabled() -> Response {
    no_store(
        (
            StatusCode::NOT_FOUND,
            "OpenID Connect sign-in is not enabled\n",
        )
            .into_response(),
    )
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    BaseUrl(base_url): BaseUrl,
    Query(query): Query<LoginQuery>,
) -> Result<Response, ApiError> {
    let model = state.security().model();
    let Some(settings) = model
        .oidc
        .clone()
        .filter(|_| model.realm_active(realm::OIDC))
    else {
        return Ok(not_enabled());
    };
    let http = http(&state, &settings).await?;
    let metadata = match state.security().oidc().metadata(&settings, &http).await {
        Ok(metadata) => metadata,
        Err(error) => return Ok(failed(&headers, &error)),
    };
    let redirect_uri = format!("{base_url}{}", oidc::CALLBACK_PATH);
    let return_to = oidc::return_path(query.hash.as_deref());
    let now = chrono::Utc::now().timestamp();
    let (pending, location) = match oidc::begin(&settings, &metadata, &redirect_uri, return_to, now)
    {
        Ok(started) => started,
        Err(error) => return Ok(failed(&headers, &error)),
    };
    let sealed = state
        .secrets()
        .seal(&serde_json::to_string(&pending).map_err(ApiError::internal)?);
    let mut response = redirect(&location);
    state_cookie(
        &mut response,
        &URL_SAFE_NO_PAD.encode(sealed),
        is_https(&headers),
        oidc::LOGIN_TTL_SECONDS,
    );
    Ok(response)
}

fn pending(state: &AppState, headers: &HeaderMap) -> Option<PendingLogin> {
    let value = cookie(headers, STATE_COOKIE)?;
    let sealed = URL_SAFE_NO_PAD
        .decode(value)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())?;
    let opened = state.secrets().open(&sealed).ok()?;
    serde_json::from_str(&opened).ok()
}

fn failed(headers: &HeaderMap, error: &OidcError) -> Response {
    tracing::warn!(kind = error.kind.code(), error = %error.message, "an OpenID Connect sign-in failed");
    let mut response = redirect(&format!("/login?sso={}", error.kind.code()));
    state_cookie(&mut response, "", is_https(headers), 0);
    response
}

async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(callback): Query<Callback>,
) -> Result<Response, ApiError> {
    let model = state.security().model();
    let Some(settings) = model
        .oidc
        .clone()
        .filter(|_| model.realm_active(realm::OIDC))
    else {
        return Ok(not_enabled());
    };
    let now = chrono::Utc::now().timestamp();
    let pending = pending(&state, &headers);
    let (pending, code) = match oidc::check_callback(pending.as_ref(), &callback, now) {
        Ok(checked) => checked,
        Err(error) => return Ok(failed(&headers, &error)),
    };
    let http = http(&state, &settings).await?;
    let provider = state.security().oidc();
    let secret = match settings.client_secret.as_deref() {
        Some(sealed) => match state.secrets().open(sealed) {
            Ok(secret) => Some(secret),
            Err(error) => {
                return Ok(failed(
                    &headers,
                    &OidcError::failed(format!("the client secret cannot be read: {error}")),
                ));
            }
        },
        None => None,
    };
    let signed_in = async {
        let tokens = provider
            .exchange_code(
                &settings,
                secret.as_deref(),
                &http,
                &code,
                &pending.redirect_uri,
                &pending.verifier,
            )
            .await?;
        let mut claims = provider
            .verify_id_token(&settings, &http, &tokens.id_token, &pending.nonce)
            .await?;
        if let Some(access_token) = tokens.access_token.as_deref() {
            match provider.user_info(&settings, &http, access_token).await {
                Ok(user_info) => oidc::merge_user_info(&mut claims, user_info),
                Err(error) => tracing::warn!(error = %error.message, "the OpenID Connect user info could not be read; the ID token claims are used alone"),
            }
        }
        let user = oidc::user_from_claims(&settings, &claims)?;
        if user.user_id == model.anonymous.user_id {
            return Err(OidcError::denied(format!(
                "The username '{}' is reserved",
                user.user_id
            )));
        }
        Ok(user)
    }
    .await;
    let user = match signed_in {
        Ok(user) => user,
        Err(error) => return Ok(failed(&headers, &error)),
    };
    let Some(subject) = state.security().oidc_sign_in(&user).await? else {
        return Ok(failed(
            &headers,
            &OidcError::new(Failure::Failed, "the OpenID Connect realm is not active"),
        ));
    };
    tracing::info!(user = %subject.user_id, "a user signed in with OpenID Connect");
    let mut response = redirect(pending.return_to.as_deref().unwrap_or("/"));
    session::begin(&state, &headers, &subject, &mut response).await?;
    state_cookie(&mut response, "", is_https(&headers), 0);
    Ok(response)
}
