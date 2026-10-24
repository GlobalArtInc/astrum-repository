use std::path::{Path, PathBuf};

use astrum_db::entity::{security_role, security_user, security_user_role};
use astrum_db::{revision, setting};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, PaginatorTrait, TransactionTrait};
use serde_json::json;

use crate::catalogue::{
    self, ADMIN_ROLE, ADMIN_USER, ANONYMOUS_ROLE, ANONYMOUS_USER, DEFAULT_SOURCE,
};
use crate::model::{ANONYMOUS_KEY, AnonymousSettings, REALMS_KEY, UserStatus};
use crate::{Error, password, realm, secrets};

pub const ADMIN_PASSWORD_FILE: &str = "admin.password";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Bootstrapped {
    pub admin_password_file: Option<PathBuf>,
}

pub async fn bootstrap<C>(db: &C, data_dir: &Path) -> Result<Bootstrapped, Error>
where
    C: ConnectionTrait + TransactionTrait,
{
    let txn = db.begin().await?;
    let now = chrono::Utc::now().fixed_offset();

    for role in catalogue::ROLES {
        let row = security_role::ActiveModel {
            id: ActiveValue::Set(role.id.to_owned()),
            source: ActiveValue::Set(DEFAULT_SOURCE.to_owned()),
            name: ActiveValue::Set(role.id.to_owned()),
            description: ActiveValue::Set(role.description.to_owned()),
            privileges: ActiveValue::Set(json!(role.privileges)),
            roles: ActiveValue::Set(json!([])),
            read_only: ActiveValue::Set(true),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };
        security_role::Entity::insert(row)
            .on_conflict(
                OnConflict::column(security_role::Column::Id)
                    .update_columns([
                        security_role::Column::Name,
                        security_role::Column::Description,
                        security_role::Column::Privileges,
                        security_role::Column::Roles,
                        security_role::Column::ReadOnly,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await?;
    }

    if setting::get::<Vec<String>>(&txn, REALMS_KEY)
        .await?
        .is_none()
    {
        setting::put(&txn, REALMS_KEY, &realm::DEFAULT_ACTIVE).await?;
    }
    if setting::get::<AnonymousSettings>(&txn, ANONYMOUS_KEY)
        .await?
        .is_none()
    {
        setting::put(&txn, ANONYMOUS_KEY, &AnonymousSettings::default()).await?;
    }

    let mut result = Bootstrapped::default();
    if security_user::Entity::find().count(&txn).await? == 0 {
        let admin_password = password::random(24);
        insert_user(
            &txn,
            ADMIN_USER,
            "Administrator",
            "admin@example.org",
            Some(password::hash(&admin_password)),
            ADMIN_ROLE,
        )
        .await?;
        insert_user(
            &txn,
            ANONYMOUS_USER,
            "Anonymous",
            "anonymous@example.org",
            None,
            ANONYMOUS_ROLE,
        )
        .await?;
        let path = data_dir.join(ADMIN_PASSWORD_FILE);
        let _ = std::fs::remove_file(&path);
        secrets::write_private(&path, admin_password.as_bytes())?;
        tracing::info!(path = %path.display(), "created the admin user; its first password is in this file");
        result.admin_password_file = Some(path);
    }

    revision::bump(&txn, revision::SECURITY).await?;
    txn.commit().await?;
    Ok(result)
}

async fn insert_user(
    db: &impl ConnectionTrait,
    id: &str,
    first_name: &str,
    email: &str,
    password_hash: Option<String>,
    role: &str,
) -> Result<(), Error> {
    let now = chrono::Utc::now().fixed_offset();
    security_user::Entity::insert(security_user::ActiveModel {
        id: ActiveValue::Set(id.to_owned()),
        first_name: ActiveValue::Set(first_name.to_owned()),
        last_name: ActiveValue::Set("User".to_owned()),
        email: ActiveValue::Set(email.to_owned()),
        password_hash: ActiveValue::Set(password_hash),
        status: ActiveValue::Set(UserStatus::Active.as_str().to_owned()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(db)
    .await?;
    security_user_role::Entity::insert(security_user_role::ActiveModel {
        user_id: ActiveValue::Set(id.to_owned()),
        source: ActiveValue::Set(DEFAULT_SOURCE.to_owned()),
        roles: ActiveValue::Set(json!([role])),
        updated_at: ActiveValue::Set(now),
    })
    .exec(db)
    .await?;
    Ok(())
}
