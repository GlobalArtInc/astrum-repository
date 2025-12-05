use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "api_key")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub realm: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub key_hash: String,
    pub sealed_key: String,
    pub created_at: DateTimeWithTimeZone,
    pub last_used_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
