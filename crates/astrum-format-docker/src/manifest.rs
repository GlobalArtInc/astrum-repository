use serde::Deserialize;
use serde_json::{Map, Value};

use crate::reference::Digest;

pub const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
pub const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
pub const DOCKER_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
pub const OCI_IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub const DOCKER_IMAGE_CONFIG: &str = "application/vnd.docker.container.image.v1+json";

pub const ACCEPTED: &str = "application/vnd.oci.image.index.v1+json, \
application/vnd.oci.image.manifest.v1+json, \
application/vnd.docker.distribution.manifest.list.v2+json, \
application/vnd.docker.distribution.manifest.v2+json";

const SCHEMA1: [&str; 2] = [
    "application/vnd.docker.distribution.manifest.v1+json",
    "application/vnd.docker.distribution.manifest.v1+prettyjws",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    Helm,
    Signature,
    Sbom,
    Attestation,
    Wasm,
    Model,
    Other,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Image => "image",
            Kind::Helm => "helm",
            Kind::Signature => "signature",
            Kind::Sbom => "sbom",
            Kind::Attestation => "attestation",
            Kind::Wasm => "wasm",
            Kind::Model => "model",
            Kind::Other => "other",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDescriptor {
    #[serde(default)]
    media_type: String,
    digest: String,
    #[serde(default)]
    urls: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawManifest {
    schema_version: Option<i64>,
    media_type: Option<String>,
    artifact_type: Option<String>,
    config: Option<RawDescriptor>,
    layers: Option<Vec<RawDescriptor>>,
    manifests: Option<Vec<RawDescriptor>>,
    subject: Option<RawDescriptor>,
    annotations: Option<Map<String, Value>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    pub media_type: String,
    pub artifact_type: Option<String>,
    pub kind: Kind,
    pub subject: Option<Digest>,
    pub annotations: Option<Map<String, Value>>,
    pub blobs: Vec<Digest>,
    pub children: Vec<Digest>,
}

pub fn is_index(media_type: &str) -> bool {
    media_type == OCI_INDEX || media_type == DOCKER_LIST
}

pub fn bare(content_type: &str) -> String {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn known(media_type: &str) -> bool {
    matches!(
        media_type,
        OCI_MANIFEST | OCI_INDEX | DOCKER_MANIFEST | DOCKER_LIST
    )
}

fn digest_of(descriptor: &RawDescriptor, field: &str) -> Result<Digest, String> {
    Digest::parse(&descriptor.digest).ok_or_else(|| {
        format!(
            "the {field} descriptor has an unsupported digest {}",
            descriptor.digest
        )
    })
}

fn foreign(descriptor: &RawDescriptor) -> bool {
    descriptor
        .urls
        .as_ref()
        .is_some_and(|urls| !urls.is_empty())
        || descriptor.media_type.contains("nondistributable")
        || descriptor.media_type.contains(".foreign.")
}

pub fn parse(bytes: &[u8], content_type: Option<&str>) -> Result<Manifest, String> {
    let raw: RawManifest = serde_json::from_slice(bytes)
        .map_err(|error| format!("the manifest is not valid JSON: {error}"))?;
    let declared = content_type.map(bare).filter(|value| !value.is_empty());
    if let Some(declared) = &declared
        && SCHEMA1.contains(&declared.as_str())
    {
        return Err("Docker image manifests of schema 1 are not supported".to_owned());
    }
    match raw.schema_version {
        Some(2) => {}
        Some(1) => return Err("Docker image manifests of schema 1 are not supported".to_owned()),
        Some(other) => return Err(format!("schemaVersion {other} is not supported")),
        None => return Err("the manifest has no schemaVersion".to_owned()),
    }
    let media_type = match (&raw.media_type, &declared) {
        (Some(body), Some(header)) if known(header) && body != header => {
            return Err(format!(
                "the manifest declares {body} but was sent as {header}"
            ));
        }
        (Some(body), _) => body.clone(),
        (None, Some(header)) if known(header) => header.clone(),
        (None, _) if raw.manifests.is_some() => OCI_INDEX.to_owned(),
        (None, _) if raw.config.is_some() => OCI_MANIFEST.to_owned(),
        (None, _) => return Err("the manifest type cannot be determined".to_owned()),
    };
    if !known(&media_type) {
        return Err(format!(
            "{media_type} is not a manifest type this registry stores"
        ));
    }
    let subject = raw
        .subject
        .as_ref()
        .map(|subject| digest_of(subject, "subject"))
        .transpose()?;
    let artifact_type = raw.artifact_type.clone().filter(|value| !value.is_empty());
    if is_index(&media_type) {
        let manifests = raw
            .manifests
            .as_ref()
            .ok_or_else(|| "an index needs a manifests list".to_owned())?;
        let children = manifests
            .iter()
            .map(|child| digest_of(child, "manifest"))
            .collect::<Result<Vec<_>, _>>()?;
        let kind =
            artifact_type
                .as_deref()
                .and_then(classify)
                .unwrap_or(if artifact_type.is_some() {
                    Kind::Other
                } else {
                    Kind::Image
                });
        return Ok(Manifest {
            media_type,
            artifact_type,
            kind,
            subject,
            annotations: raw.annotations,
            blobs: Vec::new(),
            children,
        });
    }
    let config = raw
        .config
        .as_ref()
        .ok_or_else(|| "an image manifest needs a config descriptor".to_owned())?;
    let layers = raw.layers.clone().unwrap_or_default();
    let mut blobs = vec![digest_of(config, "config")?];
    for layer in &layers {
        let digest = digest_of(layer, "layer")?;
        if !foreign(layer) {
            blobs.push(digest);
        }
    }
    let config_type = (!config.media_type.is_empty()).then(|| config.media_type.clone());
    let layer_types: Vec<&str> = layers
        .iter()
        .map(|layer| layer.media_type.as_str())
        .collect();
    let kind = kind_of(
        artifact_type.as_deref(),
        config_type.as_deref(),
        &layer_types,
    );
    let effective = artifact_type.or(config_type);
    Ok(Manifest {
        media_type,
        artifact_type: effective,
        kind,
        subject,
        annotations: raw.annotations,
        blobs,
        children: Vec::new(),
    })
}

fn classify(media_type: &str) -> Option<Kind> {
    let lower = media_type.to_ascii_lowercase();
    let has = |needle: &str| lower.contains(needle);
    if has("helm") {
        Some(Kind::Helm)
    } else if has("notary") || has("cosign") || has("sigstore") || has("signature") {
        Some(Kind::Signature)
    } else if has("in-toto") || has("dsse") || has("attestation") || has("provenance") {
        Some(Kind::Attestation)
    } else if has("spdx") || has("cyclonedx") || has("sbom") || has("syft") {
        Some(Kind::Sbom)
    } else if has("wasm") {
        Some(Kind::Wasm)
    } else if has("model") || has("docker.ai") || has("ollama") || has("gguf") || has("safetensors")
    {
        Some(Kind::Model)
    } else {
        None
    }
}

fn image_config(media_type: &str) -> bool {
    media_type == OCI_IMAGE_CONFIG || media_type == DOCKER_IMAGE_CONFIG
}

pub fn kind_of(artifact_type: Option<&str>, config: Option<&str>, layers: &[&str]) -> Kind {
    if let Some(kind) = artifact_type.and_then(classify) {
        return kind;
    }
    if let Some(kind) = config
        .filter(|config| !image_config(config))
        .and_then(classify)
    {
        return kind;
    }
    if let Some(kind) = layers.iter().find_map(|layer| classify(layer)) {
        return kind;
    }
    match (artifact_type, config) {
        (None, Some(config)) if image_config(config) => Kind::Image,
        _ => Kind::Other,
    }
}

pub fn kind_for_tag(kind: Kind, tag: &str) -> Kind {
    if kind != Kind::Image && kind != Kind::Other {
        return kind;
    }
    if tag.ends_with(".sig") {
        Kind::Signature
    } else if tag.ends_with(".att") {
        Kind::Attestation
    } else if tag.ends_with(".sbom") {
        Kind::Sbom
    } else {
        kind
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const OCI_EMPTY: &str = "application/vnd.oci.empty.v1+json";

    fn digest(seed: &str) -> String {
        Digest::sha256(seed.as_bytes()).to_string()
    }

    fn image(config_type: &str, layer_type: &str, extra: Value) -> Vec<u8> {
        let mut manifest = json!({
            "schemaVersion": 2,
            "mediaType": OCI_MANIFEST,
            "config": {"mediaType": config_type, "digest": digest("c"), "size": 2},
            "layers": [{"mediaType": layer_type, "digest": digest("l"), "size": 3}],
        });
        if let (Some(target), Some(extra)) = (manifest.as_object_mut(), extra.as_object()) {
            target.extend(extra.clone());
        }
        serde_json::to_vec(&manifest).unwrap()
    }

    #[test]
    fn images_list_their_blobs() {
        let parsed = parse(
            &image(
                OCI_IMAGE_CONFIG,
                "application/vnd.oci.image.layer.v1.tar+gzip",
                json!({}),
            ),
            Some(OCI_MANIFEST),
        )
        .unwrap();
        assert_eq!(parsed.kind, Kind::Image);
        assert_eq!(parsed.artifact_type.as_deref(), Some(OCI_IMAGE_CONFIG));
        assert_eq!(parsed.blobs.len(), 2);
        assert!(!is_index(&parsed.media_type));
        assert_eq!(parsed.subject, None);
    }

    #[test]
    fn foreign_layers_need_not_be_pushed() {
        let manifest = json!({
            "schemaVersion": 2,
            "mediaType": DOCKER_MANIFEST,
            "config": {"mediaType": DOCKER_IMAGE_CONFIG, "digest": digest("c"), "size": 2},
            "layers": [
                {"mediaType": "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip", "digest": digest("f"), "size": 3},
                {"mediaType": "application/vnd.oci.image.layer.nondistributable.v1.tar+gzip", "digest": digest("n"), "size": 3, "urls": ["https://example.com/x"]},
                {"mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip", "digest": digest("l"), "size": 3},
            ],
        });
        let parsed = parse(&serde_json::to_vec(&manifest).unwrap(), None).unwrap();
        assert_eq!(parsed.media_type, DOCKER_MANIFEST);
        assert_eq!(parsed.blobs.len(), 2);
        assert_eq!(parsed.kind, Kind::Image);
    }

    #[test]
    fn artifacts_are_told_apart() {
        let cases = [
            (
                "application/vnd.cncf.helm.config.v1+json",
                "application/vnd.cncf.helm.chart.content.v1.tar+gzip",
                json!({}),
                Kind::Helm,
            ),
            (
                OCI_IMAGE_CONFIG,
                "application/vnd.dev.cosign.simplesigning.v1+json",
                json!({}),
                Kind::Signature,
            ),
            (
                OCI_EMPTY,
                "application/jose+json",
                json!({"artifactType": "application/vnd.cncf.notary.signature"}),
                Kind::Signature,
            ),
            (
                OCI_EMPTY,
                "application/spdx+json",
                json!({"artifactType": "application/spdx+json"}),
                Kind::Sbom,
            ),
            (
                OCI_IMAGE_CONFIG,
                "application/vnd.in-toto+json",
                json!({}),
                Kind::Attestation,
            ),
            (
                "application/vnd.wasm.config.v0+json",
                "application/wasm",
                json!({}),
                Kind::Wasm,
            ),
            (
                OCI_EMPTY,
                "application/vnd.cncf.model.weight.v1.raw",
                json!({"artifactType": "application/vnd.cncf.model.manifest.v1+json"}),
                Kind::Model,
            ),
            (
                "application/vnd.docker.ai.model.config.v0.1+json",
                "application/vnd.docker.ai.gguf.v3",
                json!({}),
                Kind::Model,
            ),
            (
                OCI_EMPTY,
                "application/octet-stream",
                json!({"artifactType": "application/vnd.example.thing"}),
                Kind::Other,
            ),
        ];
        for (config, layer, extra, expected) in cases {
            let parsed = parse(&image(config, layer, extra), None).unwrap();
            assert_eq!(parsed.kind, expected, "{config} {layer}");
        }
    }

    #[test]
    fn artifact_type_falls_back_to_the_config_type() {
        let parsed = parse(
            &image(
                "application/vnd.example.config",
                "application/octet-stream",
                json!({}),
            ),
            None,
        )
        .unwrap();
        assert_eq!(
            parsed.artifact_type.as_deref(),
            Some("application/vnd.example.config")
        );
        let explicit = parse(
            &image(
                OCI_EMPTY,
                "application/octet-stream",
                json!({"artifactType": "application/vnd.example.sbom"}),
            ),
            None,
        )
        .unwrap();
        assert_eq!(
            explicit.artifact_type.as_deref(),
            Some("application/vnd.example.sbom")
        );
        assert_eq!(explicit.kind, Kind::Sbom);
    }

    #[test]
    fn indexes_list_children_and_subjects() {
        let subject = digest("s");
        let index = json!({
            "schemaVersion": 2,
            "mediaType": OCI_INDEX,
            "manifests": [
                {"mediaType": OCI_MANIFEST, "digest": digest("a"), "size": 1, "platform": {"os": "linux", "architecture": "amd64"}},
                {"mediaType": OCI_MANIFEST, "digest": digest("b"), "size": 1},
            ],
            "subject": {"mediaType": OCI_MANIFEST, "digest": subject, "size": 1},
            "annotations": {"x": "y"},
        });
        let parsed = parse(&serde_json::to_vec(&index).unwrap(), Some(OCI_INDEX)).unwrap();
        assert!(is_index(&parsed.media_type));
        assert_eq!(parsed.children.len(), 2);
        assert_eq!(parsed.subject.unwrap().to_string(), subject);
        assert_eq!(parsed.kind, Kind::Image);
        assert_eq!(parsed.artifact_type, None);
        assert_eq!(parsed.annotations.unwrap()["x"], "y");
    }

    #[test]
    fn bad_manifests_are_refused() {
        let schema1 = json!({"schemaVersion": 1, "name": "x", "tag": "y", "fsLayers": []});
        assert!(parse(&serde_json::to_vec(&schema1).unwrap(), None).is_err());
        assert!(parse(b"not json", None).is_err());
        assert!(parse(br#"{"schemaVersion": 2}"#, None).is_err());
        let mismatched = image(OCI_IMAGE_CONFIG, "x", json!({}));
        assert!(parse(&mismatched, Some(DOCKER_MANIFEST)).is_err());
        assert!(parse(&mismatched, Some("application/json")).is_ok());
        let bad_digest = json!({
            "schemaVersion": 2,
            "mediaType": OCI_MANIFEST,
            "config": {"mediaType": OCI_IMAGE_CONFIG, "digest": "sha256:nope", "size": 2},
            "layers": [],
        });
        assert!(parse(&serde_json::to_vec(&bad_digest).unwrap(), None).is_err());
        let unknown = json!({"schemaVersion": 2, "mediaType": "application/vnd.example+json", "config": {"digest": digest("c")}});
        assert!(parse(&serde_json::to_vec(&unknown).unwrap(), None).is_err());
        let inferred = json!({"schemaVersion": 2, "manifests": []});
        assert_eq!(
            parse(&serde_json::to_vec(&inferred).unwrap(), None)
                .unwrap()
                .media_type,
            OCI_INDEX
        );
    }

    #[test]
    fn cosign_tags_name_their_kind() {
        assert_eq!(kind_for_tag(Kind::Image, "sha256-abc.sig"), Kind::Signature);
        assert_eq!(
            kind_for_tag(Kind::Other, "sha256-abc.att"),
            Kind::Attestation
        );
        assert_eq!(kind_for_tag(Kind::Other, "sha256-abc.sbom"), Kind::Sbom);
        assert_eq!(kind_for_tag(Kind::Helm, "x.sig"), Kind::Helm);
        assert_eq!(kind_for_tag(Kind::Image, "1.0"), Kind::Image);
        assert_eq!(bare("Application/JSON; charset=utf-8"), "application/json");
    }
}
