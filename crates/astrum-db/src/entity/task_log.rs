use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub run_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub line: i32,
    pub at: DateTimeWithTimeZone,
    pub level: String,
    pub message: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
