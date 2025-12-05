use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ansible_import")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub repository: String,
    pub state: String,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub messages: Json,
    pub error: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub finished_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
