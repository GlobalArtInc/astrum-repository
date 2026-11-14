mod purl;
pub(crate) mod requeue;

use std::collections::{HashMap, HashSet};

use astrum_content::inventory::{Inventory, PackageRef, Source};
use astrum_content::store::{Asset, Component};
use astrum_content::{ContentError, Ctx};
use astrum_sbom::Sbom;
use serde::Deserialize;
use serde_json::Value;

use crate::MANIFEST_LIMIT;
use crate::children;
use crate::manifest;
use crate::reference::Digest;
use crate::sbom::{self, Access};
use crate::storage;
use crate::supply::Artifact;

pub const MAX_PACKAGES: usize = astrum_sbom::MAX_PACKAGES;
pub const MAX_CHILDREN: usize = 50;
const MAX_ARTIFACTS: usize = 8;
const MAX_PARENTS: u64 = 16;
const REFERENCE_TYPE: &str = "vnd.docker.reference.type";
const REFERENCE_DIGEST: &str = "vnd.docker.reference.digest";
const ATTESTATION_MANIFEST: &str = "attestation-manifest";
const ARTIFACT_KINDS: [&str; 3] = ["signature", "sbom", "attestation"];
const DOCUMENT_KINDS: [&str; 2] = ["sbom", "attestation"];

pub fn key(digest: &Digest) -> String {
    format!("oci:{digest}")
}

struct Collector {
    seen: HashSet<(String, String, String)>,
    packages: Vec<PackageRef>,
    complete: bool,
}

impl Collector {
    fn new() -> Self {
        Self {
            seen: HashSet::new(),
            packages: Vec::new(),
            complete: true,
        }
    }

    fn add(&mut self, package: PackageRef) {
        let identity = (
            package.ecosystem.clone(),
            package.name.clone(),
            package.version.clone(),
        );
        if self.seen.contains(&identity) {
            return;
        }
        if self.packages.len() >= MAX_PACKAGES {
            self.complete = false;
            return;
        }
        self.seen.insert(identity);
        self.packages.push(package);
    }

    fn absorb(&mut self, sbom: Sbom) {
        for package in sbom.packages {
            match purl::map(package.purl.as_deref(), &package.version) {
                purl::Mapped::Package(package) => self.add(package),
                purl::Mapped::Skipped => self.complete = false,
                purl::Mapped::Unlisted => {}
            }
        }
    }
}

#[derive(Deserialize)]
struct Listing {
    #[serde(default)]
    manifests: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(default)]
    digest: String,
    #[serde(default)]
    annotations: Option<HashMap<String, Value>>,
}

impl Entry {
    fn attests(&self, image: &Digest) -> bool {
        let Some(annotations) = &self.annotations else {
            return false;
        };
        let text = |key: &str| annotations.get(key).and_then(Value::as_str);
        text(REFERENCE_TYPE) == Some(ATTESTATION_MANIFEST)
            && text(REFERENCE_DIGEST) == Some(image.to_string().as_str())
    }
}

fn kind_of(asset: &Asset) -> &str {
    asset
        .attributes
        .pointer("/docker/artifactKind")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

struct Found {
    artifacts: Vec<Artifact>,
    complete: bool,
}

async fn find_artifacts(ctx: &Ctx, image: &str, digest: &Digest) -> Result<Found, ContentError> {
    let mut found = Found {
        artifacts: Vec::new(),
        complete: true,
    };
    let mut seen = HashSet::new();
    let mut keep = |asset: Asset, found: &mut Found| {
        let identity = storage::digest_of(&asset)
            .map(|digest| digest.to_string())
            .unwrap_or_else(|| asset.path.clone());
        if !seen.insert(identity) {
            return;
        }
        match found.artifacts.len() < MAX_ARTIFACTS {
            true => found.artifacts.push(Artifact {
                holder: ctx.clone(),
                asset,
            }),
            false => found.complete = false,
        }
    };
    for referrer in storage::referrers(ctx, image, digest).await? {
        if DOCUMENT_KINDS.contains(&kind_of(&referrer)) {
            keep(referrer, &mut found);
        }
    }
    let stem = digest.referrers_tag();
    for suffix in [".sbom", ".att"] {
        let path = storage::manifest_path(image, &format!("{stem}{suffix}"));
        if let Some(legacy) = storage::find(ctx, &path).await? {
            keep(legacy, &mut found);
        }
    }
    for parent in children::parents(ctx.db(), ctx.name(), digest, MAX_PARENTS).await? {
        let path = storage::manifest_path(image, &parent.to_string());
        let Some(index) = storage::find(ctx, &path).await? else {
            continue;
        };
        let bytes = storage::read_all(ctx, &index, MANIFEST_LIMIT).await?;
        let Ok(listing) = serde_json::from_slice::<Listing>(&bytes) else {
            found.complete = false;
            continue;
        };
        for entry in listing
            .manifests
            .iter()
            .filter(|entry| entry.attests(digest))
        {
            let Some(attestation) = Digest::parse(&entry.digest) else {
                found.complete = false;
                continue;
            };
            let path = storage::manifest_path(image, &attestation.to_string());
            match storage::find(ctx, &path).await? {
                Some(asset) => keep(asset, &mut found),
                None => found.complete = false,
            }
        }
    }
    Ok(found)
}

async fn gather(
    ctx: &Ctx,
    image: &str,
    digest: &Digest,
    collector: &mut Collector,
) -> Result<bool, ContentError> {
    let found = find_artifacts(ctx, image, digest).await?;
    if !found.complete {
        collector.complete = false;
    }
    let mut budget = astrum_sbom::MAX_INPUT_BYTES;
    let mut known = false;
    for artifact in &found.artifacts {
        let read =
            sbom::read_layers(ctx, image, artifact, Access::Internal, &mut budget, false).await?;
        if read.unreadable > 0 {
            collector.complete = false;
            known = true;
        }
        for sbom in read.parsed {
            known = true;
            collector.absorb(sbom);
        }
    }
    Ok(known)
}

async fn single(
    ctx: &Ctx,
    image: &str,
    digest: &Digest,
) -> Result<Option<Inventory>, ContentError> {
    let mut collector = Collector::new();
    if !gather(ctx, image, digest, &mut collector).await? {
        return Ok(None);
    }
    Ok(Some(Inventory {
        key: key(digest),
        packages: collector.packages,
        license: None,
        complete: collector.complete,
        source: Source::Sbom,
        digest: Some(digest.to_string()),
    }))
}

async fn index(ctx: &Ctx, image: &str, digest: &Digest) -> Result<Option<Inventory>, ContentError> {
    let children = children::of(ctx.db(), ctx.name(), digest, MAX_CHILDREN as u64 + 1).await?;
    let mut collector = Collector::new();
    if children.len() > MAX_CHILDREN {
        collector.complete = false;
    }
    let mut contributed = 0usize;
    for child in children.iter().take(MAX_CHILDREN) {
        let path = storage::manifest_path(image, &child.to_string());
        let Some(asset) = storage::find(ctx, &path).await? else {
            collector.complete = false;
            continue;
        };
        if ARTIFACT_KINDS.contains(&kind_of(&asset)) {
            continue;
        }
        if manifest::is_index(&storage::media_type_of(&asset)) {
            collector.complete = false;
            continue;
        }
        match gather(ctx, image, child, &mut collector).await? {
            true => contributed += 1,
            false => collector.complete = false,
        }
    }
    if contributed == 0 {
        return Ok(None);
    }
    Ok(Some(Inventory {
        key: key(digest),
        packages: collector.packages,
        license: None,
        complete: collector.complete,
        source: Source::Index,
        digest: Some(digest.to_string()),
    }))
}

pub(crate) async fn of(
    ctx: &Ctx,
    component: &Component,
) -> Result<Option<Inventory>, ContentError> {
    if ARTIFACT_KINDS.contains(&component.kind.as_str()) {
        return Ok(None);
    }
    let path = storage::manifest_path(&component.name, &component.version);
    let Some(asset) = storage::find(ctx, &path).await? else {
        return Ok(None);
    };
    let Some(digest) = storage::digest_of(&asset) else {
        return Ok(None);
    };
    match manifest::is_index(&storage::media_type_of(&asset)) {
        true => index(ctx, &component.name, &digest).await,
        false => single(ctx, &component.name, &digest).await,
    }
}

#[cfg(test)]
mod tests;
