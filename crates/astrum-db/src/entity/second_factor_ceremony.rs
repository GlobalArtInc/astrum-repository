use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "second_factor_ceremony")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub handle_hash: String,
    pub kind: String,
    pub user_id: String,
    pub realm: String,
    pub state: Option<Json>,
    pub attempts: i32,
    pub client_ip: String,
    pub created_at: DateTimeWithTimeZone,
    pub expires_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
