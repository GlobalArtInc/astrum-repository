use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use astrum_outbound::ClientOptions;
use astrum_security::ssrf::{Guard, GuardedResolver, SSRF_KEY, SsrfProtection};
use serde::Deserialize;

use crate::AppState;

const API: &str = "https://api.github.com/repos/GlobalArtInc/astrum-repository";
const KNOWN_FOR: Duration = Duration::from_secs(6 * 60 * 60);
const FAILED_FOR: Duration = Duration::from_secs(30 * 60);
const TIMEOUT_SECONDS: i32 = 5;
const MAX_BODY: u64 = 1024 * 1024;

#[derive(Default)]
pub struct Stars {
    entry: Mutex<Option<Entry>>,
    turn: tokio::sync::Mutex<()>,
}

#[derive(Clone, Copy)]
struct Entry {
    at: Instant,
    ttl: Duration,
    stars: Option<u64>,
}

#[derive(Deserialize)]
struct Repository {
    stargazers_count: u64,
}

impl Stars {
    pub fn known(&self) -> Option<Option<u64>> {
        self.entry
            .lock()
            .expect("the stars lock is never poisoned")
            .filter(|entry| entry.at.elapsed() < entry.ttl)
            .map(|entry| entry.stars)
    }

    pub fn remember(&self, stars: u64) {
        self.store(Some(stars), KNOWN_FOR);
    }

    pub fn failed(&self) {
        let last = self
            .entry
            .lock()
            .expect("the stars lock is never poisoned")
            .and_then(|entry| entry.stars);
        self.store(last, FAILED_FOR);
    }

    fn store(&self, stars: Option<u64>, ttl: Duration) {
        *self.entry.lock().expect("the stars lock is never poisoned") = Some(Entry {
            at: Instant::now(),
            ttl,
            stars,
        });
    }
}

pub async fn stars(state: &AppState) -> Option<u64> {
    let cache = state.stars();
    if let Some(known) = cache.known() {
        return known;
    }
    let _turn = cache.turn.lock().await;
    if let Some(known) = cache.known() {
        return known;
    }
    match fetch(state).await {
        Ok(found) => cache.remember(found),
        Err(message) => {
            tracing::debug!(message, "the number of GitHub stars cannot be fetched");
            cache.failed();
        }
    }
    cache.known().flatten()
}

async fn fetch(state: &AppState) -> Result<u64, String> {
    let outbound = state
        .content()
        .outbound()
        .snapshot()
        .await
        .map_err(|error| error.to_string())?;
    let ssrf: SsrfProtection = astrum_db::setting::get(state.db().conn(), SSRF_KEY)
        .await
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let guard = Arc::new(Guard::new(&outbound.guard_settings(&ssrf)));
    let options = ClientOptions {
        user_agent_suffix: None,
        timeout_seconds: Some(TIMEOUT_SECONDS),
        use_trust_store: true,
    };
    let client = outbound
        .configure(reqwest::Client::builder(), state.secrets(), &options)
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(Arc::new(GuardedResolver::new(guard)))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get(API)
        .header("accept", "application/vnd.github+json")
        .header("x-github-api-version", "2022-11-28")
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub answered {}", response.status()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BODY)
    {
        return Err("the answer of GitHub is too large".to_owned());
    }
    let body = response.bytes().await.map_err(|error| error.to_string())?;
    parse(&body)
}

fn parse(body: &[u8]) -> Result<u64, String> {
    serde_json::from_slice::<Repository>(body)
        .map(|repository| repository.stargazers_count)
        .map_err(|error| format!("the answer of GitHub has no star count: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aged(stars: Option<u64>, ttl: Duration, age: Duration) -> Stars {
        let cache = Stars::default();
        *cache.entry.lock().unwrap() = Some(Entry {
            at: Instant::now().checked_sub(age).unwrap(),
            ttl,
            stars,
        });
        cache
    }

    #[test]
    fn the_star_count_is_read_from_the_answer_of_github() {
        let body =
            br#"{"id": 1, "name": "astrum-repository", "stargazers_count": 128, "forks": 3}"#;
        assert_eq!(parse(body), Ok(128));
        assert!(parse(br#"{"message": "Not Found"}"#).is_err());
        assert!(parse(b"<html>").is_err());
    }

    #[test]
    fn a_number_is_kept_for_hours_and_a_failure_for_minutes() {
        let cache = Stars::default();
        assert_eq!(cache.known(), None);
        cache.remember(42);
        assert_eq!(cache.known(), Some(Some(42)));

        let stale = aged(Some(42), KNOWN_FOR, KNOWN_FOR + Duration::from_secs(1));
        assert_eq!(stale.known(), None);
        let fresh = aged(Some(42), KNOWN_FOR, KNOWN_FOR - Duration::from_secs(60));
        assert_eq!(fresh.known(), Some(Some(42)));

        let failed = aged(None, FAILED_FOR, FAILED_FOR + Duration::from_secs(1));
        assert_eq!(failed.known(), None);
    }

    #[test]
    fn a_failure_keeps_the_last_number_and_waits_before_asking_again() {
        let cache = aged(Some(42), KNOWN_FOR, KNOWN_FOR + Duration::from_secs(1));
        assert_eq!(cache.known(), None);
        cache.failed();
        assert_eq!(cache.known(), Some(Some(42)));

        let never = Stars::default();
        never.failed();
        assert_eq!(never.known(), Some(None));
    }
}
