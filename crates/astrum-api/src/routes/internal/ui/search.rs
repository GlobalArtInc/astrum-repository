use std::collections::BTreeMap;

use astrum_content::ids;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::access::{Sql, survey};
use super::versions::compare;
use super::{UI, time};
use crate::{ApiError, AppState, Caller, Validation};

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/search/versions"), get(versions))
}

#[derive(Deserialize)]
struct VersionsQuery {
    format: Option<String>,
    group: Option<String>,
    name: Option<String>,
    repository: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct Row {
    id: i64,
    repository: String,
    version: String,
    last: Option<DateTime<FixedOffset>>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Version {
    version: String,
    repositories: Vec<String>,
    component_ids: Vec<String>,
    last_modified: Option<String>,
    #[serde(skip)]
    latest: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Versions {
    format: String,
    group: String,
    name: String,
    versions: Vec<Version>,
}

async fn versions(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<VersionsQuery>,
) -> Result<axum::Json<Versions>, ApiError> {
    let mut validation = Validation::new();
    validation.require("format", query.format.as_deref());
    validation.require("name", query.name.as_deref());
    validation.finish()?;
    let given = query.format.unwrap_or_default();
    let format = astrum_repo::format(&given)
        .or_else(|| astrum_repo::format_by_api_name(&given))
        .map(|format| format.id.to_owned())
        .unwrap_or(given);
    let group = query.group.unwrap_or_default();
    let name = query.name.unwrap_or_default();
    let survey = survey(&state, &caller).await?;
    let wanted = query.repository.filter(|repository| !repository.is_empty());
    let seen = survey.seen.iter().filter(|seen| {
        wanted
            .as_deref()
            .is_none_or(|wanted| seen.repository.name == wanted)
    });
    let mut sql = Sql::default();
    let mut found: BTreeMap<String, Version> = BTreeMap::new();
    if let Some(condition) = sql.visible(seen) {
        let format_value = sql.bind(format.clone());
        let group_value = sql.bind(group.clone());
        let name_value = sql.bind(name.clone());
        let statement = format!(
            "SELECT c.id, c.repository, c.version, max(a.blob_updated_at) AS last FROM component c JOIN asset a ON a.component_id = c.id WHERE c.format = {format_value} AND c.namespace = {group_value} AND c.name = {name_value} AND {condition} GROUP BY c.id, c.repository, c.version ORDER BY c.repository"
        );
        for row in Row::find_by_statement(sql.statement(&statement))
            .all(state.db().conn())
            .await?
        {
            let entry = found.entry(row.version.clone()).or_default();
            entry.version = row.version;
            entry.repositories.push(row.repository.clone());
            entry
                .component_ids
                .push(ids::encode(&row.repository, row.id));
            entry.latest = entry.latest.max(row.last);
        }
    }
    let mut versions: Vec<Version> = found
        .into_values()
        .map(|mut version| {
            version.last_modified = version.latest.map(time);
            version
        })
        .collect();
    versions.sort_by(|a, b| compare(&b.version, &a.version));
    Ok(axum::Json(Versions {
        format,
        group,
        name,
        versions,
    }))
}
