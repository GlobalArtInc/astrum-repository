use astrum_security::customers::{CustomerError, UsageDay, UsageExportRow};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{READ, existing, slug_problem};
use crate::{ApiError, AppState, Caller};

const DEFAULT_DAYS: i64 = 30;
const MAX_DAYS: i64 = 730;
pub(crate) const MAX_ROWS: usize = 200_000;
const HEADER: &str = "customer,token,tokenHint,repository,day,downloads,bytes\r\n";

#[derive(Deserialize)]
pub(super) struct DaysQuery {
    days: Option<i64>,
}

#[derive(Deserialize)]
pub(super) struct ExportQuery {
    days: Option<i64>,
    customer: Option<String>,
}

#[derive(Serialize)]
struct Totals {
    downloads: i64,
    bytes: i64,
}

#[derive(Serialize)]
struct DayView {
    day: String,
    repository: String,
    downloads: i64,
    bytes: i64,
}

fn days_of(days: Option<i64>) -> Result<i64, ApiError> {
    let days = days.unwrap_or(DEFAULT_DAYS);
    match (1..=MAX_DAYS).contains(&days) {
        true => Ok(days),
        false => Err(ApiError::invalid(
            "days",
            format!("must be from 1 to {MAX_DAYS}"),
        )),
    }
}

fn too_many(error: CustomerError, one_customer: bool) -> ApiError {
    match error {
        CustomerError::TooManyRows(_) => ApiError::BadRequest(
            match one_customer {
                true => "narrow the period",
                false => "narrow the period or choose one customer",
            }
            .to_owned(),
        ),
        CustomerError::Internal(error) => error.into(),
        CustomerError::Exists => ApiError::internal(anyhow::anyhow!(error)),
    }
}

pub(super) async fn daily(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Query(query): Query<DaysQuery>,
) -> Result<axum::Json<serde_json::Value>, ApiError> {
    caller.require(READ)?;
    let days = days_of(query.days)?;
    existing(&state, &id).await?;
    let rows: Vec<UsageDay> = state
        .security()
        .customer_usage(&id, days, MAX_ROWS)
        .await
        .map_err(|error| too_many(error, true))?;
    let totals = rows.iter().fold(
        Totals {
            downloads: 0,
            bytes: 0,
        },
        |totals, row| Totals {
            downloads: totals.downloads.saturating_add(row.downloads),
            bytes: totals.bytes.saturating_add(row.bytes),
        },
    );
    let daily: Vec<DayView> = rows
        .into_iter()
        .map(|row| DayView {
            day: row.day.to_string(),
            repository: row.repository,
            downloads: row.downloads,
            bytes: row.bytes,
        })
        .collect();
    Ok(axum::Json(json!({
        "customer": id,
        "days": days,
        "totals": totals,
        "daily": daily,
    })))
}

pub(super) async fn csv(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<ExportQuery>,
) -> Result<Response, ApiError> {
    caller.require(READ)?;
    let days = days_of(query.days)?;
    if let Some(problem) = query.customer.as_deref().and_then(slug_problem) {
        return Err(ApiError::invalid("customer", problem));
    }
    if let Some(id) = query.customer.as_deref() {
        existing(&state, id).await?;
    }
    let rows = state
        .security()
        .usage_export(days, query.customer.as_deref(), MAX_ROWS)
        .await
        .map_err(|error| too_many(error, query.customer.is_some()))?;
    let mut response = render(&rows).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"customer-usage.csv\""),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}

pub(crate) fn render(rows: &[UsageExportRow]) -> String {
    let mut out = String::from(HEADER);
    for row in rows {
        let cells = [
            cell(&row.customer),
            cell(row.token.as_deref().unwrap_or_default()),
            cell(row.token_hint.as_deref().unwrap_or_default()),
            cell(&row.repository),
            row.day.to_string(),
            row.downloads.to_string(),
            row.bytes.to_string(),
        ];
        out.push_str(&cells.join(","));
        out.push_str("\r\n");
    }
    out
}

pub(crate) fn cell(text: &str) -> String {
    let mut value = String::with_capacity(text.len() + 2);
    if text.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        value.push('\'');
    }
    value.push_str(text);
    match value.contains([',', '"', '\r', '\n']) {
        true => format!("\"{}\"", value.replace('"', "\"\"")),
        false => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_cells_are_defused_and_escaped() {
        assert_eq!(cell("plain"), "plain");
        assert_eq!(cell(""), "");
        for formula in ["=1+1", "+1", "-1", "@SUM(A1)", "\tx", "\rx"] {
            assert!(
                cell(formula).trim_matches('"').starts_with('\''),
                "{formula:?}"
            );
        }
        assert_eq!(cell("a,b"), "\"a,b\"");
        assert_eq!(cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(cell("line\nbreak"), "\"line\nbreak\"");
        assert_eq!(cell("=a,b"), "\"'=a,b\"");
    }

    #[test]
    fn the_row_cap_advice_suits_the_request() {
        let message =
            |one_customer: bool| match too_many(CustomerError::TooManyRows(2), one_customer) {
                ApiError::BadRequest(message) => message,
                other => panic!("{other:?}"),
            };
        assert_eq!(message(true), "narrow the period");
        assert_eq!(message(false), "narrow the period or choose one customer");
    }

    #[test]
    fn days_are_bounded() {
        assert_eq!(days_of(None).unwrap(), 30);
        assert_eq!(days_of(Some(730)).unwrap(), 730);
        assert!(days_of(Some(0)).is_err());
        assert!(days_of(Some(731)).is_err());
    }
}
