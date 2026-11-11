use std::collections::HashMap;

use astrum_db::revision;
use chrono::{DateTime, Duration, Utc};
use sea_orm::sqlx::{self, Postgres, pool::PoolConnection};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement, TransactionTrait,
};

use super::import::{self, ImportError, Stats};
use super::source::{AdvisorySource, Fetched, SourceError};
use super::valid_ecosystem;

const MAX_ERROR_CHARS: usize = 1000;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("{0:?} is not an ecosystem name")]
    Ecosystem(String),
    #[error("{0}")]
    Source(#[from] SourceError),
    #[error("{0}")]
    Import(#[from] ImportError),
    #[error("an import of the ecosystem {0} is already running")]
    Busy(String),
    #[error("the database failed: {0}")]
    Database(#[from] DbErr),
}

const LOCK: &str = "SELECT pg_try_advisory_lock(hashtext('advisories.sync'), hashtext($1))";
const UNLOCK: &str = "SELECT pg_advisory_unlock(hashtext('advisories.sync'), hashtext($1))";

pub struct EcosystemLock {
    connection: PoolConnection<Postgres>,
    ecosystem: String,
}

impl EcosystemLock {
    pub async fn try_take(db: &DatabaseConnection, ecosystem: &str) -> Result<Option<Self>, DbErr> {
        let mut connection = db
            .get_postgres_connection_pool()
            .acquire()
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        connection.close_on_drop();
        let (taken,): (bool,) = sqlx::query_as(LOCK)
            .bind(ecosystem)
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        Ok(taken.then(|| Self {
            connection,
            ecosystem: ecosystem.to_owned(),
        }))
    }

    async fn release(mut self) {
        if let Err(error) = sqlx::query(UNLOCK)
            .bind(&self.ecosystem)
            .execute(&mut *self.connection)
            .await
        {
            tracing::warn!(%error, ecosystem = %self.ecosystem, "the advisory import lock was not released, its connection is closed instead");
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    NotModified,
    Imported(Stats),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feed {
    pub ecosystem: String,
    pub etag: Option<String>,
    pub records: i32,
    pub synced_at: Option<DateTime<Utc>>,
    pub attempted_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stale {
    pub ecosystem: String,
    pub synced_at: Option<DateTime<Utc>>,
}

fn feed_of(row: &QueryResult) -> Result<Feed, DbErr> {
    Ok(Feed {
        ecosystem: row.try_get("", "ecosystem")?,
        etag: row.try_get("", "etag")?,
        records: row.try_get("", "records")?,
        synced_at: row.try_get("", "synced_at")?,
        attempted_at: row.try_get("", "attempted_at")?,
        last_error: row.try_get("", "last_error")?,
    })
}

pub async fn feeds(db: &impl ConnectionTrait) -> Result<Vec<Feed>, DbErr> {
    db.query_all(Statement::from_string(
        DbBackend::Postgres,
        "SELECT ecosystem, etag, records, synced_at, attempted_at, last_error \
         FROM advisory_feed ORDER BY ecosystem",
    ))
    .await?
    .iter()
    .map(feed_of)
    .collect()
}

pub async fn stale(
    db: &impl ConnectionTrait,
    ecosystems: &[String],
    after_days: i64,
    now: DateTime<Utc>,
) -> Result<Vec<Stale>, DbErr> {
    let known: HashMap<String, Option<DateTime<Utc>>> = feeds(db)
        .await?
        .into_iter()
        .map(|feed| (feed.ecosystem, feed.synced_at))
        .collect();
    let limit = now - Duration::days(after_days);
    Ok(ecosystems
        .iter()
        .filter_map(|ecosystem| {
            let synced_at = known.get(ecosystem).copied().flatten();
            match synced_at {
                Some(at) if at >= limit => None,
                _ => Some(Stale {
                    ecosystem: ecosystem.clone(),
                    synced_at,
                }),
            }
        })
        .collect())
}

async fn failed(db: &DatabaseConnection, ecosystem: &str, message: &str) {
    let message: String = message.chars().take(MAX_ERROR_CHARS).collect();
    let written = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE advisory_feed SET last_error = $2 WHERE ecosystem = $1",
            [ecosystem.into(), message.into()],
        ))
        .await;
    if let Err(error) = written {
        tracing::warn!(%error, ecosystem, "the failure of an advisory sync was not recorded");
    }
}

pub async fn sync(
    db: &DatabaseConnection,
    source: &dyn AdvisorySource,
    ecosystem: &str,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Outcome, SyncError> {
    sync_with(db, source, ecosystem, false, cancelled).await
}

pub async fn sync_with(
    db: &DatabaseConnection,
    source: &dyn AdvisorySource,
    ecosystem: &str,
    force: bool,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Outcome, SyncError> {
    if !valid_ecosystem(ecosystem) {
        return Err(SyncError::Ecosystem(ecosystem.to_owned()));
    }
    let Some(lock) = EcosystemLock::try_take(db, ecosystem).await? else {
        return Err(SyncError::Busy(ecosystem.to_owned()));
    };
    let outcome = run(db, source, ecosystem, force, cancelled).await;
    lock.release().await;
    outcome
}

async fn run(
    db: &DatabaseConnection,
    source: &dyn AdvisorySource,
    ecosystem: &str,
    force: bool,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Outcome, SyncError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO advisory_feed (ecosystem, attempted_at) VALUES ($1, now()) \
             ON CONFLICT (ecosystem) DO UPDATE SET attempted_at = now() RETURNING etag",
            [ecosystem.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("advisory feed {ecosystem}")))?;
    let etag: Option<String> = row.try_get("", "etag")?;
    let fetched = match source.fetch(ecosystem, etag.as_deref()).await {
        Ok(fetched) => fetched,
        Err(error) => {
            failed(db, ecosystem, &error.to_string()).await;
            return Err(error.into());
        }
    };
    let (file, new_etag) = match fetched {
        Fetched::NotModified => {
            db.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE advisory_feed SET synced_at = now(), last_error = NULL WHERE ecosystem = $1",
                [ecosystem.into()],
            ))
            .await?;
            return Ok(Outcome::NotModified);
        }
        Fetched::Data { file, etag } => (file, etag),
    };
    let mut stats = Stats::default();
    let imported =
        import::import_archive(db, ecosystem, file.path(), force, cancelled, &mut stats).await;
    drop(file);
    if let Err(error) = imported {
        if (stats.changed > 0 || stats.removed > 0)
            && let Err(bump) = revision::bump(db, revision::ADVISORIES).await
        {
            tracing::warn!(error = %bump, "the advisories revision was not raised after a partial import");
        }
        if !matches!(error, ImportError::Cancelled) {
            failed(db, ecosystem, &error.to_string()).await;
        }
        return Err(error.into());
    }
    let txn = db.begin().await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE advisory_feed SET etag = $2, records = $3, synced_at = now(), last_error = NULL \
         WHERE ecosystem = $1",
        [
            ecosystem.into(),
            new_etag.into(),
            (stats.records.min(i32::MAX as u64) as i32).into(),
        ],
    ))
    .await?;
    revision::bump(&txn, revision::ADVISORIES).await?;
    txn.commit().await?;
    Ok(Outcome::Imported(stats))
}
