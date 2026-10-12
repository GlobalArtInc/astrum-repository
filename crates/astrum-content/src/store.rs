use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll};

use astrum_blob::{ContentStream, Written};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use bytes::Bytes;
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::Stream;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
};
use serde_json::Value;
use uuid::Uuid;

use crate::mime;
use crate::{Content, ContentError, Ctx};

pub type Asset = asset::Model;
pub type Component = component::Model;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComponentKey {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    pub attributes: Value,
}

#[derive(Clone, Debug, Default)]
pub struct PutAsset {
    pub path: String,
    pub kind: String,
    pub content_type: Option<String>,
    pub component: Option<ComponentKey>,
    pub attributes: Value,
    pub check_content_type: bool,
    pub write_policy: Option<WritePolicy>,
    pub remote_etag: Option<String>,
    pub remote_last_modified: Option<DateTime<FixedOffset>>,
    pub verified: bool,
}

impl PutAsset {
    pub fn hosted(ctx: &Ctx, path: impl Into<String>) -> Self {
        let storage = ctx.repository.attributes.storage.as_ref();
        Self {
            path: path.into(),
            check_content_type: storage
                .and_then(|storage| storage.strict_content_type_validation)
                .unwrap_or(true),
            write_policy: Some(
                storage
                    .and_then(|storage| storage.write_policy)
                    .unwrap_or(WritePolicy::Allow),
            ),
            attributes: Value::Object(Default::default()),
            ..Default::default()
        }
    }

    pub fn cached(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            verified: true,
            attributes: Value::Object(Default::default()),
            ..Default::default()
        }
    }

    pub fn component(mut self, key: ComponentKey) -> Self {
        self.component = Some(key);
        self
    }

    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = kind.into();
        self
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn attributes(mut self, attributes: Value) -> Self {
        self.attributes = attributes;
        self
    }
}

pub async fn find(
    db: &impl ConnectionTrait,
    repository: &str,
    path: &str,
) -> Result<Option<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Path.eq(path))
        .one(db)
        .await?)
}

pub async fn by_id(db: &impl ConnectionTrait, id: i64) -> Result<Option<Asset>, ContentError> {
    Ok(asset::Entity::find_by_id(id).one(db).await?)
}

pub async fn component_by_id(
    db: &impl ConnectionTrait,
    id: i64,
) -> Result<Option<Component>, ContentError> {
    Ok(component::Entity::find_by_id(id).one(db).await?)
}

pub async fn assets_of(
    db: &impl ConnectionTrait,
    component_id: i64,
) -> Result<Vec<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::ComponentId.eq(component_id))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?)
}

pub async fn under(
    db: &impl ConnectionTrait,
    repository: &str,
    prefix: &str,
    limit: u64,
) -> Result<Vec<Asset>, ContentError> {
    let escaped = prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(Expr::col(asset::Column::Path).like(format!("{escaped}%")))
        .order_by_asc(asset::Column::Path)
        .limit(limit)
        .all(db)
        .await?)
}

pub async fn count(db: &impl ConnectionTrait, repository: &str) -> Result<u64, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .count(db)
        .await?)
}

struct Sampled {
    inner: ContentStream,
    sample: Arc<Mutex<Vec<u8>>>,
}

impl Stream for Sampled {
    type Item = io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let polled = self.inner.as_mut().poll_next(cx);
        if let Poll::Ready(Some(Ok(bytes))) = &polled {
            let mut sample = self
                .sample
                .lock()
                .expect("the sample lock is never poisoned");
            let room = mime::SAMPLE.saturating_sub(sample.len());
            sample.extend_from_slice(&bytes[..room.min(bytes.len())]);
        }
        polled
    }
}

pub async fn put(ctx: &Ctx, put: PutAsset, body: ContentStream) -> Result<Asset, ContentError> {
    let repository = ctx.name().to_owned();
    if put.write_policy == Some(WritePolicy::Deny) {
        return Err(ContentError::BadRequest(format!(
            "the repository {repository} is read-only"
        )));
    }
    let existing = find(ctx.db(), &repository, &put.path).await?;
    if existing.is_some() && put.write_policy == Some(WritePolicy::AllowOnce) {
        return Err(redeploy(&repository, &put.path));
    }
    let store_name = ctx
        .repository
        .blob_store()
        .ok_or_else(|| {
            ContentError::internal(anyhow::anyhow!(
                "the repository {repository} has no blob store"
            ))
        })?
        .to_owned();
    let store = ctx.content.blob_stores().get(&store_name).await?;
    let sample = Arc::new(Mutex::new(Vec::new()));
    let written = store
        .put(Box::pin(Sampled {
            inner: body,
            sample: sample.clone(),
        }))
        .await?;
    let sample = sample
        .lock()
        .expect("the sample lock is never poisoned")
        .clone();
    let expected = mime::from_path(&put.path);
    let detected = mime::detect(&sample);
    if put.check_content_type && !mime::compatible(detected.as_deref(), expected.as_deref()) {
        discard(ctx.content.clone(), &store_name, written.id.uuid());
        return Err(ContentError::BadRequest(format!(
            "Detected content type [{}], but expected [{}]: {}",
            detected.unwrap_or_default(),
            expected.unwrap_or_default(),
            put.path
        )));
    }
    let content_type = put
        .content_type
        .clone()
        .filter(|value| !value.is_empty() && value != mime::OCTET_STREAM)
        .or(expected)
        .or(detected)
        .unwrap_or_else(|| mime::OCTET_STREAM.to_owned());
    let result = record(ctx, &put, &store_name, &written, &content_type).await;
    match result {
        Ok((asset, replaced)) => {
            if let Some((store, id)) = replaced {
                discard(ctx.content.clone(), &store, id);
            }
            Ok(asset)
        }
        Err(error) => {
            discard(ctx.content.clone(), &store_name, written.id.uuid());
            Err(error)
        }
    }
}

fn redeploy(repository: &str, path: &str) -> ContentError {
    ContentError::Conflict(format!(
        "{repository}{path} cannot be updated as asset already exists and redeploy is not allowed"
    ))
}

async fn record(
    ctx: &Ctx,
    put: &PutAsset,
    store_name: &str,
    written: &Written,
    content_type: &str,
) -> Result<(Asset, Option<(String, Uuid)>), ContentError> {
    let repository = ctx.name();
    let format = ctx.format_id();
    let now = Utc::now().fixed_offset();
    let txn = ctx.db().begin().await?;
    let component_id = match &put.component {
        Some(key) => Some(upsert_component(&txn, repository, format, key, now).await?),
        None => None,
    };
    let previous = asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Path.eq(&put.path))
        .lock_exclusive()
        .one(&txn)
        .await?;
    if previous.is_some() && put.write_policy == Some(WritePolicy::AllowOnce) {
        return Err(redeploy(repository, &put.path));
    }
    let row = asset::ActiveModel {
        id: ActiveValue::NotSet,
        repository: ActiveValue::Set(repository.to_owned()),
        format: ActiveValue::Set(format.to_owned()),
        path: ActiveValue::Set(put.path.clone()),
        kind: ActiveValue::Set(put.kind.clone()),
        component_id: ActiveValue::Set(component_id),
        blob_store: ActiveValue::Set(store_name.to_owned()),
        blob_id: ActiveValue::Set(written.id.uuid()),
        size: ActiveValue::Set(written.size as i64),
        content_type: ActiveValue::Set(content_type.to_owned()),
        sha1: ActiveValue::Set(written.digests.sha1.clone()),
        sha256: ActiveValue::Set(written.digests.sha256.clone()),
        sha512: ActiveValue::Set(written.digests.sha512.clone()),
        md5: ActiveValue::Set(written.digests.md5.clone()),
        attributes: ActiveValue::Set(put.attributes.clone()),
        uploader: ActiveValue::Set(ctx.uploader()),
        uploader_ip: ActiveValue::Set(ctx.client_ip.clone()),
        created_at: ActiveValue::Set(now),
        blob_updated_at: ActiveValue::Set(now),
        last_downloaded_at: ActiveValue::NotSet,
        download_count: ActiveValue::NotSet,
        last_verified_at: ActiveValue::Set(put.verified.then_some(now)),
        remote_etag: ActiveValue::Set(put.remote_etag.clone()),
        remote_last_modified: ActiveValue::Set(put.remote_last_modified),
    };
    let stored = asset::Entity::insert(row)
        .on_conflict(
            OnConflict::columns([asset::Column::Repository, asset::Column::Path])
                .update_columns([
                    asset::Column::Kind,
                    asset::Column::ComponentId,
                    asset::Column::BlobStore,
                    asset::Column::BlobId,
                    asset::Column::Size,
                    asset::Column::ContentType,
                    asset::Column::Sha1,
                    asset::Column::Sha256,
                    asset::Column::Sha512,
                    asset::Column::Md5,
                    asset::Column::Attributes,
                    asset::Column::Uploader,
                    asset::Column::UploaderIp,
                    asset::Column::BlobUpdatedAt,
                    asset::Column::LastVerifiedAt,
                    asset::Column::RemoteEtag,
                    asset::Column::RemoteLastModified,
                ])
                .to_owned(),
        )
        .exec_with_returning(&txn)
        .await?;
    if let Some(previous) = &previous
        && let Some(old_component) = previous.component_id
        && Some(old_component) != component_id
    {
        drop_empty_component(&txn, old_component).await?;
    }
    txn.commit().await?;
    let replaced = previous
        .filter(|previous| {
            previous.blob_id != stored.blob_id || previous.blob_store != stored.blob_store
        })
        .map(|previous| (previous.blob_store, previous.blob_id));
    Ok((stored, replaced))
}

async fn upsert_component(
    db: &impl ConnectionTrait,
    repository: &str,
    format: &str,
    key: &ComponentKey,
    now: DateTime<FixedOffset>,
) -> Result<i64, ContentError> {
    let row = component::ActiveModel {
        id: ActiveValue::NotSet,
        repository: ActiveValue::Set(repository.to_owned()),
        format: ActiveValue::Set(format.to_owned()),
        namespace: ActiveValue::Set(key.namespace.clone()),
        name: ActiveValue::Set(key.name.clone()),
        version: ActiveValue::Set(key.version.clone()),
        kind: ActiveValue::Set(key.kind.clone()),
        attributes: ActiveValue::Set(match &key.attributes {
            Value::Null => Value::Object(Default::default()),
            other => other.clone(),
        }),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    };
    let stored = component::Entity::insert(row)
        .on_conflict(
            OnConflict::columns([
                component::Column::Repository,
                component::Column::Namespace,
                component::Column::Name,
                component::Column::Version,
            ])
            .update_columns([
                component::Column::Kind,
                component::Column::Attributes,
                component::Column::UpdatedAt,
            ])
            .to_owned(),
        )
        .exec_with_returning(db)
        .await?;
    Ok(stored.id)
}

async fn drop_empty_component(
    db: &impl ConnectionTrait,
    component_id: i64,
) -> Result<(), ContentError> {
    let left = asset::Entity::find()
        .filter(asset::Column::ComponentId.eq(component_id))
        .count(db)
        .await?;
    if left == 0 {
        component::Entity::delete_by_id(component_id)
            .exec(db)
            .await?;
    }
    Ok(())
}

pub async fn touch(
    ctx: &Ctx,
    asset_id: i64,
    etag: Option<String>,
    last_modified: Option<DateTime<FixedOffset>>,
) -> Result<(), ContentError> {
    let mut row = asset::ActiveModel {
        id: ActiveValue::Unchanged(asset_id),
        last_verified_at: ActiveValue::Set(Some(Utc::now().fixed_offset())),
        ..Default::default()
    };
    if etag.is_some() {
        row.remote_etag = ActiveValue::Set(etag);
    }
    if last_modified.is_some() {
        row.remote_last_modified = ActiveValue::Set(last_modified);
    }
    asset::Entity::update(row).exec(ctx.db()).await?;
    Ok(())
}

pub async fn mark_stale(db: &impl ConnectionTrait, repository: &str) -> Result<u64, ContentError> {
    let result = asset::Entity::update_many()
        .col_expr(
            asset::Column::LastVerifiedAt,
            Expr::value(Option::<DateTime<FixedOffset>>::None),
        )
        .filter(asset::Column::Repository.eq(repository))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

pub async fn delete(ctx: &Ctx, path: &str) -> Result<bool, ContentError> {
    let Some(found) = find(ctx.db(), ctx.name(), path).await? else {
        return Ok(false);
    };
    delete_asset(&ctx.content, found).await?;
    Ok(true)
}

pub async fn delete_asset(content: &Content, found: Asset) -> Result<(), ContentError> {
    let txn = content.db().begin().await?;
    asset::Entity::delete_by_id(found.id).exec(&txn).await?;
    if let Some(component_id) = found.component_id {
        drop_empty_component(&txn, component_id).await?;
    }
    txn.commit().await?;
    discard(content.clone(), &found.blob_store, found.blob_id);
    Ok(())
}

pub async fn delete_component(content: &Content, component_id: i64) -> Result<bool, ContentError> {
    let assets = assets_of(content.db(), component_id).await?;
    let deleted = component::Entity::delete_by_id(component_id)
        .exec(content.db())
        .await?;
    delete_blobs(
        content,
        assets
            .into_iter()
            .map(|asset| (asset.blob_store, asset.blob_id))
            .collect(),
    )
    .await;
    Ok(deleted.rows_affected > 0)
}

pub async fn blobs_of_repository(
    db: &impl ConnectionTrait,
    repository: &str,
) -> Result<Vec<(String, Uuid)>, ContentError> {
    Ok(asset::Entity::find()
        .select_only()
        .column(asset::Column::BlobStore)
        .column(asset::Column::BlobId)
        .filter(asset::Column::Repository.eq(repository))
        .into_tuple()
        .all(db)
        .await?)
}

fn discard(content: Content, store: &str, id: Uuid) {
    let store = store.to_owned();
    tokio::spawn(async move {
        delete_blobs(&content, vec![(store, id)]).await;
    });
}

pub(crate) async fn delete_blobs(content: &Content, blobs: Vec<(String, Uuid)>) {
    for (store_name, id) in blobs {
        let deleted = match content.blob_stores().get(&store_name).await {
            Ok(store) => store.delete(id.into()).await.map(|_| ()),
            Err(error) => Err(error),
        };
        if let Err(error) = deleted {
            tracing::warn!(%error, store = store_name, blob = %id, "a blob that is no longer used stays in its store");
        }
    }
}
