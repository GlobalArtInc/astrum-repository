use astrum_content::share::{self, Link, Portion};
use astrum_content::{Asset, ContentError, Delivery, Serve, normalize, store};
use astrum_repo::RepositoryType;
use axum::Router;
use axum::body::Body;
use axum::extract::rejection::PathRejection;
use axum::extract::{Path, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use chrono::Utc;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::AppState;
use crate::routes::html::{self, escape};
use crate::routes::internal::client_ip;

const INDEX_LIMIT: u64 = 1000;
const MISSING: &str = "Not found\n";
const GONE: &str = "This link has expired or has no downloads left.\n";

pub const PREFIX: &str = "/service/share/";

pub fn is_endpoint(path: &str) -> bool {
    path.starts_with(PREFIX)
}

const FILENAME: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'#')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b'-')
    .remove(b'.')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'|')
    .remove(b'~');

const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(PREFIX, any(empty))
        .route("/service/share/{secret}", any(single))
        .route("/service/share/{secret}/", any(single))
        .route("/service/share/{secret}/{*rest}", any(nested))
}

enum Refusal {
    Missing,
    Gone,
    Method,
    Failed(ContentError),
}

impl From<ContentError> for Refusal {
    fn from(error: ContentError) -> Self {
        match error {
            ContentError::NotFound(_) | ContentError::BadRequest(_) => Refusal::Missing,
            other => Refusal::Failed(other),
        }
    }
}

impl From<sea_orm::DbErr> for Refusal {
    fn from(error: sea_orm::DbErr) -> Self {
        Refusal::Failed(error.into())
    }
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        let text = |status: StatusCode, body: &'static str| {
            (
                status,
                [(header::CONTENT_TYPE, "text/plain;charset=utf-8")],
                body,
            )
                .into_response()
        };
        match self {
            Refusal::Missing => text(StatusCode::NOT_FOUND, MISSING),
            Refusal::Gone => text(StatusCode::GONE, GONE),
            Refusal::Method => {
                let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
                response
                    .headers_mut()
                    .insert(header::ALLOW, HeaderValue::from_static("GET, HEAD"));
                response
            }
            Refusal::Failed(error) => error.into_response(),
        }
    }
}

async fn empty(State(state): State<AppState>, request: Request) -> Response {
    respond(state, String::new(), String::new(), request).await
}

async fn single(
    State(state): State<AppState>,
    path: Result<Path<String>, PathRejection>,
    request: Request,
) -> Response {
    let secret = path.map(|Path(secret)| secret).unwrap_or_default();
    respond(state, secret, String::new(), request).await
}

async fn nested(
    State(state): State<AppState>,
    path: Result<Path<(String, String)>, PathRejection>,
    request: Request,
) -> Response {
    let (secret, rest) = path.map(|Path(both)| both).unwrap_or_default();
    respond(state, secret, rest, request).await
}

async fn respond(state: AppState, secret: String, rest: String, request: Request) -> Response {
    let (parts, _body) = request.into_parts();
    let mut response = answer(&state, &secret, &rest, &parts)
        .await
        .unwrap_or_else(IntoResponse::into_response);
    harden(response.headers_mut());
    response
}

fn harden(headers: &mut HeaderMap) {
    for (name, value) in [
        (header::CACHE_CONTROL, "private, no-store"),
        (header::REFERRER_POLICY, "no-referrer"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::CONTENT_SECURITY_POLICY, "sandbox"),
        (header::HeaderName::from_static("x-robots-tag"), "noindex"),
    ] {
        headers.insert(name, HeaderValue::from_static(value));
    }
}

fn attachment(path: &str) -> Result<HeaderValue, Refusal> {
    let name = path.rsplit('/').next().unwrap_or_default();
    HeaderValue::from_str(&format!(
        "attachment; filename*=UTF-8''{}",
        utf8_percent_encode(name, FILENAME)
    ))
    .map_err(|error| Refusal::Failed(ContentError::internal(error)))
}

enum Target {
    Index,
    Asset(String),
}

fn target(link: &Link, rest: &str) -> Result<Target, Refusal> {
    if !link.prefix {
        return Ok(Target::Asset(link.path.clone()));
    }
    let rest = normalize(rest)?;
    if rest == "/" {
        return Ok(Target::Index);
    }
    if rest.ends_with('/') {
        return Err(Refusal::Missing);
    }
    let full = format!("{}{rest}", link.path.trim_end_matches('/'));
    Ok(Target::Asset(full))
}

async fn answer(
    state: &AppState,
    secret: &str,
    rest: &str,
    request: &Parts,
) -> Result<Response, Refusal> {
    let method = &request.method;
    if method != Method::GET && method != Method::HEAD {
        return Err(Refusal::Method);
    }
    let db = state.content().db();
    let link = share::by_secret(db, secret)
        .await?
        .ok_or(Refusal::Missing)?;
    let now = Utc::now();
    if link.expires_at.with_timezone(&Utc) <= now {
        return Err(Refusal::Gone);
    }
    let repository = state
        .content()
        .repository(&link.repository)
        .await?
        .filter(|repository| {
            repository.online
                && repository.kind == RepositoryType::Hosted
                && repository.format.id != "docker"
        })
        .ok_or(Refusal::Missing)?;
    let path = match target(&link, rest)? {
        Target::Index => {
            let assets = store::under(db, &link.repository, &link.path, INDEX_LIMIT + 1).await?;
            return Ok(index(secret, &link, &assets, method));
        }
        Target::Asset(path) => path,
    };
    let asset = store::find(db, &link.repository, &path)
        .await?
        .ok_or(Refusal::Missing)?;
    let size = asset.size.max(0) as u64;
    let delivery = astrum_content::delivery(&asset, &request.headers);
    let portion = match delivery {
        Delivery::Body { offset, length } => Portion {
            offset,
            length,
            size,
        },
        Delivery::Empty => Portion::whole(size),
    };
    let counted = *method == Method::GET && matches!(delivery, Delivery::Body { .. });
    if counted && !share::permits(&link, portion, now) {
        return Err(Refusal::Gone);
    }
    let address = crate::auth::client_address().unwrap_or_else(|| client_ip(&request.headers));
    if counted && !share::consume(db, link.id, &address, portion).await? {
        return Err(Refusal::Gone);
    }
    let ctx = state.content().context(
        repository,
        state.security().model(),
        None,
        String::new(),
        address,
    );
    let served =
        astrum_content::serve(&ctx, &asset, method, &request.headers, Serve::download()).await;
    let outcome = match served {
        Ok(mut response) => attachment(&asset.path).map(|disposition| {
            response
                .headers_mut()
                .insert(header::CONTENT_DISPOSITION, disposition);
            response
        }),
        Err(error) => Err(error.into()),
    };
    let failed = match &outcome {
        Ok(response) => response.status().is_server_error(),
        Err(_) => true,
    };
    if counted
        && failed
        && let Err(error) = share::refund(db, link.id, portion).await
    {
        tracing::warn!(%error, "a share link download was not given back");
    }
    outcome
}

fn index(secret: &str, link: &Link, assets: &[Asset], method: &Method) -> Response {
    let shown = assets.iter().take(INDEX_LIMIT as usize);
    let rows: String = shown
        .map(|asset| {
            let relative = asset
                .path
                .strip_prefix(link.path.as_str())
                .unwrap_or_default();
            let encoded: Vec<String> = relative
                .split('/')
                .map(|segment| utf8_percent_encode(segment, SEGMENT).to_string())
                .collect();
            format!(
                "<tr><td><a href=\"/service/share/{secret}/{}\">{}</a></td><td>{}</td></tr>\n",
                encoded.join("/"),
                escape(relative),
                asset.size
            )
        })
        .collect();
    let more = if assets.len() as u64 > INDEX_LIMIT {
        format!("<p>Only the first {INDEX_LIMIT} files are listed.</p>\n")
    } else {
        String::new()
    };
    let body = format!(
        "{more}<table>\n<thead><tr><th>File</th><th>Bytes</th></tr></thead>\n<tbody>\n{rows}</tbody>\n</table>\n"
    );
    let page = html::page("Shared files", &body);
    let body = if *method == Method::HEAD {
        Body::empty()
    } else {
        Body::from(page)
    };
    let mut response = Response::new(body);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html;charset=utf-8"),
    );
    response
}
