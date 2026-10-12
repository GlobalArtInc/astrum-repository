use astrum_db::entity::audit_event;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::{DateTime, Duration, FixedOffset, NaiveDate};
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
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
    q: Option<String>,
    initiator: Option<String>,
    #[serde(alias = "type")]
    kind: Option<String>,
    context: Option<String>,
    repository: Option<String>,
    since: Option<String>,
    until: Option<String>,
}

fn given(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

enum Moment {
    Time(DateTime<FixedOffset>),
    Day(DateTime<FixedOffset>),
}

fn moment(id: &str, value: &str, end_of_day: bool) -> Result<Moment, ApiError> {
    if let Ok(time) = DateTime::parse_from_rfc3339(value) {
        return Ok(Moment::Time(time));
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ApiError::invalid(id, "must be a date (2026-10-12) or an RFC 3339 time"))?;
    let day = match end_of_day {
        true => date + Duration::days(1),
        false => date,
    };
    Ok(Moment::Day(
        day.and_hms_opt(0, 0, 0)
            .expect("midnight exists")
            .and_utc()
            .fixed_offset(),
    ))
}

fn contains(pattern: &str) -> String {
    format!(
        "%{}%",
        pattern
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
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
    if let Some(domain) = given(query.domain) {
        select = select.filter(audit_event::Column::Domain.eq(domain));
    }
    if let Some(kind) = given(query.kind) {
        select = select.filter(audit_event::Column::Kind.eq(kind));
    }
    if let Some(initiator) = given(query.initiator) {
        select = select.filter(audit_event::Column::Initiator.eq(initiator));
    }
    if let Some(context) = given(query.context) {
        select = select.filter(audit_event::Column::Context.eq(context));
    }
    if let Some(repository) = given(query.repository) {
        select = select.filter(
            Condition::any()
                .add(
                    Condition::all()
                        .add(audit_event::Column::Domain.eq("repository"))
                        .add(audit_event::Column::Context.eq(repository.clone())),
                )
                .add(Expr::cust_with_values(
                    "(attributes ->> 'repository') = $1",
                    [repository],
                )),
        );
    }
    if let Some(text) = given(query.q) {
        let pattern = contains(&text);
        let mut any = Condition::any();
        for column in [
            audit_event::Column::Domain,
            audit_event::Column::Kind,
            audit_event::Column::Context,
            audit_event::Column::Initiator,
        ] {
            any = any.add(Expr::expr(Func::lower(Expr::col(column))).like(pattern.as_str()));
        }
        select = select.filter(any);
    }
    if let Some(since) = given(query.since) {
        let (Moment::Time(start) | Moment::Day(start)) = moment("since", &since, false)?;
        select = select.filter(audit_event::Column::At.gte(start));
    }
    if let Some(until) = given(query.until) {
        select = match moment("until", &until, true)? {
            Moment::Time(end) => select.filter(audit_event::Column::At.lte(end)),
            Moment::Day(end) => select.filter(audit_event::Column::At.lt(end)),
        };
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
