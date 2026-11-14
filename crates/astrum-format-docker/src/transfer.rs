use std::collections::{HashMap, HashSet};

use astrum_api::images::{Admit, Approve, ImageError, ImageItem, ImageReport, ImageTransfers};
use astrum_blob::{Digests, Written};
use astrum_content::import::{Described, Imported};
use astrum_content::store::{self, Asset, Component};
use astrum_content::transfer::{
    BlobSource, Bundle, BundleAsset, GenericTransfer, LocalSource, MAX_ASSETS, MAX_COMPONENTS,
    Outcome, Sink, Status, Terms, Transfer,
};
use astrum_content::{ContentError, Ctx, View};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use futures_util::future::BoxFuture;

use crate::import::{self, Stored};
use crate::manifest;
use crate::reference::{self, Digest, Reference};
use crate::storage::{self, BLOB, MANIFEST, TAG};
use crate::{ID, MANIFEST_LIMIT};

#[derive(Clone, Debug)]
pub struct Options {
    pub referrers: bool,
    pub target_tags: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            referrers: true,
            target_tags: Vec::new(),
        }
    }
}

const TAG_SUFFIXES: [&str; 4] = ["", ".sig", ".att", ".sbom"];

struct Origin {
    path: String,
    read: String,
}

pub struct DockerSource {
    local: LocalSource,
    source: Ctx,
    origins: HashMap<String, Origin>,
}

pub struct Prepared {
    bundle: Bundle,
    source: DockerSource,
    skipped: usize,
    root: Digest,
    untagged: bool,
}

impl Prepared {
    pub fn bundle(&self) -> &Bundle {
        &self.bundle
    }

    pub fn skipped_children(&self) -> usize {
        self.skipped
    }

    pub fn root(&self) -> String {
        self.root.to_string()
    }

    pub fn untagged(&self) -> bool {
        self.untagged
    }
}

#[async_trait]
impl BlobSource for DockerSource {
    async fn write(&self, sink: &Sink, asset: &BundleAsset) -> Result<Written, ContentError> {
        let origin = self.origins.get(&asset.path).ok_or_else(|| {
            ContentError::BadRequest(format!(
                "{} is not part of this transfer's source",
                asset.path
            ))
        })?;
        self.source.require(View::Read, &origin.read)?;
        let mut from = asset.clone();
        from.path = origin.path.clone();
        self.local.write(sink, &from).await
    }
}

fn describe_asset(asset: &Asset, path: String, kind: &str) -> Result<BundleAsset, ContentError> {
    let size = u64::try_from(asset.size).map_err(|_| {
        ContentError::internal(std::io::Error::other(format!(
            "{} has a negative size",
            asset.path
        )))
    })?;
    Ok(BundleAsset {
        path,
        kind: kind.to_owned(),
        content_type: asset.content_type.clone(),
        size,
        sha1: asset.sha1.clone(),
        sha256: asset.sha256.clone(),
        sha512: asset.sha512.clone(),
        md5: asset.md5.clone(),
        attributes: asset.attributes.clone(),
    })
}

struct Closure<'a> {
    source: &'a Ctx,
    image: &'a str,
    known: HashMap<Digest, Asset>,
    seen: HashSet<Digest>,
    order: Vec<Digest>,
    blob_seen: HashSet<Digest>,
    blobs: Vec<BundleAsset>,
    manifests: HashMap<Digest, BundleAsset>,
    tags: Vec<BundleAsset>,
    tag_paths: HashSet<String>,
    origins: HashMap<String, Origin>,
    skipped: usize,
}

impl<'a> Closure<'a> {
    fn new(source: &'a Ctx, image: &'a str) -> Self {
        Self {
            source,
            image,
            known: HashMap::new(),
            seen: HashSet::new(),
            order: Vec::new(),
            blob_seen: HashSet::new(),
            blobs: Vec::new(),
            manifests: HashMap::new(),
            tags: Vec::new(),
            tag_paths: HashSet::new(),
            origins: HashMap::new(),
            skipped: 0,
        }
    }

    fn guard(&self, assets: usize, tags: usize) -> Result<(), ContentError> {
        let tag_count = self.tags.len() + tags;
        if tag_count > MAX_COMPONENTS {
            return Err(ContentError::BadRequest(format!(
                "the image {} needs at least {tag_count} tags, more than the {MAX_COMPONENTS} allowed in one transfer",
                self.image
            )));
        }
        let stored = self.blobs.len() + self.manifests.len() + assets;
        if stored + tag_count > MAX_ASSETS {
            return Err(ContentError::BadRequest(format!(
                "the image {} needs at least {} assets ({stored} blobs and manifests, {tag_count} tags), more than the {MAX_ASSETS} one transfer takes",
                self.image,
                stored + tag_count,
            )));
        }
        Ok(())
    }

    async fn visit(&mut self, root: Digest, lenient: bool) -> Result<(), ContentError> {
        let mut stack = vec![(root.clone(), false)];
        while let Some((digest, expanded)) = stack.pop() {
            if expanded {
                self.order.push(digest);
                continue;
            }
            if !self.seen.insert(digest.clone()) {
                continue;
            }
            let skippable = lenient && digest != root;
            let Some(children) = self.load(&digest, skippable).await? else {
                self.seen.remove(&digest);
                self.skipped += 1;
                tracing::warn!(
                    repository = self.source.name(),
                    image = self.image,
                    %digest,
                    "skipping a child manifest that a tag-based index names but the repository lacks"
                );
                continue;
            };
            stack.push((digest, true));
            stack.extend(children.into_iter().rev().map(|child| (child, false)));
        }
        Ok(())
    }

    async fn load(
        &mut self,
        digest: &Digest,
        skippable: bool,
    ) -> Result<Option<Vec<Digest>>, ContentError> {
        let path = storage::manifest_path(self.image, &digest.to_string());
        self.source.require(View::Read, &path)?;
        let asset = match self.known.remove(digest) {
            Some(asset) => asset,
            None => match storage::find(self.source, &path).await? {
                Some(asset) => asset,
                None if skippable => return Ok(None),
                None => {
                    return Err(ContentError::not_found(format!(
                        "the manifest {digest} of {} is missing from the source repository {}",
                        self.image,
                        self.source.name()
                    )));
                }
            },
        };
        if asset.size > MANIFEST_LIMIT as i64 {
            return Err(ContentError::BadRequest(format!(
                "the manifest {digest} is larger than {MANIFEST_LIMIT} bytes"
            )));
        }
        let bytes = storage::read_all(self.source, &asset, MANIFEST_LIMIT).await?;
        if !digest.matches_bytes(&bytes) {
            return Err(ContentError::BadRequest(format!(
                "the stored manifest {digest} of {} does not match its digest",
                self.image
            )));
        }
        let parsed =
            manifest::parse(&bytes, Some(&storage::media_type_of(&asset))).map_err(|problem| {
                ContentError::BadRequest(format!("the manifest {digest} cannot be read: {problem}"))
            })?;
        for blob in &parsed.blobs {
            self.add_blob(digest, blob).await?;
        }
        self.guard(1, 0)?;
        self.manifests.insert(
            digest.clone(),
            describe_asset(&asset, path.clone(), MANIFEST)?,
        );
        self.origins.insert(
            path.clone(),
            Origin {
                path: asset.path.clone(),
                read: path,
            },
        );
        Ok(Some(parsed.children))
    }

    async fn add_blob(&mut self, manifest: &Digest, blob: &Digest) -> Result<(), ContentError> {
        if !self.blob_seen.insert(blob.clone()) {
            return Ok(());
        }
        let read = format!("/v2/{}/blobs/{blob}", self.image);
        self.source.require(View::Read, &read)?;
        let path = storage::blob_path(blob);
        let asset = storage::find(self.source, &path).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "the blob {blob} named by the manifest {manifest} is missing from the source repository {}",
                self.source.name()
            ))
        })?;
        self.guard(1, 0)?;
        self.blobs.push(describe_asset(&asset, path.clone(), BLOB)?);
        self.origins.insert(
            path,
            Origin {
                path: asset.path,
                read,
            },
        );
        Ok(())
    }

    async fn referrers(&mut self) -> Result<(), ContentError> {
        let mut index = 0;
        while index < self.order.len() {
            let subject = self.order[index].clone();
            index += 1;
            let found =
                storage::referrer_digests(self.source, self.image, &subject, MAX_ASSETS).await?;
            let mut listed = HashSet::new();
            let pending: Vec<Digest> = found
                .into_iter()
                .filter(|digest| !self.seen.contains(digest) && listed.insert(digest.clone()))
                .collect();
            self.guard(pending.len(), 0)?;
            for digest in pending {
                self.visit(digest, false).await?;
            }
            self.tag_referrers(&subject).await?;
        }
        Ok(())
    }

    async fn tag_referrers(&mut self, subject: &Digest) -> Result<(), ContentError> {
        let schema = subject.referrers_tag();
        for suffix in TAG_SUFFIXES {
            let path = storage::manifest_path(self.image, &format!("{schema}{suffix}"));
            let Some(found) = storage::find(self.source, &path).await? else {
                continue;
            };
            self.source.require(View::Read, &path)?;
            let digest = tag_digest(&found)?;
            if !self.seen.contains(&digest) {
                let by_digest = storage::manifest_path(self.image, &digest.to_string());
                let loaded = match storage::find(self.source, &by_digest).await? {
                    Some(stored) => stored,
                    None => found.clone(),
                };
                self.known.insert(digest.clone(), loaded);
                self.visit(digest, true).await?;
            }
            self.add_tag(&path, &found, &path)?;
        }
        Ok(())
    }

    fn add_tag(&mut self, path: &str, from: &Asset, read: &str) -> Result<(), ContentError> {
        if !self.tag_paths.insert(path.to_owned()) {
            return Err(ContentError::BadRequest(format!(
                "{path} would be written twice in one transfer"
            )));
        }
        self.guard(0, 1)?;
        self.tags.push(describe_asset(from, path.to_owned(), TAG)?);
        self.origins.insert(
            path.to_owned(),
            Origin {
                path: from.path.clone(),
                read: read.to_owned(),
            },
        );
        Ok(())
    }
}

fn tag_digest(asset: &Asset) -> Result<Digest, ContentError> {
    storage::digest_of(asset)
        .or_else(|| Digest::parse(&format!("sha256:{}", asset.sha256)))
        .ok_or_else(|| {
            ContentError::internal(std::io::Error::other(format!(
                "{} names no digest",
                asset.path
            )))
        })
}

pub async fn prepare(
    source: &Ctx,
    image: &str,
    reference: &str,
    options: &Options,
) -> Result<Prepared, ContentError> {
    if source.format_id() != ID || source.kind() != RepositoryType::Hosted {
        return Err(ContentError::BadRequest(format!(
            "{} is not a hosted Docker repository",
            source.name()
        )));
    }
    if !reference::valid_name(image) {
        return Err(ContentError::BadRequest(format!(
            "{image} is not a valid image name"
        )));
    }
    let reference = Reference::parse(reference).ok_or_else(|| {
        ContentError::BadRequest(format!("{reference} is neither a tag nor a digest"))
    })?;
    for tag in &options.target_tags {
        if let Some(problem) = reference::tag_problem(tag) {
            return Err(ContentError::BadRequest(problem));
        }
    }
    let requested = storage::manifest_path(image, &reference.as_str());
    source.require(View::Read, &requested)?;
    let found = storage::find(source, &requested).await?.ok_or_else(|| {
        ContentError::not_found(format!("{}{requested} does not exist", source.name()))
    })?;
    let (root, kept) = match &reference {
        Reference::Digest(digest) => (digest.clone(), None),
        Reference::Tag(tag) => (tag_digest(&found)?, Some(tag.clone())),
    };
    let by_digest = storage::manifest_path(image, &root.to_string());
    let root_asset = match &reference {
        Reference::Digest(_) => found,
        Reference::Tag(_) => storage::find(source, &by_digest).await?.unwrap_or(found),
    };
    let mut closure = Closure::new(source, image);
    closure.known.insert(root.clone(), root_asset.clone());
    closure.visit(root.clone(), false).await?;
    let names: Vec<String> = match options.target_tags.is_empty() {
        true => kept.into_iter().collect(),
        false => {
            let mut seen = HashSet::new();
            options
                .target_tags
                .iter()
                .filter(|tag| seen.insert(tag.as_str()))
                .cloned()
                .collect()
        }
    };
    let untagged = names.is_empty();
    for tag in &names {
        let path = storage::manifest_path(image, tag);
        closure.add_tag(&path, &root_asset, &by_digest)?;
    }
    if options.referrers {
        closure.referrers().await?;
    }
    let Closure {
        blobs,
        mut manifests,
        order,
        tags,
        origins,
        skipped,
        ..
    } = closure;
    let mut assets = blobs;
    for digest in &order {
        assets.extend(manifests.remove(digest));
    }
    assets.extend(tags);
    let bundle = Bundle {
        format: ID.to_owned(),
        namespace: String::new(),
        name: image.to_owned(),
        version: names.first().cloned().unwrap_or_else(|| root.to_string()),
        assets,
        meta: serde_json::json!({"referrers": options.referrers}),
        origin: None,
    };
    Ok(Prepared {
        bundle,
        source: DockerSource {
            local: LocalSource::new(source.clone()),
            source: source.clone(),
            origins,
        },
        skipped,
        root,
        untagged,
    })
}

fn blob_digests(asset: &BundleAsset) -> Digests {
    Digests {
        sha1: asset.sha1.to_ascii_lowercase(),
        sha256: asset.sha256.to_ascii_lowercase(),
        sha512: asset.sha512.to_ascii_lowercase(),
        md5: asset.md5.to_ascii_lowercase(),
    }
}

fn check(bundle: &Bundle) -> Result<(), ContentError> {
    if !reference::valid_name(&bundle.name) {
        return Err(ContentError::BadRequest(format!(
            "{} is not a valid image name",
            bundle.name
        )));
    }
    let mut manifests = 0;
    for asset in &bundle.assets {
        match import::classify(&asset.path) {
            Some(Stored::Blob(digest)) => {
                if !digest.matches(&blob_digests(asset)) {
                    return Err(ContentError::BadRequest(format!(
                        "{} does not hold the content its digest names",
                        asset.path
                    )));
                }
            }
            None => {}
            Some(Stored::Manifest { image, reference }) => {
                if image != bundle.name {
                    return Err(ContentError::BadRequest(format!(
                        "{} belongs to another image than {}",
                        asset.path, bundle.name
                    )));
                }
                if asset.size > MANIFEST_LIMIT as u64 {
                    return Err(ContentError::BadRequest(format!(
                        "the manifest {} is larger than {MANIFEST_LIMIT} bytes",
                        asset.path
                    )));
                }
                if Digest::parse(&reference).is_none()
                    && let Some(problem) = reference::tag_problem(&reference)
                {
                    return Err(ContentError::BadRequest(problem));
                }
                manifests += 1;
            }
        }
    }
    if manifests == 0 {
        return Err(ContentError::BadRequest(
            "a Docker bundle holds at least one manifest".to_owned(),
        ));
    }
    Ok(())
}

fn admit<'a>(_target: &'a Ctx, bundle: &'a Bundle) -> BoxFuture<'a, Result<(), ContentError>> {
    Box::pin(async move { check(bundle) })
}

fn replaceable<'a>(target: &'a Ctx, path: &'a str, asset: &'a BundleAsset) -> bool {
    asset.kind == TAG
        && match import::classify(path) {
            Some(Stored::Manifest { reference, .. }) => {
                Digest::parse(&reference).is_none() && storage::may_move_tag(target, &reference)
            }
            _ => false,
        }
}

fn describe_imported(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(import::describe(file))
}

pub struct DockerTransfer {
    generic: GenericTransfer,
}

pub static TRANSFER: DockerTransfer = DockerTransfer {
    generic: GenericTransfer {
        plan: import::plan,
        describe: describe_imported,
        after: None,
        admit: Some(admit),
        replaceable: Some(replaceable),
        check_content_type: false,
        carried: &[],
        bundle_skip: None,
    },
};

#[async_trait]
impl Transfer for DockerTransfer {
    async fn bundle(&self, source: &Ctx, component: &Component) -> Result<Bundle, ContentError> {
        if component.repository != source.name() {
            return Err(ContentError::not_found(format!(
                "the repository {} has no such component",
                source.name()
            )));
        }
        let prepared = prepare(
            source,
            &component.name,
            &component.version,
            &Options::default(),
        )
        .await?;
        Ok(prepared.bundle)
    }

    async fn receive(
        &self,
        _target: &Ctx,
        _bundle: &Bundle,
        _blobs: &dyn BlobSource,
        _terms: Terms,
    ) -> Result<Outcome, ContentError> {
        Err(ContentError::BadRequest(
            "docker bundles must come from prepare".to_owned(),
        ))
    }
}

impl DockerTransfer {
    pub async fn receive(
        &self,
        target: &Ctx,
        prepared: &Prepared,
        terms: Terms,
    ) -> Result<Outcome, ContentError> {
        self.generic
            .receive(target, &prepared.bundle, &prepared.source, terms)
            .await
    }
}

pub struct Images;

fn rights_path(image: &str, path: &str) -> String {
    match path.strip_prefix(storage::BLOB_PATH_PREFIX) {
        Some(digest) => format!("/v2/{image}/blobs/{digest}"),
        None => path.to_owned(),
    }
}

fn rights_view(image: &str, outcome: &Outcome) -> Outcome {
    let mut view = outcome.clone();
    for asset in &mut view.assets {
        asset.path = rights_path(image, &asset.path);
    }
    view
}

#[async_trait]
impl ImageTransfers for Images {
    async fn transfer(
        &self,
        source: &Ctx,
        target: &Ctx,
        item: &ImageItem,
        terms: Terms,
        admit: Admit<'_>,
        approve: Approve<'_>,
    ) -> Result<ImageReport, ImageError> {
        let options = Options {
            referrers: item.referrers,
            target_tags: item.target_tags.clone(),
        };
        let prepared = prepare(source, &item.image, &item.reference, &options)
            .await
            .map_err(ImageError::Plan)?;
        let image = prepared.bundle().name.clone();
        let paths: Vec<String> = prepared
            .bundle()
            .assets
            .iter()
            .map(|asset| rights_path(&image, &asset.path))
            .collect();
        admit(&paths).map_err(ImageError::Denied)?;
        let planned = TRANSFER
            .receive(
                target,
                &prepared,
                Terms {
                    dry_run: true,
                    ..terms
                },
            )
            .await
            .map_err(ImageError::Plan)?;
        let conflict = approve(&rights_view(&image, &planned)).map_err(ImageError::Denied)?;
        let outcome = match terms.dry_run || planned.status != Status::Copied {
            true => planned,
            false => TRANSFER
                .receive(
                    target,
                    &prepared,
                    Terms {
                        conflict,
                        dry_run: false,
                        ..terms
                    },
                )
                .await
                .map_err(ImageError::Run)?,
        };
        Ok(ImageReport {
            outcome,
            skipped_children: prepared.skipped_children(),
            untagged: prepared.untagged(),
            version: prepared.bundle().version.clone(),
            digest: prepared.root(),
        })
    }

    async fn remove_tag(
        &self,
        source: &Ctx,
        image: &str,
        tag: &str,
        digest: &str,
    ) -> Result<(), ContentError> {
        if source.format_id() != ID || source.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(format!(
                "{} is not a hosted Docker repository",
                source.name()
            )));
        }
        if let Some(problem) = reference::tag_problem(tag) {
            return Err(ContentError::BadRequest(problem));
        }
        let path = storage::manifest_path(image, tag);
        source.require(View::Delete, &path)?;
        let found = storage::find(source, &path).await?.ok_or_else(|| {
            ContentError::not_found(format!("{}{path} is already gone", source.name()))
        })?;
        if tag_digest(&found)?.to_string() != digest {
            return Err(ContentError::Conflict(format!(
                "{}{path} points at another image than the one that was copied",
                source.name()
            )));
        }
        store::delete_asset_if(&source.content, &found).await
    }
}
