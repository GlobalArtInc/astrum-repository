use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, UploadForm, View};
use astrum_repo::config::WritePolicy;
use base64::Engine;
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::archive::Compression;
use crate::db::{self, Entry, Record};
use crate::package::{Contents, PkgInfo};
use crate::paths::{self, ANY, PackageFile, Route, SIGNATURE_SUFFIX, Variant};
use crate::signing::{self, SIGNATURE_LIMIT, SIGNATURE_TYPE};
use crate::{DATABASE, DATABASE_TYPE, ID, PACKAGE, SIGNATURE, files, version};

const GENERATION: &str = "1";

pub type Known = HashMap<String, Vec<String>>;

#[derive(Clone, Debug)]
pub struct Generated {
    pub db: Asset,
    pub files: Asset,
}

impl Generated {
    pub fn of(self, variant: Variant) -> Asset {
        match variant {
            Variant::Db => self.db,
            Variant::Files => self.files,
        }
    }
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

pub enum Upload {
    Package,
    Signature(PackageFile),
}

pub fn writable(path: &str) -> Result<Upload, ContentError> {
    match paths::route(path) {
        Route::Package(_) => Ok(Upload::Package),
        Route::Signature(file) => Ok(Upload::Signature(file)),
        Route::Database(_) | Route::DatabaseSignature(_) => Err(ContentError::BadRequest(format!(
            "The databases of a hosted pacman repository are generated from its packages: {path}"
        ))),
        _ => Err(ContentError::BadRequest(format!(
            "A hosted pacman repository stores packages (.pkg.tar.*) and their signatures (.sig) only: {path}"
        ))),
    }
}

pub fn check_directory(arch: &str, path: &str) -> Result<(), ContentError> {
    let directory = paths::directory(path);
    let file = paths::file_name(path);
    let example = match arch {
        ANY => "x86_64",
        arch => arch,
    };
    if directory.is_empty() {
        return Err(ContentError::BadRequest(format!(
            "pacman packages are stored in a directory named after their architecture, for example {example}/{file}"
        )));
    }
    if arch != ANY && paths::last_segment(directory) != arch {
        return Err(ContentError::BadRequest(format!(
            "{path} holds a package for {arch}; store it in a directory named {arch}, for example {}/{arch}/{file}",
            paths::directory(directory)
        )));
    }
    Ok(())
}

pub fn extension(compression: Compression) -> &'static str {
    match compression {
        Compression::None => "",
        Compression::Gzip => ".gz",
        Compression::Bzip2 => ".bz2",
        Compression::Xz => ".xz",
        Compression::Zstd => ".zst",
    }
}

pub fn check_path(contents: &Contents, path: &str) -> Result<(), ContentError> {
    let file = paths::file_name(path);
    let given = PackageFile::parse(file).map(|file| file.extension);
    let expected = contents
        .info
        .file_name(given.as_deref().unwrap_or(extension(contents.compression)));
    if file != expected {
        return Err(ContentError::BadRequest(format!(
            "The package must be stored as {expected}, the file name its .PKGINFO gives: {path}"
        )));
    }
    check_directory(contents.info.arch(), path)
}

pub fn target_path(contents: &Contents, directory: &str) -> Result<String, ContentError> {
    let base = paths::join("", directory)
        .ok_or_else(|| ContentError::BadRequest(format!("{directory} is not a valid directory")))?;
    Ok(format!(
        "{base}/{}/{}",
        contents.info.arch(),
        contents.info.file_name(extension(contents.compression))
    ))
}

pub fn component(info: &PkgInfo) -> ComponentKey {
    ComponentKey {
        namespace: info.arch().to_owned(),
        name: info.name().to_owned(),
        version: info.version().to_owned(),
        kind: String::new(),
        attributes: json!({}),
    }
}

pub fn file_component(file: &PackageFile) -> ComponentKey {
    ComponentKey {
        namespace: file.arch.clone(),
        name: file.name.clone(),
        version: file.version.clone(),
        kind: String::new(),
        attributes: json!({}),
    }
}

fn refuse(ctx: &Ctx, written: &Written, error: ContentError) -> ContentError {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    error
}

enum Target<'a> {
    Path(&'a str),
    Directory(&'a str),
}

async fn adopt_package(
    ctx: &Ctx,
    written: Written,
    target: Target<'_>,
) -> Result<(Asset, Contents), ContentError> {
    let contents = match files::read_package(ctx, &written.store, written.id).await {
        Ok(contents) => contents,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    let path = match target {
        Target::Path(path) => check_path(&contents, path).map(|_| path.to_owned()),
        Target::Directory(directory) => target_path(&contents, directory),
    };
    let path = match path {
        Ok(path) => path,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    if let Err(error) = authorize(ctx, &path).await {
        return Err(refuse(ctx, &written, error));
    }
    let put = PutAsset::hosted(ctx, path)
        .component(component(&contents.info))
        .kind(PACKAGE)
        .content_type(contents.compression.content_type())
        .attributes(json!({ ID: contents.info.attributes() }));
    let stored = store::adopt(ctx, put, &written).await?;
    Ok((stored, contents))
}

async fn adopt_signature(
    ctx: &Ctx,
    written: Written,
    path: &str,
    file: &PackageFile,
) -> Result<Asset, ContentError> {
    if written.size > SIGNATURE_LIMIT {
        let error = ContentError::BadRequest(format!(
            "A package signature is at most {SIGNATURE_LIMIT} bytes long, the limit of pacman: {path}"
        ));
        return Err(refuse(ctx, &written, error));
    }
    let bytes = match files::range(ctx, &written.store, written.id, 0, written.size).await {
        Ok(bytes) => bytes,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    if signing::binary_signature(&bytes).is_none() {
        let error = ContentError::BadRequest(format!("Not a valid OpenPGP signature: {path}"));
        return Err(refuse(ctx, &written, error));
    }
    let checks = async {
        check_directory(&file.arch, path)?;
        authorize(ctx, path).await
    };
    if let Err(error) = checks.await {
        return Err(refuse(ctx, &written, error));
    }
    let put = PutAsset::hosted(ctx, path)
        .component(file_component(file))
        .kind(SIGNATURE)
        .content_type(SIGNATURE_TYPE);
    store::adopt(ctx, put, &written).await
}

pub async fn put(ctx: &Ctx, path: &str, body: ContentStream) -> Result<Asset, ContentError> {
    let upload = writable(path)?;
    authorize(ctx, path).await?;
    let written = store::write_blob(ctx, body).await?;
    let mut known = Known::new();
    let stored = match upload {
        Upload::Package => {
            let (stored, contents) = adopt_package(ctx, written, Target::Path(path)).await?;
            known.insert(stored.sha256.clone(), contents.files);
            stored
        }
        Upload::Signature(file) => adopt_signature(ctx, written, path, &file).await?,
    };
    refresh(ctx, [paths::directory(&stored.path).to_owned()], &known).await?;
    Ok(stored)
}

fn upload_files(form: &UploadForm) -> Vec<&astrum_content::format::UploadedFile> {
    let prefix = format!("{ID}.asset");
    form.files
        .iter()
        .filter(|(name, _)| {
            name.as_str() == "asset"
                || name
                    .strip_prefix(&prefix)
                    .is_some_and(|index| index.bytes().all(|byte| byte.is_ascii_digit()))
        })
        .map(|(_, file)| file)
        .collect()
}

async fn is_signature(file: &astrum_content::format::UploadedFile) -> Result<bool, ContentError> {
    if file.size > SIGNATURE_LIMIT {
        return Ok(false);
    }
    let bytes = tokio::fs::read(&file.path)
        .await
        .map_err(ContentError::internal)?;
    Ok(signing::binary_signature(&bytes).is_some())
}

pub async fn upload(ctx: &Ctx, form: &UploadForm) -> Result<(), ContentError> {
    may_add(ctx)?;
    let directory = form
        .field(&format!("{ID}.directory"))
        .or_else(|| form.field("directory"))
        .unwrap_or("")
        .trim()
        .to_owned();
    let base = paths::join("", &directory)
        .ok_or_else(|| ContentError::BadRequest(format!("{directory} is not a valid directory")))?;
    let uploaded = upload_files(form);
    if uploaded.is_empty() {
        return Err(ContentError::BadRequest(format!("{ID}.asset is required")));
    }
    let mut packages = Vec::new();
    let mut signatures = Vec::new();
    for file in uploaded {
        match is_signature(file).await? {
            true => signatures.push(file),
            false => packages.push(file),
        }
    }
    let mut known = Known::new();
    let mut stored: Vec<Asset> = Vec::new();
    for file in packages {
        let written = store::write_blob(ctx, UploadForm::stream(file).await?).await?;
        let (asset, contents) = adopt_package(ctx, written, Target::Directory(&base)).await?;
        known.insert(asset.sha256.clone(), contents.files);
        stored.push(asset);
    }
    let packages: Vec<String> = stored.iter().map(|asset| asset.path.clone()).collect();
    for file in signatures {
        let named = file
            .filename
            .as_deref()
            .map(paths::file_name)
            .and_then(|name| name.strip_suffix(SIGNATURE_SUFFIX))
            .and_then(PackageFile::parse);
        let target = match (&named, packages.as_slice()) {
            (Some(named), _) => {
                let wanted = named.file_name();
                match packages
                    .iter()
                    .find(|path| paths::file_name(path) == wanted)
                {
                    Some(path) => format!("{path}{SIGNATURE_SUFFIX}"),
                    None => format!("{base}/{}/{wanted}{SIGNATURE_SUFFIX}", named.arch),
                }
            }
            (None, [only]) => format!("{only}{SIGNATURE_SUFFIX}"),
            (None, _) => {
                return Err(ContentError::BadRequest(
                    "Name each signature after its package, as <package file>.sig".to_owned(),
                ));
            }
        };
        let package = PackageFile::parse(
            paths::file_name(&target)
                .strip_suffix(SIGNATURE_SUFFIX)
                .unwrap_or_default(),
        )
        .ok_or_else(|| {
            ContentError::BadRequest(format!("{target} does not name a package signature"))
        })?;
        let written = store::write_blob(ctx, UploadForm::stream(file).await?).await?;
        stored.push(adopt_signature(ctx, written, &target, &package).await?);
    }
    let directories: BTreeSet<String> = stored
        .iter()
        .map(|asset| paths::directory(&asset.path).to_owned())
        .collect();
    refresh(ctx, directories, &known).await
}

pub async fn find_package(
    ctx: &Ctx,
    path: &str,
    file: &PackageFile,
) -> Result<Option<Asset>, ContentError> {
    if let Some(found) = store::find(ctx.db(), ctx.name(), path).await? {
        return Ok(Some(found));
    }
    let directory = paths::directory(path);
    if file.arch != ANY || directory.is_empty() || paths::is_any(directory) {
        return Ok(None);
    }
    let shared = format!(
        "{}/{}",
        paths::any_directory(directory),
        paths::file_name(path)
    );
    store::find(ctx.db(), ctx.name(), &shared).await
}

#[derive(Default)]
struct Inputs {
    packages: Vec<(Asset, bool)>,
    signatures: HashMap<String, Asset>,
}

async fn inputs(
    db: &impl ConnectionTrait,
    repository: &str,
    directory: &str,
) -> Result<Inputs, ContentError> {
    let own = files::in_directory(db, repository, PACKAGE, directory).await?;
    if own.is_empty() && !paths::is_any(directory) {
        return Ok(Inputs::default());
    }
    let mut packages: Vec<(Asset, bool)> = own.into_iter().map(|found| (found, true)).collect();
    let mut directories = vec![directory.to_owned()];
    if !paths::is_any(directory) {
        let shared = paths::any_directory(directory);
        for found in files::in_directory(db, repository, PACKAGE, &shared).await? {
            packages.push((found, false));
        }
        directories.push(shared);
    }
    let mut signatures = HashMap::new();
    for directory in directories {
        for found in files::in_directory(db, repository, SIGNATURE, &directory).await? {
            signatures.insert(found.path.clone(), found);
        }
    }
    Ok(Inputs {
        packages,
        signatures,
    })
}

fn fingerprint(inputs: &Inputs) -> String {
    let mut digest = Sha256::new();
    digest.update(GENERATION.as_bytes());
    let mut lines: Vec<String> = inputs
        .packages
        .iter()
        .map(|(found, _)| format!("package {}\t{}", found.path, found.sha256))
        .chain(
            inputs
                .signatures
                .values()
                .map(|found| format!("signature {}\t{}", found.path, found.sha256)),
        )
        .collect();
    lines.sort();
    for line in lines {
        digest.update(b"\n");
        digest.update(line.as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_fingerprint(found: &Asset) -> Option<&str> {
    files::attribute(found, "fingerprint").and_then(Value::as_str)
}

fn name_and_version(found: &Asset) -> Option<(String, String)> {
    let name = files::attribute(found, "name")?.as_str()?.to_owned();
    let version = files::attribute(found, "version")?.as_str()?.to_owned();
    Some((name, version))
}

fn choose(packages: &[(Asset, bool)]) -> Vec<&Asset> {
    let mut best: BTreeMap<String, (&Asset, bool, String)> = BTreeMap::new();
    for (found, own) in packages {
        let Some((name, version)) = name_and_version(found) else {
            tracing::warn!(path = %found.path, "a package without its .PKGINFO attributes is left out of the database");
            continue;
        };
        let better = match best.get(&name) {
            None => true,
            Some((current, current_own, current_version)) => {
                version::compare(&version, current_version)
                    .then_with(|| own.cmp(current_own))
                    .then_with(|| found.blob_updated_at.cmp(&current.blob_updated_at))
                    == Ordering::Greater
            }
        };
        if better {
            best.insert(name, (found, *own, version));
        }
    }
    best.into_values().map(|(found, _, _)| found).collect()
}

async fn signature_text(ctx: &Ctx, found: Option<&Asset>) -> Option<String> {
    let found = found?;
    let bytes = match files::bytes(ctx, found, SIGNATURE_LIMIT).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(repository = ctx.name(), path = %found.path, %error, "a package signature cannot be read");
            return None;
        }
    };
    let binary = signing::binary_signature(&bytes)?;
    Some(base64::engine::general_purpose::STANDARD.encode(binary))
}

async fn previous_files(ctx: &Ctx, stored: &[Option<Asset>]) -> HashMap<String, Vec<u8>> {
    let mut known = HashMap::new();
    for found in stored.iter().flatten() {
        let bytes = match files::bytes(ctx, found, files::MAX_DATABASE).await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::info!(repository = ctx.name(), path = %found.path, %error, "a stored files database cannot be reused");
                continue;
            }
        };
        let read = tokio::task::spawn_blocking(move || db::read(&bytes)).await;
        let Ok(Ok(records)) = read else {
            continue;
        };
        for record in records {
            if let (Some(sha256), Some(list)) =
                (record.field("SHA256SUM"), record.member(db::FILES))
            {
                known.insert(sha256, list.to_vec());
            }
        }
    }
    known
}

async fn files_of(
    ctx: &Ctx,
    found: &Asset,
    known: &Known,
    previous: &HashMap<String, Vec<u8>>,
) -> Vec<u8> {
    if let Some(list) = known.get(&found.sha256) {
        return db::files_text(list).into_bytes();
    }
    if let Some(list) = previous.get(&found.sha256) {
        return list.clone();
    }
    match files::read_asset_package(ctx, found).await {
        Ok(contents) => db::files_text(&contents.files).into_bytes(),
        Err(error) => {
            tracing::warn!(repository = ctx.name(), path = %found.path, %error, "the file list of a package cannot be read");
            db::files_text(&[]).into_bytes()
        }
    }
}

async fn existing(
    db: &impl ConnectionTrait,
    ctx: &Ctx,
    directory: &str,
) -> Result<(Option<Asset>, Option<Asset>), ContentError> {
    Ok((
        store::find(
            db,
            ctx.name(),
            &paths::stored_path(directory, ctx.name(), Variant::Db),
        )
        .await?,
        store::find(
            db,
            ctx.name(),
            &paths::stored_path(directory, ctx.name(), Variant::Files),
        )
        .await?,
    ))
}

fn fresh(stored: &(Option<Asset>, Option<Asset>), wanted: &str) -> Option<Generated> {
    match stored {
        (Some(db), Some(files))
            if stored_fingerprint(db) == Some(wanted)
                && stored_fingerprint(files) == Some(wanted) =>
        {
            Some(Generated {
                db: db.clone(),
                files: files.clone(),
            })
        }
        _ => None,
    }
}

pub async fn rebuild(
    ctx: &Ctx,
    directory: &str,
    force: bool,
    known: &Known,
) -> Result<Option<Generated>, ContentError> {
    let txn = files::lock(ctx, directory).await?;
    let inputs = inputs(&txn, ctx.name(), directory).await?;
    let stored = existing(&txn, ctx, directory).await?;
    if inputs.packages.is_empty() {
        for found in [stored.0, stored.1].into_iter().flatten() {
            store::delete_asset(&ctx.content, found).await?;
        }
        txn.commit().await?;
        return Ok(None);
    }
    let wanted = fingerprint(&inputs);
    if !force && let Some(generated) = fresh(&stored, &wanted) {
        txn.commit().await?;
        return Ok(Some(generated));
    }
    let previous = match force {
        true => HashMap::new(),
        false => {
            let mut reusable = vec![stored.1.clone()];
            if !paths::is_any(directory) {
                reusable.push(
                    existing(&txn, ctx, &paths::any_directory(directory))
                        .await?
                        .1,
                );
            }
            previous_files(ctx, &reusable).await
        }
    };
    let mut records = Vec::new();
    for found in choose(&inputs.packages) {
        let Some(mut entry) = found.attributes.get(ID).and_then(Entry::from_attributes) else {
            continue;
        };
        entry.file_name = paths::file_name(&found.path).to_owned();
        entry.compressed_size = found.size.max(0) as u64;
        entry.sha256 = found.sha256.clone();
        entry.signature = signature_text(
            ctx,
            inputs
                .signatures
                .get(&format!("{}{SIGNATURE_SUFFIX}", found.path)),
        )
        .await;
        let list = files_of(ctx, found, known, &previous).await;
        records.push(Record {
            directory: entry.directory(),
            name: entry.name.clone(),
            members: vec![
                (db::DESC.to_owned(), entry.desc().into_bytes()),
                (db::FILES.to_owned(), list),
            ],
        });
    }
    let mtime = chrono::Utc::now().timestamp().max(0) as u64;
    let count = records.len();
    let (db_file, files_file) = tokio::task::spawn_blocking(move || {
        Ok::<_, std::io::Error>((
            db::archive(&records, Variant::Db, mtime)?,
            db::archive(&records, Variant::Files, mtime)?,
        ))
    })
    .await
    .map_err(ContentError::internal)?
    .map_err(ContentError::internal)?;
    let attributes = json!({ ID: {
        "fingerprint": wanted,
        "packageCount": count,
    }});
    let mut generated = Vec::with_capacity(2);
    for (variant, file) in [(Variant::Db, db_file), (Variant::Files, files_file)] {
        let path = paths::stored_path(directory, ctx.name(), variant);
        generated.push(
            store::put(
                ctx,
                files::metadata_put(ctx, &path, DATABASE, DATABASE_TYPE, attributes.clone()),
                astrum_blob::content_of(file),
            )
            .await?,
        );
    }
    txn.commit().await?;
    let files = generated.pop().expect("two databases were written");
    let db = generated.pop().expect("two databases were written");
    Ok(Some(Generated { db, files }))
}

pub async fn refresh(
    ctx: &Ctx,
    directories: impl IntoIterator<Item = String>,
    known: &Known,
) -> Result<(), ContentError> {
    let mut wanted = BTreeSet::new();
    for directory in directories {
        if paths::is_any(&directory) {
            let parent = paths::directory(&directory);
            for found in files::under(ctx.db(), ctx.name(), PACKAGE, parent).await? {
                let own = paths::directory(&found.path);
                if paths::directory(own) == parent {
                    wanted.insert(own.to_owned());
                }
            }
        }
        wanted.insert(directory);
    }
    for directory in wanted {
        rebuild(ctx, &directory, false, known).await?;
    }
    Ok(())
}

pub async fn ensure(ctx: &Ctx, directory: &str) -> Result<Option<Generated>, ContentError> {
    let mut source = directory.to_owned();
    if !paths::is_any(directory)
        && files::in_directory(ctx.db(), ctx.name(), PACKAGE, directory)
            .await?
            .is_empty()
    {
        let stale = existing(ctx.db(), ctx, directory).await?;
        if stale.0.is_some() || stale.1.is_some() {
            rebuild(ctx, directory, false, &Known::new()).await?;
        }
        if directory.is_empty() {
            return Ok(None);
        }
        source = paths::any_directory(directory);
    }
    let inputs = inputs(ctx.db(), ctx.name(), &source).await?;
    let stored = existing(ctx.db(), ctx, &source).await?;
    if inputs.packages.is_empty() {
        if stored.0.is_some() || stored.1.is_some() {
            rebuild(ctx, &source, false, &Known::new()).await?;
        }
        return Ok(None);
    }
    match fresh(&stored, &fingerprint(&inputs)) {
        Some(generated) => Ok(Some(generated)),
        None => rebuild(ctx, &source, false, &Known::new()).await,
    }
}

pub async fn delete(ctx: &Ctx, found: Asset) -> Result<(), ContentError> {
    let directory = paths::directory(&found.path).to_owned();
    if found.kind == PACKAGE {
        let signature = format!("{}{SIGNATURE_SUFFIX}", found.path);
        if let Some(signed) = store::find(ctx.db(), ctx.name(), &signature).await? {
            store::delete_asset(&ctx.content, signed).await?;
        }
    }
    let rebuilds = matches!(found.kind.as_str(), PACKAGE | SIGNATURE);
    store::delete_asset(&ctx.content, found).await?;
    if rebuilds {
        refresh(ctx, [directory], &Known::new()).await?;
    }
    Ok(())
}

pub async fn rebuild_all(ctx: &Ctx) -> Result<u64, ContentError> {
    let mut directories = BTreeSet::new();
    for kind in [PACKAGE, DATABASE] {
        for found in files::of_kind(ctx.db(), ctx.name(), kind).await? {
            directories.insert(paths::directory(&found.path).to_owned());
        }
    }
    let mut rebuilt = 0;
    for directory in directories {
        if rebuild(ctx, &directory, true, &Known::new())
            .await?
            .is_some()
        {
            rebuilt += 1;
        }
    }
    Ok(rebuilt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package;
    use crate::testing::{self, Spec};

    fn contents(arch: &str) -> Contents {
        package::read(
            testing::build(&Spec::new("hello", "1.0-1", arch)).as_slice(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn packages_go_into_the_directory_of_their_architecture() {
        let x86 = contents("x86_64");
        assert!(check_path(&x86, "/x86_64/hello-1.0-1-x86_64.pkg.tar.zst").is_ok());
        assert!(check_path(&x86, "/custom/os/x86_64/hello-1.0-1-x86_64.pkg.tar.zst").is_ok());
        assert!(check_path(&x86, "/x86_64/hello-1.0-1-x86_64.pkg.tar.xz").is_ok());
        assert!(check_path(&x86, "/aarch64/hello-1.0-1-x86_64.pkg.tar.zst").is_err());
        assert!(check_path(&x86, "/x86_64/hello-1.0-2-x86_64.pkg.tar.zst").is_err());
        assert!(check_path(&x86, "/hello-1.0-1-x86_64.pkg.tar.zst").is_err());
        assert_eq!(
            target_path(&x86, "").unwrap(),
            "/x86_64/hello-1.0-1-x86_64.pkg.tar.zst"
        );
        assert_eq!(
            target_path(&x86, "custom/os/").unwrap(),
            "/custom/os/x86_64/hello-1.0-1-x86_64.pkg.tar.zst"
        );
        assert!(target_path(&x86, "../x86_64").is_err());
        let any = contents("any");
        assert_eq!(
            target_path(&any, "").unwrap(),
            "/any/hello-1.0-1-any.pkg.tar.zst"
        );
        assert!(check_path(&any, "/x86_64/hello-1.0-1-any.pkg.tar.zst").is_ok());
        assert!(check_path(&any, "/hello-1.0-1-any.pkg.tar.zst").is_err());
        assert!(matches!(
            writable("/x86_64/core.db"),
            Err(ContentError::BadRequest(_))
        ));
        assert!(matches!(
            writable("/x86_64/README"),
            Err(ContentError::BadRequest(_))
        ));
        assert!(matches!(
            writable("/x86_64/hello-1.0-1-x86_64.pkg.tar.zst.sig"),
            Ok(Upload::Signature(_))
        ));
    }
}
