use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, Component, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{Value, json};

use crate::index::{self, Metadata, Publish};
use crate::{CRATE, CRATE_TYPE, ID, crate_path};

pub fn lowered_name() -> sea_orm::sea_query::SimpleExpr {
    Func::lower(Expr::col((component::Entity, component::Column::Name))).into()
}

pub async fn versions(ctx: &Ctx, name: &str) -> Result<Vec<Component>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(Expr::expr(lowered_name()).eq(name.to_ascii_lowercase()))
        .order_by_asc(component::Column::CreatedAt)
        .order_by_asc(component::Column::Id)
        .all(ctx.db())
        .await?)
}

pub async fn version(
    ctx: &Ctx,
    name: &str,
    version: &str,
) -> Result<Option<Component>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(Expr::expr(lowered_name()).eq(name.to_ascii_lowercase()))
        .filter(component::Column::Version.eq(version))
        .one(ctx.db())
        .await?)
}

pub fn index_entry(found: &Component) -> Option<Value> {
    let mut entry = found.attributes.get(ID)?.get("index")?.clone();
    let object = entry.as_object_mut()?;
    object.insert(
        "pubtime".to_owned(),
        json!(
            found
                .created_at
                .with_timezone(&chrono::Utc)
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ),
    );
    Some(entry)
}

pub async fn entries(ctx: &Ctx, name: &str) -> Result<Vec<Value>, ContentError> {
    let mut entries = Vec::new();
    for found in versions(ctx, name).await? {
        if !ctx.allowed(View::Read, &crate_path(&found.name, &found.version)) {
            continue;
        }
        let has_crate = asset::Entity::find()
            .filter(asset::Column::ComponentId.eq(found.id))
            .filter(asset::Column::Kind.eq(CRATE))
            .select_only()
            .column(asset::Column::Id)
            .into_tuple::<i64>()
            .one(ctx.db())
            .await?
            .is_some();
        if has_crate && let Some(entry) = index_entry(&found) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

pub async fn crate_asset(
    ctx: &Ctx,
    name: &str,
    version: &str,
) -> Result<Option<Asset>, ContentError> {
    let Some(found) = self::version(ctx, name, version).await? else {
        return Ok(None);
    };
    Ok(store::assets_of(ctx.db(), found.id)
        .await?
        .into_iter()
        .find(|candidate| candidate.kind == CRATE))
}

fn attributes(metadata: &Metadata, entry: Value) -> Value {
    json!({
        ID: {
            "index": entry,
            "description": metadata.description,
            "keywords": metadata.keywords,
            "categories": metadata.categories,
            "authors": metadata.authors,
            "license": metadata.license,
            "homepage": metadata.homepage,
            "repository": metadata.repository,
            "documentation": metadata.documentation,
        }
    })
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

pub async fn publish(ctx: &Ctx, publish: Publish) -> Result<Asset, ContentError> {
    let metadata = &publish.metadata;
    let path = crate_path(&metadata.name, &metadata.vers);
    let policy = write_policy(ctx);
    if policy == WritePolicy::Deny {
        return Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        )));
    }
    let existing = versions(ctx, &metadata.name).await?;
    if let Some(other) = existing.iter().find(|found| found.name != metadata.name) {
        return Err(ContentError::BadRequest(format!(
            "crate `{}` conflicts with existing crate `{}`",
            metadata.name, other.name
        )));
    }
    let republish = existing.iter().any(|found| found.version == metadata.vers);
    if republish && policy != WritePolicy::Allow {
        return Err(ContentError::Conflict(format!(
            "crate version already exists and repository {} does not allow redeployment",
            ctx.name()
        )));
    }
    ctx.require(if republish { View::Edit } else { View::Add }, &path)?;
    let written = store::write_blob(ctx, astrum_blob::content_of(publish.archive.clone())).await?;
    let entry = index::entry(metadata, &written.digests.sha256);
    let put = PutAsset::hosted(ctx, path)
        .component(ComponentKey {
            namespace: String::new(),
            name: metadata.name.clone(),
            version: metadata.vers.clone(),
            kind: String::new(),
            attributes: attributes(metadata, entry),
        })
        .kind(CRATE)
        .content_type(CRATE_TYPE)
        .attributes(json!({}));
    store::adopt(ctx, put, &written).await
}

pub async fn set_yanked(
    ctx: &Ctx,
    name: &str,
    version: &str,
    yanked: bool,
) -> Result<(), ContentError> {
    let found = self::version(ctx, name, version).await?.ok_or_else(|| {
        ContentError::not_found(format!(
            "crate `{name}` does not have a version `{version}`"
        ))
    })?;
    ctx.require(View::Edit, &crate_path(&found.name, &found.version))?;
    let mut attributes = found.attributes.clone();
    if let Some(entry) = attributes
        .get_mut(ID)
        .and_then(|cargo| cargo.get_mut("index"))
        .and_then(Value::as_object_mut)
    {
        entry.insert("yanked".to_owned(), json!(yanked));
    }
    store::set_component_attributes(ctx.db(), found.id, attributes).await
}

pub struct Found {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

pub async fn search(ctx: &Ctx, query: &str, limit: u64) -> Result<(Vec<Found>, u64), ContentError> {
    let escaped = query
        .to_ascii_lowercase()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let rows = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(Expr::expr(lowered_name()).like(format!("%{escaped}%")))
        .order_by_asc(component::Column::Name)
        .order_by_desc(component::Column::CreatedAt)
        .all(ctx.db())
        .await?;
    let mut found: Vec<Found> = Vec::new();
    for row in rows {
        if !ctx.allowed(View::Read, &crate_path(&row.name, &row.version)) {
            continue;
        }
        let description = row
            .attributes
            .get(ID)
            .and_then(|cargo| cargo.get("description"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        match found.iter_mut().find(|known| known.name == row.name) {
            Some(known) => {
                if newer(&row.version, &known.version) {
                    known.version = row.version.clone();
                    known.description = description;
                }
            }
            None => found.push(Found {
                name: row.name.clone(),
                version: row.version.clone(),
                description,
            }),
        }
    }
    let total = found.len() as u64;
    found.truncate(limit as usize);
    Ok((found, total))
}

fn newer(candidate: &str, known: &str) -> bool {
    match (
        semver::Version::parse(candidate),
        semver::Version::parse(known),
    ) {
        (Ok(candidate), Ok(known)) => candidate > known,
        _ => candidate > known,
    }
}
