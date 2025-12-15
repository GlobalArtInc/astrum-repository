use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ConnectionTrait};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "setting")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: Json,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<T: DeserializeOwned>(
    db: &impl ConnectionTrait,
    key: &str,
) -> Result<Option<T>, Error> {
    let Some(row) = Entity::find_by_id(key.to_owned()).one(db).await? else {
        return Ok(None);
    };
    serde_json::from_value(row.value)
        .map(Some)
        .map_err(|source| Error::Setting {
            key: key.to_owned(),
            source,
        })
}

pub async fn put<T: Serialize>(
    db: &impl ConnectionTrait,
    key: &str,
    value: &T,
) -> Result<(), Error> {
    let value = serde_json::to_value(value).map_err(|source| Error::Setting {
        key: key.to_owned(),
        source,
    })?;
    let row = ActiveModel {
        key: ActiveValue::Set(key.to_owned()),
        value: ActiveValue::Set(value),
        updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
    };
    Entity::insert(row)
        .on_conflict(
            OnConflict::column(Column::Key)
                .update_columns([Column::Value, Column::UpdatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

pub async fn delete(db: &impl ConnectionTrait, key: &str) -> Result<bool, Error> {
    let result = Entity::delete_by_id(key.to_owned()).exec(db).await?;
    Ok(result.rows_affected > 0)
}
