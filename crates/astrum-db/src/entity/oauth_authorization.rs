use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_authorization")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub requested_scope: String,
    pub state: Option<String>,
    pub resource: Option<String>,
    pub user_id: Option<String>,
    pub realm: Option<String>,
    pub granted_scope: Option<String>,
    pub code_hash: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub expires_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
