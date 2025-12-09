use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "format_token")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub token_hash: String,
    pub realm: String,
    pub user_id: String,
    pub created_at: DateTimeWithTimeZone,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub last_used_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
