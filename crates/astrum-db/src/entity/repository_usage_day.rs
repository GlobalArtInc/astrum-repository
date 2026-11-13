use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "repository_usage_day")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub repository: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub day: Date,
    pub components: i64,
    pub assets: i64,
    pub bytes: i64,
    pub downloads: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
