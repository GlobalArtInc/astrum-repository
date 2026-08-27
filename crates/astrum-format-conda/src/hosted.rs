use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::BufReader;

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View, zip};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use futures_util::TryStreamExt;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::package::{self, Info, PackageError};
use crate::paths::{self, FileName, IndexFile, PackageType};
use crate::repodata::{self, Described, Repodata};
use crate::{ID, METADATA, PACKAGE};

const GENERATION: &str = "1\n";
const TAR_PREFIX: u64 = 16 * 1024 * 1024;

pub enum Target<'a> {
    Path {
        channel: &'a str,
        subdir: &'a str,
        file: &'a FileName,
    },
    Channel {
        channel: &'a str,
        file_name: Option<&'a str>,
    },
}

fn like_prefix(channel: &str) -> String {
    let prefix = if channel.is_empty() {
        "/".to_owned()
    } else {
        format!("/{channel}/")
    };
    format!(
        "{}%",
        prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

async fn of_kind(
    db: &impl ConnectionTrait,
    repository: &str,
    kind: &str,
    channel: &str,
) -> Result<Vec<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(kind))
        .filter(Expr::col(asset::Column::Path).like(like_prefix(channel)))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?)
}

/// The packages stored directly in the subdirectories of `channel`, by subdirectory.
pub async fn packages(
    db: &impl ConnectionTrait,
    repository: &str,
    channel: &str,
) -> Result<BTreeMap<String, Vec<Asset>>, ContentError> {
    let mut found: BTreeMap<String, Vec<Asset>> = BTreeMap::new();
    for stored in of_kind(db, repository, PACKAGE, channel).await? {
        if let Some((owner, subdir)) = paths::place_of(&stored.path)
            && owner == channel
        {
            found.entry(subdir).or_default().push(stored);
        }
    }
    Ok(found)
}

fn conda_attribute<'a>(found: &'a Asset, key: &str) -> Option<&'a Value> {
    found.attributes.get(ID)?.get(key)
}

fn text_attribute<'a>(found: &'a Asset, key: &str) -> Option<&'a str> {
    conda_attribute(found, key)?.as_str()
}

async fn metadata_of(
    db: &impl ConnectionTrait,
    repository: &str,
    channel: &str,
) -> Result<Vec<Asset>, ContentError> {
    Ok(of_kind(db, repository, METADATA, channel)
        .await?
        .into_iter()
        .filter(|found| text_attribute(found, "channel") == Some(channel))
        .collect())
}

fn fingerprint<'a>(packages: impl IntoIterator<Item = &'a Asset>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(GENERATION.as_bytes());
    for found in packages {
        hasher.update(found.path.as_bytes());
        hasher.update(b" ");
        hasher.update(found.sha256.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

enum Write {
    Subdir(String, Vec<Asset>, String),
    Channeldata(Vec<Asset>, String),
}

struct Plan {
    writes: Vec<Write>,
    stale: Vec<Asset>,
}

async fn plan(
    db: &impl ConnectionTrait,
    repository: &str,
    channel: &str,
    force: bool,
) -> Result<Plan, ContentError> {
    let subdirs = packages(db, repository, channel).await?;
    let metadata = metadata_of(db, repository, channel).await?;
    let stored = |path: &str| {
        metadata
            .iter()
            .find(|found| found.path == path)
            .and_then(|found| text_attribute(found, "fingerprint"))
    };
    let mut writes = Vec::new();
    for (subdir, list) in &subdirs {
        let wanted = fingerprint(list);
        let complete = IndexFile::ALL.iter().all(|file| {
            stored(&paths::index_path(channel, subdir, *file)) == Some(wanted.as_str())
        });
        if force || !complete {
            writes.push(Write::Subdir(subdir.clone(), list.clone(), wanted));
        }
    }
    let all: Vec<Asset> = subdirs.values().flatten().cloned().collect();
    if !all.is_empty() {
        let wanted = fingerprint(&all);
        if force || stored(&paths::channeldata_path(channel)) != Some(wanted.as_str()) {
            writes.push(Write::Channeldata(all, wanted));
        }
    }
    let stale = metadata
        .iter()
        .filter(|found| match text_attribute(found, "subdir") {
            Some(subdir) => !subdirs.contains_key(subdir),
            None => subdirs.is_empty(),
        })
        .cloned()
        .collect();
    Ok(Plan { writes, stale })
}

fn record_of(found: &Asset, subdir: &str) -> Option<(PackageType, String, Vec<u8>)> {
    let index = conda_attribute(found, "index")?.as_object()?;
    let file = paths::file_of(&found.path).to_owned();
    let kind = PackageType::of(&file)?;
    Some((
        kind,
        file,
        repodata::record(index, subdir, found.size, &found.md5, &found.sha256),
    ))
}

fn described(found: &Asset, subdir: &str) -> Option<Described> {
    let index = conda_attribute(found, "index")?.as_object()?.clone();
    Some(Described {
        name: index.get("name")?.as_str()?.to_owned(),
        version: index.get("version")?.as_str()?.to_owned(),
        subdir: subdir.to_owned(),
        timestamp: index.get("timestamp").and_then(Value::as_i64),
        about: conda_attribute(found, "about")
            .and_then(Value::as_object)
            .cloned(),
        run_exports: conda_attribute(found, "runExports").cloned(),
        index,
    })
}

fn metadata_put(ctx: &Ctx, path: String, content_type: &str, attributes: Value) -> PutAsset {
    let mut put = PutAsset::hosted(ctx, path)
        .kind(METADATA)
        .content_type(content_type)
        .attributes(attributes);
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::Allow);
    put
}

async fn index_files(
    subdir: String,
    packages: Vec<Asset>,
) -> Result<Vec<(IndexFile, Vec<u8>)>, ContentError> {
    tokio::task::spawn_blocking(move || {
        let mut document = Repodata::empty(&subdir);
        for found in &packages {
            if let Some((kind, file, record)) = record_of(found, &subdir) {
                document.insert(kind, file, Cow::Owned(record));
            }
        }
        let json = document.to_bytes();
        let current = document.current().to_bytes();
        vec![
            (IndexFile::Bz2, repodata::bz2(&json)),
            (IndexFile::Zst, repodata::zst(&json)),
            (IndexFile::Current, current),
            (IndexFile::Json, json),
        ]
    })
    .await
    .map_err(ContentError::internal)
}

async fn execute(ctx: &Ctx, channel: &str, plan: Plan) -> Result<(), ContentError> {
    for write in plan.writes {
        match write {
            Write::Subdir(subdir, packages, wanted) => {
                let attributes =
                    json!({ ID: {"channel": channel, "subdir": subdir, "fingerprint": wanted} });
                for (file, bytes) in index_files(subdir.clone(), packages).await? {
                    store::put(
                        ctx,
                        metadata_put(
                            ctx,
                            paths::index_path(channel, &subdir, file),
                            file.content_type(),
                            attributes.clone(),
                        ),
                        astrum_blob::content_of(bytes),
                    )
                    .await?;
                }
            }
            Write::Channeldata(packages, wanted) => {
                let described: Vec<Described> = packages
                    .iter()
                    .filter_map(|found| {
                        let (_, subdir) = paths::place_of(&found.path)?;
                        described(found, &subdir)
                    })
                    .collect();
                store::put(
                    ctx,
                    metadata_put(
                        ctx,
                        paths::channeldata_path(channel),
                        "application/json",
                        json!({ ID: {"channel": channel, "fingerprint": wanted} }),
                    ),
                    astrum_blob::content_of(repodata::channeldata(&described)),
                )
                .await?;
            }
        }
    }
    for found in plan.stale {
        store::delete_asset(&ctx.content, found).await?;
    }
    Ok(())
}

/// Writes the indexes of `channel` again where its packages changed since they were written:
/// `repodata.json` with its compressed variants and `current_repodata.json` per subdirectory, and
/// `channeldata.json`. Indexes of subdirectories without packages are removed.
pub async fn refresh(ctx: &Ctx, channel: &str, force: bool) -> Result<(), ContentError> {
    let quick = plan(ctx.db(), ctx.name(), channel, force).await?;
    if quick.writes.is_empty() && quick.stale.is_empty() {
        return Ok(());
    }
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("conda:{}:{channel}", ctx.name()).into()],
    ))
    .await?;
    let locked = plan(&txn, ctx.name(), channel, force).await?;
    let done = execute(ctx, channel, locked).await;
    txn.commit().await?;
    done
}

/// Every channel of the repository that holds packages or indexes.
pub async fn channels(ctx: &Ctx) -> Result<Vec<String>, ContentError> {
    let mut found = std::collections::BTreeSet::new();
    for stored in of_kind(ctx.db(), ctx.name(), PACKAGE, "").await? {
        if let Some((channel, _)) = paths::place_of(&stored.path) {
            found.insert(channel);
        }
    }
    for stored in of_kind(ctx.db(), ctx.name(), METADATA, "").await? {
        if let Some(channel) = text_attribute(&stored, "channel") {
            found.insert(channel.to_owned());
        }
    }
    Ok(found.into_iter().collect())
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

fn bad(error: PackageError) -> ContentError {
    ContentError::BadRequest(format!("Not a valid conda package: {error}"))
}

async fn read_range(ctx: &Ctx, written: &Written, length: u64) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let Some(content) = store.read_range(written.id, 0, length).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob that was just written is missing",
        )));
    };
    let mut body = Vec::with_capacity(length as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn read_whole_tar_bz2(ctx: &Ctx, written: &Written) -> Result<Info, ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let Some(content) = store.read(written.id).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob that was just written is missing",
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
    tokio::task::spawn_blocking(move || {
        std::io::Seek::rewind(&mut file)?;
        package::read_tar_bz2(BufReader::new(file))
    })
    .await
    .map_err(ContentError::internal)?
    .map_err(bad)
}

pub async fn read_info(
    ctx: &Ctx,
    written: &Written,
    kind: PackageType,
) -> Result<Info, ContentError> {
    match kind {
        PackageType::Conda => {
            let source = zip::Blob {
                store: ctx.content.blob_stores().get(&written.store).await?,
                id: written.id,
                size: written.size,
            };
            package::read_conda(&source).await.map_err(bad)
        }
        PackageType::TarBz2 => {
            let prefix = read_range(ctx, written, written.size.min(TAR_PREFIX)).await?;
            let whole = prefix.len() as u64 >= written.size;
            let first =
                tokio::task::spawn_blocking(move || package::read_tar_bz2(prefix.as_slice()))
                    .await
                    .map_err(ContentError::internal)?;
            match first {
                Ok(info) => Ok(info),
                Err(error) if whole => Err(bad(error)),
                Err(_) => read_whole_tar_bz2(ctx, written).await,
            }
        }
    }
}

fn refuse(ctx: &Ctx, written: &Written, error: ContentError) -> ContentError {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    error
}

pub async fn adopt(ctx: &Ctx, written: Written, target: Target<'_>) -> Result<Asset, ContentError> {
    match place(ctx, &written, target).await {
        Ok((channel, put)) => {
            let stored = store::adopt(ctx, put, &written).await?;
            refresh(ctx, &channel, false).await?;
            Ok(stored)
        }
        Err(error) => Err(refuse(ctx, &written, error)),
    }
}

async fn place(
    ctx: &Ctx,
    written: &Written,
    target: Target<'_>,
) -> Result<(String, PutAsset), ContentError> {
    let kind = match &target {
        Target::Path { file, .. } => file.kind,
        Target::Channel { file_name, .. } => {
            let prefix = read_range(ctx, written, written.size.min(8)).await?;
            package::kind_of(*file_name, &prefix).ok_or_else(|| {
                ContentError::BadRequest(
                    "Not a valid conda package: use a .conda or .tar.bz2 file".to_owned(),
                )
            })?
        }
    };
    let info = read_info(ctx, written, kind).await?;
    let stem = FileName::stem(info.name(), info.version(), info.build());
    let (channel, subdir) = match &target {
        Target::Path {
            channel,
            subdir,
            file,
        } => {
            if let Some(built) = info.subdir()
                && built != *subdir
            {
                return Err(ContentError::BadRequest(format!(
                    "the package is built for {built}, not for {subdir}: upload it to {built}/"
                )));
            }
            let named = FileName::stem(&file.name, &file.version, &file.build);
            if named != stem {
                return Err(ContentError::BadRequest(format!(
                    "the file name {named}{} does not match the package {stem} of info/index.json",
                    kind.extension()
                )));
            }
            ((*channel).to_owned(), (*subdir).to_owned())
        }
        Target::Channel { channel, .. } => {
            let subdir = info.subdir().ok_or_else(|| {
                ContentError::BadRequest(
                    "the package does not name its subdir: upload it to <channel>/<subdir>/<file>"
                        .to_owned(),
                )
            })?;
            ((*channel).to_owned(), subdir)
        }
    };
    if !paths::valid_subdir(&subdir) {
        return Err(ContentError::BadRequest(format!(
            "{subdir} is not a conda subdir such as noarch or linux-64"
        )));
    }
    let path = paths::package_path(&channel, &subdir, &format!("{stem}{}", kind.extension()));
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    if exists && write_policy(ctx) == WritePolicy::Allow {
        ctx.require(View::Edit, &path)?;
    } else {
        ctx.require(View::Add, &path)?;
    }
    let mut attributes = Map::new();
    attributes.insert("channel".to_owned(), json!(channel));
    attributes.insert("subdir".to_owned(), json!(subdir));
    attributes.insert("build".to_owned(), json!(info.build()));
    attributes.insert("buildNumber".to_owned(), json!(info.build_number()));
    attributes.insert("packageType".to_owned(), json!(kind.id()));
    if let Some(license) = info.index.get("license").and_then(Value::as_str) {
        attributes.insert("license".to_owned(), json!(license));
    }
    if let Some(about) = &info.about {
        attributes.insert("about".to_owned(), Value::Object(about.clone()));
    }
    if let Some(run_exports) = &info.run_exports {
        attributes.insert("runExports".to_owned(), run_exports.clone());
    }
    attributes.insert("index".to_owned(), Value::Object(info.index.clone()));
    let put = PutAsset::hosted(ctx, path)
        .component(ComponentKey {
            namespace: channel.clone(),
            name: info.name().to_owned(),
            version: info.version().to_owned(),
            kind: String::new(),
            attributes: json!({}),
        })
        .kind(PACKAGE)
        .content_type(kind.content_type())
        .attributes(json!({ ID: attributes }));
    Ok((channel, put))
}
