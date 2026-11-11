use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_second_factor")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    pub requirement: String,
    pub enroll_by: Option<DateTimeWithTimeZone>,
    pub totp_secret: Option<String>,
    pub totp_confirmed_at: Option<DateTimeWithTimeZone>,
    pub totp_last_step: i64,
    pub webauthn_handle: Uuid,
    pub failed_codes: i32,
    pub failed_since: Option<DateTimeWithTimeZone>,
    pub locked_until: Option<DateTimeWithTimeZone>,
    pub lock_level: i32,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
