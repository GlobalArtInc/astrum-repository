use std::collections::BTreeMap;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip;
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::builds::compare_versions;
use crate::plugin::{self, Found, Plugin, PluginError};
use crate::query::Query;
use crate::xml::Item;
use crate::{ARTIFACT, ID, ZIP_TYPE};

const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

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
    pub plugin: Plugin,
    pub channel: String,
    pub packaging: String,
}

impl Record {
    pub fn attributes(&self) -> Value {
        json!({ ID: self })
    }

    pub fn of(attributes: &Value) -> Option<Record> {
        serde_json::from_value(attributes.get(ID)?.clone()).ok()
    }
}

pub fn encoded(segment: &str) -> String {
    utf8_percent_encode(segment, SEGMENT).to_string()
}

pub fn download_url(base: &str, plugin: &Plugin, extension: &str) -> String {
    format!(
        "{base}/{}/{}/{}",
        encoded(&plugin.id),
        encoded(&plugin.version),
        encoded(&format!("{}-{}.{extension}", plugin.id, plugin.version))
    )
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

pub async fn admit(ctx: &Ctx, path: &str, id: &str, version: &str) -> Result<(), ContentError> {
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
            "{id} {version} is already in {} and redeploy is not allowed",
            ctx.name()
        ))),
        _ => Ok(()),
    }
}

pub async fn read_plugin(ctx: &Ctx, written: &Written) -> Result<Found, Refusal> {
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
    plugin::inspect(&source).await.map_err(|error| match error {
        PluginError::Invalid(message) => Refusal::Content(message),
        PluginError::Store(error) => Refusal::Request(ContentError::from(error)),
    })
}

pub fn component_key(found: &Found, channel: &str) -> ComponentKey {
    let record = Record {
        plugin: found.plugin.clone(),
        channel: channel.to_owned(),
        packaging: found.packaging.extension().to_owned(),
    };
    ComponentKey {
        namespace: String::new(),
        name: found.plugin.id.clone(),
        version: found.plugin.version.clone(),
        kind: String::new(),
        attributes: record.attributes(),
    }
}

pub struct Claim {
    pub xml_id: Option<String>,
    pub channel: String,
    pub path: Option<String>,
}

pub async fn store_plugin(
    ctx: &Ctx,
    written: Written,
    claim: &Claim,
) -> Result<(Found, Asset), Refusal> {
    let refuse = |refusal: Refusal| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        refusal
    };
    let found = match read_plugin(ctx, &written).await {
        Ok(found) => found,
        Err(refusal) => return Err(refuse(refusal)),
    };
    if let Err(message) = found.plugin.validate() {
        return Err(refuse(Refusal::Content(message)));
    }
    let path = found.plugin.path(found.packaging);
    if let Some(claimed) = claim.xml_id.as_deref()
        && claimed != found.plugin.id
        && !claimed.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(refuse(Refusal::Content(format!(
            "plugin.xml declares the id {}, not {claimed}",
            found.plugin.id
        ))));
    }
    if let Some(expected) = claim.path.as_deref()
        && expected != path
    {
        return Err(refuse(Refusal::Content(format!(
            "the archive holds {} {}, which belongs at {path}, not at {expected}",
            found.plugin.id, found.plugin.version
        ))));
    }
    if let Err(error) = admit(ctx, &path, &found.plugin.id, &found.plugin.version).await {
        return Err(refuse(Refusal::Request(error)));
    }
    let put = PutAsset::hosted(ctx, path)
        .kind(ARTIFACT)
        .content_type(ZIP_TYPE)
        .component(component_key(&found, &claim.channel));
    let stored = store::adopt(ctx, put, &written).await?;
    Ok((found, stored))
}

#[derive(Clone, Debug)]
pub struct Stored {
    pub component: component::Model,
    pub asset: Asset,
    pub record: Record,
}

impl Stored {
    pub fn plugin(&self) -> &Plugin {
        &self.record.plugin
    }

    pub fn packaging(&self) -> &str {
        match self.record.packaging.as_str() {
            "jar" => "jar",
            _ => "zip",
        }
    }

    pub fn item(&self, base: &str) -> Item<'_> {
        Item {
            plugin: &self.record.plugin,
            url: download_url(base, &self.record.plugin, self.packaging()),
            size: self.asset.size,
            downloads: self.asset.download_count,
            created_ms: self.component.created_at.timestamp_millis(),
            updated_ms: self.component.updated_at.timestamp_millis(),
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
            let record = Record::of(&component.attributes)?;
            ctx.allowed(View::Read, &asset.path).then_some(Stored {
                component,
                asset,
                record,
            })
        })
        .collect())
}

pub fn select(stored: Vec<Stored>, query: &Query) -> Vec<Stored> {
    let mut best: BTreeMap<String, Stored> = BTreeMap::new();
    for candidate in stored {
        let plugin = candidate.plugin();
        if candidate.record.channel != query.channel
            || !query.wants(&plugin.id)
            || !query.accepts(plugin.since_build.as_deref(), plugin.until_build.as_deref())
        {
            continue;
        }
        let better = match best.get(&plugin.id) {
            Some(known) => compare_versions(&plugin.version, &known.plugin().version).is_gt(),
            None => true,
        };
        if better {
            best.insert(plugin.id.clone(), candidate);
        }
    }
    best.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_encode_every_segment() {
        let plugin = Plugin {
            id: "PlantUML integration".to_owned(),
            version: "1.0+b1".to_owned(),
            ..Default::default()
        };
        assert_eq!(
            download_url("https://r.test/repository/jb", &plugin, "zip"),
            "https://r.test/repository/jb/PlantUML%20integration/1.0%2Bb1/PlantUML%20integration-1.0%2Bb1.zip"
        );
        assert_eq!(encoded("a.b-c_d~e"), "a.b-c_d~e");
    }

    #[test]
    fn records_round_trip_through_component_attributes() {
        let record = Record {
            plugin: Plugin {
                id: "a".to_owned(),
                version: "1".to_owned(),
                ..Default::default()
            },
            channel: "eap".to_owned(),
            packaging: "jar".to_owned(),
        };
        assert_eq!(Record::of(&record.attributes()), Some(record));
        assert_eq!(Record::of(&json!({})), None);
    }
}
