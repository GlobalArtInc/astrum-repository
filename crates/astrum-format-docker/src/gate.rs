use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use serde_json::Value;

use crate::MANIFEST_LIMIT;
use crate::reference::Reference;
use crate::storage;
use crate::supply;

const ARTIFACT_KINDS: [&str; 3] = ["signature", "attestation", "sbom"];
const ARTIFACT_SUFFIXES: [&str; 3] = [".sig", ".att", ".sbom"];
const FILESYSTEM_MARKERS: [&str; 3] = [".tar", "rootfs", "image.layer"];

fn has_artifact_suffix(tag: &str) -> bool {
    ARTIFACT_SUFFIXES.iter().any(|suffix| tag.ends_with(suffix))
}

fn marked(reference: &Reference, attributes: &Value) -> bool {
    if attributes.pointer("/docker/subject").is_some() {
        return true;
    }
    match reference {
        Reference::Tag(tag) if supply::legacy_artifact_tag(tag) => true,
        Reference::Tag(tag) if has_artifact_suffix(tag) => false,
        _ => attributes
            .pointer("/docker/artifactKind")
            .and_then(Value::as_str)
            .is_some_and(|kind| ARTIFACT_KINDS.contains(&kind)),
    }
}

fn mentions_filesystem(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    FILESYSTEM_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

fn any_filesystem(value: &Value) -> bool {
    match value {
        Value::String(text) => mentions_filesystem(text),
        Value::Array(items) => items.iter().any(any_filesystem),
        Value::Object(fields) => fields
            .iter()
            .any(|(key, value)| mentions_filesystem(key) || any_filesystem(value)),
        _ => false,
    }
}

fn runnable(bytes: &[u8]) -> bool {
    match serde_json::from_slice::<Value>(bytes) {
        Ok(manifest) => any_filesystem(&manifest),
        Err(_) => true,
    }
}

async fn exempt(holder: &Ctx, reference: &Reference, asset: &Asset) -> Result<bool, ContentError> {
    if !marked(reference, &asset.attributes) {
        return Ok(false);
    }
    let bytes = storage::read_all(holder, asset, MANIFEST_LIMIT).await?;
    Ok(!runnable(&bytes))
}

fn judged_digests(asset: &Asset) -> Vec<String> {
    let stored = format!("sha256:{}", asset.sha256);
    match storage::digest_of(asset).map(|named| named.to_string()) {
        Some(named) if named != stored => vec![stored, named],
        _ => vec![stored],
    }
}

pub(crate) fn reference_of(asset: &Asset) -> Option<Reference> {
    match asset.kind.as_str() {
        storage::TAG => asset
            .attributes
            .pointer("/docker/tag")
            .and_then(Value::as_str)
            .map(|tag| Reference::Tag(tag.to_owned())),
        storage::MANIFEST => storage::digest_of(asset).map(Reference::Digest),
        _ => None,
    }
}

pub(crate) async fn gate_manifest(
    holder: &Ctx,
    reference: &Reference,
    asset: &Asset,
) -> Result<(), ContentError> {
    if exempt(holder, reference, asset).await? {
        return Ok(());
    }
    let named = match reference {
        Reference::Tag(tag) => Some(tag.as_str()),
        Reference::Digest(_) => None,
    };
    let gate = holder.content.gate();
    for digest in judged_digests(asset) {
        gate.check_digest(holder, holder.name(), &digest, named)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn tag(name: &str) -> Reference {
        Reference::Tag(name.to_owned())
    }

    fn digest() -> Reference {
        Reference::parse(&format!("sha256:{}", "ab".repeat(32))).unwrap()
    }

    #[test]
    fn signatures_attestations_sboms_and_manifests_with_a_subject_are_marked() {
        for kind in ["signature", "attestation", "sbom"] {
            let attributes = json!({"docker": {"artifactKind": kind}});
            assert!(marked(&digest(), &attributes), "{kind}");
            assert!(marked(&tag("v1"), &attributes), "{kind}");
        }
        let subject = json!({"docker": {"artifactKind": "image", "subject": "sha256:x"}});
        assert!(marked(&digest(), &subject));
        assert!(marked(&tag("v1"), &subject));
    }

    #[test]
    fn images_and_other_kinds_are_not_marked() {
        for kind in ["image", "helm", "wasm", "model", "other", ""] {
            let attributes = json!({"docker": {"artifactKind": kind}});
            assert!(!marked(&digest(), &attributes), "{kind}");
            assert!(!marked(&tag("v1"), &attributes), "{kind}");
        }
        assert!(!marked(&digest(), &json!({})));
    }

    #[test]
    fn only_the_legacy_digest_tags_are_marked_by_their_name() {
        let image = json!({"docker": {"artifactKind": "image"}});
        let hex = "ab".repeat(32);
        for suffix in [".sig", ".att", ".sbom"] {
            assert!(marked(&tag(&format!("sha256-{hex}{suffix}")), &image));
            assert!(!marked(&tag(&format!("v1{suffix}")), &image));
            assert!(!marked(&tag(&format!("sha256-short{suffix}")), &image));
        }
        let forced = json!({"docker": {"artifactKind": "signature"}});
        assert!(!marked(&tag("v1.sig"), &forced));
        assert!(marked(&digest(), &forced));
    }

    fn manifest(body: Value) -> Vec<u8> {
        serde_json::to_vec(&body).unwrap()
    }

    #[test]
    fn a_manifest_is_runnable_when_any_string_anywhere_names_a_filesystem_layer() {
        let filesystem = [
            "application/vnd.oci.image.layer.v1.tar",
            "application/vnd.oci.image.layer.v1.tar+gzip",
            "application/vnd.oci.image.layer.v1.tar+zstd",
            "application/vnd.oci.image.layer.nondistributable.v1.tar+gzip",
            "application/vnd.docker.image.rootfs.diff.tar",
            "application/vnd.docker.image.rootfs.diff.tar.gzip",
            "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
            "Application/Vnd.OCI.Image.Layer.V1.TAR+GZIP; x=y",
        ];
        for media_type in filesystem {
            for layers in ["layers", "Layers", "LAYERS"] {
                let body = json!({layers: [{"MediaType": media_type, "digest": "sha256:x"}]});
                assert!(runnable(&manifest(body)), "{layers} {media_type}");
            }
            let nested = json!({"annotations": {"a": [{"b": media_type}]}});
            assert!(runnable(&manifest(nested)), "{media_type}");
        }
        assert!(runnable(&manifest(json!({"a.tar": 1}))));
        assert!(runnable(&manifest(json!([["rootfs"]]))));
        assert!(runnable(b"not json"));
        assert!(runnable(b""));
    }

    #[test]
    fn the_manifests_of_signatures_attestations_and_sboms_are_not_runnable() {
        let harmless = [
            "application/vnd.dev.cosign.simplesigning.v1+json",
            "application/vnd.in-toto+json",
            "application/vnd.dsse.envelope.v1+json",
            "application/spdx+json",
            "application/vnd.cyclonedx+json",
            "application/jose+json",
            "application/vnd.oci.empty.v1+json",
            "application/vnd.oci.image.config.v1+json",
        ];
        for media_type in harmless {
            let body = json!({
                "schemaVersion": 2,
                "mediaType": "application/vnd.oci.image.manifest.v1+json",
                "artifactType": media_type,
                "config": {"mediaType": media_type, "digest": "sha256:x", "size": 2},
                "layers": [{"mediaType": media_type, "digest": "sha256:y", "size": 1,
                            "annotations": {"dev.cosignproject.cosign/signature": "MEUCIQ=="}}],
                "subject": {"mediaType": "application/vnd.oci.image.manifest.v1+json",
                            "digest": "sha256:z", "size": 3},
            });
            assert!(!runnable(&manifest(body)), "{media_type}");
        }
        assert!(!runnable(&manifest(json!({"schemaVersion": 2}))));
    }
}
