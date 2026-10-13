use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use astrum_content::{BrowseLayout, ContentError, View, ids, store};
use astrum_db::audit::Event;
use astrum_db::entity::{asset, component};
use astrum_repo::{Repository, RepositoryType};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::access::{Reach, Sql, context, like_prefix, reach_of, subject};
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
    asset_path: Option<String>,
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
    download_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    download_url: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    generated: bool,
}

impl Node {
    fn folder(name: String, path: String, children: i64) -> Self {
        Node {
            kind: "folder",
            name,
            path,
            child_count: Some(children),
            asset_path: None,
            asset_id: None,
            component_id: None,
            component: None,
            repository: None,
            size: None,
            content_type: None,
            last_modified: None,
            download_count: None,
            download_url: None,
            generated: false,
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

pub(super) fn layout_of(state: &AppState, repository: &Repository) -> BrowseLayout {
    state
        .content()
        .format(repository.format.id)
        .map(|format| format.browse_layout())
        .unwrap_or_default()
}

pub(super) fn tree_path(layout: &BrowseLayout) -> &'static str {
    layout.path.unwrap_or("a.path")
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
    let layout = layout_of(&state, &repository);
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
            folders(&state, &reach, &layout, &reader_names, &prefix, limit).await?;
        let (assets, more_assets) =
            assets(&state, &reach, &layout, &reader_names, &prefix, limit).await?;
        truncated = more_folders || more_assets;
        children = nodes(&state, &base.0, &repository, &prefix, folders, assets).await?;
    }
    if prefix == "/" && reach.visible() {
        for path in layout.generated {
            if !children.iter().any(|node| node.path == *path) {
                children.push(Node {
                    kind: "asset",
                    name: path.trim_start_matches('/').to_owned(),
                    path: (*path).to_owned(),
                    child_count: None,
                    download_url: Some(format!(
                        "{}/repository/{}{}",
                        base.0,
                        repository.name,
                        encode_path(path)
                    )),
                    generated: true,
                    ..Node::folder(String::new(), String::new(), 0)
                });
            }
        }
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

fn source(
    sql: &mut Sql,
    reach: &Reach,
    layout: &BrowseLayout,
    readers: &[String],
    prefix: &str,
) -> (String, String) {
    let names = sql.bind(readers.to_vec());
    let like = sql.bind(like_prefix(prefix));
    let start = sql.bind((prefix.chars().count() + 1) as i32);
    let condition = sql.reach(reach).unwrap_or_else(|| "FALSE".to_owned());
    let path = tree_path(layout);
    let join = match layout.path {
        Some(_) => "LEFT JOIN component c ON c.id = a.component_id",
        None => "",
    };
    let filter = format!("a.repository = ANY({names}) AND ({path}) LIKE {like} AND {condition}");
    let ordered = match readers.len() > 1 {
        true => format!(
            "DISTINCT ON (({path})) {{columns}}, ({path}) AS tree_path FROM asset a {join} WHERE {filter} ORDER BY ({path}), array_position({names}::text[], a.repository)"
        ),
        false => format!("{{columns}}, ({path}) AS tree_path FROM asset a {join} WHERE {filter}"),
    };
    (ordered, start)
}

async fn folders(
    state: &AppState,
    reach: &Reach,
    layout: &BrowseLayout,
    readers: &[String],
    prefix: &str,
    limit: usize,
) -> Result<(Vec<FolderRow>, bool), ApiError> {
    let mut sql = Sql::default();
    let (from, start) = source(&mut sql, reach, layout, readers, prefix);
    let inner = format!("SELECT {}", from.replace("{columns}", "a.component_id"));
    let query = format!(
        "SELECT split_part(rest, '/', 1) AS name, count(DISTINCT split_part(rest, '/', 2))::bigint AS children, count(DISTINCT component_id)::bigint AS components, min(component_id) AS component, bool_or(component_id IS NULL) AS loose FROM (SELECT substr(s.tree_path, {start}) AS rest, s.component_id FROM ({inner}) s) x WHERE strpos(rest, '/') > 0 GROUP BY 1 ORDER BY 1 LIMIT {}",
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
    layout: &BrowseLayout,
    readers: &[String],
    prefix: &str,
    limit: usize,
) -> Result<(Vec<(String, store::Asset)>, bool), ApiError> {
    let mut sql = Sql::default();
    let (from, start) = source(&mut sql, reach, layout, readers, prefix);
    let query = format!(
        "SELECT * FROM (SELECT {}) s WHERE strpos(substr(s.tree_path, {start}), '/') = 0 ORDER BY s.tree_path LIMIT {}",
        from.replace("{columns}", "a.*"),
        limit + 1
    );
    let mut rows: Vec<(String, store::Asset)> = Vec::new();
    for row in state.db().conn().query_all(sql.statement(&query)).await? {
        let place: String = row.try_get("", "tree_path")?;
        rows.push((place, asset::Model::from_query_result(&row, "")?));
    }
    let more = rows.len() > limit;
    rows.truncate(limit);
    Ok((rows, more))
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Component,
    Folder,
    Loose,
}

fn rank(node: &Node) -> Rank {
    match (node.kind, node.component_id.is_some()) {
        ("folder", _) => Rank::Folder,
        (_, true) => Rank::Component,
        _ => Rank::Loose,
    }
}

fn token_order(left: &str, right: &str) -> Ordering {
    let number = |token: &str| !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit());
    match (number(left), number(right)) {
        (true, true) => {
            let a = left.trim_start_matches('0');
            let b = right.trim_start_matches('0');
            a.len()
                .cmp(&b.len())
                .then_with(|| a.cmp(b))
                .then_with(|| left.len().cmp(&right.len()))
        }
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => left.cmp(right),
    }
}

pub(super) fn version_order(left: &str, right: &str) -> Ordering {
    let split = |value: &str| -> Vec<String> {
        value
            .split(['.', '-', '_', '+'])
            .map(str::to_owned)
            .collect()
    };
    let a = split(left);
    let b = split(right);
    for (x, y) in a.iter().zip(b.iter()) {
        let ordering = token_order(x, y);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    a.len().cmp(&b.len()).then_with(|| left.cmp(right))
}

fn tree_order(a: &Node, b: &Node) -> Ordering {
    rank(a).cmp(&rank(b)).then_with(|| match rank(a) {
        Rank::Component => version_order(&a.name, &b.name),
        _ => a
            .name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name)),
    })
}

async fn nodes(
    state: &AppState,
    base: &str,
    repository: &Repository,
    prefix: &str,
    folders: Vec<FolderRow>,
    assets: Vec<(String, store::Asset)>,
) -> Result<Vec<Node>, ApiError> {
    let wanted: BTreeSet<i64> = folders
        .iter()
        .filter(|row| row.components == 1 && row.loose == Some(false))
        .filter_map(|row| row.component)
        .chain(assets.iter().filter_map(|(_, asset)| asset.component_id))
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
    let mut children: Vec<Node> = folders
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
    children.extend(assets.into_iter().map(|(place, asset)| {
        let found = asset.component_id.and_then(|id| components.get(&id));
        Node {
            kind: "asset",
            name: place[prefix.len()..].to_owned(),
            asset_id: Some(ids::encode(&asset.repository, asset.id)),
            component_id: asset
                .component_id
                .map(|id| ids::encode(&asset.repository, id)),
            component: found.map(coordinates),
            repository: Some(asset.repository.clone()),
            size: Some(asset.size),
            content_type: Some(asset.content_type.clone()),
            last_modified: Some(time(asset.blob_updated_at)),
            download_count: Some(asset.download_count),
            download_url: Some(format!(
                "{base}/repository/{}{}",
                repository.name,
                encode_path(&asset.path)
            )),
            child_count: None,
            asset_path: Some(asset.path),
            generated: false,
            path: place,
        }
    }));
    children.sort_by(tree_order);
    Ok(children)
}

fn subtree(
    layout: &BrowseLayout,
    repository: &str,
    prefix: &str,
    after: &str,
) -> sea_orm::Statement {
    let mut sql = Sql::default();
    let name = sql.bind(repository.to_owned());
    let like = sql.bind(like_prefix(prefix));
    let after = sql.bind(after.to_owned());
    let path = tree_path(layout);
    let (join, own) = match layout.path {
        Some(_) => {
            let own = sql.bind(prefix.trim_end_matches('/').to_owned());
            (
                "LEFT JOIN component c ON c.id = a.component_id",
                format!(" OR (({path}) IS NULL AND a.path = {own})"),
            )
        }
        None => ("", String::new()),
    };
    sql.statement(&format!(
        "SELECT a.* FROM asset a {join} WHERE a.repository = {name} AND (({path}) LIKE {like}{own}) AND a.path > {after} ORDER BY a.path LIMIT {DELETE_BATCH}"
    ))
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
    let layout = layout_of(&state, &repository);
    let mut after = String::new();
    let mut visible = 0u64;
    let mut deleted = 0u64;
    let mut skipped = 0u64;
    let mut touched: Vec<(i64, String)> = Vec::new();
    loop {
        let batch = asset::Entity::find()
            .from_raw_sql(subtree(&layout, &repository.name, &prefix, &after))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_put_words_before_numbers_and_compare_numbers_as_numbers() {
        let mut names = vec!["10.0", "latest", "1.0", "0.5", "beta", "2.0"];
        names.sort_by(|a, b| version_order(a, b));
        assert_eq!(names, ["beta", "latest", "0.5", "1.0", "2.0", "10.0"]);
        let mut files = vec![
            "0.txt",
            "m.txt",
            "file9.txt",
            "file10.txt",
            "b.txt",
            "Z.txt",
        ];
        files.sort_by(|a, b| version_order(a, b));
        assert_eq!(
            files,
            [
                "Z.txt",
                "b.txt",
                "file10.txt",
                "file9.txt",
                "m.txt",
                "0.txt"
            ]
        );
        let mut versions = vec!["2.0.0", "1.10.0", "1.9.0", "1.0.0"];
        versions.sort_by(|a, b| version_order(a, b));
        assert_eq!(versions, ["1.0.0", "1.9.0", "1.10.0", "2.0.0"]);
    }

    fn node(kind: &'static str, name: &str, component: bool) -> Node {
        Node {
            kind,
            component_id: component.then(|| "x".to_owned()),
            ..Node::folder(name.to_owned(), format!("/{name}"), 0)
        }
    }

    #[test]
    fn components_come_first_then_folders_then_loose_files() {
        let mut nodes = [
            node("asset", "index.yaml", false),
            node("folder", "Q", false),
            node("folder", "a", false),
            node("asset", "b.txt", true),
            node("component", "1.0.0", true),
            node("folder", "10", false),
            node("folder", "9", false),
        ];
        nodes.sort_by(tree_order);
        let names: Vec<&str> = nodes.iter().map(|node| node.name.as_str()).collect();
        assert_eq!(names, ["b.txt", "1.0.0", "10", "9", "a", "Q", "index.yaml"]);
    }
}
