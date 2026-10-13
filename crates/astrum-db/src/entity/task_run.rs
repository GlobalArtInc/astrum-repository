use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_run")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub task_id: Uuid,
    pub state: String,
    pub trigger: String,
    pub initiator: String,
    pub node: String,
    pub message: String,
    pub counts: Json,
    pub cancel_requested: bool,
    pub queued_at: DateTimeWithTimeZone,
    pub started_at: Option<DateTimeWithTimeZone>,
    pub finished_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
