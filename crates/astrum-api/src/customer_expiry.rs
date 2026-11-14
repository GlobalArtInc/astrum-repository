use std::collections::BTreeSet;

use astrum_db::setting;
use astrum_db::system::{CUSTOMER_EXPIRY_NOTICE_KEY, CustomerExpiryNotice};
use astrum_notify::Notifier;
use astrum_notify::channel::Channel;
use astrum_notify::customers::{self, Expiring};
use astrum_notify::store;
use astrum_tasks::{CUSTOMER_TOKEN_EXPIRY, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use uuid::Uuid;

const CANDIDATES: &str = "SELECT t.id, t.customer, c.name AS customer_name, t.name AS token_name, t.expires_at, \
     ceil(extract(epoch FROM t.expires_at - now()) / 86400)::bigint AS days_left \
     FROM access_token t JOIN customer c ON c.id = t.customer \
     WHERE t.customer IS NOT NULL AND NOT c.disabled AND t.expires_at IS NOT NULL \
     AND ((t.expires_at > now() AND t.expires_at <= now() + make_interval(days => $1)) \
     OR (t.expires_at <= now() AND t.expires_at > now() - interval '7 days')) \
     ORDER BY t.expires_at, t.id";

const MARK: &str = "UPDATE access_token SET expiry_notice = $2 \
     WHERE id = $1 AND (expiry_notice IS NULL OR expiry_notice > $2)";

fn threshold_for(days_left: i64, days: &[u32]) -> Option<i16> {
    match days_left {
        ..=0 => Some(0),
        left => days
            .iter()
            .filter(|day| i64::from(**day) >= left)
            .min()
            .map(|day| *day as i16),
    }
}

pub struct CustomerTokenExpiry {
    notifier: Notifier,
}

impl CustomerTokenExpiry {
    pub fn new(notifier: Notifier) -> Self {
        Self { notifier }
    }
}

#[async_trait]
impl TaskKind for CustomerTokenExpiry {
    fn id(&self) -> &'static str {
        CUSTOMER_TOKEN_EXPIRY
    }

    fn name(&self) -> &'static str {
        "Admin - Notify about expiring customer tokens"
    }

    fn singleton(&self) -> bool {
        true
    }

    fn describe(&self, _properties: &Properties) -> String {
        "notifies the channels about customer tokens that expire soon or have just expired"
            .to_owned()
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let db = run.db();
        let notice = match setting::get::<CustomerExpiryNotice>(db, CUSTOMER_EXPIRY_NOTICE_KEY)
            .await
            .map_err(TaskError::failed)?
        {
            Some(notice) if notice.enabled => notice,
            Some(_) => return Ok("the expiry notice is disabled".to_owned()),
            None => return Ok("no channels are set".to_owned()),
        };
        let wanted: BTreeSet<String> = notice
            .channels
            .iter()
            .map(|name| name.to_lowercase())
            .collect();
        let channels: Vec<Channel> = store::list(db)
            .await?
            .into_iter()
            .filter(|channel| channel.enabled && wanted.contains(&channel.name.to_lowercase()))
            .collect();
        if channels.is_empty() {
            return Ok("no channels are set".to_owned());
        }
        let horizon = notice.days.iter().copied().max().unwrap_or(0) as i32;
        let rows = db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                CANDIDATES,
                [horizon.into()],
            ))
            .await?;
        let mut pending: Vec<(Uuid, i16, Expiring)> = Vec::new();
        for row in rows {
            let days_left: i64 = row.try_get("", "days_left")?;
            let Some(threshold) = threshold_for(days_left, &notice.days) else {
                continue;
            };
            let expires_at: DateTime<Utc> = row.try_get("", "expires_at")?;
            pending.push((
                row.try_get("", "id")?,
                threshold,
                Expiring {
                    customer: row.try_get("", "customer")?,
                    customer_name: row.try_get("", "customer_name")?,
                    token: row.try_get("", "token_name")?,
                    expires_at,
                    days_left,
                },
            ));
        }
        let base_url = self.notifier.base_url().await;
        let txn = db.begin().await?;
        let mut expiring = Vec::new();
        let mut expired = Vec::new();
        for (id, threshold, token) in pending {
            let changed = txn
                .execute(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    MARK,
                    [id.into(), threshold.into()],
                ))
                .await?
                .rows_affected();
            if changed == 0 {
                continue;
            }
            match threshold {
                0 => expired.push(token),
                _ => expiring.push(token),
            }
        }
        for channel in &channels {
            if !expiring.is_empty() {
                let text =
                    customers::render(channel, customers::EXPIRING, &expiring, base_url.as_deref());
                self.notifier
                    .enqueue(&txn, channel, customers::EXPIRING, text)
                    .await?;
            }
            if !expired.is_empty() {
                let text =
                    customers::render(channel, customers::EXPIRED, &expired, base_url.as_deref());
                self.notifier
                    .enqueue(&txn, channel, customers::EXPIRED, text)
                    .await?;
            }
        }
        txn.commit().await?;
        run.count("expiring", expiring.len() as i64);
        run.count("expired", expired.len() as i64);
        let summary = format!(
            "{} expiring and {} expired customer tokens announced",
            expiring.len(),
            expired.len()
        );
        run.info(summary.clone());
        Ok(summary)
    }
}
