use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "asset")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub repository: String,
    pub format: String,
    pub path: String,
    pub kind: String,
    pub component_id: Option<i64>,
    pub blob_store: String,
    pub blob_id: Uuid,
    pub size: i64,
    pub content_type: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub md5: String,
    pub attributes: Json,
    pub uploader: String,
    pub uploader_ip: String,
    pub created_at: DateTimeWithTimeZone,
    pub blob_updated_at: DateTimeWithTimeZone,
    pub last_downloaded_at: Option<DateTimeWithTimeZone>,
    pub download_count: i64,
    pub last_verified_at: Option<DateTimeWithTimeZone>,
    pub remote_etag: Option<String>,
    pub remote_last_modified: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
