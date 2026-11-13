use std::time::Duration;

use astrum_content::usage;
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::count;
use crate::kind::{Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "repository.usage.snapshot";
pub const DAYS_KEPT: &str = "usage.history.daysKept";
const DEFAULT_DAYS_KEPT: u32 = 730;
const AVAILABLE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct UsageSnapshot;

#[async_trait]
impl TaskKind for UsageSnapshot {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Record repository usage"
    }

    fn singleton(&self) -> bool {
        true
    }

    fn describe(&self, _properties: &Properties) -> String {
        "records yesterday's usage of every repository and blob store, and deletes the records older than the setting usage.history.daysKept".to_owned()
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let day = Utc::now().date_naive() - ChronoDuration::days(1);
        let mut failed = Vec::new();
        let repositories = repository_names(run).await?;
        for repository in &repositories {
            run.check()?;
            if let Err(error) = record_repository(run, repository, day).await {
                run.warn(format!("{repository}: the usage was not recorded: {error}"));
                failed.push(repository.as_str());
            }
        }
        let recorded = repositories.len() - failed.len();
        let stores = record_blob_stores(run, day).await?;
        let pruned = prune(run).await?;
        run.count("repositories", recorded as i64);
        run.count("blobStores", stores as i64);
        run.count("pruned", pruned as i64);
        if !failed.is_empty() {
            return Err(TaskError::failed(format!(
                "the usage of {} could not be recorded: {}",
                count(failed.len() as i64, "repository", "repositories"),
                failed.join(", ")
            )));
        }
        Ok(format!(
            "recorded {} and {} for {day}, deleted {}",
            count(recorded as i64, "repository", "repositories"),
            count(stores as i64, "blob store", "blob stores"),
            count(pruned as i64, "old row", "old rows")
        ))
    }
}

async fn repository_names(run: &Run) -> Result<Vec<String>, TaskError> {
    run.db()
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT name FROM repository ORDER BY name",
        ))
        .await?
        .iter()
        .map(|row| row.try_get::<String>("", "name").map_err(TaskError::from))
        .collect()
}

async fn record_repository(run: &Run, repository: &str, day: NaiveDate) -> Result<(), TaskError> {
    let Some(reconciled) = usage::reconcile(run.db(), repository).await? else {
        return Ok(());
    };
    run.db()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO repository_usage_day (repository, day, components, assets, bytes, downloads) \
             VALUES ($1, $2, \
                 (SELECT count(DISTINCT component_id) FROM asset WHERE repository = $1), \
                 $3, $4, \
                 (SELECT coalesce(sum(d.downloads), 0)::bigint FROM asset_download_day d \
                     JOIN asset a ON a.id = d.asset_id WHERE a.repository = $1 AND d.day = $2)) \
             ON CONFLICT (repository, day) DO UPDATE SET components = EXCLUDED.components, \
                 assets = EXCLUDED.assets, bytes = EXCLUDED.bytes, downloads = EXCLUDED.downloads",
            [
                repository.into(),
                day.into(),
                reconciled.usage.assets.into(),
                reconciled.usage.bytes.into(),
            ],
        ))
        .await?;
    Ok(())
}

async fn record_blob_stores(run: &Run, day: NaiveDate) -> Result<usize, TaskError> {
    let configs = run.content().blob_stores().configs().await?;
    let mut recorded = 0;
    for config in configs {
        run.check()?;
        let totals = match run
            .content()
            .blob_stores()
            .metrics(&config.name)
            .totals()
            .await
        {
            Ok(totals) => totals,
            Err(error) => {
                run.warn(format!(
                    "{}: the totals were not recorded: {error}",
                    config.name
                ));
                continue;
            }
        };
        let available = available_space(run, &config.name).await;
        run.db()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO blob_store_usage_day (store, day, blob_count, total_bytes, available_bytes) \
                 VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (store, day) DO UPDATE SET blob_count = EXCLUDED.blob_count, \
                     total_bytes = EXCLUDED.total_bytes, available_bytes = EXCLUDED.available_bytes",
                [
                    config.name.as_str().into(),
                    day.into(),
                    (totals.blob_count as i64).into(),
                    (totals.total_bytes as i64).into(),
                    available.into(),
                ],
            ))
            .await?;
        recorded += 1;
    }
    Ok(recorded)
}

async fn available_space(run: &Run, store: &str) -> Option<i64> {
    let measured = tokio::time::timeout(AVAILABLE_TIMEOUT, async {
        let blob_store = run.content().blob_stores().get(store).await?;
        blob_store.available_space().await
    })
    .await;
    match measured {
        Ok(Ok(available)) => available.and_then(|bytes| i64::try_from(bytes).ok()),
        Ok(Err(error)) => {
            run.warn(format!("{store}: the free space was not recorded: {error}"));
            None
        }
        Err(_) => {
            run.warn(format!(
                "{store}: the free space did not answer in {} seconds",
                AVAILABLE_TIMEOUT.as_secs()
            ));
            None
        }
    }
}

async fn prune(run: &Run) -> Result<u64, TaskError> {
    let kept = astrum_db::setting::get::<u32>(run.db(), DAYS_KEPT)
        .await
        .map_err(TaskError::failed)?
        .unwrap_or(DEFAULT_DAYS_KEPT)
        .max(1);
    let today = Utc::now().date_naive();
    let Some(cutoff) = today.checked_sub_signed(ChronoDuration::days(i64::from(kept))) else {
        return Ok(0);
    };
    let mut deleted = 0;
    for table in ["repository_usage_day", "blob_store_usage_day"] {
        deleted += run
            .db()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                format!("DELETE FROM {table} WHERE day < $1"),
                [cutoff.into()],
            ))
            .await?
            .rows_affected();
    }
    Ok(deleted)
}
