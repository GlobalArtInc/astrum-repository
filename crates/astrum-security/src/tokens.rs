use astrum_db::entity::access_token;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::catalogue::DEFAULT_SOURCE;
use crate::subject::{Scope, Subject};
use crate::{AuthError, Error, Security, password, realm};

pub const PREFIX: &str = "astrum_";
const RANDOM_LENGTH: usize = 40;
const HINT_LENGTH: usize = 11;
pub const MAX_PER_USER: u64 = 100;
pub const MIN_DAYS: i64 = 1;
pub const MAX_DAYS: i64 = 3650;
const TOUCH_EVERY: Duration = Duration::seconds(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenInfo {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub hint: String,
    pub scope: Scope,
    pub created_at: DateTime<FixedOffset>,
    pub expires_at: DateTime<FixedOffset>,
    pub last_used_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("a token needs a name")]
    Name,
    #[error("a token lives between {MIN_DAYS} and {MAX_DAYS} days")]
    Days,
    #[error("a user can hold at most {MAX_PER_USER} tokens")]
    TooMany,
    #[error(transparent)]
    Internal(#[from] Error),
}

impl From<sea_orm::DbErr> for TokenError {
    fn from(error: sea_orm::DbErr) -> Self {
        TokenError::Internal(error.into())
    }
}

pub fn looks_like_token(value: &str) -> bool {
    value.len() == PREFIX.len() + RANDOM_LENGTH
        && value.starts_with(PREFIX)
        && value[PREFIX.len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
}

pub fn hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn generate() -> String {
    format!("{PREFIX}{}", password::random(RANDOM_LENGTH))
}

fn info(row: access_token::Model) -> TokenInfo {
    TokenInfo {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        hint: row.token_hint,
        scope: match row.scope.as_str() {
            "READ" => Scope::Read,
            _ => Scope::Write,
        },
        created_at: row.created_at,
        expires_at: row.expires_at,
        last_used_at: row.last_used_at,
    }
}

impl Security {
    pub async fn create_token(
        &self,
        user_id: &str,
        name: &str,
        scope: Scope,
        days: i64,
    ) -> Result<(TokenInfo, String), TokenError> {
        let name = name.trim();
        if name.is_empty() || name.len() > 200 {
            return Err(TokenError::Name);
        }
        if !(MIN_DAYS..=MAX_DAYS).contains(&days) {
            return Err(TokenError::Days);
        }
        let held = access_token::Entity::find()
            .filter(access_token::Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;
        if held >= MAX_PER_USER {
            return Err(TokenError::TooMany);
        }
        let token = generate();
        let now = Utc::now().fixed_offset();
        let row = access_token::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user_id.to_owned()),
            source: ActiveValue::Set(DEFAULT_SOURCE.to_owned()),
            name: ActiveValue::Set(name.to_owned()),
            token_hash: ActiveValue::Set(hash(&token)),
            token_hint: ActiveValue::Set(token[..HINT_LENGTH].to_owned()),
            scope: ActiveValue::Set(
                match scope {
                    Scope::Read => "READ",
                    Scope::Write => "WRITE",
                }
                .to_owned(),
            ),
            created_at: ActiveValue::Set(now),
            expires_at: ActiveValue::Set(now + Duration::days(days)),
            last_used_at: ActiveValue::Set(None),
        };
        let stored = access_token::Entity::insert(row)
            .exec_with_returning(&self.db)
            .await?;
        Ok((info(stored), token))
    }

    pub async fn tokens(&self, user_id: Option<&str>) -> Result<Vec<TokenInfo>, Error> {
        let mut query = access_token::Entity::find().order_by_asc(access_token::Column::CreatedAt);
        if let Some(user_id) = user_id {
            query = query.filter(access_token::Column::UserId.eq(user_id));
        }
        Ok(query.all(&self.db).await?.into_iter().map(info).collect())
    }

    pub async fn token(&self, id: Uuid) -> Result<Option<TokenInfo>, Error> {
        Ok(access_token::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .map(info))
    }

    pub async fn delete_token(&self, id: Uuid) -> Result<bool, Error> {
        let deleted = access_token::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn delete_tokens_of(&self, user_id: &str) -> Result<u64, Error> {
        let deleted = access_token::Entity::delete_many()
            .filter(access_token::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn authenticate_token(
        &self,
        user_id: Option<&str>,
        token: &str,
    ) -> Result<Subject, AuthError> {
        let model = self.model();
        if !model.realm_active(realm::ACCESS_TOKEN) || !looks_like_token(token) {
            return Err(AuthError::BadCredentials);
        }
        let row = access_token::Entity::find()
            .filter(access_token::Column::TokenHash.eq(hash(token)))
            .one(&self.db)
            .await
            .map_err(Error::from)?
            .ok_or(AuthError::BadCredentials)?;
        let now = Utc::now().fixed_offset();
        if row.expires_at <= now {
            return Err(AuthError::BadCredentials);
        }
        if user_id.is_some_and(|user_id| user_id != row.user_id) {
            return Err(AuthError::BadCredentials);
        }
        let token = info(row);
        let subject = model
            .local_subject(&token.user_id, realm::ACCESS_TOKEN)
            .ok_or(AuthError::BadCredentials)?
            .with_scope(token.scope);
        if token
            .last_used_at
            .is_none_or(|used| now - used > TOUCH_EVERY)
        {
            access_token::Entity::update(access_token::ActiveModel {
                id: ActiveValue::Unchanged(token.id),
                last_used_at: ActiveValue::Set(Some(now)),
                ..Default::default()
            })
            .exec(&self.db)
            .await
            .map_err(Error::from)?;
        }
        Ok(subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_have_a_recognisable_shape() {
        let token = generate();
        assert!(looks_like_token(&token), "{token}");
        assert_eq!(token.len(), 47);
        assert!(!looks_like_token("astrum_short"));
        assert!(!looks_like_token(&token.replace("astrum_", "other__")));
        assert!(!looks_like_token(&format!("{}!", &token[..46])));
        assert_ne!(generate(), token);
    }

    #[test]
    fn the_stored_hash_is_hex_sha256() {
        assert_eq!(
            hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
