use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "repository")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    pub format: String,
    #[sea_orm(column_name = "type")]
    pub kind: String,
    pub online: bool,
    pub blob_store: Option<String>,
    pub attributes: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
