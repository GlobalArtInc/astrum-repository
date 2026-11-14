use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "policy_exception")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub policy: Option<String>,
    pub rule: String,
    pub advisory: Option<String>,
    pub license: Option<String>,
    pub format: Option<String>,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub reason: String,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub created_by: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
