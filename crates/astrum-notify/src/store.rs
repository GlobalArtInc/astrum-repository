use astrum_security::SecretBox;
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, QueryOrder};
use uuid::Uuid;

use crate::channel::{Change, Changes, Channel, Proxy, Sealed};
use crate::entity;

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Channel>, sea_orm::DbErr> {
    Ok(entity::Entity::find()
        .order_by_asc(entity::Column::CreatedAt)
        .order_by_asc(entity::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .map(Channel::from_row)
        .collect())
}

pub async fn find(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<Channel>, sea_orm::DbErr> {
    Ok(entity::Entity::find_by_id(id)
        .one(db)
        .await?
        .map(Channel::from_row))
}

fn apply(change: &Change, before: Option<String>, secrets: &SecretBox) -> Option<String> {
    match change {
        Change::Keep => before,
        Change::Set(value) => Some(secrets.seal(value)),
        Change::Clear => None,
    }
}

pub fn sealed(before: &Sealed, changes: &Changes, secrets: &SecretBox) -> Sealed {
    Sealed {
        secret: apply(&changes.secret, before.secret.clone(), secrets),
        proxy_password: apply(
            &changes.proxy_password,
            before.proxy_password.clone(),
            secrets,
        ),
        signing_secret: apply(
            &changes.signing_secret,
            before.signing_secret.clone(),
            secrets,
        ),
        authorization: apply(
            &changes.authorization,
            before.authorization.clone(),
            secrets,
        ),
    }
}

fn row(channel: &Channel) -> entity::ActiveModel {
    let now = chrono::Utc::now().fixed_offset();
    let (proxy_type, proxy_host, proxy_port, proxy_username) = match &channel.proxy {
        Proxy::Custom {
            kind,
            host,
            port,
            username,
        } => (
            Some(kind.id().to_owned()),
            Some(host.clone()),
            Some(i32::from(*port)),
            username.clone(),
        ),
        _ => (None, None, None, None),
    };
    entity::ActiveModel {
        id: ActiveValue::Set(channel.id),
        name: ActiveValue::Set(channel.name.clone()),
        kind: ActiveValue::Set(channel.kind.id().to_owned()),
        enabled: ActiveValue::Set(channel.enabled),
        chat_id: ActiveValue::Set(channel.chat_id.clone()),
        message_thread_id: ActiveValue::Set(channel.message_thread_id),
        recipients: ActiveValue::Set(channel.recipients.clone()),
        secret: ActiveValue::Set(channel.sealed.secret.clone()),
        target_host: ActiveValue::Set(channel.target_host.clone()),
        repositories: ActiveValue::Set(channel.repositories.clone()),
        events: ActiveValue::Set(
            channel
                .events
                .iter()
                .map(|event| event.id().to_owned())
                .collect(),
        ),
        language: ActiveValue::Set(channel.language.clone()),
        proxy_mode: ActiveValue::Set(channel.proxy.mode().to_owned()),
        proxy_type: ActiveValue::Set(proxy_type),
        proxy_host: ActiveValue::Set(proxy_host),
        proxy_port: ActiveValue::Set(proxy_port),
        proxy_username: ActiveValue::Set(proxy_username),
        proxy_password: ActiveValue::Set(channel.sealed.proxy_password.clone()),
        signing_secret: ActiveValue::Set(channel.sealed.signing_secret.clone()),
        authorization_header: ActiveValue::Set(channel.sealed.authorization.clone()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    }
}

pub async fn insert(db: &impl ConnectionTrait, channel: &Channel) -> Result<(), sea_orm::DbErr> {
    entity::Entity::insert(row(channel)).exec(db).await?;
    Ok(())
}

pub async fn update(db: &impl ConnectionTrait, channel: &Channel) -> Result<(), sea_orm::DbErr> {
    let mut row = row(channel);
    row.created_at = ActiveValue::NotSet;
    entity::Entity::update(row).exec(db).await?;
    Ok(())
}

pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<bool, sea_orm::DbErr> {
    Ok(entity::Entity::delete_by_id(id)
        .exec(db)
        .await?
        .rows_affected
        > 0)
}
