use std::io::Cursor;

use pgp::composed::{
    ArmorOptions, Deserializable, DetachedSignature, SignedPublicKey, SignedSecretKey,
};
use pgp::crypto::hash::HashAlgorithm;
use pgp::ser::Serialize;
use pgp::types::{KeyDetails, Password};

#[derive(Clone, Debug)]
pub struct PublicKey {
    pub key_id: String,
    pub armored: String,
}

pub struct SigningKey {
    secret: SignedSecretKey,
    password: Password,
    pub public: PublicKey,
}

fn key_id(id: impl AsRef<[u8]>) -> String {
    hex::encode_upper(id.as_ref())
}

fn armor(key: &SignedPublicKey) -> Result<String, String> {
    key.to_armored_string(ArmorOptions::default())
        .map_err(|error| format!("the key cannot be written: {error}"))
}

fn read_public(data: &[u8]) -> Result<SignedPublicKey, String> {
    let text = String::from_utf8_lossy(data);
    if text.trim_start().starts_with("-----BEGIN PGP") {
        let (key, _) = SignedPublicKey::from_string(text.trim())
            .map_err(|error| format!("not an OpenPGP public key: {error}"))?;
        return Ok(key);
    }
    SignedPublicKey::from_bytes(Cursor::new(data))
        .map_err(|error| format!("not an OpenPGP public key: {error}"))
}

pub fn read_public_key(data: &[u8]) -> Result<PublicKey, String> {
    let key = read_public(data)?;
    Ok(PublicKey {
        key_id: key_id(key.primary_key.legacy_key_id()),
        armored: armor(&key)?,
    })
}

pub fn read_signing_key(text: &str, passphrase: Option<&str>) -> Result<SigningKey, String> {
    let (secret, _) = SignedSecretKey::from_string(text.trim())
        .map_err(|error| format!("not an OpenPGP secret key: {error}"))?;
    let password = Password::from(passphrase.unwrap_or_default());
    let signing = SigningKey {
        public: PublicKey {
            key_id: key_id(secret.primary_key.legacy_key_id()),
            armored: armor(&secret.to_public_key())?,
        },
        secret,
        password,
    };
    signing
        .sign(b"probe")
        .map_err(|error| format!("the signing key cannot sign: {error}"))?;
    Ok(signing)
}

impl SigningKey {
    pub fn sign(&self, document: &[u8]) -> Result<Vec<u8>, String> {
        let rng = rand_core06::OsRng;
        let subkey = self.secret.secret_subkeys.iter().find(|subkey| {
            subkey
                .signatures
                .iter()
                .any(|signature| signature.key_flags().sign())
        });
        let signature = match subkey {
            Some(subkey) => DetachedSignature::sign_binary_data(
                rng,
                &subkey.key,
                &self.password,
                HashAlgorithm::Sha256,
                document,
            ),
            None => DetachedSignature::sign_binary_data(
                rng,
                &self.secret.primary_key,
                &self.password,
                HashAlgorithm::Sha256,
                document,
            ),
        }
        .map_err(|error| error.to_string())?;
        signature.to_bytes().map_err(|error| error.to_string())
    }
}

fn read_signature(data: &[u8]) -> Result<DetachedSignature, String> {
    let text = String::from_utf8_lossy(data);
    if text.trim_start().starts_with("-----BEGIN PGP SIGNATURE") {
        return DetachedSignature::from_string(text.trim())
            .map(|(signature, _)| signature)
            .map_err(|error| format!("not an OpenPGP signature: {error}"));
    }
    DetachedSignature::from_bytes(Cursor::new(data))
        .map_err(|error| format!("not an OpenPGP signature: {error}"))
}

pub fn binary_signature(data: &[u8]) -> Result<Vec<u8>, String> {
    read_signature(data)?
        .to_bytes()
        .map_err(|error| error.to_string())
}

pub fn verify(document: &[u8], signature: &[u8], armored: &str) -> bool {
    let (Ok(signature), Ok(key)) = (read_signature(signature), read_public(armored.as_bytes()))
    else {
        return false;
    };
    signature.verify(&key.primary_key, document).is_ok()
        || key
            .public_subkeys
            .iter()
            .any(|subkey| signature.verify(&subkey.key, document).is_ok())
}

#[cfg(test)]
pub mod testing {
    pub const SECRET: &str = include_str!("../tests/data/secret.asc");
    pub const OTHER_PUBLIC: &str = include_str!("../tests/data/other.asc");
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;

    #[test]
    fn signatures_verify_with_the_signing_key_only() {
        let key = read_signing_key(SECRET, Some("probe-passphrase")).unwrap();
        assert_eq!(key.public.key_id.len(), 16);
        let signature = key.sign(b"sums\n").unwrap();
        assert!(verify(b"sums\n", &signature, &key.public.armored));
        assert!(!verify(b"other\n", &signature, &key.public.armored));
        assert!(!verify(b"sums\n", &signature, OTHER_PUBLIC));
        assert_eq!(binary_signature(&signature).unwrap(), signature);
        let parsed = read_public_key(key.public.armored.as_bytes()).unwrap();
        assert_eq!(parsed.key_id, key.public.key_id);
        assert!(read_signing_key(SECRET, Some("wrong")).is_err());
        assert!(read_public_key(b"garbage").is_err());
    }
}
