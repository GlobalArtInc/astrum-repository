use std::collections::HashMap;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip;
use astrum_content::{ContentError, Ctx, Grant, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use chrono::Utc;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;

use crate::model::{Release, Stored};
use crate::names::{self, UNIVERSAL};
use crate::vsix::{self, Package, VsixError};
use crate::{ID, MAX_VSIX, VSIX, VSIX_TYPE};

pub enum Scope<'a> {
    All,
    Namespace(&'a str),
    Extension(&'a str, &'a str),
}

fn lower(column: component::Column) -> sea_orm::sea_query::SimpleExpr {
    Func::lower(Expr::col(column)).into()
}

pub async fn releases(ctx: &Ctx, scope: Scope<'_>) -> Result<Vec<Release>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID));
    match scope {
        Scope::All => {}
        Scope::Namespace(namespace) => {
            query = query.filter(
                Expr::expr(lower(component::Column::Namespace)).eq(namespace.to_lowercase()),
            );
        }
        Scope::Extension(namespace, name) => {
            query = query
                .filter(
                    Expr::expr(lower(component::Column::Namespace)).eq(namespace.to_lowercase()),
                )
                .filter(Expr::expr(lower(component::Column::Name)).eq(name.to_lowercase()));
        }
    }
    let components: HashMap<i64, component::Model> = query
        .all(ctx.db())
        .await?
        .into_iter()
        .map(|found| (found.id, found))
        .collect();
    if components.is_empty() {
        return Ok(Vec::new());
    }
    let whole = matches!(ctx.grant(View::Read), Grant::Whole);
    let ids: Vec<i64> = components.keys().copied().collect();
    let mut releases = Vec::new();
    for chunk in ids.chunks(1000) {
        for found in asset::Entity::find()
            .filter(asset::Column::Repository.eq(ctx.name()))
            .filter(asset::Column::Kind.eq(VSIX))
            .filter(asset::Column::ComponentId.is_in(chunk.to_vec()))
            .all(ctx.db())
            .await?
        {
            let Some(owner) = found.component_id.and_then(|id| components.get(&id)) else {
                continue;
            };
            if !whole && !ctx.allowed(View::Read, &found.path) {
                continue;
            }
            releases.push(release_of(owner, found));
        }
    }
    Ok(releases)
}

fn release_of(owner: &component::Model, found: asset::Model) -> Release {
    let stored: Stored = found
        .attributes
        .get(ID)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let platform = match names::is_platform(&stored.target_platform) {
        true => stored.target_platform,
        false => UNIVERSAL.to_owned(),
    };
    Release {
        namespace: owner.namespace.clone(),
        name: owner.name.clone(),
        version: owner.version.clone(),
        platform,
        meta: stored.meta,
        files: stored.files,
        published: found.created_at.with_timezone(&Utc),
        asset: found,
    }
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

pub fn may_publish(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Add).any() || ctx.grant(View::Edit).any() {
        true => Ok(()),
        false => ctx.require(View::Add, "/api/"),
    }
}

pub fn zip_error(error: zip::ZipError) -> ContentError {
    match error {
        zip::ZipError::Store(error) => error.into(),
        other => ContentError::internal(other),
    }
}

async fn package_of(ctx: &Ctx, written: &Written) -> Result<Package, ContentError> {
    if written.size > MAX_VSIX {
        return Err(ContentError::BadRequest(format!(
            "the extension package is larger than {MAX_VSIX} bytes"
        )));
    }
    let source = zip::Blob {
        store: ctx.content.blob_stores().get(&written.store).await?,
        id: written.id,
        size: written.size,
    };
    vsix::read(&source).await.map_err(|error| match error {
        VsixError::Store(error) => error.into(),
        other => ContentError::BadRequest(other.to_string()),
    })
}

async fn spelling(
    ctx: &Ctx,
    namespace: &str,
    name: &str,
) -> Result<(String, String), ContentError> {
    let found = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(Expr::expr(lower(component::Column::Namespace)).eq(namespace.to_lowercase()))
        .filter(Expr::expr(lower(component::Column::Name)).eq(name.to_lowercase()))
        .one(ctx.db())
        .await?;
    Ok(match found {
        Some(found) => (found.namespace, found.name),
        None => (namespace.to_owned(), name.to_owned()),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adopted {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub unchanged: bool,
}

pub async fn adopt(ctx: &Ctx, written: Written) -> Result<Adopted, ContentError> {
    let refuse = |error: ContentError| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        error
    };
    let package = match package_of(ctx, &written).await {
        Ok(package) => package,
        Err(error) => return Err(refuse(error)),
    };
    let (namespace, name) = match spelling(ctx, &package.namespace, &package.name).await {
        Ok(found) => found,
        Err(error) => return Err(refuse(error)),
    };
    let adopted = Adopted {
        namespace: namespace.clone(),
        name: name.clone(),
        version: package.version.clone(),
        platform: package.platform.clone(),
        unchanged: false,
    };
    let path = names::vsix_path(&namespace, &name, &package.platform, &package.version);
    let policy = write_policy(ctx);
    if policy == WritePolicy::Deny {
        return Err(refuse(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        ))));
    }
    let existing = store::find(ctx.db(), ctx.name(), &path).await?;
    if let Some(existing) = &existing
        && existing.sha256 == written.digests.sha256
    {
        if let Err(error) = ctx.require(View::Read, &path) {
            return Err(refuse(error));
        }
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Ok(Adopted {
            unchanged: true,
            ..adopted
        });
    }
    if existing.is_some() && policy != WritePolicy::Allow {
        return Err(refuse(ContentError::Conflict(format!(
            "{} {} ({}) is already published in {}, which does not allow redeployment",
            names::id(&namespace, &name),
            package.version,
            package.platform,
            ctx.name()
        ))));
    }
    let view = match existing {
        Some(_) => View::Edit,
        None => View::Add,
    };
    if let Err(error) = ctx.require(view, &path) {
        return Err(refuse(error));
    }
    let stored = Stored {
        target_platform: package.platform.clone(),
        files: package.files.clone(),
        meta: package.meta.clone(),
    };
    let put = PutAsset::hosted(ctx, path)
        .kind(VSIX)
        .content_type(VSIX_TYPE)
        .attributes(json!({ ID: stored }))
        .component(ComponentKey {
            namespace,
            name,
            version: package.version.clone(),
            kind: String::new(),
            attributes: json!({ ID: package.meta }),
        });
    store::adopt(ctx, put, &written).await?;
    Ok(adopted)
}

pub async fn entry(
    ctx: &Ctx,
    release: &Release,
    path: &str,
    limit: u64,
) -> Result<Option<Vec<u8>>, ContentError> {
    let source = zip::Blob {
        store: ctx
            .content
            .blob_stores()
            .get(&release.asset.blob_store)
            .await?,
        id: release.asset.blob_id.into(),
        size: release.asset.size.max(0) as u64,
    };
    let entries = zip::entries(&source).await.map_err(zip_error)?;
    let Some(found) = entries
        .iter()
        .find(|entry| entry.name == path && !entry.is_directory())
    else {
        return Ok(None);
    };
    zip::read(&source, found, limit)
        .await
        .map(Some)
        .map_err(|error| match error {
            zip::ZipError::Invalid("an entry of the archive is too large") => {
                ContentError::BadRequest(format!("{path} is larger than {limit} bytes"))
            }
            other => zip_error(other),
        })
}

pub async fn listing(
    ctx: &Ctx,
    release: &Release,
    directory: &str,
) -> Result<Vec<String>, ContentError> {
    let source = zip::Blob {
        store: ctx
            .content
            .blob_stores()
            .get(&release.asset.blob_store)
            .await?,
        id: release.asset.blob_id.into(),
        size: release.asset.size.max(0) as u64,
    };
    let mut children: Vec<String> = Vec::new();
    for entry in zip::entries(&source).await.map_err(zip_error)? {
        let Some(rest) = entry.name.strip_prefix(directory) else {
            continue;
        };
        let child = match rest.split_once('/') {
            Some((folder, _)) => format!("{directory}{folder}/"),
            None if rest.is_empty() => continue,
            None => entry.name.clone(),
        };
        if !children.contains(&child) {
            children.push(child);
        }
    }
    Ok(children)
}

pub async fn delete(
    ctx: &Ctx,
    namespace: &str,
    name: &str,
    wanted: Option<Vec<(String, Option<String>)>>,
) -> Result<usize, ContentError> {
    let chosen: Vec<Release> = releases(ctx, Scope::Extension(namespace, name))
        .await?
        .into_iter()
        .filter(|release| match &wanted {
            None => true,
            Some(wanted) => wanted.iter().any(|(version, platform)| {
                *version == release.version
                    && platform
                        .as_deref()
                        .is_none_or(|platform| platform == release.platform)
            }),
        })
        .collect();
    for release in &chosen {
        ctx.require(View::Delete, &release.asset.path)?;
    }
    for release in &chosen {
        store::delete_asset(&ctx.content, release.asset.clone()).await?;
    }
    Ok(chosen.len())
}
