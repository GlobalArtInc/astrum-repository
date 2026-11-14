use astrum_content::share::{self, Link, NewLink};
use astrum_content::{ContentError, Grant, View, normalize};
use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_repo::RepositoryType;
use astrum_repo::config::Repository;
use astrum_security::{Scope, Subject};
use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use chrono::{DateTime, Duration, FixedOffset, SecondsFormat, Utc};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::{ApiError, Validation};
use crate::routes::repositories::BaseUrl;
use crate::{AppState, Caller};

pub const PATH: &str = "/service/rest/v1/share-links";
pub const DOMAIN: &str = "security.sharelink";
pub const CREATE: &str = "astrum:sharelinks:create";
pub const READ: &str = "astrum:sharelinks:read";
pub const DELETE: &str = "astrum:sharelinks:delete";
pub const MAX_DAYS_KEY: &str = "shareLinks.maxDays";
pub const DEFAULT_MAX_DAYS: u32 = 30;
const DEFAULT_HOURS: i64 = 7 * 24;
const NOTE_LIMIT: usize = 500;
const CONTROL_TEXT: &str = "must not contain control characters";
const DIRECTION_TEXT: &str = "must not contain control or text direction characters";

const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(PATH, get(list).post(create))
        .route("/service/rest/v1/share-links/{id}", delete(revoke))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateRequest {
    repository: String,
    path: String,
    #[serde(default)]
    prefix: Option<bool>,
    #[serde(default)]
    expires_in_hours: Option<i64>,
    #[serde(default)]
    max_downloads: Option<i32>,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    all: bool,
}

fn member<'a>(caller: &'a Caller, permission: &str) -> Result<&'a Subject, ApiError> {
    caller.signed_in()?;
    caller.require(permission)
}

fn content_error(error: ContentError) -> ApiError {
    match error {
        ContentError::NotFound(message) => ApiError::NotFound(message),
        ContentError::BadRequest(message) => ApiError::BadRequest(message),
        ContentError::Unauthorized => ApiError::Unauthorized,
        ContentError::Forbidden => ApiError::forbidden(),
        other => ApiError::internal(other),
    }
}

fn time(at: DateTime<FixedOffset>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn has_control(text: &str) -> bool {
    text.chars().any(char::is_control)
}

fn has_direction(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{061C}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
        )
    })
}

fn owns(subject: &Subject, link: &Link) -> bool {
    link.created_by == subject.user_id && link.created_source == subject.source
}

fn view(link: &Link) -> Value {
    json!({
        "id": link.id,
        "repository": link.repository,
        "path": link.path,
        "prefix": link.prefix,
        "note": link.note,
        "createdBy": link.created_by,
        "createdAt": time(link.created_at),
        "expiresAt": time(link.expires_at),
        "maxDownloads": link.max_downloads,
        "downloads": link.downloads,
        "bytesServed": link.bytes_served,
        "lastUsedAt": link.last_used_at.map(time),
        "expired": link.expires_at.with_timezone(&Utc) <= Utc::now(),
    })
}

fn origin(repository: &Repository, base: &str) -> String {
    match repository.domain() {
        Some((host, _)) => {
            let (scheme, rest) = base.split_once("://").unwrap_or(("https", base));
            let authority = rest.split('/').next().unwrap_or_default();
            let port = authority
                .rsplit_once(':')
                .map(|(_, port)| port)
                .filter(|port| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()));
            match port {
                Some(port) if !host.contains(':') => format!("{scheme}://{host}:{port}"),
                _ => format!("{scheme}://{host}"),
            }
        }
        None => base.trim_end_matches('/').to_owned(),
    }
}

fn address(repository: &Repository, base: &str, link: &Link, secret: &str) -> String {
    let origin = origin(repository, base);
    if link.prefix {
        return format!("{origin}/service/share/{secret}/");
    }
    let name = link.path.rsplit('/').next().unwrap_or_default();
    format!(
        "{origin}/service/share/{secret}/{}",
        utf8_percent_encode(name, SEGMENT)
    )
}

async fn maximum_hours(state: &AppState) -> Result<i64, ApiError> {
    let days = match setting::get::<u32>(state.db().conn(), MAX_DAYS_KEY).await {
        Ok(days) => days.unwrap_or(DEFAULT_MAX_DAYS),
        Err(error @ astrum_db::Error::Setting { .. }) => {
            tracing::warn!(
                key = MAX_DAYS_KEY,
                %error,
                default = DEFAULT_MAX_DAYS,
                "the share link lifetime setting is unreadable, using the default"
            );
            DEFAULT_MAX_DAYS
        }
        Err(error) => return Err(error.into()),
    };
    Ok(i64::from(days) * 24)
}

fn expiry(requested: Option<i64>, maximum: i64) -> Result<DateTime<Utc>, ApiError> {
    if maximum < 1 {
        return Err(ApiError::invalid(
            "expiresInHours",
            "share links are switched off: the setting shareLinks.maxDays is 0",
        ));
    }
    let hours = requested.unwrap_or_else(|| DEFAULT_HOURS.min(maximum));
    if !(1..=maximum).contains(&hours) {
        return Err(ApiError::invalid(
            "expiresInHours",
            format!("must be between 1 and {maximum} (the setting shareLinks.maxDays)"),
        ));
    }
    Duration::try_hours(hours)
        .and_then(|span| Utc::now().checked_add_signed(span))
        .ok_or_else(|| ApiError::invalid("expiresInHours", "is too far in the future"))
}

fn checked_target(request: &CreateRequest) -> Result<(String, bool), ApiError> {
    let mut validation = Validation::new();
    validation.check(
        !request.repository.trim().is_empty(),
        "repository",
        "must not be empty",
    );
    validation.check(
        !has_control(&request.repository),
        "repository",
        CONTROL_TEXT,
    );
    validation.check(!request.path.trim().is_empty(), "path", "must not be empty");
    validation.check(
        request
            .note
            .as_deref()
            .is_none_or(|note| note.chars().count() <= NOTE_LIMIT),
        "note",
        "must be at most 500 characters",
    );
    validation.check(
        request
            .note
            .as_deref()
            .is_none_or(|note| !has_control(note) && !has_direction(note)),
        "note",
        DIRECTION_TEXT,
    );
    validation.check(
        request.max_downloads.is_none_or(|maximum| maximum >= 1),
        "maxDownloads",
        "must be at least 1",
    );
    validation.finish()?;
    let path = normalize(&request.path).map_err(|error| match error {
        ContentError::BadRequest(message) => ApiError::invalid("path", message),
        other => content_error(other),
    })?;
    let prefix = request.prefix.unwrap_or_else(|| path.ends_with('/'));
    if prefix != path.ends_with('/') {
        return Err(ApiError::invalid(
            "prefix",
            if prefix {
                "the path of a prefix link must end with a slash"
            } else {
                "the path of a single asset link must not end with a slash"
            },
        ));
    }
    if prefix && request.max_downloads.is_some() {
        return Err(ApiError::invalid(
            "maxDownloads",
            "a prefix link cannot have a download limit; for several files create several links",
        ));
    }
    Ok((path, prefix))
}

fn shareable(repository: &Repository) -> Result<(), ApiError> {
    if repository.kind != RepositoryType::Hosted {
        return Err(ApiError::invalid(
            "repository",
            format!(
                "{} is a {} repository; share links work on hosted repositories only",
                repository.name, repository.kind
            ),
        ));
    }
    if repository.format.id == "docker" {
        return Err(ApiError::invalid(
            "repository",
            "share links do not work on docker repositories",
        ));
    }
    if !repository.online {
        return Err(ApiError::invalid(
            "repository",
            format!("{} is offline", repository.name),
        ));
    }
    Ok(())
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    BaseUrl(base): BaseUrl,
    body: Result<crate::Json<CreateRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let subject = member(&caller, CREATE)?;
    let crate::Json(request) = body?;
    let (path, prefix) = checked_target(&request)?;
    let expires_at = expiry(request.expires_in_hours, maximum_hours(&state).await?)?;
    let name = request.repository.trim();
    let repository = state
        .content()
        .repository(name)
        .await
        .map_err(content_error)?
        .ok_or_else(|| ApiError::not_found(format!("there is no repository named {name}")))?;
    let ctx = state.content().context(
        repository.clone(),
        state.security().model(),
        Some(subject.clone()),
        "",
        "",
    );
    if prefix {
        if !matches!(ctx.grant(View::Read), Grant::Whole) {
            return Err(ApiError::Forbidden(
                "a prefix link needs read access to the whole repository".to_owned(),
            ));
        }
    } else {
        ctx.require(View::Read, &path).map_err(content_error)?;
    }
    shareable(&repository)?;
    if share::count_active(state.db().conn(), &subject.source, &subject.user_id)
        .await
        .map_err(content_error)?
        >= share::MAX_ACTIVE_PER_USER
    {
        return Err(ApiError::BadRequest(format!(
            "you already have {} active share links; revoke some first",
            share::MAX_ACTIVE_PER_USER
        )));
    }
    let (link, secret) = share::create(
        state.db().conn(),
        NewLink {
            repository: repository.name.clone(),
            path,
            prefix,
            note: request.note.unwrap_or_default().trim().to_owned(),
            created_by: subject.user_id.clone(),
            created_source: subject.source.clone(),
            expires_at,
            max_downloads: request.max_downloads,
        },
    )
    .await
    .map_err(content_error)?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "created", link.id.to_string()).with(json!({
                "repository": link.repository,
                "path": link.path,
                "prefix": link.prefix,
                "expiresAt": time(link.expires_at),
                "maxDownloads": link.max_downloads,
            })),
        )
        .await;
    let mut document = view(&link);
    if let Value::Object(map) = &mut document {
        map.insert(
            "url".to_owned(),
            Value::String(address(&repository, &base, &link, &secret)),
        );
    }
    Ok((
        StatusCode::CREATED,
        [(header::CACHE_CONTROL, "no-store")],
        Json(document),
    )
        .into_response())
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    let subject = caller.signed_in()?;
    if query.all {
        caller.require(READ)?;
    }
    let creator = (!query.all).then_some((subject.source.as_str(), subject.user_id.as_str()));
    if query.repository.as_deref().is_some_and(has_control) {
        return Err(ApiError::invalid("repository", CONTROL_TEXT));
    }
    let repository = query
        .repository
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    let links = share::list(state.db().conn(), creator, repository)
        .await
        .map_err(content_error)?;
    Ok(Json(Value::Array(links.iter().map(view).collect())))
}

async fn revoke(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let subject = caller.signed_in()?;
    let missing = || ApiError::not_found("there is no such share link");
    let id = Uuid::parse_str(&id).map_err(|_| missing())?;
    let link = share::find(state.db().conn(), id)
        .await
        .map_err(content_error)?
        .ok_or_else(missing)?;
    if !owns(subject, &link) {
        caller.require(DELETE)?;
    } else if subject.scope == Scope::Read {
        return Err(ApiError::forbidden());
    }
    if !share::delete(state.db().conn(), id)
        .await
        .map_err(content_error)?
    {
        return Err(missing());
    }
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "deleted", link.id.to_string()).with(json!({
                "repository": link.repository,
                "path": link.path,
                "prefix": link.prefix,
                "createdBy": link.created_by,
            })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn link(created_by: &str, created_source: &str) -> Link {
        let now = Utc::now().fixed_offset();
        Link {
            id: Uuid::nil(),
            secret_hash: String::new(),
            repository: "files".to_owned(),
            path: "/a".to_owned(),
            prefix: false,
            note: String::new(),
            created_by: created_by.to_owned(),
            created_source: created_source.to_owned(),
            created_at: now,
            expires_at: now,
            max_downloads: None,
            downloads: 0,
            bytes_served: 0,
            last_used_at: None,
            last_used_ip: None,
        }
    }

    fn subject(user: &str, source: &str) -> Subject {
        Subject::new(
            user,
            source,
            "realm",
            BTreeSet::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn a_link_belongs_to_the_pair_of_source_and_user() {
        let local = link("alice", "default");
        assert!(owns(&subject("alice", "default"), &local));
        assert!(!owns(&subject("alice", "LDAP"), &local));
        assert!(!owns(&subject("bob", "default"), &local));
        assert!(owns(&subject("alice", "LDAP"), &link("alice", "LDAP")));
        assert!(!owns(&subject("alice", "default"), &link("alice", "LDAP")));
    }
}
