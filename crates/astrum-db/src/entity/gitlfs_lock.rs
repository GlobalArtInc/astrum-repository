use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "gitlfs_lock")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub repository: String,
    pub path: String,
    pub ref_name: Option<String>,
    pub owner: String,
    pub locked_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
