use astrum_content::store::{Asset, ComponentKey};
use astrum_content::{ContentError, Ctx};
use astrum_db::entity::{asset, component};
use chrono::Utc;
use futures_util::TryStreamExt;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde_json::Value;

use crate::{ID, NARINFO};

pub async fn referrers(ctx: &Ctx, nar_path: &str) -> Result<Vec<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Format.eq(ID))
        .filter(asset::Column::Kind.eq(NARINFO))
        .filter(Expr::cust_with_values(
            "attributes #>> '{nix,nar}' = $1",
            [nar_path.to_owned()],
        ))
        .order_by_asc(asset::Column::Id)
        .all(ctx.db())
        .await?)
}

pub async fn upsert_component(ctx: &Ctx, key: &ComponentKey) -> Result<i64, ContentError> {
    let now = Utc::now().fixed_offset();
    let row = component::ActiveModel {
        id: ActiveValue::NotSet,
        repository: ActiveValue::Set(ctx.name().to_owned()),
        format: ActiveValue::Set(ID.to_owned()),
        namespace: ActiveValue::Set(key.namespace.clone()),
        name: ActiveValue::Set(key.name.clone()),
        version: ActiveValue::Set(key.version.clone()),
        kind: ActiveValue::Set(key.kind.clone()),
        attributes: ActiveValue::Set(key.attributes.clone()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    };
    let stored = component::Entity::insert(row)
        .on_conflict(
            OnConflict::columns([
                component::Column::Repository,
                component::Column::Namespace,
                component::Column::Name,
                component::Column::Version,
            ])
            .update_columns([
                component::Column::Kind,
                component::Column::Attributes,
                component::Column::UpdatedAt,
            ])
            .to_owned(),
        )
        .exec_with_returning(ctx.db())
        .await?;
    Ok(stored.id)
}

pub async fn place(
    ctx: &Ctx,
    found: &Asset,
    component_id: Option<i64>,
    attributes: Option<Value>,
) -> Result<(), ContentError> {
    if found.component_id == component_id && attributes.is_none() {
        return Ok(());
    }
    let mut row = asset::ActiveModel {
        id: ActiveValue::Unchanged(found.id),
        component_id: ActiveValue::Set(component_id),
        ..Default::default()
    };
    if let Some(attributes) = attributes {
        row.attributes = ActiveValue::Set(attributes);
    }
    asset::Entity::update(row).exec(ctx.db()).await?;
    if let Some(previous) = found.component_id
        && Some(previous) != component_id
    {
        drop_if_empty(ctx, previous).await?;
    }
    Ok(())
}

pub async fn drop_if_empty(ctx: &Ctx, component_id: i64) -> Result<(), ContentError> {
    let left = asset::Entity::find()
        .filter(asset::Column::ComponentId.eq(component_id))
        .count(ctx.db())
        .await?;
    if left == 0 {
        component::Entity::delete_by_id(component_id)
            .exec(ctx.db())
            .await?;
    }
    Ok(())
}

pub async fn read(ctx: &Ctx, found: &Asset, limit: usize) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let content = store
        .read(found.blob_id.into())
        .await?
        .ok_or_else(|| ContentError::not_found(found.path.clone()))?;
    let mut stream = content.stream;
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if bytes.len() + chunk.len() > limit {
            return Err(ContentError::BadGateway(format!(
                "{} is larger than {limit} bytes",
                found.path
            )));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
