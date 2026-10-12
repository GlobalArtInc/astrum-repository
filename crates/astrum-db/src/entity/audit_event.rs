use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "audit_event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub at: DateTimeWithTimeZone,
    pub domain: String,
    #[sea_orm(column_name = "type")]
    pub kind: String,
    pub context: String,
    pub initiator: String,
    pub attributes: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
