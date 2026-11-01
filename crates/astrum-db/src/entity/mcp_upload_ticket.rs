use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "mcp_upload_ticket")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ticket_hash: String,
    pub user_id: String,
    pub realm: String,
    pub scope: String,
    pub repository: String,
    pub fields: Json,
    pub file_field: String,
    pub file_name: String,
    pub max_bytes: i64,
    pub client: String,
    pub created_at: DateTimeWithTimeZone,
    pub expires_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
