use astrum_db::entity::trusted_key;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, QueryOrder, SqlErr};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedKey {
    pub name: String,
    pub algorithm: String,
    pub public_key: String,
    pub fingerprint: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

impl TrustedKey {
    fn from_row(row: trusted_key::Model) -> Self {
        Self {
            name: row.name,
            algorithm: row.algorithm,
            public_key: row.public_key,
            fingerprint: row.fingerprint,
            notes: row.notes,
            created_at: row.created_at.with_timezone(&Utc),
        }
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<TrustedKey>, Error> {
    Ok(trusted_key::Entity::find()
        .order_by_asc(trusted_key::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(TrustedKey::from_row)
        .collect())
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<TrustedKey>, Error> {
    Ok(trusted_key::Entity::find_by_id(name.to_owned())
        .one(db)
        .await?
        .map(TrustedKey::from_row))
}

pub async fn create(db: &impl ConnectionTrait, key: &TrustedKey) -> Result<(), Error> {
    if key.public_key.contains("PRIVATE KEY") {
        return Err(Error::Invalid(
            "a private key cannot be stored as a trusted key".to_owned(),
        ));
    }
    let row = trusted_key::ActiveModel {
        name: ActiveValue::Set(key.name.clone()),
        algorithm: ActiveValue::Set(key.algorithm.clone()),
        public_key: ActiveValue::Set(key.public_key.clone()),
        fingerprint: ActiveValue::Set(key.fingerprint.clone()),
        notes: ActiveValue::Set(key.notes.clone()),
        created_at: ActiveValue::Set(key.created_at.fixed_offset()),
    };
    trusted_key::Entity::insert(row)
        .exec(db)
        .await
        .map_err(|error| match error.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => Error::Duplicate(format!(
                "the trusted key \"{}\" or its fingerprint",
                key.name
            )),
            _ => Error::Database(error),
        })?;
    Ok(())
}

pub async fn delete(db: &impl ConnectionTrait, name: &str) -> Result<bool, Error> {
    let deleted = trusted_key::Entity::delete_by_id(name.to_owned())
        .exec(db)
        .await?;
    Ok(deleted.rows_affected > 0)
}
