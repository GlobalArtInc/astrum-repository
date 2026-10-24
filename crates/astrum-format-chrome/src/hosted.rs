use std::collections::BTreeMap;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip;
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::crx::{self, CrxError, Package, artifact_path, compare_versions};
use crate::{ARTIFACT, CRX_TYPE, ID};

const CHUNK: usize = 500;

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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_version: u64,
    pub description: String,
    pub update_url: String,
}

impl Record {
    pub fn of_package(package: &Package) -> Self {
        Record {
            id: package.id.clone(),
            name: package.manifest.name.clone(),
            version: package.manifest.version.clone(),
            manifest_version: package.manifest.manifest_version,
            description: package.manifest.description.clone(),
            update_url: package.manifest.update_url.clone(),
        }
    }

    pub fn attributes(&self) -> Value {
        json!({ ID: self })
    }

    pub fn of(attributes: &Value) -> Option<Record> {
        serde_json::from_value(attributes.get(ID)?.clone()).ok()
    }
}

pub fn component_key(package: &Package) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: package.id.clone(),
        version: package.manifest.version.clone(),
        kind: String::new(),
        attributes: Record::of_package(package).attributes(),
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

pub async fn admit(ctx: &Ctx, path: &str, package: &Package) -> Result<(), ContentError> {
    let exists = store::find(ctx.db(), ctx.name(), path).await?.is_some();
    let policy = write_policy(ctx);
    if exists && policy == WritePolicy::Allow {
        ctx.require(View::Edit, path)?;
    } else {
        ctx.require(View::Add, path)?;
    }
    match policy {
        WritePolicy::Deny => Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        ))),
        WritePolicy::AllowOnce if exists => Err(ContentError::Conflict(format!(
            "{} {} is already in {} and redeploy is not allowed",
            package.id,
            package.manifest.version,
            ctx.name()
        ))),
        _ => Ok(()),
    }
}

pub async fn read_package(ctx: &Ctx, written: &Written) -> Result<Package, Refusal> {
    let blob_store = ctx
        .content
        .blob_stores()
        .get(&written.store)
        .await
        .map_err(ContentError::from)?;
    let source = zip::Blob {
        store: blob_store,
        id: written.id,
        size: written.size,
    };
    crx::inspect(&source).await.map_err(|error| match error {
        CrxError::Invalid(message) => Refusal::Content(message),
        CrxError::Store(error) => Refusal::Request(ContentError::from(error)),
    })
}

pub async fn store_package(
    ctx: &Ctx,
    written: Written,
    expected: Option<&str>,
) -> Result<(Package, Asset), Refusal> {
    let refuse = |refusal: Refusal| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        refusal
    };
    let package = match read_package(ctx, &written).await {
        Ok(package) => package,
        Err(refusal) => return Err(refuse(refusal)),
    };
    let path = artifact_path(&package.id, &package.manifest.version);
    if let Some(expected) = expected
        && expected != path
    {
        return Err(refuse(Refusal::Content(format!(
            "the package holds {} {}, which belongs at {path}, not at {expected}",
            package.id, package.manifest.version
        ))));
    }
    if let Err(error) = admit(ctx, &path, &package).await {
        return Err(refuse(Refusal::Request(error)));
    }
    let put = PutAsset::hosted(ctx, path)
        .kind(ARTIFACT)
        .content_type(CRX_TYPE)
        .component(component_key(&package));
    let stored = store::adopt(ctx, put, &written).await?;
    Ok((package, stored))
}

#[derive(Clone, Debug)]
pub struct Stored {
    pub asset: Asset,
    pub record: Record,
}

pub async fn newest(ctx: &Ctx, ids: &[String]) -> Result<BTreeMap<String, Stored>, ContentError> {
    let mut best: BTreeMap<String, Stored> = BTreeMap::new();
    for chunk in ids.chunks(CHUNK) {
        let components = component::Entity::find()
            .filter(component::Column::Repository.eq(ctx.name()))
            .filter(component::Column::Format.eq(ID))
            .filter(component::Column::Name.is_in(chunk.iter().cloned()))
            .all(ctx.db())
            .await?;
        if components.is_empty() {
            continue;
        }
        let mut records: BTreeMap<i64, Record> = components
            .into_iter()
            .filter_map(|found| Record::of(&found.attributes).map(|record| (found.id, record)))
            .collect();
        let assets = asset::Entity::find()
            .filter(asset::Column::Repository.eq(ctx.name()))
            .filter(asset::Column::Kind.eq(ARTIFACT))
            .filter(asset::Column::ComponentId.is_in(records.keys().copied().collect::<Vec<_>>()))
            .all(ctx.db())
            .await?;
        for found in assets {
            let Some(record) = found.component_id.and_then(|id| records.remove(&id)) else {
                continue;
            };
            if !ctx.allowed(View::Read, &found.path) {
                continue;
            }
            let better = match best.get(&record.id) {
                Some(known) => compare_versions(&record.version, &known.record.version).is_gt(),
                None => true,
            };
            if better {
                best.insert(
                    record.id.clone(),
                    Stored {
                        asset: found,
                        record,
                    },
                );
            }
        }
    }
    Ok(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_round_trip_through_component_attributes() {
        let record = Record {
            id: "a".repeat(32),
            name: "Acme".to_owned(),
            version: "1.0".to_owned(),
            manifest_version: 3,
            ..Default::default()
        };
        let attributes = record.attributes();
        assert_eq!(attributes["chrome"]["manifestVersion"], 3);
        assert_eq!(attributes["chrome"]["id"], "a".repeat(32));
        assert_eq!(Record::of(&attributes), Some(record));
        assert_eq!(Record::of(&json!({})), None);
    }
}
