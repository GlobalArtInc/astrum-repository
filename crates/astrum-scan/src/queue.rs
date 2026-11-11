use std::time::Duration;

use sea_orm::sqlx;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, RuntimeErr, Statement, TransactionTrait,
};

use crate::worker::EXTRACTOR;

pub const CHUNK: i64 = 10_000;

const LOCK_NOT_AVAILABLE: &str = "55P03";
const DEADLOCK_DETECTED: &str = "40P01";

#[derive(Clone, Copy, Debug)]
pub struct Retry {
    pub lock_timeout: Duration,
    pub attempts: u32,
    pub pause: Duration,
}

impl Default for Retry {
    fn default() -> Self {
        Self {
            lock_timeout: Duration::from_millis(200),
            attempts: 8,
            pause: Duration::from_millis(100),
        }
    }
}

const RESCAN: &str = "ON CONFLICT (component_id) DO UPDATE SET queued_at = now(), \
     next_attempt_at = now() WHERE scan_queue.queued_at <> now()";

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("the database failed: {0}")]
    Database(#[from] DbErr),
    #[error("the enqueue was stopped")]
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Every,
    Unscanned,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Enqueued {
    pub components: u64,
    pub queued: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Depth {
    pub queued: i64,
    pub oldest_seconds: f64,
}

fn chunk_sql(selection: Selection) -> String {
    let (filter, conflict) = match selection {
        Selection::Every => (String::new(), RESCAN.to_owned()),
        Selection::Unscanned => (
            format!(
                "AND NOT EXISTS (SELECT 1 FROM component_scan s \
             LEFT JOIN sca_inventory i ON i.id = s.inventory_id \
             WHERE s.component_id = c.id AND s.state NOT IN ('failed', 'incomplete') \
             AND (i.id IS NULL OR i.extractor >= {EXTRACTOR}))"
            ),
            "ON CONFLICT (component_id) DO NOTHING".to_owned(),
        ),
    };
    format!(
        "WITH chunk AS (\
           SELECT c.id FROM component c WHERE c.repository = $1 AND c.id > $2 {filter} \
           ORDER BY c.id LIMIT $3), \
         live AS (\
           SELECT c.id FROM component c WHERE c.id IN (SELECT id FROM chunk) \
           ORDER BY c.id FOR KEY SHARE), \
         added AS (\
           INSERT INTO scan_queue (component_id, reason) \
           SELECT id, $4 FROM live ORDER BY id {conflict} RETURNING component_id) \
         SELECT (SELECT max(id) FROM chunk) AS last, \
                (SELECT count(*) FROM chunk) AS seen, \
                (SELECT count(*) FROM added) AS queued"
    )
}

fn retryable(error: &DbErr) -> bool {
    let sqlx_error = match error {
        DbErr::Query(RuntimeErr::SqlxError(error)) | DbErr::Exec(RuntimeErr::SqlxError(error)) => {
            error
        }
        _ => return false,
    };
    match sqlx_error {
        sqlx::Error::Database(error) => matches!(
            error.code().as_deref(),
            Some(LOCK_NOT_AVAILABLE) | Some(DEADLOCK_DETECTED)
        ),
        _ => false,
    }
}

struct ChunkOutcome {
    last: Option<i64>,
    seen: i64,
    queued: i64,
}

async fn run_chunk(
    db: &DatabaseConnection,
    sql: &str,
    values: [sea_orm::Value; 4],
    lock_timeout: Duration,
) -> Result<ChunkOutcome, DbErr> {
    let txn = db.begin().await?;
    txn.execute_unprepared(&format!(
        "SET LOCAL lock_timeout = '{}ms'",
        lock_timeout.as_millis()
    ))
    .await?;
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            values,
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the chunk query returned no row".to_owned()))?;
    let outcome = ChunkOutcome {
        last: row.try_get("", "last")?,
        seen: row.try_get("", "seen")?,
        queued: row.try_get("", "queued")?,
    };
    txn.commit().await?;
    Ok(outcome)
}

async fn chunk_with_retry(
    db: &DatabaseConnection,
    sql: &str,
    values: [sea_orm::Value; 4],
    retry: Retry,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<ChunkOutcome, QueueError> {
    let mut attempt = 1;
    loop {
        match run_chunk(db, sql, values.clone(), retry.lock_timeout).await {
            Ok(outcome) => return Ok(outcome),
            Err(error) if retryable(&error) && attempt < retry.attempts => {
                tracing::debug!(%error, attempt, "a scan queue chunk met a lock and is repeated");
                tokio::time::sleep(retry.pause * attempt).await;
                if cancelled() {
                    return Err(QueueError::Cancelled);
                }
                attempt += 1;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

pub async fn enqueue_repository(
    db: &DatabaseConnection,
    repository: &str,
    reason: &str,
    selection: Selection,
    chunk: i64,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Enqueued, QueueError> {
    enqueue_repository_with(
        db,
        repository,
        reason,
        selection,
        chunk,
        Retry::default(),
        cancelled,
    )
    .await
}

pub async fn enqueue_repository_with(
    db: &DatabaseConnection,
    repository: &str,
    reason: &str,
    selection: Selection,
    chunk: i64,
    retry: Retry,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Enqueued, QueueError> {
    let sql = chunk_sql(selection);
    let mut total = Enqueued::default();
    let mut after = 0i64;
    loop {
        if cancelled() {
            return Err(QueueError::Cancelled);
        }
        let values = [repository.into(), after.into(), chunk.into(), reason.into()];
        let done = chunk_with_retry(db, &sql, values, retry, cancelled).await?;
        total.components += done.seen as u64;
        total.queued += done.queued as u64;
        match done.last {
            Some(last) if done.seen >= chunk => after = last,
            _ => return Ok(total),
        }
    }
}

pub async fn enqueue_component(
    db: &impl ConnectionTrait,
    repository: &str,
    component_id: i64,
    reason: &str,
) -> Result<Option<u64>, DbErr> {
    let found = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 AS found FROM component WHERE id = $1 AND repository = $2",
            [component_id.into(), repository.into()],
        ))
        .await?;
    if found.is_none() {
        return Ok(None);
    }
    let added = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!(
                "INSERT INTO scan_queue (component_id, reason) \
                 SELECT id, $2 FROM component WHERE id = $1 {RESCAN}"
            ),
            [component_id.into(), reason.into()],
        ))
        .await?;
    Ok(Some(added.rows_affected()))
}

pub async fn depth(db: &impl ConnectionTrait) -> Result<Depth, DbErr> {
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS queued, \
             COALESCE(EXTRACT(EPOCH FROM now() - min(queued_at)), 0)::float8 AS oldest \
             FROM scan_queue",
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the depth query returned no row".to_owned()))?;
    Ok(Depth {
        queued: row.try_get("", "queued")?,
        oldest_seconds: row.try_get("", "oldest")?,
    })
}

#[cfg(test)]
mod tests;
