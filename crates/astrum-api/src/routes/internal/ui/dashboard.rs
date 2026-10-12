use std::collections::{BTreeMap, BTreeSet};

use astrum_content::ids;
use astrum_db::entity::{asset, component};
use astrum_repo::RepositoryType;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use chrono::{DateTime, FixedOffset};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter};
use serde::Serialize;

use super::access::{Seen, Sql, survey};
use super::{UI, time};
use crate::routes::blobstores::measure_all;
use crate::{ApiError, AppState, Caller};

const TOP: usize = 10;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/dashboard"), get(dashboard))
}

#[derive(Default, Serialize)]
struct TypeCount {
    hosted: u64,
    proxy: u64,
    group: u64,
}

impl TypeCount {
    fn add(&mut self, kind: RepositoryType) {
        match kind {
            RepositoryType::Hosted => self.hosted += 1,
            RepositoryType::Proxy => self.proxy += 1,
            RepositoryType::Group => self.group += 1,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FormatCount {
    format: String,
    total: u64,
    #[serde(flatten)]
    types: TypeCount,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryTotals {
    total: u64,
    by_type: TypeCount,
    by_format: Vec<FormatCount>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopAsset {
    repository: String,
    format: String,
    path: String,
    asset_id: String,
    component_id: Option<String>,
    downloads: i64,
    last_downloaded: Option<String>,
}

#[derive(Serialize, FromQueryResult)]
#[serde(rename_all = "camelCase")]
struct TopComponentRow {
    id: i64,
    repository: String,
    format: String,
    namespace: String,
    name: String,
    version: String,
    downloads: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TopComponent {
    repository: String,
    format: String,
    group: String,
    name: String,
    version: String,
    component_id: String,
    downloads: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Downloads {
    total: i64,
    top_assets: Vec<TopAsset>,
    top_components: Vec<TopComponent>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Upload {
    kind: &'static str,
    repository: String,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    path: String,
    asset_id: String,
    asset_count: i64,
    uploaded_at: String,
    uploader: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StoreUsage {
    name: String,
    #[serde(rename = "type")]
    kind: &'static str,
    unavailable: bool,
    blob_count: u64,
    total_size: u64,
    available_space: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Dashboard {
    repositories: RepositoryTotals,
    components: i64,
    assets: i64,
    total_size: i64,
    downloads: Downloads,
    recent_uploads: Vec<Upload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blob_stores: Option<Vec<StoreUsage>>,
}

#[derive(Debug, Default, FromQueryResult)]
struct Totals {
    assets: i64,
    components: i64,
    bytes: i64,
    downloads: i64,
}

#[derive(Debug, FromQueryResult)]
struct RecentRow {
    component_id: Option<i64>,
    asset_id: i64,
    at: DateTime<FixedOffset>,
    assets: i64,
}

async fn dashboard(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Dashboard>, ApiError> {
    let survey = survey(&state, &caller).await?;
    let db = state.db().conn();
    let mut by_type = TypeCount::default();
    let mut by_format: BTreeMap<String, TypeCount> = BTreeMap::new();
    for seen in &survey.seen {
        by_type.add(seen.repository.kind);
        by_format
            .entry(seen.repository.format.id.to_owned())
            .or_default()
            .add(seen.repository.kind);
    }
    let repositories = RepositoryTotals {
        total: survey.seen.len() as u64,
        by_type,
        by_format: by_format
            .into_iter()
            .map(|(format, types)| FormatCount {
                format,
                total: types.hosted + types.proxy + types.group,
                types,
            })
            .collect(),
    };

    let mut totals = Totals::default();
    let mut top_assets = Vec::new();
    let mut top_components = Vec::new();
    let mut recent_uploads = Vec::new();

    let mut sql = Sql::default();
    if let Some(condition) = sql.visible(&survey.seen) {
        let statement = format!(
            "SELECT count(*)::bigint AS assets, count(DISTINCT a.component_id)::bigint AS components, coalesce(sum(a.size), 0)::bigint AS bytes, coalesce(sum(a.download_count), 0)::bigint AS downloads FROM asset a WHERE {condition}"
        );
        if let Some(found) = Totals::find_by_statement(sql.statement(&statement))
            .one(db)
            .await?
        {
            totals = found;
        }
        let statement = format!(
            "SELECT a.* FROM asset a WHERE {condition} AND a.download_count > 0 ORDER BY a.download_count DESC, a.id LIMIT {TOP}"
        );
        top_assets = asset::Entity::find()
            .from_raw_sql(sql.statement(&statement))
            .all(db)
            .await?
            .into_iter()
            .map(|found| TopAsset {
                asset_id: ids::encode(&found.repository, found.id),
                component_id: found
                    .component_id
                    .map(|id| ids::encode(&found.repository, id)),
                downloads: found.download_count,
                last_downloaded: found.last_downloaded_at.map(time),
                repository: found.repository,
                format: found.format,
                path: found.path,
            })
            .collect();
        let statement = format!(
            "SELECT c.id, c.repository, c.format, c.namespace, c.name, c.version, sum(a.download_count)::bigint AS downloads FROM asset a JOIN component c ON c.id = a.component_id WHERE {condition} GROUP BY c.id HAVING sum(a.download_count) > 0 ORDER BY downloads DESC, c.id LIMIT {TOP}"
        );
        top_components = TopComponentRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?
            .into_iter()
            .map(|row| TopComponent {
                component_id: ids::encode(&row.repository, row.id),
                repository: row.repository,
                format: row.format,
                group: row.namespace,
                name: row.name,
                version: row.version,
                downloads: row.downloads,
            })
            .collect();
    }

    let hosted: Vec<&Seen> = survey
        .seen
        .iter()
        .filter(|seen| seen.repository.kind == RepositoryType::Hosted)
        .collect();
    let mut sql = Sql::default();
    if let Some(condition) = sql.visible(hosted) {
        let statement = format!(
            "SELECT a.component_id, (array_agg(a.id ORDER BY a.blob_updated_at DESC, a.id DESC))[1] AS asset_id, max(a.blob_updated_at) AS at, count(*)::bigint AS assets FROM asset a WHERE {condition} GROUP BY a.component_id, CASE WHEN a.component_id IS NULL THEN a.id END ORDER BY at DESC, asset_id DESC LIMIT {TOP}"
        );
        let rows = RecentRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?;
        recent_uploads = recent(&state, rows).await?;
    }

    let blob_stores = match survey.subject.is_permitted("astrum:blobstores:read") {
        true => Some(
            measure_all(&state)
                .await?
                .into_iter()
                .map(|(config, measured)| StoreUsage {
                    kind: config.kind().as_str(),
                    name: config.name,
                    unavailable: measured.unavailable,
                    blob_count: measured.totals.blob_count,
                    total_size: measured.totals.total_bytes,
                    available_space: measured.available,
                })
                .collect(),
        ),
        false => None,
    };

    Ok(axum::Json(Dashboard {
        repositories,
        components: totals.components,
        assets: totals.assets,
        total_size: totals.bytes,
        downloads: Downloads {
            total: totals.downloads,
            top_assets,
            top_components,
        },
        recent_uploads,
        blob_stores,
    }))
}

async fn recent(state: &AppState, rows: Vec<RecentRow>) -> Result<Vec<Upload>, ApiError> {
    let db = state.db().conn();
    let asset_ids: BTreeSet<i64> = rows.iter().map(|row| row.asset_id).collect();
    let component_ids: BTreeSet<i64> = rows.iter().filter_map(|row| row.component_id).collect();
    let assets: BTreeMap<i64, asset::Model> = match asset_ids.is_empty() {
        true => BTreeMap::new(),
        false => asset::Entity::find()
            .filter(asset::Column::Id.is_in(asset_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|found| (found.id, found))
            .collect(),
    };
    let components: BTreeMap<i64, component::Model> = match component_ids.is_empty() {
        true => BTreeMap::new(),
        false => component::Entity::find()
            .filter(component::Column::Id.is_in(component_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|found| (found.id, found))
            .collect(),
    };
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let found = assets.get(&row.asset_id)?;
            let owner = row.component_id.and_then(|id| components.get(&id));
            Some(Upload {
                kind: if owner.is_some() {
                    "component"
                } else {
                    "asset"
                },
                repository: found.repository.clone(),
                format: found.format.clone(),
                component_id: owner.map(|owner| ids::encode(&owner.repository, owner.id)),
                group: owner.map(|owner| owner.namespace.clone()),
                name: owner
                    .map(|owner| owner.name.clone())
                    .unwrap_or_else(|| found.path.clone()),
                version: owner.map(|owner| owner.version.clone()),
                path: found.path.clone(),
                asset_id: ids::encode(&found.repository, found.id),
                asset_count: row.assets,
                uploaded_at: time(row.at),
                uploader: found.uploader.clone(),
            })
        })
        .collect())
}
