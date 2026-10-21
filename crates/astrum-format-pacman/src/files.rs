use astrum_blob::BlobId;
use astrum_content::store::PutAsset;
use astrum_content::{Asset, ContentError, Ctx};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use futures_util::TryStreamExt;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    Statement, TransactionTrait,
};
use serde_json::Value;

use crate::package::{self, Contents};
use crate::paths;

const FIRST_READ: u64 = 256 * 1024;
const MAX_HEAD: u64 = 64 * 1024 * 1024;
pub const MAX_DATABASE: u64 = 512 * 1024 * 1024;

pub fn invalid_package(error: impl std::fmt::Display) -> ContentError {
    ContentError::BadRequest(format!("Not a valid pacman package: {error}"))
}

pub async fn range(
    ctx: &Ctx,
    store: &str,
    id: BlobId,
    offset: u64,
    length: u64,
) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(store).await?;
    let Some(content) = store.read_range(id, offset, length).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a stored blob is missing",
        )));
    };
    let mut body = Vec::with_capacity(content.length as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn download(ctx: &Ctx, store: &str, id: BlobId) -> Result<std::fs::File, ContentError> {
    let store = ctx.content.blob_stores().get(store).await?;
    let Some(content) = store.read(id).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a stored blob is missing",
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
    tokio::io::AsyncWriteExt::flush(&mut target)
        .await
        .map_err(ContentError::internal)?;
    let mut file = target.into_std().await;
    std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(0)).map_err(ContentError::internal)?;
    Ok(file)
}

pub async fn read_package(ctx: &Ctx, store: &str, id: BlobId) -> Result<Contents, ContentError> {
    let file = download(ctx, store, id).await?;
    tokio::task::spawn_blocking(move || package::read(std::io::BufReader::new(file), true))
        .await
        .map_err(ContentError::internal)?
        .map_err(invalid_package)
}

pub async fn read_asset_package(ctx: &Ctx, found: &Asset) -> Result<Contents, ContentError> {
    read_package(ctx, &found.blob_store, found.blob_id.into()).await
}

pub async fn read_head(ctx: &Ctx, found: &Asset) -> Result<Contents, ContentError> {
    let size = found.size.max(0) as u64;
    let id: BlobId = found.blob_id.into();
    let mut wanted = FIRST_READ.min(size);
    let mut prefix = range(ctx, &found.blob_store, id, 0, wanted).await?;
    loop {
        let data = std::mem::take(&mut prefix);
        let (data, parsed) = tokio::task::spawn_blocking(move || {
            let parsed = package::read(data.as_slice(), false);
            (data, parsed)
        })
        .await
        .map_err(ContentError::internal)?;
        prefix = data;
        match parsed {
            Ok(contents) => return Ok(contents),
            Err(_) if wanted < size.min(MAX_HEAD) => {
                let next = (wanted * 4).min(size).min(MAX_HEAD);
                let more = range(ctx, &found.blob_store, id, wanted, next - wanted).await?;
                if more.is_empty() {
                    return Err(invalid_package("the file ends early"));
                }
                prefix.extend_from_slice(&more);
                wanted = next;
            }
            Err(error) => return Err(invalid_package(error)),
        }
    }
}

pub async fn bytes(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
        return Err(ContentError::BadGateway(format!(
            "{} is too large to read",
            found.path
        )));
    }
    range(
        ctx,
        &found.blob_store,
        found.blob_id.into(),
        0,
        found.size.max(0) as u64,
    )
    .await
}

pub fn metadata_put(
    ctx: &Ctx,
    path: &str,
    kind: &str,
    content_type: &str,
    attributes: Value,
) -> PutAsset {
    let mut put = PutAsset::hosted(ctx, path)
        .kind(kind)
        .content_type(content_type)
        .attributes(attributes);
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::Allow);
    put
}

pub async fn of_kind(
    db: &impl ConnectionTrait,
    repository: &str,
    kind: &str,
) -> Result<Vec<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(kind))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?)
}

fn escaped(prefix: &str) -> String {
    prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub async fn under(
    db: &impl ConnectionTrait,
    repository: &str,
    kind: &str,
    directory: &str,
) -> Result<Vec<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(kind))
        .filter(Expr::col(asset::Column::Path).like(format!("{}/%", escaped(directory))))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?)
}

pub async fn in_directory(
    db: &impl ConnectionTrait,
    repository: &str,
    kind: &str,
    directory: &str,
) -> Result<Vec<Asset>, ContentError> {
    Ok(under(db, repository, kind, directory)
        .await?
        .into_iter()
        .filter(|found| paths::directory(&found.path) == directory)
        .collect())
}

pub async fn lock(ctx: &Ctx, directory: &str) -> Result<DatabaseTransaction, ContentError> {
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("pacman:{}:{directory}", ctx.name()).into()],
    ))
    .await?;
    Ok(txn)
}

pub fn attribute<'a>(found: &'a Asset, key: &str) -> Option<&'a Value> {
    found
        .attributes
        .get(crate::ID)
        .and_then(|pacman| pacman.get(key))
}
