use std::collections::BTreeSet;

use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, ComponentKey};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_repo::config::WritePolicy;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::index::{self, IndexEntry};
use crate::package::{NOARCH, Package};
use crate::signing::{self, Keys, SigningKey};
use crate::{APK, APK_TYPE, ID, INDEX, INDEX_TYPE, files, paths};

const GENERATION: &str = "1";

pub enum Target<'a> {
    Path(&'a str),
    Directory(&'a str),
}

pub async fn authorize(ctx: &Ctx, path: &str) -> Result<(), ContentError> {
    let redeploys = ctx
        .repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
        == WritePolicy::Allow;
    let exists = store::find(ctx.db(), ctx.name(), path).await?.is_some();
    ctx.require(
        if exists && redeploys {
            View::Edit
        } else {
            View::Add
        },
        path,
    )
}

pub fn may_add(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Add).any() || ctx.grant(View::Edit).any() {
        true => Ok(()),
        false => ctx.require(View::Add, "/"),
    }
}

pub fn writable(path: &str) -> Result<(), ContentError> {
    if paths::is_package(path) {
        return Ok(());
    }
    Err(ContentError::BadRequest(format!(
        "A hosted Alpine repository stores .apk packages only; its APKINDEX.tar.gz files are generated: {path}"
    )))
}

pub fn index_arch(package: &Package, directory: &str) -> String {
    match package.info.arch() {
        NOARCH => paths::last_segment(directory).to_owned(),
        arch => arch.to_owned(),
    }
}

pub fn check_path(package: &Package, path: &str) -> Result<(), ContentError> {
    let expected = package.file_name();
    let directory = paths::directory(path);
    let arch = package.info.arch();
    let example = match arch {
        NOARCH => "x86_64",
        arch => arch,
    };
    if paths::file_name(path) != expected {
        return Err(ContentError::BadRequest(format!(
            "The package must be stored as {expected}, the file name apk asks for: {path}"
        )));
    }
    if directory.is_empty() {
        return Err(ContentError::BadRequest(format!(
            "Alpine packages are stored in a directory named after their architecture, for example {example}/{expected}"
        )));
    }
    if arch != NOARCH && paths::last_segment(directory) != arch {
        return Err(ContentError::BadRequest(format!(
            "{path} holds a package for {arch}; store it in a directory named {arch}, for example {arch}/{expected}"
        )));
    }
    Ok(())
}

pub fn target_path(package: &Package, directory: &str) -> Result<String, ContentError> {
    let directory = paths::join("", directory)
        .ok_or_else(|| ContentError::BadRequest(format!("{directory} is not a valid directory")))?;
    let directory = match (directory.is_empty(), package.info.arch()) {
        (false, _) => directory,
        (true, NOARCH) => {
            return Err(ContentError::BadRequest(
                "noarch packages need a directory that ends in an architecture, for example v3.20/main/x86_64".to_owned(),
            ));
        }
        (true, arch) => format!("/{arch}"),
    };
    let path = format!("{directory}/{}", package.file_name());
    check_path(package, &path)?;
    Ok(path)
}

fn refuse(ctx: &Ctx, written: &Written, error: ContentError) -> ContentError {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    error
}

pub async fn put(
    ctx: &Ctx,
    keys: &Keys,
    path: &str,
    body: ContentStream,
) -> Result<Asset, ContentError> {
    writable(path)?;
    authorize(ctx, path).await?;
    let written = store::write_blob(ctx, body).await?;
    adopt(ctx, keys, written, Target::Path(path)).await
}

pub async fn upload(
    ctx: &Ctx,
    keys: &Keys,
    directory: &str,
    body: ContentStream,
) -> Result<Asset, ContentError> {
    may_add(ctx)?;
    let written = store::write_blob(ctx, body).await?;
    adopt(ctx, keys, written, Target::Directory(directory)).await
}

async fn adopt(
    ctx: &Ctx,
    keys: &Keys,
    written: Written,
    target: Target<'_>,
) -> Result<Asset, ContentError> {
    let package = match files::read_package(ctx, &written.store, written.id, written.size).await {
        Ok(package) => package,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    let path = match target {
        Target::Path(path) => check_path(&package, path).map(|_| path.to_owned()),
        Target::Directory(directory) => target_path(&package, directory),
    };
    let path = match path {
        Ok(path) => path,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    if let Err(error) = authorize(ctx, &path).await {
        return Err(refuse(ctx, &written, error));
    }
    let directory = paths::directory(&path).to_owned();
    let mut entry = package.entry(written.size);
    entry.arch = index_arch(&package, &directory);
    let put = astrum_content::PutAsset::hosted(ctx, path)
        .component(ComponentKey {
            namespace: entry.arch.clone(),
            name: entry.name.clone(),
            version: entry.version.clone(),
            kind: String::new(),
            attributes: json!({}),
        })
        .kind(APK)
        .content_type(APK_TYPE)
        .attributes(json!({ ID: entry.attributes() }));
    let stored = store::adopt(ctx, put, &written).await?;
    rebuild(ctx, keys, &directory, false).await?;
    Ok(stored)
}

fn fingerprint(signer: &str, packages: &[Asset]) -> String {
    let mut digest = Sha256::new();
    digest.update(GENERATION.as_bytes());
    digest.update(b"\n");
    digest.update(signer.as_bytes());
    for found in packages {
        digest.update(format!("\n{}\t{}", found.path, found.sha256).as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_fingerprint(found: &Asset) -> Option<&str> {
    files::attribute(found, "fingerprint").and_then(Value::as_str)
}

pub fn description(repository: &str, directory: &str) -> String {
    match paths::branch(directory) {
        "" => repository.to_owned(),
        branch => format!("{repository} {branch}"),
    }
}

pub fn entries_of(packages: &[Asset]) -> Vec<IndexEntry> {
    packages
        .iter()
        .filter_map(|found| {
            let attributes = found.attributes.get(ID)?;
            IndexEntry::from_attributes(attributes, found.size.max(0) as u64)
        })
        .collect()
}

pub fn signed(key: Option<&SigningKey>, unsigned: Vec<u8>, mtime: u64) -> Vec<u8> {
    match key {
        Some(key) => [key.signature_member(&unsigned, mtime), unsigned].concat(),
        None => unsigned,
    }
}

pub async fn ensure(
    ctx: &Ctx,
    keys: &Keys,
    directory: &str,
) -> Result<Option<Asset>, ContentError> {
    let key = keys.get(ctx)?;
    let packages = files::in_directory(ctx.db(), ctx.name(), APK, directory).await?;
    let existing = store::find(ctx.db(), ctx.name(), &paths::index_path(directory)).await?;
    let wanted = fingerprint(&signing::signer(key.as_deref()), &packages);
    match &existing {
        Some(found) if stored_fingerprint(found) == Some(wanted.as_str()) => Ok(existing),
        None if packages.is_empty() => Ok(None),
        _ => rebuild(ctx, keys, directory, false).await,
    }
}

pub async fn rebuild(
    ctx: &Ctx,
    keys: &Keys,
    directory: &str,
    force: bool,
) -> Result<Option<Asset>, ContentError> {
    let key = keys.get(ctx)?;
    let signer = signing::signer(key.as_deref());
    let path = paths::index_path(directory);
    let txn = files::lock(ctx, directory).await?;
    let packages = files::in_directory(&txn, ctx.name(), APK, directory).await?;
    let existing = store::find(&txn, ctx.name(), &path).await?;
    let wanted = fingerprint(&signer, &packages);
    if packages.is_empty() {
        if let Some(found) = existing {
            store::delete_asset(&ctx.content, found).await?;
        }
        txn.commit().await?;
        return Ok(None);
    }
    if !force
        && existing
            .as_ref()
            .and_then(stored_fingerprint)
            .is_some_and(|stored| stored == wanted)
    {
        txn.commit().await?;
        return Ok(existing);
    }
    let entries = entries_of(&packages);
    let text = index::render(&entries);
    let mtime = chrono::Utc::now().timestamp().max(0) as u64;
    let unsigned = index::archive(&description(ctx.name(), directory), &text, mtime);
    let file = signed(key.as_deref(), unsigned, mtime);
    let attributes = json!({ ID: {
        "fingerprint": wanted,
        "packageCount": entries.len(),
        "signer": signer,
    }});
    let stored = store::put(
        ctx,
        files::metadata_put(ctx, &path, INDEX, INDEX_TYPE, attributes),
        astrum_blob::content_of(file),
    )
    .await?;
    txn.commit().await?;
    Ok(Some(stored))
}

pub async fn rebuild_all(ctx: &Ctx, keys: &Keys) -> Result<u64, ContentError> {
    let mut directories = BTreeSet::new();
    for kind in [APK, INDEX] {
        for found in files::of_kind(ctx.db(), ctx.name(), kind).await? {
            directories.insert(paths::directory(&found.path).to_owned());
        }
    }
    let mut rebuilt = 0;
    for directory in directories {
        if rebuild(ctx, keys, &directory, true).await?.is_some() {
            rebuilt += 1;
        }
    }
    Ok(rebuilt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package;
    use crate::testing;

    fn package(arch: &str) -> Package {
        package::read(&testing::build(&testing::Spec::new("hello", "1.0-r0", arch)).bytes).unwrap()
    }

    #[test]
    fn packages_go_into_the_directory_of_their_architecture() {
        let x86 = package("x86_64");
        assert!(check_path(&x86, "/v3.20/main/x86_64/hello-1.0-r0.apk").is_ok());
        assert!(check_path(&x86, "/v3.20/main/aarch64/hello-1.0-r0.apk").is_err());
        assert!(check_path(&x86, "/v3.20/main/x86_64/hello.apk").is_err());
        assert!(check_path(&x86, "/hello-1.0-r0.apk").is_err());
        assert_eq!(target_path(&x86, "").unwrap(), "/x86_64/hello-1.0-r0.apk");
        assert_eq!(
            target_path(&x86, "v3.20/main/x86_64/").unwrap(),
            "/v3.20/main/x86_64/hello-1.0-r0.apk"
        );
        assert!(target_path(&x86, "v3.20/main").is_err());
        assert!(target_path(&x86, "../x86_64").is_err());
        let any = package("noarch");
        assert!(target_path(&any, "").is_err());
        assert_eq!(
            target_path(&any, "edge/main/aarch64").unwrap(),
            "/edge/main/aarch64/hello-1.0-r0.apk"
        );
        assert_eq!(index_arch(&any, "/edge/main/aarch64"), "aarch64");
        assert_eq!(index_arch(&x86, "/x86_64"), "x86_64");
        assert!(writable("/x86_64/APKINDEX.tar.gz").is_err());
    }

    #[test]
    fn fingerprints_follow_packages_and_the_signer() {
        assert_eq!(description("apks", "/v3.20/main/x86_64"), "apks v3.20/main");
        assert_eq!(description("apks", "/x86_64"), "apks");
        assert_ne!(fingerprint("a", &[]), fingerprint("b", &[]));
    }
}
