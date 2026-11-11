mod evaluate;
mod metrics;
mod notices;
mod rejudge;
mod store;

use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_db::setting;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use tokio::task::JoinHandle;

pub use evaluate::{Candidate, Evaluation, Found, evaluate, resolve};
pub use metrics::{Metrics, Snapshot};
pub use notices::{NOTICE_ROWS, Queued, queue as queue_notices};
pub use rejudge::{Rejudged, fan_out, sweep};
pub use store::{Batch, Change, Marked, match_batch, match_batch_with, match_component};

pub const MATCHERS_KEY: &str = "scan.matchers";
pub const DEFAULT_MATCHERS: usize = 1;
pub const MAX_MATCHERS: usize = 4;
pub const BATCH: i64 = 500;
pub const ADVISORY_CAP: usize = 10_000;
pub const CHUNK: i64 = 5000;
pub const IDLE: Duration = Duration::from_secs(5);
pub const GRACE: Duration = Duration::from_secs(120);

#[derive(Clone, Debug)]
pub struct Config {
    pub matchers: usize,
    pub batch: i64,
    pub cap: usize,
    pub chunk: i64,
    pub idle: Duration,
    pub grace: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            matchers: DEFAULT_MATCHERS,
            batch: BATCH,
            cap: ADVISORY_CAP,
            chunk: CHUNK,
            idle: IDLE,
            grace: GRACE,
        }
    }
}

impl Config {
    pub async fn load(db: &DatabaseConnection) -> Self {
        let mut config = Self::default();
        match setting::get::<usize>(db, MATCHERS_KEY).await {
            Ok(Some(matchers)) => config.matchers = matchers.min(MAX_MATCHERS),
            Ok(None) => {}
            Err(error) => tracing::warn!(
                %error,
                default = DEFAULT_MATCHERS,
                "the setting {MATCHERS_KEY} is not a number of matchers, so the default is used"
            ),
        }
        config
    }
}

pub struct Matchers {
    handles: Vec<JoinHandle<()>>,
}

impl Matchers {
    pub fn count(&self) -> usize {
        self.handles.len()
    }

    pub fn abort(&self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

pub fn spawn(db: DatabaseConnection, config: Config, metrics: Arc<Metrics>) -> Matchers {
    match config.matchers {
        0 => tracing::info!("scan matchers disabled ({MATCHERS_KEY} = 0)"),
        matchers => tracing::info!(matchers, "scan matchers started"),
    }
    let handles = (0..config.matchers)
        .map(|_| tokio::spawn(run(db.clone(), config.clone(), metrics.clone())))
        .collect();
    Matchers { handles }
}

async fn run(db: DatabaseConnection, config: Config, metrics: Arc<Metrics>) {
    loop {
        if !step(&db, &config, &metrics).await {
            tokio::time::sleep(config.idle).await;
        }
    }
}

pub async fn step(db: &DatabaseConnection, config: &Config, metrics: &Metrics) -> bool {
    let started = Instant::now();
    let mut busy = false;
    match match_batch(db, config).await {
        Ok(batch) => {
            if batch.matched > 0 {
                metrics.observe(started.elapsed().as_secs_f64());
                busy = true;
            }
            metrics.record(&batch);
            let rejudged = fan_out(db, config, &batch.marked).await;
            metrics.failed(rejudged.failed);
        }
        Err(error) => {
            metrics.failed(1);
            tracing::warn!(%error, "a batch of packages was not matched against the advisories");
        }
    }
    match sweep(db, config).await {
        Ok(swept) => {
            metrics.failed(swept.failed);
            busy || swept.packages + swept.failed > 0
        }
        Err(error) => {
            metrics.failed(1);
            tracing::warn!(%error, "the packages waiting to be queued for judgement could not be listed");
            busy
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Backlog {
    pub unmatched: i64,
    pub rejudge: i64,
}

pub async fn backlog(db: &impl ConnectionTrait) -> Result<Backlog, DbErr> {
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT (SELECT count(*) FROM sca_package WHERE matched_at IS NULL) AS unmatched, \
                    (SELECT count(*) FROM sca_package WHERE rejudge_at IS NOT NULL) AS rejudge",
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("the backlog query returned no row".to_owned()))?;
    Ok(Backlog {
        unmatched: row.try_get("", "unmatched")?,
        rejudge: row.try_get("", "rejudge")?,
    })
}
