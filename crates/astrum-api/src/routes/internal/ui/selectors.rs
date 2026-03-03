use astrum_content::ids;
use astrum_security::Selector;
use axum::Router;
use axum::extract::State;
use axum::routing::post;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::UI;
use super::access::{Seen, Sql, survey};
use crate::{ApiError, AppState, Caller, Json};

const PREVIEW: usize = 100;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/content-selectors/preview"), post(preview))
}

#[derive(Deserialize)]
struct PreviewRequest {
    expression: Option<String>,
    repository: Option<String>,
    format: Option<String>,
}

#[derive(FromQueryResult)]
struct Row {
    id: i64,
    repository: String,
    format: String,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Match {
    repository: String,
    format: String,
    path: String,
    asset_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Preview {
    items: Vec<Match>,
    more: bool,
}

async fn preview(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<PreviewRequest>,
) -> Result<axum::Json<Preview>, ApiError> {
    caller.require("astrum:selectors:read")?;
    let expression = body.expression.unwrap_or_default();
    if expression.trim().is_empty() {
        return Err(ApiError::invalid("expression", "must not be empty"));
    }
    let selector = Selector::parse(&expression)
        .map_err(|error| ApiError::invalid("expression", error.to_string()))?;
    let survey = survey(&state, &caller).await?;
    let repository = body
        .repository
        .map(|repository| repository.trim().to_owned())
        .filter(|repository| !repository.is_empty() && repository != "*");
    let format = body
        .format
        .map(|format| format.trim().to_owned())
        .filter(|format| !format.is_empty() && format != "*");
    if let Some(name) = &repository
        && survey.find(name).is_none_or(|seen| !seen.reach.visible())
    {
        return Err(ApiError::not_found(format!("Repository not found: {name}")));
    }
    let seen: Vec<&Seen> = survey
        .seen
        .iter()
        .filter(|seen| {
            repository
                .as_deref()
                .is_none_or(|name| seen.repository.name == name)
        })
        .filter(|seen| {
            format
                .as_deref()
                .is_none_or(|format| seen.repository.format.id == format)
        })
        .collect();
    let mut sql = Sql::default();
    let Some(condition) = sql.visible(seen) else {
        return Ok(axum::Json(Preview {
            items: Vec::new(),
            more: false,
        }));
    };
    let matching = sql.selectors(std::slice::from_ref(&selector));
    let statement = format!(
        "SELECT a.id, a.repository, a.format, a.path FROM asset a WHERE {condition} AND {matching} ORDER BY a.repository, a.path LIMIT {}",
        PREVIEW + 1
    );
    let mut rows = Row::find_by_statement(sql.statement(&statement))
        .all(state.db().conn())
        .await?;
    let more = rows.len() > PREVIEW;
    rows.truncate(PREVIEW);
    Ok(axum::Json(Preview {
        items: rows
            .into_iter()
            .map(|row| Match {
                asset_id: ids::encode(&row.repository, row.id),
                repository: row.repository,
                format: row.format,
                path: row.path,
            })
            .collect(),
        more,
    }))
}
