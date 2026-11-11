use std::sync::Arc;
use std::time::Duration;

use astrum_content::Content;
use astrum_policy::cache::Cache;
use astrum_policy::subject;
use astrum_security::SecurityModel;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};

use crate::worker::{self, Claim, ScanError, State};
use crate::{matcher, verdict};

const LEASE: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(100);
const WAIT: Duration = Duration::from_millis(500);

#[derive(Debug, thiserror::Error)]
pub enum InlineError {
    #[error("the database failed: {0}")]
    Database(#[from] DbErr),
    #[error("the scan failed: {0}")]
    Scan(#[from] ScanError),
    #[error("the component is gone")]
    Gone,
    #[error("the scan of the component is not due: a lease or a backoff is running")]
    NotDue,
    #[error("another worker holds the component and no verdict appeared within {} ms", WAIT.as_millis())]
    Contended,
    #[error("no verdict appeared within {} ms", WAIT.as_millis())]
    NoVerdict,
}

pub async fn mark(
    db: &DatabaseConnection,
    repository: &str,
    component_id: i64,
) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO policy_dirty (repository, subject) \
         SELECT repository, $3 FROM component WHERE id = $1 AND repository = $2 \
         ON CONFLICT (repository, subject) DO NOTHING",
        [
            component_id.into(),
            repository.into(),
            subject::component(component_id).into(),
        ],
    ))
    .await?;
    Ok(())
}

pub async fn judge_component(
    content: &Content,
    security: &Arc<SecurityModel>,
    cache: &Cache,
    repository: &str,
    component_id: i64,
) -> Result<(), InlineError> {
    let db = content.db();
    let since = database_time(db).await?;
    if !scanned(db, component_id).await? {
        worker::queue_component(db, component_id).await?;
        match worker::claim_component(db, component_id, LEASE).await? {
            Claim::Taken(item) => {
                if worker::scan_claimed(content, security, item).await? == State::Gone {
                    return Err(InlineError::Gone);
                }
            }
            Claim::NotDue => return Err(InlineError::NotDue),
            Claim::Held => {
                return wait_for_verdict(db, repository, component_id, since)
                    .await
                    .map_err(|error| match error {
                        InlineError::NoVerdict => InlineError::Contended,
                        other => other,
                    });
            }
            Claim::Gone => {}
        }
    }
    let matching = matcher::Config::default();
    let matched = matcher::match_component(db, &matching, component_id).await?;
    matcher::fan_out(db, &matching, &matched.marked).await;
    mark(db, repository, component_id).await?;
    verdict::judge_subject(
        db,
        cache,
        &verdict::Config::default(),
        repository,
        &subject::component(component_id),
    )
    .await?;
    wait_for_verdict(db, repository, component_id, since).await
}

async fn database_time(db: &DatabaseConnection) -> Result<DateTime<Utc>, DbErr> {
    db.query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT clock_timestamp() AS at",
    ))
    .await?
    .ok_or_else(|| DbErr::RecordNotFound("the clock query returned no row".to_owned()))?
    .try_get("", "at")
}

async fn scanned(db: &DatabaseConnection, component_id: i64) -> Result<bool, DbErr> {
    Ok(db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 AS found FROM component_scan WHERE component_id = $1",
            [component_id.into()],
        ))
        .await?
        .is_some())
}

async fn wait_for_verdict(
    db: &DatabaseConnection,
    repository: &str,
    component_id: i64,
    since: DateTime<Utc>,
) -> Result<(), InlineError> {
    match tokio::time::timeout(WAIT, poll_verdict(db, repository, component_id, since)).await {
        Ok(found) => found,
        Err(_) => Err(InlineError::NoVerdict),
    }
}

async fn poll_verdict(
    db: &DatabaseConnection,
    repository: &str,
    component_id: i64,
    since: DateTime<Utc>,
) -> Result<(), InlineError> {
    let subject = subject::component(component_id);
    loop {
        let found = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT 1 AS found FROM policy_verdict \
                 WHERE repository = $1 AND subject = $2 AND computed_at >= $3",
                [repository.into(), subject.clone().into(), since.into()],
            ))
            .await?;
        if found.is_some() {
            return Ok(());
        }
        tokio::time::sleep(POLL).await;
    }
}
