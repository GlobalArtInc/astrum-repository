use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use astrum_content::Content;
use astrum_db::audit::{self, Event};
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use crate::kind::{Properties, TaskError};

pub const MAX_LOG_LINES: usize = 1000;
pub const MAX_LINE_LENGTH: usize = 2000;
pub const INITIATOR: &str = "*TASK";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

#[derive(Default)]
struct Log {
    pending: Vec<(i32, DateTime<Utc>, Level, String)>,
    written: usize,
    dropped: usize,
}

pub struct Run {
    pub id: i64,
    pub task_id: Uuid,
    pub task_name: String,
    pub task_type: String,
    pub properties: Properties,
    content: Content,
    cancel: Arc<AtomicBool>,
    log: Mutex<Log>,
    counts: Mutex<BTreeMap<String, i64>>,
    expiries: Vec<Arc<dyn crate::kind::Expiry>>,
}

impl Run {
    pub fn new(
        id: i64,
        task_id: Uuid,
        task_name: impl Into<String>,
        task_type: impl Into<String>,
        properties: Properties,
        content: Content,
        cancel: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            task_id,
            task_name: task_name.into(),
            task_type: task_type.into(),
            properties,
            content,
            cancel,
            log: Mutex::new(Log::default()),
            counts: Mutex::new(BTreeMap::new()),
            expiries: Vec::new(),
        }
    }

    pub fn with_expiries(mut self, expiries: Vec<Arc<dyn crate::kind::Expiry>>) -> Self {
        self.expiries = expiries;
        self
    }

    pub fn expiries(&self) -> Vec<Arc<dyn crate::kind::Expiry>> {
        self.expiries.clone()
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub fn db(&self) -> &DatabaseConnection {
        self.content.db()
    }

    pub fn property(&self, id: &str) -> Option<&str> {
        self.properties
            .get(id)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
    }

    pub fn number(&self, id: &str) -> Option<i64> {
        self.property(id).and_then(|value| value.parse().ok())
    }

    pub fn flag(&self, id: &str) -> bool {
        self.property(id)
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    }

    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn check(&self) -> Result<(), TaskError> {
        match self.cancelled() {
            true => Err(TaskError::Cancelled),
            false => Ok(()),
        }
    }

    pub fn info(&self, message: impl Into<String>) {
        self.line(Level::Info, message.into());
    }

    pub fn warn(&self, message: impl Into<String>) {
        self.line(Level::Warn, message.into());
    }

    pub fn error(&self, message: impl Into<String>) {
        self.line(Level::Error, message.into());
    }

    pub(crate) fn line(&self, level: Level, message: String) {
        match level {
            Level::Info => tracing::info!(task = %self.task_name, run = self.id, "{message}"),
            Level::Warn => tracing::warn!(task = %self.task_name, run = self.id, "{message}"),
            Level::Error => tracing::error!(task = %self.task_name, run = self.id, "{message}"),
        }
        let mut log = self
            .log
            .lock()
            .expect("the task log lock is never poisoned");
        let line = log.written + log.pending.len();
        if line >= MAX_LOG_LINES {
            log.dropped += 1;
            return;
        }
        let message: String = message.chars().take(MAX_LINE_LENGTH).collect();
        log.pending
            .push((line as i32 + 1, Utc::now(), level, message));
    }

    pub fn count(&self, key: &str, amount: i64) {
        *self
            .counts
            .lock()
            .expect("the task counter lock is never poisoned")
            .entry(key.to_owned())
            .or_default() += amount;
    }

    pub fn counts(&self) -> BTreeMap<String, i64> {
        self.counts
            .lock()
            .expect("the task counter lock is never poisoned")
            .clone()
    }

    pub async fn audit(&self, event: Event) {
        let mut event = event.by(INITIATOR);
        if let serde_json::Value::Object(attributes) = &mut event.attributes {
            attributes.insert("task".to_owned(), self.task_name.clone().into());
            attributes.insert("run".to_owned(), self.id.into());
        }
        if let Err(error) = audit::record(self.db(), event).await {
            tracing::error!(%error, "an audit event of a task was not recorded");
        }
    }

    pub async fn flush(&self) {
        let pending = {
            let mut log = self
                .log
                .lock()
                .expect("the task log lock is never poisoned");
            let pending = std::mem::take(&mut log.pending);
            log.written += pending.len();
            pending
        };
        if pending.is_empty() {
            return;
        }
        let mut lines = Vec::with_capacity(pending.len());
        let mut times = Vec::with_capacity(pending.len());
        let mut levels = Vec::with_capacity(pending.len());
        let mut messages = Vec::with_capacity(pending.len());
        for (line, at, level, message) in pending {
            lines.push(line);
            times.push(at.to_rfc3339());
            levels.push(level.as_str().to_owned());
            messages.push(message);
        }
        let written = self
            .db()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO task_log (run_id, line, at, level, message) \
                 SELECT $1, line, at::timestamptz, level, message \
                 FROM unnest($2::int[], $3::text[], $4::text[], $5::text[]) \
                 AS lines(line, at, level, message) ON CONFLICT DO NOTHING",
                [
                    self.id.into(),
                    lines.into(),
                    times.into(),
                    levels.into(),
                    messages.into(),
                ],
            ))
            .await;
        if let Err(error) = written {
            tracing::warn!(%error, run = self.id, "task log lines were not saved");
        }
    }

    pub(crate) fn note_dropped(&self) {
        let dropped = {
            let mut log = self
                .log
                .lock()
                .expect("the task log lock is never poisoned");
            std::mem::take(&mut log.dropped)
        };
        if dropped > 0 {
            let mut log = self
                .log
                .lock()
                .expect("the task log lock is never poisoned");
            let line = log.written + log.pending.len();
            log.pending.push((
                line as i32 + 1,
                Utc::now(),
                Level::Warn,
                format!("{dropped} more lines were not kept; the log of a run keeps {MAX_LOG_LINES} lines"),
            ));
        }
    }
}
