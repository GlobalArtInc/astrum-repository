use astrum_db::entity::audit_event;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use serde_json::{Value, json};

use super::INTERNAL;
use crate::{ApiError, AppState, Caller};

const DEFAULT_LIMIT: u64 = 100;
const MAX_LIMIT: u64 = 1000;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{INTERNAL}/audit"), get(list))
}

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<u64>,
    before: Option<i64>,
    domain: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<AuditQuery>,
) -> Result<axum::Json<Vec<Value>>, ApiError> {
    caller.require("astrum:audit:read")?;
    let mut select = audit_event::Entity::find().order_by_desc(audit_event::Column::Id);
    if let Some(before) = query.before {
        select = select.filter(audit_event::Column::Id.lt(before));
    }
    if let Some(domain) = query.domain.filter(|domain| !domain.is_empty()) {
        select = select.filter(audit_event::Column::Domain.eq(domain));
    }
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let rows = select.limit(limit).all(state.db().conn()).await?;
    Ok(axum::Json(
        rows.into_iter()
            .map(|row| {
                json!({
                    "id": row.id,
                    "at": row.at.to_rfc3339(),
                    "domain": row.domain,
                    "type": row.kind,
                    "context": row.context,
                    "initiator": row.initiator,
                    "attributes": row.attributes,
                })
            })
            .collect(),
    ))
}
