use serde::Deserialize;

use super::json::{JsonError, Object, parse_object};
use super::{
    Claim, MAX_PAYLOAD_BYTES, MAX_SIGNATURE_BYTES, Outcome, PublicKey, decode_base64,
    first_verifying, manifest_hex, quote,
};

const COSIGN_TYPE: &str = "cosign container image signature";

#[derive(Deserialize)]
struct Payload {
    critical: Object<Critical>,
}

#[derive(Deserialize)]
struct Critical {
    image: Object<Image>,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct Image {
    #[serde(rename = "docker-manifest-digest")]
    digest: String,
}

pub fn verify_simple_signing(
    payload: &[u8],
    signature_base64: &str,
    keys: &[PublicKey],
    manifest_digest: &str,
) -> Outcome {
    if let Err(outcome) = manifest_hex(manifest_digest) {
        return outcome;
    }
    if keys.is_empty() {
        return Outcome::Invalid("no trusted keys".to_owned());
    }
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Outcome::Invalid(format!("payload is larger than {MAX_PAYLOAD_BYTES} bytes"));
    }
    let signature = match decode_base64(signature_base64, MAX_SIGNATURE_BYTES, "signature") {
        Ok(signature) => signature,
        Err(reason) => return Outcome::Invalid(reason),
    };
    let Some(key) = first_verifying(keys, payload, std::slice::from_ref(&signature)) else {
        return Outcome::Invalid("no trusted key verifies the signature".to_owned());
    };
    match check_payload(payload, manifest_digest) {
        Ok(()) => Outcome::Verified {
            key: key.fingerprint().to_owned(),
            claim: Claim::ImageSignature,
        },
        Err(reason) => Outcome::Invalid(reason),
    }
}

pub(super) fn check_payload(payload: &[u8], manifest_digest: &str) -> Result<(), String> {
    let parsed: Payload = parse_object(payload).map_err(|error| {
        match error {
            JsonError::NotObject => "the payload is not a JSON object",
            JsonError::Syntax => "payload is not valid JSON",
            JsonError::Structure => "payload does not have the cosign simple signing structure",
        }
        .to_owned()
    })?;
    let critical = parsed.critical.0;
    let image = critical.image.0;
    if critical.kind != COSIGN_TYPE {
        return Err(format!(
            "payload type is {}, not a cosign container image signature",
            quote(&critical.kind)
        ));
    }
    if image.digest != manifest_digest {
        return Err(format!(
            "payload names {} instead of {manifest_digest}",
            quote(&image.digest)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::tests::{
        DIGEST, OTHER_DIGEST, is_invalid, key, read, signer, text, verified,
    };
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    fn payload(kind: &str, digest: &str) -> Vec<u8> {
        format!(
            r#"{{"critical":{{"identity":{{}},"image":{{"docker-manifest-digest":"{digest}"}},"type":"{kind}"}},"optional":null}}"#
        )
        .into_bytes()
    }

    #[test]
    fn valid_signatures_verify_for_every_algorithm() {
        let cases = [
            ("p256", "p256.sig"),
            ("p384", "p384.sig"),
            ("ed25519", "ed25519.sig"),
            ("rsa2048", "rsa2048-pkcs1.sig"),
            ("rsa2048", "rsa2048-pss.sig"),
        ];
        let payload = read("payload.json");
        for (key_name, signature) in cases {
            let trusted = key(key_name);
            let outcome = verify_simple_signing(
                &payload,
                &text(signature),
                std::slice::from_ref(&trusted),
                DIGEST,
            );
            assert_eq!(
                outcome,
                verified(&trusted, Claim::ImageSignature),
                "{signature}"
            );
        }
    }

    #[test]
    fn signatures_that_must_not_be_accepted() {
        let mut tampered = read("payload.json");
        tampered[10] ^= 1;
        let upper = DIGEST.to_uppercase().replace("SHA256", "sha256");
        let cases = [
            (
                "tampered",
                tampered,
                text("p256.sig"),
                "p256",
                DIGEST,
                "no trusted key",
            ),
            (
                "wrong key",
                read("payload.json"),
                text("p256.sig"),
                "p384",
                DIGEST,
                "no trusted key",
            ),
            (
                "other digest",
                read("payload.json"),
                text("p256.sig"),
                "p256",
                OTHER_DIGEST,
                "payload names",
            ),
            (
                "upper case argument",
                read("payload.json"),
                text("p256.sig"),
                "p256",
                upper.as_str(),
                "manifest digest",
            ),
            (
                "upper case payload",
                read("payload-upper.json"),
                text("p256-upper.sig"),
                "p256",
                DIGEST,
                "payload names",
            ),
            (
                "sha512 argument",
                read("payload.json"),
                text("p256.sig"),
                "p256",
                "sha512:ab",
                "manifest digest",
            ),
            (
                "bad base64",
                read("payload.json"),
                "***".to_owned(),
                "p256",
                DIGEST,
                "not valid base64",
            ),
            (
                "empty signature",
                read("payload.json"),
                String::new(),
                "p256",
                DIGEST,
                "signature is empty",
            ),
            (
                "oversized signature",
                read("payload.json"),
                "A".repeat(30_000),
                "p256",
                DIGEST,
                "signature is larger",
            ),
            (
                "oversized payload",
                vec![b'x'; MAX_PAYLOAD_BYTES + 1],
                text("p256.sig"),
                "p256",
                DIGEST,
                "payload is larger",
            ),
        ];
        for (name, payload, signature, trusted, digest, reason) in cases {
            let outcome = verify_simple_signing(&payload, &signature, &[key(trusted)], digest);
            assert!(is_invalid(&outcome, reason), "{name}: {outcome:?}");
        }
    }

    #[test]
    fn the_payload_structure_is_checked() {
        let duplicate = DIGEST.to_owned();
        let doubled = format!(
            r#"{{"critical":{{"image":{{"docker-manifest-digest":"{duplicate}"}},"type":"{COSIGN_TYPE}","type":"x"}}}}"#
        );
        let positional = format!(r#"[[["{DIGEST}"],"{COSIGN_TYPE}"]]"#);
        let nested = format!(r#"{{"critical":[["{DIGEST}"],"{COSIGN_TYPE}"]}}"#);
        let image_array =
            format!(r#"{{"critical":{{"image":["{DIGEST}"],"type":"{COSIGN_TYPE}"}}}}"#);
        let cases = [
            (b"{".to_vec(), "not valid JSON"),
            (b"not json".to_vec(), "not a JSON object"),
            (Vec::new(), "not a JSON object"),
            (b" \n[]".to_vec(), "not a JSON object"),
            (positional.clone().into_bytes(), "not a JSON object"),
            (nested.into_bytes(), "simple signing structure"),
            (image_array.into_bytes(), "simple signing structure"),
            (b"{}".to_vec(), "simple signing structure"),
            (
                br#"{"critical":{"image":{}}}"#.to_vec(),
                "simple signing structure",
            ),
            (doubled.into_bytes(), "simple signing structure"),
            (payload("other type", DIGEST), "payload type"),
            (payload(COSIGN_TYPE, OTHER_DIGEST), "payload names"),
            (payload(COSIGN_TYPE, &"x".repeat(500)), "payload names"),
        ];
        for (bytes, reason) in cases {
            let error = check_payload(&bytes, DIGEST).unwrap_err();
            assert!(error.contains(reason), "{error}");
            assert!(error.len() < 400, "{error}");
        }
        assert_eq!(check_payload(&payload(COSIGN_TYPE, DIGEST), DIGEST), Ok(()));
    }

    #[test]
    fn a_positional_payload_signed_by_a_trusted_key_is_not_accepted() {
        let (pair, trusted) = signer();
        let payload = format!(r#"[[["{DIGEST}"],"{COSIGN_TYPE}"]]"#).into_bytes();
        let signature = STANDARD.encode(pair.sign(&payload).as_ref());
        let outcome = verify_simple_signing(&payload, &signature, &[trusted], DIGEST);
        assert!(is_invalid(&outcome, "not a JSON object"), "{outcome:?}");
    }
}
