mod metrics;
mod store;
mod sweeper;

use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_db::setting;
use astrum_policy::cache::Cache;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use tokio::task::JoinHandle;

pub use metrics::{Metrics, STATES, Snapshot};
pub use store::{Judged, judge_batch, judge_batch_with, judge_subject};
pub use sweeper::{State as SweepState, sweep};

pub const WORKERS_KEY: &str = "scan.verdictWorkers";
pub const DEFAULT_WORKERS: usize = 1;
pub const MAX_WORKERS: usize = 4;
pub const BATCH: i64 = 100;
pub const CHUNK: i64 = 5000;
pub const IDLE: Duration = Duration::from_secs(5);
pub const CONTENDED: Duration = Duration::from_millis(100);
pub const SWEEP_EVERY: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub struct Config {
    pub workers: usize,
    pub batch: i64,
    pub chunk: i64,
    pub idle: Duration,
    pub contended: Duration,
    pub sweep_every: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workers: DEFAULT_WORKERS,
            batch: BATCH,
            chunk: CHUNK,
            idle: IDLE,
            contended: CONTENDED,
            sweep_every: SWEEP_EVERY,
        }
    }
}

impl Config {
    pub async fn load(db: &DatabaseConnection) -> Self {
        let mut config = Self::default();
        match setting::get::<usize>(db, WORKERS_KEY).await {
            Ok(Some(workers)) => config.workers = workers.min(MAX_WORKERS),
            Ok(None) => {}
            Err(error) => tracing::warn!(
                %error,
                default = DEFAULT_WORKERS,
                "the setting {WORKERS_KEY} is not a number of workers, so the default is used"
            ),
        }
        config
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Swept {
    Busy,
    Done { marked: u64 },
}

pub struct Verdicts {
    handles: Vec<JoinHandle<()>>,
}

impl Verdicts {
    pub fn count(&self) -> usize {
        self.handles.len()
    }

    pub fn abort(&self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

pub fn spawn(
    db: DatabaseConnection,
    config: Config,
    cache: Arc<Cache>,
    metrics: Arc<Metrics>,
) -> Verdicts {
    let mut handles = Vec::new();
    match config.workers {
        0 => tracing::info!("verdict workers disabled ({WORKERS_KEY} = 0)"),
        workers => {
            tracing::info!(workers, "verdict workers started");
            for _ in 0..workers {
                handles.push(tokio::spawn(run(
                    db.clone(),
                    config.clone(),
                    cache.clone(),
                    metrics.clone(),
                )));
            }
            handles.push(tokio::spawn(run_sweeper(db, config, metrics)));
        }
    }
    Verdicts { handles }
}

async fn run(db: DatabaseConnection, config: Config, cache: Arc<Cache>, metrics: Arc<Metrics>) {
    loop {
        let pause = step(&db, &cache, &config, &metrics).await;
        if !pause.is_zero() {
            tokio::time::sleep(pause).await;
        }
    }
}

pub async fn step(
    db: &DatabaseConnection,
    cache: &Cache,
    config: &Config,
    metrics: &Metrics,
) -> Duration {
    let started = Instant::now();
    match judge_batch(db, cache, config).await {
        Ok(judged) => {
            if judged.taken > 0 {
                metrics.observe(started.elapsed().as_secs_f64());
            }
            metrics.record(&judged);
            match (judged.taken > 0, judged.busy) {
                (true, _) => Duration::ZERO,
                (false, true) => config.contended,
                (false, false) => config.idle,
            }
        }
        Err(error) => {
            metrics.failed();
            tracing::warn!(%error, "a batch of policy marks was not judged and stays queued");
            config.idle
        }
    }
}

async fn run_sweeper(db: DatabaseConnection, config: Config, metrics: Arc<Metrics>) {
    let mut state = SweepState::default();
    loop {
        match sweep(&db, &config, &mut state).await {
            Ok(swept) => metrics.swept(&swept),
            Err(error) => {
                metrics.sweep_failed();
                tracing::warn!(%error, "the sweep for expired exceptions and changed policies failed; it repeats");
            }
        }
        tokio::time::sleep(config.sweep_every).await;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Depth {
    pub queued: i64,
    pub oldest_seconds: f64,
}

pub async fn depth(db: &impl ConnectionTrait) -> Result<Depth, DbErr> {
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS queued, \
               COALESCE(EXTRACT(EPOCH FROM now() - min(queued_at)), 0)::float8 AS oldest \
             FROM policy_dirty",
        ))
        .await?
        .ok_or_else(|| {
            DbErr::RecordNotFound("the verdict queue query returned no row".to_owned())
        })?;
    Ok(Depth {
        queued: row.try_get("", "queued")?,
        oldest_seconds: row.try_get("", "oldest")?,
    })
}
