use std::collections::{HashMap, HashSet};

use astrum_blob::ContentStream;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::Value;

use crate::ID;
use crate::blobs;
use crate::documents::{self, Digests, Document, MAX_DOCUMENT};
use crate::module_file;
use crate::names::{self, Entry, REGISTRY_PATH, VersionFile};

pub const REGISTRY_KIND: &str = "registry";
pub const METADATA_KIND: &str = "metadata";
pub const JSON_TYPE: &str = "application/json";

const MAX_TEXT: usize = 4 * 1024 * 1024;

fn bad(message: impl Into<String>) -> ContentError {
    ContentError::BadRequest(message.into())
}

fn configured_policy(ctx: &Ctx) -> WritePolicy {
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

fn ensure_writable(ctx: &Ctx) -> Result<(), ContentError> {
    match configured_policy(ctx) {
        WritePolicy::Deny => Err(bad(format!("the repository {} is read-only", ctx.name()))),
        _ => Ok(()),
    }
}

pub fn component_key(file: &VersionFile) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: file.module.clone(),
        version: file.version.clone(),
        kind: String::new(),
        attributes: Value::Object(Default::default()),
    }
}

pub async fn has_module(ctx: &Ctx, module: &str) -> Result<bool, ContentError> {
    let known = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Name.eq(module))
        .one(ctx.db())
        .await?
        .is_some();
    if known {
        return Ok(true);
    }
    Ok(
        store::find(ctx.db(), ctx.name(), &names::metadata_path(module))
            .await?
            .is_some(),
    )
}

pub async fn versions(ctx: &Ctx, module: &str) -> Result<Vec<String>, ContentError> {
    let components = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Name.eq(module))
        .all(ctx.db())
        .await?;
    if components.is_empty() {
        return Ok(Vec::new());
    }
    let by_id: HashMap<i64, &component::Model> =
        components.iter().map(|one| (one.id, one)).collect();
    let files = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::ComponentId.is_in(by_id.keys().copied().collect::<Vec<_>>()))
        .filter(asset::Column::Kind.is_in([Entry::Module.kind(), Entry::Source.kind()]))
        .all(ctx.db())
        .await?;
    let mut present: HashMap<i64, HashSet<&'static str>> = HashMap::new();
    for stored in &files {
        let Some(owner) = stored.component_id.and_then(|id| by_id.get(&id)) else {
            continue;
        };
        let entry = match stored.kind == Entry::Module.kind() {
            true => Entry::Module,
            false => Entry::Source,
        };
        let expected = VersionFile {
            module: module.to_owned(),
            version: owner.version.clone(),
            entry: entry.clone(),
        };
        if stored.path == expected.path() {
            present.entry(owner.id).or_default().insert(entry.kind());
        }
    }
    let mut listed: Vec<String> = components
        .iter()
        .filter(|one| present.get(&one.id).is_some_and(|kinds| kinds.len() == 2))
        .map(|one| one.version.clone())
        .collect();
    names::sort_versions(&mut listed);
    Ok(listed)
}

async fn stored_document(ctx: &Ctx, path: &str) -> Result<Option<(Asset, Document)>, ContentError> {
    let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
        return Ok(None);
    };
    let body = blobs::read_all(ctx, &found, MAX_DOCUMENT).await?;
    match documents::parse(&body, path) {
        Ok(document) => Ok(Some((found, document))),
        Err(error) => {
            tracing::warn!(
                repository = ctx.name(),
                path,
                error,
                "a stored document is not valid"
            );
            Ok(None)
        }
    }
}

pub async fn metadata(ctx: &Ctx, module: &str) -> Result<Option<Document>, ContentError> {
    let stored = stored_document(ctx, &names::metadata_path(module))
        .await?
        .map(|(_, document)| document);
    let versions = versions(ctx, module).await?;
    if stored.is_none() && versions.is_empty() {
        return Ok(None);
    }
    Ok(Some(documents::build_metadata(stored, versions)))
}

pub async fn registry(ctx: &Ctx) -> Result<Document, ContentError> {
    Ok(match stored_document(ctx, REGISTRY_PATH).await? {
        Some((_, document)) => document,
        None => documents::merge_registries(&[]),
    })
}

pub async fn put_registry(ctx: &Ctx, body: ContentStream) -> Result<bool, ContentError> {
    put_document(ctx, REGISTRY_PATH, REGISTRY_KIND, body, |bytes| {
        documents::validate_registry(bytes).map(|_| ())
    })
    .await
}

pub async fn put_metadata(
    ctx: &Ctx,
    module: &str,
    body: ContentStream,
) -> Result<bool, ContentError> {
    put_document(
        ctx,
        &names::metadata_path(module),
        METADATA_KIND,
        body,
        |bytes| documents::validate_metadata(bytes).map(|_| ()),
    )
    .await
}

async fn put_document(
    ctx: &Ctx,
    path: &str,
    kind: &str,
    body: ContentStream,
    validate: impl FnOnce(&[u8]) -> Result<(), String>,
) -> Result<bool, ContentError> {
    let existing = store::find(ctx.db(), ctx.name(), path).await?;
    ctx.require(
        match existing {
            Some(_) => View::Edit,
            None => View::Add,
        },
        path,
    )?;
    ensure_writable(ctx)?;
    let bytes = blobs::collect(body, MAX_DOCUMENT, path).await?;
    validate(&bytes).map_err(bad)?;
    let mut put = PutAsset::hosted(ctx, path)
        .kind(kind)
        .content_type(JSON_TYPE);
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::Allow);
    store::put(ctx, put, blobs::once(bytes)).await?;
    Ok(existing.is_some())
}

pub async fn ensure_new(ctx: &Ctx, file: &VersionFile) -> Result<(), ContentError> {
    match store::find(ctx.db(), ctx.name(), &file.path()).await? {
        Some(_) => Err(exists_error(ctx, file)),
        None => Ok(()),
    }
}

pub fn exists_error(ctx: &Ctx, file: &VersionFile) -> ContentError {
    ContentError::Conflict(format!(
        "{} of {} {} already exists in {}; the files of a published version are not replaced, delete it first or publish a new version",
        file.entry.relative(),
        file.module,
        file.version,
        ctx.name()
    ))
}

fn mismatch(file: &VersionFile, integrity: &str) -> ContentError {
    bad(format!(
        "{} does not match the integrity {integrity} that source.json lists for it",
        file.entry.relative()
    ))
}

async fn verify_listed_files(
    ctx: &Ctx,
    file: &VersionFile,
    source: &documents::Source,
) -> Result<(), ContentError> {
    for (entry, integrity) in source.entries() {
        let Some(listed) = VersionFile::new(&file.module, &file.version, entry) else {
            continue;
        };
        let Some(found) = store::find(ctx.db(), ctx.name(), &listed.path()).await? else {
            continue;
        };
        let digests = Digests {
            sha1: &found.sha1,
            sha256: &found.sha256,
            sha512: &found.sha512,
        };
        if documents::integrity_matches(integrity, digests) == Some(false) {
            return Err(mismatch(&listed, integrity));
        }
    }
    Ok(())
}

async fn listed_integrity(ctx: &Ctx, file: &VersionFile) -> Result<Option<String>, ContentError> {
    let Some(source) = VersionFile::new(&file.module, &file.version, Entry::Source) else {
        return Ok(None);
    };
    let Some(found) = store::find(ctx.db(), ctx.name(), &source.path()).await? else {
        return Ok(None);
    };
    let body = blobs::read_all(ctx, &found, MAX_TEXT).await?;
    Ok(documents::validate_source(&body)
        .ok()
        .and_then(|source| source.integrity_of(&file.entry).map(str::to_owned)))
}

fn check_module_file(file: &VersionFile, bytes: &[u8]) -> Result<(), ContentError> {
    let text = std::str::from_utf8(bytes).map_err(|_| bad("MODULE.bazel must be UTF-8 text"))?;
    if text.contains('\0') {
        return Err(bad("MODULE.bazel must be text"));
    }
    let Some(declared) = module_file::declaration(text) else {
        return Ok(());
    };
    if let Some(name) = declared.name
        && name != file.module
    {
        return Err(bad(format!(
            "MODULE.bazel declares the module {name}, but it is published as {}",
            file.module
        )));
    }
    if let Some(version) = declared.version.filter(|version| !version.is_empty())
        && version != file.version
    {
        return Err(bad(format!(
            "MODULE.bazel declares version {version}, but it is published as {}; Bazel refuses such a file",
            file.version
        )));
    }
    Ok(())
}

pub async fn put_file(
    ctx: &Ctx,
    file: &VersionFile,
    body: ContentStream,
) -> Result<Asset, ContentError> {
    let path = file.path();
    ctx.require(View::Add, &path)?;
    ensure_writable(ctx)?;
    ensure_new(ctx, file).await?;
    let new_put = || {
        let mut put = PutAsset::hosted(ctx, path.clone())
            .kind(file.entry.kind())
            .content_type(file.entry.content_type())
            .component(component_key(file));
        put.check_content_type = false;
        put.write_policy = Some(WritePolicy::AllowOnce);
        put
    };
    match &file.entry {
        Entry::Module => {
            let bytes = blobs::collect(body, MAX_TEXT, "MODULE.bazel").await?;
            check_module_file(file, &bytes)?;
            store::put(ctx, new_put(), blobs::once(bytes)).await
        }
        Entry::Source => {
            let bytes = blobs::collect(body, MAX_DOCUMENT, "source.json").await?;
            let source = documents::validate_source(&bytes).map_err(bad)?;
            verify_listed_files(ctx, file, &source).await?;
            store::put(ctx, new_put(), blobs::once(bytes)).await
        }
        Entry::Patch(_) | Entry::Overlay(_) => {
            let written = store::write_blob(ctx, body).await?;
            let expected = match listed_integrity(ctx, file).await {
                Ok(expected) => expected,
                Err(error) => {
                    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                    return Err(error);
                }
            };
            if let Some(integrity) = expected {
                let digests = Digests {
                    sha1: &written.digests.sha1,
                    sha256: &written.digests.sha256,
                    sha512: &written.digests.sha512,
                };
                if documents::integrity_matches(&integrity, digests) == Some(false) {
                    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                    return Err(mismatch(file, &integrity));
                }
            }
            store::adopt(ctx, new_put(), &written).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_file_becomes_a_component_of_its_module() {
        let file = VersionFile::new("zlib", "1.3.1", Entry::Module).unwrap();
        let key = component_key(&file);
        assert_eq!(
            (
                key.namespace.as_str(),
                key.name.as_str(),
                key.version.as_str()
            ),
            ("", "zlib", "1.3.1")
        );
    }

    #[test]
    fn a_module_file_must_declare_the_module_it_is_published_as() {
        let file = VersionFile::new("zlib", "1.3.1", Entry::Module).unwrap();
        assert!(check_module_file(&file, b"module(name = \"zlib\", version = \"1.3.1\")").is_ok());
        assert!(check_module_file(&file, b"bazel_dep(name = \"a\", version = \"1\")").is_ok());
        assert!(check_module_file(&file, b"module(name = \"other\")").is_err());
        assert!(check_module_file(&file, b"module(name = \"zlib\", version = \"1.3.2\")").is_err());
        assert!(check_module_file(&file, b"module(name = \"zlib\", version = \"\")").is_ok());
        assert!(check_module_file(&file, b"module(name = \"zlib\")").is_ok());
        assert!(check_module_file(&file, b"module(name = \"zlib\")\x00").is_err());
        assert!(check_module_file(&file, &[0xff, 0xfe]).is_err());
    }
}
