use std::io::{Seek, SeekFrom};

use astrum_blob::{BlobId, ContentStream};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    Statement, TransactionTrait,
};
use serde_json::Value;

use crate::rpm::{self, Package};

const FIRST_READ: u64 = 256 * 1024;
const MAX_SMALL: u64 = 16 * 1024 * 1024;

pub fn invalid_rpm(error: impl std::fmt::Display) -> ContentError {
    ContentError::BadRequest(format!("Not a valid RPM package: {error}"))
}

async fn range(
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

pub async fn read_package(
    ctx: &Ctx,
    store: &str,
    id: BlobId,
    size: u64,
) -> Result<Package, ContentError> {
    let mut prefix = range(ctx, store, id, 0, FIRST_READ.min(size)).await?;
    loop {
        match rpm::needed(&prefix).map_err(invalid_rpm)? {
            None => break,
            Some(wanted) if wanted as u64 > size || (prefix.len() as u64) >= size => {
                return Err(invalid_rpm(rpm::RpmError::Truncated));
            }
            Some(wanted) => {
                let offset = prefix.len() as u64;
                let more = range(ctx, store, id, offset, wanted as u64 - offset).await?;
                if more.is_empty() {
                    return Err(invalid_rpm(rpm::RpmError::Truncated));
                }
                prefix.extend_from_slice(&more);
            }
        }
    }
    tokio::task::spawn_blocking(move || rpm::parse(&prefix))
        .await
        .map_err(ContentError::internal)?
        .map_err(invalid_rpm)
}

pub async fn read_asset_package(ctx: &Ctx, found: &Asset) -> Result<Package, ContentError> {
    read_package(
        ctx,
        &found.blob_store,
        found.blob_id.into(),
        found.size.max(0) as u64,
    )
    .await
}

pub async fn bytes(ctx: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > MAX_SMALL {
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

pub async fn download(ctx: &Ctx, found: &Asset) -> Result<std::fs::File, ContentError> {
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
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
    Ok(target.into_std().await)
}

pub fn stream_file(mut file: std::fs::File) -> Result<ContentStream, ContentError> {
    file.seek(SeekFrom::Start(0))
        .map_err(ContentError::internal)?;
    Ok(tokio_util::io::ReaderStream::new(tokio::fs::File::from_std(file)).boxed())
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

pub async fn of_kind_under(
    db: &impl ConnectionTrait,
    repository: &str,
    kind: &str,
    directory: &str,
) -> Result<Vec<Asset>, ContentError> {
    let prefix = format!("{}/", directory.trim_end_matches('/'));
    let pattern = format!(
        "{}%",
        prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(kind))
        .filter(Expr::col(asset::Column::Path).like(pattern))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?
        .into_iter()
        .filter(|found| found.path.starts_with(&prefix))
        .collect())
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

pub async fn mark_stale(db: &impl ConnectionTrait, id: i64) -> Result<(), ContentError> {
    asset::Entity::update_many()
        .col_expr(
            asset::Column::LastVerifiedAt,
            Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None),
        )
        .filter(asset::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn lock(ctx: &Ctx, name: &str) -> Result<DatabaseTransaction, ContentError> {
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("yum:{}:{name}", ctx.name()).into()],
    ))
    .await?;
    Ok(txn)
}

pub async fn store_generated(
    ctx: &Ctx,
    path: &str,
    kind: &str,
    content_type: &str,
    file: std::fs::File,
    attributes: Value,
) -> Result<Asset, ContentError> {
    store::put(
        ctx,
        metadata_put(ctx, path, kind, content_type, attributes),
        stream_file(file)?,
    )
    .await
}
