use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "blob_store")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    #[sea_orm(column_name = "type")]
    pub kind: String,
    pub attributes: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
