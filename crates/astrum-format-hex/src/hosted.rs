use std::collections::BTreeSet;

use astrum_content::store::{self, ComponentKey};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_repo::config::WritePolicy;
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::keys::{Keys, RegistryKey};
use crate::registry::{self, Record, Retirement};
use crate::tarball::{self, Requirement};
use crate::{
    DOCS, DOCS_TYPE, ID, REGISTRY, REGISTRY_TYPE, Resource, TAR_TYPE, TARBALL, docs_path, files,
    tarball_path,
};

const GENERATION: &str = "1";

#[derive(Clone, Debug)]
pub struct Published {
    pub name: String,
    pub version: String,
    pub replaced: bool,
}

pub fn write_policy(ctx: &Ctx) -> WritePolicy {
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
        false => ctx.require(View::Add, "/"),
    }
}

pub fn record(found: &Asset) -> Option<Record> {
    let hex = found.attributes.get(ID)?;
    Some(Record {
        name: hex.get("name")?.as_str()?.to_owned(),
        version: hex.get("version")?.as_str()?.to_owned(),
        inner_checksum: hex::decode(hex.get("innerChecksum")?.as_str()?).ok()?,
        outer_checksum: hex::decode(&found.sha256).ok()?,
        requirements: hex
            .get("requirements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Requirement::from_attributes)
            .collect(),
        retired: hex.get("retired").and_then(Retirement::from_attributes),
        updated_at: found.blob_updated_at.timestamp(),
    })
}

fn fingerprint(
    key: &RegistryKey,
    repository: &str,
    resource: &Resource,
    found: &[Asset],
) -> String {
    let mut digest = Sha256::new();
    digest.update(GENERATION.as_bytes());
    digest.update(format!("\n{}\n{repository}\n{}", key.fingerprint, resource.path()).as_bytes());
    for found in found {
        digest.update(
            format!(
                "\n{}\t{}\t{}\t{}",
                found.path,
                found.sha256,
                files::attribute(found, "retired")
                    .cloned()
                    .unwrap_or(Value::Null),
                found.blob_updated_at.timestamp()
            )
            .as_bytes(),
        );
    }
    hex::encode(digest.finalize())
}

fn stored_fingerprint(found: &Asset) -> Option<&str> {
    files::attribute(found, "fingerprint").and_then(Value::as_str)
}

async fn relevant(
    db: &impl sea_orm::ConnectionTrait,
    ctx: &Ctx,
    resource: &Resource,
) -> Result<Vec<Asset>, ContentError> {
    match resource {
        Resource::Package(name) => files::tarballs_of(db, ctx.name(), name).await,
        _ => files::of_kind(db, ctx.name(), TARBALL).await,
    }
}

fn payload(repository: &str, resource: &Resource, records: &[Record]) -> Vec<u8> {
    match resource {
        Resource::Names => registry::names(repository, records).encode_to_vec(),
        Resource::Versions => registry::versions(repository, records).encode_to_vec(),
        Resource::Package(name) => registry::package(repository, name, records).encode_to_vec(),
    }
}

async fn write(
    ctx: &Ctx,
    key: &RegistryKey,
    resource: &Resource,
    found: &[Asset],
    force: bool,
) -> Result<Option<Asset>, ContentError> {
    let path = resource.path();
    let existing = store::find(ctx.db(), ctx.name(), &path).await?;
    if let Resource::Package(_) = resource
        && found.is_empty()
    {
        if let Some(stale) = existing {
            store::delete_asset(&ctx.content, stale).await?;
        }
        return Ok(None);
    }
    let wanted = fingerprint(key, ctx.name(), resource, found);
    if !force
        && let Some(existing) = &existing
        && stored_fingerprint(existing) == Some(wanted.as_str())
    {
        return Ok(Some(existing.clone()));
    }
    let records: Vec<Record> = found.iter().filter_map(record).collect();
    let signed = key.sign(payload(ctx.name(), resource, &records));
    let attributes = json!({ ID: {
        "fingerprint": wanted,
        "releaseCount": records.len(),
    }});
    let stored = store::put(
        ctx,
        files::metadata_put(ctx, &path, REGISTRY, REGISTRY_TYPE, attributes),
        astrum_blob::content_of(signed),
    )
    .await?;
    Ok(Some(stored))
}

pub async fn ensure(
    ctx: &Ctx,
    keys: &Keys,
    resource: &Resource,
) -> Result<Option<Asset>, ContentError> {
    let key = keys.get(ctx).await?;
    let found = relevant(ctx.db(), ctx, resource).await?;
    let existing = store::find(ctx.db(), ctx.name(), &resource.path()).await?;
    let wanted = fingerprint(&key, ctx.name(), resource, &found);
    match &existing {
        Some(stored) if stored_fingerprint(stored) == Some(wanted.as_str()) => Ok(existing),
        None if found.is_empty() && matches!(resource, Resource::Package(_)) => Ok(None),
        _ => {
            let txn = files::lock(ctx).await?;
            let found = relevant(&txn, ctx, resource).await?;
            let written = write(ctx, &key, resource, &found, false).await?;
            txn.commit().await?;
            Ok(written)
        }
    }
}

pub async fn rebuild(
    ctx: &Ctx,
    keys: &Keys,
    touched: Option<&str>,
    force: bool,
) -> Result<u64, ContentError> {
    let key = keys.get(ctx).await?;
    let txn = files::lock(ctx).await?;
    let all = files::of_kind(&txn, ctx.name(), TARBALL).await?;
    let mut packages: BTreeSet<String> = BTreeSet::new();
    match touched {
        Some(name) => {
            packages.insert(name.to_owned());
        }
        None => {
            for found in all
                .iter()
                .chain(files::of_kind(&txn, ctx.name(), REGISTRY).await?.iter())
            {
                if let Some(name) = found.path.strip_prefix("/packages/") {
                    packages.insert(name.to_owned());
                } else if let Some((name, _)) = found
                    .path
                    .strip_prefix("/tarballs/")
                    .and_then(tarball::split_file)
                {
                    packages.insert(name.to_owned());
                }
            }
        }
    }
    let mut written = 0;
    for name in packages {
        let found: Vec<Asset> = all
            .iter()
            .filter(|found| {
                found
                    .path
                    .strip_prefix("/tarballs/")
                    .and_then(tarball::split_file)
                    .is_some_and(|(found_name, _)| found_name == name)
            })
            .cloned()
            .collect();
        if write(ctx, &key, &Resource::Package(name), &found, force)
            .await?
            .is_some()
        {
            written += 1;
        }
    }
    for resource in [Resource::Names, Resource::Versions] {
        write(ctx, &key, &resource, &all, force).await?;
        written += 1;
    }
    txn.commit().await?;
    Ok(written)
}

fn refused(error: tarball::TarballError) -> ContentError {
    ContentError::BadRequest(format!("the package was refused: {error}"))
}

pub async fn publish(
    ctx: &Ctx,
    keys: &Keys,
    body: bytes::Bytes,
    replace: bool,
    name: Option<&str>,
    version: Option<&str>,
) -> Result<Published, ContentError> {
    let read = {
        let body = body.clone();
        tokio::task::spawn_blocking(move || tarball::read(&body))
            .await
            .map_err(ContentError::internal)?
            .map_err(refused)?
    };
    let metadata = &read.metadata;
    if name.is_some_and(|name| name != metadata.name)
        || version.is_some_and(|version| version != metadata.version)
    {
        return Err(ContentError::BadRequest(format!(
            "the tarball holds {} {}, not {} {}",
            metadata.name,
            metadata.version,
            name.unwrap_or(&metadata.name),
            version.unwrap_or(&metadata.version)
        )));
    }
    let path = tarball_path(&metadata.name, &metadata.version);
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    let policy = write_policy(ctx);
    match exists {
        true => ctx.require(View::Edit, &path)?,
        false => ctx.require(View::Add, &path)?,
    }
    if policy == WritePolicy::Deny {
        return Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        )));
    }
    if exists && !replace {
        return Err(ContentError::Conflict(format!(
            "version {} of {} is already in {}; publish it with --replace to overwrite it",
            metadata.version,
            metadata.name,
            ctx.name()
        )));
    }
    if exists && policy == WritePolicy::AllowOnce {
        return Err(ContentError::Conflict(format!(
            "version {} of {} is already in {} and the repository does not allow redeploying",
            metadata.version,
            metadata.name,
            ctx.name()
        )));
    }
    let mut put = store::PutAsset::hosted(ctx, &path)
        .kind(TARBALL)
        .content_type(TAR_TYPE)
        .attributes(json!({ ID: read.attributes() }))
        .component(ComponentKey {
            namespace: String::new(),
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            kind: String::new(),
            attributes: json!({ ID: metadata.summary() }),
        });
    put.check_content_type = false;
    store::put(ctx, put, astrum_blob::content_of(body)).await?;
    rebuild(ctx, keys, Some(&metadata.name), false).await?;
    Ok(Published {
        name: metadata.name.clone(),
        version: metadata.version.clone(),
        replaced: exists,
    })
}

pub async fn release(ctx: &Ctx, name: &str, version: &str) -> Result<Asset, ContentError> {
    let path = tarball_path(name, version);
    ctx.require(View::Read, &path)?;
    store::find(ctx.db(), ctx.name(), &path)
        .await?
        .ok_or_else(|| {
            ContentError::not_found(format!("{name} {version} is not in {}", ctx.name()))
        })
}

pub async fn remove(ctx: &Ctx, keys: &Keys, name: &str, version: &str) -> Result<(), ContentError> {
    let path = tarball_path(name, version);
    ctx.require(View::Delete, &path)?;
    let found = store::find(ctx.db(), ctx.name(), &path)
        .await?
        .ok_or_else(|| {
            ContentError::not_found(format!("{name} {version} is not in {}", ctx.name()))
        })?;
    if let Some(docs) = store::find(ctx.db(), ctx.name(), &docs_path(name, version)).await? {
        store::delete_asset(&ctx.content, docs).await?;
    }
    store::delete_asset(&ctx.content, found).await?;
    rebuild(ctx, keys, Some(name), false).await?;
    Ok(())
}

pub async fn has_docs(ctx: &Ctx, name: &str, version: &str) -> Result<bool, ContentError> {
    Ok(store::find(ctx.db(), ctx.name(), &docs_path(name, version))
        .await?
        .is_some())
}

pub async fn put_docs(
    ctx: &Ctx,
    name: &str,
    version: &str,
    body: bytes::Bytes,
) -> Result<bool, ContentError> {
    let release = store::find(ctx.db(), ctx.name(), &tarball_path(name, version))
        .await?
        .ok_or_else(|| {
            ContentError::not_found(format!(
                "{name} {version} is not in {}; publish the package before its documentation",
                ctx.name()
            ))
        })?;
    let path = docs_path(name, version);
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    match exists {
        true => ctx.require(View::Edit, &path)?,
        false => ctx.require(View::Add, &path)?,
    }
    if write_policy(ctx) == WritePolicy::Deny {
        return Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        )));
    }
    if !body.starts_with(&[0x1f, 0x8b]) {
        return Err(ContentError::BadRequest(
            "the documentation must be a gzip-compressed tarball".to_owned(),
        ));
    }
    let mut put = files::metadata_put(
        ctx,
        &path,
        DOCS,
        DOCS_TYPE,
        json!({ ID: {"name": name, "version": version} }),
    );
    if let Some(component) = match release.component_id {
        Some(id) => store::component_by_id(ctx.db(), id).await?,
        None => None,
    } {
        put = put.component(ComponentKey {
            namespace: component.namespace,
            name: component.name,
            version: component.version,
            kind: component.kind,
            attributes: component.attributes,
        });
    }
    store::put(ctx, put, astrum_blob::content_of(body)).await?;
    Ok(exists)
}

pub async fn remove_docs(ctx: &Ctx, name: &str, version: &str) -> Result<(), ContentError> {
    let path = docs_path(name, version);
    ctx.require(View::Delete, &path)?;
    let found = store::find(ctx.db(), ctx.name(), &path)
        .await?
        .ok_or_else(|| ContentError::not_found(format!("{name} {version} has no documentation")))?;
    store::delete_asset(&ctx.content, found).await
}

pub async fn retire(
    ctx: &Ctx,
    keys: &Keys,
    name: &str,
    version: &str,
    retirement: Option<Retirement>,
) -> Result<(), ContentError> {
    let path = tarball_path(name, version);
    ctx.require(View::Edit, &path)?;
    let found = store::find(ctx.db(), ctx.name(), &path)
        .await?
        .ok_or_else(|| {
            ContentError::not_found(format!("{name} {version} is not in {}", ctx.name()))
        })?;
    let mut attributes = found.attributes.clone();
    if let Some(hex) = attributes.get_mut(ID).and_then(Value::as_object_mut) {
        match &retirement {
            Some(retirement) => {
                hex.insert("retired".to_owned(), retirement.attributes());
            }
            None => {
                hex.remove("retired");
            }
        }
    }
    store::set_attributes(ctx.db(), found.id, attributes).await?;
    rebuild(ctx, keys, Some(name), false).await?;
    Ok(())
}

pub async fn releases(ctx: &Ctx, name: &str) -> Result<Vec<(Asset, Record)>, ContentError> {
    let mut found: Vec<(Asset, Record)> = files::tarballs_of(ctx.db(), ctx.name(), name)
        .await?
        .into_iter()
        .filter(|found| ctx.allowed(View::Read, &found.path))
        .filter_map(|found| record(&found).map(|record| (found, record)))
        .collect();
    found.sort_by(|left, right| registry::compare_versions(&left.1.version, &right.1.version));
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_come_from_the_tarball_attributes() {
        let built = crate::testing::Spec::new("hello", "1.0.0")
            .requirement("jason", "~> 1.4", false)
            .build();
        let read = tarball::read(&built.bytes).unwrap();
        let now = chrono::Utc::now().fixed_offset();
        let found = Asset {
            id: 1,
            repository: "r".to_owned(),
            format: ID.to_owned(),
            path: tarball_path("hello", "1.0.0"),
            kind: TARBALL.to_owned(),
            component_id: None,
            blob_store: "default".to_owned(),
            blob_id: uuid_zero(),
            size: built.bytes.len() as i64,
            content_type: TAR_TYPE.to_owned(),
            sha1: String::new(),
            sha256: hex::encode(built.outer_checksum),
            sha512: String::new(),
            md5: String::new(),
            attributes: json!({ ID: read.attributes() }),
            uploader: "admin".to_owned(),
            uploader_ip: String::new(),
            created_at: now,
            blob_updated_at: now,
            last_downloaded_at: None,
            download_count: 0,
            last_verified_at: None,
            remote_etag: None,
            remote_last_modified: None,
        };
        let record = record(&found).unwrap();
        assert_eq!(record.name, "hello");
        assert_eq!(record.inner_checksum, built.inner_checksum);
        assert_eq!(record.outer_checksum, built.outer_checksum);
        assert_eq!(record.requirements[0].name, "jason");
        assert_eq!(record.retired, None);
    }

    fn uuid_zero() -> sea_orm::prelude::Uuid {
        sea_orm::prelude::Uuid::nil()
    }
}
