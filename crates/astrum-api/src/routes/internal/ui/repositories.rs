use std::collections::{BTreeMap, BTreeSet};

use astrum_repo::RepositoryType;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::Serialize;

use super::access::{Flags, Sql, survey};
use super::{UI, time};
use crate::routes::repositories::BaseUrl;
use crate::{ApiError, AppState, Caller};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/repositories"), get(list))
        .route(&format!("{UI}/repositories/recipes"), get(recipes))
}

#[derive(Debug, FromQueryResult)]
struct Stats {
    repository: String,
    assets: i64,
    components: i64,
    bytes: i64,
    last: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryXo {
    name: String,
    format: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
    url: String,
    online: bool,
    blob_store_name: Option<String>,
    remote_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alpine_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    members: Option<Vec<String>>,
    member_of: Vec<String>,
    component_count: Option<i64>,
    asset_count: Option<i64>,
    total_size: Option<i64>,
    last_uploaded: Option<String>,
    limited: bool,
    permissions: Flags,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Recipe {
    id: String,
    format: &'static str,
    api_format: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
) -> Result<axum::Json<Vec<RepositoryXo>>, ApiError> {
    let survey = survey(&state, &caller).await?;
    let visible: BTreeSet<&str> = survey.names();
    let mut stats: BTreeMap<String, Stats> = BTreeMap::new();
    let mut sql = Sql::default();
    if let Some(condition) = sql.visible(&survey.seen) {
        let query = format!(
            "SELECT a.repository, count(*)::bigint AS assets, count(DISTINCT a.component_id)::bigint AS components, coalesce(sum(a.size), 0)::bigint AS bytes, max(a.blob_updated_at) AS last FROM asset a WHERE {condition} GROUP BY a.repository"
        );
        for row in Stats::find_by_statement(sql.statement(&query))
            .all(state.db().conn())
            .await?
        {
            stats.insert(row.repository.clone(), row);
        }
    }
    let mut member_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for group in &survey.all {
        let Some(members) = group.group() else {
            continue;
        };
        if !visible.contains(group.name.as_str()) {
            continue;
        }
        for member in members.members() {
            member_of
                .entry(member.to_lowercase())
                .or_default()
                .push(group.name.clone());
        }
    }
    let items = survey
        .seen
        .iter()
        .map(|seen| {
            let repository = &seen.repository;
            let counted = seen.reach.visible();
            let found = stats.get(&repository.name);
            let number = |pick: fn(&Stats) -> i64| match counted {
                true => Some(found.map(pick).unwrap_or(0)),
                false => None,
            };
            RepositoryXo {
                name: repository.name.clone(),
                format: repository.format.id,
                kind: repository.kind.as_str(),
                url: repository.url(&base.0),
                online: repository.online,
                blob_store_name: repository.blob_store().map(str::to_owned),
                remote_url: repository
                    .attributes
                    .proxy
                    .as_ref()
                    .and_then(|proxy| proxy.remote_url.clone()),
                distribution: repository
                    .attributes
                    .apt
                    .as_ref()
                    .and_then(|apt| apt.distribution.clone()),
                alpine_key: repository
                    .attributes
                    .alpine_signing
                    .as_ref()
                    .filter(|signing| signing.keypair.is_some())
                    .map(|_| format!("{}.rsa.pub", repository.name)),
                members: repository.group().map(|group| {
                    group
                        .members()
                        .iter()
                        .filter_map(|member| {
                            survey
                                .seen
                                .iter()
                                .find(|seen| seen.repository.is_named(member))
                                .map(|seen| seen.repository.name.clone())
                        })
                        .collect()
                }),
                member_of: member_of
                    .get(&repository.name.to_lowercase())
                    .cloned()
                    .unwrap_or_default(),
                component_count: number(|stats| stats.components),
                asset_count: number(|stats| stats.assets),
                total_size: number(|stats| stats.bytes),
                last_uploaded: found.and_then(|stats| stats.last).map(time),
                limited: seen.limited,
                permissions: seen.flags,
            }
        })
        .collect();
    Ok(axum::Json(items))
}

async fn recipes(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<Recipe>>, ApiError> {
    caller.signed_in()?;
    let mut recipes = Vec::new();
    for served in state.content().formats() {
        let Some(format) = astrum_repo::format(served.id()) else {
            continue;
        };
        for kind in RepositoryType::ALL {
            if format.supports(kind) {
                recipes.push(Recipe {
                    id: format!("{}-{}", format.id, kind.as_str()),
                    format: format.id,
                    api_format: format.api_name,
                    kind: kind.as_str(),
                });
            }
        }
    }
    Ok(axum::Json(recipes))
}
