use std::collections::BTreeMap;
use std::io::BufReader;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use chrono::Utc;
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};

use crate::api::{Entry, Summary, Urls, time};
use crate::archive::{self, ArchiveError, Collection};
use crate::names::Coll;
use crate::{ARTIFACT, GZIP_TYPE, ID};

/// Why a collection was not stored: the request itself is refused (permissions, write policy,
/// a checksum that does not match), or its content is not a collection this server takes.
#[derive(Debug)]
pub enum Refusal {
    Request(ContentError),
    Content(String),
}

impl From<ContentError> for Refusal {
    fn from(error: ContentError) -> Self {
        Refusal::Request(error)
    }
}

impl Refusal {
    pub fn into_error(self) -> ContentError {
        match self {
            Refusal::Request(error) => error,
            Refusal::Content(message) => ContentError::BadRequest(message),
        }
    }
}

pub fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

/// Checks what can be checked before the archive is read: the caller may write the version,
/// and the write policy lets it.
pub async fn admit(ctx: &Ctx, coll: &Coll, version: &str) -> Result<(), ContentError> {
    let path = coll.artifact_path(version);
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
            "{coll} {version} already exists in {} and redeploy is not allowed",
            ctx.name()
        ))),
        _ => Ok(()),
    }
}

async fn read_archive(ctx: &Ctx, written: &Written) -> Result<Collection, Refusal> {
    let blob_store = ctx
        .content
        .blob_stores()
        .get(&written.store)
        .await
        .map_err(ContentError::from)?;
    let Some(content) = blob_store
        .read(written.id)
        .await
        .map_err(ContentError::from)?
    else {
        return Err(Refusal::Request(ContentError::internal(
            std::io::Error::other("a blob that was just written is missing"),
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
    read.map_err(|error| match error {
        ArchiveError::Invalid(message) => Refusal::Content(message),
        ArchiveError::Io(error) => Refusal::Content(format!(
            "the file is not a collection archive (a gzip-compressed tar): {error}"
        )),
    })
}

/// Reads the archive of a written blob, checks it against the file name it came with and keeps
/// it as the collection version it names. The blob is discarded when it is refused.
pub async fn store_collection(
    ctx: &Ctx,
    written: Written,
    named: Option<(Coll, String)>,
) -> Result<(Collection, Asset), Refusal> {
    let refuse = |refusal: Refusal| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        refusal
    };
    let collection = match read_archive(ctx, &written).await {
        Ok(collection) => collection,
        Err(refusal) => return Err(refuse(refusal)),
    };
    if let Some((coll, version)) = &named
        && (coll != &collection.coll || version != &collection.version)
    {
        return Err(refuse(Refusal::Content(format!(
            "the file {} does not match MANIFEST.json, which names {} {}",
            coll.file_name(version),
            collection.coll,
            collection.version
        ))));
    }
    if let Err(error) = admit(ctx, &collection.coll, &collection.version).await {
        return Err(refuse(Refusal::Request(error)));
    }
    let put = PutAsset::hosted(ctx, collection.coll.artifact_path(&collection.version))
        .kind(ARTIFACT)
        .content_type(GZIP_TYPE)
        .attributes(json!({ID: {"manifest": collection.manifest}}))
        .component(ComponentKey {
            namespace: collection.coll.namespace.clone(),
            name: collection.coll.name.clone(),
            version: collection.version.clone(),
            kind: String::new(),
            attributes: json!({ ID: collection.attributes() }),
        });
    let stored = store::adopt(ctx, put, &written).await?;
    Ok((collection, stored))
}

/// A version kept in a hosted repository: its component and its archive.
#[derive(Clone, Debug)]
pub struct Stored {
    pub component: component::Model,
    pub asset: Asset,
}

impl Stored {
    pub fn coll(&self) -> Coll {
        Coll {
            namespace: self.component.namespace.clone(),
            name: self.component.name.clone(),
        }
    }

    fn info(&self, key: &str) -> Value {
        self.component
            .attributes
            .get(ID)
            .and_then(|attributes| attributes.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    }

    pub fn entry(&self) -> Entry {
        Entry {
            version: self.component.version.clone(),
            created_at: time(self.component.created_at.with_timezone(&Utc)),
            updated_at: time(self.component.updated_at.with_timezone(&Utc)),
            requires_ansible: self.info("requires_ansible"),
            marks: json!([]),
        }
    }

    pub fn document(&self, urls: &Urls) -> Value {
        let coll = self.coll();
        let version = &self.component.version;
        let list = |key: &str| match self.info(key) {
            Value::Null => json!([]),
            other => other,
        };
        let text = |key: &str| self.info(key);
        let entry = self.entry();
        json!({
            "version": version,
            "href": urls.version(&coll, version),
            "created_at": entry.created_at,
            "updated_at": entry.updated_at,
            "requires_ansible": entry.requires_ansible,
            "marks": [],
            "artifact": {
                "filename": coll.file_name(version),
                "sha256": self.asset.sha256,
                "size": self.asset.size,
            },
            "collection": {"name": coll.name, "href": urls.collection(&coll)},
            "download_url": urls.download(&coll, version),
            "name": coll.name,
            "namespace": {"name": coll.namespace, "metadata_sha256": null},
            "signatures": [],
            "metadata": {
                "authors": list("authors"),
                "contents": list("contents"),
                "dependencies": match self.info("dependencies") {
                    Value::Null => json!({}),
                    other => other,
                },
                "description": text("description"),
                "documentation": text("documentation"),
                "homepage": text("homepage"),
                "issues": text("issues"),
                "license": list("license"),
                "repository": text("repository"),
                "tags": list("tags"),
            },
            "git_url": null,
            "git_commit_sha": null,
            "manifest": self
                .asset
                .attributes
                .get(ID)
                .and_then(|attributes| attributes.get("manifest"))
                .cloned()
                .unwrap_or(Value::Null),
        })
    }
}

/// The versions the caller may read, of one collection or of all.
pub async fn stored(ctx: &Ctx, only: Option<&Coll>) -> Result<Vec<Stored>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID));
    if let Some(coll) = only {
        query = query
            .filter(component::Column::Namespace.eq(&coll.namespace))
            .filter(component::Column::Name.eq(&coll.name));
    }
    let components = query
        .order_by_asc(component::Column::Namespace)
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
            .filter(asset::Column::Kind.eq(ARTIFACT))
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
            ctx.allowed(View::Read, &asset.path)
                .then_some(Stored { component, asset })
        })
        .collect())
}

pub fn summaries(versions: &[Stored]) -> Vec<Summary> {
    let mut by_collection: BTreeMap<Coll, (Vec<Entry>, i64)> = BTreeMap::new();
    for version in versions {
        let slot = by_collection.entry(version.coll()).or_default();
        slot.0.push(version.entry());
        slot.1 += version.asset.download_count;
    }
    by_collection
        .into_iter()
        .filter_map(|(coll, (entries, downloads))| Summary::from_entries(coll, &entries, downloads))
        .collect()
}
