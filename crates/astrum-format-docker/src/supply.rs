use std::collections::BTreeMap;

use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx, group};
use astrum_repo::RepositoryType;
use serde::Serialize;
use serde_json::Value;

use crate::MANIFEST_LIMIT;
use crate::manifest;
use crate::reference::{Digest, Reference};
use crate::storage;

const COSIGN_SIGNATURE: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";
const SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const BUNDLE: &str = "application/vnd.dev.sigstore.bundle";
const NOTATION: &str = "application/vnd.cncf.notary.signature";

#[derive(Serialize)]
pub struct Listing {
    digest: String,
    children: Vec<String>,
    signatures: Vec<Signature>,
    attestations: Vec<Attestation>,
    sboms: Vec<Sbom>,
}

#[derive(Serialize)]
pub struct Signature {
    digest: String,
    scheme: &'static str,
}

#[derive(Serialize)]
pub struct Attestation {
    digest: String,
}

#[derive(Serialize)]
pub struct Sbom {
    digest: String,
    format: &'static str,
}

#[derive(Clone, Copy)]
enum Found {
    Signature(&'static str),
    Attestation,
    Sbom(&'static str),
}

pub async fn list(
    ctx: &Ctx,
    image: &str,
    reference: &Reference,
) -> Result<Option<Listing>, ContentError> {
    let members = members_of(ctx).await?;
    let path = storage::manifest_path(image, &reference.as_str());
    let mut held = None;
    for member in &members {
        if let Some(asset) = storage::find(member, &path).await? {
            held = Some((member, asset));
            break;
        }
    }
    let Some((holder, asset)) = held else {
        return Ok(None);
    };
    let digest = match reference {
        Reference::Digest(digest) => digest.clone(),
        Reference::Tag(_) => storage::digest_of(&asset).ok_or_else(|| {
            ContentError::internal(std::io::Error::other(format!(
                "{} names no digest",
                asset.path
            )))
        })?,
    };
    let children = match manifest::is_index(&storage::media_type_of(&asset)) {
        true => children_of(holder, &asset).await?,
        false => Vec::new(),
    };
    let mut found: BTreeMap<String, Found> = BTreeMap::new();
    let stem = digest.referrers_tag();
    for member in &members {
        for referrer in storage::referrers(member, image, &digest).await? {
            if let Some(kind) = classify(member, &referrer).await?
                && let Some(target) = storage::digest_of(&referrer)
            {
                found.entry(target.to_string()).or_insert(kind);
            }
        }
        for suffix in [".sig", ".att", ".sbom"] {
            let tag = storage::manifest_path(image, &format!("{stem}{suffix}"));
            let Some(legacy) = storage::find(member, &tag).await? else {
                continue;
            };
            let Some(target) = storage::digest_of(&legacy) else {
                continue;
            };
            let kind = match suffix {
                ".sig" => Found::Signature("legacy"),
                ".att" => Found::Attestation,
                _ => Found::Sbom(format_of(&source_of(&legacy))),
            };
            found.entry(target.to_string()).or_insert(kind);
        }
    }
    let mut listing = Listing {
        digest: digest.to_string(),
        children,
        signatures: Vec::new(),
        attestations: Vec::new(),
        sboms: Vec::new(),
    };
    for (digest, kind) in found {
        match kind {
            Found::Signature(scheme) => listing.signatures.push(Signature { digest, scheme }),
            Found::Attestation => listing.attestations.push(Attestation { digest }),
            Found::Sbom(format) => listing.sboms.push(Sbom { digest, format }),
        }
    }
    Ok(Some(listing))
}

pub(crate) async fn members_of(ctx: &Ctx) -> Result<Vec<Ctx>, ContentError> {
    match ctx.kind() {
        RepositoryType::Group => Ok(group::members(ctx)
            .await?
            .into_iter()
            .map(|member| ctx.member(member))
            .collect()),
        _ => Ok(vec![ctx.clone()]),
    }
}

async fn children_of(ctx: &Ctx, asset: &Asset) -> Result<Vec<String>, ContentError> {
    let bytes = storage::read_all(ctx, asset, MANIFEST_LIMIT).await?;
    let parsed = manifest::parse(&bytes, Some(&storage::media_type_of(asset)))
        .map_err(|problem| ContentError::internal(std::io::Error::other(problem)))?;
    Ok(parsed.children.iter().map(ToString::to_string).collect())
}

pub struct Artifact {
    pub holder: Ctx,
    pub asset: Asset,
}

fn legacy_tag(tag: &str, suffixes: &[&str]) -> bool {
    let Some(stem) = suffixes.iter().find_map(|suffix| tag.strip_suffix(suffix)) else {
        return false;
    };
    stem.split_once('-').is_some_and(|(algorithm, hex)| {
        matches!(algorithm, "sha256" | "sha512")
            && hex.len() == 64
            && hex
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    })
}

fn legacy_document_tag(tag: &str) -> bool {
    legacy_tag(tag, &[".sbom", ".att"])
}

pub(crate) fn legacy_artifact_tag(tag: &str) -> bool {
    legacy_tag(tag, &[".sig", ".att", ".sbom"])
}

pub async fn artifact(
    ctx: &Ctx,
    image: &str,
    digest: &Digest,
) -> Result<Option<Artifact>, ContentError> {
    let path = storage::manifest_path(image, &digest.to_string());
    for member in members_of(ctx).await? {
        if let Some(asset) = storage::find(&member, &path).await?
            && asset.attributes.pointer("/docker/subject").is_some()
            && matches!(
                classify(&member, &asset).await?,
                Some(Found::Sbom(_) | Found::Attestation)
            )
        {
            return Ok(Some(Artifact {
                holder: member,
                asset,
            }));
        }
        for tag in storage::tags_pointing_at(&member, image, digest).await? {
            if legacy_document_tag(&storage::tag_name(&tag, image)) {
                return Ok(Some(Artifact {
                    holder: member,
                    asset: tag,
                }));
            }
        }
    }
    Ok(None)
}

async fn classify(ctx: &Ctx, referrer: &Asset) -> Result<Option<Found>, ContentError> {
    let artifact = artifact_of(referrer);
    if artifact == COSIGN_SIGNATURE {
        return Ok(Some(Found::Signature("referrer")));
    }
    if artifact.starts_with(BUNDLE) {
        return Ok(Some(Found::Signature("bundle")));
    }
    if artifact == NOTATION {
        return Ok(Some(Found::Signature("notation")));
    }
    match referrer
        .attributes
        .pointer("/docker/artifactKind")
        .and_then(Value::as_str)
    {
        Some("attestation") => return Ok(Some(Found::Attestation)),
        Some("sbom") => return Ok(Some(Found::Sbom(format_of(&source_of(referrer))))),
        _ => {}
    }
    let bytes = storage::read_all(ctx, referrer, MANIFEST_LIMIT).await?;
    let simple = manifest::layer_types(&bytes)
        .iter()
        .any(|layer| layer == SIMPLE_SIGNING);
    Ok(simple.then_some(Found::Signature("referrer")))
}

fn artifact_of(asset: &Asset) -> String {
    asset
        .attributes
        .pointer("/docker/artifactType")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn source_of(asset: &Asset) -> String {
    match artifact_of(asset) {
        artifact if artifact.is_empty() => storage::media_type_of(asset),
        artifact => artifact,
    }
}

fn format_of(source: &str) -> &'static str {
    let lower = source.to_ascii_lowercase();
    if lower.contains("spdx") {
        "spdx"
    } else if lower.contains("cyclonedx") {
        "cyclonedx"
    } else {
        "unknown"
    }
}
