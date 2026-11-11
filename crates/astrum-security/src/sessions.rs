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
const FAILURES_ALLOWED: u32 = 10;
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
    count: u32,
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

    pub(crate) fn attempt(&self, user_id: &str, address: &str) -> Result<(), StdDuration> {
        let key = Self::key(user_id, address);
        let mut state = self.state();
        let window = self.window;
        if state.entries.len() > KEPT_ENTRIES && state.last_prune.elapsed() >= PRUNE_EVERY {
            state
                .entries
                .retain(|_, found| found.since.elapsed() < window);
            state.last_prune = Instant::now();
        }
        if let Some(found) = state.entries.get_mut(&key) {
            let elapsed = found.since.elapsed();
            if elapsed >= window {
                *found = Attempts {
                    count: 1,
                    since: Instant::now(),
                    reported: false,
                };
                return Ok(());
            }
            if found.count >= FAILURES_ALLOWED {
                return Err(window - elapsed);
            }
            found.count += 1;
            return Ok(());
        }
        if state.entries.len() >= self.cap {
            self.make_room(&mut state);
        }
        if state.entries.len() < self.cap {
            state.entries.insert(
                key,
                Attempts {
                    count: 1,
                    since: Instant::now(),
                    reported: false,
                },
            );
        }
        Ok(())
    }

    fn make_room(&self, state: &mut BasicState) {
        let window = self.window;
        state
            .entries
            .retain(|_, found| found.since.elapsed() < window);
        state.last_prune = Instant::now();
        if state.entries.len() < self.cap {
            return;
        }
        let lowest = state
            .entries
            .values()
            .filter(|found| found.count < FAILURES_ALLOWED)
            .map(|found| found.count)
            .min();
        if let Some(lowest) = lowest {
            state
                .entries
                .retain(|_, found| found.count >= FAILURES_ALLOWED || found.count > lowest);
        }
    }

    pub(crate) fn blocked(&self, user_id: &str, address: &str) -> Option<StdDuration> {
        let key = Self::key(user_id, address);
        let state = self.state();
        let found = state.entries.get(&key)?;
        let elapsed = found.since.elapsed();
        (found.count >= FAILURES_ALLOWED && elapsed < self.window).then(|| self.window - elapsed)
    }

    pub(crate) fn failed(&self, user_id: &str, address: &str) -> bool {
        let key = Self::key(user_id, address);
        let mut state = self.state();
        let Some(found) = state.entries.get_mut(&key) else {
            return false;
        };
        let blocked = found.count >= FAILURES_ALLOWED && found.since.elapsed() < self.window;
        let first = blocked && !found.reported;
        if first {
            found.reported = true;
        }
        first
    }

    pub(crate) fn refund(&self, user_id: &str, address: &str) {
        let key = Self::key(user_id, address);
        let mut state = self.state();
        if let Some(found) = state.entries.get_mut(&key) {
            found.count = found.count.saturating_sub(1);
            if found.count == 0 {
                state.entries.remove(&key);
            }
        }
    }

    pub(crate) fn succeeded(&self, user_id: &str, address: &str) {
        let key = Self::key(user_id, address);
        self.state().entries.remove(&key);
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

    pub fn basic_attempt(&self, user_id: &str, address: Option<IpAddr>) -> Result<(), StdDuration> {
        self.basic_failures
            .attempt(user_id, &throttle_address(address))
    }

    pub fn basic_blocked(&self, user_id: &str, address: Option<IpAddr>) -> Option<StdDuration> {
        self.basic_failures
            .blocked(user_id, &throttle_address(address))
    }

    pub fn basic_failed(&self, user_id: &str, address: Option<IpAddr>) -> bool {
        self.basic_failures
            .failed(user_id, &throttle_address(address))
    }

    pub fn basic_refunded(&self, user_id: &str, address: Option<IpAddr>) {
        self.basic_failures
            .refund(user_id, &throttle_address(address));
    }

    pub fn basic_succeeded(&self, user_id: &str, address: Option<IpAddr>) {
        self.basic_failures
            .succeeded(user_id, &throttle_address(address));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_attempts_are_counted_per_user_and_address() {
        let failures = BasicFailures::default();
        for _ in 0..FAILURES_ALLOWED - 1 {
            failures.attempt("Admin", "203.0.113.1").unwrap();
        }
        assert!(failures.blocked("admin", "203.0.113.1").is_none());
        failures.attempt(" admin ", "203.0.113.1").unwrap();
        let wait = failures.blocked("ADMIN", "203.0.113.1").unwrap();
        assert!(wait > StdDuration::ZERO && wait <= FAILURE_WINDOW);
        assert!(failures.attempt("admin", "203.0.113.1").unwrap_err() <= wait);
        assert!(failures.attempt("admin", "203.0.113.2").is_ok());
        assert!(failures.attempt("other", "203.0.113.1").is_ok());
        failures.succeeded("admin", "203.0.113.1");
        assert!(failures.attempt("admin", "203.0.113.1").is_ok());
    }

    #[test]
    fn a_block_is_reported_once_and_a_refund_gives_the_attempt_back() {
        let failures = BasicFailures::default();
        for _ in 0..FAILURES_ALLOWED {
            failures.attempt("ci", "a").unwrap();
        }
        assert!(failures.failed("ci", "a"));
        assert!(!failures.failed("ci", "a"));
        failures.refund("ci", "a");
        assert!(failures.blocked("ci", "a").is_none());
        assert!(!failures.failed("ci", "a"));
        failures.attempt("ci", "a").unwrap();
        assert!(failures.blocked("ci", "a").is_some());
    }

    #[test]
    fn the_window_ends() {
        let failures = BasicFailures::with_limits(StdDuration::from_millis(100), HARD_CAP);
        for _ in 0..FAILURES_ALLOWED {
            failures.attempt("ci", "a").unwrap();
        }
        assert!(failures.attempt("ci", "a").is_err());
        std::thread::sleep(StdDuration::from_millis(150));
        assert!(failures.blocked("ci", "a").is_none());
        assert!(failures.attempt("ci", "a").is_ok());
    }

    #[test]
    fn the_map_stays_within_its_cap_and_new_keys_fail_open() {
        let failures = BasicFailures::with_limits(FAILURE_WINDOW, 3);
        for user in ["a", "b", "c"] {
            failures.attempt(user, "x").unwrap();
        }
        failures.attempt("d", "x").unwrap();
        assert!(failures.state().entries.len() <= 3);
        for user in ["a", "b", "c"] {
            for _ in 0..FAILURES_ALLOWED {
                let _ = failures.attempt(user, "x");
            }
        }
        for user in ["a", "b", "c"] {
            assert!(failures.blocked(user, "x").is_some(), "{user}");
        }
        for _ in 0..FAILURES_ALLOWED * 2 {
            assert!(failures.attempt("new", "x").is_ok());
        }
        assert!(failures.blocked("new", "x").is_none());
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
            failures
                .attempt("ci", &key("2001:db8:1:2:aaaa::1"))
                .unwrap();
        }
        assert!(
            failures
                .attempt("ci", &key("2001:db8:1:2:bbbb::2"))
                .is_err()
        );
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
