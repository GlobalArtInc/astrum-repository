use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "share_link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub secret_hash: String,
    pub repository: String,
    pub path: String,
    pub prefix: bool,
    pub note: String,
    pub created_by: String,
    pub created_at: DateTimeWithTimeZone,
    pub expires_at: DateTimeWithTimeZone,
    pub max_downloads: Option<i32>,
    pub downloads: i32,
    pub bytes_served: i64,
    pub last_used_at: Option<DateTimeWithTimeZone>,
    pub last_used_ip: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
