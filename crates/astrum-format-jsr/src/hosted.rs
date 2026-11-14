use std::collections::HashMap;

use astrum_content::store::{self, ComponentKey};
use astrum_content::{ContentError, Ctx};
use astrum_db::entity::{asset, component};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde_json::{Value, json};

use crate::ID;
use crate::documents::Versions;
use crate::names::{self, Package};

pub const FILE_KIND: &str = "file";
pub const META_KIND: &str = "meta";

pub fn component_key(package: &Package, version: &str) -> ComponentKey {
    ComponentKey {
        namespace: package.namespace(),
        name: package.name.clone(),
        version: version.to_owned(),
        kind: String::new(),
        attributes: Value::Object(Default::default()),
    }
}

async fn components(ctx: &Ctx, package: &Package) -> Result<Vec<component::Model>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Namespace.eq(package.namespace()))
        .filter(component::Column::Name.eq(package.name.as_str()))
        .all(ctx.db())
        .await?)
}

pub async fn component_of(
    ctx: &Ctx,
    package: &Package,
    version: &str,
) -> Result<Option<component::Model>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Namespace.eq(package.namespace()))
        .filter(component::Column::Name.eq(package.name.as_str()))
        .filter(component::Column::Version.eq(version))
        .one(ctx.db())
        .await?)
}

pub async fn versions(ctx: &Ctx, package: &Package) -> Result<Vec<String>, ContentError> {
    let found = components(ctx, package).await?;
    if found.is_empty() {
        return Ok(Vec::new());
    }
    let by_id: HashMap<i64, &component::Model> = found.iter().map(|one| (one.id, one)).collect();
    let documents = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(META_KIND))
        .filter(asset::Column::ComponentId.is_in(by_id.keys().copied().collect::<Vec<_>>()))
        .all(ctx.db())
        .await?;
    let mut listed: Vec<String> = documents
        .iter()
        .filter_map(|stored| {
            let owner = by_id.get(&stored.component_id?)?;
            (stored.path == package.version_meta_path(&owner.version))
                .then(|| owner.version.clone())
        })
        .collect();
    names::sorted(&mut listed);
    Ok(listed)
}

pub async fn known(ctx: &Ctx, package: &Package) -> Result<bool, ContentError> {
    Ok(asset::Entity::find()
        .select_only()
        .column(asset::Column::Id)
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(META_KIND))
        .filter(Expr::col(asset::Column::Path).like(format!("{}/%", package.directory())))
        .limit(1)
        .into_tuple::<i64>()
        .one(ctx.db())
        .await?
        .is_some())
}

pub async fn listing(ctx: &Ctx, package: &Package) -> Result<Option<Versions>, ContentError> {
    let listed = versions(ctx, package).await?;
    if listed.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        listed
            .into_iter()
            .map(|version| (version, json!({})))
            .collect(),
    ))
}

pub async fn discard_version(
    ctx: &Ctx,
    package: &Package,
    version: &str,
) -> Result<bool, ContentError> {
    match component_of(ctx, package, version).await? {
        Some(found) => store::delete_component(&ctx.content, found.id).await,
        None => Ok(false),
    }
}
