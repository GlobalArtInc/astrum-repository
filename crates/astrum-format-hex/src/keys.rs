use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::{ContentError, Ctx};
use astrum_repo::Section;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::pkcs8::{
    DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding,
};
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256, Sha512};

use crate::proto;

pub const KEY_BITS: usize = 2048;
pub const KEY_TYPE: &str = "application/x-pem-file";

pub struct RegistryKey {
    signer: SigningKey<Sha512>,
    pub public_key: String,
    pub fingerprint: String,
}

impl std::fmt::Debug for RegistryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistryKey")
            .field("fingerprint", &self.fingerprint)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("the key is not a PEM-encoded RSA private key")]
    NotRsa,
    #[error("the key cannot be written: {0}")]
    Encode(String),
}

fn mpint(bytes: &[u8]) -> Vec<u8> {
    let trimmed: &[u8] = match bytes.iter().position(|byte| *byte != 0) {
        Some(start) => &bytes[start..],
        None => &[],
    };
    let mut value = Vec::with_capacity(trimmed.len() + 1);
    if trimmed.first().is_some_and(|first| first & 0x80 != 0) {
        value.push(0);
    }
    value.extend_from_slice(trimmed);
    value
}

fn ssh_string(out: &mut Vec<u8>, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(data);
}

pub fn fingerprint(public: &RsaPublicKey) -> String {
    let mut blob = Vec::new();
    ssh_string(&mut blob, b"ssh-rsa");
    ssh_string(&mut blob, &mpint(&public.e().to_bytes_be()));
    ssh_string(&mut blob, &mpint(&public.n().to_bytes_be()));
    format!("SHA256:{}", STANDARD_NO_PAD.encode(Sha256::digest(&blob)))
}

pub fn public_key_of(pem: &str) -> Option<RsaPublicKey> {
    RsaPublicKey::from_public_key_pem(pem.trim())
        .ok()
        .or_else(|| rsa::pkcs1::DecodeRsaPublicKey::from_pkcs1_pem(pem.trim()).ok())
}

pub fn verify(public: &RsaPublicKey, resource: &[u8]) -> Option<Vec<u8>> {
    let signed = proto::open(resource).ok()?;
    let signature = Signature::try_from(signed.signature.as_deref()?).ok()?;
    VerifyingKey::<Sha512>::new(public.clone())
        .verify(&signed.payload, &signature)
        .ok()?;
    Some(signed.payload)
}

impl RegistryKey {
    pub fn read(pem: &str) -> Result<RegistryKey, KeyError> {
        let private = RsaPrivateKey::from_pkcs8_pem(pem.trim())
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem.trim()))
            .map_err(|_| KeyError::NotRsa)?;
        RegistryKey::new(private)
    }

    pub fn new(private: RsaPrivateKey) -> Result<RegistryKey, KeyError> {
        let public = private.to_public_key();
        let public_key = public
            .to_public_key_pem(LineEnding::LF)
            .map_err(|error| KeyError::Encode(error.to_string()))?;
        Ok(RegistryKey {
            fingerprint: fingerprint(&public),
            signer: SigningKey::<Sha512>::new(private),
            public_key,
        })
    }

    pub fn generate() -> Result<String, KeyError> {
        let private = RsaPrivateKey::new(&mut rand_core06::OsRng, KEY_BITS)
            .map_err(|error| KeyError::Encode(error.to_string()))?;
        Ok(private
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|error| KeyError::Encode(error.to_string()))?
            .to_string())
    }

    pub fn sign(&self, payload: Vec<u8>) -> Vec<u8> {
        let signature = self.signer.sign(&payload).to_vec();
        proto::wrap(payload, signature)
    }
}

#[derive(Default)]
pub struct Keys {
    known: Mutex<HashMap<String, (String, Arc<RegistryKey>)>>,
    making: tokio::sync::Mutex<()>,
}

fn stored(ctx: &Ctx) -> Option<String> {
    ctx.repository
        .attributes
        .hex_signing
        .as_ref()
        .and_then(|signing| signing.keypair.clone())
        .filter(|sealed| !sealed.is_empty())
}

fn failed(ctx: &Ctx, error: impl std::fmt::Display) -> ContentError {
    ContentError::internal(std::io::Error::other(format!(
        "the registry key of {}: {error}",
        ctx.name()
    )))
}

impl Keys {
    pub async fn get(&self, ctx: &Ctx) -> Result<Arc<RegistryKey>, ContentError> {
        let sealed = match stored(ctx) {
            Some(sealed) => sealed,
            None => self.make(ctx).await?,
        };
        if let Some((known, key)) = self
            .known
            .lock()
            .expect("the key cache lock is never poisoned")
            .get(ctx.name())
            && *known == sealed
        {
            return Ok(key.clone());
        }
        let pem = ctx
            .content
            .secrets()
            .open(&sealed)
            .map_err(|error| failed(ctx, error))?;
        let key = tokio::task::spawn_blocking(move || RegistryKey::read(&pem))
            .await
            .map_err(ContentError::internal)?
            .map_err(|error| failed(ctx, error))?;
        let key = Arc::new(key);
        self.known
            .lock()
            .expect("the key cache lock is never poisoned")
            .insert(ctx.name().to_owned(), (sealed, key.clone()));
        Ok(key)
    }

    async fn make(&self, ctx: &Ctx) -> Result<String, ContentError> {
        let _making = self.making.lock().await;
        let current = astrum_repo::store::find(ctx.db(), ctx.name())
            .await
            .map_err(ContentError::internal)?;
        if let Some(sealed) = current
            .as_ref()
            .and_then(|repository| repository.attributes.hex_signing.as_ref())
            .and_then(|signing| signing.keypair.clone())
            .filter(|sealed| !sealed.is_empty())
        {
            return Ok(sealed);
        }
        let pem = tokio::task::spawn_blocking(RegistryKey::generate)
            .await
            .map_err(ContentError::internal)?
            .map_err(|error| failed(ctx, error))?;
        let sealed = ctx.content.secrets().seal(&pem);
        let kept =
            astrum_repo::store::keep_signing_key(ctx.db(), ctx.name(), Section::HexSigning, sealed)
                .await
                .map_err(ContentError::internal)?;
        match kept {
            Some(kept) => {
                tracing::info!(repository = ctx.name(), "a registry signing key was made");
                Ok(kept)
            }
            None => Err(ContentError::not_found(format!(
                "the repository {} is gone",
                ctx.name()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn keys_sign_registry_resources_with_sha512() {
        let key = RegistryKey::read(&testing::private_key_pem()).unwrap();
        let signed = key.sign(b"payload".to_vec());
        assert_eq!(
            testing::verified(&signed, &key.public_key).as_deref(),
            Some(&b"payload"[..])
        );
        let public = public_key_of(&key.public_key).unwrap();
        assert_eq!(verify(&public, &signed).as_deref(), Some(&b"payload"[..]));
        let mut tampered = proto::open(&signed).unwrap();
        tampered.payload = b"other".to_vec();
        let tampered = proto::wrap(tampered.payload, tampered.signature.unwrap());
        assert!(verify(&public, &tampered).is_none());
        assert!(key.public_key.starts_with("-----BEGIN PUBLIC KEY-----\n"));
        assert!(key.fingerprint.starts_with("SHA256:"));
        assert_eq!(key.fingerprint.len(), 7 + 43);
        let pkcs1 =
            rsa::pkcs1::EncodeRsaPrivateKey::to_pkcs1_pem(&testing::rsa_key(), LineEnding::LF)
                .unwrap()
                .to_string();
        assert_eq!(
            RegistryKey::read(&pkcs1).unwrap().public_key,
            key.public_key
        );
        assert!(RegistryKey::read("garbage").is_err());
    }

    #[test]
    fn fingerprints_follow_the_ssh_encoding() {
        assert_eq!(mpint(&[0x00, 0x01, 0x00, 0x01]), [0x01, 0x00, 0x01]);
        assert_eq!(mpint(&[0x80]), [0x00, 0x80]);
        assert_eq!(mpint(&[0x00]), Vec::<u8>::new());
        let public = public_key_of(KNOWN_PUBLIC).unwrap();
        assert_eq!(fingerprint(&public), KNOWN_FINGERPRINT);
    }

    const KNOWN_PUBLIC: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQC71p0f9Jbv2tu3mSkMsAQ5ru8Q
uS4B3BSI14RsT6JdktrC3W3ojz51fFDj+E1fWPZwYenSDD7dsQV3zMpVLxGoxiRW
RJ+r9P0cgGrOVxWYcj74Z3nlV8qqDDWZkw/8nTG0O+/QQENwVFiE7NfyICwpDLrf
BA3M+uFELmeDgOAN/wIDAQAB
-----END PUBLIC KEY-----
";
    const KNOWN_FINGERPRINT: &str = "SHA256:sReRjwvrTRaTlnklZ0XJ951NIbXjMMHaeVujHe2HbxM";
}
