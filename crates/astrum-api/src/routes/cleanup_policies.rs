use std::collections::BTreeMap;
use std::time::Duration;

use astrum_db::audit::Event;
use astrum_repo::RepositoryType;
use astrum_tasks::Problems;
use astrum_tasks::cleanup::query::{self, Matcher};
use astrum_tasks::cleanup::{self, Policy, PolicyError, PolicyRequest};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::ApiError;
use crate::routes::tasks::{problems, status};
use crate::{AppState, Caller};

pub const INTERNAL: &str = "/service/rest/internal/cleanup-policies";
pub const PUBLIC: &str = "/service/rest/v1/cleanup-policies";
pub const DOMAIN: &str = "cleanupPolicy";
pub const READ: &str = "astrum:cleanup-policies:read";
pub const CREATE: &str = "astrum:cleanup-policies:create";
pub const UPDATE: &str = "astrum:cleanup-policies:update";
pub const DELETE: &str = "astrum:cleanup-policies:delete";
pub const PREVIEW_PAGE: i64 = 50;
pub const PREVIEW_MAX_PAGE: i64 = 500;
const PREVIEW_TOTAL_LIMIT: Duration = Duration::from_secs(30);
const CSV_BATCH: i64 = 500;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(INTERNAL, get(list).post(create_internal))
        .route(
            "/service/rest/internal/cleanup-policies/criteria/formats",
            get(formats),
        )
        .route(
            "/service/rest/internal/cleanup-policies/preview/components",
            post(preview),
        )
        .route(
            "/service/rest/internal/cleanup-policies/preview/summary",
            post(summary),
        )
        .route(
            "/service/rest/internal/cleanup-policies/preview/components/csv",
            get(csv),
        )
        .route(
            "/service/rest/internal/cleanup-policies/{name}",
            get(read_internal).put(update_internal).delete(remove),
        )
        .route(
            "/service/rest/internal/cleanup-policies/{name}/repositories",
            get(repositories),
        )
        .route(PUBLIC, get(list_public).post(create_public))
        .route(
            "/service/rest/v1/cleanup-policies/{name}",
            get(read_public).put(update_public).delete(remove),
        )
}

type Users = Vec<(String, String, String, Vec<String>)>;

async fn users(state: &AppState) -> Result<Users, ApiError> {
    cleanup::users(state.db().conn())
        .await
        .map_err(ApiError::internal)
}

fn public(policy: &Policy) -> Value {
    let mut document = policy.document(&[]);
    if let Value::Object(map) = &mut document {
        map.remove("inUseCount");
        map.remove("repositories");
        map.remove("sortBy");
    }
    document
}

fn missing(name: &str) -> ApiError {
    ApiError::not_found(format!("there is no cleanup policy named {name}"))
}

async fn existing(state: &AppState, name: &str) -> Result<Policy, ApiError> {
    cleanup::find(state.db().conn(), name)
        .await?
        .ok_or_else(|| missing(name))
}

async fn list(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let users = users(&state).await?;
    let policies = cleanup::list(state.db().conn()).await?;
    Ok(Json(Value::Array(
        policies
            .iter()
            .map(|policy| policy.document(&cleanup::repositories_using(&users, &policy.name)))
            .collect(),
    )))
}

async fn list_public(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let policies = cleanup::list(state.db().conn()).await?;
    Ok(Json(Value::Array(policies.iter().map(public).collect())))
}

async fn read_internal(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let policy = existing(&state, &name).await?;
    let users = users(&state).await?;
    Ok(Json(policy.document(&cleanup::repositories_using(
        &users,
        &policy.name,
    ))))
}

async fn read_public(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    Ok(Json(public(&existing(&state, &name).await?)))
}

async fn repositories(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let policy = existing(&state, &name).await?;
    let users = users(&state).await?;
    Ok(Json(Value::Array(
        users
            .iter()
            .filter(|(_, _, _, names)| names.contains(&policy.name))
            .map(|(name, format, kind, _)| json!({"name": name, "format": format, "type": kind}))
            .collect(),
    )))
}

async fn formats(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let mut formats = vec![json!({
        "id": cleanup::ALL_FORMATS,
        "name": "All Formats",
        "availableCriteria": cleanup::criteria_of(cleanup::ALL_FORMATS),
    })];
    for format in state.content().formats() {
        formats.push(json!({
            "id": format.id(),
            "name": format.id(),
            "availableCriteria": cleanup::criteria_of(format.id()),
        }));
    }
    Ok(Json(Value::Array(formats)))
}

fn validated(request: &PolicyRequest) -> Result<Policy, ApiError> {
    let mut problems = Problems::default();
    match cleanup::validate(request, &cleanup::known_formats(), &mut problems) {
        Some(policy) => Ok(policy),
        None => Err(self::problems(problems.into_inner())),
    }
}

async fn create(
    state: &AppState,
    caller: &Caller,
    request: PolicyRequest,
) -> Result<Policy, ApiError> {
    caller.require(CREATE)?;
    let policy = validated(&request)?;
    if cleanup::find_ignoring_case(state.db().conn(), &policy.name)
        .await?
        .is_some()
    {
        return Err(ApiError::invalid(
            "name",
            "is already used by another cleanup policy (names ignore case)",
        ));
    }
    cleanup::create(state.db().conn(), &policy)
        .await
        .map_err(policy_error)?;
    state
        .audit(
            caller,
            Event::new(DOMAIN, "created", &policy.name).with(audit(&policy)),
        )
        .await;
    Ok(policy)
}

fn audit(policy: &Policy) -> Value {
    json!({"format": policy.format, "criteria": policy.criteria})
}

fn policy_error(error: PolicyError) -> ApiError {
    match error {
        PolicyError::Duplicate(_) => ApiError::invalid(
            "name",
            "is already used by another cleanup policy (names ignore case)",
        ),
        PolicyError::NotFound(name) => missing(&name),
        PolicyError::Refused(message) => ApiError::BadRequest(message),
        PolicyError::Database(error) => error.into(),
        PolicyError::Repositories(error) => ApiError::internal(error),
    }
}

async fn create_internal(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<PolicyRequest>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    let crate::Json(request) = body?;
    let policy = create(&state, &caller, request).await?;
    Ok(Json(policy.document(&[])))
}

async fn create_public(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<PolicyRequest>, ApiError>,
) -> Result<Response, ApiError> {
    let crate::Json(request) = body?;
    let policy = create(&state, &caller, request).await?;
    Ok((StatusCode::CREATED, Json(public(&policy))).into_response())
}

async fn update(
    state: &AppState,
    caller: &Caller,
    name: &str,
    mut request: PolicyRequest,
) -> Result<Policy, ApiError> {
    caller.require(UPDATE)?;
    let stored = existing(state, name).await?;
    request.name = Some(stored.name.clone());
    let policy = validated(&request)?;
    if policy.format != stored.format && policy.format != cleanup::ALL_FORMATS {
        let users = users(state).await?;
        let clashing: Vec<&String> = users
            .iter()
            .filter(|(_, format, _, names)| {
                names.contains(&stored.name) && *format != policy.format
            })
            .map(|(name, _, _, _)| name)
            .collect();
        if !clashing.is_empty() {
            return Err(ApiError::invalid(
                "format",
                format!(
                    "the policy is used by repositories of another format: {}",
                    clashing
                        .iter()
                        .map(|name| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }
    cleanup::update(state.db().conn(), &policy)
        .await
        .map_err(policy_error)?;
    state
        .audit(
            caller,
            Event::new(DOMAIN, "updated", &policy.name).with(audit(&policy)),
        )
        .await;
    Ok(policy)
}

async fn update_internal(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<crate::Json<PolicyRequest>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    let crate::Json(request) = body?;
    let policy = update(&state, &caller, &name, request).await?;
    let users = users(&state).await?;
    Ok(Json(policy.document(&cleanup::repositories_using(
        &users,
        &policy.name,
    ))))
}

async fn update_public(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<crate::Json<PolicyRequest>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    let crate::Json(request) = body?;
    let policy = update(&state, &caller, &name, request).await?;
    Ok(Json(public(&policy)))
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(DELETE)?;
    let policy = existing(&state, &name).await?;
    let changed = cleanup::delete(state.db().conn(), &policy.name)
        .await
        .map_err(policy_error)?;
    if !changed.is_empty() {
        state.security().reload().await?;
    }
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "deleted", &policy.name).with(json!({"repositories": changed})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    #[serde(flatten)]
    policy: PolicyRequest,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    page: Option<i64>,
    #[serde(default)]
    size: Option<i64>,
}

#[derive(Default, Deserialize)]
struct Paging {
    page: Option<i64>,
    size: Option<i64>,
}

async fn matcher(
    state: &AppState,
    caller: &Caller,
    mut policy: PolicyRequest,
    repository: Option<&str>,
) -> Result<Matcher, ApiError> {
    caller.require(READ)?;
    if policy
        .name
        .as_deref()
        .is_none_or(|name| name.trim().is_empty())
    {
        policy.name = Some("preview".to_owned());
    }
    let policy = validated(&policy)?;
    let Some(name) = repository.map(str::trim).filter(|name| !name.is_empty()) else {
        return Err(ApiError::invalid(
            "repository",
            "choose a repository to preview",
        ));
    };
    let repository = astrum_repo::store::find(state.db().conn(), name)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| {
            ApiError::invalid("repository", format!("there is no repository named {name}"))
        })?;
    if repository.kind == RepositoryType::Group {
        return Err(ApiError::invalid(
            "repository",
            "a group holds no content of its own; preview one of its members",
        ));
    }
    if !policy.fits(repository.format.id) {
        return Err(ApiError::invalid(
            "repository",
            format!(
                "{} is a {} repository; the policy is for {}",
                repository.name, repository.format.id, policy.format
            ),
        ));
    }
    Ok(Matcher::new(
        &repository.name,
        repository.format.id,
        policy.criteria,
    ))
}

fn time(value: Option<chrono::DateTime<chrono::FixedOffset>>) -> Value {
    value
        .map(|at| {
            at.to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
                .into()
        })
        .unwrap_or(Value::Null)
}

async fn preview(
    State(state): State<AppState>,
    caller: Caller,
    Query(paging): Query<Paging>,
    body: Result<crate::Json<PreviewRequest>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    let crate::Json(request) = body?;
    let matcher = matcher(
        &state,
        &caller,
        request.policy,
        request.repository.as_deref(),
    )
    .await?;
    let size = paging
        .size
        .or(request.size)
        .unwrap_or(PREVIEW_PAGE)
        .clamp(1, PREVIEW_MAX_PAGE);
    let page = paging.page.or(request.page).unwrap_or(0).max(0);
    let db = state.db().conn();
    let found = matcher.page(db, page * size, size).await?;
    let summary = matcher.summary(db, PREVIEW_TOTAL_LIMIT).await?;
    let results: Vec<Value> = found
        .iter()
        .map(|component| {
            json!({
                "id": astrum_content::ids::encode(&matcher.repository, component.id),
                "repository": matcher.repository,
                "format": matcher.format,
                "group": component.namespace,
                "name": component.name,
                "version": component.version,
                "assets": null,
                "assetCount": component.asset_count,
                "size": component.total_size,
                "lastUpdated": time(component.last_updated),
                "lastDownloaded": time(component.last_downloaded),
            })
        })
        .collect();
    Ok(Json(json!({
        "total": if summary.complete { summary.component_count } else { -1 },
        "complete": summary.complete,
        "page": page,
        "size": size,
        "results": results,
    })))
}

async fn summary(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<PreviewRequest>, ApiError>,
) -> Result<Json<query::Summary>, ApiError> {
    let crate::Json(request) = body?;
    let matcher = matcher(
        &state,
        &caller,
        request.policy,
        request.repository.as_deref(),
    )
    .await?;
    Ok(Json(
        matcher
            .summary(state.db().conn(), query::SUMMARY_LIMIT)
            .await?,
    ))
}

async fn csv(
    State(state): State<AppState>,
    caller: Caller,
    Query(params): Query<BTreeMap<String, String>>,
) -> Result<Response, ApiError> {
    let text = |key: &str| {
        params
            .get(key)
            .cloned()
            .filter(|value| !value.trim().is_empty())
    };
    let value = |key: &str| text(key).map(Value::String);
    let policy = PolicyRequest {
        name: text("name"),
        format: text("format"),
        notes: None,
        criteria_last_blob_updated: value("criteriaLastBlobUpdated"),
        criteria_last_downloaded: value("criteriaLastDownloaded"),
        criteria_release_type: value("criteriaReleaseType"),
        criteria_asset_regex: text("criteriaAssetRegex"),
        retain: value("retain"),
        sort_by: value("sortBy"),
    };
    let matcher = matcher(
        &state,
        &caller,
        policy,
        params.get("repository").map(String::as_str),
    )
    .await?;
    let filename = format!(
        "CleanupPreview-{}-{}.csv",
        matcher.repository,
        chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
    );
    let db = state.db().conn().clone();
    let header_sent = false;
    let body = stream::unfold(
        (matcher, db, Some(0i64), header_sent),
        |(matcher, db, after, header_sent)| async move {
            let after = after?;
            let mut chunk = String::new();
            if !header_sent {
                chunk.push_str(query::csv_header());
            }
            match matcher.assets(&db, after, CSV_BATCH).await {
                Ok((rows, next)) => {
                    for row in &rows {
                        chunk.push_str(&query::csv_line(row));
                    }
                    Some((
                        Ok::<_, std::io::Error>(bytes::Bytes::from(chunk)),
                        (matcher, db, next, true),
                    ))
                }
                Err(error) => Some((Err(std::io::Error::other(error)), (matcher, db, None, true))),
            }
        },
    );
    let mut response = Body::from_stream(body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    if let Ok(value) =
        header::HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
    {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

pub async fn names_for(state: &AppState) -> Result<Vec<Policy>, ApiError> {
    Ok(cleanup::list(state.db().conn()).await?)
}

pub fn conflict(message: impl Into<String>) -> Response {
    status(StatusCode::CONFLICT, message)
}
