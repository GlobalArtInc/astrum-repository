use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use serde_json::Value;

use crate::reference::Reference;
use crate::supply;

const ARTIFACT_KINDS: [&str; 3] = ["signature", "attestation", "sbom"];
const ARTIFACT_SUFFIXES: [&str; 3] = [".sig", ".att", ".sbom"];

fn has_artifact_suffix(tag: &str) -> bool {
    ARTIFACT_SUFFIXES.iter().any(|suffix| tag.ends_with(suffix))
}

fn exempt(reference: &Reference, attributes: &Value) -> bool {
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

pub(crate) async fn gate_manifest(
    holder: &Ctx,
    reference: &Reference,
    asset: &Asset,
    digest: &str,
) -> Result<(), ContentError> {
    if exempt(reference, &asset.attributes) {
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
    fn signatures_attestations_sboms_and_manifests_with_a_subject_are_exempt() {
        for kind in ["signature", "attestation", "sbom"] {
            let attributes = json!({"docker": {"artifactKind": kind}});
            assert!(exempt(&digest(), &attributes), "{kind}");
            assert!(exempt(&tag("v1"), &attributes), "{kind}");
        }
        let subject = json!({"docker": {"artifactKind": "image", "subject": "sha256:x"}});
        assert!(exempt(&digest(), &subject));
        assert!(exempt(&tag("v1"), &subject));
    }

    #[test]
    fn images_and_other_kinds_are_not_exempt() {
        for kind in ["image", "helm", "wasm", "model", "other", ""] {
            let attributes = json!({"docker": {"artifactKind": kind}});
            assert!(!exempt(&digest(), &attributes), "{kind}");
            assert!(!exempt(&tag("v1"), &attributes), "{kind}");
        }
        assert!(!exempt(&digest(), &json!({})));
    }

    #[test]
    fn only_the_legacy_digest_tags_are_exempt_by_their_name() {
        let image = json!({"docker": {"artifactKind": "image"}});
        let hex = "ab".repeat(32);
        for suffix in [".sig", ".att", ".sbom"] {
            assert!(exempt(&tag(&format!("sha256-{hex}{suffix}")), &image));
            assert!(!exempt(&tag(&format!("v1{suffix}")), &image));
            assert!(!exempt(&tag(&format!("sha256-short{suffix}")), &image));
        }
        let forced = json!({"docker": {"artifactKind": "signature"}});
        assert!(!exempt(&tag("v1.sig"), &forced));
        assert!(exempt(&digest(), &forced));
    }
}
