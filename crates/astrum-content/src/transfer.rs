use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use astrum_blob::{BlobStore, Digests, Written, content_of};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::import::{Body, Described, Imported, ImportedComponent, Plan};
use crate::store::{self, Component, ComponentKey, Planned, PutAsset};
use crate::{Content, ContentError, Ctx, holds, mime, normalize};

pub const MAX_ASSETS: usize = 10_000;
pub const PROMOTED: &str = "promoted";
const HOLD_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_TEXT: usize = 4096;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Conflict {
    #[default]
    Fail,
    Skip,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Copied,
    Unchanged,
    Skipped,
    Conflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetStatus {
    New,
    Replaced,
    Unchanged,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetOutcome {
    pub path: String,
    pub status: AssetStatus,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub status: Status,
    pub component_id: Option<i64>,
    pub assets: Vec<AssetOutcome>,
    pub bytes_copied: u64,
    pub bytes_reused: u64,
}

impl Outcome {
    fn nothing() -> Self {
        Self {
            status: Status::Unchanged,
            component_id: None,
            assets: Vec::new(),
            bytes_copied: 0,
            bytes_reused: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Terms {
    pub conflict: Conflict,
    pub action: &'static str,
    pub dry_run: bool,
}

impl Terms {
    pub fn new(conflict: Conflict, action: &'static str) -> Self {
        Self {
            conflict,
            action,
            dry_run: false,
        }
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleAsset {
    pub path: String,
    pub kind: String,
    pub content_type: String,
    pub size: u64,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub md5: String,
    pub attributes: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub format: String,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub assets: Vec<BundleAsset>,
    #[serde(default)]
    pub meta: Value,
    #[serde(default)]
    pub origin: Option<String>,
}

#[derive(Clone)]
pub struct Sink {
    pub name: String,
    pub store: Arc<dyn BlobStore>,
}

impl Sink {
    pub async fn of(target: &Ctx) -> Result<Self, ContentError> {
        let name = target.repository.blob_store().ok_or_else(|| {
            ContentError::internal(anyhow::anyhow!(
                "the repository {} has no blob store",
                target.name()
            ))
        })?;
        Ok(Self {
            name: name.to_owned(),
            store: target.content.blob_stores().get(name).await?,
        })
    }
}

#[async_trait]
pub trait BlobSource: Send + Sync {
    async fn write(&self, sink: &Sink, asset: &BundleAsset) -> Result<Written, ContentError>;
}

pub struct LocalSource {
    source: Ctx,
}

impl LocalSource {
    pub fn new(source: Ctx) -> Self {
        Self { source }
    }
}

#[async_trait]
impl BlobSource for LocalSource {
    async fn write(&self, sink: &Sink, asset: &BundleAsset) -> Result<Written, ContentError> {
        let row = store::find(self.source.db(), self.source.name(), &asset.path)
            .await?
            .ok_or_else(|| {
                ContentError::not_found(format!(
                    "{}{} is gone from the source repository",
                    self.source.name(),
                    asset.path
                ))
            })?;
        let digests = Digests {
            sha1: row.sha1.clone(),
            sha256: row.sha256.clone(),
            sha512: row.sha512.clone(),
            md5: row.md5.clone(),
        };
        if row.size as u64 != asset.size || !row.sha256.eq_ignore_ascii_case(&asset.sha256) {
            return Err(ContentError::Conflict(format!(
                "{}{} changed in the source repository after the bundle was made",
                self.source.name(),
                asset.path
            )));
        }
        if row.blob_store == sink.name {
            return Ok(sink.store.duplicate(row.blob_id.into(), &digests).await?);
        }
        let origin = self
            .source
            .content
            .blob_stores()
            .get(&row.blob_store)
            .await?;
        let content = origin.read(row.blob_id.into()).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "the blob of {}{} is missing from the blob store {}",
                self.source.name(),
                asset.path,
                row.blob_store
            ))
        })?;
        Ok(sink.store.put(content.stream).await?)
    }
}

#[async_trait]
pub trait Transfer: Send + Sync {
    async fn bundle(&self, source: &Ctx, component: &Component) -> Result<Bundle, ContentError>;

    async fn receive(
        &self,
        target: &Ctx,
        bundle: &Bundle,
        blobs: &dyn BlobSource,
        terms: Terms,
    ) -> Result<Outcome, ContentError>;
}

pub type PlanFn = fn(RepositoryType, &str, &str) -> Plan;
pub type DescribeFn = for<'a> fn(&'a Imported) -> BoxFuture<'a, Result<Option<Described>, String>>;
pub type AfterFn = for<'a> fn(&'a Ctx) -> BoxFuture<'a, Result<(), ContentError>>;
pub type AdmitFn = for<'a> fn(&'a Ctx, &'a Bundle) -> BoxFuture<'a, Result<(), ContentError>>;

pub struct GenericTransfer {
    pub plan: PlanFn,
    pub describe: DescribeFn,
    pub after: Option<AfterFn>,
    pub admit: Option<AdmitFn>,
}

struct Staged<'a> {
    content: &'a Content,
    held: Vec<Written>,
    keeper: holds::Keeper,
}

impl<'a> Staged<'a> {
    fn new(content: &'a Content) -> Self {
        Self {
            content,
            held: Vec::new(),
            keeper: holds::Keeper::start(content.db().clone(), HOLD_TTL, HOLD_TTL / 3),
        }
    }

    async fn hold(&mut self, written: Written) -> Result<(), ContentError> {
        match holds::hold(self.content.db(), &written.store, written.id, HOLD_TTL).await {
            Ok(()) => {
                self.keeper.add(&written.store, written.id);
                self.held.push(written);
                Ok(())
            }
            Err(error) => {
                store::abandon(
                    self.content,
                    vec![(written.store, written.id.uuid())],
                    false,
                )
                .await;
                Err(error.into())
            }
        }
    }

    fn check(&self) -> Result<(), ContentError> {
        self.keeper.check()
    }

    async fn drop_blob(&mut self, written: &Written) {
        self.keeper.remove(&written.store, written.id);
        self.held
            .retain(|held| !(held.store == written.store && held.id == written.id));
        store::abandon(
            self.content,
            vec![(written.store.clone(), written.id.uuid())],
            true,
        )
        .await;
    }

    async fn abandon(self) {
        let blobs = self
            .held
            .iter()
            .map(|written| (written.store.clone(), written.id.uuid()))
            .collect();
        store::abandon(self.content, blobs, true).await;
    }
}

fn text(field: &str, value: &str) -> Result<(), ContentError> {
    if value.len() > MAX_TEXT || value.chars().any(char::is_control) {
        return Err(ContentError::BadRequest(format!(
            "the bundle field {field} is too long or holds control characters"
        )));
    }
    Ok(())
}

fn hex(field: &str, value: &str, length: usize, required: bool) -> Result<(), ContentError> {
    let valid = match value.is_empty() {
        true => !required,
        false => value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
    };
    match valid {
        true => Ok(()),
        false => Err(ContentError::BadRequest(format!(
            "the bundle field {field} is not a hexadecimal digest of {length} characters"
        ))),
    }
}

fn check_bundle(target: &Ctx, bundle: &Bundle) -> Result<(), ContentError> {
    if target.kind() != RepositoryType::Hosted {
        return Err(ContentError::BadRequest(format!(
            "the repository {} is not hosted",
            target.name()
        )));
    }
    if bundle.format != target.format_id() {
        return Err(ContentError::BadRequest(format!(
            "a {} bundle cannot go into the {} repository {}",
            bundle.format,
            target.format_id(),
            target.name()
        )));
    }
    if bundle.assets.is_empty() || bundle.assets.len() > MAX_ASSETS {
        return Err(ContentError::BadRequest(format!(
            "a bundle holds between 1 and {MAX_ASSETS} assets"
        )));
    }
    for (field, value) in [
        ("namespace", &bundle.namespace),
        ("name", &bundle.name),
        ("version", &bundle.version),
    ] {
        text(field, value)?;
    }
    if let Some(origin) = &bundle.origin {
        text("origin", origin)?;
    }
    let mut paths = HashSet::new();
    for asset in &bundle.assets {
        text("path", &asset.path)?;
        text("kind", &asset.kind)?;
        text("contentType", &asset.content_type)?;
        if normalize(&asset.path)? != asset.path || asset.path.ends_with('/') {
            return Err(ContentError::BadRequest(format!(
                "the bundle path {} is not a normalised file path",
                asset.path
            )));
        }
        if !paths.insert(asset.path.as_str()) {
            return Err(ContentError::BadRequest(format!(
                "the bundle lists {} twice",
                asset.path
            )));
        }
        if asset.size > i64::MAX as u64 {
            return Err(ContentError::BadRequest(format!(
                "the size of {} is not valid",
                asset.path
            )));
        }
        hex("sha256", &asset.sha256, 64, true)?;
        hex("sha1", &asset.sha1, 40, false)?;
        hex("sha512", &asset.sha512, 128, false)?;
        hex("md5", &asset.md5, 32, false)?;
    }
    Ok(())
}

fn verify(written: &Written, asset: &BundleAsset) -> Result<(), ContentError> {
    if written.size != asset.size {
        return Err(ContentError::BadRequest(format!(
            "{} holds {} bytes, the bundle says {}",
            asset.path, written.size, asset.size
        )));
    }
    for (algorithm, wanted, found) in [
        ("SHA-256", &asset.sha256, &written.digests.sha256),
        ("SHA-1", &asset.sha1, &written.digests.sha1),
        ("SHA-512", &asset.sha512, &written.digests.sha512),
        ("MD5", &asset.md5, &written.digests.md5),
    ] {
        let required = algorithm == "SHA-256";
        if (required || !wanted.is_empty()) && !wanted.eq_ignore_ascii_case(found) {
            return Err(ContentError::BadRequest(format!(
                "{} has {algorithm} {found}, the bundle says {wanted}",
                asset.path
            )));
        }
    }
    Ok(())
}

fn hint(bundle: &Bundle) -> ComponentKey {
    ComponentKey {
        namespace: bundle.namespace.clone(),
        name: bundle.name.clone(),
        version: bundle.version.clone(),
        kind: String::new(),
        attributes: Value::Object(Default::default()),
    }
}

impl GenericTransfer {
    fn wanted<'a>(
        &self,
        target: &Ctx,
        bundle: &'a Bundle,
    ) -> Result<Vec<&'a BundleAsset>, ContentError> {
        check_bundle(target, bundle)?;
        let mut wanted = Vec::new();
        for asset in &bundle.assets {
            match (self.plan)(RepositoryType::Hosted, &asset.path, &asset.kind) {
                Plan::Import => wanted.push(asset),
                Plan::Derived => {}
                Plan::Skip(reason) => {
                    return Err(ContentError::BadRequest(format!(
                        "{} cannot be transferred: {reason}",
                        asset.path
                    )));
                }
            }
        }
        Ok(wanted)
    }

    async fn preview(
        &self,
        target: &Ctx,
        bundle: &Bundle,
        wanted: &[&BundleAsset],
        terms: Terms,
    ) -> Result<Outcome, ContentError> {
        let key = hint(bundle);
        let planned: Vec<Planned> = wanted
            .iter()
            .map(|asset| Planned {
                put: PutAsset {
                    kind: asset.kind.clone(),
                    component: Some(key.clone()),
                    ..PutAsset::hosted(target, asset.path.clone())
                },
                size: asset.size,
                sha256: asset.sha256.clone(),
            })
            .collect();
        store::preview_many(target, &planned, terms.conflict).await
    }

    async fn stage(
        &self,
        target: &Ctx,
        bundle: &Bundle,
        blobs: &dyn BlobSource,
        sink: &Sink,
        wanted: &[&BundleAsset],
        staged: &mut Staged<'_>,
    ) -> Result<Vec<(PutAsset, Written)>, ContentError> {
        let mut items = Vec::new();
        for asset in wanted {
            staged.check()?;
            let written = blobs.write(sink, asset).await?;
            staged.hold(written.clone()).await?;
            verify(&written, asset)?;
            let imported = Imported {
                repository: RepositoryType::Hosted,
                config: target.repository.clone(),
                path: asset.path.clone(),
                kind: asset.kind.clone(),
                content_type: asset.content_type.clone(),
                attributes: asset.attributes.clone(),
                component: Some(ImportedComponent {
                    namespace: bundle.namespace.clone(),
                    name: bundle.name.clone(),
                    version: bundle.version.clone(),
                    ..Default::default()
                }),
                store: sink.store.clone(),
                blob: written.id,
                size: written.size,
                sha256: written.digests.sha256.clone(),
            };
            let Some(described) = (self.describe)(&imported)
                .await
                .map_err(ContentError::BadRequest)?
            else {
                staged.drop_blob(&written).await;
                continue;
            };
            if described.keep_component {
                return Err(ContentError::BadRequest(format!(
                    "{} needs its component kept, which the transfer does not support yet",
                    asset.path
                )));
            }
            let Described {
                put, body, extra, ..
            } = described;
            let (primary, sample) = match body {
                Body::Source => {
                    let sample = imported
                        .head(mime::SAMPLE as u64)
                        .await
                        .map_err(|error| ContentError::BadRequest(error.to_string()))?;
                    (written, sample)
                }
                Body::Bytes(bytes) => {
                    staged.drop_blob(&written).await;
                    let fresh = Self::write_bytes(sink, staged, &bytes).await?;
                    (fresh, bytes)
                }
            };
            items.push(Self::finish(target, put, primary, &sample)?);
            for (put, bytes) in extra {
                staged.check()?;
                let fresh = Self::write_bytes(sink, staged, &bytes).await?;
                items.push(Self::finish(target, put, fresh, &bytes)?);
            }
        }
        staged.check()?;
        Ok(items)
    }

    async fn write_bytes(
        sink: &Sink,
        staged: &mut Staged<'_>,
        bytes: &[u8],
    ) -> Result<Written, ContentError> {
        let written = sink.store.put(content_of(bytes.to_vec())).await?;
        staged.hold(written.clone()).await?;
        Ok(written)
    }

    fn finish(
        target: &Ctx,
        mut put: PutAsset,
        written: Written,
        sample: &[u8],
    ) -> Result<(PutAsset, Written), ContentError> {
        let hosted = PutAsset::hosted(target, put.path.clone());
        put.write_policy = hosted.write_policy;
        put.check_content_type = hosted.check_content_type;
        if normalize(&put.path)? != put.path || put.path.ends_with('/') {
            return Err(ContentError::BadRequest(format!(
                "the path {} is not a normalised file path",
                put.path
            )));
        }
        let head = &sample[..sample.len().min(mime::SAMPLE)];
        put.content_type = Some(store::content_type_of(&put, head)?);
        Ok((put, written))
    }
}

#[async_trait]
impl Transfer for GenericTransfer {
    async fn bundle(&self, source: &Ctx, component: &Component) -> Result<Bundle, ContentError> {
        if component.repository != source.name() {
            return Err(ContentError::not_found(format!(
                "the repository {} has no such component",
                source.name()
            )));
        }
        let assets = store::assets_of(source.db(), component.id).await?;
        Ok(Bundle {
            format: source.format_id().to_owned(),
            namespace: component.namespace.clone(),
            name: component.name.clone(),
            version: component.version.clone(),
            assets: assets
                .into_iter()
                .map(|asset| BundleAsset {
                    path: asset.path,
                    kind: asset.kind,
                    content_type: asset.content_type,
                    size: asset.size as u64,
                    sha1: asset.sha1,
                    sha256: asset.sha256,
                    sha512: asset.sha512,
                    md5: asset.md5,
                    attributes: asset.attributes,
                })
                .collect(),
            meta: Value::Object(Default::default()),
            origin: None,
        })
    }

    async fn receive(
        &self,
        target: &Ctx,
        bundle: &Bundle,
        blobs: &dyn BlobSource,
        terms: Terms,
    ) -> Result<Outcome, ContentError> {
        let wanted = self.wanted(target, bundle)?;
        if wanted.is_empty() {
            return Ok(Outcome::nothing());
        }
        if let Some(admit) = self.admit {
            admit(target, bundle).await?;
        }
        if terms.dry_run {
            return self.preview(target, bundle, &wanted, terms).await;
        }
        let sink = Sink::of(target).await?;
        let mut staged = Staged::new(&target.content);
        let items = match self
            .stage(target, bundle, blobs, &sink, &wanted, &mut staged)
            .await
        {
            Ok(items) => items,
            Err(error) => {
                staged.abandon().await;
                return Err(error);
            }
        };
        if items.is_empty() {
            return Ok(Outcome::nothing());
        }
        let outcome = store::record_many(target, items, terms.conflict, terms.action).await?;
        if outcome.status == Status::Copied
            && let Some(after) = self.after
            && let Err(error) = after(target).await
        {
            tracing::warn!(%error, repository = target.name(), "derived data of a transferred component was not refreshed");
        }
        Ok(outcome)
    }
}
