use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::worker::BUCKETS;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub buckets: [u64; BUCKETS.len()],
    pub sum: f64,
    pub count: u64,
    pub added: u64,
    pub removed: u64,
    pub skipped_advisories: u64,
    pub unresolved_packages: u64,
    pub truncated_packages: u64,
    pub failures: u64,
}

#[derive(Default)]
pub struct Metrics {
    added: AtomicU64,
    removed: AtomicU64,
    skipped_advisories: AtomicU64,
    unresolved_packages: AtomicU64,
    truncated_packages: AtomicU64,
    failures: AtomicU64,
    durations: Mutex<Snapshot>,
}

impl Metrics {
    pub fn observe(&self, seconds: f64) {
        let mut durations = self
            .durations
            .lock()
            .expect("the match duration lock is never poisoned");
        for (bucket, bound) in durations.buckets.iter_mut().zip(BUCKETS) {
            if seconds <= bound {
                *bucket += 1;
            }
        }
        durations.sum += seconds;
        durations.count += 1;
    }

    pub fn record(&self, batch: &super::Batch) {
        self.added
            .fetch_add(batch.added.len() as u64, Ordering::Relaxed);
        self.removed
            .fetch_add(batch.removed.len() as u64, Ordering::Relaxed);
        self.skipped_advisories
            .fetch_add(batch.skipped_advisories, Ordering::Relaxed);
        self.unresolved_packages
            .fetch_add(batch.unresolved_packages, Ordering::Relaxed);
        self.truncated_packages
            .fetch_add(batch.truncated_packages, Ordering::Relaxed);
    }

    pub fn failed(&self, count: u64) {
        self.failures.fetch_add(count, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = self
            .durations
            .lock()
            .expect("the match duration lock is never poisoned")
            .clone();
        snapshot.added = self.added.load(Ordering::Relaxed);
        snapshot.removed = self.removed.load(Ordering::Relaxed);
        snapshot.skipped_advisories = self.skipped_advisories.load(Ordering::Relaxed);
        snapshot.unresolved_packages = self.unresolved_packages.load(Ordering::Relaxed);
        snapshot.truncated_packages = self.truncated_packages.load(Ordering::Relaxed);
        snapshot.failures = self.failures.load(Ordering::Relaxed);
        snapshot
    }
}
