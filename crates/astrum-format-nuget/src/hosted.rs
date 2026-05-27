use std::collections::HashMap;
use std::io;

use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, Component, ContentError, ContentRequest, Ctx, UploadForm, View};
use astrum_db::entity::{asset, component};
use astrum_repo::RepositoryType;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{DateTime, FixedOffset};
use futures_util::TryStreamExt;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};

use crate::archive::{self, ArchiveError};
use crate::metadata::{Metadata, parse_nuspec};
use crate::route::{package_path, symbols_path};
use crate::version::Version;
use crate::{ID, PACKAGE_KIND, SYMBOLS_KIND, ZIP, reply};

#[derive(Clone, Debug)]
pub struct Stored {
    pub component: Component,
    pub metadata: Metadata,
    pub package: Option<Asset>,
    pub symbols: Option<Asset>,
}

impl Stored {
    pub fn version(&self) -> Option<Version> {
        self.metadata.parsed_version()
    }

    pub fn downloads(&self) -> i64 {
        self.package
            .as_ref()
            .map(|asset| asset.download_count)
            .unwrap_or_default()
    }

    pub fn published(&self) -> DateTime<FixedOffset> {
        self.package
            .as_ref()
            .map(|asset| asset.created_at)
            .unwrap_or(self.component.created_at)
    }

    pub fn updated(&self) -> DateTime<FixedOffset> {
        self.package
            .as_ref()
            .map(|asset| asset.blob_updated_at)
            .unwrap_or(self.component.updated_at)
    }
}

pub fn metadata_of(component: &Component) -> Metadata {
    component
        .attributes
        .get(ID)
        .cloned()
        .and_then(|value| serde_json::from_value::<Metadata>(value).ok())
        .filter(|metadata| !metadata.id.is_empty() && !metadata.normalized_version.is_empty())
        .unwrap_or_else(|| {
            let parsed = Version::parse(&component.version);
            Metadata {
                id: component.name.clone(),
                version: parsed
                    .as_ref()
                    .map(Version::full)
                    .unwrap_or_else(|| component.version.clone()),
                normalized_version: parsed
                    .as_ref()
                    .map(Version::normalized)
                    .unwrap_or_else(|| component.version.clone()),
                original_version: component.version.clone(),
                is_prerelease: parsed.as_ref().is_some_and(Version::is_prerelease),
                semver2: parsed.as_ref().is_some_and(Version::is_semver2),
                package_types: vec![crate::metadata::DEFAULT_TYPE.to_owned()],
                ..Default::default()
            }
        })
}

pub async fn load(
    ctx: &Ctx,
    id: Option<&str>,
    terms: &[String],
) -> Result<Vec<Stored>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID));
    if let Some(id) = id {
        query = query.filter(
            Expr::expr(Func::lower(Expr::col(component::Column::Name))).eq(id.to_lowercase()),
        );
    }
    for term in terms {
        let escaped = term
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        query = query.filter(Expr::cust_with_values(
            "(lower(name) LIKE $1 OR lower(attributes::text) LIKE $1)",
            [format!("%{escaped}%")],
        ));
    }
    let components = query
        .order_by_asc(component::Column::Id)
        .all(ctx.db())
        .await?;
    if components.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<i64> = components.iter().map(|component| component.id).collect();
    let assets = asset::Entity::find()
        .filter(asset::Column::ComponentId.is_in(ids))
        .all(ctx.db())
        .await?;
    let mut by_component: HashMap<i64, Vec<Asset>> = HashMap::new();
    for found in assets {
        if let Some(component_id) = found.component_id {
            by_component.entry(component_id).or_default().push(found);
        }
    }
    Ok(components
        .into_iter()
        .map(|component| {
            let assets = by_component.remove(&component.id).unwrap_or_default();
            let package = assets
                .iter()
                .find(|asset| asset.path.ends_with(".nupkg"))
                .cloned();
            let symbols = assets
                .iter()
                .find(|asset| asset.path.ends_with(".snupkg"))
                .cloned();
            Stored {
                metadata: metadata_of(&component),
                component,
                package,
                symbols,
            }
        })
        .filter(|stored| stored.package.is_some())
        .collect())
}

pub async fn versions(ctx: &Ctx, id: &str) -> Result<Vec<Stored>, ContentError> {
    let mut found = load(ctx, Some(id), &[]).await?;
    found.sort_by(compare);
    Ok(found)
}

pub fn compare(left: &Stored, right: &Stored) -> std::cmp::Ordering {
    match (left.version(), right.version()) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => left
            .metadata
            .normalized_version
            .cmp(&right.metadata.normalized_version),
    }
}

pub async fn one(ctx: &Ctx, id: &str, version: &str) -> Result<Option<Stored>, ContentError> {
    let wanted = Version::parse(version);
    Ok(load(ctx, Some(id), &[]).await?.into_iter().find(|stored| {
        match (&wanted, stored.version()) {
            (Some(wanted), Some(found)) => *wanted == found || wanted.cmp(&found).is_eq(),
            _ => stored
                .metadata
                .normalized_version
                .eq_ignore_ascii_case(version),
        }
    }))
}

pub async fn nuspec_of(ctx: &Ctx, package: &Asset) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&package.blob_store).await?;
    let source = archive::Blob {
        store,
        id: package.blob_id.into(),
        size: package.size as u64,
    };
    archive::nuspec(&source)
        .await
        .map_err(|error| ContentError::internal(anyhow::anyhow!("{error}")))?
        .ok_or_else(|| ContentError::not_found("the package holds no .nuspec file"))
}

fn invalid(message: impl std::fmt::Display) -> ContentError {
    ContentError::BadRequest(format!("Invalid NuGet package: {message}"))
}

async fn write(ctx: &Ctx, request: ContentRequest) -> Result<Written, ContentError> {
    let boundary = request
        .header("content-type")
        .filter(|value| value.to_ascii_lowercase().starts_with("multipart/"))
        .and_then(|value| multer::parse_boundary(value).ok());
    let Some(boundary) = boundary else {
        return store::write_blob(ctx, request.into_stream()).await;
    };
    let mut multipart = multer::Multipart::new(request.take_body().into_data_stream(), boundary);
    if let Some(part) = multipart.next_field().await.map_err(|error| {
        ContentError::BadRequest(format!("the upload form cannot be read: {error}"))
    })? {
        let stream: ContentStream = Box::pin(part.map_err(io::Error::other));
        return store::write_blob(ctx, stream).await;
    }
    Err(ContentError::BadRequest(
        "the upload form holds no package".to_owned(),
    ))
}

async fn read_metadata(ctx: &Ctx, written: &Written) -> Result<Metadata, ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let source = archive::Blob {
        store,
        id: written.id,
        size: written.size,
    };
    let bytes = match archive::nuspec(&source).await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Err(invalid("The package does not contain a .nuspec file")),
        Err(ArchiveError::Store(error)) => return Err(error.into()),
        Err(error) => return Err(invalid(error)),
    };
    parse_nuspec(&bytes).map_err(invalid)
}

fn may_write(ctx: &Ctx) -> Result<(), ContentError> {
    if ctx.grant(View::Add).any() || ctx.grant(View::Edit).any() {
        return Ok(());
    }
    ctx.require(View::Add, "/")
}

pub async fn push(ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
    if ctx.kind() != RepositoryType::Hosted {
        return Err(ContentError::MethodNotAllowed {
            method: request.method.to_string(),
            allow: "GET,HEAD",
        });
    }
    may_write(ctx)?;
    let written = write(ctx, request).await?;
    match store_written(ctx, &written).await {
        Ok(()) => Ok(reply::status(StatusCode::CREATED)),
        Err(error) => {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            Err(error)
        }
    }
}

async fn existing_name(ctx: &Ctx, id: &str) -> Result<Option<String>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(Expr::expr(Func::lower(Expr::col(component::Column::Name))).eq(id.to_lowercase()))
        .order_by_asc(component::Column::Id)
        .one(ctx.db())
        .await?
        .map(|found| found.name))
}

async fn store_written(ctx: &Ctx, written: &Written) -> Result<(), ContentError> {
    let mut metadata = read_metadata(ctx, written).await?;
    if metadata.is_symbols() {
        return store_symbols(ctx, written, &metadata).await;
    }
    if let Some(name) = existing_name(ctx, &metadata.id).await? {
        metadata.id = name;
    }
    let path = package_path(&metadata.id, &metadata.normalized_version);
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
    let key = ComponentKey {
        namespace: String::new(),
        name: metadata.id.clone(),
        version: metadata.normalized_version.clone(),
        kind: PACKAGE_KIND.to_owned(),
        attributes: json!({ ID: metadata }),
    };
    let put = PutAsset::hosted(ctx, path)
        .component(key)
        .kind(PACKAGE_KIND)
        .content_type(ZIP)
        .attributes(json!({ ID: {} }));
    store::adopt(ctx, put, written).await?;
    Ok(())
}

async fn store_symbols(
    ctx: &Ctx,
    written: &Written,
    metadata: &Metadata,
) -> Result<(), ContentError> {
    let Some(stored) = one(ctx, &metadata.id, &metadata.normalized_version).await? else {
        return Err(ContentError::BadRequest(format!(
            "the package {} {} is not in {}; push it before its symbols",
            metadata.id,
            metadata.normalized_version,
            ctx.name()
        )));
    };
    let path = symbols_path(&stored.metadata.id, &stored.metadata.normalized_version);
    let exists = stored.symbols.is_some();
    ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
    let key = ComponentKey {
        namespace: stored.component.namespace.clone(),
        name: stored.component.name.clone(),
        version: stored.component.version.clone(),
        kind: stored.component.kind.clone(),
        attributes: stored.component.attributes.clone(),
    };
    let put = PutAsset::hosted(ctx, path)
        .component(key)
        .kind(SYMBOLS_KIND)
        .content_type(ZIP)
        .attributes(json!({ ID: {} }));
    store::adopt(ctx, put, written).await?;
    Ok(())
}

pub async fn delete(ctx: &Ctx, id: &str, version: &str) -> Result<Response, ContentError> {
    if ctx.kind() == RepositoryType::Group {
        return Err(ContentError::MethodNotAllowed {
            method: "DELETE".to_owned(),
            allow: "GET,HEAD",
        });
    }
    let path = package_path(id, version);
    let Some(stored) = one(ctx, id, version).await? else {
        ctx.require(View::Delete, &path)?;
        return Err(ContentError::not_found(format!(
            "{id} {version} is not in {}",
            ctx.name()
        )));
    };
    for asset in stored.package.iter().chain(stored.symbols.iter()) {
        ctx.require(View::Delete, &asset.path)?;
    }
    store::delete_component(&ctx.content, stored.component.id).await?;
    Ok(reply::status(StatusCode::NO_CONTENT))
}

pub async fn upload(ctx: &Ctx, form: UploadForm) -> Result<(), ContentError> {
    if ctx.kind() != RepositoryType::Hosted {
        return Err(ContentError::BadRequest(
            "only hosted repositories take uploads".to_owned(),
        ));
    }
    let file = form
        .files
        .iter()
        .find(|(name, _)| name.starts_with("nuget.asset"))
        .map(|(_, file)| file)
        .ok_or_else(|| ContentError::BadRequest("nuget.asset is required".to_owned()))?;
    may_write(ctx)?;
    let written = store::write_blob(ctx, UploadForm::stream(file).await?).await?;
    match store_written(ctx, &written).await {
        Ok(()) => Ok(()),
        Err(error) => {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            Err(error)
        }
    }
}

pub fn attributes(metadata: &Metadata) -> Value {
    json!({ ID: metadata })
}
