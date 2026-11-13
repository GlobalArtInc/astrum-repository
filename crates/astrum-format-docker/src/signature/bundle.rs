use serde::Deserialize;
use serde::de::IgnoredAny;
use sha2::{Digest, Sha256};

use super::json::{JsonError, Object, parse_object};
use super::simple::check_payload;
use super::{
    Claim, MAX_BUNDLE_BYTES, MAX_PAYLOAD_BYTES, MAX_SIGNATURE_BYTES, Outcome, PublicKey,
    decode_base64, first_verifying, manifest_hex, quote,
};

const MEDIA_TYPES: [&str; 2] = [
    "application/vnd.dev.sigstore.bundle.v0.3+json",
    "application/vnd.dev.sigstore.bundle+json;version=0.3",
];
const PAYLOAD_SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const PAYLOAD_IN_TOTO: &str = "application/vnd.in-toto+json";
const STATEMENT_TYPE_PREFIX: &str = "https://in-toto.io/Statement/";
const DIGEST_ALGORITHM_SHA256: &str = "SHA2_256";
const MAX_ENVELOPE_SIGNATURES: usize = 16;
const SHA256_LEN: usize = 32;

#[derive(Deserialize)]
struct Bundle {
    #[serde(rename = "mediaType")]
    media_type: Option<String>,
    #[serde(rename = "verificationMaterial")]
    material: Option<Object<Material>>,
    #[serde(rename = "dsseEnvelope")]
    envelope: Option<Object<Envelope>>,
    #[serde(rename = "messageSignature")]
    message_signature: Option<Object<MessageSignature>>,
}

#[derive(Deserialize)]
struct Material {
    certificate: Option<IgnoredAny>,
    #[serde(rename = "x509CertificateChain")]
    certificate_chain: Option<IgnoredAny>,
}

#[derive(Deserialize)]
struct Envelope {
    payload: Option<String>,
    #[serde(rename = "payloadType")]
    payload_type: Option<String>,
    #[serde(default)]
    signatures: Vec<Object<EnvelopeSignature>>,
}

#[derive(Deserialize)]
struct EnvelopeSignature {
    sig: Option<String>,
}

#[derive(Deserialize)]
struct MessageSignature {
    #[serde(rename = "messageDigest")]
    digest: Option<Object<HashOutput>>,
    signature: Option<String>,
}

#[derive(Deserialize)]
struct HashOutput {
    algorithm: Option<String>,
    digest: Option<String>,
}

#[derive(Deserialize)]
struct Statement {
    #[serde(rename = "_type")]
    kind: String,
    subject: Vec<Object<Subject>>,
}

#[derive(Deserialize)]
struct Subject {
    digest: Option<Object<SubjectDigest>>,
}

#[derive(Deserialize)]
struct SubjectDigest {
    sha256: Option<String>,
}

#[derive(Clone, Copy)]
enum EnvelopeKind {
    SimpleSigning,
    Statement,
}

pub fn verify_bundle(
    bundle_json: &[u8],
    message: Option<&[u8]>,
    keys: &[PublicKey],
    manifest_digest: &str,
) -> Outcome {
    let hex = match manifest_hex(manifest_digest) {
        Ok(hex) => hex,
        Err(outcome) => return outcome,
    };
    if keys.is_empty() {
        return Outcome::Invalid("no trusted keys".to_owned());
    }
    if bundle_json.len() > MAX_BUNDLE_BYTES {
        return Outcome::Invalid(format!("bundle is larger than {MAX_BUNDLE_BYTES} bytes"));
    }
    let bundle: Bundle = match parse_object(bundle_json) {
        Ok(bundle) => bundle,
        Err(error) => {
            return Outcome::Invalid(
                match error {
                    JsonError::NotObject => "the bundle is not a JSON object",
                    JsonError::Syntax => "bundle is not valid JSON",
                    JsonError::Structure => "bundle does not have the expected structure",
                }
                .to_owned(),
            );
        }
    };
    let Some(media_type) = bundle.media_type.as_deref() else {
        return Outcome::Invalid("bundle has no mediaType".to_owned());
    };
    if !MEDIA_TYPES.contains(&media_type) {
        return Outcome::Unsupported(format!(
            "bundle media type {} is not supported",
            quote(media_type)
        ));
    }
    let Some(material) = bundle.material else {
        return Outcome::Invalid("bundle has no verificationMaterial".to_owned());
    };
    if material.0.certificate.is_some() || material.0.certificate_chain.is_some() {
        return Outcome::Unsupported("keyless signatures are not supported".to_owned());
    }
    match (bundle.envelope, bundle.message_signature) {
        (Some(envelope), None) => verify_envelope(&envelope.0, keys, manifest_digest, hex),
        (None, Some(signature)) => verify_message(&signature.0, message, keys, manifest_digest),
        (None, None) => Outcome::Invalid("bundle has no content".to_owned()),
        (Some(_), Some(_)) => Outcome::Invalid("bundle has two kinds of content".to_owned()),
    }
}

fn verify_envelope(
    envelope: &Envelope,
    keys: &[PublicKey],
    manifest_digest: &str,
    hex: &str,
) -> Outcome {
    let Some(payload_type) = envelope.payload_type.as_deref() else {
        return Outcome::Invalid("envelope has no payloadType".to_owned());
    };
    let kind = match payload_type {
        PAYLOAD_SIMPLE_SIGNING => EnvelopeKind::SimpleSigning,
        PAYLOAD_IN_TOTO => EnvelopeKind::Statement,
        other => {
            return Outcome::Unsupported(format!(
                "envelope payload type {} is not supported",
                quote(other)
            ));
        }
    };
    if envelope.signatures.is_empty() {
        return Outcome::Invalid("envelope has no signatures".to_owned());
    }
    if envelope.signatures.len() > MAX_ENVELOPE_SIGNATURES {
        return Outcome::Invalid(format!(
            "envelope has more than {MAX_ENVELOPE_SIGNATURES} signatures"
        ));
    }
    let Some(encoded_payload) = envelope.payload.as_deref() else {
        return Outcome::Invalid("envelope has no payload".to_owned());
    };
    let payload = match decode_base64(encoded_payload, MAX_PAYLOAD_BYTES, "envelope payload") {
        Ok(payload) => payload,
        Err(reason) => return Outcome::Invalid(reason),
    };
    let mut signatures = Vec::with_capacity(envelope.signatures.len());
    for entry in &envelope.signatures {
        let Some(encoded) = entry.0.sig.as_deref() else {
            return Outcome::Invalid("envelope signature has no sig".to_owned());
        };
        match decode_base64(encoded, MAX_SIGNATURE_BYTES, "envelope signature") {
            Ok(signature) => signatures.push(signature),
            Err(reason) => return Outcome::Invalid(reason),
        }
    }
    let signed = pre_authentication_encoding(payload_type, &payload);
    let Some(key) = first_verifying(keys, &signed, &signatures) else {
        return Outcome::Invalid("no trusted key verifies the signature".to_owned());
    };
    let (checked, claim) = match kind {
        EnvelopeKind::SimpleSigning => (
            check_payload(&payload, manifest_digest),
            Claim::ImageSignature,
        ),
        EnvelopeKind::Statement => (
            check_statement(&payload, manifest_digest, hex),
            Claim::Attestation,
        ),
    };
    match checked {
        Ok(()) => Outcome::Verified {
            key: key.fingerprint().to_owned(),
            claim,
        },
        Err(reason) => Outcome::Invalid(reason),
    }
}

fn verify_message(
    signature: &MessageSignature,
    message: Option<&[u8]>,
    keys: &[PublicKey],
    manifest_digest: &str,
) -> Outcome {
    let Some(encoded) = signature.signature.as_deref() else {
        return Outcome::Invalid("message signature has no signature".to_owned());
    };
    let decoded = match decode_base64(encoded, MAX_SIGNATURE_BYTES, "message signature") {
        Ok(decoded) => decoded,
        Err(reason) => return Outcome::Invalid(reason),
    };
    let Some(message) = message else {
        return Outcome::Unsupported("the signed bytes are not available".to_owned());
    };
    if message.len() > MAX_PAYLOAD_BYTES {
        return Outcome::Invalid(format!("message is larger than {MAX_PAYLOAD_BYTES} bytes"));
    }
    let Some(key) = first_verifying(keys, message, std::slice::from_ref(&decoded)) else {
        return Outcome::Invalid("no trusted key verifies the signature".to_owned());
    };
    if let Some(declared) = signature.digest.as_ref().map(|declared| &declared.0) {
        match declared.algorithm.as_deref() {
            Some(DIGEST_ALGORITHM_SHA256) => {}
            Some(other) => {
                return Outcome::Unsupported(format!(
                    "message digest algorithm {} is not supported",
                    quote(other)
                ));
            }
            None => {
                return Outcome::Unsupported("message digest algorithm is missing".to_owned());
            }
        }
        let Some(encoded_digest) = declared.digest.as_deref() else {
            return Outcome::Invalid("messageDigest has no digest".to_owned());
        };
        match decode_base64(encoded_digest, SHA256_LEN, "messageDigest") {
            Ok(digest) if digest.as_slice() == Sha256::digest(message).as_slice() => {}
            Ok(_) => {
                return Outcome::Invalid(
                    "messageDigest does not match the supplied message".to_owned(),
                );
            }
            Err(reason) => return Outcome::Invalid(reason),
        }
    }
    match check_payload(message, manifest_digest) {
        Ok(()) => Outcome::Verified {
            key: key.fingerprint().to_owned(),
            claim: Claim::ImageSignature,
        },
        Err(reason) => Outcome::Invalid(reason),
    }
}

fn check_statement(payload: &[u8], manifest_digest: &str, hex: &str) -> Result<(), String> {
    let statement: Statement = parse_object(payload).map_err(|error| {
        match error {
            JsonError::NotObject => "the payload is not a JSON object",
            JsonError::Syntax => "payload is not valid JSON",
            JsonError::Structure => "payload does not have the in-toto statement structure",
        }
        .to_owned()
    })?;
    if !statement.kind.starts_with(STATEMENT_TYPE_PREFIX) {
        return Err(format!(
            "payload type {} is not an in-toto statement",
            quote(&statement.kind)
        ));
    }
    let names_digest = statement
        .subject
        .iter()
        .filter_map(|subject| subject.0.digest.as_ref())
        .any(|digest| digest.0.sha256.as_deref() == Some(hex));
    if names_digest {
        Ok(())
    } else {
        Err(format!("no statement subject names {manifest_digest}"))
    }
}

fn pre_authentication_encoding(payload_type: &str, body: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(
        payload_type
            .len()
            .saturating_add(body.len())
            .saturating_add(48),
    );
    encoded.extend_from_slice(b"DSSEv1 ");
    encoded.extend_from_slice(payload_type.len().to_string().as_bytes());
    encoded.push(b' ');
    encoded.extend_from_slice(payload_type.as_bytes());
    encoded.push(b' ');
    encoded.extend_from_slice(body.len().to_string().as_bytes());
    encoded.push(b' ');
    encoded.extend_from_slice(body);
    encoded
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::signature::tests::{
        DIGEST, OTHER_DIGEST, bundle_json, is_invalid, is_unsupported, key, read, signer, verified,
    };
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    fn edited(name: &str, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
        let mut bundle = bundle_json(name);
        edit(&mut bundle);
        serde_json::to_vec(&bundle).unwrap()
    }

    #[test]
    fn valid_bundles_verify() {
        let payload = read("payload.json");
        let cases = [
            (
                "ed25519-dsse-simple.bundle.json",
                "ed25519",
                None,
                Claim::ImageSignature,
            ),
            ("p256-intoto.bundle.json", "p256", None, Claim::Attestation),
            (
                "p256-message.bundle.json",
                "p256",
                Some(payload.as_slice()),
                Claim::ImageSignature,
            ),
        ];
        for (name, trusted, message, claim) in cases {
            let trusted = key(trusted);
            let outcome =
                verify_bundle(&read(name), message, std::slice::from_ref(&trusted), DIGEST);
            assert_eq!(outcome, verified(&trusted, claim), "{name}");
        }
        let alternative = edited("p256-intoto.bundle.json", |bundle| {
            bundle["mediaType"] = json!(MEDIA_TYPES[1]);
        });
        let trusted = key("p256");
        assert_eq!(
            verify_bundle(&alternative, None, std::slice::from_ref(&trusted), DIGEST),
            verified(&trusted, Claim::Attestation)
        );
    }

    #[test]
    fn bundles_outside_the_supported_set() {
        let payload = read("payload.json");
        let bad_media = edited("p256-intoto.bundle.json", |b| {
            b["mediaType"] = json!("text/plain")
        });
        let old_media = edited("p256-intoto.bundle.json", |b| {
            b["mediaType"] = json!("application/vnd.dev.sigstore.bundle+json;version=0.2");
        });
        let sha512 = edited("p256-message.bundle.json", |b| {
            b["messageSignature"]["messageDigest"]["algorithm"] = json!("SHA2_512");
        });
        let cases = [
            (
                "message without bytes",
                read("p256-message.bundle.json"),
                None,
                "p256",
                "signed bytes are not available",
            ),
            (
                "keyless",
                read("keyless-message.bundle.json"),
                Some(payload.as_slice()),
                "p256",
                "keyless signatures are not supported",
            ),
            (
                "other payload type",
                read("ed25519-dsse-other.bundle.json"),
                None,
                "ed25519",
                "payload type",
            ),
            ("bad media type", bad_media, None, "p256", "media type"),
            ("old media type", old_media, None, "p256", "media type"),
            (
                "other digest algorithm",
                sha512,
                Some(payload.as_slice()),
                "p256",
                "SHA2_512",
            ),
        ];
        for (name, bundle, message, trusted, reason) in cases {
            let outcome = verify_bundle(&bundle, message, &[key(trusted)], DIGEST);
            assert!(is_unsupported(&outcome, reason), "{name}: {outcome:?}");
        }
    }

    #[test]
    fn bundles_that_must_not_be_accepted() {
        let payload = read("payload.json");
        let mut flipped = payload.clone();
        flipped[10] ^= 1;
        let retyped = edited("p256-intoto.bundle.json", |b| {
            b["dsseEnvelope"]["payloadType"] = json!(PAYLOAD_SIMPLE_SIGNING);
        });
        let wrong_digest = edited("p256-message.bundle.json", |b| {
            b["messageSignature"]["messageDigest"]["digest"] =
                json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
        });
        let many = edited("p256-intoto.bundle.json", |b| {
            let sig = b["dsseEnvelope"]["signatures"][0].clone();
            b["dsseEnvelope"]["signatures"] = Value::Array(vec![sig; MAX_ENVELOPE_SIGNATURES + 1]);
        });
        let both = edited("p256-intoto.bundle.json", |b| {
            b["messageSignature"] = json!({"signature": "AAAA"});
        });
        let no_content = edited("p256-intoto.bundle.json", |b| {
            b.as_object_mut().unwrap().remove("dsseEnvelope");
        });
        let no_media = edited("p256-intoto.bundle.json", |b| {
            b.as_object_mut().unwrap().remove("mediaType");
        });
        let no_material = edited("p256-intoto.bundle.json", |b| {
            b.as_object_mut().unwrap().remove("verificationMaterial");
        });
        let intoto = || read("p256-intoto.bundle.json");
        let message = || read("p256-message.bundle.json");
        let signed = Some(payload.as_slice());
        let cases = [
            (
                "key not in the set",
                intoto(),
                None,
                "p384",
                DIGEST,
                "no trusted key",
            ),
            (
                "statement names another digest",
                intoto(),
                None,
                "p256",
                OTHER_DIGEST,
                "no statement subject",
            ),
            (
                "message names another digest",
                message(),
                signed,
                "p256",
                OTHER_DIGEST,
                "payload names",
            ),
            (
                "tampered message",
                message(),
                Some(flipped.as_slice()),
                "p256",
                DIGEST,
                "no trusted key",
            ),
            (
                "payload type is signed",
                retyped,
                None,
                "p256",
                DIGEST,
                "no trusted key",
            ),
            (
                "message digest differs",
                wrong_digest,
                signed,
                "p256",
                DIGEST,
                "messageDigest does not match",
            ),
            (
                "too many signatures",
                many,
                None,
                "p256",
                DIGEST,
                "more than",
            ),
            (
                "two kinds of content",
                both,
                None,
                "p256",
                DIGEST,
                "two kinds",
            ),
            ("no content", no_content, None, "p256", DIGEST, "no content"),
            (
                "no media type",
                no_media,
                None,
                "p256",
                DIGEST,
                "no mediaType",
            ),
            (
                "no verification material",
                no_material,
                None,
                "p256",
                DIGEST,
                "verificationMaterial",
            ),
            (
                "not json",
                b"{".to_vec(),
                None,
                "p256",
                DIGEST,
                "not valid JSON",
            ),
            (
                "wrong shape",
                b"[1]".to_vec(),
                None,
                "p256",
                DIGEST,
                "not a JSON object",
            ),
            (
                "positional bundle",
                b"[\"application/vnd.dev.sigstore.bundle.v0.3+json\"]".to_vec(),
                None,
                "p256",
                DIGEST,
                "not a JSON object",
            ),
            (
                "positional envelope",
                edited("p256-intoto.bundle.json", |b| {
                    b["dsseEnvelope"] = json!(["AA==", "application/vnd.in-toto+json", []]);
                }),
                None,
                "p256",
                DIGEST,
                "expected structure",
            ),
            (
                "oversized",
                vec![b' '; MAX_BUNDLE_BYTES + 1],
                None,
                "p256",
                DIGEST,
                "bundle is larger",
            ),
            (
                "bad digest argument",
                intoto(),
                None,
                "p256",
                "sha256:AB",
                "manifest digest",
            ),
        ];
        for (name, bundle, message, trusted, digest, reason) in cases {
            let outcome = verify_bundle(&bundle, message, &[key(trusted)], digest);
            assert!(is_invalid(&outcome, reason), "{name}: {outcome:?}");
        }
    }

    #[test]
    fn the_statement_must_name_the_digest() {
        let hex = DIGEST.trim_start_matches("sha256:");
        let statement = |kind: &str, subject: Value| {
            serde_json::to_vec(&json!({"_type": kind, "subject": subject})).unwrap()
        };
        let v1 = "https://in-toto.io/Statement/v1";
        let other = "https://example.test/other";
        let positional = format!(r#"["{v1}",[[["{hex}"]]]]"#).into_bytes();
        let nested = format!(r#"{{"_type":"{v1}","subject":[[["{hex}"]]]}}"#).into_bytes();
        let cases = [
            (b"{".to_vec(), "not valid JSON"),
            (b"nope".to_vec(), "not a JSON object"),
            (positional, "not a JSON object"),
            (nested, "in-toto statement structure"),
            (b"{}".to_vec(), "statement structure"),
            (
                statement(other, json!([{"digest": {"sha256": hex}}])),
                "not an in-toto statement",
            ),
            (statement(v1, json!([])), "no statement subject"),
            (
                statement(v1, json!([{"name": "x"}, {"digest": {"sha512": "ab"}}])),
                "no statement subject",
            ),
            (
                statement(v1, json!([{"digest": {"sha256": hex.to_uppercase()}}])),
                "no statement subject",
            ),
        ];
        for (bytes, reason) in cases {
            let error = check_statement(&bytes, DIGEST, hex).unwrap_err();
            assert!(error.contains(reason), "{error}");
        }
        let second = statement(
            v1,
            json!([{"digest": {"sha256": "00"}}, {"digest": {"sha256": hex}}]),
        );
        assert_eq!(check_statement(&second, DIGEST, hex), Ok(()));
    }

    #[test]
    fn pre_authentication_encoding_follows_the_dsse_layout() {
        assert_eq!(
            pre_authentication_encoding("text/x", b"hello"),
            b"DSSEv1 6 text/x 5 hello".to_vec()
        );
        assert_eq!(
            pre_authentication_encoding("", b""),
            b"DSSEv1 0  0 ".to_vec()
        );
    }

    #[test]
    fn an_envelope_signed_by_a_trusted_key_is_judged_by_its_payload() {
        let (pair, trusted) = signer();
        let hex = DIGEST.trim_start_matches("sha256:");
        let v1 = "https://in-toto.io/Statement/v1";
        let signed = |payload_type: &str, body: &str| {
            let signature = pair.sign(&pre_authentication_encoding(payload_type, body.as_bytes()));
            let bundle = json!({
                "mediaType": MEDIA_TYPES[0],
                "verificationMaterial": {},
                "dsseEnvelope": {
                    "payload": STANDARD.encode(body),
                    "payloadType": payload_type,
                    "signatures": [{"sig": STANDARD.encode(signature.as_ref())}],
                },
            });
            serde_json::to_vec(&bundle).unwrap()
        };
        let statement = json!({"_type": v1, "subject": [{"digest": {"sha256": hex}}]}).to_string();
        let positional = format!(r#"["{v1}",[[["{hex}"]]]]"#);
        let cases = [
            (
                signed(PAYLOAD_IN_TOTO, &statement),
                verified(&trusted, Claim::Attestation),
            ),
            (
                signed(PAYLOAD_IN_TOTO, &positional),
                Outcome::Invalid("the payload is not a JSON object".to_owned()),
            ),
        ];
        for (bundle, expected) in cases {
            let outcome = verify_bundle(&bundle, None, std::slice::from_ref(&trusted), DIGEST);
            assert_eq!(outcome, expected);
        }
    }
}
