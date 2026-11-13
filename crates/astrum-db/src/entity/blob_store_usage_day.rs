use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "blob_store_usage_day")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub store: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub day: Date,
    pub blob_count: i64,
    pub total_bytes: i64,
    pub available_bytes: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
