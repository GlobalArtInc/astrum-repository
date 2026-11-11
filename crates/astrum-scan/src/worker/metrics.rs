use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

pub const BUCKETS: [f64; 13] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0,
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub buckets: [u64; BUCKETS.len()],
    pub sum: f64,
    pub count: u64,
    pub failures: u64,
}

#[derive(Default)]
pub struct Metrics {
    failures: AtomicU64,
    durations: Mutex<Snapshot>,
}

impl Metrics {
    pub fn observe(&self, seconds: f64) {
        let mut durations = self
            .durations
            .lock()
            .expect("the scan duration lock is never poisoned");
        for (bucket, bound) in durations.buckets.iter_mut().zip(BUCKETS) {
            if seconds <= bound {
                *bucket += 1;
            }
        }
        durations.sum += seconds;
        durations.count += 1;
    }

    pub fn failed(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = self
            .durations
            .lock()
            .expect("the scan duration lock is never poisoned")
            .clone();
        snapshot.failures = self.failures.load(Ordering::Relaxed);
        snapshot
    }
}
