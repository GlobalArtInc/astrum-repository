use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rand::RngCore;

const PREFIX: &str = "astrum:v1:";
const NONCE: usize = 12;
const KEY: usize = 32;

pub const KEY_FILE: &str = "etc/secrets.key";

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("cannot read the secrets key {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write the secrets key {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("the secrets key {0} does not hold 32 bytes in base64")]
    Key(PathBuf),
    #[error("the value is not a sealed secret")]
    NotSealed,
    #[error("the secret was sealed with another key or was changed")]
    Open,
}

#[derive(Clone)]
pub struct SecretBox {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for SecretBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretBox")
    }
}

impl SecretBox {
    pub fn from_key(key: [u8; KEY]) -> Self {
        Self {
            cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)),
        }
    }

    pub fn load_or_create(path: &Path) -> Result<Self, SecretError> {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let bytes = STANDARD
                    .decode(text.trim())
                    .map_err(|_| SecretError::Key(path.to_owned()))?;
                let key: [u8; KEY] = bytes
                    .try_into()
                    .map_err(|_| SecretError::Key(path.to_owned()))?;
                Ok(Self::from_key(key))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut key = [0u8; KEY];
                rand::rng().fill_bytes(&mut key);
                write_private(path, STANDARD.encode(key).as_bytes()).map_err(|source| {
                    SecretError::Write {
                        path: path.to_owned(),
                        source,
                    }
                })?;
                tracing::info!(path = %path.display(), "created a new secrets key");
                Ok(Self::from_key(key))
            }
            Err(source) => Err(SecretError::Read {
                path: path.to_owned(),
                source,
            }),
        }
    }

    pub fn seal(&self, plain: &str) -> String {
        let mut nonce = [0u8; NONCE];
        rand::rng().fill_bytes(&mut nonce);
        let sealed = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), plain.as_bytes())
            .expect("AES-GCM encrypts any input of sane length");
        let mut out = Vec::with_capacity(NONCE + sealed.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&sealed);
        format!("{PREFIX}{}", STANDARD.encode(out))
    }

    pub fn open(&self, sealed: &str) -> Result<String, SecretError> {
        let encoded = sealed.strip_prefix(PREFIX).ok_or(SecretError::NotSealed)?;
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| SecretError::NotSealed)?;
        if bytes.len() <= NONCE {
            return Err(SecretError::NotSealed);
        }
        let (nonce, body) = bytes.split_at(NONCE);
        let plain = self
            .cipher
            .decrypt(Nonce::from_slice(nonce), body)
            .map_err(|_| SecretError::Open)?;
        String::from_utf8(plain).map_err(|_| SecretError::Open)
    }

    pub fn is_sealed(value: &str) -> bool {
        value.starts_with(PREFIX)
    }
}

pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_values_open_with_the_same_key_only() {
        let first = SecretBox::from_key([7; KEY]);
        let sealed = first.seal("s3cret");
        assert!(SecretBox::is_sealed(&sealed));
        assert!(!sealed.contains("s3cret"));
        assert_ne!(first.seal("s3cret"), sealed, "every seal uses a new nonce");
        assert_eq!(first.open(&sealed).unwrap(), "s3cret");

        let second = SecretBox::from_key([8; KEY]);
        assert!(matches!(second.open(&sealed), Err(SecretError::Open)));
        assert!(matches!(first.open("s3cret"), Err(SecretError::NotSealed)));
        assert!(matches!(
            first.open("astrum:v1:AAAA"),
            Err(SecretError::NotSealed)
        ));
    }

    #[test]
    fn a_tampered_secret_does_not_open() {
        let secrets = SecretBox::from_key([1; KEY]);
        let sealed = secrets.seal("value");
        let mut bytes = STANDARD
            .decode(sealed.strip_prefix(PREFIX).unwrap())
            .unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        let tampered = format!("{PREFIX}{}", STANDARD.encode(bytes));
        assert!(matches!(secrets.open(&tampered), Err(SecretError::Open)));
    }

    #[test]
    fn the_key_file_is_created_once_and_kept_private() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(KEY_FILE);
        let created = SecretBox::load_or_create(&path).unwrap();
        let sealed = created.seal("kept");
        let loaded = SecretBox::load_or_create(&path).unwrap();
        assert_eq!(loaded.open(&sealed).unwrap(), "kept");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        std::fs::write(&path, "short").unwrap();
        assert!(matches!(
            SecretBox::load_or_create(&path),
            Err(SecretError::Key(_))
        ));
    }
}
