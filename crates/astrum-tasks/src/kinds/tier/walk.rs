use std::collections::HashMap;
use std::time::Duration;

use astrum_blob::{BlobId, Limiter};
use astrum_content::store::Asset;
use astrum_content::tiering::{Options, Outcome, Snapshot, move_asset};
use sea_orm::{DbBackend, FromQueryResult, Statement};

use crate::kind::TaskError;
use crate::run::Run;

const BATCH: i64 = 200;
const HOLD_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_DUPLICATES: usize = 100_000;
const MAX_FAILURES: u64 = 50;

const SELECT: &str = "SELECT * FROM asset WHERE id > $1 AND blob_store = ANY($2) \
     AND GREATEST(blob_updated_at, coalesce(last_downloaded_at, '-infinity')) < now() - make_interval(days => $3) \
     AND (component_id IS NOT NULL OR (format = 'docker' AND kind = 'blob')) \
     AND size >= $4 AND ($5::text IS NULL OR repository = $5) \
     ORDER BY id LIMIT $6";

pub struct Plan {
    pub source: String,
    pub target: String,
    pub stores: Vec<String>,
    pub repository: Option<String>,
    pub idle_days: i32,
    pub min_size: i64,
    pub bytes_limit: u64,
    pub assets_limit: u64,
    pub limiter: Option<Limiter>,
    pub drain: Duration,
    pub dry_run: bool,
}

#[derive(Debug, Default)]
pub struct Totals {
    pub candidates: u64,
    pub moved: u64,
    pub bytes: u64,
    pub changed: u64,
    pub corrupt: u64,
    pub failed: u64,
}

pub fn reached(assets_limit: u64, bytes_limit: u64, totals: &Totals) -> Option<&'static str> {
    if assets_limit > 0 && totals.moved >= assets_limit {
        return Some("the asset limit");
    }
    if bytes_limit > 0 && totals.bytes >= bytes_limit {
        return Some("the byte limit");
    }
    None
}

pub async fn walk(run: &Run, plan: &Plan, totals: &mut Totals) -> Result<(), TaskError> {
    let mut duplicates = Duplicates::default();
    let mut streak = 0;
    let mut after = 0;
    'pages: loop {
        run.check()?;
        let found = page(run, plan, after).await?;
        let Some(last) = found.last() else {
            break;
        };
        after = last.id;
        for asset in &found {
            run.check()?;
            if let Some(limit) = reached(plan.assets_limit, plan.bytes_limit, totals) {
                run.info(format!(
                    "stopped at {limit}; the rest waits for the next run"
                ));
                break 'pages;
            }
            totals.candidates += 1;
            if plan.dry_run {
                totals.moved += 1;
                totals.bytes += bytes_of(asset);
                continue;
            }
            let options = Options {
                drain: plan.drain,
                hold_ttl: HOLD_TTL,
                limiter: plan.limiter.clone(),
                duplicate_of: duplicates.get(asset),
            };
            match move_asset(run.content(), Snapshot::from(asset), &plan.target, &options).await {
                Outcome::Moved { new_id, bytes } => {
                    duplicates.remember(asset, new_id);
                    totals.moved += 1;
                    totals.bytes += bytes;
                    streak = 0;
                }
                Outcome::Changed => {
                    totals.changed += 1;
                    streak = 0;
                }
                Outcome::Corrupt => {
                    run.warn(format!(
                        "asset {} of {} is corrupt at its source and was not moved",
                        asset.id, asset.repository
                    ));
                    totals.corrupt += 1;
                    streak = 0;
                }
                Outcome::Failed(message) => {
                    run.warn(format!(
                        "asset {} of {} did not move: {message}",
                        asset.id, asset.repository
                    ));
                    totals.failed += 1;
                    streak += 1;
                    if streak > MAX_FAILURES {
                        return Err(TaskError::Failed(format!(
                            "stopped after {streak} failed moves in a row; the target {} may be down; the last error: {message}",
                            plan.target
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

async fn page(run: &Run, plan: &Plan, after: i64) -> Result<Vec<Asset>, TaskError> {
    Ok(Asset::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        SELECT,
        [
            after.into(),
            plan.stores.clone().into(),
            plan.idle_days.into(),
            plan.min_size.into(),
            plan.repository.clone().into(),
            BATCH.into(),
        ],
    ))
    .all(run.db())
    .await?)
}

fn bytes_of(asset: &Asset) -> u64 {
    asset.size.max(0) as u64
}

#[derive(Default)]
struct Duplicates(HashMap<String, (BlobId, u64)>);

impl Duplicates {
    fn get(&self, asset: &Asset) -> Option<BlobId> {
        self.0
            .get(&asset.sha256.to_ascii_lowercase())
            .filter(|(_, size)| *size == bytes_of(asset))
            .map(|(id, _)| *id)
    }

    fn remember(&mut self, asset: &Asset, id: BlobId) {
        if self.0.len() < MAX_DUPLICATES {
            self.0
                .entry(asset.sha256.to_ascii_lowercase())
                .or_insert((id, bytes_of(asset)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_limits_mean_none_and_a_limit_stops_once_reached() {
        let mut totals = Totals::default();
        assert_eq!(reached(0, 0, &totals), None);
        totals.moved = 2;
        totals.bytes = 10;
        assert_eq!(reached(3, 0, &totals), None);
        assert_eq!(reached(2, 0, &totals), Some("the asset limit"));
        assert_eq!(reached(0, 10, &totals), Some("the byte limit"));
    }
}
