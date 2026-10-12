use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use astrum_content::format::{SearchTarget, UploadedFile};
use astrum_content::store::{self, Asset, Component};
use astrum_content::{ContentError, Ctx, Format, UploadForm, View, ids};
use astrum_repo::config::Repository;
use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, FixedOffset, SecondsFormat};
use sea_orm::{DbBackend, FromQueryResult, Statement, Value as SqlValue};
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

use crate::routes::content::context;
use crate::routes::repositories::base_url;
use crate::{ApiError, AppState, Caller, REST};

const PAGE: usize = 10;
const BATCH: u64 = 50;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{REST}/v1/components"),
            get(list_components).post(upload),
        )
        .layer(DefaultBodyLimit::disable())
        .route(
            &format!("{REST}/v1/components/{{id}}"),
            get(get_component).delete(delete_component),
        )
        .route(&format!("{REST}/v1/assets"), get(list_assets))
        .route(
            &format!("{REST}/v1/assets/{{id}}"),
            get(get_asset).delete(delete_asset),
        )
        .route(&format!("{REST}/v1/search"), get(search))
        .route(&format!("{REST}/v1/search/assets"), get(search_assets))
        .route(
            &format!("{REST}/v1/search/assets/download"),
            get(search_download),
        )
        .route(
            &format!("{REST}/v1/formats/upload-specs"),
            get(upload_specs),
        )
        .route(
            &format!("{REST}/v1/formats/{{format}}/upload-specs"),
            get(upload_spec),
        )
        .route(
            &format!("{REST}/repository/browse/{{name}}"),
            get(browse_root),
        )
        .route(
            &format!("{REST}/repository/browse/{{name}}/"),
            get(browse_root),
        )
        .route(
            &format!("{REST}/repository/browse/{{name}}/{{*path}}"),
            get(browse),
        )
}

fn time(value: DateTime<FixedOffset>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, false)
}

fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            percent_encoding::utf8_percent_encode(segment, percent_encoding::NON_ALPHANUMERIC)
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("/")
        .replace("%2E", ".")
        .replace("%2D", "-")
        .replace("%5F", "_")
        .replace("%7E", "~")
}

pub fn asset_json(base: &str, asset: &Asset) -> Value {
    let format_attributes = asset
        .attributes
        .get(&asset.format)
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut value = json!({
        "downloadUrl": format!("{base}/repository/{}{}", asset.repository, encode_path(&asset.path)),
        "path": asset.path,
        "id": ids::encode(&asset.repository, asset.id),
        "repository": asset.repository,
        "format": asset.format,
        "checksum": {
            "md5": asset.md5,
            "sha1": asset.sha1,
            "sha256": asset.sha256,
            "sha512": asset.sha512,
        },
        "contentType": asset.content_type,
        "lastModified": time(asset.blob_updated_at),
        "lastDownloaded": asset.last_downloaded_at.map(time),
        "uploader": asset.uploader,
        "uploaderIp": asset.uploader_ip,
        "fileSize": asset.size,
        "blobCreated": time(asset.created_at),
        "blobStoreName": asset.blob_store,
        "blobUpdated": time(asset.blob_updated_at),
        "blobRef": format!("{}@{}", asset.blob_store, asset.blob_id),
        "lastVerified": asset.last_verified_at.map(time),
        "downloadCount": asset.download_count,
        "registryUrl": Value::Null,
    });
    value[asset.format.as_str()] = format_attributes;
    value
}

fn component_json(base: &str, component: &Component, assets: &[Asset]) -> Value {
    json!({
        "id": ids::encode(&component.repository, component.id),
        "repository": component.repository,
        "format": component.format,
        "group": component.namespace,
        "name": component.name,
        "version": component.version,
        "assets": assets.iter().map(|asset| asset_json(base, asset)).collect::<Vec<_>>(),
    })
}

struct Visibility {
    full: BTreeSet<String>,
    selected: BTreeMap<String, Ctx>,
}

impl Visibility {
    fn names(&self) -> Vec<String> {
        self.full
            .iter()
            .chain(self.selected.keys())
            .cloned()
            .collect()
    }

    fn sees(&self, repository: &str, path: &str) -> bool {
        self.full.contains(repository)
            || self
                .selected
                .get(repository)
                .is_some_and(|ctx| ctx.allowed(View::Browse, path) || ctx.allowed(View::Read, path))
    }
}

async fn visibility(
    state: &AppState,
    caller: &Caller,
    only: Option<&str>,
) -> Result<Visibility, ApiError> {
    let repositories = astrum_repo::store::list(state.db().conn())
        .await
        .map_err(ApiError::internal)?;
    let model = state.security().model();
    let mut full = BTreeSet::new();
    let mut selected = BTreeMap::new();
    let Some(subject) = caller.subject() else {
        return Ok(Visibility { full, selected });
    };
    for repository in repositories {
        if only.is_some_and(|only| only != repository.name) {
            continue;
        }
        let format = repository.format.id;
        let name = repository.name.clone();
        if subject.is_permitted(&format!("astrum:repository-view:{format}:{name}:browse"))
            || subject.is_permitted(&format!("astrum:repository-view:{format}:{name}:read"))
        {
            full.insert(name);
            continue;
        }
        let selectable = subject.selectors().iter().any(|grant| {
            (grant.format == "*" || grant.format == format)
                && (grant.repository == "*"
                    || grant.repository.eq_ignore_ascii_case(&name)
                    || grant.repository.strip_prefix("*-") == Some(format))
        });
        if selectable {
            let ctx = state.content().context(
                Arc::new(repository),
                model.clone(),
                Some(subject.clone()),
                "",
                "",
            );
            selected.insert(name, ctx);
        }
    }
    Ok(Visibility { full, selected })
}

#[derive(Default)]
struct Filters {
    clauses: Vec<String>,
    values: Vec<SqlValue>,
}

impl Filters {
    fn bind(&mut self, value: impl Into<SqlValue>) -> String {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }

    fn like(&mut self, column: &str, pattern: &str) {
        let escaped = pattern
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
            .replace('*', "%");
        let parameter = self.bind(escaped);
        self.clauses.push(format!("{column} ILIKE {parameter}"));
    }

    fn equals(&mut self, column: &str, value: &str) {
        let parameter = self.bind(value.to_owned());
        self.clauses.push(format!("{column} = {parameter}"));
    }
}

fn token(offset: u64) -> String {
    URL_SAFE_NO_PAD.encode(format!("offset:{offset}"))
}

fn offset(token: Option<&str>) -> Result<u64, ApiError> {
    let Some(token) = token.filter(|token| !token.is_empty()) else {
        return Ok(0);
    };
    URL_SAFE_NO_PAD
        .decode(token)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|text| text.strip_prefix("offset:").and_then(|n| n.parse().ok()))
        .ok_or_else(|| ApiError::BadRequest("the continuation token is not valid".to_owned()))
}

fn search_filters(
    state: &AppState,
    params: &HashMap<String, String>,
    target: SearchTarget,
    filters: &mut Filters,
) -> Result<(), ApiError> {
    let (prefix, asset_prefix) = match target {
        SearchTarget::Component => ("c", "a"),
        SearchTarget::Asset => ("c", "a"),
    };
    let formats: Vec<Arc<dyn Format>> = state.content().formats();
    for (key, value) in params {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "q" => {
                let escaped = format!(
                    "%{}%",
                    value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_")
                );
                let parameter = filters.bind(escaped);
                filters.clauses.push(format!(
                    "({prefix}.name ILIKE {parameter} OR {prefix}.namespace ILIKE {parameter} OR {prefix}.version ILIKE {parameter} OR EXISTS (SELECT 1 FROM asset q WHERE q.component_id = {prefix}.id AND q.path ILIKE {parameter}))"
                ));
            }
            "format" => filters.equals(&format!("{prefix}.format"), value),
            "group" => filters.like(&format!("{prefix}.namespace"), value),
            "name" => filters.like(&format!("{prefix}.name"), value),
            "version" => filters.like(&format!("{prefix}.version"), value),
            "md5" | "sha1" | "sha256" | "sha512" => {
                let parameter = filters.bind(value.to_ascii_lowercase());
                filters
                    .clauses
                    .push(format!("{asset_prefix}.{key} = {parameter}"));
            }
            "repository" | "repositoryName" | "continuationToken" | "sort" | "direction"
            | "timeout" => {}
            other => {
                let Some(found) = formats
                    .iter()
                    .flat_map(|format| format.search_keys().iter())
                    .find(|search| search.param == other)
                else {
                    continue;
                };
                let path = format!("{{{}}}", found.pointer.join(","));
                let alias = match found.target {
                    SearchTarget::Component => prefix,
                    SearchTarget::Asset => asset_prefix,
                };
                let column = format!("({alias}.attributes #>> '{path}')");
                filters.like(&column, value);
            }
        }
    }
    Ok(())
}

#[derive(Debug, FromQueryResult)]
struct Row {
    component_id: Option<i64>,
    asset_id: i64,
}

async fn find_rows(
    state: &AppState,
    visibility: &Visibility,
    params: &HashMap<String, String>,
    target: SearchTarget,
    start: u64,
) -> Result<(Vec<(Option<Component>, Vec<Asset>)>, Option<String>), ApiError> {
    let names = visibility.names();
    if names.is_empty() {
        return Ok((Vec::new(), None));
    }
    let mut filters = Filters::default();
    let repositories = filters.bind(names);
    filters
        .clauses
        .push(format!("a.repository = ANY({repositories})"));
    search_filters(state, params, target, &mut filters)?;
    let order = match params.get("sort").map(String::as_str) {
        Some("name") => "c.name",
        Some("version") => "c.version",
        Some("group") => "c.namespace",
        Some("repository") => "a.repository",
        _ => "a.id",
    };
    let direction = match params
        .get("direction")
        .map(|value| value.to_ascii_lowercase())
    {
        Some(direction) if direction == "desc" => "DESC",
        _ => "ASC",
    };
    let grouping = match target {
        SearchTarget::Component => "DISTINCT ON (c.id) c.id AS component_id, a.id AS asset_id",
        SearchTarget::Asset => "c.id AS component_id, a.id AS asset_id",
    };
    let joined = match target {
        SearchTarget::Component => "FROM asset a JOIN component c ON c.id = a.component_id",
        SearchTarget::Asset => "FROM asset a LEFT JOIN component c ON c.id = a.component_id",
    };
    let inner_order = match target {
        SearchTarget::Component => "c.id, a.id".to_owned(),
        SearchTarget::Asset => format!("{order} {direction}, a.id"),
    };
    let base = format!(
        "SELECT {grouping} {joined} WHERE {} ORDER BY {inner_order}",
        filters.clauses.join(" AND ")
    );
    let wrapped = match target {
        SearchTarget::Component => format!(
            "SELECT component_id, asset_id FROM ({base}) found ORDER BY component_id {direction}"
        ),
        SearchTarget::Asset => base,
    };
    let mut examined = start;
    let mut results: Vec<(Option<Component>, Vec<Asset>)> = Vec::new();
    loop {
        let sql = format!("{wrapped} OFFSET {examined} LIMIT {BATCH}");
        let rows = Row::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            &sql,
            filters.values.clone(),
        ))
        .all(state.db().conn())
        .await?;
        let fetched = rows.len() as u64;
        for row in rows {
            examined += 1;
            let entry = match target {
                SearchTarget::Component => {
                    let Some(id) = row.component_id else { continue };
                    let Some(component) = store::component_by_id(state.db().conn(), id)
                        .await
                        .map_err(ApiError::internal)?
                    else {
                        continue;
                    };
                    let assets: Vec<Asset> = store::assets_of(state.db().conn(), id)
                        .await
                        .map_err(ApiError::internal)?
                        .into_iter()
                        .filter(|asset| visibility.sees(&asset.repository, &asset.path))
                        .collect();
                    if assets.is_empty() {
                        continue;
                    }
                    (Some(component), assets)
                }
                SearchTarget::Asset => {
                    let Some(asset) = store::by_id(state.db().conn(), row.asset_id)
                        .await
                        .map_err(ApiError::internal)?
                    else {
                        continue;
                    };
                    if !visibility.sees(&asset.repository, &asset.path) {
                        continue;
                    }
                    (None, vec![asset])
                }
            };
            results.push(entry);
            if results.len() == PAGE {
                return Ok((results, Some(token(examined))));
            }
        }
        if fetched < BATCH {
            return Ok((results, None));
        }
    }
}

fn repository_param(params: &HashMap<String, String>) -> Result<String, ApiError> {
    params
        .get("repository")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| ApiError::invalid("repository", "must not be empty"))
}

async fn list_components(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let repository = repository_param(&params)?;
    let visibility = guarded_visibility(&state, &caller, &repository).await?;
    let start = offset(params.get("continuationToken").map(String::as_str))?;
    let only: HashMap<String, String> = HashMap::new();
    let (rows, next) =
        find_rows(&state, &visibility, &only, SearchTarget::Component, start).await?;
    let base = base_url(&headers, &uri);
    Ok(axum::Json(json!({
        "items": rows.iter().filter_map(|(component, assets)| component.as_ref().map(|component| component_json(&base, component, assets))).collect::<Vec<_>>(),
        "continuationToken": next,
    }))
    .into_response())
}

async fn list_assets(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let repository = repository_param(&params)?;
    let visibility = guarded_visibility(&state, &caller, &repository).await?;
    let start = offset(params.get("continuationToken").map(String::as_str))?;
    let (rows, next) = find_rows(
        &state,
        &visibility,
        &HashMap::new(),
        SearchTarget::Asset,
        start,
    )
    .await?;
    let base = base_url(&headers, &uri);
    Ok(axum::Json(json!({
        "items": rows.iter().flat_map(|(_, assets)| assets.iter().map(|asset| asset_json(&base, asset))).collect::<Vec<_>>(),
        "continuationToken": next,
    }))
    .into_response())
}

async fn guarded_visibility(
    state: &AppState,
    caller: &Caller,
    repository: &str,
) -> Result<Visibility, ApiError> {
    let exists = astrum_repo::store::find(state.db().conn(), repository)
        .await
        .map_err(ApiError::internal)?
        .is_some_and(|found| found.name == repository);
    if caller.subject().is_none() {
        return Err(ApiError::Unauthorized);
    }
    let visibility = visibility(state, caller, Some(repository)).await?;
    if !exists || visibility.names().is_empty() {
        return Err(ApiError::not_found(format!(
            "Repository not found: {repository}"
        )));
    }
    Ok(visibility)
}

async fn search(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let visibility = visibility(
        &state,
        &caller,
        params.get("repository").map(String::as_str),
    )
    .await?;
    let start = offset(params.get("continuationToken").map(String::as_str))?;
    let (rows, next) =
        find_rows(&state, &visibility, &params, SearchTarget::Component, start).await?;
    let base = base_url(&headers, &uri);
    Ok(axum::Json(json!({
        "items": rows.iter().filter_map(|(component, assets)| component.as_ref().map(|component| component_json(&base, component, assets))).collect::<Vec<_>>(),
        "continuationToken": next,
    }))
    .into_response())
}

async fn search_assets(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let visibility = visibility(
        &state,
        &caller,
        params.get("repository").map(String::as_str),
    )
    .await?;
    let start = offset(params.get("continuationToken").map(String::as_str))?;
    let (rows, next) = find_rows(&state, &visibility, &params, SearchTarget::Asset, start).await?;
    let base = base_url(&headers, &uri);
    Ok(axum::Json(json!({
        "items": rows.iter().flat_map(|(_, assets)| assets.iter().map(|asset| asset_json(&base, asset))).collect::<Vec<_>>(),
        "continuationToken": next,
    }))
    .into_response())
}

async fn search_download(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let visibility = visibility(
        &state,
        &caller,
        params.get("repository").map(String::as_str),
    )
    .await?;
    let (rows, more) = find_rows(&state, &visibility, &params, SearchTarget::Asset, 0).await?;
    let assets: Vec<&Asset> = rows.iter().flat_map(|(_, assets)| assets.iter()).collect();
    match (assets.as_slice(), more) {
        ([asset], None) => {
            let target = asset_json(&base_url(&headers, &uri), asset)["downloadUrl"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            Ok((StatusCode::FOUND, [(header::LOCATION, target)]).into_response())
        }
        ([], _) if caller.subject().is_none() => Err(ApiError::Unauthorized),
        ([], _) => Err(ApiError::not_found("no asset matches the search")),
        _ => Err(ApiError::BadRequest(
            "the search matches more than one asset; narrow it down".to_owned(),
        )),
    }
}

async fn located(
    state: &AppState,
    caller: &Caller,
    id: &str,
) -> Result<(Visibility, String, i64), ApiError> {
    let missing = || ApiError::not_found("there is nothing with this id");
    let (repository, number) = ids::decode(id).ok_or_else(missing)?;
    if caller.subject().is_none() {
        return Err(ApiError::Unauthorized);
    }
    let visibility = visibility(state, caller, Some(&repository)).await?;
    Ok((visibility, repository, number))
}

async fn get_component(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let (visibility, repository, number) = located(&state, &caller, &id).await?;
    let missing = || ApiError::not_found("Component not found");
    let component = store::component_by_id(state.db().conn(), number)
        .await
        .map_err(ApiError::internal)?
        .filter(|component| component.repository == repository)
        .ok_or_else(missing)?;
    let assets: Vec<Asset> = store::assets_of(state.db().conn(), number)
        .await
        .map_err(ApiError::internal)?
        .into_iter()
        .filter(|asset| visibility.sees(&asset.repository, &asset.path))
        .collect();
    if assets.is_empty() {
        return Err(missing());
    }
    Ok(axum::Json(component_json(
        &base_url(&headers, &uri),
        &component,
        &assets,
    ))
    .into_response())
}

async fn delete_component(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let (visibility, repository, number) = located(&state, &caller, &id).await?;
    let missing = || ApiError::not_found("Component not found");
    let component = store::component_by_id(state.db().conn(), number)
        .await
        .map_err(ApiError::internal)?
        .filter(|component| component.repository == repository)
        .ok_or_else(missing)?;
    let assets = store::assets_of(state.db().conn(), number)
        .await
        .map_err(ApiError::internal)?;
    if !assets
        .iter()
        .any(|asset| visibility.sees(&asset.repository, &asset.path))
    {
        return Err(missing());
    }
    let ctx = delete_context(&state, &caller, &repository).await?;
    for asset in &assets {
        ctx.require(View::Delete, &asset.path).map_err(forbidden)?;
    }
    store::delete_component(state.content(), component.id)
        .await
        .map_err(ApiError::internal)?;
    audit(
        &state,
        &caller,
        "component",
        &format!(
            "{}:{}:{}",
            component.namespace, component.name, component.version
        ),
        &repository,
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_asset(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let (visibility, repository, number) = located(&state, &caller, &id).await?;
    let asset = store::by_id(state.db().conn(), number)
        .await
        .map_err(ApiError::internal)?
        .filter(|asset| {
            asset.repository == repository && visibility.sees(&asset.repository, &asset.path)
        })
        .ok_or_else(|| ApiError::not_found("Asset not found"))?;
    Ok(axum::Json(asset_json(&base_url(&headers, &uri), &asset)).into_response())
}

async fn delete_asset(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let (visibility, repository, number) = located(&state, &caller, &id).await?;
    let asset = store::by_id(state.db().conn(), number)
        .await
        .map_err(ApiError::internal)?
        .filter(|asset| {
            asset.repository == repository && visibility.sees(&asset.repository, &asset.path)
        })
        .ok_or_else(|| ApiError::not_found("Asset not found"))?;
    let ctx = delete_context(&state, &caller, &repository).await?;
    ctx.require(View::Delete, &asset.path).map_err(forbidden)?;
    let path = asset.path.clone();
    store::delete_asset(state.content(), asset)
        .await
        .map_err(ApiError::internal)?;
    audit(&state, &caller, "asset", &path, &repository).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_context(
    state: &AppState,
    caller: &Caller,
    repository: &str,
) -> Result<Ctx, ApiError> {
    let found = state
        .content()
        .repository(repository)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;
    Ok(state
        .content()
        .context(found, state.security().model(), caller.0.clone(), "", ""))
}

fn forbidden(error: ContentError) -> ApiError {
    match error {
        ContentError::Unauthorized => ApiError::Unauthorized,
        _ => ApiError::forbidden(),
    }
}

async fn audit(state: &AppState, caller: &Caller, kind: &str, name: &str, repository: &str) {
    state
        .audit(
            caller,
            astrum_db::audit::Event::new(kind, "deleted", name)
                .with(json!({"repository": repository})),
        )
        .await;
}

async fn upload_specs(State(state): State<AppState>, caller: Caller) -> Result<Response, ApiError> {
    caller.signed_in()?;
    let specs: Vec<Value> = state
        .content()
        .formats()
        .iter()
        .filter_map(|format| format.upload_spec())
        .map(|spec| serde_json::to_value(spec).unwrap_or(Value::Null))
        .collect();
    Ok(axum::Json(specs).into_response())
}

async fn upload_spec(
    State(state): State<AppState>,
    caller: Caller,
    Path(format): Path<String>,
) -> Result<Response, ApiError> {
    caller.signed_in()?;
    let spec = state
        .content()
        .format(&format)
        .and_then(|format| format.upload_spec())
        .ok_or_else(|| {
            ApiError::not_found(format!("No upload specification for the format {format}"))
        })?;
    Ok(axum::Json(spec).into_response())
}

enum Failure {
    Api(ApiError),
    Content(ContentError),
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        match self {
            Failure::Api(error) => error.into_response(),
            Failure::Content(error) => error.into_response(),
        }
    }
}

async fn upload(
    state: State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    params: Query<HashMap<String, String>>,
    multipart: Multipart,
) -> Response {
    match upload_inner(state, caller, headers, uri, params, multipart).await {
        Ok(status) => status.into_response(),
        Err(failure) => failure.into_response(),
    }
}

async fn upload_inner(
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
    mut multipart: Multipart,
) -> Result<StatusCode, Failure> {
    let repository = repository_param(&params).map_err(Failure::Api)?;
    if caller.subject().is_none() {
        return Err(Failure::Api(ApiError::Unauthorized));
    }
    let (ctx, format) = context(&state, caller, &repository, &headers, &uri)
        .await
        .map_err(|error| match error {
            ContentError::NotFound(_) => Failure::Api(ApiError::not_found(format!(
                "Repository not found: {repository}"
            ))),
            other => Failure::Content(other),
        })?;
    if format.upload_spec().is_none() {
        return Err(Failure::Api(ApiError::BadRequest(format!(
            "{} repositories do not take uploads",
            format.id()
        ))));
    }
    let mut form = UploadForm::default();
    let temporary = state.data_dir().join("tmp");
    let _ = tokio::fs::create_dir_all(&temporary).await;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| Failure::Api(ApiError::BadRequest(error.body_text())))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        match field.file_name().map(str::to_owned) {
            Some(filename) => {
                let path = temporary.join(format!("upload-{}", uuid::Uuid::new_v4()));
                let mut file = tokio::fs::File::create(&path)
                    .await
                    .map_err(|error| Failure::Api(ApiError::internal(error)))?;
                let mut size = 0u64;
                let mut field = field;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|error| Failure::Api(ApiError::BadRequest(error.body_text())))?
                {
                    size += chunk.len() as u64;
                    file.write_all(&chunk)
                        .await
                        .map_err(|error| Failure::Api(ApiError::internal(error)))?;
                }
                file.flush()
                    .await
                    .map_err(|error| Failure::Api(ApiError::internal(error)))?;
                form.files.insert(
                    name,
                    UploadedFile {
                        filename: Some(filename),
                        path,
                        size,
                    },
                );
            }
            None => {
                let text = field
                    .text()
                    .await
                    .map_err(|error| Failure::Api(ApiError::BadRequest(error.body_text())))?;
                form.fields.insert(name, text);
            }
        }
    }
    format
        .upload(ctx, form)
        .await
        .map_err(|error| match error {
            ContentError::Unauthorized => Failure::Api(ApiError::Unauthorized),
            ContentError::Forbidden => Failure::Api(ApiError::forbidden()),
            ContentError::BadRequest(message) | ContentError::Conflict(message) => {
                Failure::Api(ApiError::BadRequest(message))
            }
            other => Failure::Content(other),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn browse_root(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Response {
    browse_page(state, caller, name, String::new())
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

async fn browse(
    State(state): State<AppState>,
    caller: Caller,
    Path((name, path)): Path<(String, String)>,
) -> Response {
    browse_page(state, caller, name, path)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

async fn browse_page(
    state: AppState,
    caller: Caller,
    name: String,
    path: String,
) -> Result<Response, ContentError> {
    let repository = state
        .content()
        .repository(&name)
        .await?
        .ok_or_else(|| ContentError::not_found("Repository not found"))?;
    let ctx = state.content().context(
        repository.clone(),
        state.security().model(),
        caller.0.clone(),
        "",
        "",
    );
    let prefix = astrum_content::normalize(&format!("{path}/"))?;
    let readers: Vec<Arc<Repository>> = match repository.kind {
        astrum_repo::RepositoryType::Group => astrum_content::group::members(&ctx).await?,
        _ => vec![repository.clone()],
    };
    let mut children: BTreeMap<String, Option<i64>> = BTreeMap::new();
    let mut any_visible = false;
    for reader in readers {
        for asset in store::under(ctx.db(), &reader.name, &prefix, 20_000).await? {
            if !ctx.allowed(View::Browse, &asset.path) && !ctx.allowed(View::Read, &asset.path) {
                continue;
            }
            any_visible = true;
            let rest = &asset.path[prefix.len()..];
            match rest.split_once('/') {
                Some((directory, _)) => {
                    children.insert(format!("{directory}/"), None);
                }
                None => {
                    children.entry(rest.to_owned()).or_insert(Some(asset.size));
                }
            }
        }
    }
    if !any_visible {
        ctx.require(View::Browse, &prefix)?;
        if prefix != "/" {
            return Err(ContentError::not_found(format!(
                "{prefix} is not in the repository {name}"
            )));
        }
    }
    let mut rows = String::new();
    if prefix != "/" {
        rows.push_str("<tr><td><a href=\"../\">Parent Directory</a></td><td></td></tr>\n");
    }
    for (child, size) in &children {
        let href = encode_path(child);
        let target = match size {
            Some(_) => format!(
                "/repository/{}{}{}",
                escape(&name),
                encode_path(&prefix),
                href
            ),
            None => href,
        };
        let size = size.map(|size| size.to_string()).unwrap_or_default();
        rows.push_str(&format!(
            "<tr><td><a href=\"{target}\">{}</a></td><td>{size}</td></tr>\n",
            escape(child)
        ));
    }
    let title = format!("Index of /{}{}", escape(&name), escape(&prefix));
    Ok(Html(format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>{title}</title></head>\n<body>\n<h1>{title}</h1>\n<table>\n<tr><th>Name</th><th>Size</th></tr>\n{rows}</table>\n</body>\n</html>\n"
    ))
    .into_response())
}
