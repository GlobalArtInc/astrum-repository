mod metrics;
mod store;

use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_content::{Content, ContentError};
use astrum_db::setting;
use astrum_security::SecurityModel;
use chrono::{DateTime, Utc};
use sea_orm::DbErr;
use tokio::task::JoinHandle;

pub use metrics::{BUCKETS, Metrics, Snapshot};
pub use store::{Claim, claim_component, queue_component};

pub const EXTRACTOR: i32 = 1;
const FORMAT_EXTRACTORS: [(&str, i32); 1] = [("docker", 2)];

pub fn extractor_of(format: &str) -> i32 {
    FORMAT_EXTRACTORS
        .iter()
        .find(|(name, _)| *name == format)
        .map_or(EXTRACTOR, |(_, version)| *version)
}

pub fn extractor_case(alias: &str) -> String {
    let arms: String = FORMAT_EXTRACTORS
        .iter()
        .map(|(name, version)| format!("WHEN {alias}.format = '{name}' THEN {version} "))
        .collect();
    format!("CASE {arms}ELSE {EXTRACTOR} END")
}
pub const WORKERS_KEY: &str = "scan.workers";
pub const DEFAULT_WORKERS: usize = 2;
pub const MAX_WORKERS: usize = 8;
pub const BATCH: i64 = 10;
pub const IDLE: Duration = Duration::from_secs(5);
pub const TIMEOUT: Duration = Duration::from_secs(600);
pub const LEASE_SLACK: Duration = Duration::from_secs(60);
pub const BACKOFF: [Duration; 3] = [
    Duration::from_secs(60),
    Duration::from_secs(300),
    Duration::from_secs(1800),
];

#[derive(Clone, Debug)]
pub struct Config {
    pub workers: usize,
    pub batch: i64,
    pub idle: Duration,
    pub timeout: Duration,
    pub backoff: Vec<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workers: DEFAULT_WORKERS,
            batch: BATCH,
            idle: IDLE,
            timeout: TIMEOUT,
            backoff: BACKOFF.to_vec(),
        }
    }
}

impl Config {
    pub async fn load(db: &sea_orm::DatabaseConnection) -> Self {
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

    fn lease(&self) -> Duration {
        self.timeout + LEASE_SLACK
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Scanned,
    Incomplete,
    Unsupported,
    Failed,
    Gone,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Scanned => "scanned",
            State::Incomplete => "incomplete",
            State::Unsupported => "unsupported",
            State::Failed => "failed",
            State::Gone => "gone",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Claimed {
    pub component_id: i64,
    pub attempts: i32,
    pub queued_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("{0}")]
    Content(#[from] ContentError),
    #[error("the database failed: {0}")]
    Database(#[from] DbErr),
    #[error("{0}")]
    Stopped(String),
}

struct Abort<T>(JoinHandle<T>);

impl<T> Drop for Abort<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub struct Workers {
    handles: Vec<JoinHandle<()>>,
}

impl Workers {
    pub fn count(&self) -> usize {
        self.handles.len()
    }

    pub fn abort(&self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

pub fn spawn(content: Content, config: Config, metrics: Arc<Metrics>) -> Workers {
    match config.workers {
        0 => tracing::info!("scan workers disabled ({WORKERS_KEY} = 0)"),
        workers => tracing::info!(workers, "scan workers started"),
    }
    let security = Arc::new(SecurityModel::default());
    let handles = (0..config.workers)
        .map(|_| {
            tokio::spawn(run(
                content.clone(),
                config.clone(),
                metrics.clone(),
                security.clone(),
            ))
        })
        .collect();
    Workers { handles }
}

async fn run(
    content: Content,
    config: Config,
    metrics: Arc<Metrics>,
    security: Arc<SecurityModel>,
) {
    loop {
        if drain(&content, &config, &metrics, &security).await == 0 {
            tokio::time::sleep(config.idle).await;
        }
    }
}

pub async fn drain(
    content: &Content,
    config: &Config,
    metrics: &Metrics,
    security: &Arc<SecurityModel>,
) -> usize {
    let claimed = match store::claim(content.db(), config.batch, config.lease()).await {
        Ok(claimed) => claimed,
        Err(error) => {
            tracing::warn!(%error, "the scan queue could not be read");
            return 0;
        }
    };
    for (index, item) in claimed.iter().enumerate() {
        if index > 0
            && let Err(error) = store::renew(content.db(), &claimed[index..], config.lease()).await
        {
            tracing::warn!(%error, component = item.component_id, "the scan leases of a batch were not renewed");
        }
        handle(content, config, metrics, security, *item).await;
    }
    claimed.len()
}

async fn handle(
    content: &Content,
    config: &Config,
    metrics: &Metrics,
    security: &Arc<SecurityModel>,
    item: Claimed,
) {
    let started = Instant::now();
    let mut scan = Abort(tokio::spawn(scan(content.clone(), security.clone(), item)));
    let result = match tokio::time::timeout(config.timeout, &mut scan.0).await {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => Err(ScanError::Stopped(format!("the scan stopped: {error}"))),
        Err(_) => {
            scan.0.abort();
            match (&mut scan.0).await {
                Ok(finished) => finished,
                Err(_) => Err(ScanError::Stopped(format!(
                    "the scan took longer than {} seconds",
                    config.timeout.as_secs()
                ))),
            }
        }
    };
    metrics.observe(started.elapsed().as_secs_f64());
    let Err(error) = result else {
        return;
    };
    metrics.failed();
    let message = error.to_string();
    let outcome = match config.backoff.get(item.attempts.max(1) as usize - 1) {
        Some(delay) => {
            tracing::warn!(component = item.component_id, attempt = item.attempts, error = %message, "a scan failed and is retried");
            store::retry_later(content.db(), item, *delay).await
        }
        None => {
            tracing::error!(component = item.component_id, attempt = item.attempts, error = %message, "a scan failed for good");
            store::give_up(content.db(), item, &message).await
        }
    };
    if let Err(error) = outcome {
        tracing::error!(%error, component = item.component_id, "the outcome of a failed scan was not saved");
    }
}

pub async fn scan_claimed(
    content: &Content,
    security: &Arc<SecurityModel>,
    item: Claimed,
) -> Result<State, ScanError> {
    scan(content.clone(), security.clone(), item).await
}

async fn scan(
    content: Content,
    security: Arc<SecurityModel>,
    item: Claimed,
) -> Result<State, ScanError> {
    let db = content.db();
    let component_id = item.component_id;
    let Some(component) = astrum_content::store::component_by_id(db, component_id).await? else {
        return Ok(State::Gone);
    };
    let Some(repository) = content.repository(&component.repository).await? else {
        return Err(ScanError::Stopped(format!(
            "the repository {} of the component is missing",
            component.repository
        )));
    };
    let inventory = match content.format(&component.format) {
        Some(format) => {
            let ctx = content.context(repository, security, None, "", "");
            format.inventory(&ctx, &component).await?
        }
        None => None,
    };
    match inventory {
        None => match store::record_unsupported(db, item, &component).await? {
            true => Ok(State::Unsupported),
            false => Ok(State::Gone),
        },
        Some(mut inventory) => {
            store::sanitise(&mut inventory);
            let state = match inventory.complete {
                true => State::Scanned,
                false => State::Incomplete,
            };
            match store::record_inventory(db, item, &component, &inventory, state).await? {
                true => Ok(state),
                false => Ok(State::Gone),
            }
        }
    }
}
