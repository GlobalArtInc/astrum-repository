use std::collections::BTreeMap;
use std::io::BufReader;

use astrum_blob::{BlobId, Written};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Map, Value, json};

use crate::archive::{self, ArchiveError, Metadata};
use crate::docs::{Cookbook, Item, Universe};
use crate::names;
use crate::{COOKBOOK, GZIP_TYPE, ID};

pub const DEFAULT_CATEGORY: &str = "Other";

pub fn time(at: DateTime<FixedOffset>) -> String {
    at.with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[derive(Clone, Debug)]
pub struct Stored {
    pub component: component::Model,
    pub asset: Asset,
}

impl Stored {
    pub fn name(&self) -> &str {
        &self.component.name
    }

    pub fn version(&self) -> &str {
        &self.component.version
    }

    pub fn info(&self, key: &str) -> Value {
        self.component
            .attributes
            .get(ID)
            .and_then(|attributes| attributes.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    }

    fn object(&self, key: &str) -> Value {
        match self.info(key) {
            Value::Object(map) => Value::Object(map),
            _ => json!({}),
        }
    }

    pub fn dependencies(&self) -> Value {
        self.object("dependencies")
    }

    pub fn version_document(&self) -> Map<String, Value> {
        let mut document = Map::new();
        document.insert("license".to_owned(), self.info("license"));
        document.insert("tarball_file_size".to_owned(), json!(self.asset.size));
        document.insert("version".to_owned(), json!(self.version()));
        document.insert(
            "published_at".to_owned(),
            json!(
                self.asset
                    .created_at
                    .with_timezone(&Utc)
                    .to_rfc3339_opts(SecondsFormat::Secs, true)
            ),
        );
        document.insert("average_rating".to_owned(), Value::Null);
        document.insert("dependencies".to_owned(), self.dependencies());
        document.insert("platforms".to_owned(), self.object("platforms"));
        document
    }
}

pub async fn stored(ctx: &Ctx, only: Option<&str>) -> Result<Vec<Stored>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID));
    if let Some(name) = only {
        query = query.filter(component::Column::Name.eq(name));
    }
    let components = query
        .order_by_asc(component::Column::Name)
        .all(ctx.db())
        .await?;
    if components.is_empty() {
        return Ok(Vec::new());
    }
    let mut assets: BTreeMap<i64, Asset> = BTreeMap::new();
    for chunk in components.chunks(1000) {
        let ids: Vec<i64> = chunk.iter().map(|component| component.id).collect();
        for found in asset::Entity::find()
            .filter(asset::Column::Repository.eq(ctx.name()))
            .filter(asset::Column::Kind.eq(COOKBOOK))
            .filter(asset::Column::ComponentId.is_in(ids))
            .all(ctx.db())
            .await?
        {
            if let Some(component_id) = found.component_id {
                assets.insert(component_id, found);
            }
        }
    }
    Ok(components
        .into_iter()
        .filter_map(|component| {
            let asset = assets.remove(&component.id)?;
            (asset.path == names::tarball_path(&component.name, &component.version))
                .then_some(Stored { component, asset })
        })
        .collect())
}

pub fn universe(stored: &[Stored]) -> Universe {
    let mut universe = Universe::new();
    for one in stored {
        universe
            .entry(one.name().to_owned())
            .or_default()
            .insert(one.version().to_owned(), one.dependencies());
    }
    universe
}

fn by_name(stored: Vec<Stored>) -> BTreeMap<String, Vec<Stored>> {
    let mut grouped: BTreeMap<String, Vec<Stored>> = BTreeMap::new();
    for one in stored {
        grouped.entry(one.name().to_owned()).or_default().push(one);
    }
    grouped
}

fn newest(versions: &[Stored]) -> Option<&Stored> {
    versions
        .iter()
        .max_by(|a, b| names::compare(a.version(), b.version()))
}

pub fn cookbook(name: &str, versions: &[Stored]) -> Option<Cookbook> {
    let newest = newest(versions)?;
    let created = versions
        .iter()
        .map(|one| one.component.created_at)
        .min()
        .unwrap_or(newest.component.created_at);
    let updated = versions
        .iter()
        .map(|one| one.component.updated_at.max(one.asset.blob_updated_at))
        .max()
        .unwrap_or(newest.component.updated_at);
    let mut downloads = Map::new();
    let mut total = 0;
    for one in versions {
        downloads.insert(one.version().to_owned(), json!(one.asset.download_count));
        total += one.asset.download_count;
    }
    let category = match newest.info("category") {
        Value::String(category) if !category.is_empty() => category,
        _ => DEFAULT_CATEGORY.to_owned(),
    };
    let document = json!({
        "name": name,
        "maintainer": newest.info("maintainer"),
        "description": newest.info("description"),
        "category": category,
        "external_url": newest.info("source_url"),
        "source_url": newest.info("source_url"),
        "issues_url": newest.info("issues_url"),
        "average_rating": null,
        "created_at": time(created),
        "updated_at": time(updated),
        "up_for_adoption": null,
        "deprecated": false,
        "metrics": {
            "downloads": {"total": total, "versions": downloads},
            "followers": 0,
            "collaborators": 0,
        },
    });
    Some(Cookbook {
        name: name.to_owned(),
        document: document.as_object().cloned().unwrap_or_default(),
        versions: versions
            .iter()
            .map(|one| one.version().to_owned())
            .collect(),
    })
}

pub fn items(stored: Vec<Stored>) -> Vec<Item> {
    by_name(stored)
        .into_iter()
        .filter_map(|(name, versions)| {
            let newest = newest(&versions)?;
            Some(Item {
                maintainer: newest.info("maintainer"),
                description: newest.info("description"),
                created_at: versions
                    .iter()
                    .map(|one| time(one.component.created_at))
                    .min()
                    .unwrap_or_default(),
                updated_at: versions
                    .iter()
                    .map(|one| time(one.component.updated_at))
                    .max()
                    .unwrap_or_default(),
                downloads: versions.iter().map(|one| one.asset.download_count).sum(),
                versions: versions
                    .iter()
                    .map(|one| one.version().to_owned())
                    .collect(),
                name,
            })
        })
        .collect()
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

pub fn may_add(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Add).any() || ctx.grant(View::Edit).any() {
        true => Ok(()),
        false => ctx.require(View::Add, "/cookbooks/"),
    }
}

async fn admit(ctx: &Ctx, name: &str, version: &str) -> Result<(), ContentError> {
    let path = names::tarball_path(name, version);
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    let policy = write_policy(ctx);
    if exists && policy == WritePolicy::Allow {
        ctx.require(View::Edit, &path)?;
    } else {
        ctx.require(View::Add, &path)?;
    }
    match policy {
        WritePolicy::Deny => Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        ))),
        WritePolicy::AllowOnce if exists => Err(ContentError::Conflict(format!(
            "version {version} of cookbook {name} already exists in {} and redeploy is not allowed",
            ctx.name()
        ))),
        _ => Ok(()),
    }
}

pub async fn read_metadata(
    ctx: &Ctx,
    store_name: &str,
    blob: BlobId,
) -> Result<Result<Metadata, String>, ContentError> {
    let blob_store = ctx.content.blob_stores().get(store_name).await?;
    let Some(content) = blob_store.read(blob).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob of the repository is missing",
        )));
    };
    let file = tempfile::tempfile().map_err(ContentError::internal)?;
    let mut target = tokio::fs::File::from_std(file);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        tokio::io::AsyncWriteExt::write_all(&mut target, &chunk)
            .await
            .map_err(ContentError::internal)?;
    }
    let mut file = target.into_std().await;
    let read = tokio::task::spawn_blocking(move || {
        std::io::Seek::rewind(&mut file)?;
        archive::read(BufReader::new(file))
    })
    .await
    .map_err(ContentError::internal)?;
    Ok(read.map_err(|error| match error {
        ArchiveError::Invalid(message) => message,
        ArchiveError::Io(error) => {
            format!("the file is not a cookbook archive (a gzip-compressed tar): {error}")
        }
    }))
}

async fn inherited_category(ctx: &Ctx, name: &str) -> Result<Option<String>, ContentError> {
    let known = stored(ctx, Some(name)).await?;
    Ok(newest(&known).and_then(|one| one.info("category").as_str().map(str::to_owned)))
}

pub async fn adopt(
    ctx: &Ctx,
    written: Written,
    category: Option<String>,
) -> Result<Metadata, ContentError> {
    let refuse = |error: ContentError| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        error
    };
    let metadata = match read_metadata(ctx, &written.store, written.id).await {
        Ok(Ok(metadata)) => metadata,
        Ok(Err(message)) => return Err(refuse(ContentError::BadRequest(message))),
        Err(error) => return Err(refuse(error)),
    };
    if let Err(error) = admit(ctx, &metadata.name, &metadata.version).await {
        return Err(refuse(error));
    }
    let category = match category.filter(|category| !category.trim().is_empty()) {
        Some(category) => category.trim().to_owned(),
        None => inherited_category(ctx, &metadata.name)
            .await?
            .unwrap_or_else(|| DEFAULT_CATEGORY.to_owned()),
    };
    let put = PutAsset::hosted(ctx, names::tarball_path(&metadata.name, &metadata.version))
        .kind(COOKBOOK)
        .content_type(GZIP_TYPE)
        .component(ComponentKey {
            namespace: String::new(),
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            kind: String::new(),
            attributes: json!({ ID: metadata.attributes(&category) }),
        });
    store::adopt(ctx, put, &written).await?;
    Ok(metadata)
}
