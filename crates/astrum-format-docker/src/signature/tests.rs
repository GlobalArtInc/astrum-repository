use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{Value, json};

use super::*;

pub const ED25519_HEADER: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

pub const DIGEST: &str = "sha256:51da4efc742c8d62972ba45d0a77ce317ce92d052b10e7ddccda19e407836294";
pub const OTHER_DIGEST: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000000";

pub fn read(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/tests/fixtures/signature/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

pub fn text(name: &str) -> String {
    String::from_utf8(read(name)).unwrap()
}

pub fn key(name: &str) -> PublicKey {
    PublicKey::from_pem(&text(&format!("{name}.pub"))).unwrap()
}

pub fn verified(key: &PublicKey, claim: Claim) -> Outcome {
    Outcome::Verified {
        key: key.fingerprint().to_owned(),
        claim,
    }
}

pub fn signer() -> (Ed25519KeyPair, PublicKey) {
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
    let mut der = ED25519_HEADER.to_vec();
    der.extend_from_slice(pair.public_key().as_ref());
    let pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
        STANDARD.encode(der)
    );
    (pair, PublicKey::from_pem(&pem).unwrap())
}

pub fn is_invalid(outcome: &Outcome, reason: &str) -> bool {
    matches!(outcome, Outcome::Invalid(found) if found.contains(reason))
}

pub fn is_unsupported(outcome: &Outcome, reason: &str) -> bool {
    matches!(outcome, Outcome::Unsupported(found) if found.contains(reason))
}

pub fn bundle_json(name: &str) -> Value {
    serde_json::from_slice(&read(name)).unwrap()
}

#[test]
fn the_first_key_that_verifies_wins() {
    let payload = read("payload.json");
    let signature = text("p256.sig");
    let p256 = key("p256");
    let key_sets = [
        vec![key("p384"), key("ed25519"), p256.clone()],
        vec![p256.clone(), key("p384")],
    ];
    for keys in key_sets {
        assert_eq!(
            verify_simple_signing(&payload, &signature, &keys, DIGEST),
            verified(&p256, Claim::ImageSignature)
        );
    }
}

#[test]
fn an_empty_key_set_trusts_nothing() {
    let simple = verify_simple_signing(&read("payload.json"), &text("p256.sig"), &[], DIGEST);
    let bundle = verify_bundle(&read("p256-intoto.bundle.json"), None, &[], DIGEST);
    assert_eq!(simple, Outcome::Invalid("no trusted keys".to_owned()));
    assert_eq!(bundle, Outcome::Invalid("no trusted keys".to_owned()));
}

pub fn variants(original: &[u8]) -> Vec<Vec<u8>> {
    let truncated = (0..original.len()).map(|length| original[..length].to_vec());
    let flipped = (0..original.len()).map(|index| {
        let mut changed = original.to_vec();
        changed[index] ^= 1 << (index % 7);
        changed
    });
    truncated.chain(flipped).collect()
}

fn compact_bundle(name: &str) -> Vec<u8> {
    let mut bundle = bundle_json(name);
    bundle["verificationMaterial"] = json!({});
    if let Some(signature) = bundle
        .get_mut("messageSignature")
        .and_then(Value::as_object_mut)
    {
        signature.remove("messageDigest");
    }
    serde_json::to_vec(&bundle).unwrap()
}

#[test]
fn damaged_inputs_never_verify_and_never_panic() {
    let keys = [key("p256"), key("ed25519")];
    let payload = read("payload.json");
    let signature = text("p256.sig");
    assert!(matches!(
        verify_simple_signing(&payload, &signature, &keys, DIGEST),
        Outcome::Verified { .. }
    ));
    let mut checked = 0;
    for changed in variants(signature.as_bytes()) {
        let Ok(changed) = String::from_utf8(changed) else {
            continue;
        };
        let outcome = verify_simple_signing(&payload, &changed, &keys, DIGEST);
        assert!(!matches!(outcome, Outcome::Verified { .. }), "{changed}");
        checked += 1;
    }
    for changed in variants(&payload) {
        let outcome = verify_simple_signing(&changed, &signature, &keys, DIGEST);
        assert!(!matches!(outcome, Outcome::Verified { .. }));
        checked += 1;
    }
    for (name, message) in [
        ("ed25519-dsse-simple.bundle.json", None),
        ("p256-intoto.bundle.json", None),
        ("p256-message.bundle.json", Some(payload.as_slice())),
    ] {
        let pristine = compact_bundle(name);
        assert!(matches!(
            verify_bundle(&pristine, message, &keys, DIGEST),
            Outcome::Verified { .. }
        ));
        for changed in variants(&pristine) {
            let outcome = verify_bundle(&changed, message, &keys, DIGEST);
            assert!(
                !matches!(outcome, Outcome::Verified { .. }),
                "{}",
                String::from_utf8_lossy(&changed)
            );
            checked += 1;
        }
    }
    assert!(checked > 1000);
}

#[test]
fn claims_tell_image_signatures_from_attestations() {
    let payload = read("payload.json");
    let simple = key("p256");
    let outcome = verify_simple_signing(
        &payload,
        &text("p256.sig"),
        std::slice::from_ref(&simple),
        DIGEST,
    );
    assert_eq!(outcome, verified(&simple, Claim::ImageSignature));
    let cases = [
        (
            "ed25519-dsse-simple.bundle.json",
            "ed25519",
            None,
            Claim::ImageSignature,
        ),
        (
            "p256-message.bundle.json",
            "p256",
            Some(payload.as_slice()),
            Claim::ImageSignature,
        ),
        ("p256-intoto.bundle.json", "p256", None, Claim::Attestation),
    ];
    for (name, trusted, message, claim) in cases {
        let trusted = key(trusted);
        let outcome = verify_bundle(&read(name), message, std::slice::from_ref(&trusted), DIGEST);
        assert_eq!(outcome, verified(&trusted, claim), "{name}");
    }
}

#[test]
fn text_echoed_in_reasons_is_quoted_and_cut() {
    assert_eq!(quote("plain text"), "\"plain text\"");
    assert_eq!(quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
    assert_eq!(quote("line\nbreak\u{202e}"), "\"line?break?\"");
    let long = quote(&"x".repeat(500));
    assert_eq!(long, format!("\"{}...\"", "x".repeat(80)));
}
