use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use astrum_policy::verdict::State;

use crate::worker::BUCKETS;

pub const STATES: [State; 5] = [
    State::Pass,
    State::Report,
    State::Block,
    State::Unscanned,
    State::Error,
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub buckets: [u64; BUCKETS.len()],
    pub sum: f64,
    pub count: u64,
    pub written: [u64; STATES.len()],
    pub dropped: u64,
    pub failures: u64,
    pub sweeps_done: u64,
    pub sweeps_busy: u64,
    pub sweeps_failed: u64,
    pub swept_subjects: u64,
}

#[derive(Default)]
pub struct Metrics {
    written: [AtomicU64; STATES.len()],
    dropped: AtomicU64,
    failures: AtomicU64,
    sweeps_done: AtomicU64,
    sweeps_busy: AtomicU64,
    sweeps_failed: AtomicU64,
    swept_subjects: AtomicU64,
    durations: Mutex<Snapshot>,
}

impl Metrics {
    pub fn observe(&self, seconds: f64) {
        let mut durations = self
            .durations
            .lock()
            .expect("the verdict duration lock is never poisoned");
        for (bucket, bound) in durations.buckets.iter_mut().zip(BUCKETS) {
            if seconds <= bound {
                *bucket += 1;
            }
        }
        durations.sum += seconds;
        durations.count += 1;
    }

    pub fn record(&self, judged: &super::Judged) {
        for (counter, count) in self.written.iter().zip(judged.written) {
            counter.fetch_add(count, Ordering::Relaxed);
        }
        self.dropped.fetch_add(judged.dropped, Ordering::Relaxed);
    }

    pub fn failed(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn swept(&self, outcome: &super::Swept) {
        match outcome {
            super::Swept::Busy => &self.sweeps_busy,
            super::Swept::Done { .. } => &self.sweeps_done,
        }
        .fetch_add(1, Ordering::Relaxed);
        if let super::Swept::Done { marked } = outcome {
            self.swept_subjects.fetch_add(*marked, Ordering::Relaxed);
        }
    }

    pub fn sweep_failed(&self) {
        self.sweeps_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = self
            .durations
            .lock()
            .expect("the verdict duration lock is never poisoned")
            .clone();
        for (slot, counter) in snapshot.written.iter_mut().zip(&self.written) {
            *slot = counter.load(Ordering::Relaxed);
        }
        snapshot.dropped = self.dropped.load(Ordering::Relaxed);
        snapshot.failures = self.failures.load(Ordering::Relaxed);
        snapshot.sweeps_done = self.sweeps_done.load(Ordering::Relaxed);
        snapshot.sweeps_busy = self.sweeps_busy.load(Ordering::Relaxed);
        snapshot.sweeps_failed = self.sweeps_failed.load(Ordering::Relaxed);
        snapshot.swept_subjects = self.swept_subjects.load(Ordering::Relaxed);
        snapshot
    }
}
