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

use crate::api::{self, time};
use crate::archive::{self, ArchiveError, Release};
use crate::names::Module;
use crate::{ARTIFACT, GZIP_TYPE, ID};

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

pub fn may_write(ctx: &Ctx) -> bool {
    ctx.grant(View::Add).any() || ctx.grant(View::Edit).any()
}

pub async fn admit(ctx: &Ctx, module: &Module, version: &str) -> Result<(), ContentError> {
    let path = module.artifact_path(version);
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
            "A release with the version specified in the metadata.json already exists for this module: {module} {version} is in {} and redeploy is not allowed",
            ctx.name()
        ))),
        _ => Ok(()),
    }
}

async fn read_archive(ctx: &Ctx, written: &Written) -> Result<Release, Refusal> {
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
            "the file is not a module archive (a gzip-compressed tar): {error}"
        )),
    })
}

pub async fn store_release(ctx: &Ctx, written: Written) -> Result<(Release, Asset), Refusal> {
    let refuse = |refusal: Refusal| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        refusal
    };
    let release = match read_archive(ctx, &written).await {
        Ok(release) => release,
        Err(refusal) => return Err(refuse(refusal)),
    };
    if let Err(error) = admit(ctx, &release.module, &release.version).await {
        return Err(refuse(Refusal::Request(error)));
    }
    let put = PutAsset::hosted(ctx, release.module.artifact_path(&release.version))
        .kind(ARTIFACT)
        .content_type(GZIP_TYPE)
        .component(ComponentKey {
            namespace: release.module.owner.clone(),
            name: release.module.name.clone(),
            version: release.version.clone(),
            kind: String::new(),
            attributes: json!({ ID: release.attributes() }),
        });
    let stored = store::adopt(ctx, put, &written).await?;
    Ok((release, stored))
}

#[derive(Clone, Debug)]
pub struct Stored {
    pub component: component::Model,
    pub asset: Asset,
}

impl Stored {
    pub fn module(&self) -> Module {
        Module {
            owner: self.component.namespace.clone(),
            name: self.component.name.clone(),
        }
    }

    fn metadata(&self) -> Value {
        self.component
            .attributes
            .pointer(&format!("/{ID}/metadata"))
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    pub fn document(&self) -> Value {
        let module = self.module();
        let version = &self.component.version;
        let metadata = self.metadata();
        let tags: Vec<Value> = metadata
            .get("tags")
            .and_then(Value::as_array)
            .map(|tags| tags.iter().filter(|tag| tag.is_string()).cloned().collect())
            .unwrap_or_default();
        json!({
            "uri": module.release_uri(version),
            "slug": module.release_slug(version),
            "module": api::module_summary(&module),
            "version": version,
            "metadata": metadata,
            "tags": tags,
            "supported": false,
            "pdk": metadata.get("pdk-version").is_some(),
            "validation_score": null,
            "file_uri": module.file_uri(version),
            "file_size": self.asset.size,
            "file_md5": self.asset.md5,
            "file_sha256": self.asset.sha256,
            "downloads": self.asset.download_count,
            "readme": null,
            "changelog": null,
            "license": null,
            "reference": null,
            "tasks": [],
            "plans": [],
            "created_at": time(self.component.created_at.with_timezone(&Utc)),
            "updated_at": time(self.component.updated_at.with_timezone(&Utc)),
            "deleted_at": null,
            "deleted_for": null,
        })
    }
}

pub async fn stored(
    ctx: &Ctx,
    module: Option<&Module>,
    owner: Option<&str>,
) -> Result<Vec<Stored>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID));
    if let Some(module) = module {
        query = query
            .filter(component::Column::Namespace.eq(&module.owner))
            .filter(component::Column::Name.eq(&module.name));
    }
    if let Some(owner) = owner {
        query = query.filter(component::Column::Namespace.eq(owner.to_ascii_lowercase()));
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

pub fn modules(stored: &[Stored]) -> Vec<Value> {
    let mut by_module: BTreeMap<Module, Vec<Value>> = BTreeMap::new();
    for release in stored {
        by_module
            .entry(release.module())
            .or_default()
            .push(release.document());
    }
    by_module
        .into_iter()
        .filter_map(|(module, releases)| api::module_document(&module, &releases))
        .collect()
}
