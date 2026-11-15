use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub ok: u64,
    pub failed: u64,
    pub dead: u64,
    pub bytes: u64,
}

#[derive(Debug, Default)]
pub struct Counters {
    totals: Mutex<HashMap<Uuid, Totals>>,
}

impl Counters {
    fn bump(&self, replication: Uuid, change: impl FnOnce(&mut Totals)) {
        let mut totals = self
            .totals
            .lock()
            .expect("the counters lock is never poisoned");
        change(totals.entry(replication).or_default());
    }

    pub fn transferred(&self, replication: Uuid) {
        self.bump(replication, |totals| totals.ok += 1);
    }

    pub fn failed(&self, replication: Uuid) {
        self.bump(replication, |totals| totals.failed += 1);
    }

    pub fn died(&self, replication: Uuid) {
        self.bump(replication, |totals| totals.dead += 1);
    }

    pub fn uploaded(&self, replication: Uuid, bytes: u64) {
        self.bump(replication, |totals| {
            totals.bytes = totals.bytes.saturating_add(bytes);
        });
    }

    pub fn of(&self, replication: Uuid) -> Totals {
        self.totals
            .lock()
            .expect("the counters lock is never poisoned")
            .get(&replication)
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_replication_is_counted_on_its_own() {
        let counters = Counters::default();
        let (first, second) = (Uuid::from_u128(1), Uuid::from_u128(2));
        counters.transferred(first);
        counters.transferred(first);
        counters.failed(first);
        counters.died(second);
        counters.uploaded(second, 10);
        counters.uploaded(second, u64::MAX);
        assert_eq!(
            counters.of(first),
            Totals {
                ok: 2,
                failed: 1,
                dead: 0,
                bytes: 0
            }
        );
        assert_eq!(
            counters.of(second),
            Totals {
                ok: 0,
                failed: 0,
                dead: 1,
                bytes: u64::MAX
            }
        );
        assert_eq!(counters.of(Uuid::from_u128(3)), Totals::default());
    }
}
