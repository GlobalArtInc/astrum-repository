use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration as StdDuration, Instant};

use astrum_db::entity::user_session as session;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rand::RngCore;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use sha2::{Digest, Sha256};

use crate::catalogue::DEFAULT_SOURCE;
use crate::realm;
use crate::subject::Subject;
use crate::tokens::hex;
use crate::{Error, Security};

pub const TIMEOUT_KEY: &str = "security.sessionTimeoutMinutes";
const TOUCH_EVERY: Duration = Duration::seconds(60);
pub const FAILURES_ALLOWED: u32 = 10;
pub const CONCURRENT_ALLOWED: u32 = 64;
pub const GUESSES_BEFORE_BLOCK: u32 = FAILURES_ALLOWED + CONCURRENT_ALLOWED - 1;
const FAILURE_WINDOW: StdDuration = StdDuration::from_secs(300);
const KEPT_ENTRIES: usize = 10_000;
const HARD_CAP: usize = 50_000;
const PRUNE_EVERY: StdDuration = StdDuration::from_secs(30);
pub const KEY_USER_LENGTH: usize = 256;
pub const KEY_ADDRESS_LENGTH: usize = 64;
const UNKNOWN_ADDRESS: &str = "unknown";
const V6_PREFIX_BITS: u32 = 64;

pub fn throttle_address(address: Option<IpAddr>) -> String {
    match address.map(|address| address.to_canonical()) {
        None => UNKNOWN_ADDRESS.to_owned(),
        Some(IpAddr::V4(address)) => address.to_string(),
        Some(IpAddr::V6(address)) => {
            let prefix = u128::from(address) & (u128::MAX << (128 - V6_PREFIX_BITS));
            format!("{}/{V6_PREFIX_BITS}", std::net::Ipv6Addr::from(prefix))
        }
    }
}

struct Attempts {
    failures: u32,
    in_flight: u32,
    since: Instant,
    reported: bool,
}

struct BasicState {
    entries: HashMap<(String, String), Attempts>,
    last_prune: Instant,
}

pub(crate) struct BasicFailures {
    state: Mutex<BasicState>,
    window: StdDuration,
    cap: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicDenied {
    Blocked(StdDuration),
    Busy,
}

#[must_use = "a slot must be settled with failed or succeeded, or dropped to release it"]
pub struct BasicSlot<'a> {
    owner: &'a BasicFailures,
    key: Option<(String, String)>,
}

impl Default for BasicFailures {
    fn default() -> Self {
        Self::with_limits(FAILURE_WINDOW, HARD_CAP)
    }
}

impl BasicFailures {
    fn with_limits(window: StdDuration, cap: usize) -> Self {
        Self {
            state: Mutex::new(BasicState {
                entries: HashMap::new(),
                last_prune: Instant::now(),
            }),
            window,
            cap,
        }
    }

    fn key(user_id: &str, address: &str) -> (String, String) {
        (
            user_id
                .trim()
                .to_lowercase()
                .chars()
                .take(KEY_USER_LENGTH)
                .collect(),
            address.trim().chars().take(KEY_ADDRESS_LENGTH).collect(),
        )
    }

    fn state(&self) -> MutexGuard<'_, BasicState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn expire(found: &mut Attempts, window: StdDuration) {
        if found.failures > 0 && found.since.elapsed() >= window {
            found.failures = 0;
            found.reported = false;
        }
    }

    pub(crate) fn begin(&self, user_id: &str, address: &str) -> Result<BasicSlot<'_>, BasicDenied> {
        let key = Self::key(user_id, address);
        let mut state = self.state();
        let window = self.window;
        if state.entries.len() > KEPT_ENTRIES && state.last_prune.elapsed() >= PRUNE_EVERY {
            state
                .entries
                .retain(|_, found| found.in_flight > 0 || found.since.elapsed() < window);
            state.last_prune = Instant::now();
        }
        if let Some(found) = state.entries.get_mut(&key) {
            Self::expire(found, window);
            if found.failures >= FAILURES_ALLOWED {
                return Err(BasicDenied::Blocked(
                    window.saturating_sub(found.since.elapsed()),
                ));
            }
            if found.in_flight >= CONCURRENT_ALLOWED {
                return Err(BasicDenied::Busy);
            }
            found.in_flight += 1;
            return Ok(BasicSlot {
                owner: self,
                key: Some(key),
            });
        }
        if state.entries.len() >= self.cap {
            self.make_room(&mut state);
        }
        if state.entries.len() < self.cap {
            state.entries.insert(
                key.clone(),
                Attempts {
                    failures: 0,
                    in_flight: 1,
                    since: Instant::now(),
                    reported: false,
                },
            );
            return Ok(BasicSlot {
                owner: self,
                key: Some(key),
            });
        }
        Ok(BasicSlot {
            owner: self,
            key: None,
        })
    }

    fn make_room(&self, state: &mut BasicState) {
        let window = self.window;
        state
            .entries
            .retain(|_, found| found.in_flight > 0 || found.since.elapsed() < window);
        state.last_prune = Instant::now();
        if state.entries.len() < self.cap {
            return;
        }
        let lowest = state
            .entries
            .values()
            .filter(|found| found.in_flight == 0 && found.failures < FAILURES_ALLOWED)
            .map(|found| found.failures)
            .min();
        if let Some(lowest) = lowest {
            state.entries.retain(|_, found| {
                found.in_flight > 0 || found.failures >= FAILURES_ALLOWED || found.failures > lowest
            });
        }
    }

    fn release(&self, key: &(String, String), outcome: Outcome) -> bool {
        let mut state = self.state();
        let window = self.window;
        let Some(found) = state.entries.get_mut(key) else {
            return false;
        };
        found.in_flight = found.in_flight.saturating_sub(1);
        let mut first = false;
        match outcome {
            Outcome::Failed => {
                Self::expire(found, window);
                if found.failures == 0 {
                    found.since = Instant::now();
                }
                found.failures += 1;
                first = found.failures >= FAILURES_ALLOWED && !found.reported;
                if first {
                    found.reported = true;
                }
            }
            Outcome::Succeeded => {
                found.failures = 0;
                found.reported = false;
            }
            Outcome::Released => {}
        }
        if found.failures == 0 && found.in_flight == 0 {
            state.entries.remove(key);
        }
        first
    }
}

enum Outcome {
    Failed,
    Succeeded,
    Released,
}

impl BasicSlot<'_> {
    pub fn failed(mut self) -> bool {
        match self.key.take() {
            Some(key) => self.owner.release(&key, Outcome::Failed),
            None => false,
        }
    }

    pub fn succeeded(mut self) {
        if let Some(key) = self.key.take() {
            self.owner.release(&key, Outcome::Succeeded);
        }
    }
}

impl Drop for BasicSlot<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.owner.release(&key, Outcome::Released);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewSession {
    pub token: String,
    pub csrf: String,
}

#[derive(Clone, Debug)]
pub struct ActiveSession {
    pub subject: Subject,
    pub csrf: String,
    pub restricted: bool,
}

enum Verdict {
    Valid(Subject),
    Ended,
    Unconfirmed,
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex(&bytes)
}

fn token_hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

pub(crate) fn credential_stamp(password_hash: Option<&str>) -> String {
    hex(&Sha256::digest(
        password_hash.unwrap_or_default().as_bytes(),
    ))[..16]
        .to_owned()
}

impl Security {
    pub async fn open_session(
        &self,
        subject: &Subject,
        client_ip: &str,
        user_agent: &str,
        restricted: bool,
    ) -> Result<NewSession, Error> {
        let stamp = self.sign_in_stamp(subject);
        let token = random_token();
        let csrf = random_token();
        let now = Utc::now().fixed_offset();
        session::Entity::insert(session::ActiveModel {
            token_hash: ActiveValue::Set(token_hash(&token)),
            user_id: ActiveValue::Set(subject.user_id.clone()),
            realm: ActiveValue::Set(subject.realm.clone()),
            credential_stamp: ActiveValue::Set(stamp),
            csrf_token: ActiveValue::Set(csrf.clone()),
            created_at: ActiveValue::Set(now),
            last_seen_at: ActiveValue::Set(now),
            client_ip: ActiveValue::Set(client_ip.chars().take(100).collect()),
            user_agent: ActiveValue::Set(user_agent.chars().take(300).collect()),
            restricted: ActiveValue::Set(restricted),
        })
        .exec(&self.db)
        .await?;
        Ok(NewSession { token, csrf })
    }

    pub fn sign_in_stamp(&self, subject: &Subject) -> String {
        let model = self.model();
        credential_stamp(
            model
                .users
                .get(&subject.user_id)
                .filter(|_| subject.source == DEFAULT_SOURCE || subject.source.is_empty())
                .and_then(|user| user.password_hash.as_deref()),
        )
    }

    pub async fn session(&self, token: &str) -> Result<Option<ActiveSession>, Error> {
        if token.is_empty() || token.len() > 128 {
            return Ok(None);
        }
        let hash = token_hash(token);
        let Some(row) = session::Entity::find_by_id(hash.clone())
            .one(&self.db)
            .await?
        else {
            return Ok(None);
        };
        let now = Utc::now().fixed_offset();
        let mut verdict = self.verdict(&row, now).await;
        if matches!(verdict, Verdict::Ended) && self.sync().await? {
            verdict = self.verdict(&row, now).await;
        }
        let subject = match verdict {
            Verdict::Valid(subject) => subject,
            Verdict::Unconfirmed => return Ok(None),
            Verdict::Ended => {
                session::Entity::delete_by_id(hash).exec(&self.db).await?;
                return Ok(None);
            }
        };
        if now - row.last_seen_at > TOUCH_EVERY {
            session::Entity::update(session::ActiveModel {
                token_hash: ActiveValue::Unchanged(hash),
                last_seen_at: ActiveValue::Set(now),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;
        }
        Ok(Some(ActiveSession {
            subject,
            csrf: row.csrf_token,
            restricted: row.restricted,
        }))
    }

    async fn verdict(&self, row: &session::Model, now: DateTime<FixedOffset>) -> Verdict {
        let model = self.model();
        let timed_out = model.session_timeout_minutes > 0
            && now - row.last_seen_at > Duration::minutes(i64::from(model.session_timeout_minutes));
        let external_stamp = credential_stamp(None);
        let (subject, stamp_matches) = match row.realm.as_str() {
            realm::LDAP => match self.ldap_subject(&row.user_id).await {
                Ok(subject) => (subject, row.credential_stamp == external_stamp),
                Err(error) => {
                    tracing::warn!(user = %row.user_id, %error, "LDAP cannot confirm a session; the request goes on without it");
                    return Verdict::Unconfirmed;
                }
            },
            realm::OIDC => (
                model.oidc_subject(&row.user_id),
                row.credential_stamp == external_stamp,
            ),
            _ => {
                let user = model.users.get(&row.user_id);
                let stamp_matches = user.is_some_and(|user| {
                    credential_stamp(user.password_hash.as_deref()) == row.credential_stamp
                });
                (model.local_subject(&row.user_id, &row.realm), stamp_matches)
            }
        };
        match (subject, stamp_matches, timed_out) {
            (Some(subject), true, false) => Verdict::Valid(subject),
            _ => Verdict::Ended,
        }
    }

    pub async fn close_session(&self, token: &str) -> Result<bool, Error> {
        let deleted = session::Entity::delete_by_id(token_hash(token))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn close_sessions_of(&self, user_id: &str) -> Result<u64, Error> {
        let deleted = session::Entity::delete_many()
            .filter(session::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn close_sessions_in_realm(&self, user_id: &str, realm: &str) -> Result<u64, Error> {
        let deleted = session::Entity::delete_many()
            .filter(session::Column::UserId.eq(user_id))
            .filter(session::Column::Realm.eq(realm))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn keep_only_session(&self, user_id: &str, keep: Option<&str>) -> Result<u64, Error> {
        let mut others = session::Entity::delete_many().filter(session::Column::UserId.eq(user_id));
        if let Some(token) = keep {
            let hash = token_hash(token);
            others = others.filter(session::Column::TokenHash.ne(hash.clone()));
            let model = self.model();
            let stamp = credential_stamp(
                model
                    .users
                    .get(user_id)
                    .and_then(|user| user.password_hash.as_deref()),
            );
            session::Entity::update_many()
                .col_expr(session::Column::CredentialStamp, Expr::value(stamp))
                .filter(session::Column::TokenHash.eq(hash))
                .filter(session::Column::UserId.eq(user_id))
                .exec(&self.db)
                .await?;
        }
        Ok(others.exec(&self.db).await?.rows_affected)
    }

    pub async fn settle_after_enrollment(
        &self,
        user_id: &str,
        keep: Option<&str>,
    ) -> Result<u64, Error> {
        let txn = self.db.begin().await?;
        let mut others = session::Entity::delete_many().filter(session::Column::UserId.eq(user_id));
        if let Some(token) = keep {
            let hash = token_hash(token);
            session::Entity::update_many()
                .col_expr(session::Column::Restricted, Expr::value(false))
                .filter(session::Column::TokenHash.eq(hash.clone()))
                .filter(session::Column::UserId.eq(user_id))
                .exec(&txn)
                .await?;
            others = others.filter(session::Column::TokenHash.ne(hash));
        }
        let closed = others.exec(&txn).await?.rows_affected;
        txn.commit().await?;
        Ok(closed)
    }

    pub fn sign_in_blocked(&self, user_id: &str) -> bool {
        let failures = self
            .failures
            .lock()
            .expect("the sign-in failures lock is never poisoned");
        failures.get(user_id).is_some_and(|(count, since)| {
            *count >= FAILURES_ALLOWED && since.elapsed() < FAILURE_WINDOW
        })
    }

    pub fn sign_in_failed(&self, user_id: &str) {
        let mut failures = self
            .failures
            .lock()
            .expect("the sign-in failures lock is never poisoned");
        if failures.len() > 10_000 {
            failures.retain(|_, (_, since)| since.elapsed() < FAILURE_WINDOW);
        }
        let entry = failures
            .entry(user_id.to_owned())
            .or_insert((0, Instant::now()));
        if entry.1.elapsed() >= FAILURE_WINDOW {
            *entry = (0, Instant::now());
        }
        entry.0 += 1;
    }

    pub fn sign_in_succeeded(&self, user_id: &str) {
        self.failures
            .lock()
            .expect("the sign-in failures lock is never poisoned")
            .remove(user_id);
    }

    pub fn basic_begin(
        &self,
        user_id: &str,
        address: Option<IpAddr>,
    ) -> Result<BasicSlot<'_>, BasicDenied> {
        self.basic_failures
            .begin(user_id, &throttle_address(address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fail(failures: &BasicFailures, user: &str, address: &str) -> bool {
        failures.begin(user, address).unwrap().failed()
    }

    fn blocked(failures: &BasicFailures, user: &str, address: &str) -> Option<StdDuration> {
        match failures.begin(user, address) {
            Err(BasicDenied::Blocked(wait)) => Some(wait),
            _ => None,
        }
    }

    #[test]
    fn failures_are_counted_per_user_and_address() {
        let failures = BasicFailures::default();
        for _ in 0..FAILURES_ALLOWED - 1 {
            fail(&failures, "Admin", "203.0.113.1");
        }
        assert!(blocked(&failures, "admin", "203.0.113.1").is_none());
        fail(&failures, " admin ", "203.0.113.1");
        let wait = blocked(&failures, "ADMIN", "203.0.113.1").unwrap();
        assert!(wait > StdDuration::ZERO && wait <= FAILURE_WINDOW);
        assert!(blocked(&failures, "admin", "203.0.113.2").is_none());
        assert!(blocked(&failures, "other", "203.0.113.1").is_none());
        failures.begin("admin", "203.0.113.2").unwrap().succeeded();
        assert!(failures.begin("admin", "203.0.113.1").is_err());
    }

    #[test]
    fn a_success_clears_the_failures() {
        let failures = BasicFailures::default();
        for _ in 0..FAILURES_ALLOWED - 1 {
            fail(&failures, "ci", "a");
        }
        failures.begin("ci", "a").unwrap().succeeded();
        for _ in 0..FAILURES_ALLOWED - 1 {
            fail(&failures, "ci", "a");
        }
        assert!(failures.begin("ci", "a").is_ok());
        assert_eq!(failures.state().entries.len(), 1);
    }

    #[test]
    fn a_block_is_reported_once() {
        let failures = BasicFailures::default();
        for _ in 0..FAILURES_ALLOWED - 1 {
            assert!(!fail(&failures, "ci", "a"));
        }
        assert!(fail(&failures, "ci", "a"));
        assert!(blocked(&failures, "ci", "a").is_some());
        assert!(blocked(&failures, "ci", "a").is_some());
    }

    #[test]
    fn a_slot_is_released_on_every_exit_and_leaves_no_entry() {
        let failures = BasicFailures::default();
        let held: Vec<BasicSlot<'_>> = (0..CONCURRENT_ALLOWED)
            .map(|_| failures.begin("ci", "a").unwrap())
            .collect();
        assert_eq!(failures.begin("ci", "a").err(), Some(BasicDenied::Busy));
        let mut held = held.into_iter();
        held.next().unwrap().failed();
        assert!(failures.begin("ci", "a").is_ok());
        held.next().unwrap().succeeded();
        drop(held.next());
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _slot = failures.begin("ci", "b").unwrap();
            panic!("a verification panics");
        }));
        assert!(panicked.is_err());
        assert!(failures.begin("ci", "b").is_ok());
        drop(held);
        assert_eq!(failures.state().entries.len(), 0);
    }

    #[test]
    fn busy_requests_do_not_count_as_failures() {
        let failures = BasicFailures::default();
        let held: Vec<BasicSlot<'_>> = (0..CONCURRENT_ALLOWED)
            .map(|_| failures.begin("ci", "a").unwrap())
            .collect();
        for _ in 0..FAILURES_ALLOWED * 3 {
            assert_eq!(failures.begin("ci", "a").err(), Some(BasicDenied::Busy));
        }
        drop(held);
        assert!(failures.begin("ci", "a").is_ok());
        assert_eq!(failures.state().entries.len(), 0);
    }

    #[test]
    fn parallel_guesses_stop_at_the_documented_bound() {
        let failures = BasicFailures::default();
        let mut running: Vec<BasicSlot<'_>> = Vec::new();
        let mut started = 0u32;
        while let Ok(slot) = failures.begin("ci", "a") {
            running.push(slot);
            started += 1;
        }
        assert_eq!(started, CONCURRENT_ALLOWED);
        while let Some(slot) = running.pop() {
            slot.failed();
            match failures.begin("ci", "a") {
                Ok(slot) => {
                    running.push(slot);
                    started += 1;
                }
                Err(denied) => assert!(matches!(denied, BasicDenied::Blocked(_))),
            }
        }
        assert_eq!(started, GUESSES_BEFORE_BLOCK);
        assert_eq!(GUESSES_BEFORE_BLOCK, 73);
        assert!(matches!(
            failures.begin("ci", "a").err(),
            Some(BasicDenied::Blocked(_))
        ));
    }

    #[test]
    fn the_window_ends() {
        let failures = BasicFailures::with_limits(StdDuration::from_millis(100), HARD_CAP);
        for _ in 0..FAILURES_ALLOWED {
            fail(&failures, "ci", "a");
        }
        assert!(blocked(&failures, "ci", "a").is_some());
        std::thread::sleep(StdDuration::from_millis(150));
        assert!(failures.begin("ci", "a").is_ok());
    }

    #[test]
    fn the_map_stays_within_its_cap_and_new_keys_fail_open() {
        let failures = BasicFailures::with_limits(FAILURE_WINDOW, 3);
        for user in ["a", "b", "c"] {
            fail(&failures, user, "x");
        }
        fail(&failures, "d", "x");
        assert!(failures.state().entries.len() <= 3);
        for user in ["a", "b", "c"] {
            for _ in 0..FAILURES_ALLOWED {
                if let Ok(slot) = failures.begin(user, "x") {
                    slot.failed();
                }
            }
        }
        for user in ["a", "b", "c"] {
            assert!(blocked(&failures, user, "x").is_some(), "{user}");
        }
        for _ in 0..FAILURES_ALLOWED * 2 {
            assert!(!fail(&failures, "new", "x"));
        }
        assert!(blocked(&failures, "new", "x").is_none());
        assert_eq!(failures.state().entries.len(), 3);
    }

    #[test]
    fn throttle_addresses_are_canonical_and_group_ipv6_by_its_first_64_bits() {
        let key = |text: &str| throttle_address(Some(text.parse().unwrap()));
        assert_eq!(key("203.0.113.9"), "203.0.113.9");
        assert_eq!(key("::ffff:203.0.113.9"), "203.0.113.9");
        assert_eq!(key("2001:db8:1:2:3:4:5:6"), "2001:db8:1:2::/64");
        assert_eq!(
            key("2001:db8:1:2:ffff:ffff:ffff:ffff"),
            key("2001:0db8:1:2::1")
        );
        assert_ne!(key("2001:db8:1:3::1"), key("2001:db8:1:2::1"));
        assert_eq!(throttle_address(None), "unknown");
        let failures = BasicFailures::default();
        for _ in 0..FAILURES_ALLOWED {
            fail(&failures, "ci", &key("2001:db8:1:2:aaaa::1"));
        }
        assert!(blocked(&failures, "ci", &key("2001:db8:1:2:bbbb::2")).is_some());
    }

    #[test]
    fn basic_keys_stay_short() {
        let long = "u".repeat(10_000);
        let (user, address) = BasicFailures::key(&long, &long);
        assert_eq!(user.len(), KEY_USER_LENGTH);
        assert_eq!(address.len(), KEY_ADDRESS_LENGTH);
    }

    #[test]
    fn a_new_password_changes_the_stamp() {
        let first = credential_stamp(Some("$argon2id$a"));
        assert_eq!(first.len(), 16);
        assert_eq!(first, credential_stamp(Some("$argon2id$a")));
        assert_ne!(first, credential_stamp(Some("$argon2id$b")));
        assert_ne!(first, credential_stamp(None));
    }

    #[test]
    fn session_tokens_are_long_and_random() {
        let token = random_token();
        assert_eq!(token.len(), 64);
        assert_ne!(random_token(), token);
        assert_ne!(token_hash(&token), token);
    }
}
