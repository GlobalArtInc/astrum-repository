use astrum_db::entity::{access_token, api_key, format_token, oauth_grant, user_session};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::Error;
use crate::catalogue::DEFAULT_SOURCE;
use crate::oauth::granted_in;
use crate::realm;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClearedCredentials {
    pub access_tokens: u64,
    pub api_keys: u64,
    pub format_tokens: u64,
    pub oauth_grants: u64,
    pub sessions: u64,
}

impl ClearedCredentials {
    pub fn total(&self) -> u64 {
        self.access_tokens + self.api_keys + self.format_tokens + self.oauth_grants + self.sessions
    }
}

pub async fn clear_credentials_of_new_local_user(
    db: &impl ConnectionTrait,
    user_id: &str,
) -> Result<ClearedCredentials, Error> {
    let access_tokens = access_token::Entity::delete_many()
        .filter(access_token::Column::UserId.eq(user_id))
        .exec(db)
        .await?
        .rows_affected;
    let api_keys = api_key::Entity::delete_many()
        .filter(api_key::Column::UserId.eq(user_id))
        .exec(db)
        .await?
        .rows_affected;
    let format_tokens = format_token::Entity::delete_many()
        .filter(format_token::Column::UserId.eq(user_id))
        .exec(db)
        .await?
        .rows_affected;
    let oauth_grants = oauth_grant::Entity::delete_many()
        .filter(oauth_grant::Column::UserId.eq(user_id))
        .filter(granted_in(DEFAULT_SOURCE))
        .exec(db)
        .await?
        .rows_affected;
    let sessions = user_session::Entity::delete_many()
        .filter(user_session::Column::UserId.eq(user_id))
        .filter(user_session::Column::Realm.is_not_in([realm::LDAP, realm::OIDC]))
        .exec(db)
        .await?
        .rows_affected;
    Ok(ClearedCredentials {
        access_tokens,
        api_keys,
        format_tokens,
        oauth_grants,
        sessions,
    })
}
