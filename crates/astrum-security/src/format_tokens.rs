use astrum_db::entity::format_token;
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};

use crate::subject::Subject;
use crate::tokens::hash;
use crate::{AuthError, Error, Security, password};

const TOUCH_EVERY: Duration = Duration::seconds(60);
const RANDOM_LENGTH: usize = 40;

pub fn looks_like_format_token(value: &str) -> Option<&str> {
    let (realm, secret) = value.split_once('.')?;
    let known = crate::realm::AVAILABLE
        .iter()
        .any(|candidate| candidate.id == realm);
    (known
        && secret.len() >= 32
        && secret
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-'))
    .then_some(realm)
}

impl Security {
    pub async fn issue_format_token(
        &self,
        realm: &str,
        user_id: &str,
        lifetime: Option<Duration>,
    ) -> Result<String, Error> {
        let token = format!("{realm}.{}", password::random(RANDOM_LENGTH));
        let now = Utc::now().fixed_offset();
        format_token::Entity::insert(format_token::ActiveModel {
            token_hash: ActiveValue::Set(hash(&token)),
            realm: ActiveValue::Set(realm.to_owned()),
            user_id: ActiveValue::Set(user_id.to_owned()),
            created_at: ActiveValue::Set(now),
            expires_at: ActiveValue::Set(lifetime.map(|lifetime| now + lifetime)),
            last_used_at: ActiveValue::Set(None),
        })
        .exec(&self.db)
        .await?;
        Ok(token)
    }

    pub async fn revoke_format_token(&self, token: &str) -> Result<bool, Error> {
        let deleted = format_token::Entity::delete_by_id(hash(token))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn revoke_format_tokens_of(&self, user_id: &str) -> Result<u64, Error> {
        let deleted = format_token::Entity::delete_many()
            .filter(format_token::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn revoke_format_tokens_of_realm(
        &self,
        user_id: &str,
        realm: &str,
    ) -> Result<u64, Error> {
        let deleted = format_token::Entity::delete_many()
            .filter(format_token::Column::UserId.eq(user_id))
            .filter(format_token::Column::Realm.eq(realm))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn authenticate_format_token(&self, token: &str) -> Result<Subject, AuthError> {
        let Some(realm) = looks_like_format_token(token) else {
            return Err(AuthError::BadCredentials);
        };
        let model = self.model();
        if !model.realm_active(realm) {
            return Err(AuthError::BadCredentials);
        }
        let row = format_token::Entity::find_by_id(hash(token))
            .one(&self.db)
            .await
            .map_err(Error::from)?
            .ok_or(AuthError::BadCredentials)?;
        let now = Utc::now().fixed_offset();
        if row.realm != realm || row.expires_at.is_some_and(|expires| expires <= now) {
            return Err(AuthError::BadCredentials);
        }
        let subject = model
            .local_subject(&row.user_id, realm)
            .ok_or(AuthError::BadCredentials)?;
        if row.last_used_at.is_none_or(|used| now - used > TOUCH_EVERY) {
            format_token::Entity::update(format_token::ActiveModel {
                token_hash: ActiveValue::Unchanged(row.token_hash),
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
    fn format_tokens_name_their_realm() {
        let token = format!("NpmToken.{}", "a".repeat(40));
        assert_eq!(looks_like_format_token(&token), Some("NpmToken"));
        assert_eq!(
            looks_like_format_token(&format!("DockerToken.{}", "b".repeat(40))),
            Some("DockerToken")
        );
        assert_eq!(looks_like_format_token("NpmToken.short"), None);
        assert_eq!(
            looks_like_format_token(&format!("Unknown.{}", "a".repeat(40))),
            None
        );
        assert_eq!(looks_like_format_token("no-dot"), None);
    }
}
