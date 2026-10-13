pub mod bootstrap;
pub mod cleanup;
pub mod cron;
pub mod kind;
pub mod kinds;
pub mod run;
pub mod schedule;
pub mod scheduler;
pub mod store;
pub mod validate;

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use astrum_content::Content;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use tokio::sync::{Notify, Semaphore};

pub use bootstrap::bootstrap;
pub use kind::{Expiry, Field, FieldType, Properties, TaskError, TaskKind};
pub use run::Run;
pub use schedule::Schedule;
pub use store::{RunState, Task, TaskRun};

pub const MAX_CONCURRENT_RUNS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Problem {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Problems(Vec<Problem>);

impl Problems {
    pub fn add(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.0.push(Problem {
            id: id.into(),
            message: message.into(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_inner(self) -> Vec<Problem> {
        self.0
    }

    pub fn finish(self) -> Result<(), Vec<Problem>> {
        match self.0.is_empty() {
            true => Ok(()),
            false => Err(self.0),
        }
    }
}

#[derive(Clone)]
pub struct Tasks {
    inner: Arc<Inner>,
}

struct Inner {
    content: Content,
    kinds: RwLock<BTreeMap<&'static str, Arc<dyn TaskKind>>>,
    expiries: RwLock<Vec<Arc<dyn Expiry>>>,
    wake: Notify,
    running: Mutex<HashMap<i64, Arc<AtomicBool>>>,
    permits: Arc<Semaphore>,
    stopping: AtomicBool,
    node: String,
}

impl std::fmt::Debug for Tasks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Tasks")
    }
}

impl Tasks {
    pub fn new(content: Content) -> Self {
        let tasks = Self {
            inner: Arc::new(Inner {
                content,
                kinds: RwLock::new(BTreeMap::new()),
                expiries: RwLock::new(Vec::new()),
                wake: Notify::new(),
                running: Mutex::new(HashMap::new()),
                permits: Arc::new(Semaphore::new(MAX_CONCURRENT_RUNS)),
                stopping: AtomicBool::new(false),
                node: node_name(),
            }),
        };
        kinds::register_builtin(&tasks);
        tasks
    }

    pub fn content(&self) -> &Content {
        &self.inner.content
    }

    pub fn db(&self) -> &DatabaseConnection {
        self.inner.content.db()
    }

    pub fn node(&self) -> &str {
        &self.inner.node
    }

    pub fn register(&self, kind: Arc<dyn TaskKind>) {
        self.inner
            .kinds
            .write()
            .expect("the task kind registry lock is never poisoned")
            .insert(kind.id(), kind);
    }

    pub fn kind(&self, id: &str) -> Option<Arc<dyn TaskKind>> {
        self.inner
            .kinds
            .read()
            .expect("the task kind registry lock is never poisoned")
            .get(id)
            .cloned()
    }

    pub fn kinds(&self) -> Vec<Arc<dyn TaskKind>> {
        self.inner
            .kinds
            .read()
            .expect("the task kind registry lock is never poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn add_expiry(&self, expiry: Arc<dyn Expiry>) {
        self.inner
            .expiries
            .write()
            .expect("the expiry registry lock is never poisoned")
            .push(expiry);
    }

    pub fn expiries(&self) -> Vec<Arc<dyn Expiry>> {
        self.inner
            .expiries
            .read()
            .expect("the expiry registry lock is never poisoned")
            .clone()
    }

    pub fn wake(&self) {
        self.inner.wake.notify_one();
    }
}

fn node_name() -> String {
    let host = std::env::var("HOSTNAME")
        .ok()
        .filter(|host| !host.trim().is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|host| host.trim().to_owned())
        })
        .filter(|host| !host.is_empty())
        .unwrap_or_else(|| "astrum".to_owned());
    format!("{host}:{}", std::process::id())
}
