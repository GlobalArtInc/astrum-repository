use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::{ContentError, Ctx};
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey as Pkcs1v15Key;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::archive;
use crate::pem::{self, KeyError};

pub const UNSIGNED: &str = "unsigned";
pub const KEY_SUFFIX: &str = ".rsa.pub";
pub const KEY_TYPE: &str = "application/x-pem-file";
const SIGNATURE_KIND: &str = ".SIGN.RSA.";

pub struct SigningKey {
    key: Pkcs1v15Key<Sha1>,
    pub name: String,
    pub public_key: String,
    pub fingerprint: String,
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKey")
            .field("name", &self.name)
            .field("fingerprint", &self.fingerprint)
            .finish_non_exhaustive()
    }
}

pub fn key_name(repository: &str) -> String {
    format!("{repository}{KEY_SUFFIX}")
}

impl SigningKey {
    pub fn read(
        pem_text: &str,
        passphrase: Option<&str>,
        name: String,
    ) -> Result<SigningKey, KeyError> {
        let private = pem::private_key(pem_text, passphrase)?;
        Ok(SigningKey::new(private, name))
    }

    pub fn new(private: RsaPrivateKey, name: String) -> SigningKey {
        let public_key = pem::public_key(&private);
        let fingerprint = hex::encode(Sha256::digest(public_key.as_bytes()));
        SigningKey {
            key: Pkcs1v15Key::<Sha1>::new(private),
            name,
            public_key,
            fingerprint,
        }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.key
            .sign_with_rng(&mut rand_core06::OsRng, data)
            .to_vec()
    }

    pub fn signature_file(&self) -> String {
        format!("{SIGNATURE_KIND}{}", self.name)
    }

    pub fn signature_member(&self, data: &[u8], mtime: u64) -> Vec<u8> {
        archive::signature_member(&self.signature_file(), &self.sign(data), mtime)
    }

    pub fn signer(&self) -> String {
        format!("{} {}", self.name, self.fingerprint)
    }
}

pub fn signer(key: Option<&SigningKey>) -> String {
    key.map(SigningKey::signer)
        .unwrap_or_else(|| UNSIGNED.to_owned())
}

#[derive(Default)]
pub struct Keys {
    known: Mutex<HashMap<String, Arc<SigningKey>>>,
}

impl Keys {
    pub fn get(&self, ctx: &Ctx) -> Result<Option<Arc<SigningKey>>, ContentError> {
        let Some(signing) = ctx.repository.attributes.alpine_signing.as_ref() else {
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
            SigningKey::read(&keypair, passphrase.as_deref(), key_name(ctx.name())).map_err(
                |error| {
                    ContentError::internal(std::io::Error::other(format!(
                        "the Alpine signing key of {}: {error}",
                        ctx.name()
                    )))
                },
            )?,
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
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use rsa::pkcs8::DecodePublicKey;
    use rsa::signature::Verifier;

    use super::*;

    #[test]
    fn signatures_are_sha1_rsa_over_the_data() {
        let key = SigningKey::new(crate::testing::rsa_key(), key_name("apks"));
        assert_eq!(key.name, "apks.rsa.pub");
        assert_eq!(key.signature_file(), ".SIGN.RSA.apks.rsa.pub");
        let signature = key.sign(b"index");
        let public = rsa::RsaPublicKey::from_public_key_pem(&key.public_key).unwrap();
        let verifier = VerifyingKey::<Sha1>::new(public);
        let parsed = Signature::try_from(signature.as_slice()).unwrap();
        verifier.verify(b"index", &parsed).unwrap();
        assert!(verifier.verify(b"other", &parsed).is_err());
        let member = key.signature_member(b"index", 0);
        let split = archive::split(&member, 1 << 20).unwrap();
        assert_eq!(split.names, [".SIGN.RSA.apks.rsa.pub"]);
        assert!(split.rest.is_empty());
        assert_ne!(signer(Some(&key)), signer(None));
    }
}
