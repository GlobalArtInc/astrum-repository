use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

use astrum_content::{ContentError, Ctx};
use pgp::composed::{ArmorOptions, Deserializable, DetachedSignature, SignedSecretKey};
use pgp::crypto::hash::HashAlgorithm;
use pgp::ser::Serialize;
use pgp::types::{KeyDetails, Password};

pub const SIGNATURE_TYPE: &str = "application/pgp-signature";
pub const KEY_TYPE: &str = "application/pgp-keys";
pub const SIGNATURE_LIMIT: u64 = 16 * 1024;

pub struct SigningKey {
    key: SignedSecretKey,
    password: Password,
    pub fingerprint: String,
    pub public_key: Vec<u8>,
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKey")
            .field("fingerprint", &self.fingerprint)
            .finish_non_exhaustive()
    }
}

impl SigningKey {
    pub fn read(armored: &str, passphrase: Option<&str>) -> Result<SigningKey, String> {
        let (key, _) = SignedSecretKey::from_string(armored).map_err(|error| {
            format!("the signing key is not an armored PGP private key: {error}")
        })?;
        let password = Password::from(passphrase.unwrap_or_default());
        let fingerprint = hex::encode_upper(key.primary_key.fingerprint().as_bytes());
        let public_key = key
            .to_public_key()
            .to_armored_string(ArmorOptions::default())
            .map_err(|error| format!("the public key cannot be written: {error}"))?
            .into_bytes();
        let signing = SigningKey {
            key,
            password,
            fingerprint,
            public_key,
        };
        signing
            .sign(&b"probe"[..])
            .map_err(|_| "the signing key cannot be read with the given passphrase".to_owned())?;
        Ok(signing)
    }

    pub fn sign(&self, data: impl Read) -> Result<Vec<u8>, String> {
        let signature = DetachedSignature::sign_binary_data(
            rand08::thread_rng(),
            &self.key.primary_key,
            &self.password,
            HashAlgorithm::Sha256,
            data,
        )
        .map_err(|error| error.to_string())?;
        signature.to_bytes().map_err(|error| error.to_string())
    }
}

pub fn binary_signature(data: &[u8]) -> Option<Vec<u8>> {
    let (signature, _) = DetachedSignature::from_reader_single(data).ok()?;
    signature.to_bytes().ok()
}

#[derive(Default)]
pub struct Keys {
    known: Mutex<HashMap<String, Arc<SigningKey>>>,
}

impl Keys {
    pub fn get(&self, ctx: &Ctx) -> Result<Option<Arc<SigningKey>>, ContentError> {
        let Some(signing) = ctx.repository.attributes.pacman_signing.as_ref() else {
            return Ok(None);
        };
        let Some(sealed) = signing
            .keypair
            .as_deref()
            .filter(|key| !key.trim().is_empty())
        else {
            return Ok(None);
        };
        let cache_key = format!(
            "{}\n{sealed}\n{}",
            ctx.name(),
            signing.passphrase.as_deref().unwrap_or_default()
        );
        if let Some(found) = self
            .known
            .lock()
            .expect("the signing key cache lock is never poisoned")
            .get(&cache_key)
        {
            return Ok(Some(found.clone()));
        }
        let secrets = ctx.content.secrets();
        let keypair = secrets.open(sealed).map_err(ContentError::internal)?;
        let passphrase = match signing.passphrase.as_deref() {
            Some(sealed) if !sealed.is_empty() => {
                Some(secrets.open(sealed).map_err(ContentError::internal)?)
            }
            _ => None,
        };
        let key = Arc::new(
            SigningKey::read(&keypair, passphrase.as_deref()).map_err(|error| {
                ContentError::internal(std::io::Error::other(format!(
                    "the pacman signing key of {}: {error}",
                    ctx.name()
                )))
            })?,
        );
        let mut known = self
            .known
            .lock()
            .expect("the signing key cache lock is never poisoned");
        known.retain(|other, _| !other.starts_with(&format!("{}\n", ctx.name())));
        known.insert(cache_key, key.clone());
        Ok(Some(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn keys_make_binary_detached_signatures() {
        let armored = testing::signing_key(Some("secret"));
        assert!(SigningKey::read(&armored, Some("wrong")).is_err());
        let key = SigningKey::read(&armored, Some("secret")).unwrap();
        assert_eq!(key.fingerprint.len(), 40);
        let signature = key.sign(&b"database"[..]).unwrap();
        assert!(!signature.starts_with(b"-----BEGIN"));
        let public = String::from_utf8(key.public_key.clone()).unwrap();
        assert!(public.starts_with("-----BEGIN PGP PUBLIC KEY BLOCK-----"));
        assert!(testing::verify(&signature, b"database", &public));
        assert!(!testing::verify(&signature, b"other", &public));
        assert!(SigningKey::read("garbage", None).is_err());
    }

    #[test]
    fn uploaded_signatures_become_binary() {
        let key = SigningKey::read(&testing::signing_key(None), None).unwrap();
        let binary = key.sign(&b"package"[..]).unwrap();
        assert_eq!(binary_signature(&binary), Some(binary.clone()));
        let (parsed, _) = DetachedSignature::from_reader_single(binary.as_slice()).unwrap();
        let armored = parsed.to_armored_bytes(ArmorOptions::default()).unwrap();
        assert_eq!(binary_signature(&armored), Some(binary));
        assert_eq!(binary_signature(b"not a signature"), None);
    }
}
