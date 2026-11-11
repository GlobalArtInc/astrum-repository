use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use astrum_security::TokenRef;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use crate::ContentError;

pub const DOWNLOAD_DAYS_KEPT: i32 = 180;

pub const DOWNLOAD_AUDIT_INTERVAL: chrono::Duration = chrono::Duration::hours(12);

pub const TOKEN_DAYS_KEPT_KEY: &str = "customers.usage.daysKept";

pub const TOKEN_DAYS_KEPT_DEFAULT: i64 = 730;

const TOKEN_DAYS_KEPT_MAX: i64 = 3650;

const SECONDS_PER_DAY: u64 = 86_400;

type TokenKey = (Uuid, String, i32);

#[derive(Default)]
struct TokenDay {
    customer: Option<String>,
    downloads: i64,
    bytes: i64,
}

#[derive(Default)]
pub(crate) struct Downloads {
    pending: Mutex<HashMap<(i64, i32), i64>>,
    tokens: Mutex<HashMap<TokenKey, TokenDay>>,
    audited: Mutex<HashSet<i64>>,
}

fn today() -> i32 {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    i32::try_from(seconds / SECONDS_PER_DAY).unwrap_or(i32::MAX)
}

impl Downloads {
    pub(crate) fn note(&self, asset_id: i64) {
        *self
            .pending
            .lock()
            .expect("the download counter lock is never poisoned")
            .entry((asset_id, today()))
            .or_default() += 1;
    }

    pub(crate) fn note_token(&self, token: &TokenRef, repository: &str, counted: bool, bytes: u64) {
        let mut tokens = self
            .tokens
            .lock()
            .expect("the token download counter lock is never poisoned");
        let entry = tokens
            .entry((token.id, repository.to_owned(), today()))
            .or_default();
        entry.customer.clone_from(&token.customer);
        if counted {
            entry.downloads = entry.downloads.saturating_add(1);
        }
        entry.bytes = entry
            .bytes
            .saturating_add(i64::try_from(bytes).unwrap_or(i64::MAX));
    }

    pub(crate) fn first_since_audit(
        &self,
        asset_id: i64,
        last_downloaded: Option<chrono::DateTime<chrono::Utc>>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> bool {
        let stale = last_downloaded.is_none_or(|last| now - last >= DOWNLOAD_AUDIT_INTERVAL);
        stale
            && self
                .audited
                .lock()
                .expect("the download audit lock is never poisoned")
                .insert(asset_id)
    }

    pub(crate) async fn flush(&self, db: &DatabaseConnection) -> Result<(), ContentError> {
        let assets = self.save(db).await;
        let tokens = self.save_tokens(db).await;
        self.audited
            .lock()
            .expect("the download audit lock is never poisoned")
            .clear();
        match (assets, tokens) {
            (Err(first), Err(second)) => {
                tracing::warn!(error = %second, "token download counts were not saved");
                Err(first)
            }
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    async fn save(&self, db: &DatabaseConnection) -> Result<(), ContentError> {
        let pending = std::mem::take(
            &mut *self
                .pending
                .lock()
                .expect("the download counter lock is never poisoned"),
        );
        if pending.is_empty() {
            return Ok(());
        }
        let mut ids = Vec::with_capacity(pending.len());
        let mut days = Vec::with_capacity(pending.len());
        let mut counts = Vec::with_capacity(pending.len());
        for ((id, day), count) in pending {
            ids.push(id);
            days.push(day);
            counts.push(count);
        }
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "WITH counted AS ( \
               SELECT * FROM unnest($1::bigint[], $2::integer[], $3::bigint[]) AS counted(id, day, count) \
             ), counted_assets AS ( \
               UPDATE asset SET download_count = asset.download_count + total.count, \
                 last_downloaded_at = now() \
               FROM (SELECT id, sum(count)::bigint AS count FROM counted GROUP BY id) AS total \
               WHERE asset.id = total.id \
               RETURNING asset.id \
             ) \
             INSERT INTO asset_download_day (asset_id, day, downloads) \
             SELECT counted.id, DATE '1970-01-01' + counted.day, counted.count \
             FROM counted JOIN counted_assets ON counted_assets.id = counted.id \
             ON CONFLICT (asset_id, day) DO UPDATE \
               SET downloads = asset_download_day.downloads + EXCLUDED.downloads",
            [ids.into(), days.into(), counts.into()],
        ))
        .await?;
        Ok(())
    }

    async fn save_tokens(&self, db: &DatabaseConnection) -> Result<(), ContentError> {
        let pending = std::mem::take(
            &mut *self
                .tokens
                .lock()
                .expect("the token download counter lock is never poisoned"),
        );
        if pending.is_empty() {
            return Ok(());
        }
        let rows = pending.into_iter().collect::<Vec<_>>();
        let mut ids = Vec::with_capacity(rows.len());
        let mut customers = Vec::with_capacity(rows.len());
        let mut repositories = Vec::with_capacity(rows.len());
        let mut days = Vec::with_capacity(rows.len());
        let mut downloads = Vec::with_capacity(rows.len());
        let mut bytes = Vec::with_capacity(rows.len());
        for ((id, repository, day), usage) in rows {
            ids.push(id);
            customers.push(usage.customer.unwrap_or_default());
            repositories.push(repository);
            days.push(day);
            downloads.push(usage.downloads);
            bytes.push(usage.bytes);
        }
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "WITH counted AS ( \
               SELECT * FROM unnest($1::uuid[], $2::text[], $3::text[], $4::integer[], $5::bigint[], $6::bigint[]) \
                 AS counted(token_id, customer, repository, day, downloads, bytes) \
             ), live AS MATERIALIZED ( \
               SELECT id FROM customer \
               WHERE id IN (SELECT customer FROM counted WHERE customer <> '') \
               ORDER BY id FOR KEY SHARE \
             ) \
             INSERT INTO token_download_day (token_id, customer, repository, day, downloads, bytes) \
             SELECT counted.token_id, NULLIF(counted.customer, ''), counted.repository, \
               DATE '1970-01-01' + counted.day, counted.downloads, counted.bytes \
             FROM counted \
             WHERE counted.customer = '' OR counted.customer IN (SELECT id FROM live) \
             ORDER BY counted.token_id, counted.repository, counted.day \
             ON CONFLICT (token_id, repository, day) DO UPDATE \
               SET downloads = token_download_day.downloads + EXCLUDED.downloads, \
                 bytes = token_download_day.bytes + EXCLUDED.bytes",
            [
                ids.into(),
                customers.into(),
                repositories.into(),
                days.into(),
                downloads.into(),
                bytes.into(),
            ],
        ))
        .await?;
        Ok(())
    }

    pub(crate) async fn prune_tokens(db: &DatabaseConnection) -> Result<u64, ContentError> {
        let kept = match astrum_db::setting::get::<i64>(db, TOKEN_DAYS_KEPT_KEY).await {
            Ok(Some(kept)) => kept,
            Ok(None) => TOKEN_DAYS_KEPT_DEFAULT,
            Err(astrum_db::Error::Setting { key, source }) => {
                tracing::warn!(%key, error = %source, "the setting is not a number of days; the default is used");
                TOKEN_DAYS_KEPT_DEFAULT
            }
            Err(error) => return Err(error.into()),
        }
        .clamp(1, TOKEN_DAYS_KEPT_MAX);
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM token_download_day WHERE day < DATE '1970-01-01' + $1::integer",
                [(i64::from(today()) - kept + 1)
                    .clamp(0, i64::from(i32::MAX))
                    .into()],
            ))
            .await?;
        Ok(result.rows_affected())
    }

    pub(crate) async fn prune(db: &DatabaseConnection) -> Result<u64, ContentError> {
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM asset_download_day WHERE day < DATE '1970-01-01' + $1::integer",
                [(today() - DOWNLOAD_DAYS_KEPT + 1).into()],
            ))
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_download_is_audited_once_per_interval() {
        let downloads = Downloads::default();
        let now = chrono::Utc::now();
        assert!(downloads.first_since_audit(1, None, now));
        assert!(!downloads.first_since_audit(1, None, now));
        assert!(!downloads.first_since_audit(2, Some(now - chrono::Duration::hours(1)), now));
        assert!(downloads.first_since_audit(2, Some(now - chrono::Duration::hours(13)), now));
        downloads.audited.lock().unwrap().clear();
        assert!(downloads.first_since_audit(1, None, now));
    }
}
