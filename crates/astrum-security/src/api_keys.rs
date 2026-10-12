use astrum_db::entity::api_key;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::secrets::SecretBox;
use crate::subject::Subject;
use crate::tokens::hash;
use crate::{AuthError, Error, Security};

pub const NUGET: &str = "NuGetApiKey";
pub const NUGET_HEADER: &str = "x-nuget-apikey";
const TOUCH_EVERY: Duration = Duration::seconds(60);
const MAX_LENGTH: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiKeyInfo {
    pub created_at: DateTime<FixedOffset>,
    pub last_used_at: Option<DateTime<FixedOffset>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiKey {
    pub key: String,
    pub created_at: DateTime<FixedOffset>,
    pub created: bool,
}

fn find(realm: &str, user_id: &str) -> sea_orm::Select<api_key::Entity> {
    api_key::Entity::find()
        .filter(api_key::Column::Realm.eq(realm))
        .filter(api_key::Column::UserId.eq(user_id))
}

impl Security {
    pub async fn api_key_info(
        &self,
        realm: &str,
        user_id: &str,
    ) -> Result<Option<ApiKeyInfo>, Error> {
        Ok(find(realm, user_id)
            .one(&self.db)
            .await?
            .map(|row| ApiKeyInfo {
                created_at: row.created_at,
                last_used_at: row.last_used_at,
            }))
    }

    pub async fn api_key(
        &self,
        secrets: &SecretBox,
        realm: &str,
        user_id: &str,
    ) -> Result<ApiKey, Error> {
        if let Some(row) = find(realm, user_id).one(&self.db).await? {
            return Ok(ApiKey {
                key: secrets.open(&row.sealed_key)?,
                created_at: row.created_at,
                created: false,
            });
        }
        let fresh = Uuid::new_v4().to_string();
        let now = Utc::now().fixed_offset();
        let inserted = api_key::Entity::insert(api_key::ActiveModel {
            realm: ActiveValue::Set(realm.to_owned()),
            user_id: ActiveValue::Set(user_id.to_owned()),
            key_hash: ActiveValue::Set(hash(&fresh)),
            sealed_key: ActiveValue::Set(secrets.seal(&fresh)),
            created_at: ActiveValue::Set(now),
            last_used_at: ActiveValue::Set(None),
        })
        .on_conflict(
            OnConflict::columns([api_key::Column::Realm, api_key::Column::UserId])
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(&self.db)
        .await?;
        if inserted == 1 {
            return Ok(ApiKey {
                key: fresh,
                created_at: now,
                created: true,
            });
        }
        let row = find(realm, user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Error::Query(sea_orm::DbErr::RecordNotFound("api key".to_owned())))?;
        Ok(ApiKey {
            key: secrets.open(&row.sealed_key)?,
            created_at: row.created_at,
            created: false,
        })
    }

    pub async fn reset_api_key(
        &self,
        secrets: &SecretBox,
        realm: &str,
        user_id: &str,
    ) -> Result<ApiKey, Error> {
        self.delete_api_key(realm, user_id).await?;
        self.api_key(secrets, realm, user_id).await
    }

    pub async fn delete_api_key(&self, realm: &str, user_id: &str) -> Result<bool, Error> {
        let deleted = api_key::Entity::delete_many()
            .filter(api_key::Column::Realm.eq(realm))
            .filter(api_key::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn delete_api_keys_of(&self, user_id: &str) -> Result<u64, Error> {
        let deleted = api_key::Entity::delete_many()
            .filter(api_key::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn authenticate_api_key(&self, realm: &str, key: &str) -> Result<Subject, AuthError> {
        let key = key.trim();
        if key.is_empty() || key.len() > MAX_LENGTH {
            return Err(AuthError::BadCredentials);
        }
        let model = self.model();
        if !model.realm_active(realm) {
            return Err(AuthError::BadCredentials);
        }
        let row = api_key::Entity::find()
            .filter(api_key::Column::Realm.eq(realm))
            .filter(api_key::Column::KeyHash.eq(hash(key)))
            .one(&self.db)
            .await
            .map_err(Error::from)?
            .ok_or(AuthError::BadCredentials)?;
        let subject = model
            .local_subject(&row.user_id, realm)
            .ok_or(AuthError::BadCredentials)?;
        let now = Utc::now().fixed_offset();
        if row.last_used_at.is_none_or(|used| now - used > TOUCH_EVERY) {
            api_key::Entity::update(api_key::ActiveModel {
                realm: ActiveValue::Unchanged(row.realm),
                user_id: ActiveValue::Unchanged(row.user_id),
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
