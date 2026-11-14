use std::fmt;

use serde::Deserialize;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde_json::Value;

const ALLOWED_LAYERS: [&str; 11] = [
    "application/vnd.dev.cosign.simplesigning.v1+json",
    "application/vnd.in-toto+json",
    "application/vnd.dsse.envelope.v1+json",
    "application/spdx+json",
    "text/spdx+json",
    "application/vnd.cyclonedx+json",
    "application/vnd.syft+json",
    "application/vnd.cncf.notary.signature",
    "application/vnd.oci.empty.v1+json",
    "application/jose+json",
    "application/cose",
];
const ALLOWED_LAYER_PREFIX: &str = "application/vnd.dev.sigstore.bundle";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const IN_TOTO: &str = "application/vnd.in-toto+json";
const DSSE: &str = "application/vnd.dsse.envelope.v1+json";
const CYCLONEDX: &str = "application/vnd.cyclonedx+json";
const SYFT: &str = "application/vnd.syft+json";
const TAR_BLOCK: usize = 512;
const TAR_CHECKSUM: std::ops::Range<usize> = 148..156;
pub const MAX_LAYER_BYTES: u64 = 1024 * 1024;
pub const MAX_LAYERS: usize = 20;

struct Pairs(Vec<(String, Value)>);

impl<'de> Deserialize<'de> for Pairs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Collect;

        impl<'de> Visitor<'de> for Collect {
            type Value = Pairs;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Pairs, A::Error> {
                let mut pairs = Vec::new();
                while let Some(pair) = map.next_entry::<String, Value>()? {
                    pairs.push(pair);
                }
                Ok(Pairs(pairs))
            }
        }

        deserializer.deserialize_map(Collect)
    }
}

#[derive(Deserialize)]
struct Probe {
    layers: Vec<Pairs>,
}

pub fn fold(key: &str) -> String {
    key.chars()
        .map(|c| match c {
            '\u{17f}' => 's',
            '\u{212a}' => 'k',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

fn distinct(pairs: &[(String, Value)]) -> bool {
    let mut names: Vec<String> = pairs.iter().map(|(key, _)| fold(key)).collect();
    names.sort_unstable();
    names.windows(2).all(|pair| pair[0] != pair[1])
}

fn exactly<'a>(pairs: &'a [(String, Value)], name: &str) -> Option<&'a Value> {
    let wanted = fold(name);
    let mut hits = pairs.iter().filter(|(key, _)| fold(key) == wanted);
    let first = hits.next()?;
    match hits.next() {
        None if first.0 == name => Some(&first.1),
        _ => None,
    }
}

pub struct Layer {
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}

fn allowed(layer: &Pairs) -> Option<Layer> {
    if !distinct(&layer.0) || layer.0.iter().any(|(key, _)| fold(key) == "urls") {
        return None;
    }
    let Some(Value::String(media_type)) = exactly(&layer.0, "mediaType") else {
        return None;
    };
    if !(ALLOWED_LAYERS.contains(&media_type.as_str())
        || media_type.starts_with(ALLOWED_LAYER_PREFIX))
    {
        return None;
    }
    let Some(Value::String(digest)) = exactly(&layer.0, "digest") else {
        return None;
    };
    let size = exactly(&layer.0, "size")?.as_u64()?;
    Some(Layer {
        media_type: media_type.clone(),
        digest: digest.clone(),
        size,
    })
}

pub fn signature_artifact(bytes: &[u8]) -> Option<Vec<Layer>> {
    let Pairs(top) = serde_json::from_slice::<Pairs>(bytes).ok()?;
    if !distinct(&top) || top.iter().any(|(key, _)| fold(key) == "manifests") {
        return None;
    }
    if !matches!(exactly(&top, "layers"), Some(Value::Array(_))) {
        return None;
    }
    let probe = serde_json::from_slice::<Probe>(bytes).ok()?;
    if probe.layers.len() > MAX_LAYERS {
        return None;
    }
    let layers: Option<Vec<Layer>> = probe.layers.iter().map(allowed).collect();
    layers.filter(|layers| !layers.is_empty())
}

fn octal(field: &[u8]) -> Option<u64> {
    let start = field.iter().position(|byte| !matches!(byte, b' ' | 0))?;
    let end = field.iter().rposition(|byte| !matches!(byte, b' ' | 0))?;
    field[start..=end]
        .iter()
        .try_fold(0u64, |value, byte| match byte {
            b'0'..=b'7' => value.checked_mul(8)?.checked_add(u64::from(byte - b'0')),
            _ => None,
        })
}

fn tar_header(block: &[u8]) -> bool {
    let (mut unsigned, mut signed) = (0u64, 0i64);
    for (index, byte) in block.iter().enumerate() {
        let counted = if TAR_CHECKSUM.contains(&index) {
            b' '
        } else {
            *byte
        };
        unsigned += u64::from(counted);
        signed += i64::from(counted as i8);
    }
    let sums = [Some(unsigned), u64::try_from(signed).ok()];
    let field = &block[TAR_CHECKSUM];
    if octal(field).is_some_and(|declared| sums.contains(&Some(declared))) {
        return true;
    }
    let digits: String = field
        .iter()
        .filter(|byte| (b'0'..=b'7').contains(byte))
        .map(|byte| char::from(*byte))
        .collect();
    sums.iter()
        .flatten()
        .any(|sum| digits.contains(&format!("{sum:o}")))
}

fn archive_like(bytes: &[u8]) -> bool {
    bytes
        .as_chunks::<TAR_BLOCK>()
        .0
        .iter()
        .any(|block| tar_header(block))
}

fn controlled(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .any(|byte| *byte < 0x20 && !matches!(byte, b'\t' | b'\n' | b'\r'))
}

fn shaped(media_type: &str, bytes: &[u8]) -> bool {
    let Ok(Value::Object(document)) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    let text = |key: &str| document.get(key).and_then(Value::as_str);
    let array = |key: &str| matches!(document.get(key), Some(Value::Array(_)));
    match media_type {
        SIMPLE_SIGNING => document
            .get("critical")
            .and_then(Value::as_object)
            .is_some_and(|critical| {
                critical.get("image").is_some_and(Value::is_object)
                    && critical.get("type").is_some_and(Value::is_string)
            }),
        other if other.starts_with(ALLOWED_LAYER_PREFIX) => {
            text("mediaType").is_some()
                && document
                    .get("verificationMaterial")
                    .is_some_and(Value::is_object)
        }
        IN_TOTO => {
            text("_type").is_some_and(|kind| kind.starts_with("https://in-toto.io/Statement/"))
                && array("subject")
                && text("predicateType").is_some()
        }
        DSSE => text("payloadType").is_some() && text("payload").is_some() && array("signatures"),
        "application/spdx+json" | "text/spdx+json" => {
            text("spdxVersion").is_some_and(|version| version.starts_with("SPDX-"))
        }
        CYCLONEDX => text("bomFormat") == Some("CycloneDX"),
        SYFT => array("artifacts") || document.contains_key("schema"),
        "application/vnd.cncf.notary.signature" | "application/jose+json" | "application/cose" => {
            text("payload").is_some() && text("protected").is_some() && text("signature").is_some()
        }
        _ => false,
    }
}

pub fn json_content(media_type: &str, bytes: &[u8]) -> bool {
    if media_type == EMPTY {
        return bytes == b"{}";
    }
    bytes.len() as u64 <= MAX_LAYER_BYTES
        && !controlled(bytes)
        && !archive_like(bytes)
        && shaped(media_type, bytes)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const SIGNATURE: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
    const IMAGE: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
    const CONFIG: &str = "application/vnd.oci.image.config.v1+json";

    fn layer(media_type: &str) -> Value {
        json!({"mediaType": media_type, "digest": "sha256:aa", "size": 1})
    }

    fn manifest(key: &str, layers: &[&str]) -> Vec<u8> {
        let layers: Vec<Value> = layers.iter().map(|media| layer(media)).collect();
        json!({"schemaVersion": 2, "config": {"mediaType": CONFIG}, key: layers, "subject": {"digest": "sha256:bb"}})
            .to_string()
            .into_bytes()
    }

    #[test]
    fn keys_fold_the_way_go_decoders_match_them() {
        for (key, folded) in [
            ("layers", "layers"),
            ("Layers", "layers"),
            ("LAYERS", "layers"),
            ("layer\u{17f}", "layers"),
            ("LAYER\u{17f}", "layers"),
            ("mediaType", "mediatype"),
            ("MEDIATYPE", "mediatype"),
            ("\u{212a}ey", "key"),
            ("layer", "layer"),
        ] {
            assert_eq!(fold(key), folded, "{key}");
        }
        assert_ne!(fold("layerz"), "layers");
        assert_ne!(fold("layers "), "layers");
    }

    #[test]
    fn only_manifests_made_of_signature_layers_are_exempt() {
        for media_type in [
            SIGNATURE,
            "application/vnd.dev.sigstore.bundle.v0.3+json",
            "application/vnd.dev.sigstore.bundle+json;version=0.3",
            "application/vnd.in-toto+json",
            "application/vnd.dsse.envelope.v1+json",
            "application/spdx+json",
            "text/spdx+json",
            "application/vnd.cyclonedx+json",
            "application/vnd.syft+json",
            "application/vnd.cncf.notary.signature",
            "application/vnd.oci.empty.v1+json",
        ] {
            assert!(
                signature_artifact(&manifest("layers", &[media_type])).is_some(),
                "{media_type}"
            );
        }
        let layers =
            signature_artifact(&manifest("layers", &[SIGNATURE, "application/spdx+json"])).unwrap();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].media_type, SIGNATURE);
        assert_eq!(
            (layers[0].digest.as_str(), layers[0].size),
            ("sha256:aa", 1)
        );
    }

    #[test]
    fn everything_else_is_judged_as_the_image_it_may_be() {
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("image layer", manifest("layers", &[IMAGE])),
            ("mixed layers", manifest("layers", &[SIGNATURE, IMAGE])),
            (
                "wasm",
                manifest("layers", &["application/vnd.wasm.content.layer.v1+wasm"]),
            ),
            (
                "helm",
                manifest(
                    "layers",
                    &["application/vnd.cncf.helm.chart.content.v1.tar+gzip"],
                ),
            ),
            (
                "flux",
                manifest("layers", &["application/vnd.cncf.flux.content.v1.tar+gzip"]),
            ),
            ("no layers", manifest("layers", &[])),
            ("capitalised", manifest("Layers", &[IMAGE])),
            ("upper case", manifest("LAYERS", &[IMAGE])),
            ("long s", manifest("layer\u{17f}", &[IMAGE])),
            ("capitalised signature", manifest("Layers", &[SIGNATURE])),
            (
                "upper case case media type",
                manifest("layers", &["APPLICATION/VND.OCI.IMAGE.LAYER.V1.TAR"]),
            ),
            ("not json", b"layers".to_vec()),
            ("an array", b"[]".to_vec()),
            ("an empty object", b"{}".to_vec()),
            (
                "layers is not a list",
                br#"{"layers": {"mediaType": "application/spdx+json"}}"#.to_vec(),
            ),
            (
                "a layer that is not an object",
                br#"{"layers": ["application/spdx+json"]}"#.to_vec(),
            ),
        ];
        for (label, bytes) in cases {
            assert!(signature_artifact(&bytes).is_none(), "{label}");
        }
    }

    #[test]
    fn keys_that_collide_or_hide_another_spelling_are_never_exempt() {
        let allowed = format!(r#"[{{"mediaType": "{SIGNATURE}"}}]"#);
        let image = format!(r#"[{{"mediaType": "{IMAGE}"}}]"#);
        let cases = [
            (
                "two layers keys",
                format!(r#"{{"layers": {allowed}, "layers": {image}}}"#),
            ),
            (
                "two spellings",
                format!(r#"{{"layers": {allowed}, "Layers": {image}}}"#),
            ),
            (
                "a long s twin",
                format!(r#"{{"layers": {allowed}, "layerſ": {image}}}"#),
            ),
            (
                "an index",
                format!(r#"{{"layers": {allowed}, "manifests": []}}"#),
            ),
            (
                "a capitalised index",
                format!(r#"{{"layers": {allowed}, "Manifests": []}}"#),
            ),
            (
                "a media type twin",
                format!(
                    r#"{{"layers": [{{"mediaType": "{SIGNATURE}", "MediaType": "{IMAGE}"}}]}}"#
                ),
            ),
            (
                "a doubled media type",
                format!(
                    r#"{{"layers": [{{"mediaType": "{SIGNATURE}", "mediaType": "{IMAGE}"}}]}}"#
                ),
            ),
            (
                "a capitalised media type",
                format!(r#"{{"layers": [{{"MediaType": "{SIGNATURE}"}}]}}"#),
            ),
            (
                "no media type",
                r#"{"layers": [{"digest": "sha256:aa"}]}"#.to_owned(),
            ),
            (
                "a media type that is not text",
                r#"{"layers": [{"mediaType": 5}]}"#.to_owned(),
            ),
        ];
        for (label, text) in cases {
            assert!(signature_artifact(text.as_bytes()).is_none(), "{label}");
        }
        let whole = format!(
            r#"{{"layers": [{{"mediaType": "{SIGNATURE}", "digest": "sha256:aa", "size": 1}}]}}"#
        );
        assert!(signature_artifact(whole.as_bytes()).is_some());
    }

    #[test]
    fn foreign_or_incomplete_layer_descriptors_are_never_exempt() {
        for key in ["urls", "URLS", "Urls"] {
            let text = format!(
                r#"{{"layers": [{{"mediaType": "{SIGNATURE}", "digest": "sha256:aa", "size": 1, "{key}": ["https://x.test/a"]}}]}}"#
            );
            assert!(signature_artifact(text.as_bytes()).is_none(), "{key}");
        }
        let without_size =
            format!(r#"{{"layers": [{{"mediaType": "{SIGNATURE}", "digest": "sha256:aa"}}]}}"#);
        assert!(signature_artifact(without_size.as_bytes()).is_none());
        let many = |count: usize| {
            let layers: Vec<Value> = (0..count).map(|_| layer(SIGNATURE)).collect();
            json!({"layers": layers}).to_string().into_bytes()
        };
        assert_eq!(
            signature_artifact(&many(MAX_LAYERS)).unwrap().len(),
            MAX_LAYERS
        );
        assert!(signature_artifact(&many(MAX_LAYERS + 1)).is_none());
        assert!(signature_artifact(&many(2000)).is_none());
    }

    const BUNDLE: &str = "application/vnd.dev.sigstore.bundle.v0.3+json";
    const DSSE_LABEL: &str = "application/vnd.dsse.envelope.v1+json";
    const SPDX: &str = "application/spdx+json";
    const CYCLONEDX_LABEL: &str = "application/vnd.cyclonedx+json";
    const SYFT_LABEL: &str = "application/vnd.syft+json";
    const NOTARY: &str = "application/vnd.cncf.notary.signature";

    fn documents() -> Vec<(&'static str, &'static str)> {
        let statement = r#"{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"x"}],"predicateType":"https://slsa.dev/provenance/v1","predicate":{}}"#;
        let jws = r#"{"payload":"e30","protected":"e30","header":{},"signature":"AA"}"#;
        vec![
            (
                SIGNATURE,
                r#"{"critical":{"identity":{"docker-reference":"x"},"image":{"docker-manifest-digest":"sha256:aa"},"type":"cosign container image signature"},"optional":null}"#,
            ),
            (
                BUNDLE,
                r#"{"mediaType":"application/vnd.dev.sigstore.bundle.v0.3+json","verificationMaterial":{},"dsseEnvelope":{}}"#,
            ),
            (IN_TOTO, statement),
            (
                DSSE_LABEL,
                r#"{"payloadType":"application/vnd.in-toto+json","payload":"e30=","signatures":[{"sig":"AA=="}]}"#,
            ),
            (
                SPDX,
                r#"{"spdxVersion":"SPDX-2.3","SPDXID":"SPDXRef-DOCUMENT"}"#,
            ),
            ("text/spdx+json", r#"{"spdxVersion":"SPDX-2.2"}"#),
            (
                CYCLONEDX_LABEL,
                r#"{"bomFormat":"CycloneDX","specVersion":"1.5"}"#,
            ),
            (SYFT_LABEL, r#"{"artifacts":[],"schema":{"version":"1"}}"#),
            (SYFT_LABEL, r#"{"schema":{"version":"1"}}"#),
            (NOTARY, jws),
            ("application/jose+json", jws),
            ("application/cose", jws),
            (EMPTY, "{}"),
        ]
    }

    #[test]
    fn blobs_must_have_the_shape_their_label_promises() {
        for (label, document) in documents() {
            assert!(json_content(label, document.as_bytes()), "{label}");
        }
        let spdx = r#"{"spdxVersion":"SPDX-2.3"}"#;
        let statement = documents()[2].1;
        let wrong = [
            (IN_TOTO, spdx),
            (DSSE_LABEL, statement),
            (SIGNATURE, "{}"),
            (SIGNATURE, r#"{"critical":{}}"#),
            (SIGNATURE, r#"{"critical":{"image":{}}}"#),
            (SIGNATURE, r#"{"critical":"x"}"#),
            (BUNDLE, r#"{"mediaType":"x"}"#),
            (BUNDLE, r#"{"verificationMaterial":{}}"#),
            (
                IN_TOTO,
                r#"{"_type":"https://example.test/Statement/v1","subject":[],"predicateType":"p"}"#,
            ),
            (
                IN_TOTO,
                r#"{"_type":"https://in-toto.io/Statement/v1","subject":{},"predicateType":"p"}"#,
            ),
            (DSSE_LABEL, r#"{"payloadType":"a","payload":"b"}"#),
            (SPDX, r#"{"spdxVersion":"1.0"}"#),
            (SPDX, r#"{"spdxVersion":2}"#),
            (CYCLONEDX_LABEL, r#"{"bomFormat":"SPDX"}"#),
            (SYFT_LABEL, "{}"),
            (NOTARY, r#"{"payload":"a"}"#),
            (
                "application/jose+json",
                r#"{"payload":"a","protected":"b","signature":3}"#,
            ),
            (SPDX, "[]"),
            (SPDX, "\"SPDX-2.3\""),
            (SPDX, "not json"),
            (SPDX, ""),
            (EMPTY, "{ }"),
            (EMPTY, spdx),
            ("application/octet-stream", spdx),
            ("application/vnd.oci.image.layer.v1.tar+gzip", spdx),
        ];
        for (label, bytes) in wrong {
            assert!(!json_content(label, bytes.as_bytes()), "{label}: {bytes}");
        }
        let gzip = [
            0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x4b,
        ];
        assert!(!json_content(SPDX, &gzip));
        assert!(!json_content(SPDX, b"{\"spdxVersion\":\"SPDX-2.3\"}\0"));
        assert!(!json_content(
            SPDX,
            b"{\"spdxVersion\":\"SPDX-2.3\",\x0b\"a\":1}"
        ));
        assert!(json_content(
            SPDX,
            b" \t\r\n{\"spdxVersion\":\"SPDX-2.3\"}\n"
        ));
    }

    type Field = fn(&str) -> String;

    fn header_with(name: &str, field: Field) -> Vec<u8> {
        assert!(name.len() <= 100);
        let mut block = vec![b'a'; TAR_BLOCK];
        block[..name.len()].copy_from_slice(name.as_bytes());
        let mut put =
            |at: usize, text: &str| block[at..at + text.len()].copy_from_slice(text.as_bytes());
        put(100, "0000755 ");
        put(108, "0000000 ");
        put(116, "0000000 ");
        put(124, "00000001000 ");
        put(136, "00000000000 ");
        put(148, "        ");
        block[156] = b'0';
        let sum: u32 = block.iter().map(|byte| u32::from(*byte)).sum();
        let text = field(&format!("{sum:o}"));
        assert_eq!(text.len(), 8, "{text}");
        block[148..156].copy_from_slice(text.as_bytes());
        block
    }

    fn exact(sum: &str) -> String {
        format!("{sum:0>7} ")
    }

    fn header(name: &str) -> Vec<u8> {
        header_with(name, exact)
    }

    fn polyglot_with(prefix: &str, offset: usize, field: Field) -> Vec<u8> {
        let mut blob = Vec::new();
        match offset {
            0 => {
                blob.extend(header_with(prefix, field));
                blob.extend(vec![b'b'; TAR_BLOCK]);
            }
            _ => {
                let mut first = vec![b'x'; TAR_BLOCK];
                first[..prefix.len()].copy_from_slice(prefix.as_bytes());
                blob.extend(first);
                blob.extend(header_with("", field));
            }
        }
        blob.extend(b"\"}");
        blob
    }

    fn polyglot(prefix: &str, offset: usize) -> Vec<u8> {
        polyglot_with(prefix, offset, exact)
    }

    #[test]
    fn a_json_document_that_is_also_a_tar_archive_is_never_exempt() {
        let plain = header("a");
        assert!(tar_header(&plain));
        let mut damaged = plain.clone();
        damaged[10] ^= 1;
        assert!(!tar_header(&damaged));
        let mut unsigned = plain.clone();
        unsigned[148..156].copy_from_slice(b"0000000 ");
        assert!(!tar_header(&unsigned));
        let mut signed = plain.clone();
        signed[0] = 0xc3;
        let sum: i64 = signed
            .iter()
            .enumerate()
            .map(|(index, byte)| {
                if (148..156).contains(&index) {
                    32
                } else {
                    i64::from(*byte as i8)
                }
            })
            .sum();
        signed[148..156].copy_from_slice(format!("{sum:07o} ").as_bytes());
        assert!(tar_header(&signed));
        assert!(!tar_header(&[0u8; TAR_BLOCK]));

        let prefixes = [
            (SIGNATURE, r#"{"critical":{"image":{},"type":"t"},"k":""#),
            (
                BUNDLE,
                r#"{"mediaType":"m","verificationMaterial":{},"k":""#,
            ),
            (
                IN_TOTO,
                r#"{"_type":"https://in-toto.io/Statement/v1","subject":[],"predicateType":"p","k":""#,
            ),
            (
                DSSE_LABEL,
                r#"{"payloadType":"a","payload":"b","signatures":[],"k":""#,
            ),
            (SPDX, r#"{"spdxVersion":"SPDX-2.3","k":""#),
            ("text/spdx+json", r#"{"spdxVersion":"SPDX-2.3","k":""#),
            (CYCLONEDX_LABEL, r#"{"bomFormat":"CycloneDX","k":""#),
            (SYFT_LABEL, r#"{"schema":{},"k":""#),
            (
                NOTARY,
                r#"{"payload":"a","protected":"b","signature":"c","k":""#,
            ),
            (
                "application/jose+json",
                r#"{"payload":"a","protected":"b","signature":"c","k":""#,
            ),
            (
                "application/cose",
                r#"{"payload":"a","protected":"b","signature":"c","k":""#,
            ),
        ];
        for (label, prefix) in prefixes {
            for offset in [0, TAR_BLOCK] {
                let blob = polyglot(prefix, offset);
                assert!(
                    serde_json::from_slice::<Value>(&blob).is_ok(),
                    "{label} {offset}"
                );
                assert!(!controlled(&blob), "{label} {offset}");
                assert!(shaped(label, &blob), "{label} {offset}");
                assert!(archive_like(&blob), "{label} {offset}");
                assert!(!json_content(label, &blob), "{label} {offset}");
            }
        }
    }

    #[test]
    fn lenient_checksum_fields_do_not_hide_a_tar_header() {
        let variants: [(&str, Field); 4] = [
            ("garbage after a space", |sum| format!("{sum} 7")),
            ("a plus sign", |sum| format!("+{sum} ")),
            ("a 0o prefix", |sum| format!("0o{sum}")),
            ("an underscore", |sum| {
                format!("{}_{} ", &sum[..3], &sum[3..])
            }),
        ];
        let labels = [
            (SIGNATURE, r#"{"critical":{"image":{},"type":"t"},"k":""#),
            (SPDX, r#"{"spdxVersion":"SPDX-2.3","k":""#),
        ];
        for (name, field) in variants {
            for (label, prefix) in labels {
                for offset in [0, TAR_BLOCK] {
                    let blob = polyglot_with(prefix, offset, field);
                    assert!(serde_json::from_slice::<Value>(&blob).is_ok(), "{name}");
                    assert!(shaped(label, &blob), "{name}");
                    assert!(archive_like(&blob), "{name} {label} {offset}");
                    assert!(!json_content(label, &blob), "{name} {label} {offset}");
                }
            }
        }
        for (label, document) in documents() {
            let mut padded = document.as_bytes().to_vec();
            padded.resize(2048, b' ');
            assert!(json_content(label, &padded) || label == EMPTY, "{label}");
        }
        let plain = br#"{"spdxVersion":"SPDX-2.3"}"#;
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(plain);
        assert!(!json_content(SPDX, &bom));
        assert!(!json_content(SPDX, &[b"junk".as_slice(), plain].concat()));
    }
}
