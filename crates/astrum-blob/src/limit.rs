use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::Instant;

#[derive(Clone, Debug)]
pub struct Limiter {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    rate: f64,
    capacity: f64,
    bucket: Mutex<Bucket>,
}

#[derive(Debug)]
struct Bucket {
    tokens: f64,
    refilled: Instant,
}

impl Limiter {
    pub fn new(bytes_per_second: u64) -> Option<Self> {
        if bytes_per_second == 0 {
            return None;
        }
        let rate = bytes_per_second as f64;
        Some(Self {
            shared: Arc::new(Shared {
                rate,
                capacity: rate,
                bucket: Mutex::new(Bucket {
                    tokens: 0.0,
                    refilled: Instant::now(),
                }),
            }),
        })
    }

    pub fn bytes_per_second(&self) -> u64 {
        self.shared.rate as u64
    }

    pub async fn acquire(&self, bytes: usize) {
        let wait = {
            let mut bucket = self
                .shared
                .bucket
                .lock()
                .expect("the limiter lock is never poisoned");
            let now = Instant::now();
            let elapsed = now.duration_since(bucket.refilled).as_secs_f64();
            bucket.refilled = now;
            bucket.tokens = (bucket.tokens + elapsed * self.shared.rate).min(self.shared.capacity);
            bucket.tokens -= bytes as f64;
            match bucket.tokens < 0.0 {
                true => Duration::from_secs_f64(-bucket.tokens / self.shared.rate),
                false => Duration::ZERO,
            }
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_total_wait_follows_the_rate() {
        let limiter = Limiter::new(1_000_000).unwrap();
        let started = Instant::now();
        for _ in 0..10 {
            limiter.acquire(10_000).await;
        }
        let taken = started.elapsed().as_secs_f64();
        assert!((0.1..0.2).contains(&taken), "{taken}");
    }

    #[tokio::test]
    async fn clones_share_one_budget() {
        let limiter = Limiter::new(100_000).unwrap();
        let other = limiter.clone();
        let started = Instant::now();
        tokio::join!(limiter.acquire(10_000), other.acquire(10_000));
        let taken = started.elapsed().as_secs_f64();
        assert!((0.2..0.3).contains(&taken), "{taken}");
    }

    #[test]
    fn a_zero_rate_is_no_limiter() {
        assert!(Limiter::new(0).is_none());
        assert_eq!(Limiter::new(5).unwrap().bytes_per_second(), 5);
    }
}
