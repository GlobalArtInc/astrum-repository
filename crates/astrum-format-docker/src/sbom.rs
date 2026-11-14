use std::sync::Arc;

use astrum_content::{ContentError, Ctx, View};
use astrum_sbom::{Format, Package, Sbom};
use serde::Serialize;
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::MANIFEST_LIMIT;
use crate::manifest;
use crate::reference::Digest;
use crate::storage;
use crate::supply::{self, Artifact};

pub const PERMITS: usize = 3;
pub const RETRY_AFTER_SECONDS: &str = "5";
pub const PAGE_SIZE: usize = 100;
pub const FILTER_CHARS: usize = 200;
const LAYERS_READ: usize = 4;
const DOCUMENT_TYPES: [&str; 5] = [
    "application/spdx+json",
    "application/vnd.cyclonedx+json",
    "application/vnd.in-toto+json",
    "application/vnd.dsse.envelope.v1+json",
    "text/spdx+json",
];

#[derive(Clone)]
pub struct Limit(Arc<Semaphore>);

impl Limit {
    pub fn new(permits: usize) -> Self {
        Limit(Arc::new(Semaphore::new(permits)))
    }

    pub fn take(&self) -> Option<SemaphorePermit<'_>> {
        self.0.try_acquire().ok()
    }
}

pub enum Outcome {
    Parsed(Sbom),
    Refused(astrum_sbom::Error),
    TooLarge,
    Missing(String),
    NoDocument,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    format: Format,
    total: usize,
    page: u32,
    page_size: usize,
    packages: Vec<Package>,
}

pub fn page_number(raw: Option<&str>) -> Result<u32, String> {
    match raw {
        None => Ok(1),
        Some(raw) => raw
            .parse::<u32>()
            .ok()
            .filter(|page| *page >= 1)
            .ok_or_else(|| format!("page must be a number from 1 to {}", u32::MAX)),
    }
}

pub fn page_of(sbom: Sbom, page: u32, filter: &str) -> Page {
    let needle = filter.to_lowercase();
    let mut packages = sbom.packages;
    if !needle.is_empty() {
        packages.retain(|package| matches(package, &needle));
    }
    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
    });
    let total = packages.len();
    let skipped = usize::try_from(page.saturating_sub(1))
        .unwrap_or(usize::MAX)
        .saturating_mul(PAGE_SIZE);
    Page {
        format: sbom.format,
        total,
        page,
        page_size: PAGE_SIZE,
        packages: packages.into_iter().skip(skipped).take(PAGE_SIZE).collect(),
    }
}

fn matches(package: &Package, needle: &str) -> bool {
    package.name.to_lowercase().contains(needle)
        || package
            .purl
            .as_deref()
            .is_some_and(|purl| purl.to_lowercase().contains(needle))
        || package
            .licenses
            .iter()
            .any(|license| license.to_lowercase().contains(needle))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Checked,
    Internal,
}

pub struct Read {
    pub parsed: Vec<Sbom>,
    pub failed: Option<Outcome>,
    pub unreadable: usize,
}

pub async fn load(ctx: &Ctx, image: &str, artifact: &Artifact) -> Result<Outcome, ContentError> {
    let mut budget = astrum_sbom::MAX_INPUT_BYTES;
    let read = read_layers(ctx, image, artifact, Access::Checked, &mut budget, true).await?;
    Ok(match read.parsed.into_iter().next() {
        Some(sbom) => Outcome::Parsed(sbom),
        None => read.failed.unwrap_or(Outcome::NoDocument),
    })
}

pub async fn read_layers(
    ctx: &Ctx,
    image: &str,
    artifact: &Artifact,
    access: Access,
    budget: &mut usize,
    first_only: bool,
) -> Result<Read, ContentError> {
    let manifest_bytes =
        storage::read_all(&artifact.holder, &artifact.asset, MANIFEST_LIMIT).await?;
    let layers: Vec<manifest::Layer> = manifest::layers(&manifest_bytes)
        .into_iter()
        .filter(|layer| DOCUMENT_TYPES.contains(&manifest::bare(&layer.media_type).as_str()))
        .take(LAYERS_READ)
        .collect();
    let mut holders = vec![artifact.holder.clone()];
    holders.extend(
        supply::members_of(ctx)
            .await?
            .into_iter()
            .filter(|member| member.name() != artifact.holder.name()),
    );
    let mut read = Read {
        parsed: Vec::new(),
        failed: None,
        unreadable: 0,
    };
    for layer in layers {
        if access == Access::Checked {
            ctx.require(View::Read, &format!("/v2/{image}/blobs/{}", layer.digest))?;
        }
        let bytes = match read_layer(&holders, &layer.digest, *budget).await? {
            Layer::Bytes(bytes) => bytes,
            Layer::TooLarge => {
                read.unreadable += 1;
                read.failed.get_or_insert(Outcome::TooLarge);
                continue;
            }
            Layer::Missing => {
                read.unreadable += 1;
                read.failed.get_or_insert(Outcome::Missing(format!(
                    "the document blob {} is not stored in this repository",
                    layer.digest
                )));
                continue;
            }
        };
        *budget -= bytes.len();
        let parsed = tokio::task::spawn_blocking(move || astrum_sbom::parse(&bytes))
            .await
            .map_err(ContentError::internal)?;
        match parsed {
            Ok(sbom) => {
                read.parsed.push(sbom);
                if first_only {
                    return Ok(read);
                }
            }
            Err(error) => {
                if !matches!(error, astrum_sbom::Error::Unsupported(_)) {
                    read.unreadable += 1;
                }
                read.failed.get_or_insert(Outcome::Refused(error));
            }
        }
    }
    Ok(read)
}

enum Layer {
    Bytes(Vec<u8>),
    TooLarge,
    Missing,
}

async fn read_layer(
    holders: &[Ctx],
    digest: &Digest,
    budget: usize,
) -> Result<Layer, ContentError> {
    let path = storage::blob_path(digest);
    for holder in holders {
        let Some(blob) = storage::find(holder, &path).await? else {
            continue;
        };
        if blob.size > budget as i64 {
            return Ok(Layer::TooLarge);
        }
        let bytes = storage::read_capped(holder, &blob, budget).await?;
        return Ok(match bytes.len() > budget {
            true => Layer::TooLarge,
            false => Layer::Bytes(bytes),
        });
    }
    Ok(Layer::Missing)
}
