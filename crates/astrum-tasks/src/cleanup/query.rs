use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, FromQueryResult, Statement, Value};
use serde::Serialize;

use super::{Criteria, ReleaseType, prerelease_rule};

pub const SUMMARY_LIMIT: Duration = Duration::from_secs(120);
const BATCH: i64 = 1000;

#[derive(Clone, Debug)]
pub struct Matcher {
    pub repository: String,
    pub format: String,
    pub criteria: Criteria,
    pub now: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub id: i64,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub asset_count: i64,
    pub total_size: i64,
    pub last_updated: Option<DateTime<FixedOffset>>,
    pub last_downloaded: Option<DateTime<FixedOffset>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub repository: String,
    pub component_count: i64,
    pub asset_count: i64,
    pub total_size: i64,
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, FromQueryResult)]
pub struct AssetRow {
    pub component_id: i64,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub blob_store: String,
    pub size: i64,
    pub last_downloaded_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    ById { after: i64 },
    ByCoordinates { offset: i64 },
}

impl Matcher {
    pub fn new(repository: &str, format: &str, criteria: Criteria) -> Self {
        Self {
            repository: repository.to_owned(),
            format: format.to_owned(),
            criteria,
            now: Utc::now(),
        }
    }

    fn cutoff(&self, days: i64) -> DateTime<FixedOffset> {
        (self.now - chrono::Duration::days(days)).fixed_offset()
    }

    fn statement(&self, order: Order, limit: i64) -> Statement {
        let mut values: Vec<Value> = vec![self.repository.clone().into()];
        let mut filters = Vec::new();
        if let Some(release_type) = self.criteria.release_type {
            filters.push(match prerelease_rule(&self.format) {
                Some(rule) => match release_type {
                    ReleaseType::Prereleases => rule.to_owned(),
                    ReleaseType::Releases => format!("NOT {rule}"),
                },
                None => "false".to_owned(),
            });
        }
        if let Order::ById { after } = order {
            values.push(after.into());
            filters.push(format!("c.id > ${}", values.len()));
        }
        let mut having = Vec::new();
        if let Some(days) = self.criteria.last_blob_updated {
            values.push(self.cutoff(days).into());
            having.push(format!("max(a.blob_updated_at) < ${}", values.len()));
        }
        if let Some(days) = self.criteria.last_downloaded {
            values.push(self.cutoff(days).into());
            having.push(format!(
                "max(coalesce(a.last_downloaded_at, a.created_at)) < ${}",
                values.len()
            ));
        }
        if let Some(pattern) = &self.criteria.asset_regex {
            let (flags, body) = match pattern.strip_prefix("(?i)") {
                Some(body) => ("(?i)", body),
                None => ("", pattern.as_str()),
            };
            values.push(format!("{flags}^(?:{body})$").into());
            having.push(format!(
                "bool_or(substr(a.path, 2) ~ ${0} OR a.path ~ ${0})",
                values.len()
            ));
        }
        if self.criteria.is_empty() {
            having.push("false".to_owned());
        }
        let filters = filters
            .iter()
            .map(|filter| format!(" AND {filter}"))
            .collect::<String>();
        let having = match having.is_empty() {
            true => String::new(),
            false => format!(" HAVING {}", having.join(" AND ")),
        };
        values.push(limit.into());
        let limit_at = values.len();
        let tail = match order {
            Order::ById { .. } => format!(" ORDER BY c.id LIMIT ${limit_at}"),
            Order::ByCoordinates { offset } => {
                values.push(offset.into());
                format!(
                    " ORDER BY c.namespace, c.name, c.version, c.id LIMIT ${limit_at} OFFSET ${}",
                    values.len()
                )
            }
        };
        let sql = format!(
            "SELECT c.id, c.namespace, c.name, c.version, count(a.id) AS asset_count, \
             coalesce(sum(a.size), 0)::bigint AS total_size, \
             max(a.blob_updated_at) AS last_updated, max(a.last_downloaded_at) AS last_downloaded \
             FROM component c JOIN asset a ON a.component_id = c.id \
             WHERE c.repository = $1{filters} GROUP BY c.id{having}{tail}"
        );
        Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
    }

    pub async fn batch(
        &self,
        db: &impl ConnectionTrait,
        after: i64,
        limit: i64,
    ) -> Result<Vec<Match>, DbErr> {
        Match::find_by_statement(self.statement(Order::ById { after }, limit))
            .all(db)
            .await
    }

    pub async fn page(
        &self,
        db: &impl ConnectionTrait,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Match>, DbErr> {
        Match::find_by_statement(self.statement(Order::ByCoordinates { offset }, limit))
            .all(db)
            .await
    }

    pub async fn summary(
        &self,
        db: &impl ConnectionTrait,
        limit: Duration,
    ) -> Result<Summary, DbErr> {
        let started = Instant::now();
        let mut summary = Summary {
            repository: self.repository.clone(),
            complete: true,
            ..Summary::default()
        };
        let mut after = 0;
        loop {
            if started.elapsed() >= limit {
                summary.complete = false;
                return Ok(summary);
            }
            let found = self.batch(db, after, BATCH).await?;
            for found in &found {
                summary.component_count += 1;
                summary.asset_count += found.asset_count;
                summary.total_size += found.total_size;
                after = found.id;
            }
            if (found.len() as i64) < BATCH {
                return Ok(summary);
            }
        }
    }

    pub async fn assets(
        &self,
        db: &impl ConnectionTrait,
        after: i64,
        limit: i64,
    ) -> Result<(Vec<AssetRow>, Option<i64>), DbErr> {
        let found = self.batch(db, after, limit).await?;
        let Some(last) = found.last().map(|found| found.id) else {
            return Ok((Vec::new(), None));
        };
        let ids: Vec<i64> = found.iter().map(|found| found.id).collect();
        let rows = AssetRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT a.component_id, c.namespace, c.name, c.version, a.path, a.blob_store, a.size, \
             a.last_downloaded_at, a.created_at FROM asset a JOIN component c ON c.id = a.component_id \
             WHERE a.component_id = ANY($1) ORDER BY a.component_id, a.path",
            [ids.into()],
        ))
        .all(db)
        .await?;
        let next = ((found.len() as i64) == limit).then_some(last);
        Ok((rows, next))
    }
}

pub fn csv_header() -> &'static str {
    "Namespace,Name,Version,Path,Blob Store Name,Asset Size,Downloaded Date ISO-8601,Published Date ISO-8601\n"
}

pub fn csv_line(row: &AssetRow) -> String {
    let time = |at: &DateTime<FixedOffset>| {
        at.with_timezone(&Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    };
    let fields = [
        row.namespace.clone(),
        row.name.clone(),
        row.version.clone(),
        row.path.clone(),
        row.blob_store.clone(),
        row.size.to_string(),
        row.last_downloaded_at
            .as_ref()
            .map(time)
            .unwrap_or_default(),
        time(&row.created_at),
    ];
    let mut line = fields
        .iter()
        .map(|field| csv_field(field))
        .collect::<Vec<_>>()
        .join(",");
    line.push('\n');
    line
}

fn csv_field(value: &str) -> String {
    let guarded = match value.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{value}"),
        _ => value.to_owned(),
    };
    match guarded.contains([',', '"', '\n', '\r']) {
        true => format!("\"{}\"", guarded.replace('"', "\"\"")),
        false => guarded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_fields_are_quoted_and_guarded() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("=cmd()"), "'=cmd()");
        assert_eq!(csv_field("-1"), "'-1");
    }

    #[test]
    fn statements_bind_every_criterion() {
        let matcher = Matcher::new(
            "maven-snapshots",
            "maven2",
            Criteria {
                last_blob_updated: Some(10),
                last_downloaded: Some(20),
                release_type: Some(ReleaseType::Prereleases),
                asset_regex: Some("(?i).*\\.jar".to_owned()),
            },
        );
        let statement = matcher.statement(Order::ByCoordinates { offset: 50 }, 25);
        assert!(statement.sql.contains("LIKE '%-SNAPSHOT'"));
        assert!(statement.sql.contains("max(a.blob_updated_at) < $2"));
        assert!(statement.sql.contains("< $3"));
        assert!(
            statement
                .sql
                .contains("bool_or(substr(a.path, 2) ~ $4 OR a.path ~ $4)")
        );
        assert!(statement.sql.contains("LIMIT $5 OFFSET $6"));
        let values = statement.values.unwrap().0;
        assert_eq!(values[3], Value::from("(?i)^(?:.*\\.jar)$"));
        let empty =
            Matcher::new("raw", "raw", Criteria::default()).statement(Order::ById { after: 0 }, 5);
        assert!(empty.sql.contains("HAVING false"));
    }
}
