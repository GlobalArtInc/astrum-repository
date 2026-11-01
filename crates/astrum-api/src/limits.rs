use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Rate {
    pub burst: f64,
    pub per_second: f64,
}

pub const AGENT_CALLS: Rate = Rate {
    burst: 120.0,
    per_second: 4.0,
};
pub const CLIENT_REGISTRATIONS: Rate = Rate {
    burst: 10.0,
    per_second: 10.0 / 3600.0,
};
pub const OAUTH_REQUESTS: Rate = Rate {
    burst: 30.0,
    per_second: 0.5,
};
pub const UPLOAD_ATTEMPTS: Rate = Rate {
    burst: 30.0,
    per_second: 1.0,
};
const PRUNE_ABOVE: usize = 50_000;
const IDLE: Duration = Duration::from_secs(3600);

#[derive(Debug, Default)]
pub struct Limiter {
    buckets: Mutex<HashMap<String, (f64, Instant)>>,
}

impl Limiter {
    pub fn take(&self, key: &str, rate: Rate) -> Result<(), Duration> {
        let now = Instant::now();
        let mut buckets = self
            .buckets
            .lock()
            .expect("the rate limit lock is never poisoned");
        if buckets.len() > PRUNE_ABOVE {
            buckets.retain(|_, (_, at)| now.duration_since(*at) < IDLE);
        }
        let entry = buckets.entry(key.to_owned()).or_insert((rate.burst, now));
        let available =
            (entry.0 + now.duration_since(entry.1).as_secs_f64() * rate.per_second).min(rate.burst);
        if available >= 1.0 {
            *entry = (available - 1.0, now);
            return Ok(());
        }
        *entry = (available, now);
        Err(Duration::from_secs_f64(
            ((1.0 - available) / rate.per_second).max(1.0),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bucket_lets_a_burst_through_and_then_waits() {
        let limiter = Limiter::default();
        let rate = Rate {
            burst: 3.0,
            per_second: 0.001,
        };
        for _ in 0..3 {
            assert!(limiter.take("a", rate).is_ok());
        }
        let wait = limiter.take("a", rate).unwrap_err();
        assert!(wait >= Duration::from_secs(1), "{wait:?}");
        assert!(limiter.take("b", rate).is_ok());
    }

    #[test]
    fn a_bucket_fills_up_again() {
        let limiter = Limiter::default();
        let rate = Rate {
            burst: 1.0,
            per_second: 1000.0,
        };
        assert!(limiter.take("a", rate).is_ok());
        std::thread::sleep(Duration::from_millis(5));
        assert!(limiter.take("a", rate).is_ok());
    }
}
