use axum::Router;
use axum::extract::State;
use axum::routing::get;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use serde::Serialize;

use super::UI;
use crate::{ApiError, AppState, Caller};

const RECENT: i64 = 10_000;
const MAX_VALUES: i64 = 200;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/audit/facets"), get(facets))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Facets {
    domains: Vec<String>,
    types: Vec<String>,
    initiators: Vec<String>,
}

#[derive(FromQueryResult)]
struct Value {
    facet: String,
    value: String,
}

async fn facets(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Facets>, ApiError> {
    caller.require("astrum:audit:read")?;
    let sql = format!(
        "WITH recent AS (SELECT domain, type, initiator FROM audit_event ORDER BY id DESC LIMIT {RECENT}), \
         found AS (\
         SELECT 'domain' AS facet, domain AS value FROM recent \
         UNION SELECT 'type', type FROM recent \
         UNION SELECT 'initiator', initiator FROM recent), \
         ranked AS (SELECT facet, value, row_number() OVER (PARTITION BY facet ORDER BY value) AS position FROM found WHERE value <> '') \
         SELECT facet, value FROM ranked WHERE position <= {MAX_VALUES} ORDER BY facet, value"
    );
    let rows = Value::find_by_statement(Statement::from_string(DbBackend::Postgres, sql))
        .all(state.db().conn())
        .await?;
    let mut facets = Facets::default();
    for row in rows {
        match row.facet.as_str() {
            "domain" => facets.domains.push(row.value),
            "type" => facets.types.push(row.value),
            _ => facets.initiators.push(row.value),
        }
    }
    for values in [
        &mut facets.domains,
        &mut facets.types,
        &mut facets.initiators,
    ] {
        values.sort_by(|left, right| {
            left.to_lowercase()
                .cmp(&right.to_lowercase())
                .then_with(|| left.cmp(right))
        });
    }
    Ok(axum::Json(facets))
}
