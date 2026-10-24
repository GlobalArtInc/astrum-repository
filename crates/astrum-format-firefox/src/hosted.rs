use std::collections::BTreeMap;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip;
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};

use crate::addon::{self, Addon, AddonError};
use crate::updates::{Update, Updates, link, sha256_hash};
use crate::version::compare;
use crate::{ADDON, ID, XPI_TYPE};

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

pub fn attributes(addon: &Addon) -> Value {
    json!({ ID: addon })
}

pub fn record_of(attributes: &Value) -> Option<Addon> {
    serde_json::from_value(attributes.get(ID)?.clone()).ok()
}

pub fn component_key(addon: &Addon) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: addon.id.clone(),
        version: addon.version.clone(),
        kind: String::new(),
        attributes: attributes(addon),
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

pub async fn read_addon(ctx: &Ctx, written: &Written) -> Result<Addon, Refusal> {
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
    addon::inspect(&source).await.map_err(|error| match error {
        AddonError::Invalid(message) => Refusal::Content(message),
        AddonError::Store(error) => Refusal::Request(ContentError::from(error)),
    })
}

fn admit(
    ctx: &Ctx,
    path: &str,
    addon: &Addon,
    existing: Option<&Asset>,
    same: bool,
) -> Result<(), ContentError> {
    let policy = write_policy(ctx);
    match existing.is_some() && !same && policy == WritePolicy::Allow {
        true => ctx.require(View::Edit, path)?,
        false => ctx.require(View::Add, path)?,
    }
    match policy {
        WritePolicy::Deny => Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        ))),
        WritePolicy::AllowOnce if existing.is_some() && !same => {
            Err(ContentError::Conflict(format!(
                "{} {} is already in {} with other content and redeploy is not allowed",
                addon.id,
                addon.version,
                ctx.name()
            )))
        }
        _ => Ok(()),
    }
}

pub struct Outcome {
    pub addon: Addon,
    pub asset: Asset,
    pub created: bool,
}

pub async fn store_addon(
    ctx: &Ctx,
    written: Written,
    expected: Option<&str>,
) -> Result<Outcome, Refusal> {
    let refuse = |refusal: Refusal| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        refusal
    };
    let addon = match read_addon(ctx, &written).await {
        Ok(addon) => addon,
        Err(refusal) => return Err(refuse(refusal)),
    };
    if let Err(message) = addon.validate() {
        return Err(refuse(Refusal::Content(message)));
    }
    let path = addon.path();
    if let Some(expected) = expected
        && expected != path
    {
        return Err(refuse(Refusal::Content(format!(
            "the file holds {} {}, which belongs at {path}, not at {expected}",
            addon.id, addon.version
        ))));
    }
    let existing = match store::find(ctx.db(), ctx.name(), &path).await {
        Ok(existing) => existing,
        Err(error) => return Err(refuse(Refusal::Request(error))),
    };
    let same = existing
        .as_ref()
        .is_some_and(|found| found.sha256.eq_ignore_ascii_case(&written.digests.sha256));
    if let Err(error) = admit(ctx, &path, &addon, existing.as_ref(), same) {
        return Err(refuse(Refusal::Request(error)));
    }
    if let (true, Some(asset)) = (same, existing) {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Ok(Outcome {
            addon,
            asset,
            created: false,
        });
    }
    let put = PutAsset::hosted(ctx, path)
        .kind(ADDON)
        .content_type(XPI_TYPE)
        .component(component_key(&addon));
    let asset = store::adopt(ctx, put, &written).await?;
    Ok(Outcome {
        addon,
        asset,
        created: true,
    })
}

#[derive(Clone, Debug)]
pub struct Stored {
    pub asset: Asset,
    pub addon: Addon,
}

impl Stored {
    pub fn update(&self, base: &str) -> Update {
        Update {
            version: self.addon.version.clone(),
            link: link(base, &self.addon.id, &self.addon.version),
            hash: sha256_hash(&self.asset.sha256),
            min: self.addon.min_version.clone(),
            max: self.addon.max_version.clone(),
        }
    }
}

pub async fn stored(ctx: &Ctx, id: Option<&str>) -> Result<Vec<Stored>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID));
    if let Some(id) = id {
        query = query.filter(component::Column::Name.eq(id));
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
            .filter(asset::Column::Kind.eq(ADDON))
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
            let mut addon = record_of(&component.attributes)?;
            addon.id = component.name;
            addon.version = component.version;
            ctx.allowed(View::Read, &asset.path)
                .then_some(Stored { asset, addon })
        })
        .collect())
}

pub fn newest(stored: Vec<Stored>) -> Option<Stored> {
    stored
        .into_iter()
        .max_by(|left, right| compare(&left.addon.version, &right.addon.version))
}

pub fn updates(stored: &[Stored], base: &str) -> Updates {
    let mut updates = Updates::new();
    for found in stored {
        updates
            .entry(found.addon.id.clone())
            .or_default()
            .push(found.update(base));
    }
    updates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_round_trip_through_component_attributes() {
        let addon = Addon {
            id: "a@b.org".to_owned(),
            name: "A".to_owned(),
            version: "1.0".to_owned(),
            signed: true,
            min_version: Some("115.0".to_owned()),
            max_version: None,
        };
        let attributes = attributes(&addon);
        assert_eq!(attributes["firefox"]["signed"], true);
        assert_eq!(attributes["firefox"]["minVersion"], "115.0");
        assert!(attributes["firefox"].get("maxVersion").is_none());
        assert_eq!(record_of(&attributes), Some(addon));
        assert_eq!(record_of(&json!({})), None);
    }
}
