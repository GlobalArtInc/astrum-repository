use std::collections::HashMap;

use astrum_blob::{BlobId, ContentStream};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{ContentError, Ctx, View, mime};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use futures_util::TryStreamExt;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};

use crate::catalog::{Catalog, Provider};
use crate::names::{ARCHITECTURES, BoxFile, BoxName, known_architecture};
use crate::{BOX, ID};

pub const ARCHIVE_TYPES: &[&str] = &[
    "application/gzip",
    "application/x-tar",
    "application/zip",
    "application/x-bzip2",
    "application/x-xz",
    "application/zstd",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Published {
    pub file: BoxFile,
    pub sha256: String,
    pub size: i64,
}

fn attributes_of(found: &component::Model) -> &Value {
    found.attributes.get(ID).unwrap_or(&Value::Null)
}

pub fn asset_attributes(file: &BoxFile) -> Value {
    json!({ID: {"provider": file.provider, "architecture": file.architecture}})
}

pub fn key(file: &BoxFile, attributes: Value) -> ComponentKey {
    ComponentKey {
        namespace: file.boxed.org.clone(),
        name: file.boxed.name.clone(),
        version: file.version.clone(),
        kind: String::new(),
        attributes,
    }
}

async fn components(
    ctx: &Ctx,
    boxed: &BoxName,
    version: Option<&str>,
) -> Result<Vec<component::Model>, ContentError> {
    let mut query = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Namespace.eq(&boxed.org))
        .filter(component::Column::Name.eq(&boxed.name));
    if let Some(version) = version {
        query = query.filter(component::Column::Version.eq(version));
    }
    Ok(query.all(ctx.db()).await?)
}

pub async fn catalog(ctx: &Ctx, boxed: &BoxName) -> Result<Option<Catalog>, ContentError> {
    let found = components(ctx, boxed, None).await?;
    if found.is_empty() {
        return Ok(None);
    }
    let by_id: HashMap<i64, &component::Model> = found.iter().map(|one| (one.id, one)).collect();
    let files = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(BOX))
        .filter(asset::Column::ComponentId.is_in(by_id.keys().copied().collect::<Vec<_>>()))
        .all(ctx.db())
        .await?;
    let mut catalog = Catalog::new(boxed.clone());
    for stored in files {
        let Some(owner) = stored.component_id.and_then(|id| by_id.get(&id)) else {
            continue;
        };
        let Some(file) = BoxFile::parse(&stored.path) else {
            continue;
        };
        if file.boxed != *boxed || file.version != owner.version {
            continue;
        }
        let description = attributes_of(owner)
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_owned);
        catalog.add(
            &file.version,
            description,
            Provider {
                name: file.provider,
                architecture: file.architecture,
                default_architecture: None,
                checksum_type: Some("sha256".to_owned()),
                checksum: Some(stored.sha256),
                url: None,
            },
        );
    }
    catalog.finish();
    Ok((!catalog.is_empty()).then_some(catalog))
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

fn strict(ctx: &Ctx) -> bool {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.strict_content_type_validation)
        .unwrap_or(true)
}

pub fn may_add(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Add).any() || ctx.grant(View::Edit).any() {
        true => Ok(()),
        false => ctx.require(View::Add, "/"),
    }
}

pub fn check_architecture(architecture: &str) -> Result<(), ContentError> {
    match known_architecture(architecture) {
        true => Ok(()),
        false => Err(ContentError::BadRequest(format!(
            "the architecture {architecture} is not one Vagrant knows; use one of {}",
            ARCHITECTURES.join(", ")
        ))),
    }
}

pub async fn sniff(ctx: &Ctx, store: &str, blob: BlobId) -> Result<Option<String>, ContentError> {
    let blob_store = ctx.content.blob_stores().get(store).await?;
    let Some(content) = blob_store.read_range(blob, 0, mime::SAMPLE as u64).await? else {
        return Ok(None);
    };
    let mut sample = Vec::with_capacity(content.length as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        sample.extend_from_slice(&chunk);
    }
    Ok(mime::detect(&sample).filter(|kind| ARCHIVE_TYPES.contains(&kind.as_str())))
}

pub async fn set_content_type(
    ctx: &Ctx,
    asset_id: i64,
    content_type: &str,
) -> Result<(), ContentError> {
    let row = asset::ActiveModel {
        id: ActiveValue::Unchanged(asset_id),
        content_type: ActiveValue::Set(content_type.to_owned()),
        ..Default::default()
    };
    asset::Entity::update(row).exec(ctx.db()).await?;
    Ok(())
}

async fn component_attributes(
    ctx: &Ctx,
    file: &BoxFile,
    description: Option<&str>,
) -> Result<Value, ContentError> {
    let existing = components(ctx, &file.boxed, Some(&file.version))
        .await?
        .into_iter()
        .next();
    let mut attributes = existing
        .map(|found| found.attributes)
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    if !attributes.get(ID).is_some_and(Value::is_object) {
        attributes[ID] = json!({});
    }
    if let Some(description) = description.map(str::trim).filter(|text| !text.is_empty()) {
        attributes[ID]["description"] = json!(description);
    }
    Ok(attributes)
}

pub async fn publish(
    ctx: &Ctx,
    file: &BoxFile,
    description: Option<&str>,
    body: ContentStream,
) -> Result<Published, ContentError> {
    let path = file.path();
    let existing = store::find(ctx.db(), ctx.name(), &path).await?;
    ctx.require(
        match existing {
            Some(_) => View::Edit,
            None => View::Add,
        },
        &path,
    )?;
    check_architecture(&file.architecture)?;
    match write_policy(ctx) {
        WritePolicy::Deny => {
            return Err(ContentError::BadRequest(format!(
                "the repository {} is read-only",
                ctx.name()
            )));
        }
        WritePolicy::AllowOnce if existing.is_some() => {
            return Err(ContentError::Conflict(format!(
                "version {} of {} already has a {} box for {} in {}, which does not allow redeployment",
                file.version,
                file.boxed.tag(),
                file.provider,
                file.architecture,
                ctx.name()
            )));
        }
        _ => {}
    }
    let written = store::write_blob(ctx, body).await?;
    let detected = match sniff(ctx, &written.store, written.id).await {
        Ok(detected) => detected,
        Err(error) => {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(error);
        }
    };
    if detected.is_none() && strict(ctx) {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Err(ContentError::BadRequest(
            "the box is not a tar, tar.gz or zip archive; build it with vagrant package or Packer"
                .to_owned(),
        ));
    }
    let attributes = match component_attributes(ctx, file, description).await {
        Ok(attributes) => attributes,
        Err(error) => {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(error);
        }
    };
    let put = PutAsset::hosted(ctx, path)
        .kind(BOX)
        .content_type(detected.unwrap_or_else(|| mime::OCTET_STREAM.to_owned()))
        .attributes(asset_attributes(file))
        .component(key(file, attributes));
    let stored = store::adopt(ctx, put, &written).await?;
    Ok(Published {
        file: file.clone(),
        sha256: stored.sha256,
        size: stored.size,
    })
}
