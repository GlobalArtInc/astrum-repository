use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "notification_channel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_name = "type")]
    pub kind: String,
    pub enabled: bool,
    pub chat_id: Option<String>,
    pub message_thread_id: Option<i64>,
    pub recipients: Vec<String>,
    pub secret: Option<String>,
    pub target_host: Option<String>,
    pub repositories: Vec<String>,
    pub events: Vec<String>,
    pub language: String,
    pub proxy_mode: String,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub signing_secret: Option<String>,
    pub authorization_header: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
