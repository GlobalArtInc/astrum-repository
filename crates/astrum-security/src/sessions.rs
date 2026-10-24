use std::time::{Duration as StdDuration, Instant};

use astrum_db::entity::user_session as session;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rand::RngCore;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewSession {
    pub token: String,
    pub csrf: String,
}

#[derive(Clone, Debug)]
pub struct ActiveSession {
    pub subject: Subject,
    pub csrf: String,
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
    ) -> Result<NewSession, Error> {
        let model = self.model();
        let stamp = credential_stamp(
            model
                .users
                .get(&subject.user_id)
                .filter(|_| subject.source == DEFAULT_SOURCE || subject.source.is_empty())
                .and_then(|user| user.password_hash.as_deref()),
        );
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
        })
        .exec(&self.db)
        .await?;
        Ok(NewSession { token, csrf })
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
