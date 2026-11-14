use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use serde_json::Value;

use crate::MANIFEST_LIMIT;
use crate::manifest;
use crate::reference::Reference;
use crate::storage;
use crate::supply;

const ARTIFACT_KINDS: [&str; 3] = ["signature", "attestation", "sbom"];
const ARTIFACT_SUFFIXES: [&str; 3] = [".sig", ".att", ".sbom"];
const FILESYSTEM_LAYERS: [&str; 8] = [
    "application/vnd.oci.image.layer.v1.tar",
    "application/vnd.oci.image.layer.v1.tar+gzip",
    "application/vnd.oci.image.layer.v1.tar+zstd",
    "application/vnd.oci.image.layer.nondistributable.v1.tar",
    "application/vnd.oci.image.layer.nondistributable.v1.tar+gzip",
    "application/vnd.oci.image.layer.nondistributable.v1.tar+zstd",
    "application/vnd.docker.image.rootfs.diff.tar.gzip",
    "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
];

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

fn filesystem_layer(media_type: &str) -> bool {
    let bare = manifest::bare(media_type);
    FILESYSTEM_LAYERS.contains(&bare.as_str())
}

fn runnable(bytes: &[u8]) -> bool {
    match manifest::try_layer_types(bytes) {
        Some(types) => types.iter().any(|media_type| filesystem_layer(media_type)),
        None => true,
    }
}

async fn exempt(holder: &Ctx, reference: &Reference, asset: &Asset) -> Result<bool, ContentError> {
    if !marked(reference, &asset.attributes) {
        return Ok(false);
    }
    let bytes = storage::read_all(holder, asset, MANIFEST_LIMIT).await?;
    Ok(!runnable(&bytes))
}

pub(crate) async fn gate_manifest(
    holder: &Ctx,
    reference: &Reference,
    asset: &Asset,
    digest: &str,
) -> Result<(), ContentError> {
    if exempt(holder, reference, asset).await? {
        return Ok(());
    }
    holder
        .content
        .gate()
        .check_digest(holder, holder.name(), digest)
        .await
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

    #[test]
    fn a_manifest_is_runnable_when_any_layer_is_a_filesystem_layer_and_unreadable_ones_count_as_runnable()
     {
        let with = |types: &[&str]| {
            let layers: Vec<Value> = types
                .iter()
                .map(|media_type| json!({"mediaType": media_type, "digest": "sha256:x", "size": 1}))
                .collect();
            serde_json::to_vec(&json!({"schemaVersion": 2, "layers": layers})).unwrap()
        };
        for media_type in FILESYSTEM_LAYERS {
            assert!(runnable(&with(&[media_type])), "{media_type}");
            assert!(runnable(&with(&["application/jose+json", media_type])));
        }
        assert!(runnable(&with(&[
            "Application/Vnd.OCI.image.layer.v1.TAR+gzip; x=y"
        ])));
        for harmless in [
            "application/vnd.dev.cosign.simplesigning.v1+json",
            "application/vnd.in-toto+json",
            "application/spdx+json",
            "application/vnd.oci.empty.v1+json",
        ] {
            assert!(!runnable(&with(&[harmless])), "{harmless}");
        }
        assert!(!runnable(&with(&[])));
        assert!(!runnable(br#"{"schemaVersion": 2}"#));
        assert!(runnable(b"not json"));
    }
}
