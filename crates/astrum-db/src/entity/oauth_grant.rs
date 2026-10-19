use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_grant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub client_id: String,
    pub user_id: String,
    pub realm: String,
    pub scope: String,
    pub resource: Option<String>,
    pub access_hash: String,
    pub access_expires_at: DateTimeWithTimeZone,
    pub refresh_hash: String,
    pub refresh_expires_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub last_used_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
