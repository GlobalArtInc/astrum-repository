use astrum_db::entity::audit_event;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::DateTime;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ColumnTrait, Condition, DbBackend, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Statement,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value as Json, json};

use super::{UI, time};
use crate::{ApiError, AppState, Caller};

const RECENT: i64 = 10_000;
const MAX_VALUES: i64 = 200;
const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 250;
const SESSION: &str = "security.session";
const SECURITY: &[&str] = &[
    "security.user",
    "security.role",
    "security.privilege",
    "security.ldap",
    "security.realm",
    "security.anonymous",
    "security.jwt",
    SESSION,
];
const REPOSITORY: &[&str] = &[
    "repository",
    "repository.asset",
    "repository.component",
    "asset",
    "component",
    "blobstore",
];
const PROTECTION: &[&str] = &["firewall", "firewall.quarantine", "malware"];
const SIGN_IN: &[&str] = &["login", "logout"];

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{UI}/audit/facets"), get(facets))
        .route(&format!("{UI}/audit-log"), get(log))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogQuery {
    page: Option<u64>,
    limit: Option<u64>,
    start_date: Option<String>,
    end_date: Option<String>,
    categories: Option<String>,
    types: Option<String>,
    repository: Option<String>,
    initiator: Option<String>,
    q: Option<String>,
}

fn list(value: &Option<String>) -> Vec<String> {
    value
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn given(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn contains(column: audit_event::Column, text: &str) -> sea_orm::sea_query::SimpleExpr {
    let pattern = format!(
        "%{}%",
        text.to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    Expr::expr(Func::lower(Expr::col(column))).like(pattern)
}

fn sign_in() -> Condition {
    Condition::any().add(audit_event::Column::Kind.is_in(SIGN_IN.iter().copied()))
}

fn session(kind: &str) -> Condition {
    Condition::all()
        .add(audit_event::Column::Domain.eq(SESSION))
        .add(audit_event::Column::Kind.eq(kind))
}

fn category(name: &str) -> Option<Condition> {
    let known = SECURITY
        .iter()
        .chain(REPOSITORY)
        .chain(PROTECTION)
        .copied()
        .collect::<Vec<_>>();
    match name {
        "security" => Some(
            Condition::any()
                .add(audit_event::Column::Domain.is_in(SECURITY.iter().copied()))
                .add(sign_in()),
        ),
        "repository" => Some(
            Condition::all()
                .add(audit_event::Column::Domain.is_in(REPOSITORY.iter().copied()))
                .add(sign_in().not()),
        ),
        "protection" => Some(
            Condition::all()
                .add(audit_event::Column::Domain.is_in(PROTECTION.iter().copied()))
                .add(sign_in().not()),
        ),
        "configuration" => Some(
            Condition::all()
                .add(audit_event::Column::Domain.is_not_in(known))
                .add(sign_in().not()),
        ),
        _ => None,
    }
}

fn kind(name: &str) -> Condition {
    let plain = |kinds: &[&str]| {
        Condition::all()
            .add(audit_event::Column::Kind.is_in(kinds.iter().copied()))
            .add(audit_event::Column::Domain.ne(SESSION))
    };
    match name {
        "updated" => plain(&["updated", "changed"]),
        "finished" => plain(&["finished", "completed"]),
        "login" => Condition::any()
            .add(audit_event::Column::Kind.eq("login"))
            .add(session("created")),
        "logout" => Condition::any()
            .add(audit_event::Column::Kind.eq("logout"))
            .add(session("deleted")),
        other => {
            let any = Condition::any().add(plain(&[other]));
            match other {
                "created" | "deleted" => any,
                _ => any.add(session(other)),
            }
        }
    }
}

fn moment(id: &str, value: &str) -> Result<DateTime<chrono::FixedOffset>, ApiError> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| ApiError::invalid(id, "must be an RFC 3339 time"))
}

async fn log(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<LogQuery>,
) -> Result<axum::Json<Json>, ApiError> {
    caller.require("astrum:audit:read")?;
    let mut filter = Condition::all();
    let categories = list(&query.categories);
    if !categories.is_empty() {
        let mut any = Condition::any();
        for name in &categories {
            any = any
                .add(category(name).unwrap_or_else(|| Condition::all().add(Expr::cust("FALSE"))));
        }
        filter = filter.add(any);
    }
    let types = list(&query.types);
    if !types.is_empty() {
        let mut any = Condition::any();
        for name in &types {
            any = any.add(kind(name));
        }
        filter = filter.add(any);
    }
    if let Some(repository) = given(&query.repository) {
        filter = filter.add(
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
    if let Some(initiator) = given(&query.initiator) {
        filter = filter.add(contains(audit_event::Column::Initiator, &initiator));
    }
    if let Some(text) = given(&query.q) {
        filter = filter.add(contains(audit_event::Column::Context, &text));
    }
    if let Some(start) = given(&query.start_date) {
        filter = filter.add(audit_event::Column::At.gte(moment("startDate", &start)?));
    }
    if let Some(end) = given(&query.end_date) {
        filter = filter.add(audit_event::Column::At.lte(moment("endDate", &end)?));
    }
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let page = query.page.unwrap_or(1).max(1);
    let select = audit_event::Entity::find().filter(filter);
    let total = select.clone().count(state.db().conn()).await?;
    let rows = select
        .order_by_desc(audit_event::Column::Id)
        .offset((page - 1) * limit)
        .limit(limit)
        .all(state.db().conn())
        .await?;
    let items: Vec<Json> = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.id,
                "domain": row.domain,
                "type": row.kind,
                "context": row.context,
                "timestamp": time(row.at),
                "initiator": row.initiator,
                "attributes": row.attributes,
            })
        })
        .collect();
    Ok(axum::Json(json!({
        "items": items,
        "pagination": {
            "totalItems": total,
            "totalPages": total.div_ceil(limit),
            "currentPage": page,
            "itemsPerPage": limit,
        },
    })))
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
