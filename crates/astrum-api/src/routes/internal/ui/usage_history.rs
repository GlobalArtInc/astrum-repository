use std::collections::BTreeMap;

use astrum_db::entity::repository_usage_day;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use super::UI;
use super::access::survey;
use crate::routes::repositories::{history_days, history_from};
use crate::{ApiError, AppState, Caller};

const DEFAULT_DAYS: i64 = 30;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/usage-history"), get(history))
}

#[derive(Deserialize)]
struct HistoryQuery {
    days: Option<String>,
}

async fn history(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<HistoryQuery>,
) -> Result<axum::Json<BTreeMap<String, Vec<i64>>>, ApiError> {
    let survey = survey(&state, &caller).await?;
    let days = history_days(query.days.as_deref(), DEFAULT_DAYS)?;
    let names: Vec<String> = survey
        .seen
        .iter()
        .filter(|seen| seen.reach.visible())
        .map(|seen| seen.repository.name.clone())
        .collect();
    let mut series: BTreeMap<String, Vec<i64>> = names
        .iter()
        .map(|name| (name.clone(), Vec::new()))
        .collect();
    if !names.is_empty() {
        let rows = repository_usage_day::Entity::find()
            .filter(repository_usage_day::Column::Repository.is_in(names))
            .filter(repository_usage_day::Column::Day.gte(history_from(days)))
            .order_by_asc(repository_usage_day::Column::Day)
            .all(state.db().conn())
            .await?;
        for row in rows {
            series.entry(row.repository).or_default().push(row.bytes);
        }
    }
    Ok(axum::Json(series))
}
