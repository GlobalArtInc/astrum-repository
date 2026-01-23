use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use astrum_db::audit::{self, Event};
use chrono::Utc;
use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Postgres};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, FromQueryResult, Statement};
use serde_json::json;
use tokio::sync::OwnedSemaphorePermit;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::Tasks;
use crate::kind::TaskError;
use crate::run::{INITIATOR, Run};
use crate::store::{self, RunState};

pub const POLL: Duration = Duration::from_secs(5);
pub const CANCEL_POLL: Duration = Duration::from_secs(2);
pub const LOG_FLUSH: Duration = Duration::from_secs(1);
pub const ORPHAN_CHECK: Duration = Duration::from_secs(30);
const LOCK_SPACE: i64 = i64::from_be_bytes(*b"astrtask");

pub fn lock_key(task: Uuid) -> i64 {
    let bytes = task.as_bytes();
    let mut first = [0u8; 8];
    first.copy_from_slice(&bytes[..8]);
    i64::from_be_bytes(first) ^ LOCK_SPACE
}

pub(crate) struct Lock {
    connection: PoolConnection<Postgres>,
    key: i64,
}

impl Lock {
    pub(crate) async fn try_take(tasks: &Tasks, task: Uuid) -> Result<Option<Self>, DbErr> {
        let pool = tasks.db().get_postgres_connection_pool();
        let mut connection = pool
            .acquire()
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        connection.close_on_drop();
        let key = lock_key(task);
        let (taken,): (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
            .bind(key)
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        Ok(taken.then_some(Self { connection, key }))
    }

    pub(crate) async fn release(mut self) {
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(self.key)
            .execute(&mut *self.connection)
            .await;
    }
}

#[derive(FromQueryResult)]
struct Due {
    id: Uuid,
    frequency: serde_json::Value,
    next_run_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult)]
struct Pending {
    id: i64,
    task_id: Uuid,
}

impl Tasks {
    pub fn spawn(&self) -> JoinHandle<()> {
        let tasks = self.clone();
        tokio::spawn(async move {
            let mut last_orphan_check = None::<tokio::time::Instant>;
            loop {
                if tasks.inner.stopping.load(Ordering::Relaxed) {
                    break;
                }
                let check_orphans = last_orphan_check.is_none_or(|at| at.elapsed() >= ORPHAN_CHECK);
                if check_orphans {
                    last_orphan_check = Some(tokio::time::Instant::now());
                    if let Err(error) = tasks.fail_orphans().await {
                        tracing::warn!(%error, "abandoned task runs could not be checked");
                    }
                }
                match tasks.tick().await {
                    Ok(_) => tasks.ticked(),
                    Err(error) => {
                        tracing::warn!(%error, "the task scheduler could not look for work")
                    }
                }
                tokio::select! {
                    () = tokio::time::sleep(POLL) => {}
                    () = tasks.inner.wake.notified() => {}
                }
            }
        })
    }

    pub async fn tick(&self) -> Result<Vec<JoinHandle<()>>, DbErr> {
        self.enqueue_due().await?;
        self.start_waiting().await
    }

    pub async fn run_pending(&self) -> Result<(), DbErr> {
        loop {
            let started = self.tick().await?;
            if started.is_empty() {
                return Ok(());
            }
            for handle in started {
                let _ = handle.await;
            }
        }
    }

    async fn enqueue_due(&self) -> Result<(), DbErr> {
        let due = Due::find_by_statement(Statement::from_string(
            DbBackend::Postgres,
            "SELECT id, frequency, next_run_at FROM task \
             WHERE enabled AND next_run_at IS NOT NULL AND next_run_at <= now() \
             ORDER BY next_run_at LIMIT 100",
        ))
        .all(self.db())
        .await?;
        let now = Utc::now();
        for task in due {
            let schedule = crate::schedule::Schedule::from_json(&task.frequency);
            let next = schedule.next_after(now).map(|next| next.fixed_offset());
            let advanced = self
                .db()
                .execute(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE task SET next_run_at = $2 WHERE id = $1 AND next_run_at = $3",
                    [task.id.into(), next.into(), task.next_run_at.into()],
                ))
                .await?;
            if advanced.rows_affected() == 1 {
                let requested =
                    store::request_run(self.db(), task.id, "schedule", INITIATOR).await?;
                if requested == store::Requested::AlreadyActive {
                    tracing::info!(task = %task.id, "a scheduled run was skipped because the task is still running");
                }
            }
        }
        Ok(())
    }

    async fn start_waiting(&self) -> Result<Vec<JoinHandle<()>>, DbErr> {
        let pending = Pending::find_by_statement(Statement::from_string(
            DbBackend::Postgres,
            "SELECT id, task_id FROM task_run WHERE state = 'waiting' ORDER BY queued_at, id LIMIT 50",
        ))
        .all(self.db())
        .await?;
        let mut started = Vec::new();
        for run in pending {
            if self.inner.stopping.load(Ordering::Relaxed) {
                break;
            }
            let Ok(permit) = self.inner.permits.clone().try_acquire_owned() else {
                break;
            };
            let Some(lock) = Lock::try_take(self, run.task_id).await? else {
                continue;
            };
            let claimed = self
                .db()
                .execute(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE task_run SET state = 'running', started_at = now(), node = $2 \
                     WHERE id = $1 AND state = 'waiting'",
                    [run.id.into(), self.inner.node.clone().into()],
                ))
                .await?;
            if claimed.rows_affected() != 1 {
                lock.release().await;
                continue;
            }
            started.push(self.launch(run.id, run.task_id, lock, permit));
        }
        Ok(started)
    }

    fn launch(
        &self,
        run_id: i64,
        task_id: Uuid,
        lock: Lock,
        permit: OwnedSemaphorePermit,
    ) -> JoinHandle<()> {
        let cancel = Arc::new(AtomicBool::new(false));
        self.local_runs().insert(run_id, cancel.clone());
        let tasks = self.clone();
        tokio::spawn(async move {
            let outcome = tasks.execute(run_id, task_id, cancel).await;
            if let Err(error) = outcome {
                tracing::error!(%error, run = run_id, "a task run could not be recorded");
            }
            tasks.local_runs().remove(&run_id);
            lock.release().await;
            drop(permit);
            tasks.wake();
        })
    }

    pub(crate) fn local_runs(&self) -> std::sync::MutexGuard<'_, HashMap<i64, Arc<AtomicBool>>> {
        self.inner
            .running
            .lock()
            .expect("the running task lock is never poisoned")
    }

    async fn execute(
        &self,
        run_id: i64,
        task_id: Uuid,
        cancel: Arc<AtomicBool>,
    ) -> Result<(), DbErr> {
        let started = Utc::now();
        let Some(task) = store::find(self.db(), task_id).await? else {
            return finish(
                self,
                run_id,
                RunState::Failed,
                "the task was deleted",
                json!({}),
            )
            .await;
        };
        let Some(kind) = self.kind(&task.kind) else {
            let message = format!("this server cannot run tasks of the type {}", task.kind);
            return finish(self, run_id, RunState::Failed, &message, json!({})).await;
        };
        let run = Arc::new(
            Run::new(
                run_id,
                task.id,
                task.name.clone(),
                task.kind.clone(),
                store::properties_of(&task),
                self.content().clone(),
                cancel.clone(),
            )
            .with_expiries(self.expiries()),
        );
        run.info(format!(
            "{} started: {}",
            task.name,
            kind.describe(&run.properties)
        ));
        let helper = {
            let run = run.clone();
            let db = self.db().clone();
            tokio::spawn(async move {
                let mut flushed = tokio::time::Instant::now();
                loop {
                    tokio::time::sleep(LOG_FLUSH).await;
                    run.flush().await;
                    if flushed.elapsed() >= CANCEL_POLL {
                        flushed = tokio::time::Instant::now();
                        if cancel_requested(&db, run_id).await {
                            cancel.store(true, Ordering::Relaxed);
                        }
                    }
                }
            })
        };
        let worker = {
            let run = run.clone();
            let kind = kind.clone();
            tokio::spawn(async move { kind.run(&run).await })
        };
        let result = match worker.await {
            Ok(result) => result,
            Err(error) => Err(TaskError::Failed(format!(
                "the task failed unexpectedly: {error}"
            ))),
        };
        helper.abort();
        let duration = Utc::now() - started;
        let (state, message) = match result {
            Ok(message) => (RunState::Done, message),
            Err(TaskError::Cancelled) => {
                (RunState::Cancelled, "stopped before it finished".to_owned())
            }
            Err(TaskError::Failed(message)) => (RunState::Failed, message),
        };
        match state {
            RunState::Done => run.info(format!("finished in {}: {message}", describe(duration))),
            RunState::Cancelled => run.warn(format!("stopped after {}", describe(duration))),
            _ => run.error(format!("failed after {}: {message}", describe(duration))),
        }
        run.note_dropped();
        run.flush().await;
        let counts = json!(run.counts());
        finish(self, run_id, state, &message, counts.clone()).await?;
        let kind_name = match state {
            RunState::Done => "finished",
            RunState::Cancelled => "cancelled",
            _ => "failed",
        };
        let event = Event::new("tasks", kind_name, &task.name)
            .by(INITIATOR)
            .with(json!({
                "type": task.kind,
                "run": run_id,
                "message": message,
                "counts": counts,
                "durationMs": duration.num_milliseconds(),
            }));
        if let Err(error) = audit::record(self.db(), event).await {
            tracing::error!(%error, "the end of a task run was not audited");
        }
        if let Err(error) = store::prune(self.db(), task.id).await {
            tracing::warn!(%error, "old runs of a task were not pruned");
        }
        Ok(())
    }

    pub async fn fail_orphans(&self) -> Result<u64, DbErr> {
        let running = Pending::find_by_statement(Statement::from_string(
            DbBackend::Postgres,
            "SELECT id, task_id FROM task_run WHERE state = 'running'",
        ))
        .all(self.db())
        .await?;
        let mut failed = 0;
        for run in running {
            if self.local_runs().contains_key(&run.id) {
                continue;
            }
            let Some(lock) = Lock::try_take(self, run.task_id).await? else {
                continue;
            };
            let changed = self
                .db()
                .execute(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE task_run SET state = 'failed', finished_at = now(), \
                     message = 'the server running the task stopped before it finished' \
                     WHERE id = $1 AND state = 'running'",
                    [run.id.into()],
                ))
                .await?;
            lock.release().await;
            failed += changed.rows_affected();
        }
        Ok(failed)
    }

    pub fn signal_stop(&self, run_id: i64) {
        if let Some(flag) = self.local_runs().get(&run_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub async fn shutdown(&self, wait: Duration) {
        self.inner.stopping.store(true, Ordering::Relaxed);
        self.inner.wake.notify_waiters();
        let flags: Vec<Arc<AtomicBool>> = self.local_runs().values().cloned().collect();
        for flag in flags {
            flag.store(true, Ordering::Relaxed);
        }
        let deadline = tokio::time::Instant::now() + wait;
        while !self.local_runs().is_empty() && tokio::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

async fn cancel_requested(db: &sea_orm::DatabaseConnection, run_id: i64) -> bool {
    #[derive(FromQueryResult)]
    struct Flag {
        cancel_requested: bool,
    }
    Flag::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT cancel_requested FROM task_run WHERE id = $1",
        [run_id.into()],
    ))
    .one(db)
    .await
    .ok()
    .flatten()
    .is_none_or(|flag| flag.cancel_requested)
}

async fn finish(
    tasks: &Tasks,
    run_id: i64,
    state: RunState,
    message: &str,
    counts: serde_json::Value,
) -> Result<(), DbErr> {
    let message: String = message.chars().take(2000).collect();
    tasks
        .db()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE task_run SET state = $2, message = $3, counts = $4, finished_at = now() \
             WHERE id = $1",
            [
                run_id.into(),
                state.as_str().into(),
                message.into(),
                counts.into(),
            ],
        ))
        .await?;
    Ok(())
}

pub fn describe(duration: chrono::Duration) -> String {
    let millis = duration.num_milliseconds().max(0);
    match millis {
        0..=999 => format!("{millis} ms"),
        1_000..=59_999 => format!("{:.1} s", millis as f64 / 1000.0),
        _ => {
            let seconds = millis / 1000;
            format!("{} min {} s", seconds / 60, seconds % 60)
        }
    }
}
