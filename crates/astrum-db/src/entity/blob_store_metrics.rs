use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "blob_store_metrics")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub store: String,
    pub blob_count: i64,
    pub total_bytes: i64,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
