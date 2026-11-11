use std::time::{SystemTime, UNIX_EPOCH};

use astrum_db::entity::customer;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sea_orm::{
    ActiveValue, DbBackend, EntityTrait, FromQueryResult, QuerySelect, SqlErr, Statement,
    TransactionTrait, Value,
};

use crate::{Error, Security};

const SECONDS_PER_DAY: u64 = 86_400;
pub const SUMMARY_DAYS: i64 = 30;

#[derive(Clone, Debug)]
pub struct NewCustomer {
    pub id: String,
    pub name: String,
    pub reference: String,
    pub note: String,
    pub created_by: String,
}

#[derive(Clone, Debug)]
pub struct CustomerFields {
    pub name: String,
    pub reference: String,
    pub note: String,
    pub disabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CustomerError {
    #[error("a customer with this id already exists")]
    Exists,
    #[error("the answer would hold more than {0} rows")]
    TooManyRows(usize),
    #[error(transparent)]
    Internal(#[from] Error),
}

impl From<sea_orm::DbErr> for CustomerError {
    fn from(error: sea_orm::DbErr) -> Self {
        CustomerError::Internal(error.into())
    }
}

#[derive(Clone, Debug, FromQueryResult)]
pub struct CustomerSummary {
    pub id: String,
    pub name: String,
    pub reference: String,
    pub note: String,
    pub disabled: bool,
    pub created_by: String,
    pub created_at: DateTime<FixedOffset>,
    pub tokens: i64,
    pub nearest_expiry: Option<DateTime<FixedOffset>>,
    pub last_access_at: Option<DateTime<FixedOffset>>,
    pub downloads: i64,
    pub bytes: i64,
}

#[derive(Clone, Debug, FromQueryResult)]
pub struct UsageDay {
    pub day: NaiveDate,
    pub repository: String,
    pub downloads: i64,
    pub bytes: i64,
}

#[derive(Clone, Debug, FromQueryResult)]
pub struct UsageExportRow {
    pub customer: String,
    pub token: Option<String>,
    pub token_hint: Option<String>,
    pub repository: String,
    pub day: NaiveDate,
    pub downloads: i64,
    pub bytes: i64,
}

fn today() -> i64 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    i64::try_from(seconds / SECONDS_PER_DAY).unwrap_or(i64::MAX)
}

fn first_day(days: i64) -> i64 {
    today() - days + 1
}

fn statement(sql: &str, values: impl IntoIterator<Item = Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

const SUMMARY: &str = "\
SELECT c.id, c.name, c.reference, c.note, c.disabled, c.created_by, c.created_at, \
       COALESCE(t.tokens, 0)::bigint AS tokens, t.nearest_expiry, t.last_access_at, \
       COALESCE(u.downloads, 0)::bigint AS downloads, COALESCE(u.bytes, 0)::bigint AS bytes \
FROM customer c \
LEFT JOIN ( \
    SELECT customer, \
           count(*) FILTER (WHERE expires_at IS NULL OR expires_at > now()) AS tokens, \
           min(expires_at) FILTER (WHERE expires_at > now()) AS nearest_expiry, \
           max(last_used_at) AS last_access_at \
    FROM access_token WHERE customer IS NOT NULL GROUP BY customer \
) t ON t.customer = c.id \
LEFT JOIN ( \
    SELECT customer, sum(downloads) AS downloads, sum(bytes) AS bytes \
    FROM token_download_day \
    WHERE customer IS NOT NULL AND day >= DATE '1970-01-01' + $1::integer \
    GROUP BY customer \
) u ON u.customer = c.id \
WHERE ($2::text IS NULL OR c.id = $2::text) \
ORDER BY c.id";

const USAGE: &str = "\
SELECT day, repository, sum(downloads)::bigint AS downloads, sum(bytes)::bigint AS bytes \
FROM token_download_day \
WHERE customer = $1 AND day >= DATE '1970-01-01' + $2::integer \
GROUP BY day, repository \
ORDER BY day, repository \
LIMIT $3";

const EXPORT: &str = "\
SELECT d.customer, a.name AS token, a.token_hint AS token_hint, d.repository, d.day, \
       d.downloads, d.bytes \
FROM token_download_day d \
LEFT JOIN access_token a ON a.id = d.token_id \
WHERE d.customer IS NOT NULL AND d.day >= DATE '1970-01-01' + $1::integer \
  AND ($2::text IS NULL OR d.customer = $2::text) \
ORDER BY d.customer, d.day, d.repository, d.token_id \
LIMIT $3";

impl Security {
    pub async fn customer_summaries(
        &self,
        only: Option<&str>,
    ) -> Result<Vec<CustomerSummary>, Error> {
        let values = [
            Value::from(first_day(SUMMARY_DAYS) as i32),
            Value::from(only.map(str::to_owned)),
        ];
        Ok(
            CustomerSummary::find_by_statement(statement(SUMMARY, values))
                .all(&self.db)
                .await?,
        )
    }

    pub async fn customer(&self, id: &str) -> Result<Option<customer::Model>, Error> {
        Ok(customer::Entity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn create_customer(
        &self,
        new: NewCustomer,
    ) -> Result<customer::Model, CustomerError> {
        let inserted = customer::Entity::insert(customer::ActiveModel {
            id: ActiveValue::Set(new.id),
            name: ActiveValue::Set(new.name),
            reference: ActiveValue::Set(new.reference),
            note: ActiveValue::Set(new.note),
            disabled: ActiveValue::Set(false),
            created_by: ActiveValue::Set(new.created_by),
            created_at: ActiveValue::Set(Utc::now().fixed_offset()),
        })
        .exec_with_returning(&self.db)
        .await;
        match inserted {
            Ok(model) => Ok(model),
            Err(error) if matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => {
                Err(CustomerError::Exists)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn update_customer(
        &self,
        id: &str,
        fields: CustomerFields,
    ) -> Result<Option<(customer::Model, customer::Model)>, Error> {
        let txn = self.db.begin().await?;
        let Some(before) = customer::Entity::find_by_id(id)
            .lock_exclusive()
            .one(&txn)
            .await?
        else {
            return Ok(None);
        };
        let after = customer::Entity::update(customer::ActiveModel {
            id: ActiveValue::Unchanged(id.to_owned()),
            name: ActiveValue::Set(fields.name),
            reference: ActiveValue::Set(fields.reference),
            note: ActiveValue::Set(fields.note),
            disabled: ActiveValue::Set(fields.disabled),
            ..Default::default()
        })
        .exec(&txn)
        .await?;
        txn.commit().await?;
        Ok(Some((before, after)))
    }

    pub async fn delete_customer(&self, id: &str) -> Result<Option<customer::Model>, Error> {
        let txn = self.db.begin().await?;
        let Some(found) = customer::Entity::find_by_id(id)
            .lock_exclusive()
            .one(&txn)
            .await?
        else {
            return Ok(None);
        };
        customer::Entity::delete_by_id(id).exec(&txn).await?;
        txn.commit().await?;
        Ok(Some(found))
    }

    pub async fn customer_usage(
        &self,
        id: &str,
        days: i64,
        limit: usize,
    ) -> Result<Vec<UsageDay>, CustomerError> {
        let values = [
            Value::from(id.to_owned()),
            Value::from(first_day(days) as i32),
            Value::from(row_limit(limit)),
        ];
        let rows = UsageDay::find_by_statement(statement(USAGE, values))
            .all(&self.db)
            .await?;
        within(rows, limit)
    }

    pub async fn usage_export(
        &self,
        days: i64,
        customer: Option<&str>,
        limit: usize,
    ) -> Result<Vec<UsageExportRow>, CustomerError> {
        let values = [
            Value::from(first_day(days) as i32),
            Value::from(customer.map(str::to_owned)),
            Value::from(row_limit(limit)),
        ];
        let rows = UsageExportRow::find_by_statement(statement(EXPORT, values))
            .all(&self.db)
            .await?;
        within(rows, limit)
    }
}

fn row_limit(limit: usize) -> i64 {
    i64::try_from(limit).map_or(i64::MAX, |limit| limit.saturating_add(1))
}

fn within<T>(rows: Vec<T>, limit: usize) -> Result<Vec<T>, CustomerError> {
    match rows.len() > limit {
        true => Err(CustomerError::TooManyRows(limit)),
        false => Ok(rows),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_counts_today_as_its_last_day() {
        assert_eq!(first_day(1), today());
        assert_eq!(first_day(30), today() - 29);
    }

    #[test]
    fn a_full_answer_is_refused_only_beyond_the_limit() {
        assert_eq!(within(vec![1, 2], 2).unwrap().len(), 2);
        assert!(matches!(
            within(vec![1, 2, 3], 2),
            Err(CustomerError::TooManyRows(2))
        ));
    }
}
