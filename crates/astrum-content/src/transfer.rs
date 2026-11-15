use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use astrum_blob::{BlobId, BlobStore, Digests, Written, content_of};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::import::{Body, Described, Imported, ImportedComponent, Plan};
use crate::store::{
    self, Asset, Component, ComponentIdentity, ComponentKey, Intent, Planned, PutAsset,
};
use crate::{Content, ContentError, Ctx, holds, mime, normalize};

pub const MAX_ASSETS: usize = 10_000;
pub const MAX_COMPONENTS: usize = 100;
pub const PROMOTED: &str = "promoted";
pub const REPLICATED: &str = "replicated";
pub const REPLICATION_ATTRIBUTE: &str = "replication";
pub const REPLICATION_VIA: &str = "replication";
const FINGERPRINT_VERSION: &str = "astrum-component-fingerprint-1";
const HOLD_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_TEXT: usize = 4096;
const MAX_META_BYTES: usize = 64 * 1024;
const MAX_META_DEPTH: usize = 8;
const RESERVED_ATTRIBUTES: &[&str] = &["replication"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Conflict {
    #[default]
    Fail,
    Skip,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Copied,
    Unchanged,
    Skipped,
    Conflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetStatus {
    New,
    Replaced,
    Unchanged,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetOutcome {
    pub path: String,
    pub status: AssetStatus,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub status: Status,
    pub component_id: Option<i64>,
    pub component_ids: Vec<i64>,
    pub assets: Vec<AssetOutcome>,
    pub bytes_copied: u64,
    pub bytes_reused: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromotedComponent {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Promoted {
    pub components: Vec<PromotedComponent>,
    pub loose: Vec<String>,
}

impl Outcome {
    fn nothing() -> Self {
        Self {
            status: Status::Unchanged,
            component_id: None,
            component_ids: Vec::new(),
            assets: Vec::new(),
            bytes_copied: 0,
            bytes_reused: 0,
            warnings: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Terms {
    pub conflict: Conflict,
    pub action: &'static str,
    pub dry_run: bool,
    pub dist_tags: bool,
}

impl Terms {
    pub fn new(conflict: Conflict, action: &'static str) -> Self {
        Self {
            conflict,
            action,
            dry_run: false,
            dist_tags: true,
        }
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn with_dist_tags(mut self, dist_tags: bool) -> Self {
        self.dist_tags = dist_tags;
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

fn field(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn canonical(hasher: &mut Sha256, value: &Value) {
    match value {
        Value::Null => hasher.update(b"n"),
        Value::Bool(flag) => hasher.update(if *flag { b"t" } else { b"f" }),
        Value::Number(number) => {
            hasher.update(b"d");
            field(hasher, &number.to_string());
        }
        Value::String(text) => {
            hasher.update(b"s");
            field(hasher, text);
        }
        Value::Array(items) => {
            hasher.update(b"[");
            hasher.update((items.len() as u64).to_be_bytes());
            for item in items {
                canonical(hasher, item);
            }
        }
        Value::Object(members) => {
            hasher.update(b"{");
            hasher.update((members.len() as u64).to_be_bytes());
            let mut keys: Vec<&String> = members.keys().collect();
            keys.sort();
            for key in keys {
                field(hasher, key);
                canonical(hasher, &members[key]);
            }
        }
    }
}

pub fn fingerprint(bundle: &Bundle) -> String {
    let mut hasher = Sha256::new();
    field(&mut hasher, FINGERPRINT_VERSION);
    for part in [
        &bundle.format,
        &bundle.namespace,
        &bundle.name,
        &bundle.version,
    ] {
        field(&mut hasher, part);
    }
    let mut assets: Vec<&BundleAsset> = bundle.assets.iter().collect();
    assets.sort_by(|a, b| a.path.cmp(&b.path));
    hasher.update((assets.len() as u64).to_be_bytes());
    for asset in assets {
        field(&mut hasher, &asset.path);
        field(&mut hasher, &asset.sha256.to_ascii_lowercase());
        hasher.update(asset.size.to_be_bytes());
    }
    match &bundle.meta {
        Value::Object(members) => {
            let kept: serde_json::Map<String, Value> = members
                .iter()
                .filter(|(key, _)| key.as_str() != REPLICATION_ATTRIBUTE)
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            canonical(&mut hasher, &Value::Object(kept));
        }
        other => canonical(&mut hasher, other),
    }
    hex::encode(hasher.finalize())
}

pub fn origin_of(attributes: &Value) -> Option<&str> {
    attributes
        .get(REPLICATION_ATTRIBUTE)?
        .get("origin")?
        .as_str()
}

fn mark(put: &mut PutAsset, origin: Option<&str>) -> Result<(), ContentError> {
    let (Some(origin), Some(component)) = (origin, put.component.as_mut()) else {
        return Ok(());
    };
    if component.attributes.is_null() {
        component.attributes = Value::Object(Default::default());
    }
    let Value::Object(attributes) = &mut component.attributes else {
        return Err(ContentError::internal(anyhow::anyhow!(
            "the component attributes of {} are not an object",
            put.path
        )));
    };
    attributes.insert(
        REPLICATION_ATTRIBUTE.to_owned(),
        serde_json::json!({"origin": origin, "via": REPLICATION_VIA}),
    );
    Ok(())
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
                "the content of {}{} is missing from the storage of the source repository",
                self.source.name(),
                asset.path
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

    fn bundle_skip(&self) -> Option<SkipFn> {
        None
    }

    fn replicable(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

pub type PlanFn = fn(RepositoryType, &str, &str) -> Plan;
pub type DescribeFn = for<'a> fn(&'a Imported) -> BoxFuture<'a, Result<Option<Described>, String>>;
pub type AfterFn = for<'a> fn(&'a Ctx) -> BoxFuture<'a, Result<(), ContentError>>;
pub type AfterHook = Arc<
    dyn for<'a> Fn(&'a Ctx, &'a Promoted) -> BoxFuture<'a, Result<(), ContentError>> + Send + Sync,
>;
pub type ReplaceFn = for<'a> fn(&'a Ctx, &'a str, &'a BundleAsset) -> bool;
pub type SkipFn = fn(&Asset) -> bool;
pub type AdmitFn = for<'a> fn(&'a Ctx, &'a Bundle) -> BoxFuture<'a, Result<(), ContentError>>;

pub struct GenericTransfer {
    pub plan: PlanFn,
    pub describe: DescribeFn,
    pub after: Option<AfterHook>,
    pub admit: Option<AdmitFn>,
    pub replaceable: Option<ReplaceFn>,
    pub check_content_type: bool,
    pub carried: &'static [&'static str],
    pub bundle_skip: Option<SkipFn>,
}

impl GenericTransfer {
    pub fn after_with<F>(mut self, hook: F) -> Self
    where
        F: for<'a> Fn(&'a Ctx, &'a Promoted) -> BoxFuture<'a, Result<(), ContentError>>
            + Send
            + Sync
            + 'static,
    {
        self.after = Some(Arc::new(hook));
        self
    }

    pub fn after_fn(self, after: AfterFn) -> Self {
        self.after_with(move |ctx, _| after(ctx))
    }
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

fn strictness(policy: Option<WritePolicy>) -> u8 {
    match policy {
        None | Some(WritePolicy::Allow) => 0,
        Some(WritePolicy::AllowOnce) => 1,
        Some(WritePolicy::Deny) => 2,
    }
}

fn stricter(target: Option<WritePolicy>, described: Option<WritePolicy>) -> Option<WritePolicy> {
    match strictness(described) > strictness(target) {
        true => described,
        false => target,
    }
}

type Placed = (String, Option<ComponentKey>);

fn promoted(placed: &[Placed], outcome: &Outcome, refreshed: &[ComponentIdentity]) -> Promoted {
    let written: HashSet<&str> = outcome
        .assets
        .iter()
        .filter(|asset| matches!(asset.status, AssetStatus::New | AssetStatus::Replaced))
        .map(|asset| asset.path.as_str())
        .collect();
    let mut promoted = Promoted::default();
    for (path, component) in placed {
        if !written.contains(path.as_str()) {
            continue;
        }
        if let Some(key) = component
            && refreshed.contains(&(key.namespace.clone(), key.name.clone(), key.version.clone()))
        {
            continue;
        }
        let Some(key) = component else {
            promoted.loose.push(path.clone());
            continue;
        };
        let position = promoted.components.iter().position(|found| {
            found.namespace == key.namespace
                && found.name == key.name
                && found.version == key.version
        });
        match position {
            Some(index) => promoted.components[index].paths.push(path.clone()),
            None => promoted.components.push(PromotedComponent {
                namespace: key.namespace.clone(),
                name: key.name.clone(),
                version: key.version.clone(),
                paths: vec![path.clone()],
            }),
        }
    }
    for (namespace, name, version) in refreshed {
        let listed = promoted.components.iter().any(|found| {
            found.namespace == *namespace && found.name == *name && found.version == *version
        });
        if !listed {
            promoted.components.push(PromotedComponent {
                namespace: namespace.clone(),
                name: name.clone(),
                version: version.clone(),
                paths: Vec::new(),
            });
        }
    }
    promoted
}

fn within_depth(value: &Value, level: usize) -> bool {
    match value {
        Value::Object(members) => {
            level <= MAX_META_DEPTH && members.values().all(|inner| within_depth(inner, level + 1))
        }
        Value::Array(items) => {
            level <= MAX_META_DEPTH && items.iter().all(|inner| within_depth(inner, level + 1))
        }
        _ => true,
    }
}

fn plain(text: &str, keys: bool) -> bool {
    text.chars().all(|found| {
        let tolerated = !keys && matches!(found, '\t' | '\n');
        !(found.is_control() && !tolerated)
    })
}

fn clean(value: &Value) -> bool {
    match value {
        Value::String(found) => plain(found, false),
        Value::Array(items) => items.iter().all(clean),
        Value::Object(members) => members
            .iter()
            .all(|(key, inner)| plain(key, true) && clean(inner)),
        _ => true,
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

struct Held {
    store: Arc<dyn BlobStore>,
    blob: BlobId,
    size: u64,
    sha256: String,
}

fn imported_of(target: &Ctx, incoming: &Incoming<'_>, asset: &BundleAsset, held: Held) -> Imported {
    Imported {
        repository: RepositoryType::Hosted,
        config: target.repository.clone(),
        path: asset.path.clone(),
        kind: asset.kind.clone(),
        content_type: asset.content_type.clone(),
        attributes: asset.attributes.clone(),
        component: Some(ImportedComponent {
            namespace: incoming.bundle.namespace.clone(),
            name: incoming.bundle.name.clone(),
            version: incoming.bundle.version.clone(),
            attributes: match incoming.carried.is_empty() {
                true => Value::Null,
                false => Value::Object(incoming.carried.clone()),
            },
            ..Default::default()
        }),
        store: held.store,
        blob: held.blob,
        size: held.size,
        sha256: held.sha256,
    }
}

fn same_component(left: &ComponentKey, right: &ComponentKey) -> bool {
    left.namespace == right.namespace && left.name == right.name && left.version == right.version
}

struct Incoming<'a> {
    bundle: &'a Bundle,
    carried: &'a Map<String, Value>,
}

type Transferred<'a> = (
    Vec<(PutAsset, Written)>,
    HashMap<String, &'a BundleAsset>,
    Vec<ComponentIdentity>,
);

impl GenericTransfer {
    fn declared(&self) -> Vec<&'static str> {
        self.carried
            .iter()
            .copied()
            .filter(|name| !RESERVED_ATTRIBUTES.contains(name))
            .collect()
    }

    fn carried_meta(&self, bundle: &Bundle) -> Result<Map<String, Value>, ContentError> {
        let members = match &bundle.meta {
            Value::Null => return Ok(Map::new()),
            Value::Object(members) => members,
            _ => {
                return Err(ContentError::BadRequest(
                    "the bundle meta is not an object".to_owned(),
                ));
            }
        };
        let size = serde_json::to_vec(&bundle.meta)
            .map_err(ContentError::internal)?
            .len();
        if size > MAX_META_BYTES {
            return Err(ContentError::BadRequest(format!(
                "the bundle meta is larger than {MAX_META_BYTES} bytes"
            )));
        }
        if !within_depth(&bundle.meta, 1) {
            return Err(ContentError::BadRequest(format!(
                "the bundle meta is nested deeper than {MAX_META_DEPTH} levels"
            )));
        }
        if !clean(&bundle.meta) {
            return Err(ContentError::BadRequest(
                "the bundle meta holds control characters".to_owned(),
            ));
        }
        let mut carried = Map::new();
        for name in self.declared() {
            match members.get(name) {
                None | Some(Value::Null) => {}
                Some(Value::Array(_)) => {
                    return Err(ContentError::BadRequest(format!(
                        "the bundle meta {name} is neither an object nor a scalar"
                    )));
                }
                Some(value) => {
                    carried.insert(name.to_owned(), value.clone());
                }
            }
        }
        Ok(carried)
    }

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
        intents: &[Intent],
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
        let origins: HashMap<String, &BundleAsset> = wanted
            .iter()
            .map(|asset| (asset.path.clone(), *asset))
            .collect();
        let allowed = self.allowance(target, origins);
        store::preview_many_carrying(target, &planned, terms.conflict, Some(&allowed), intents)
            .await
    }

    async fn intents(
        &self,
        target: &Ctx,
        incoming: &Incoming<'_>,
        wanted: &[&BundleAsset],
    ) -> Result<Vec<Intent>, ContentError> {
        if incoming.carried.is_empty() {
            return Ok(Vec::new());
        }
        let sink = Sink::of(target).await?;
        let mut primaries: Vec<ComponentKey> = Vec::new();
        for asset in wanted {
            let held = Held {
                store: sink.store.clone(),
                blob: BlobId::from(uuid::Uuid::nil()),
                size: asset.size,
                sha256: asset.sha256.to_ascii_lowercase(),
            };
            let imported = imported_of(target, incoming, asset, held);
            let Some(described) = (self.describe)(&imported)
                .await
                .map_err(ContentError::BadRequest)?
            else {
                continue;
            };
            let Some(key) = described.put.component else {
                continue;
            };
            let seen = primaries.iter().any(|found| same_component(found, &key));
            if !described.keep_component && !seen {
                primaries.push(key);
            }
        }
        Ok(primaries
            .into_iter()
            .map(|key| Intent {
                attributes: incoming
                    .carried
                    .iter()
                    .filter_map(|(name, _)| {
                        key.attributes
                            .get(name)
                            .filter(|value| !value.is_null())
                            .map(|value| (name.clone(), value.clone()))
                    })
                    .collect(),
                identity: (key.namespace, key.name, key.version),
                companion_only: false,
            })
            .collect())
    }

    fn allowance<'a>(
        &self,
        target: &'a Ctx,
        origins: HashMap<String, &'a BundleAsset>,
    ) -> impl Fn(&str) -> bool + Send + Sync + 'a {
        let hook = self.replaceable;
        move |path: &str| {
            hook.is_some_and(|hook| {
                origins
                    .get(path)
                    .is_some_and(|asset| hook(target, path, asset))
            })
        }
    }

    async fn stage<'a>(
        &self,
        target: &Ctx,
        incoming: &Incoming<'_>,
        blobs: &dyn BlobSource,
        sink: &Sink,
        wanted: &[&'a BundleAsset],
        staged: &mut Staged<'_>,
    ) -> Result<Transferred<'a>, ContentError> {
        let mut items: Vec<(PutAsset, Written)> = Vec::new();
        let mut keeping: Vec<bool> = Vec::new();
        let mut origins: HashMap<String, &'a BundleAsset> = HashMap::new();
        let mut components: HashSet<(String, String, String)> = HashSet::new();
        for asset in wanted {
            staged.check()?;
            let written = blobs.write(sink, asset).await?;
            staged.hold(written.clone()).await?;
            verify(&written, asset)?;
            let held = Held {
                store: sink.store.clone(),
                blob: written.id,
                size: written.size,
                sha256: written.digests.sha256.clone(),
            };
            let imported = imported_of(target, incoming, asset, held);
            let Some(described) = (self.describe)(&imported)
                .await
                .map_err(ContentError::BadRequest)?
            else {
                staged.drop_blob(&written).await;
                continue;
            };
            let Described {
                put,
                body,
                extra,
                keep_component,
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
            let mut item = Self::finish(target, put, primary, &sample, self.check_content_type)?;
            mark(&mut item.0, incoming.bundle.origin.as_deref())?;
            Self::count_component(&mut components, &item.0)?;
            origins.insert(item.0.path.clone(), *asset);
            items.push(item);
            keeping.push(keep_component);
            for (put, bytes) in extra {
                staged.check()?;
                let fresh = Self::write_bytes(sink, staged, &bytes).await?;
                let mut item = Self::finish(target, put, fresh, &bytes, self.check_content_type)?;
                mark(&mut item.0, incoming.bundle.origin.as_deref())?;
                Self::count_component(&mut components, &item.0)?;
                origins.insert(item.0.path.clone(), *asset);
                items.push(item);
                keeping.push(keep_component);
            }
        }
        staged.check()?;
        let companions = Self::primary_first(&mut items, &keeping);
        Ok((items, origins, companions))
    }

    fn primary_first(
        items: &mut [(PutAsset, Written)],
        keeping: &[bool],
    ) -> Vec<ComponentIdentity> {
        let mut primaries: HashMap<ComponentIdentity, ComponentKey> = HashMap::new();
        let mut companions: Vec<ComponentIdentity> = Vec::new();
        for pass in [false, true] {
            for ((put, _), keep) in items.iter().zip(keeping) {
                if *keep != pass {
                    continue;
                }
                if let Some(found) = put.component.as_ref() {
                    let identity = (
                        found.namespace.clone(),
                        found.name.clone(),
                        found.version.clone(),
                    );
                    if pass && !primaries.contains_key(&identity) {
                        companions.push(identity.clone());
                    }
                    primaries.entry(identity).or_insert_with(|| found.clone());
                }
            }
        }
        for (put, _) in items.iter_mut() {
            let Some(found) = put.component.as_ref() else {
                continue;
            };
            let identity = (
                found.namespace.clone(),
                found.name.clone(),
                found.version.clone(),
            );
            if let Some(primary) = primaries.get(&identity) {
                put.component = Some(primary.clone());
            }
        }
        companions
    }

    fn count_component(
        components: &mut HashSet<(String, String, String)>,
        put: &PutAsset,
    ) -> Result<(), ContentError> {
        if let Some(found) = put.component.as_ref() {
            components.insert((
                found.namespace.clone(),
                found.name.clone(),
                found.version.clone(),
            ));
        }
        match components.len() > MAX_COMPONENTS {
            true => Err(ContentError::BadRequest(format!(
                "one transfer takes at most {MAX_COMPONENTS} components"
            ))),
            false => Ok(()),
        }
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
        check_content_type: bool,
    ) -> Result<(PutAsset, Written), ContentError> {
        let hosted = PutAsset::hosted(target, put.path.clone());
        put.write_policy = stricter(hosted.write_policy, put.write_policy);
        put.check_content_type =
            check_content_type && hosted.check_content_type && put.check_content_type;
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
    fn bundle_skip(&self) -> Option<SkipFn> {
        self.bundle_skip
    }

    async fn bundle(&self, source: &Ctx, component: &Component) -> Result<Bundle, ContentError> {
        if component.repository != source.name() {
            return Err(ContentError::not_found(format!(
                "the repository {} has no such component",
                source.name()
            )));
        }
        let assets = store::assets_of(source.db(), component.id).await?;
        let meta: Map<String, Value> = self
            .declared()
            .into_iter()
            .filter_map(|name| {
                component
                    .attributes
                    .get(name)
                    .filter(|value| !value.is_null())
                    .map(|value| (name.to_owned(), value.clone()))
            })
            .collect();
        Ok(Bundle {
            format: source.format_id().to_owned(),
            namespace: component.namespace.clone(),
            name: component.name.clone(),
            version: component.version.clone(),
            assets: assets
                .into_iter()
                .filter(|asset| !self.bundle_skip.is_some_and(|skip| skip(asset)))
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
            meta: Value::Object(meta),
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
        let carried = self.carried_meta(bundle)?;
        if wanted.is_empty() {
            return Ok(Outcome::nothing());
        }
        if let Some(admit) = self.admit {
            admit(target, bundle).await?;
        }
        let incoming = Incoming {
            bundle,
            carried: &carried,
        };
        let intents = self.intents(target, &incoming, &wanted).await?;
        if terms.dry_run {
            return self.preview(target, bundle, &wanted, terms, &intents).await;
        }
        let sink = Sink::of(target).await?;
        let mut staged = Staged::new(&target.content);
        let (items, origins, companions) = match self
            .stage(target, &incoming, blobs, &sink, &wanted, &mut staged)
            .await
        {
            Ok(done) => done,
            Err(error) => {
                staged.abandon().await;
                return Err(error);
            }
        };
        if items.is_empty() {
            return Ok(Outcome::nothing());
        }
        let mut intents = intents;
        intents.extend(companions.into_iter().map(|identity| Intent {
            identity,
            attributes: Map::new(),
            companion_only: true,
        }));
        let allowed = self.allowance(target, origins);
        let placed: Vec<Placed> = items
            .iter()
            .map(|(put, _)| (put.path.clone(), put.component.clone()))
            .collect();
        let (mut outcome, refreshed) = store::record_many_carrying(
            target,
            items,
            terms.conflict,
            terms.action,
            Some(&allowed),
            &intents,
        )
        .await?;
        if outcome.status == Status::Copied
            && let Some(after) = &self.after
        {
            let promoted = promoted(&placed, &outcome, &refreshed);
            if let Err(error) = after(target, &promoted).await {
                tracing::warn!(%error, repository = target.name(), "derived data of a transferred component was not refreshed");
                outcome
                    .warnings
                    .push(format!("derived data was not refreshed: {error}"));
            }
        }
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn asset(path: &str, sha256: &str, size: u64) -> BundleAsset {
        BundleAsset {
            path: path.to_owned(),
            kind: "asset".to_owned(),
            content_type: "text/plain".to_owned(),
            size,
            sha1: String::new(),
            sha256: sha256.to_owned(),
            sha512: String::new(),
            md5: String::new(),
            attributes: json!({"seen": 1}),
        }
    }

    fn bundle() -> Bundle {
        Bundle {
            format: "raw".to_owned(),
            namespace: "/d".to_owned(),
            name: "/d/a".to_owned(),
            version: String::new(),
            assets: vec![
                asset("/d/a", &"a".repeat(64), 3),
                asset("/d/b", &"b".repeat(64), 4),
            ],
            meta: json!({}),
            origin: None,
        }
    }

    #[test]
    fn a_fingerprint_follows_the_content_and_nothing_else() {
        let base = fingerprint(&bundle());
        assert_eq!(base.len(), 64);

        let mut same = bundle();
        same.assets.reverse();
        same.assets[0].sha256 = "B".repeat(64);
        same.assets[1].attributes = json!({"seen": 2});
        same.assets[1].content_type = "other/type".to_owned();
        same.origin = Some("someone".to_owned());
        same.meta = json!({"replication": {"origin": "x"}});
        assert_eq!(fingerprint(&same), base);

        let changes: [fn(&mut Bundle); 6] = [
            |b| b.namespace = "/e".to_owned(),
            |b| b.name = "/d/c".to_owned(),
            |b| b.version = "1".to_owned(),
            |b| b.assets[0].sha256 = "c".repeat(64),
            |b| b.assets[1].size = 5,
            |b| b.meta = json!({"tags": ["latest"]}),
        ];
        for change in changes {
            let mut other = bundle();
            change(&mut other);
            assert_ne!(fingerprint(&other), base);
        }
        let mut fewer = bundle();
        fewer.assets.pop();
        assert_ne!(fingerprint(&fewer), base);
    }
}
