use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ring::signature::{self, RsaPublicKeyComponents, UnparsedPublicKey};
use serde_json::{Map, Value};

pub const SUPPORTED_ALGORITHMS: [&str; 8] = [
    "ES256", "ES384", "PS256", "PS384", "PS512", "RS256", "RS384", "RS512",
];

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct JwtError(pub String);

fn fail(message: impl Into<String>) -> JwtError {
    JwtError(message.into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyMaterial {
    Rsa { n: Vec<u8>, e: Vec<u8> },
    Ec { curve: Curve, point: Vec<u8> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    P256,
    P384,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub id: Option<String>,
    pub material: KeyMaterial,
}

pub fn parse_jwks(value: &Value) -> Result<Vec<Key>, JwtError> {
    let keys = value
        .get("keys")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("The JSON Web Key Set has no keys"))?;
    let mut parsed = Vec::new();
    for jwk in keys {
        if jwk
            .get("use")
            .and_then(Value::as_str)
            .is_some_and(|usage| usage != "sig")
        {
            continue;
        }
        match parse_key(jwk) {
            Ok(Some(material)) => parsed.push(Key {
                id: jwk.get("kid").and_then(Value::as_str).map(str::to_owned),
                material,
            }),
            Ok(None) => {}
            Err(error) => tracing::warn!(%error, "an unusable JSON Web Key is skipped"),
        }
    }
    if parsed.is_empty() {
        return Err(fail("The JSON Web Key Set has no usable signing keys"));
    }
    Ok(parsed)
}

fn field(jwk: &Value, name: &str) -> Result<Vec<u8>, JwtError> {
    let text = jwk
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| fail(format!("the key has no {name}")))?;
    URL_SAFE_NO_PAD
        .decode(text.trim_end_matches('='))
        .map_err(|_| fail(format!("the {name} of the key is not base64url")))
}

fn parse_key(jwk: &Value) -> Result<Option<KeyMaterial>, JwtError> {
    match jwk.get("kty").and_then(Value::as_str) {
        Some("RSA") => Ok(Some(KeyMaterial::Rsa {
            n: field(jwk, "n")?,
            e: field(jwk, "e")?,
        })),
        Some("EC") => {
            let (curve, size) = match jwk.get("crv").and_then(Value::as_str) {
                Some("P-256") => (Curve::P256, 32),
                Some("P-384") => (Curve::P384, 48),
                other => return Err(fail(format!("unsupported curve {other:?}"))),
            };
            let x = field(jwk, "x")?;
            let y = field(jwk, "y")?;
            if x.len() != size || y.len() != size {
                return Err(fail("the curve point has the wrong size"));
            }
            let mut point = Vec::with_capacity(1 + 2 * size);
            point.push(4);
            point.extend_from_slice(&x);
            point.extend_from_slice(&y);
            Ok(Some(KeyMaterial::Ec { curve, point }))
        }
        _ => Ok(None),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decoded {
    pub algorithm: String,
    pub key_id: Option<String>,
    pub claims: Map<String, Value>,
    signing_input: String,
    signature: Vec<u8>,
}

pub fn decode(token: &str) -> Result<Decoded, JwtError> {
    let mut parts = token.trim().split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(fail("The ID token is malformed"));
    };
    let header: Value = URL_SAFE_NO_PAD
        .decode(header)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| fail("The ID token header is malformed"))?;
    let claims: Map<String, Value> = URL_SAFE_NO_PAD
        .decode(payload)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| fail("The ID token payload is not valid JSON"))?;
    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| fail("The ID token signature is malformed"))?;
    let algorithm = header
        .get("alg")
        .and_then(Value::as_str)
        .ok_or_else(|| fail("The ID token names no signature algorithm"))?
        .to_owned();
    Ok(Decoded {
        algorithm,
        key_id: header.get("kid").and_then(Value::as_str).map(str::to_owned),
        claims,
        signing_input: token[..token.rfind('.').unwrap_or_default()].to_owned(),
        signature,
    })
}

impl Decoded {
    pub fn verify(&self, keys: &[Key]) -> Result<(), JwtError> {
        let key = select(keys, self.key_id.as_deref())?;
        let input = self.signing_input.as_bytes();
        let verified = match (&key.material, self.algorithm.as_str()) {
            (KeyMaterial::Rsa { n, e }, algorithm) => {
                let parameters: &signature::RsaParameters = match algorithm {
                    "RS256" => &signature::RSA_PKCS1_2048_8192_SHA256,
                    "RS384" => &signature::RSA_PKCS1_2048_8192_SHA384,
                    "RS512" => &signature::RSA_PKCS1_2048_8192_SHA512,
                    "PS256" => &signature::RSA_PSS_2048_8192_SHA256,
                    "PS384" => &signature::RSA_PSS_2048_8192_SHA384,
                    "PS512" => &signature::RSA_PSS_2048_8192_SHA512,
                    other => return Err(mismatch(other)),
                };
                RsaPublicKeyComponents { n, e }
                    .verify(parameters, input, &self.signature)
                    .is_ok()
            }
            (KeyMaterial::Ec { curve, point }, algorithm) => {
                let parameters: &signature::EcdsaVerificationAlgorithm = match (curve, algorithm) {
                    (Curve::P256, "ES256") => &signature::ECDSA_P256_SHA256_FIXED,
                    (Curve::P384, "ES384") => &signature::ECDSA_P384_SHA384_FIXED,
                    (_, other) => return Err(mismatch(other)),
                };
                UnparsedPublicKey::new(parameters, point)
                    .verify(input, &self.signature)
                    .is_ok()
            }
        };
        match verified {
            true => Ok(()),
            false => Err(fail("The ID token signature is not valid")),
        }
    }
}

fn mismatch(algorithm: &str) -> JwtError {
    match SUPPORTED_ALGORITHMS.contains(&algorithm) {
        true => fail(format!(
            "The signing key does not match the {algorithm} algorithm"
        )),
        false => fail(format!(
            "Unsupported ID token signature algorithm {algorithm}"
        )),
    }
}

fn select<'a>(keys: &'a [Key], id: Option<&str>) -> Result<&'a Key, JwtError> {
    match id {
        Some(id) => keys
            .iter()
            .find(|key| key.id.as_deref() == Some(id))
            .ok_or_else(|| fail(format!("The provider has no signing key with id {id}"))),
        None => match keys {
            [only] => Ok(only),
            _ => Err(fail(format!(
                "The ID token has no key id and the provider publishes {} keys",
                keys.len()
            ))),
        },
    }
}

pub fn has_key(keys: &[Key], id: Option<&str>) -> bool {
    id.is_none_or(|id| keys.iter().any(|key| key.id.as_deref() == Some(id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
    use serde_json::json;

    const RSA_JWK: &str = r#"{"kty":"RSA","kid":"rsa-1","use":"sig","alg":"RS256","n":"qBPs7eMXsMk3inrN9O2pbtCfOVXOy5gQSg6qTPtnwAgrCR4Oabw1rNlRQa91O3fTUFS3I5xws7X52Z46nU7pFHUzskJU0M3IK7OOy3zXJUY7oKejQC0sGa8NuOfcDkoh3yCORSpVqeltDl-oSQHFvvUyeGKjaRxNLj2U9D1zO0cc0_zRlGJ28d5vuceF3N5x08koI1qADJ4SEJBAKkpdEzFf6Ten1Am0-s67pKfybRl46KXhdEv2h9nhys2qQLQg1AXlqvoBsFS0YGH6EaTrlIDFLa6Aty86IuJ9lbEwLhrjNOVI9vLN3k84DRA3ssQXPr9oZJ9qItN3uM6xlufFhQ","e":"AQAB"}"#;
    const RSA_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6InJzYS0xIiwidHlwIjoiSldUIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwiYXVkIjoiYXN0cnVtIiwic3ViIjoidS0xIiwicHJlZmVycmVkX3VzZXJuYW1lIjoicml0YSIsIm5vbmNlIjoibi0xMjMiLCJpYXQiOjE3OTE4MDAwMDAsImV4cCI6NDEwMjQ0NDgwMCwiZ3JvdXBzIjpbImRldnMiXX0.OBWtula0aEkqleHFS4OTOpje7Ua4nXCA6nW17vzziwYl9ko58myv3PEnfJ0srV82jIBKHeBEcdIeK82KQg0sY_SMCEdbiEvyxHX9QvlTYKwhbUVkpPWHCd1XTwfz0Di3VapqlaoeWfi7FjWnSStIXPhNcdFV00Crv0ANGM6Vi1pOuJtWwsIH1W93d7tPe4kj149rcr8pV6PD90bPmVj_tJWvhqW4lQwIxUEisWnwtDWEP3WEJI7OzKRc24FWS2aMbm0hcTlgY9_aa_aGKyW2m7tXdqbtMlgCbxOmAptHDqaPPcNL4MWE0V0lB96KVDfgm-54G4tLD7l17KagjyupuA";

    fn jwks(jwk: &str) -> Vec<Key> {
        parse_jwks(&json!({ "keys": [serde_json::from_str::<Value>(jwk).unwrap()] })).unwrap()
    }

    #[test]
    fn an_rs256_token_verifies_against_its_key() {
        let keys = jwks(RSA_JWK);
        let decoded = decode(RSA_TOKEN).unwrap();
        assert_eq!(decoded.algorithm, "RS256");
        assert_eq!(decoded.key_id.as_deref(), Some("rsa-1"));
        assert_eq!(decoded.claims["preferred_username"], "rita");
        decoded.verify(&keys).unwrap();

        let mut tampered = RSA_TOKEN.to_owned();
        tampered.replace_range(40..41, if &RSA_TOKEN[40..41] == "a" { "b" } else { "a" });
        assert!(
            decode(&tampered)
                .and_then(|token| token.verify(&keys))
                .is_err()
        );
    }

    #[test]
    fn an_es256_token_verifies_and_a_wrong_key_does_not() {
        let random = SystemRandom::new();
        let pkcs8 =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &random).unwrap();
        let pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref(), &random)
                .unwrap();
        let public = pair.public_key().as_ref();
        let jwk = json!({
            "kty": "EC", "crv": "P-256", "kid": "ec",
            "x": URL_SAFE_NO_PAD.encode(&public[1..33]),
            "y": URL_SAFE_NO_PAD.encode(&public[33..65]),
        });
        let keys = parse_jwks(&json!({ "keys": [jwk, {"kty": "oct", "k": "x"}] })).unwrap();
        assert_eq!(keys.len(), 1);
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"ES256","kid":"ec"}"#);
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"x"}"#);
        let input = format!("{header}.{payload}");
        let signature = pair.sign(&random, input.as_bytes()).unwrap();
        let token = format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature.as_ref()));
        decode(&token).unwrap().verify(&keys).unwrap();
        let error = decode(&token).unwrap().verify(&jwks(RSA_JWK)).unwrap_err();
        assert_eq!(error.0, "The provider has no signing key with id ec");
        let renamed = Key {
            id: Some("ec".to_owned()),
            material: jwks(RSA_JWK).remove(0).material,
        };
        let error = decode(&token).unwrap().verify(&[renamed]).unwrap_err();
        assert_eq!(
            error.0,
            "The signing key does not match the ES256 algorithm"
        );
    }

    #[test]
    fn malformed_tokens_and_key_sets_are_refused() {
        assert!(decode("a.b").is_err());
        assert!(decode("a.b.c.d").is_err());
        assert!(decode("e30.e30.AA").is_err());
        assert!(parse_jwks(&json!({})).is_err());
        assert!(
            parse_jwks(&json!({"keys": [{"kty": "RSA", "use": "enc", "n": "AQ", "e": "AQ"}]}))
                .is_err()
        );
        let token = format!(
            "{}.{}.AA",
            URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256"}"#),
            URL_SAFE_NO_PAD.encode(br#"{}"#)
        );
        let error = decode(&token).unwrap().verify(&jwks(RSA_JWK)).unwrap_err();
        assert_eq!(error.0, "Unsupported ID token signature algorithm HS256");
        assert!(has_key(&jwks(RSA_JWK), Some("rsa-1")));
        assert!(!has_key(&jwks(RSA_JWK), Some("other")));
        assert!(has_key(&jwks(RSA_JWK), None));
    }
}
