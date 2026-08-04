use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::{ContentError, Ctx};
use pgp::composed::{
    ArmorOptions, CleartextSignedMessage, Deserializable, DetachedSignature, SignedSecretKey,
};
use pgp::crypto::hash::HashAlgorithm;
use pgp::types::{KeyDetails, Password};

pub const UNSIGNED: &str = "unsigned";

pub struct SigningKey {
    key: SignedSecretKey,
    password: Password,
    pub fingerprint: String,
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
        let fingerprint = hex::encode(key.primary_key.fingerprint().as_bytes());
        let signing = SigningKey {
            key,
            password,
            fingerprint,
        };
        signing
            .sign(b"probe")
            .map_err(|_| "the signing key cannot be read with the given passphrase".to_owned())?;
        Ok(signing)
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let signature = DetachedSignature::sign_binary_data(
            rand08::thread_rng(),
            &self.key.primary_key,
            &self.password,
            HashAlgorithm::Sha256,
            data,
        )
        .map_err(|error| error.to_string())?;
        signature
            .to_armored_bytes(ArmorOptions::default())
            .map_err(|error| error.to_string())
    }
}

impl SigningKey {
    pub fn clearsign(&self, text: &str) -> Result<String, String> {
        let message = CleartextSignedMessage::sign(
            rand08::thread_rng(),
            text,
            &self.key.primary_key,
            &self.password,
        )
        .map_err(|error| error.to_string())?;
        message
            .to_armored_string(ArmorOptions::default())
            .map_err(|error| error.to_string())
    }
}

#[derive(Default)]
pub struct Keys {
    known: Mutex<HashMap<String, Arc<SigningKey>>>,
}

impl Keys {
    pub fn get(&self, ctx: &Ctx) -> Result<Option<Arc<SigningKey>>, ContentError> {
        let Some(signing) = ctx.repository.attributes.apt_signing.as_ref() else {
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
            SigningKey::read(&keypair, passphrase.as_deref())
                .map_err(|error| ContentError::internal(std::io::Error::other(error)))?,
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

pub fn signer(key: Option<&SigningKey>) -> &str {
    key.map(|key| key.fingerprint.as_str()).unwrap_or(UNSIGNED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_clearsign_release_files() {
        let key = SigningKey::read(&crate::testing::signing_key(), None).unwrap();
        let signed = key.clearsign("Suite: bookworm\n").unwrap();
        assert!(
            signed.starts_with(
                "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA256\n\nSuite: bookworm\n"
            )
        );
        let (message, _) = CleartextSignedMessage::from_string(&signed).unwrap();
        let public = key.key.to_public_key();
        message.verify(&public).unwrap();
        let detached = String::from_utf8(key.sign(b"Suite: bookworm\n").unwrap()).unwrap();
        assert!(detached.starts_with("-----BEGIN PGP SIGNATURE-----"));
    }
}
