use astrum_db::audit::Event;
use astrum_outbound::HttpSettings;
use astrum_outbound::certificate::{self, FetchError};
use astrum_outbound::truststore::{self, TrustStoreError};
use astrum_security::ssrf::{Guard, SSRF_KEY, SsrfProtection};
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{ApiError, FieldError};
use crate::{AppState, Caller};

pub const SETTINGS_READ: &str = "astrum:settings:read";
pub const SETTINGS_UPDATE: &str = "astrum:settings:update";
pub const TRUSTSTORE_READ: &str = "astrum:ssl-truststore:read";
pub const TRUSTSTORE_CREATE: &str = "astrum:ssl-truststore:create";
pub const TRUSTSTORE_DELETE: &str = "astrum:ssl-truststore:delete";
pub const HTTP_DOMAIN: &str = "httpclient";
pub const TRUSTSTORE_DOMAIN: &str = "security.truststore";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/service/rest/v1/http", get(read_http).put(update_http))
        .route("/service/rest/v1/security/ssl", get(retrieve))
        .route(
            "/service/rest/v1/security/ssl/truststore",
            get(list_truststore).post(add_certificate),
        )
        .route(
            "/service/rest/v1/security/ssl/truststore/{id}",
            delete(remove_certificate),
        )
        .route(
            "/service/rest/internal/ui/ssl/certificate-details",
            post(details),
        )
}

async fn read_http(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(SETTINGS_READ)?;
    let settings = state
        .content()
        .outbound()
        .http_settings()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(settings.document()))
}

async fn update_http(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<HttpSettings>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let crate::Json(given) = body?;
    let outbound = state.content().outbound();
    let stored = outbound.http_settings().await.map_err(ApiError::internal)?;
    let secrets = state.secrets().clone();
    let seal = move |plain: &str| secrets.seal(plain);
    let prepared = given.prepare(&stored, &seal).map_err(|problems| {
        ApiError::Invalid(
            problems
                .into_iter()
                .map(|problem| FieldError {
                    id: problem.id,
                    message: problem.message,
                })
                .collect(),
        )
    })?;
    outbound
        .save_http_settings(&prepared)
        .await
        .map_err(ApiError::internal)?;
    let masked = prepared.masked();
    state
        .audit(
            &caller,
            Event::new(HTTP_DOMAIN, "updated", "http").with(json!({
                "userAgent": masked.user_agent,
                "timeout": masked.timeout,
                "retries": masked.retries,
                "httpProxy": masked.http_proxy.url(),
                "httpsProxy": masked.https_proxy.url(),
                "nonProxyHosts": masked.non_proxy_hosts,
            })),
        )
        .await;
    Ok(Json(prepared.document()))
}

fn certificate_error(message: impl Into<String>) -> ApiError {
    ApiError::BadRequest(message.into())
}

fn object(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(json!({"id": "*", "message": message.into()}))).into_response()
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Retrieve {
    host: Option<String>,
    port: Option<String>,
    protocol_hint: Option<String>,
}

fn host_and_port(given: &str, port: Option<&str>) -> Result<(String, u16), String> {
    let given = given.trim();
    let (scheme_port, authority) = match given.split_once("://") {
        Some((scheme, rest)) => (
            match scheme.to_ascii_lowercase().as_str() {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            },
            rest.split(['/', '?', '#']).next().unwrap_or_default(),
        ),
        None => (None, given),
    };
    let authority = authority.rsplit('@').next().unwrap_or_default();
    let (host, found_port) = match authority.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') || host.ends_with(']') => {
            (host.to_owned(), port.parse::<u16>().ok().or(scheme_port))
        }
        _ => (authority.to_owned(), scheme_port),
    };
    let port = match port.map(str::trim).filter(|port| !port.is_empty()) {
        Some(port) => port
            .parse::<u16>()
            .ok()
            .filter(|port| *port > 0)
            .ok_or_else(|| "port must be a number from 1 to 65535".to_owned())?,
        None => found_port.unwrap_or(443),
    };
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_owned();
    if host.is_empty() {
        return Err("host must not be empty".to_owned());
    }
    Ok((host, port))
}

async fn retrieve(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<Retrieve>,
) -> Result<Response, ApiError> {
    caller.require(TRUSTSTORE_READ)?;
    let Some(given) = query.host.as_deref().filter(|host| !host.trim().is_empty()) else {
        return Ok(object(StatusCode::BAD_REQUEST, "host must not be empty"));
    };
    let (host, port) = match host_and_port(given, query.port.as_deref()) {
        Ok(found) => found,
        Err(message) => return Ok(object(StatusCode::BAD_REQUEST, message)),
    };
    let settings: SsrfProtection = astrum_db::setting::get(state.db().conn(), SSRF_KEY)
        .await?
        .unwrap_or_default();
    let guard = Guard::new(&settings);
    match certificate::fetch(&guard, &host, port, query.protocol_hint.as_deref()).await {
        Ok(chain) => match certificate::describe(&chain[0]) {
            Ok(described) => Ok(Json(described).into_response()),
            Err(error) => Ok(object(StatusCode::BAD_REQUEST, error.to_string())),
        },
        Err(FetchError::Blocked(message) | FetchError::Failed(message)) => {
            Ok(object(StatusCode::BAD_REQUEST, message))
        }
    }
}

async fn list_truststore(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<Json<Vec<certificate::Certificate>>, ApiError> {
    caller.require(TRUSTSTORE_READ)?;
    Ok(Json(
        truststore::list(state.db().conn())
            .await?
            .into_iter()
            .map(|stored| stored.certificate)
            .collect(),
    ))
}

fn pem_text(body: &Bytes) -> Result<String, ApiError> {
    std::str::from_utf8(body)
        .map(str::to_owned)
        .map_err(|_| certificate_error("the body must be a PEM encoded certificate"))
}

async fn add_certificate(
    State(state): State<AppState>,
    caller: Caller,
    body: Bytes,
) -> Result<Response, ApiError> {
    caller.require(TRUSTSTORE_CREATE)?;
    let text = pem_text(&body)?;
    match truststore::add(state.db().conn(), &text).await {
        Ok(added) => {
            state.content().outbound().forget();
            state.security().reload().await?;
            state
                .audit(
                    &caller,
                    Event::new(TRUSTSTORE_DOMAIN, "created", &added.id).with(json!({
                        "subjectCommonName": added.subject_common_name,
                        "issuerCommonName": added.issuer_common_name,
                    })),
                )
                .await;
            Ok((StatusCode::CREATED, Json(added)).into_response())
        }
        Err(TrustStoreError::Duplicate(id)) => Ok(object(
            StatusCode::CONFLICT,
            format!("the certificate {id} is already in the truststore"),
        )),
        Err(TrustStoreError::Certificate(error)) => {
            Ok(object(StatusCode::BAD_REQUEST, error.to_string()))
        }
        Err(TrustStoreError::Database(error)) => Err(error.into()),
    }
}

async fn remove_certificate(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(TRUSTSTORE_DELETE)?;
    let removed = truststore::remove(state.db().conn(), &id).await?;
    let Some(removed) = removed else {
        return Err(ApiError::not_found(format!(
            "there is no certificate {id} in the truststore"
        )));
    };
    state.content().outbound().forget();
    state.security().reload().await?;
    state
        .audit(
            &caller,
            Event::new(TRUSTSTORE_DOMAIN, "deleted", &removed.id).with(json!({
                "subjectCommonName": removed.subject_common_name,
            })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn details(caller: Caller, body: Bytes) -> Result<Response, ApiError> {
    caller.require(TRUSTSTORE_READ)?;
    let text = pem_text(&body)?;
    let text = match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => map
            .get("pem")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_default(),
        _ => text,
    };
    match certificate::der_from_pem(&text).and_then(|der| certificate::describe(&der)) {
        Ok(described) => Ok(Json(described).into_response()),
        Err(error) => Ok(object(StatusCode::BAD_REQUEST, error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts_come_as_names_pairs_or_urls() {
        assert_eq!(
            host_and_port("example.com", None).unwrap(),
            ("example.com".to_owned(), 443)
        );
        assert_eq!(
            host_and_port("example.com:8443", None).unwrap(),
            ("example.com".to_owned(), 8443)
        );
        assert_eq!(
            host_and_port("https://repo.example/maven2/", None).unwrap(),
            ("repo.example".to_owned(), 443)
        );
        assert_eq!(
            host_and_port("http://repo.example:8080/x", None).unwrap(),
            ("repo.example".to_owned(), 8080)
        );
        assert_eq!(
            host_and_port("mail.example", Some("587")).unwrap(),
            ("mail.example".to_owned(), 587)
        );
        assert_eq!(
            host_and_port("[::1]:9443", None).unwrap(),
            ("::1".to_owned(), 9443)
        );
        assert!(host_and_port("x", Some("0")).is_err());
    }
}
