use std::collections::{BTreeSet, HashMap};

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use chrono::SecondsFormat;
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{Value, json};

use crate::archive::{self, Pubspec};
use crate::listing::{Entry, Listing, archive_path};
use crate::{ARCHIVE, ARCHIVE_TYPE, ID, MAX_ARCHIVE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Published {
    pub name: String,
    pub version: String,
    pub unchanged: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub discontinued: bool,
    pub replaced_by: Option<String>,
}

fn attributes_of(found: &component::Model) -> &Value {
    found.attributes.get(ID).unwrap_or(&Value::Null)
}

fn flags_of(found: &component::Model) -> Flags {
    let attributes = attributes_of(found);
    Flags {
        discontinued: attributes
            .get("discontinued")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        replaced_by: attributes
            .get("replacedBy")
            .and_then(Value::as_str)
            .map(str::to_owned),
    }
}

pub fn component_attributes(pubspec: &Value, retracted: bool, flags: &Flags) -> Value {
    let mut attributes = json!({
        "pubspec": pubspec,
        "retracted": retracted,
        "discontinued": flags.discontinued,
    });
    if let Some(description) = pubspec.get("description").and_then(Value::as_str) {
        attributes["description"] = json!(description);
    }
    if let Some(replaced_by) = &flags.replaced_by {
        attributes["replacedBy"] = json!(replaced_by);
    }
    json!({ ID: attributes })
}

pub fn asset_attributes(name: &str, version: &str, pubspec: &Value) -> Value {
    let mut attributes = json!({"name": name, "version": version});
    if let Some(description) = pubspec.get("description").and_then(Value::as_str) {
        attributes["description"] = json!(description);
    }
    if let Some(sdk) = pubspec
        .get("environment")
        .and_then(|environment| environment.get("sdk"))
        .and_then(Value::as_str)
    {
        attributes["sdk"] = json!(sdk);
    }
    for link in ["homepage", "repository"] {
        if let Some(url) = pubspec.get(link).and_then(Value::as_str) {
            attributes[link] = json!(url);
        }
    }
    json!({ ID: attributes })
}

pub fn key(name: &str, version: &str, attributes: Value) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: name.to_owned(),
        version: version.to_owned(),
        kind: String::new(),
        attributes,
    }
}

async fn components(ctx: &Ctx, name: &str) -> Result<Vec<component::Model>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Name.eq(name))
        .all(ctx.db())
        .await?)
}

pub async fn listing(ctx: &Ctx, name: &str) -> Result<Option<Listing>, ContentError> {
    let found = components(ctx, name).await?;
    if found.is_empty() {
        return Ok(None);
    }
    let by_id: HashMap<i64, &component::Model> = found.iter().map(|one| (one.id, one)).collect();
    let archives = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(ARCHIVE))
        .filter(asset::Column::ComponentId.is_in(by_id.keys().copied().collect::<Vec<_>>()))
        .all(ctx.db())
        .await?;
    let mut listing = Listing {
        name: name.to_owned(),
        ..Listing::default()
    };
    let mut newest: Option<(chrono::DateTime<chrono::FixedOffset>, Flags)> = None;
    for archive in archives {
        let Some(owner) = archive.component_id.and_then(|id| by_id.get(&id)) else {
            continue;
        };
        if archive.path != archive_path(name, &owner.version) {
            continue;
        }
        let attributes = attributes_of(owner);
        let flags = flags_of(owner);
        if newest
            .as_ref()
            .is_none_or(|(when, _)| owner.updated_at > *when)
        {
            newest = Some((owner.updated_at, flags));
        }
        listing.versions.push(Entry {
            version: owner.version.clone(),
            retracted: attributes
                .get("retracted")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            sha256: Some(archive.sha256.clone()),
            published: Some(
                archive
                    .created_at
                    .to_utc()
                    .to_rfc3339_opts(SecondsFormat::Millis, true),
            ),
            pubspec: attributes
                .get("pubspec")
                .cloned()
                .unwrap_or_else(|| json!({"name": name, "version": owner.version})),
            archive_url: None,
        });
    }
    if listing.versions.is_empty() {
        return Ok(None);
    }
    if let Some((_, flags)) = newest {
        listing.discontinued = flags.discontinued;
        listing.replaced_by = flags.replaced_by;
    }
    listing.sort();
    Ok(Some(listing))
}

pub async fn names(ctx: &Ctx) -> Result<Vec<String>, ContentError> {
    let names: Vec<String> = component::Entity::find()
        .select_only()
        .column(component::Column::Name)
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .distinct()
        .order_by_asc(component::Column::Name)
        .into_tuple()
        .all(ctx.db())
        .await?;
    Ok(names)
}

pub async fn read_blob(
    ctx: &Ctx,
    store_name: &str,
    blob: uuid::Uuid,
    limit: u64,
) -> Result<Vec<u8>, ContentError> {
    let blob_store = ctx.content.blob_stores().get(store_name).await?;
    let Some(content) = blob_store.read(blob.into()).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob of the repository is missing",
        )));
    };
    if content.size > limit {
        return Err(ContentError::BadRequest(format!(
            "the package archive is larger than {limit} bytes"
        )));
    }
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
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
        false => ctx.require(View::Add, "/packages/"),
    }
}

fn refuse(ctx: &Ctx, written: &Written, error: ContentError) -> ContentError {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    error
}

async fn pubspec_of(ctx: &Ctx, written: &Written) -> Result<Pubspec, ContentError> {
    if written.size > MAX_ARCHIVE {
        return Err(ContentError::BadRequest(format!(
            "the package archive is larger than {MAX_ARCHIVE} bytes"
        )));
    }
    let data = read_blob(ctx, &written.store, written.id.uuid(), MAX_ARCHIVE).await?;
    tokio::task::spawn_blocking(move || archive::read(&data))
        .await
        .map_err(ContentError::internal)?
        .map_err(|error| ContentError::BadRequest(error.to_string()))
}

pub async fn package_flags(ctx: &Ctx, name: &str) -> Result<Flags, ContentError> {
    Ok(components(ctx, name)
        .await?
        .iter()
        .max_by_key(|found| found.updated_at)
        .map(flags_of)
        .unwrap_or_default())
}

pub async fn adopt(ctx: &Ctx, written: Written) -> Result<Published, ContentError> {
    let pubspec = match pubspec_of(ctx, &written).await {
        Ok(pubspec) => pubspec,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    if let Err(message) = archive::valid_new_name(&pubspec.name) {
        return Err(refuse(ctx, &written, ContentError::BadRequest(message)));
    }
    let path = archive_path(&pubspec.name, &pubspec.version);
    let policy = write_policy(ctx);
    if policy == WritePolicy::Deny {
        let message = format!("the repository {} is read-only", ctx.name());
        return Err(refuse(ctx, &written, ContentError::BadRequest(message)));
    }
    let existing = store::find(ctx.db(), ctx.name(), &path).await?;
    if let Some(existing) = &existing
        && existing.sha256 == written.digests.sha256
    {
        if let Err(error) = ctx.require(View::Read, &path) {
            return Err(refuse(ctx, &written, error));
        }
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Ok(Published {
            name: pubspec.name,
            version: pubspec.version,
            unchanged: true,
        });
    }
    if existing.is_some() && policy != WritePolicy::Allow {
        let message = format!(
            "version {} of package {} already exists in {}, which does not allow redeployment",
            pubspec.version,
            pubspec.name,
            ctx.name()
        );
        return Err(refuse(ctx, &written, ContentError::BadRequest(message)));
    }
    let view = match existing {
        Some(_) => View::Edit,
        None => View::Add,
    };
    if let Err(error) = ctx.require(view, &path) {
        return Err(refuse(ctx, &written, error));
    }
    let flags = package_flags(ctx, &pubspec.name).await?;
    let put = PutAsset::hosted(ctx, path)
        .kind(ARCHIVE)
        .content_type(ARCHIVE_TYPE)
        .attributes(asset_attributes(
            &pubspec.name,
            &pubspec.version,
            &pubspec.json,
        ))
        .component(key(
            &pubspec.name,
            &pubspec.version,
            component_attributes(&pubspec.json, false, &flags),
        ));
    store::adopt(ctx, put, &written).await?;
    Ok(Published {
        name: pubspec.name,
        version: pubspec.version,
        unchanged: false,
    })
}

pub async fn set_retracted(
    ctx: &Ctx,
    name: &str,
    version: &str,
    retracted: bool,
) -> Result<bool, ContentError> {
    ctx.require(View::Edit, &archive_path(name, version))?;
    let Some(found) = components(ctx, name)
        .await?
        .into_iter()
        .find(|found| found.version == version)
    else {
        return Err(ContentError::not_found(format!(
            "version {version} of package {name} was not found"
        )));
    };
    let mut attributes = found.attributes.clone();
    if !attributes.get(ID).is_some_and(Value::is_object) {
        attributes[ID] = json!({});
    }
    attributes[ID]["retracted"] = json!(retracted);
    store::set_component_attributes(ctx.db(), found.id, attributes).await?;
    Ok(retracted)
}

pub async fn set_flags(ctx: &Ctx, name: &str, flags: &Flags) -> Result<(), ContentError> {
    let found = components(ctx, name).await?;
    if found.is_empty() {
        return Err(ContentError::not_found(format!(
            "package {name} was not found"
        )));
    }
    for one in &found {
        ctx.require(View::Edit, &archive_path(name, &one.version))?;
    }
    for one in found {
        let mut attributes = one.attributes.clone();
        if !attributes.get(ID).is_some_and(Value::is_object) {
            attributes[ID] = json!({});
        }
        attributes[ID]["discontinued"] = json!(flags.discontinued);
        match &flags.replaced_by {
            Some(replaced_by) if flags.discontinued => {
                attributes[ID]["replacedBy"] = json!(replaced_by);
            }
            _ => {
                if let Some(object) = attributes[ID].as_object_mut() {
                    object.remove("replacedBy");
                }
            }
        }
        store::set_component_attributes(ctx.db(), one.id, attributes).await?;
    }
    Ok(())
}

pub fn visible(ctx: &Ctx, mut listing: Listing) -> Option<Listing> {
    if !matches!(ctx.grant(View::Read), astrum_content::Grant::Whole) {
        let name = listing.name.clone();
        listing
            .versions
            .retain(|entry| ctx.allowed(View::Read, &archive_path(&name, &entry.version)));
    }
    (!listing.versions.is_empty()).then_some(listing)
}

pub fn unique_names(lists: Vec<Vec<String>>) -> Vec<String> {
    lists
        .into_iter()
        .flatten()
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect()
}
