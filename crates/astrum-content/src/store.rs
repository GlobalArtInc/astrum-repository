use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll};

use astrum_blob::{ContentStream, Written};
use astrum_db::entity::{asset, component};
use astrum_repo::config::{Repository, WritePolicy};
use bytes::Bytes;
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::Stream;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionTrait,
};
use serde_json::Value;
use uuid::Uuid;

use crate::mime;
use crate::transfer::{AssetOutcome, AssetStatus, Conflict, MAX_COMPONENTS, Outcome, Status};
use crate::usage::Usage;
use crate::{Content, ContentError, Ctx, holds};

mod drift;

pub use drift::Intent;
use drift::{
    Stance, drifting, judge, keeps_stored, mark_replaced, merged, renew, wanted, written_identities,
};

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
        Self::hosted_in(&ctx.repository, path)
    }

    pub fn hosted_in(repository: &Repository, path: impl Into<String>) -> Self {
        let storage = repository.attributes.storage.as_ref();
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
    let content_type = match content_type_of(&put, &sample) {
        Ok(content_type) => content_type,
        Err(error) => {
            discard(ctx.content.clone(), &store_name, written.id.uuid());
            return Err(error);
        }
    };
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

pub(crate) fn content_type_of(put: &PutAsset, sample: &[u8]) -> Result<String, ContentError> {
    let expected = mime::from_path(&put.path);
    let detected = mime::detect(sample);
    if put.check_content_type && !mime::compatible(detected.as_deref(), expected.as_deref()) {
        return Err(ContentError::BadRequest(format!(
            "Detected content type [{}], but expected [{}]: {}",
            detected.unwrap_or_default(),
            expected.unwrap_or_default(),
            put.path
        )));
    }
    Ok(put
        .content_type
        .clone()
        .filter(|value| !value.is_empty() && value != mime::OCTET_STREAM)
        .or(expected)
        .or(detected)
        .unwrap_or_else(|| mime::OCTET_STREAM.to_owned()))
}

pub async fn write_blob(ctx: &Ctx, body: ContentStream) -> Result<Written, ContentError> {
    let store_name = ctx.repository.blob_store().ok_or_else(|| {
        ContentError::internal(anyhow::anyhow!(
            "the repository {} has no blob store",
            ctx.name()
        ))
    })?;
    let store = ctx.content.blob_stores().get(store_name).await?;
    Ok(store.put(body).await?)
}

pub async fn adopt(ctx: &Ctx, put: PutAsset, written: &Written) -> Result<Asset, ContentError> {
    let repository = ctx.name();
    let refused = match put.write_policy {
        Some(WritePolicy::Deny) => Some(ContentError::BadRequest(format!(
            "the repository {repository} is read-only"
        ))),
        Some(WritePolicy::AllowOnce) if find(ctx.db(), repository, &put.path).await?.is_some() => {
            Some(redeploy(repository, &put.path))
        }
        _ => None,
    };
    if let Some(error) = refused {
        discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Err(error);
    }
    let content_type = put
        .content_type
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| mime::OCTET_STREAM.to_owned());
    match record(ctx, &put, &written.store, written, &content_type).await {
        Ok((asset, replaced)) => {
            if let Some((store, id)) = replaced {
                discard(ctx.content.clone(), &store, id);
            }
            Ok(asset)
        }
        Err(error) => {
            discard(ctx.content.clone(), &written.store, written.id.uuid());
            Err(error)
        }
    }
}

fn redeploy(repository: &str, path: &str) -> ContentError {
    ContentError::Conflict(format!(
        "{repository}{path} cannot be updated as asset already exists and redeploy is not allowed"
    ))
}

const ATTEMPTS: usize = 3;

type Recorded = (Asset, Option<(String, Uuid)>);

async fn record(
    ctx: &Ctx,
    put: &PutAsset,
    store_name: &str,
    written: &Written,
    content_type: &str,
) -> Result<Recorded, ContentError> {
    for _ in 0..ATTEMPTS {
        let txn = ctx.db().begin().await?;
        match record_once(ctx, &txn, put, store_name, written, content_type).await {
            Ok(Some(recorded)) => {
                txn.commit().await?;
                return Ok(recorded);
            }
            Ok(None) => rolled_back(txn).await,
            Err(error) => {
                rolled_back(txn).await;
                return Err(error);
            }
        }
    }
    Err(changed_meanwhile(ctx.name()))
}

async fn record_once(
    ctx: &Ctx,
    txn: &DatabaseTransaction,
    put: &PutAsset,
    store_name: &str,
    written: &Written,
    content_type: &str,
) -> Result<Option<Recorded>, ContentError> {
    let repository = ctx.name();
    let format = ctx.format_id();
    let now = Utc::now().fixed_offset();
    if !lock_repository(txn, repository).await? {
        return Err(missing_repository(repository));
    }
    let seen = find(txn, repository, &put.path).await?;
    let seen_component = seen.as_ref().and_then(|found| found.component_id);
    let new_component = match &put.component {
        Some(key) => {
            insert_component_if_absent(txn, repository, format, key, now).await?;
            match find_component(txn, repository, key).await? {
                Some(found) => Some(found.id),
                None => return Ok(None),
            }
        }
        None => None,
    };
    let wanted: Vec<i64> = new_component.into_iter().chain(seen_component).collect();
    let locked = lock_components(txn, &wanted).await?;
    if wanted.iter().any(|id| !locked.contains(id)) {
        return Ok(None);
    }
    let component_id = match &put.component {
        Some(key) => Some(upsert_component(txn, repository, format, key, now).await?),
        None => None,
    };
    if component_id != new_component {
        return Ok(None);
    }
    lock_usage(txn, repository).await?;
    let previous = asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Path.eq(&put.path))
        .lock_exclusive()
        .one(txn)
        .await?;
    if previous.as_ref().and_then(|row| row.component_id) != seen_component {
        return Ok(None);
    }
    if previous.is_some() && put.write_policy == Some(WritePolicy::AllowOnce) {
        return Err(redeploy(repository, &put.path));
    }
    let stored = upsert_asset(
        txn,
        asset_row(
            ctx,
            put,
            store_name,
            written,
            content_type,
            component_id,
            now,
        ),
    )
    .await?;
    if let Some(previous) = &previous
        && let Some(old_component) = previous.component_id
        && Some(old_component) != component_id
    {
        drop_empty_component(txn, old_component).await?;
    }
    let replaced = previous
        .filter(|previous| {
            previous.blob_id != stored.blob_id || previous.blob_store != stored.blob_store
        })
        .map(|previous| (previous.blob_store, previous.blob_id));
    Ok(Some((stored, replaced)))
}

fn asset_row(
    ctx: &Ctx,
    put: &PutAsset,
    store_name: &str,
    written: &Written,
    content_type: &str,
    component_id: Option<i64>,
    now: DateTime<FixedOffset>,
) -> asset::ActiveModel {
    let repository = ctx.name();
    let format = ctx.format_id();
    asset::ActiveModel {
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
    }
}

async fn upsert_asset(
    db: &impl ConnectionTrait,
    row: asset::ActiveModel,
) -> Result<Asset, ContentError> {
    Ok(asset::Entity::insert(row)
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
        .exec_with_returning(db)
        .await?)
}

async fn insert_new_asset(
    db: &impl ConnectionTrait,
    repository: &str,
    row: asset::ActiveModel,
) -> Result<(), ContentError> {
    let inserted = asset::Entity::insert(row)
        .on_conflict(
            OnConflict::columns([asset::Column::Repository, asset::Column::Path])
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(db)
        .await?;
    match inserted {
        0 => Err(changed_meanwhile(repository)),
        _ => Ok(()),
    }
}

fn component_row(
    repository: &str,
    format: &str,
    key: &ComponentKey,
    now: DateTime<FixedOffset>,
) -> component::ActiveModel {
    component::ActiveModel {
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
    }
}

fn component_identity_columns() -> [component::Column; 4] {
    [
        component::Column::Repository,
        component::Column::Namespace,
        component::Column::Name,
        component::Column::Version,
    ]
}

async fn upsert_component(
    db: &impl ConnectionTrait,
    repository: &str,
    format: &str,
    key: &ComponentKey,
    now: DateTime<FixedOffset>,
) -> Result<i64, ContentError> {
    let stored = component::Entity::insert(component_row(repository, format, key, now))
        .on_conflict(
            OnConflict::columns(component_identity_columns())
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

async fn insert_component_if_absent(
    db: &impl ConnectionTrait,
    repository: &str,
    format: &str,
    key: &ComponentKey,
    now: DateTime<FixedOffset>,
) -> Result<bool, ContentError> {
    let inserted = component::Entity::insert(component_row(repository, format, key, now))
        .on_conflict(
            OnConflict::columns(component_identity_columns())
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(db)
        .await?;
    Ok(inserted > 0)
}

async fn lock_components(
    db: &impl ConnectionTrait,
    ids: &[i64],
) -> Result<HashSet<i64>, ContentError> {
    if ids.is_empty() {
        return Ok(HashSet::new());
    }
    db.query_all(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT id FROM component WHERE id = ANY($1) ORDER BY id FOR UPDATE",
        [ids.to_vec().into()],
    ))
    .await?
    .iter()
    .map(|row| row.try_get::<i64>("", "id").map_err(ContentError::from))
    .collect()
}

fn missing_repository(repository: &str) -> ContentError {
    ContentError::not_found(format!("the repository {repository} does not exist"))
}

async fn lock_repository(
    db: &impl ConnectionTrait,
    repository: &str,
) -> Result<bool, ContentError> {
    Ok(db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 AS present FROM repository WHERE name = $1 FOR KEY SHARE",
            [repository.into()],
        ))
        .await?
        .is_some())
}

async fn lock_usage(db: &impl ConnectionTrait, repository: &str) -> Result<Usage, ContentError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT assets, bytes FROM repository_usage WHERE repository = $1 FOR UPDATE",
            [repository.into()],
        ))
        .await?
        .ok_or_else(|| missing_repository(repository))?;
    Ok(Usage {
        assets: row.try_get("", "assets")?,
        bytes: row.try_get("", "bytes")?,
    })
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

pub async fn set_attributes(
    db: &impl ConnectionTrait,
    asset_id: i64,
    attributes: Value,
) -> Result<(), ContentError> {
    let row = asset::ActiveModel {
        id: ActiveValue::Unchanged(asset_id),
        attributes: ActiveValue::Set(attributes),
        ..Default::default()
    };
    asset::Entity::update(row).exec(db).await?;
    Ok(())
}

pub async fn set_component_attributes(
    db: &impl ConnectionTrait,
    component_id: i64,
    attributes: Value,
) -> Result<(), ContentError> {
    let row = component::ActiveModel {
        id: ActiveValue::Unchanged(component_id),
        attributes: ActiveValue::Set(attributes),
        updated_at: ActiveValue::Set(Utc::now().fixed_offset()),
        ..Default::default()
    };
    component::Entity::update(row).exec(db).await?;
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

enum Removal {
    Removed((String, Uuid)),
    Gone,
    Again,
}

pub async fn delete_asset(content: &Content, found: Asset) -> Result<(), ContentError> {
    for _ in 0..ATTEMPTS {
        let txn = content.db().begin().await?;
        match remove_asset(&txn, &found).await {
            Ok(Removal::Removed((store, id))) => {
                txn.commit().await?;
                discard(content.clone(), &store, id);
                return Ok(());
            }
            Ok(Removal::Gone) => {
                rolled_back(txn).await;
                return Ok(());
            }
            Ok(Removal::Again) => rolled_back(txn).await,
            Err(error) => {
                rolled_back(txn).await;
                return Err(error);
            }
        }
    }
    Err(changed_meanwhile(&found.repository))
}

async fn remove_asset(txn: &DatabaseTransaction, found: &Asset) -> Result<Removal, ContentError> {
    if !lock_repository(txn, &found.repository).await? {
        return Ok(Removal::Gone);
    }
    let Some(current) = by_id(txn, found.id).await? else {
        return Ok(Removal::Gone);
    };
    if let Some(component_id) = current.component_id
        && !lock_components(txn, &[component_id])
            .await?
            .contains(&component_id)
    {
        return Ok(Removal::Again);
    }
    lock_usage(txn, &current.repository).await?;
    let Some(deleted) = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM asset WHERE id = $1 AND component_id IS NOT DISTINCT FROM $2 RETURNING blob_store, blob_id",
            [current.id.into(), current.component_id.into()],
        ))
        .await?
    else {
        return Ok(Removal::Again);
    };
    let blob = (
        deleted.try_get::<String>("", "blob_store")?,
        deleted.try_get::<Uuid>("", "blob_id")?,
    );
    if let Some(component_id) = current.component_id {
        drop_empty_component(txn, component_id).await?;
    }
    Ok(Removal::Removed(blob))
}

pub async fn delete_component(content: &Content, component_id: i64) -> Result<bool, ContentError> {
    let txn = content.db().begin().await?;
    match remove_component(&txn, component_id).await {
        Ok(Some(blobs)) => {
            txn.commit().await?;
            delete_blobs(content, blobs).await;
            Ok(true)
        }
        Ok(None) => {
            rolled_back(txn).await;
            Ok(false)
        }
        Err(error) => {
            rolled_back(txn).await;
            Err(error)
        }
    }
}

async fn remove_component(
    txn: &DatabaseTransaction,
    component_id: i64,
) -> Result<Option<Vec<(String, Uuid)>>, ContentError> {
    let Some(found) = component_by_id(txn, component_id).await? else {
        return Ok(None);
    };
    if !lock_repository(txn, &found.repository).await? {
        return Ok(None);
    }
    if !lock_components(txn, &[component_id])
        .await?
        .contains(&component_id)
    {
        return Ok(None);
    }
    let assets = assets_of(txn, component_id).await?;
    lock_usage(txn, &found.repository).await?;
    let deleted = component::Entity::delete_by_id(component_id)
        .exec(txn)
        .await?;
    Ok((deleted.rows_affected > 0).then(|| {
        assets
            .into_iter()
            .map(|asset| (asset.blob_store, asset.blob_id))
            .collect()
    }))
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

pub fn discard(content: Content, store: &str, id: Uuid) {
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

pub type Replaceable<'a> = &'a (dyn Fn(&str) -> bool + Send + Sync);

pub struct Planned {
    pub put: PutAsset,
    pub size: u64,
    pub sha256: String,
}

struct Candidate<'a> {
    put: &'a PutAsset,
    size: u64,
    sha256: &'a str,
}

struct Decision {
    status: Status,
    assets: Vec<AssetOutcome>,
    bytes_copied: u64,
    bytes_reused: u64,
    writes: bool,
}

impl Decision {
    fn outcome(self, component_ids: Vec<i64>) -> Outcome {
        Outcome {
            status: self.status,
            component_id: component_ids.first().copied(),
            component_ids,
            assets: self.assets,
            bytes_copied: self.bytes_copied,
            bytes_reused: self.bytes_reused,
            warnings: Vec::new(),
        }
    }
}

struct Executed {
    outcome: Outcome,
    committed: bool,
    unused: Vec<(String, Uuid)>,
    refreshed: Vec<ComponentIdentity>,
}

fn check_write(
    repository: &str,
    put: &PutAsset,
    existing: Option<&Asset>,
    conflict: Conflict,
    replaceable: Option<&dyn Fn(&str) -> bool>,
) -> Result<(), ContentError> {
    match (put.write_policy, existing) {
        (None, _) => Err(ContentError::internal(anyhow::anyhow!(
            "the write policy of {repository}{} is not set",
            put.path
        ))),
        (Some(WritePolicy::Deny), _) => Err(ContentError::BadRequest(format!(
            "the repository {repository} is read-only"
        ))),
        (Some(WritePolicy::AllowOnce), Some(_)) => {
            match conflict == Conflict::Replace
                && replaceable.is_some_and(|allowed| allowed(&put.path))
            {
                true => Ok(()),
                false => Err(redeploy(repository, &put.path)),
            }
        }
        (Some(WritePolicy::Allow | WritePolicy::AllowOnce), _) => Ok(()),
    }
}

fn decide(
    repository: &str,
    candidates: &[Candidate<'_>],
    existing: &HashMap<String, Asset>,
    conflict: Conflict,
    planning: bool,
    replaceable: Option<&dyn Fn(&str) -> bool>,
) -> Result<Decision, ContentError> {
    let mut statuses: Vec<AssetStatus> = candidates
        .iter()
        .map(|candidate| match existing.get(&candidate.put.path) {
            None => AssetStatus::New,
            Some(found) if found.sha256.eq_ignore_ascii_case(candidate.sha256) => {
                AssetStatus::Unchanged
            }
            Some(_) => AssetStatus::Conflict,
        })
        .collect();
    let report = |statuses: &[AssetStatus]| -> Vec<AssetOutcome> {
        candidates
            .iter()
            .zip(statuses)
            .map(|(candidate, status)| AssetOutcome {
                path: candidate.put.path.clone(),
                status: *status,
                size: candidate.size,
            })
            .collect()
    };
    let conflicting: Vec<&str> = candidates
        .iter()
        .zip(&statuses)
        .filter(|(_, status)| **status == AssetStatus::Conflict)
        .map(|(candidate, _)| candidate.put.path.as_str())
        .collect();
    if !conflicting.is_empty() {
        let refused = |status| Decision {
            status,
            assets: report(&statuses),
            bytes_copied: 0,
            bytes_reused: 0,
            writes: false,
        };
        match conflict {
            Conflict::Fail if planning => return Ok(refused(Status::Conflict)),
            Conflict::Fail => {
                return Err(ContentError::Conflict(format!(
                    "{repository}: {} exist with different content",
                    conflicting.join(", ")
                )));
            }
            Conflict::Skip => return Ok(refused(Status::Skipped)),
            Conflict::Replace => {}
        }
    }
    for (candidate, status) in candidates.iter().zip(statuses.iter_mut()) {
        match status {
            AssetStatus::New => {
                check_write(repository, candidate.put, None, conflict, replaceable)?
            }
            AssetStatus::Conflict => {
                check_write(
                    repository,
                    candidate.put,
                    existing.get(&candidate.put.path),
                    conflict,
                    replaceable,
                )?;
                *status = AssetStatus::Replaced;
            }
            AssetStatus::Unchanged | AssetStatus::Replaced => {}
        }
    }
    let sum = |wanted: fn(AssetStatus) -> bool| -> u64 {
        candidates
            .iter()
            .zip(&statuses)
            .filter(|(_, status)| wanted(**status))
            .map(|(candidate, _)| candidate.size)
            .sum()
    };
    let bytes_copied = sum(|status| matches!(status, AssetStatus::New | AssetStatus::Replaced));
    let bytes_reused = sum(|status| status == AssetStatus::Unchanged);
    let writes = statuses
        .iter()
        .any(|status| matches!(status, AssetStatus::New | AssetStatus::Replaced));
    Ok(Decision {
        status: match writes {
            true => Status::Copied,
            false => Status::Unchanged,
        },
        assets: report(&statuses),
        bytes_copied,
        bytes_reused,
        writes,
    })
}

async fn existing_assets(
    db: &impl ConnectionTrait,
    repository: &str,
    paths: &[String],
) -> Result<HashMap<String, Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Path.is_in(paths.iter().cloned()))
        .all(db)
        .await?
        .into_iter()
        .map(|found| (found.path.clone(), found))
        .collect())
}

async fn find_component(
    db: &impl ConnectionTrait,
    repository: &str,
    key: &ComponentKey,
) -> Result<Option<Component>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(repository))
        .filter(component::Column::Namespace.eq(&key.namespace))
        .filter(component::Column::Name.eq(&key.name))
        .filter(component::Column::Version.eq(&key.version))
        .one(db)
        .await?)
}

pub type ComponentIdentity = (String, String, String);

fn component_keys<'a>(
    puts: impl Iterator<Item = &'a PutAsset>,
) -> Result<Vec<ComponentKey>, ContentError> {
    let mut keys: BTreeMap<ComponentIdentity, ComponentKey> = BTreeMap::new();
    for found in puts.filter_map(|put| put.component.as_ref()) {
        let identity = (
            found.namespace.clone(),
            found.name.clone(),
            found.version.clone(),
        );
        keys.entry(identity).or_insert_with(|| found.clone());
        if keys.len() > MAX_COMPONENTS {
            return Err(ContentError::BadRequest(format!(
                "one transfer takes at most {MAX_COMPONENTS} components"
            )));
        }
    }
    Ok(keys.into_values().collect())
}

fn identity_of(key: &ComponentKey) -> ComponentIdentity {
    (key.namespace.clone(), key.name.clone(), key.version.clone())
}

pub async fn preview_many(
    ctx: &Ctx,
    planned: &[Planned],
    conflict: Conflict,
) -> Result<Outcome, ContentError> {
    preview_many_with(ctx, planned, conflict, None).await
}

pub async fn preview_many_with(
    ctx: &Ctx,
    planned: &[Planned],
    conflict: Conflict,
    replaceable: Option<Replaceable<'_>>,
) -> Result<Outcome, ContentError> {
    preview_many_carrying(ctx, planned, conflict, replaceable, &[]).await
}

pub async fn preview_many_carrying(
    ctx: &Ctx,
    planned: &[Planned],
    conflict: Conflict,
    replaceable: Option<Replaceable<'_>>,
    intents: &[Intent],
) -> Result<Outcome, ContentError> {
    let repository = ctx.name();
    let paths: Vec<String> = planned.iter().map(|item| item.put.path.clone()).collect();
    let existing = existing_assets(ctx.db(), repository, &paths).await?;
    let candidates: Vec<Candidate<'_>> = planned
        .iter()
        .map(|item| Candidate {
            put: &item.put,
            size: item.size,
            sha256: &item.sha256,
        })
        .collect();
    let decision = decide(
        repository,
        &candidates,
        &existing,
        conflict,
        true,
        replaceable.map(|allowed| allowed as &dyn Fn(&str) -> bool),
    )?;
    let written = written_identities(&candidates, &decision);
    let mut component_ids = Vec::new();
    let mut drifted = HashSet::new();
    for key in component_keys(planned.iter().map(|item| &item.put))? {
        let Some(found) = find_component(ctx.db(), repository, &key).await? else {
            continue;
        };
        component_ids.push(found.id);
        let identity = identity_of(&key);
        if !written.contains(&identity)
            && wanted(intents, &identity)
                .map(|wanted| merged(&found.attributes, wanted))
                .transpose()?
                .flatten()
                .is_some()
        {
            drifted.insert(identity);
        }
    }
    let (decision, replaced) = judge(
        repository,
        &candidates,
        &existing,
        decision,
        &drifted,
        Stance {
            conflict,
            planning: true,
            replaceable,
        },
    )?;
    let mut outcome = decision.outcome(component_ids);
    mark_replaced(&mut outcome, &replaced);
    Ok(outcome)
}

fn validate_many(items: &[(PutAsset, Written)]) -> Result<Vec<ComponentKey>, ContentError> {
    if items.is_empty() {
        return Err(ContentError::BadRequest(
            "there is nothing to record".to_owned(),
        ));
    }
    let mut paths = HashSet::new();
    for (put, written) in items {
        if put.write_policy.is_none() {
            return Err(ContentError::internal(anyhow::anyhow!(
                "the write policy of {} is not set",
                put.path
            )));
        }
        if written.digests.sha256.is_empty() {
            return Err(ContentError::internal(anyhow::anyhow!(
                "the blob of {} has no SHA-256",
                put.path
            )));
        }
        if !paths.insert(put.path.as_str()) {
            return Err(ContentError::BadRequest(format!(
                "{} is recorded twice in one transfer",
                put.path
            )));
        }
    }
    component_keys(items.iter().map(|(put, _)| put))
}

pub async fn record_many(
    ctx: &Ctx,
    items: Vec<(PutAsset, Written)>,
    conflict: Conflict,
    action: &str,
) -> Result<Outcome, ContentError> {
    record_many_with(ctx, items, conflict, action, None).await
}

pub async fn record_many_with(
    ctx: &Ctx,
    items: Vec<(PutAsset, Written)>,
    conflict: Conflict,
    action: &str,
    replaceable: Option<Replaceable<'_>>,
) -> Result<Outcome, ContentError> {
    record_many_carrying(ctx, items, conflict, action, replaceable, &[])
        .await
        .map(|(outcome, _)| outcome)
}

pub async fn record_many_carrying(
    ctx: &Ctx,
    items: Vec<(PutAsset, Written)>,
    conflict: Conflict,
    action: &str,
    replaceable: Option<Replaceable<'_>>,
    intents: &[Intent],
) -> Result<(Outcome, Vec<ComponentIdentity>), ContentError> {
    let blobs: Vec<(String, Uuid)> = items
        .iter()
        .map(|(_, written)| (written.store.clone(), written.id.uuid()))
        .collect();
    let keys = match validate_many(&items) {
        Ok(keys) => keys,
        Err(error) => {
            abandon(&ctx.content, blobs, true).await;
            return Err(error);
        }
    };
    let txn = match ctx.db().begin().await {
        Ok(txn) => txn,
        Err(error) => {
            abandon(&ctx.content, blobs, true).await;
            return Err(error.into());
        }
    };
    let rules = Rules {
        conflict,
        action,
        replaceable,
        intents,
    };
    let executed = write_many(ctx, &txn, &items, &keys, rules).await;
    match executed {
        Ok(Executed {
            outcome,
            committed: true,
            unused,
            refreshed,
        }) => match txn.commit().await {
            Ok(()) => {
                abandon(&ctx.content, unused, false).await;
                Ok((outcome, refreshed))
            }
            Err(error) => Err(error.into()),
        },
        Ok(Executed { outcome, .. }) => {
            rolled_back(txn).await;
            abandon(&ctx.content, blobs, true).await;
            Ok((outcome, Vec::new()))
        }
        Err(error) => {
            rolled_back(txn).await;
            abandon(&ctx.content, blobs, true).await;
            Err(error)
        }
    }
}

async fn rolled_back(txn: sea_orm::DatabaseTransaction) {
    if let Err(error) = txn.rollback().await {
        tracing::warn!(%error, "a transfer transaction could not be rolled back explicitly");
    }
}

struct Resolved<'a> {
    key: &'a ComponentKey,
    id: Option<i64>,
    created: bool,
}

fn changed_meanwhile(repository: &str) -> ContentError {
    ContentError::Conflict(format!(
        "{repository} changed while the transfer ran; try again"
    ))
}

struct Rules<'a> {
    conflict: Conflict,
    action: &'a str,
    replaceable: Option<Replaceable<'a>>,
    intents: &'a [Intent],
}

async fn write_many(
    ctx: &Ctx,
    txn: &impl ConnectionTrait,
    items: &[(PutAsset, Written)],
    keys: &[ComponentKey],
    rules: Rules<'_>,
) -> Result<Executed, ContentError> {
    let Rules {
        conflict,
        action,
        replaceable,
        intents,
    } = rules;
    let repository = ctx.name();
    if !lock_repository(txn, repository).await? {
        return Err(missing_repository(repository));
    }
    let new_blobs: Vec<(String, astrum_blob::BlobId)> = items
        .iter()
        .map(|(_, written)| (written.store.clone(), written.id))
        .collect();
    if holds::consume(txn, &new_blobs).await? != new_blobs.len() as u64 {
        return Err(ContentError::Conflict(
            "a blob of the transfer lost its hold; the transfer was abandoned".to_owned(),
        ));
    }
    txn.query_one(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('astrum.action', $1, true), set_config('astrum.initiator', $2, true)",
        [action.into(), ctx.uploader().into()],
    ))
    .await?;
    let now = Utc::now().fixed_offset();
    let mut paths: Vec<String> = items.iter().map(|(put, _)| put.path.clone()).collect();
    paths.sort();
    let preliminary = existing_assets(txn, repository, &paths).await?;
    let mut to_write: HashSet<ComponentIdentity> = HashSet::new();
    for (put, written) in items {
        let same = preliminary
            .get(&put.path)
            .is_some_and(|found| found.sha256.eq_ignore_ascii_case(&written.digests.sha256));
        if let (false, Some(found)) = (same, put.component.as_ref()) {
            to_write.insert(identity_of(found));
        }
    }
    let mut resolved: Vec<Resolved<'_>> = Vec::with_capacity(keys.len());
    for key in keys {
        let mut id = find_component(txn, repository, key)
            .await?
            .map(|found| found.id);
        let mut created = false;
        if id.is_none() && to_write.contains(&identity_of(key)) {
            created =
                insert_component_if_absent(txn, repository, ctx.format_id(), key, now).await?;
            id = Some(
                find_component(txn, repository, key)
                    .await?
                    .ok_or_else(|| changed_meanwhile(repository))?
                    .id,
            );
        }
        resolved.push(Resolved { key, id, created });
    }
    let own: HashMap<ComponentIdentity, i64> = resolved
        .iter()
        .filter_map(|found| found.id.map(|id| (identity_of(found.key), id)))
        .collect();
    let mut component_ids: BTreeSet<i64> = own.values().copied().collect();
    component_ids.extend(
        existing_assets(txn, repository, &paths)
            .await?
            .values()
            .filter_map(|found| found.component_id),
    );
    let locked = lock_components(txn, &component_ids.into_iter().collect::<Vec<i64>>()).await?;
    if resolved
        .iter()
        .any(|found| found.id.is_some_and(|id| !locked.contains(&id)))
    {
        return Err(changed_meanwhile(repository));
    }
    lock_usage(txn, repository).await?;
    let existing: HashMap<String, Asset> = asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Path.is_in(paths))
        .order_by_asc(asset::Column::Path)
        .lock_exclusive()
        .all(txn)
        .await?
        .into_iter()
        .map(|found| (found.path.clone(), found))
        .collect();
    if existing
        .values()
        .any(|found| found.component_id.is_some_and(|id| !locked.contains(&id)))
    {
        return Err(changed_meanwhile(repository));
    }
    let candidates: Vec<Candidate<'_>> = items
        .iter()
        .map(|(put, written)| Candidate {
            put,
            size: written.size,
            sha256: &written.digests.sha256,
        })
        .collect();
    let decision = decide(
        repository,
        &candidates,
        &existing,
        conflict,
        false,
        replaceable.map(|allowed| allowed as &dyn Fn(&str) -> bool),
    )?;
    let written_keys = written_identities(&candidates, &decision);
    let drift = match matches!(decision.status, Status::Copied | Status::Unchanged) {
        true => drifting(txn, repository, &resolved, &written_keys, intents).await?,
        false => Vec::new(),
    };
    let drifted: HashSet<ComponentIdentity> =
        drift.iter().map(|found| found.identity.clone()).collect();
    let (decision, replaced) = judge(
        repository,
        &candidates,
        &existing,
        decision,
        &drifted,
        Stance {
            conflict,
            planning: false,
            replaceable,
        },
    )?;
    let drift = match replaced.is_empty() {
        true => Vec::new(),
        false => drift,
    };
    if !decision.writes && drift.is_empty() {
        let ids: Vec<i64> = match keys.is_empty() {
            true => existing
                .values()
                .filter_map(|found| found.component_id)
                .collect::<BTreeSet<i64>>()
                .into_iter()
                .collect(),
            false => resolved
                .iter()
                .filter(|found| !found.created)
                .filter_map(|found| found.id)
                .collect(),
        };
        return Ok(Executed {
            outcome: decision.outcome(ids),
            committed: false,
            unused: Vec::new(),
            refreshed: Vec::new(),
        });
    }
    for found in resolved
        .iter()
        .filter(|found| written_keys.contains(&identity_of(found.key)))
    {
        if found.id.is_none() {
            return Err(changed_meanwhile(repository));
        }
        let identity = identity_of(found.key);
        match !found.created && keeps_stored(intents, &identity) {
            true => renew(txn, found.id.unwrap_or_default(), now).await?,
            false => {
                upsert_component(txn, repository, ctx.format_id(), found.key, now).await?;
            }
        }
    }
    for found in &drift {
        component::Entity::update_many()
            .col_expr(
                component::Column::Attributes,
                Expr::value(found.attributes.clone()),
            )
            .col_expr(component::Column::UpdatedAt, Expr::value(now))
            .filter(component::Column::Id.eq(found.id))
            .exec(txn)
            .await?;
    }
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|a, b| items[*a].0.path.cmp(&items[*b].0.path));
    let mut unused = Vec::new();
    let mut left_components = BTreeSet::new();
    for index in order {
        let (put, written) = &items[index];
        let status = decision.assets[index].status;
        if status == AssetStatus::Unchanged {
            unused.push((written.store.clone(), written.id.uuid()));
            continue;
        }
        let content_type = put
            .content_type
            .clone()
            .filter(|value| !value.is_empty() && value != mime::OCTET_STREAM)
            .or_else(|| mime::from_path(&put.path))
            .unwrap_or_else(|| mime::OCTET_STREAM.to_owned());
        let asset_component = put
            .component
            .as_ref()
            .and_then(|key| own.get(&identity_of(key)).copied());
        let row = asset_row(
            ctx,
            put,
            &written.store,
            written,
            &content_type,
            asset_component,
            now,
        );
        match existing.contains_key(&put.path) {
            true => {
                upsert_asset(txn, row).await?;
            }
            false => insert_new_asset(txn, repository, row).await?,
        }
        if let Some(previous) = existing.get(&put.path) {
            unused.push((previous.blob_store.clone(), previous.blob_id));
            if let Some(old) = previous.component_id
                && Some(old) != asset_component
            {
                left_components.insert(old);
            }
        }
    }
    let mut ids = Vec::new();
    for found in &resolved {
        let Some(id) = found.id else { continue };
        match !found.created || written_keys.contains(&identity_of(found.key)) {
            true => ids.push(id),
            false => {
                left_components.insert(id);
            }
        }
    }
    for old in left_components {
        drop_empty_component(txn, old).await?;
    }
    let mut outcome = decision.outcome(ids);
    mark_replaced(&mut outcome, &replaced);
    Ok(Executed {
        outcome,
        committed: true,
        unused,
        refreshed: drift.into_iter().map(|found| found.identity).collect(),
    })
}

pub(crate) async fn abandon(content: &Content, blobs: Vec<(String, Uuid)>, release: bool) {
    for (store_name, id) in blobs {
        let deleted = match content.blob_stores().get(&store_name).await {
            Ok(store) => store.delete(id.into()).await.map(|_| ()),
            Err(error) => Err(error),
        };
        match (deleted, release) {
            (Ok(()), true) => {
                if let Err(error) = holds::release(content.db(), &store_name, id.into()).await {
                    tracing::warn!(%error, store = store_name, blob = %id, "the hold of a discarded blob stays until the sweep removes it");
                }
            }
            (Ok(()), false) => {}
            (Err(error), true) => {
                tracing::warn!(%error, store = store_name, blob = %id, "a blob of an abandoned transfer stays held until the sweep retries its removal");
            }
            (Err(error), false) => {
                tracing::warn!(%error, store = store_name, blob = %id, "a blob that is no longer used stays in its store");
            }
        }
    }
}
