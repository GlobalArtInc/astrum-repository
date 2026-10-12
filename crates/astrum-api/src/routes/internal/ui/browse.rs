use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use astrum_content::{ContentError, View, ids, store};
use astrum_db::audit::Event;
use astrum_db::entity::{asset, component};
use astrum_repo::{Repository, RepositoryType};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::access::{Reach, Sql, context, like_prefix, reach_of, subject};
use super::versions::natural;
use super::{UI, time};
use crate::routes::components::encode_path;
use crate::routes::repositories::BaseUrl;
use crate::{ApiError, AppState, Caller};

const DEFAULT_LIMIT: usize = 1000;
const MAX_LIMIT: usize = 10_000;
const DELETE_BATCH: u64 = 500;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        &format!("{UI}/browse/{{repository}}"),
        get(tree).delete(remove),
    )
}

#[derive(Deserialize)]
struct BrowseQuery {
    path: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, FromQueryResult)]
struct FolderRow {
    name: String,
    children: i64,
    components: i64,
    component: Option<i64>,
    loose: Option<bool>,
}

#[derive(Clone, Serialize)]
struct Coordinates {
    group: String,
    name: String,
    version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Node {
    kind: &'static str,
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    child_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    asset_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<Coordinates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    download_url: Option<String>,
}

impl Node {
    fn folder(name: String, path: String, children: i64) -> Self {
        Node {
            kind: "folder",
            name,
            path,
            child_count: Some(children),
            asset_id: None,
            component_id: None,
            component: None,
            repository: None,
            size: None,
            content_type: None,
            last_modified: None,
            download_url: None,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Tree {
    repository: String,
    format: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
    path: String,
    truncated: bool,
    children: Vec<Node>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Deleted {
    deleted_assets: u64,
    deleted_components: u64,
    skipped_assets: u64,
}

pub(super) fn content_error(error: ContentError) -> ApiError {
    match error {
        ContentError::NotFound(message) => ApiError::NotFound(message),
        ContentError::BadRequest(message) => ApiError::BadRequest(message),
        ContentError::Unauthorized => ApiError::Unauthorized,
        ContentError::Forbidden => ApiError::forbidden(),
        other => ApiError::internal(other),
    }
}

fn folder(path: Option<&str>) -> Result<String, ApiError> {
    astrum_content::normalize(&format!("{}/", path.unwrap_or_default())).map_err(content_error)
}

fn repository_missing(name: &str) -> ApiError {
    ApiError::not_found(format!("Repository not found: {name}"))
}

fn folder_missing(prefix: &str, name: &str) -> ApiError {
    ApiError::not_found(format!("{prefix} is not in the repository {name}"))
}

async fn located(
    state: &AppState,
    caller: &Caller,
    name: &str,
) -> Result<(astrum_content::Ctx, Reach), ApiError> {
    let subject = subject(caller)?;
    let repository = state
        .content()
        .repository(name)
        .await
        .map_err(content_error)?
        .ok_or_else(|| repository_missing(name))?;
    let ctx = context(state, &subject, repository);
    let reach = reach_of(&ctx);
    if !reach.visible() {
        return Err(repository_missing(name));
    }
    Ok((ctx, reach))
}

async fn tree(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
    Path(name): Path<String>,
    Query(query): Query<BrowseQuery>,
) -> Result<axum::Json<Tree>, ApiError> {
    let (ctx, reach) = located(&state, &caller, &name).await?;
    let prefix = folder(query.path.as_deref())?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let repository = ctx.repository.clone();
    let readers: Vec<Arc<Repository>> = match repository.kind {
        RepositoryType::Group => astrum_content::group::members(&ctx)
            .await
            .map_err(content_error)?,
        _ => vec![repository.clone()],
    };
    let reader_names: Vec<String> = readers.iter().map(|reader| reader.name.clone()).collect();
    let mut children = Vec::new();
    let mut truncated = false;
    if !reader_names.is_empty() {
        let (folders, more_folders) =
            folders(&state, &reach, &reader_names, &prefix, limit).await?;
        let (assets, more_assets) = assets(&state, &reach, &reader_names, &prefix, limit).await?;
        truncated = more_folders || more_assets;
        children = nodes(&state, &base.0, &repository, &prefix, folders, assets).await?;
    }
    if children.is_empty() && prefix != "/" {
        return Err(folder_missing(&prefix, &repository.name));
    }
    Ok(axum::Json(Tree {
        repository: repository.name.clone(),
        format: repository.format.id,
        kind: repository.kind.as_str(),
        path: prefix,
        truncated,
        children,
    }))
}

fn source(sql: &mut Sql, reach: &Reach, readers: &[String], prefix: &str) -> (String, String) {
    let names = sql.bind(readers.to_vec());
    let like = sql.bind(like_prefix(prefix));
    let start = sql.bind((prefix.chars().count() + 1) as i32);
    let condition = sql.reach(reach).unwrap_or_else(|| "FALSE".to_owned());
    let filter = format!("a.repository = ANY({names}) AND a.path LIKE {like} AND {condition}",);
    let ordered = match readers.len() > 1 {
        true => format!(
            "DISTINCT ON (a.path) {{columns}} FROM asset a WHERE {filter} ORDER BY a.path, array_position({names}::text[], a.repository)"
        ),
        false => format!("{{columns}} FROM asset a WHERE {filter}"),
    };
    (ordered, start)
}

async fn folders(
    state: &AppState,
    reach: &Reach,
    readers: &[String],
    prefix: &str,
    limit: usize,
) -> Result<(Vec<FolderRow>, bool), ApiError> {
    let mut sql = Sql::default();
    let (from, start) = source(&mut sql, reach, readers, prefix);
    let inner = format!(
        "SELECT {}",
        from.replace("{columns}", "a.path, a.component_id")
    );
    let query = format!(
        "SELECT split_part(rest, '/', 1) AS name, count(DISTINCT split_part(rest, '/', 2))::bigint AS children, count(DISTINCT component_id)::bigint AS components, min(component_id) AS component, bool_or(component_id IS NULL) AS loose FROM (SELECT substr(s.path, {start}) AS rest, s.component_id FROM ({inner}) s) x WHERE strpos(rest, '/') > 0 GROUP BY 1 ORDER BY 1 LIMIT {}",
        limit + 1
    );
    let mut rows = FolderRow::find_by_statement(sql.statement(&query))
        .all(state.db().conn())
        .await?;
    let more = rows.len() > limit;
    rows.truncate(limit);
    Ok((rows, more))
}

async fn assets(
    state: &AppState,
    reach: &Reach,
    readers: &[String],
    prefix: &str,
    limit: usize,
) -> Result<(Vec<store::Asset>, bool), ApiError> {
    let mut sql = Sql::default();
    let (from, start) = source(&mut sql, reach, readers, prefix);
    let query = format!(
        "SELECT * FROM (SELECT {}) s WHERE strpos(substr(s.path, {start}), '/') = 0 ORDER BY s.path LIMIT {}",
        from.replace("{columns}", "a.*"),
        limit + 1
    );
    let mut rows = asset::Entity::find()
        .from_raw_sql(sql.statement(&query))
        .all(state.db().conn())
        .await?;
    let more = rows.len() > limit;
    rows.truncate(limit);
    Ok((rows, more))
}

async fn nodes(
    state: &AppState,
    base: &str,
    repository: &Repository,
    prefix: &str,
    folders: Vec<FolderRow>,
    assets: Vec<store::Asset>,
) -> Result<Vec<Node>, ApiError> {
    let wanted: BTreeSet<i64> = folders
        .iter()
        .filter(|row| row.components == 1 && row.loose == Some(false))
        .filter_map(|row| row.component)
        .chain(assets.iter().filter_map(|asset| asset.component_id))
        .collect();
    let components: BTreeMap<i64, component::Model> = match wanted.is_empty() {
        true => BTreeMap::new(),
        false => component::Entity::find()
            .filter(component::Column::Id.is_in(wanted))
            .all(state.db().conn())
            .await?
            .into_iter()
            .map(|found| (found.id, found))
            .collect(),
    };
    let coordinates = |found: &component::Model| Coordinates {
        group: found.namespace.clone(),
        name: found.name.clone(),
        version: found.version.clone(),
    };
    let mut branches: Vec<Node> = folders
        .into_iter()
        .map(|row| {
            let path = format!("{prefix}{}/", row.name);
            let single = (row.components == 1 && row.loose == Some(false))
                .then_some(row.component)
                .flatten()
                .and_then(|id| components.get(&id))
                .filter(|found| !found.version.is_empty() && found.version == row.name);
            match single {
                Some(found) => Node {
                    kind: "component",
                    component_id: Some(ids::encode(&found.repository, found.id)),
                    component: Some(coordinates(found)),
                    repository: Some(found.repository.clone()),
                    ..Node::folder(row.name, path, row.children)
                },
                None => Node::folder(row.name, path, row.children),
            }
        })
        .collect();
    branches.sort_by(|a, b| natural(&a.name, &b.name));
    let mut leaves: Vec<Node> = assets
        .into_iter()
        .map(|asset| {
            let found = asset.component_id.and_then(|id| components.get(&id));
            Node {
                kind: "asset",
                name: asset.path[prefix.len()..].to_owned(),
                asset_id: Some(ids::encode(&asset.repository, asset.id)),
                component_id: asset
                    .component_id
                    .map(|id| ids::encode(&asset.repository, id)),
                component: found.map(coordinates),
                repository: Some(asset.repository.clone()),
                size: Some(asset.size),
                content_type: Some(asset.content_type.clone()),
                last_modified: Some(time(asset.blob_updated_at)),
                download_url: Some(format!(
                    "{base}/repository/{}{}",
                    repository.name,
                    encode_path(&asset.path)
                )),
                child_count: None,
                path: asset.path,
            }
        })
        .collect();
    leaves.sort_by(|a, b| match natural(&a.name, &b.name) {
        Ordering::Equal => a.path.cmp(&b.path),
        other => other,
    });
    branches.extend(leaves);
    Ok(branches)
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    Query(query): Query<BrowseQuery>,
) -> Result<axum::Json<Deleted>, ApiError> {
    let (ctx, reach) = located(&state, &caller, &name).await?;
    let repository = ctx.repository.clone();
    if repository.kind == RepositoryType::Group {
        return Err(ApiError::BadRequest(format!(
            "{} is a group and holds no content of its own; delete the content from its member repositories",
            repository.name
        )));
    }
    let prefix = folder(query.path.as_deref())?;
    if prefix == "/" {
        return Err(ApiError::BadRequest(
            "choose a folder; the whole content of a repository is not deleted this way".to_owned(),
        ));
    }
    if !ctx.grant(View::Delete).any() {
        ctx.require(View::Delete, &prefix).map_err(content_error)?;
    }
    let db = state.db().conn();
    let like = like_prefix(&prefix);
    let mut after = String::new();
    let mut visible = 0u64;
    let mut deleted = 0u64;
    let mut skipped = 0u64;
    let mut touched: Vec<(i64, String)> = Vec::new();
    loop {
        let batch = asset::Entity::find()
            .filter(asset::Column::Repository.eq(repository.name.as_str()))
            .filter(asset::Column::Path.like(like.as_str()))
            .filter(asset::Column::Path.gt(after.as_str()))
            .order_by_asc(asset::Column::Path)
            .limit(DELETE_BATCH)
            .all(db)
            .await?;
        let fetched = batch.len() as u64;
        for found in batch {
            after = found.path.clone();
            if !reach.sees(&found.format, &found.path) {
                continue;
            }
            visible += 1;
            if !ctx.allowed(View::Delete, &found.path) {
                skipped += 1;
                continue;
            }
            if let Some(id) = found.component_id
                && !touched.iter().any(|(known, _)| *known == id)
                && let Some(owner) = store::component_by_id(db, id)
                    .await
                    .map_err(content_error)?
            {
                touched.push((
                    id,
                    format!("{}:{}:{}", owner.namespace, owner.name, owner.version),
                ));
            }
            let path = found.path.clone();
            store::delete_asset(state.content(), found)
                .await
                .map_err(content_error)?;
            deleted += 1;
            state
                .audit(
                    &caller,
                    Event::new("asset", "deleted", &path)
                        .with(json!({"repository": repository.name, "folder": prefix})),
                )
                .await;
        }
        if fetched < DELETE_BATCH {
            break;
        }
    }
    if visible == 0 {
        return Err(folder_missing(&prefix, &repository.name));
    }
    if deleted == 0 {
        return Err(ApiError::forbidden());
    }
    let left: BTreeSet<i64> = match touched.is_empty() {
        true => BTreeSet::new(),
        false => component::Entity::find()
            .select_only()
            .column(component::Column::Id)
            .filter(component::Column::Id.is_in(touched.iter().map(|(id, _)| *id)))
            .into_tuple::<i64>()
            .all(db)
            .await?
            .into_iter()
            .collect(),
    };
    let mut components = 0u64;
    for (id, coordinates) in &touched {
        if left.contains(id) {
            continue;
        }
        components += 1;
        state
            .audit(
                &caller,
                Event::new("component", "deleted", coordinates)
                    .with(json!({"repository": repository.name, "folder": prefix})),
            )
            .await;
    }
    Ok(axum::Json(Deleted {
        deleted_assets: deleted,
        deleted_components: components,
        skipped_assets: skipped,
    }))
}
