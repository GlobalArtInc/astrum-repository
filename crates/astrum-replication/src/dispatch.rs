use std::sync::Arc;
use std::time::Duration;

use astrum_content::Content;
use astrum_db::setting;
use astrum_security::ssrf::{Guard, SSRF_KEY, SsrfProtection};
use sea_orm::DbErr;
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Postgres};
use tokio::sync::OnceCell;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::error::backoff;
use crate::queue::{self, Finished, Item, RemoteRow};
use crate::work::{Done, Failure};
use crate::{Client, Remote};

pub const LOCK_KEY: i64 = i64::from_be_bytes(*b"astrrepl");

#[derive(Clone, Debug)]
pub struct Settings {
    pub lease: Duration,
    pub tick: Duration,
    pub idle: Duration,
    pub heartbeat: Duration,
    pub per_worker: usize,
    pub max_remotes: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lease: Duration::from_secs(600),
            tick: Duration::from_secs(5),
            idle: Duration::from_secs(10),
            heartbeat: Duration::from_secs(15),
            per_worker: 20,
            max_remotes: 256,
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
}

impl Pass {
    fn merge(&mut self, other: Pass) {
        self.expired += other.expired;
        self.leased += other.leased;
        self.sent += other.sent;
        self.dropped += other.dropped;
        self.failed += other.failed;
        self.dead += other.dead;
        self.again += other.again;
        self.abandoned += other.abandoned;
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
            if let Err(error) = sqlx::query("SELECT 1").execute(&mut *self.connection).await {
                return DbErr::Custom(error.to_string());
            }
        }
    }
}

pub(crate) struct Target {
    pub remote: RemoteRow,
    pub client: Result<Client, String>,
    pub learned: OnceCell<Uuid>,
}

impl Target {
    pub(crate) fn client(&self) -> Result<&Client, Failure> {
        self.client
            .as_ref()
            .map_err(|reason| Failure::Local(reason.clone()))
    }
}

pub(crate) struct Inner {
    pub content: Content,
    pub settings: Settings,
    pub own: OnceCell<Uuid>,
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
        Self {
            inner: Arc::new(Inner {
                content,
                settings,
                own: OnceCell::new(),
            }),
        }
    }

    pub(crate) fn content(&self) -> &Content {
        &self.inner.content
    }

    pub(crate) fn settings(&self) -> &Settings {
        &self.inner.settings
    }

    pub async fn instance(&self) -> Result<Uuid, DbErr> {
        self.inner
            .own
            .get_or_try_init(|| queue::own_instance(self.content().db()))
            .await
            .copied()
    }

    pub fn spawn(self) -> JoinHandle<()> {
        tokio::spawn(self.serve(CancellationToken::new()))
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
            tokio::select! {
                () = stop.cancelled() => return,
                () = tokio::time::sleep(self.settings().idle) => {}
            }
        }
    }

    async fn hold(&self, mut lock: Lock, stop: &CancellationToken) {
        tokio::select! {
            () = self.pump(stop) => {
                if let Err(error) = lock.release().await {
                    tracing::warn!(%error, "the replication lock was not released");
                }
            }
            error = lock.lost(self.settings().heartbeat) => {
                tracing::warn!(%error, "the replication lock was lost; the workers stop");
            }
        }
    }

    async fn pump(&self, stop: &CancellationToken) {
        loop {
            let busy = tokio::select! {
                () = stop.cancelled() => return,
                pass = self.run_once() => match pass {
                    Ok(pass) => pass.leased > 0,
                    Err(error) => {
                        tracing::warn!(%error, "a replication pass failed");
                        false
                    }
                },
            };
            if busy {
                continue;
            }
            tokio::select! {
                () = stop.cancelled() => return,
                () = tokio::time::sleep(self.settings().tick) => {}
            }
        }
    }

    pub async fn run_once(&self) -> Result<Pass, DbErr> {
        let db = self.content().db();
        let mut pass = Pass {
            expired: queue::requeue_expired(db).await?,
            ..Pass::default()
        };
        let remotes = queue::remotes_due(db, self.settings().max_remotes).await?;
        let mut serving = JoinSet::new();
        for remote in remotes {
            let dispatcher = self.clone();
            serving.spawn(async move { dispatcher.serve_remote(remote).await });
        }
        while let Some(joined) = serving.join_next().await {
            match joined {
                Ok(part) => pass.merge(part),
                Err(error) => tracing::error!(%error, "a replication remote task ended abnormally"),
            }
        }
        Ok(pass)
    }

    async fn serve_remote(&self, remote: RemoteRow) -> Pass {
        let workers = usize::try_from(remote.max_concurrency)
            .unwrap_or(1)
            .clamp(1, 16);
        let client = self
            .client_for(&remote)
            .await
            .map_err(|failure| failure.text());
        let target = Arc::new(Target {
            remote,
            client,
            learned: OnceCell::new(),
        });
        let mut pool = JoinSet::new();
        for _ in 0..workers {
            let (dispatcher, target) = (self.clone(), target.clone());
            pool.spawn(async move { dispatcher.work(&target).await });
        }
        let mut pass = Pass::default();
        while let Some(joined) = pool.join_next().await {
            match joined {
                Ok(part) => pass.merge(part),
                Err(error) => tracing::error!(%error, "a replication worker ended abnormally"),
            }
        }
        pass
    }

    async fn client_for(&self, remote: &RemoteRow) -> Result<Client, Failure> {
        let content = self.content();
        let token = content.secrets().open(&remote.token).map_err(|_| {
            Failure::Local(
                "the stored token cannot be opened with the key of this server".to_owned(),
            )
        })?;
        let snapshot = content
            .outbound()
            .snapshot()
            .await
            .map_err(|error| Failure::Local(error.to_string()))?;
        let ssrf: SsrfProtection = setting::get(content.db(), SSRF_KEY)
            .await
            .map_err(|error| Failure::Local(error.to_string()))?
            .unwrap_or_default();
        let guard = Guard::new(&snapshot.guard_settings(&ssrf));
        let settings = Remote {
            url: remote.url.clone(),
            use_trust_store: remote.use_trust_store,
            allow_private_network: remote.allow_private_network,
            max_concurrency: usize::try_from(remote.max_concurrency).unwrap_or(1),
            max_bytes_per_second: remote
                .max_bytes_per_second
                .and_then(|rate| u64::try_from(rate).ok()),
        };
        let own = self.instance().await?;
        Ok(
            Client::new(&settings, &token, &snapshot, content.secrets(), &guard)?
                .with_instance(&own.to_string())?,
        )
    }

    async fn work(&self, target: &Target) -> Pass {
        let mut pass = Pass::default();
        for _ in 0..self.settings().per_worker {
            match self.next(target, &mut pass).await {
                Ok(true) => {}
                Ok(false) => break,
                Err(error) => {
                    tracing::warn!(%error, "a replication worker stopped on a database error");
                    break;
                }
            }
        }
        pass
    }

    async fn next(&self, target: &Target, pass: &mut Pass) -> Result<bool, DbErr> {
        let db = self.content().db();
        let mut leased = queue::lease(db, target.remote.id, self.settings().lease, 1).await?;
        let Some(item) = leased.pop() else {
            return Ok(false);
        };
        pass.leased += 1;
        match self.transfer(target, &item).await {
            Ok(Done::Lost) => pass.abandoned += 1,
            Ok(Done::Released) => queue::release(db, item.id).await?,
            Ok(done) => self.complete(&item, done, pass).await?,
            Err(failure) => self.abandon(&item, failure, pass).await?,
        }
        Ok(true)
    }

    async fn complete(&self, item: &Item, done: Done, pass: &mut Pass) -> Result<(), DbErr> {
        let db = self.content().db();
        match done {
            Done::Sent | Done::Deleted => {
                pass.sent += 1;
                queue::succeeded(db, item.replication_id).await?;
            }
            Done::Dropped(reason) => {
                pass.dropped += 1;
                tracing::debug!(replication = %item.replication_id, reason, "a replication item was dropped");
            }
            Done::Lost | Done::Released => {}
        }
        if queue::finish(db, item).await? == Finished::Again {
            pass.again += 1;
        }
        Ok(())
    }

    async fn abandon(&self, item: &Item, failure: Failure, pass: &mut Pass) -> Result<(), DbErr> {
        let db = self.content().db();
        let attempts = u32::try_from(item.attempts).unwrap_or(0);
        let delay = failure
            .retry_after()
            .map_or_else(|| backoff(attempts), |after| backoff(attempts).max(after));
        let text = failure.text();
        let failed = queue::fail(db, item, failure.permanent(), delay, &text).await?;
        queue::failed(db, item.replication_id, &text).await?;
        pass.failed += 1;
        if failed.is_some_and(|failed| failed.state == "dead") {
            pass.dead += 1;
            tracing::warn!(
                replication = %item.replication_id,
                component = %crate::protocol::component_key(&item.namespace, &item.name, &item.version),
                error = %text,
                "a replication item is dead"
            );
        }
        Ok(())
    }
}
