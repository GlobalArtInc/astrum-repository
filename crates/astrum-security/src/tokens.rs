use std::collections::BTreeSet;
use std::net::IpAddr;

use astrum_db::entity::{access_token, customer};
use astrum_repo::config::valid_name;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::sea_query::{Expr, Func, SimpleExpr};
use sea_orm::{
    ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, DbBackend,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionTrait,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::catalogue::DEFAULT_SOURCE;
use crate::subject::{Scope, Subject, TokenRef};
use crate::{AuthError, Error, Security, ldap, oidc, password, realm};

pub const PREFIX: &str = "astrum_";
const RANDOM_LENGTH: usize = 40;
const HINT_LENGTH: usize = 11;
pub const MAX_PER_USER: u64 = 100;
pub const MAX_PER_CUSTOMER: u64 = 20;
pub const MAX_REPOSITORIES: usize = 100;
const MAX_REPOSITORY_NAME: usize = 200;
pub const MIN_DAYS: i64 = 1;
pub const MAX_DAYS: i64 = 3650;
const TOUCH_EVERY: Duration = Duration::seconds(60);
const LOCK_REPOSITORIES: &str =
    "SELECT name FROM repository WHERE lower(name) = ANY($1) ORDER BY name FOR SHARE";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenInfo {
    pub id: Uuid,
    pub user_id: String,
    pub source: String,
    pub name: String,
    pub hint: String,
    pub scope: Scope,
    pub created_at: DateTime<FixedOffset>,
    pub expires_at: Option<DateTime<FixedOffset>>,
    pub last_used_at: Option<DateTime<FixedOffset>>,
    pub customer: Option<String>,
    pub repositories: Option<Vec<String>>,
    pub created_by: Option<String>,
    pub last_used_ip: Option<String>,
    pub expiry_notice: Option<i16>,
}

#[derive(Clone, Debug)]
pub struct NewToken {
    pub source: String,
    pub external_subject: Option<String>,
    pub name: String,
    pub scope: Scope,
    pub days: Option<i64>,
    pub repositories: Option<Vec<String>>,
    pub customer: Option<String>,
    pub created_by: Option<String>,
}

impl NewToken {
    pub fn personal(name: &str, scope: Scope, days: Option<i64>) -> Self {
        Self {
            source: DEFAULT_SOURCE.to_owned(),
            external_subject: None,
            name: name.to_owned(),
            scope,
            days,
            repositories: None,
            customer: None,
            created_by: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("a token needs a name")]
    Name,
    #[error("a token lives between {MIN_DAYS} and {MAX_DAYS} days, or never expires")]
    Days,
    #[error("a user can hold at most {MAX_PER_USER} tokens")]
    TooMany,
    #[error("a customer can hold at most {MAX_PER_CUSTOMER} tokens")]
    TooManyForCustomer,
    #[error("a token limited to repositories can only read")]
    Scope,
    #[error("{0}")]
    Repositories(String),
    #[error("a customer token needs an expiry")]
    CustomerExpiry,
    #[error("the customer does not exist")]
    UnknownCustomer,
    #[error("a token belongs to a local user, an OpenID Connect account or an LDAP account")]
    UnknownSource,
    #[error("a token of an OpenID Connect account must be bound to the subject of the account")]
    MissingSubject,
    #[error("a customer token belongs to a local user")]
    ExternalCustomer,
    #[error("{}", crate::EXTERNAL_TOKENS_OFF)]
    ExternalOff,
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
        source: row.source,
        name: row.name,
        hint: row.token_hint,
        scope: match row.scope.as_str() {
            "READ" => Scope::Read,
            _ => Scope::Write,
        },
        created_at: row.created_at,
        expires_at: row.expires_at,
        last_used_at: row.last_used_at,
        customer: row.customer,
        repositories: row.repositories,
        created_by: row.created_by,
        last_used_ip: row.last_used_ip,
        expiry_notice: row.expiry_notice,
    }
}

fn same_id_in_any_case(user_id: &str) -> SimpleExpr {
    Expr::expr(Func::lower(Expr::col(access_token::Column::UserId)))
        .eq(Func::lower(Expr::val(user_id)))
}

fn held_by(source: &str, user_id: &str) -> Condition {
    let same = match source == DEFAULT_SOURCE {
        true => access_token::Column::UserId.eq(user_id),
        false => same_id_in_any_case(user_id),
    };
    Condition::all()
        .add(access_token::Column::Source.eq(source))
        .add(same)
}

pub(crate) fn held_by_a_new_local_user(user_id: &str) -> Condition {
    Condition::any().add(held_by(DEFAULT_SOURCE, user_id)).add(
        Condition::all()
            .add(access_token::Column::Source.ne(DEFAULT_SOURCE))
            .add(same_id_in_any_case(user_id)),
    )
}

pub async fn delete_tokens_of_user(
    db: &impl ConnectionTrait,
    source: &str,
    user_id: &str,
) -> Result<u64, Error> {
    let deleted = access_token::Entity::delete_many()
        .filter(held_by(source, user_id))
        .exec(db)
        .await?;
    Ok(deleted.rows_affected)
}

fn normalise(repositories: &[String]) -> Result<Vec<String>, TokenError> {
    let mut names = BTreeSet::new();
    for name in repositories {
        if name.len() > MAX_REPOSITORY_NAME || !valid_name(name) {
            return Err(TokenError::Repositories(format!(
                "\"{name}\" is not a repository name"
            )));
        }
        names.insert(name.to_ascii_lowercase());
    }
    if names.is_empty() {
        return Err(TokenError::Repositories(
            "list at least one repository".to_owned(),
        ));
    }
    if names.len() > MAX_REPOSITORIES {
        return Err(TokenError::Repositories(format!(
            "a token can be limited to at most {MAX_REPOSITORIES} repositories"
        )));
    }
    Ok(names.into_iter().collect())
}

async fn lock_repositories(txn: &DatabaseTransaction, listed: &[String]) -> Result<(), TokenError> {
    let rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            LOCK_REPOSITORIES,
            [listed.to_vec().into()],
        ))
        .await?;
    let mut found = BTreeSet::new();
    for row in &rows {
        let name: String = row.try_get("", "name")?;
        found.insert(name.to_ascii_lowercase());
    }
    match listed.iter().find(|name| !found.contains(*name)) {
        Some(missing) => Err(TokenError::Repositories(format!(
            "the repository \"{missing}\" does not exist"
        ))),
        None => Ok(()),
    }
}

impl Security {
    pub async fn create_token(
        &self,
        user_id: &str,
        new: NewToken,
    ) -> Result<(TokenInfo, String), TokenError> {
        let name = new.name.trim();
        if name.is_empty() || name.len() > 200 {
            return Err(TokenError::Name);
        }
        if new
            .days
            .is_some_and(|days| !(MIN_DAYS..=MAX_DAYS).contains(&days))
        {
            return Err(TokenError::Days);
        }
        match new.source.as_str() {
            DEFAULT_SOURCE => {}
            oidc::SOURCE | ldap::SOURCE => {
                if new.customer.is_some() {
                    return Err(TokenError::ExternalCustomer);
                }
                if !self.external_tokens_enabled() {
                    return Err(TokenError::ExternalOff);
                }
                if new.source == oidc::SOURCE
                    && new
                        .external_subject
                        .as_deref()
                        .is_none_or(|subject| subject.is_empty())
                {
                    return Err(TokenError::MissingSubject);
                }
            }
            _ => return Err(TokenError::UnknownSource),
        }
        let repositories = new.repositories.as_deref().map(normalise).transpose()?;
        if repositories.is_some() && new.scope != Scope::Read {
            return Err(TokenError::Scope);
        }
        if new.customer.is_some() {
            if repositories.is_none() {
                return Err(TokenError::Repositories(
                    "a customer token needs a list of repositories".to_owned(),
                ));
            }
            if new.days.is_none() {
                return Err(TokenError::CustomerExpiry);
            }
        }
        let now = Utc::now().fixed_offset();
        let txn = self.db.begin().await?;
        match new.customer.as_deref() {
            Some(customer_id) => {
                customer::Entity::find_by_id(customer_id)
                    .lock_exclusive()
                    .one(&txn)
                    .await?
                    .ok_or(TokenError::UnknownCustomer)?;
                let held = access_token::Entity::find()
                    .filter(access_token::Column::Customer.eq(customer_id))
                    .filter(
                        Condition::any()
                            .add(access_token::Column::ExpiresAt.is_null())
                            .add(access_token::Column::ExpiresAt.gt(now)),
                    )
                    .count(&txn)
                    .await?;
                if held >= MAX_PER_CUSTOMER {
                    return Err(TokenError::TooManyForCustomer);
                }
            }
            None => {
                let held = access_token::Entity::find()
                    .filter(access_token::Column::UserId.eq(user_id))
                    .filter(access_token::Column::Source.eq(new.source.as_str()))
                    .filter(access_token::Column::Customer.is_null())
                    .count(&txn)
                    .await?;
                if held >= MAX_PER_USER {
                    return Err(TokenError::TooMany);
                }
            }
        }
        if let Some(listed) = &repositories {
            lock_repositories(&txn, listed).await?;
        }
        let token = generate();
        let row = access_token::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user_id.to_owned()),
            source: ActiveValue::Set(new.source),
            name: ActiveValue::Set(name.to_owned()),
            token_hash: ActiveValue::Set(hash(&token)),
            token_hint: ActiveValue::Set(token[..HINT_LENGTH].to_owned()),
            scope: ActiveValue::Set(
                match new.scope {
                    Scope::Read => "READ",
                    Scope::Write => "WRITE",
                }
                .to_owned(),
            ),
            created_at: ActiveValue::Set(now),
            expires_at: ActiveValue::Set(new.days.map(|days| now + Duration::days(days))),
            last_used_at: ActiveValue::Set(None),
            customer: ActiveValue::Set(new.customer),
            repositories: ActiveValue::Set(repositories),
            created_by: ActiveValue::Set(new.created_by),
            last_used_ip: ActiveValue::Set(None),
            expiry_notice: ActiveValue::Set(None),
            external_subject: ActiveValue::Set(new.external_subject),
        };
        let stored = access_token::Entity::insert(row)
            .exec_with_returning(&txn)
            .await?;
        txn.commit().await?;
        Ok((info(stored), token))
    }

    pub async fn tokens(&self, user_id: Option<&str>) -> Result<Vec<TokenInfo>, Error> {
        let mut query = access_token::Entity::find()
            .filter(access_token::Column::Customer.is_null())
            .order_by_asc(access_token::Column::CreatedAt);
        if let Some(user_id) = user_id {
            query = query.filter(access_token::Column::UserId.eq(user_id));
        }
        Ok(query.all(&self.db).await?.into_iter().map(info).collect())
    }

    pub async fn tokens_of(&self, source: &str, user_id: &str) -> Result<Vec<TokenInfo>, Error> {
        Ok(access_token::Entity::find()
            .filter(access_token::Column::Customer.is_null())
            .filter(access_token::Column::Source.eq(source))
            .filter(access_token::Column::UserId.eq(user_id))
            .order_by_asc(access_token::Column::CreatedAt)
            .all(&self.db)
            .await?
            .into_iter()
            .map(info)
            .collect())
    }

    pub async fn token_of(
        &self,
        source: &str,
        user_id: &str,
        id: Uuid,
    ) -> Result<Option<TokenInfo>, Error> {
        Ok(access_token::Entity::find_by_id(id)
            .filter(access_token::Column::Customer.is_null())
            .filter(access_token::Column::Source.eq(source))
            .filter(access_token::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?
            .map(info))
    }

    pub async fn token(&self, id: Uuid) -> Result<Option<TokenInfo>, Error> {
        Ok(access_token::Entity::find_by_id(id)
            .filter(access_token::Column::Customer.is_null())
            .one(&self.db)
            .await?
            .map(info))
    }

    pub async fn delete_token(&self, id: Uuid) -> Result<bool, Error> {
        let deleted = access_token::Entity::delete_many()
            .filter(access_token::Column::Id.eq(id))
            .filter(access_token::Column::Customer.is_null())
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn customer_tokens(&self, customer_id: &str) -> Result<Vec<TokenInfo>, Error> {
        Ok(access_token::Entity::find()
            .filter(access_token::Column::Customer.eq(customer_id))
            .order_by_asc(access_token::Column::CreatedAt)
            .all(&self.db)
            .await?
            .into_iter()
            .map(info)
            .collect())
    }

    pub async fn revoke_customer_token(&self, customer_id: &str, id: Uuid) -> Result<bool, Error> {
        let deleted = access_token::Entity::delete_many()
            .filter(access_token::Column::Id.eq(id))
            .filter(access_token::Column::Customer.eq(customer_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn revoke_customer_tokens(&self, customer_id: &str) -> Result<u64, Error> {
        let deleted = access_token::Entity::delete_many()
            .filter(access_token::Column::Customer.eq(customer_id))
            .exec(&self.db)
            .await?;
        Ok(deleted.rows_affected)
    }

    pub async fn delete_tokens_of(&self, source: &str, user_id: &str) -> Result<u64, Error> {
        delete_tokens_of_user(&self.db, source, user_id).await
    }

    pub async fn authenticate_token(
        &self,
        user_id: Option<&str>,
        token: &str,
        address: Option<IpAddr>,
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
        if row.expires_at.is_some_and(|expires| expires <= now) {
            return Err(AuthError::BadCredentials);
        }
        if user_id
            .is_some_and(|given| given != row.user_id && row.customer.as_deref() != Some(given))
        {
            return Err(AuthError::BadCredentials);
        }
        let bound_subject = row.external_subject.clone();
        let token = info(row);
        if token.source != DEFAULT_SOURCE
            && (token.customer.is_some() || !self.external_tokens_enabled())
        {
            return Err(AuthError::BadCredentials);
        }
        if let Some(customer_id) = token.customer.as_deref() {
            let active = customer::Entity::find_by_id(customer_id)
                .one(&self.db)
                .await
                .map_err(Error::from)?
                .is_some_and(|found| !found.disabled);
            if !active {
                return Err(AuthError::BadCredentials);
            }
        }
        let limit = match (&token.customer, &token.repositories) {
            (_, Some(repositories)) if !repositories.is_empty() => {
                Some(repositories.iter().cloned().collect::<BTreeSet<_>>())
            }
            (None, None) => None,
            _ => {
                tracing::error!(token = %token.id, "a token has a customer or a repository list that cannot be applied; refused");
                return Err(AuthError::BadCredentials);
            }
        };
        let subject = match token.source.as_str() {
            DEFAULT_SOURCE => model.local_subject(&token.user_id, realm::ACCESS_TOKEN),
            oidc::SOURCE => model.oidc_token_subject(&token.user_id, bound_subject.as_deref(), now),
            ldap::SOURCE => match self.ldap_subject(&token.user_id, realm::ACCESS_TOKEN).await {
                Ok(Some(subject)) if subject.user_id == token.user_id => Some(subject),
                Ok(_) => None,
                Err(error) => {
                    tracing::warn!(user = %token.user_id, %error, "LDAP cannot confirm an access token");
                    None
                }
            },
            _ => None,
        };
        let mut subject = subject
            .ok_or(AuthError::BadCredentials)?
            .with_scope(token.scope)
            .with_token(TokenRef {
                id: token.id,
                customer: token.customer.clone(),
            });
        if let Some(limit) = limit {
            subject = subject.narrowed_to(&limit);
        }
        if token
            .last_used_at
            .is_none_or(|used| now - used > TOUCH_EVERY)
        {
            access_token::Entity::update(access_token::ActiveModel {
                id: ActiveValue::Unchanged(token.id),
                last_used_at: ActiveValue::Set(Some(now)),
                last_used_ip: match address {
                    Some(address) => ActiveValue::Set(Some(address.to_string())),
                    None => ActiveValue::NotSet,
                },
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
