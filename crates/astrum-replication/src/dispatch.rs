use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use astrum_content::Content;
use sea_orm::DbErr;
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Postgres};
use tokio::sync::{OnceCell, Semaphore};
use tokio::task::{JoinHandle, JoinSet};
pub use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::counters::Counters;
use crate::gate::Gated;
use crate::queue::{self, RemoteRow};

pub const LOCK_KEY: i64 = i64::from_be_bytes(*b"astrrepl");
pub const MAX_WORKERS: usize = 32;
pub const API_CONNECTIONS: u32 = 5;
const LOST_SLACK: Duration = Duration::from_secs(1);

pub fn permits(max_connections: u32, cap: usize) -> usize {
    let spare = max_connections.saturating_sub(API_CONNECTIONS + 1);
    usize::try_from(spare).unwrap_or(usize::MAX).clamp(1, cap)
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub lease: Duration,
    pub tick: Duration,
    pub idle: Duration,
    pub heartbeat: Duration,
    pub discover: Duration,
    pub alerts: Duration,
    pub fresh: Duration,
    pub grace: Duration,
    pub keepalive: Duration,
    pub idle_max: Duration,
    pub batch: usize,
    pub rounds: usize,
    pub max_remotes: i64,
    pub workers: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lease: Duration::from_secs(600),
            tick: Duration::from_secs(5),
            idle: Duration::from_secs(10),
            heartbeat: Duration::from_secs(15),
            discover: Duration::from_secs(30),
            alerts: Duration::from_secs(60),
            fresh: Duration::from_secs(1),
            grace: Duration::from_secs(10),
            keepalive: Duration::from_secs(120),
            idle_max: Duration::from_secs(30),
            batch: 20,
            rounds: 10,
            max_remotes: 256,
            workers: MAX_WORKERS,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pass {
    pub expired: u64,
    pub leased: u32,
    pub sent: u32,
    pub dropped: u32,
    pub failed: u32,
    pub dead: u32,
    pub again: u32,
    pub abandoned: u32,
    pub released: u32,
    pub suspended: u32,
}

impl Pass {
    pub(crate) fn merge(&mut self, other: Pass) {
        self.expired += other.expired;
        self.leased += other.leased;
        self.sent += other.sent;
        self.dropped += other.dropped;
        self.failed += other.failed;
        self.dead += other.dead;
        self.again += other.again;
        self.abandoned += other.abandoned;
        self.released += other.released;
        self.suspended += other.suspended;
    }
}

pub struct Lock {
    connection: PoolConnection<Postgres>,
}

impl Lock {
    pub async fn try_take(content: &Content) -> Result<Option<Self>, DbErr> {
        let mut connection = content
            .db()
            .get_postgres_connection_pool()
            .acquire()
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        connection.close_on_drop();
        let (taken,): (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
            .bind(LOCK_KEY)
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        Ok(taken.then_some(Self { connection }))
    }

    pub async fn release(mut self) -> Result<(), DbErr> {
        sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(LOCK_KEY)
            .execute(&mut *self.connection)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        Ok(())
    }

    async fn lost(&mut self, every: Duration) -> DbErr {
        loop {
            tokio::time::sleep(every).await;
            let beat = sqlx::query("SELECT 1").execute(&mut *self.connection);
            match tokio::time::timeout(every, beat).await {
                Ok(Ok(_)) => {}
                Ok(Err(error)) => return DbErr::Custom(error.to_string()),
                Err(_) => return DbErr::Custom("the heartbeat of the lock timed out".to_owned()),
            }
        }
    }
}

pub(crate) struct Inner {
    pub content: Content,
    pub settings: Settings,
    pub own: OnceCell<Uuid>,
    pub gate: Semaphore,
    pub leading: AtomicBool,
    pub counters: Arc<Counters>,
}

#[derive(Clone)]
pub struct Dispatcher {
    pub(crate) inner: Arc<Inner>,
}

impl Dispatcher {
    pub fn new(content: Content) -> Self {
        Self::with_settings(content, Settings::default())
    }

    pub fn with_settings(content: Content, settings: Settings) -> Self {
        Self::with_counters(content, settings, Arc::default())
    }

    pub fn with_counters(content: Content, settings: Settings, counters: Arc<Counters>) -> Self {
        let connections = content
            .db()
            .get_postgres_connection_pool()
            .options()
            .get_max_connections();
        Self {
            inner: Arc::new(Inner {
                gate: Semaphore::new(permits(connections, settings.workers)),
                content,
                settings,
                own: OnceCell::new(),
                leading: AtomicBool::new(false),
                counters,
            }),
        }
    }

    pub(crate) fn content(&self) -> &Content {
        &self.inner.content
    }

    pub(crate) fn db(&self) -> Gated<'_> {
        Gated::new(self.inner.content.db(), &self.inner.gate)
    }

    pub(crate) fn settings(&self) -> &Settings {
        &self.inner.settings
    }

    pub fn counters(&self) -> &Arc<Counters> {
        &self.inner.counters
    }

    pub fn leading(&self) -> bool {
        self.inner.leading.load(Ordering::SeqCst)
    }

    pub async fn instance(&self) -> Result<Uuid, DbErr> {
        self.inner
            .own
            .get_or_try_init(|| async { queue::own_instance(&self.db()).await })
            .await
            .copied()
    }

    pub fn spawn(self) -> JoinHandle<()> {
        self.spawn_until(CancellationToken::new())
    }

    pub fn spawn_until(self, stop: CancellationToken) -> JoinHandle<()> {
        tokio::spawn(self.serve(stop))
    }

    pub async fn serve(self, stop: CancellationToken) {
        while !stop.is_cancelled() {
            match Lock::try_take(self.content()).await {
                Ok(Some(lock)) => self.hold(lock, &stop).await,
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(%error, "the replication lock could not be asked for");
                }
            }
            self.nap(&stop, self.settings().idle).await;
        }
    }

    pub(crate) async fn nap(&self, stop: &CancellationToken, time: Duration) {
        tokio::select! {
            () = stop.cancelled() => {}
            () = tokio::time::sleep(time) => {}
        }
    }

    async fn hold(&self, mut lock: Lock, stop: &CancellationToken) {
        let leading = stop.child_token();
        let mut running = std::pin::pin!(self.lead(leading.clone()));
        self.inner.leading.store(true, Ordering::SeqCst);
        tokio::select! {
            () = &mut running => {
                if let Err(error) = lock.release().await {
                    tracing::warn!(%error, "the replication lock was not released");
                }
            }
            error = lock.lost(self.settings().heartbeat) => {
                tracing::warn!(%error, "the replication lock was lost; the workers stop");
                leading.cancel();
                let grace = self.settings().grace + LOST_SLACK;
                if tokio::time::timeout(grace, &mut running).await.is_err() {
                    tracing::warn!("the replication workers did not stop in time and were dropped");
                }
            }
        }
        self.inner.leading.store(false, Ordering::SeqCst);
    }

    pub async fn run_once(&self) -> Result<Pass, DbErr> {
        let db = &self.db();
        let mut pass = Pass {
            expired: queue::requeue_expired(db).await?,
            ..Pass::default()
        };
        let remotes = queue::remotes_due(db, self.settings().max_remotes).await?;
        let mut serving = JoinSet::new();
        for remote in remotes {
            let dispatcher = self.clone();
            serving.spawn(async move { dispatcher.round_of(remote).await });
        }
        while let Some(joined) = serving.join_next().await {
            match joined {
                Ok(part) => pass.merge(part),
                Err(error) => tracing::error!(%error, "a replication remote task ended abnormally"),
            }
        }
        Ok(pass)
    }

    async fn round_of(&self, remote: RemoteRow) -> Pass {
        let workers = Self::workers_of(&remote);
        let target = Arc::new(self.target(remote).await);
        let stop = CancellationToken::new();
        let mut pool = JoinSet::new();
        for _ in 0..workers {
            let (dispatcher, target, stop) = (self.clone(), target.clone(), stop.clone());
            pool.spawn(async move {
                let mut pass = Pass::default();
                for _ in 0..dispatcher.settings().rounds {
                    let part = dispatcher.drain(&target, &stop).await;
                    let idle = part.leased == 0;
                    pass.merge(part);
                    if idle {
                        break;
                    }
                }
                pass
            });
        }
        let mut pass = Pass::default();
        while let Some(joined) = pool.join_next_with_id().await {
            match joined {
                Ok((_, part)) => pass.merge(part),
                Err(error) => self.crashed(&target, error).await,
            }
        }
        pass
    }

    pub(crate) fn workers_of(remote: &RemoteRow) -> usize {
        usize::try_from(remote.max_concurrency)
            .unwrap_or(1)
            .clamp(1, 16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workers_leave_connections_to_the_api() {
        assert_eq!(permits(100, MAX_WORKERS), MAX_WORKERS);
        assert_eq!(permits(20, MAX_WORKERS), 14);
        assert_eq!(permits(10, MAX_WORKERS), 4);
        assert_eq!(permits(6, MAX_WORKERS), 1);
        assert_eq!(permits(1, MAX_WORKERS), 1);
        assert_eq!(permits(100, 3), 3);
    }
}
