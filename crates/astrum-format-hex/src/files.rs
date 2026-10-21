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

use crate::{ID, TARBALL, tarball};

pub async fn bytes(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    let size = found.size.max(0) as u64;
    if size > limit {
        return Err(ContentError::BadGateway(format!(
            "{} is too large to read",
            found.path
        )));
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a stored blob is missing",
        )));
    };
    let mut body = Vec::with_capacity(size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
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

pub async fn tarballs_of(
    db: &impl ConnectionTrait,
    repository: &str,
    name: &str,
) -> Result<Vec<Asset>, ContentError> {
    let prefix = format!("/tarballs/{name}-")
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(TARBALL))
        .filter(Expr::col(asset::Column::Path).like(format!("{prefix}%")))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?
        .into_iter()
        .filter(|found| {
            found
                .path
                .strip_prefix("/tarballs/")
                .and_then(tarball::split_file)
                .is_some_and(|(found_name, _)| found_name == name)
        })
        .collect())
}

pub async fn lock(ctx: &Ctx) -> Result<DatabaseTransaction, ContentError> {
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("hex:{}", ctx.name()).into()],
    ))
    .await?;
    Ok(txn)
}

pub fn attribute<'a>(found: &'a Asset, key: &str) -> Option<&'a Value> {
    found.attributes.get(ID).and_then(|hex| hex.get(key))
}
