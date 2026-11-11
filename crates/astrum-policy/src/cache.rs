use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use astrum_db::revision;
use sea_orm::ConnectionTrait;
use tokio::sync::Mutex as Flight;

use crate::Error;
use crate::exception::{self, Exception};
use crate::policy::{self, Policy};

pub const MAX_AGE: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct Stored {
    pub revision: i64,
    pub policy: Result<Policy, String>,
}

#[derive(Debug)]
pub struct Snapshot {
    pub revision: i64,
    pub policies: HashMap<String, Stored>,
    pub exceptions: Vec<Exception>,
    pub unreadable_exceptions: usize,
}

struct Held {
    snapshot: Arc<Snapshot>,
    checked: Option<Instant>,
}

#[derive(Default)]
struct Slot {
    held: Option<Held>,
    generation: u64,
}

pub struct Cache {
    max_age: Duration,
    slot: Mutex<Slot>,
    flight: Flight<()>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(MAX_AGE)
    }
}

impl Cache {
    pub fn new(max_age: Duration) -> Self {
        Self {
            max_age,
            slot: Mutex::new(Slot::default()),
            flight: Flight::new(()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Slot> {
        self.slot
            .lock()
            .expect("the policy cache lock is never poisoned")
    }

    fn peek(&self) -> Option<(Arc<Snapshot>, Option<Instant>)> {
        self.lock()
            .held
            .as_ref()
            .map(|held| (held.snapshot.clone(), held.checked))
    }

    fn young(&self) -> Option<Arc<Snapshot>> {
        match self.peek() {
            Some((snapshot, Some(checked))) if checked.elapsed() < self.max_age => Some(snapshot),
            _ => None,
        }
    }

    pub fn invalidate(&self) {
        let mut slot = self.lock();
        slot.generation += 1;
        if let Some(held) = slot.held.as_mut() {
            held.checked = None;
        }
    }

    pub fn unreadable_exceptions(&self) -> usize {
        self.peek()
            .map_or(0, |(snapshot, _)| snapshot.unreadable_exceptions)
    }

    pub async fn get(&self, db: &impl ConnectionTrait) -> Result<Arc<Snapshot>, Error> {
        if let Some(snapshot) = self.young() {
            return Ok(snapshot);
        }
        let _flight = self.flight.lock().await;
        if let Some(snapshot) = self.young() {
            return Ok(snapshot);
        }
        self.fresh(db).await
    }

    pub async fn fresh(&self, db: &impl ConnectionTrait) -> Result<Arc<Snapshot>, Error> {
        let generation = self.lock().generation;
        let current = revision::current(db, revision::POLICIES).await?;
        if let Some((snapshot, _)) = self.peek()
            && snapshot.revision == current
        {
            self.store(snapshot.clone(), generation);
            return Ok(snapshot);
        }
        let policies = policy::entries(db)
            .await?
            .into_iter()
            .map(|entry| {
                (
                    entry.name,
                    Stored {
                        revision: entry.revision,
                        policy: entry.policy,
                    },
                )
            })
            .collect();
        let readable = exception::list_readable(db).await?;
        if !readable.unreadable.is_empty() {
            tracing::warn!(
                revision = current,
                unreadable = readable.unreadable.len(),
                first = %readable.unreadable[0],
                "policy exceptions that cannot be read are skipped, which only makes the verdicts stricter"
            );
        }
        let snapshot = Arc::new(Snapshot {
            revision: current,
            policies,
            exceptions: readable.exceptions,
            unreadable_exceptions: readable.unreadable.len(),
        });
        self.store(snapshot.clone(), generation);
        Ok(snapshot)
    }

    fn store(&self, snapshot: Arc<Snapshot>, generation: u64) {
        let mut slot = self.lock();
        let newer = slot
            .held
            .as_ref()
            .is_none_or(|known| known.snapshot.revision <= snapshot.revision);
        if newer {
            let checked = (slot.generation == generation).then(Instant::now);
            slot.held = Some(Held { snapshot, checked });
        }
    }
}
