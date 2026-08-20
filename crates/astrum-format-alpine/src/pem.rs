use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use base64::Engine;
use hmac::{Hmac, Mac};
use md5::Md5;
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

const PBES2: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x05, 0x0d];
const PBKDF2: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x05, 0x0c];
const HMAC_PREFIX: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x02];
const AES_PREFIX: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01];
const DES_EDE3_CBC: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x03, 0x07];
const SEQUENCE: u8 = 0x30;
const INTEGER: u8 = 0x02;
const OCTET_STRING: u8 = 0x04;
const OID: u8 = 0x06;
const MAX_ITERATIONS: u64 = 10_000_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KeyError {
    #[error("the signing key is not a PEM-encoded private key")]
    NotPem,
    #[error("the signing key is not an RSA private key")]
    NotRsa,
    #[error("the signing key is encrypted; enter its passphrase")]
    NeedsPassphrase,
    #[error("the signing key cannot be read with the given passphrase")]
    WrongPassphrase,
    #[error(
        "the signing key is encrypted with {0}, which is not supported; use AES-CBC with PBKDF2"
    )]
    Unsupported(String),
}

struct Block {
    label: String,
    headers: Vec<(String, String)>,
    der: Vec<u8>,
}

fn block(text: &str) -> Result<Block, KeyError> {
    let mut lines = text
        .lines()
        .map(str::trim)
        .skip_while(|line| line.is_empty());
    let label = lines
        .next()
        .and_then(|line| line.strip_prefix("-----BEGIN "))
        .and_then(|line| line.strip_suffix("-----"))
        .ok_or(KeyError::NotPem)?
        .to_owned();
    let end = format!("-----END {label}-----");
    let mut headers = Vec::new();
    let mut body = String::new();
    let mut ended = false;
    for line in lines.by_ref() {
        if line == end {
            ended = true;
            break;
        }
        match line.split_once(':') {
            Some((key, value)) if body.is_empty() && !key.contains(char::is_whitespace) => {
                headers.push((key.to_owned(), value.trim().to_owned()))
            }
            _ => body.push_str(line),
        }
    }
    if !ended {
        return Err(KeyError::NotPem);
    }
    let der = base64::engine::general_purpose::STANDARD
        .decode(body.as_bytes())
        .map_err(|_| KeyError::NotPem)?;
    Ok(Block {
        label,
        headers,
        der,
    })
}

fn tlv(input: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, rest) = input.split_first()?;
    let (&first, mut rest) = rest.split_first()?;
    let length = if first < 0x80 {
        first as usize
    } else {
        let count = (first & 0x7f) as usize;
        if count == 0 || count > 4 {
            return None;
        }
        let (bytes, after) = rest.split_at_checked(count)?;
        rest = after;
        bytes
            .iter()
            .fold(0usize, |length, byte| (length << 8) | *byte as usize)
    };
    let (value, rest) = rest.split_at_checked(length)?;
    Some((tag, value, rest))
}

fn expect(input: &[u8], tag: u8) -> Option<(&[u8], &[u8])> {
    let (found, value, rest) = tlv(input)?;
    (found == tag).then_some((value, rest))
}

fn unsigned(value: &[u8]) -> Option<u64> {
    if value.is_empty() || value.len() > 8 || value[0] & 0x80 != 0 {
        return None;
    }
    Some(
        value
            .iter()
            .fold(0u64, |number, byte| (number << 8) | *byte as u64),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prf {
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cipher {
    Aes128,
    Aes192,
    Aes256,
    DesEde3,
}

impl Cipher {
    fn key_length(self) -> usize {
        match self {
            Cipher::Aes128 => 16,
            Cipher::Aes192 | Cipher::DesEde3 => 24,
            Cipher::Aes256 => 32,
        }
    }

    fn block_length(self) -> usize {
        match self {
            Cipher::DesEde3 => 8,
            _ => 16,
        }
    }

    fn from_oid(oid: &[u8]) -> Option<Cipher> {
        if oid == DES_EDE3_CBC {
            return Some(Cipher::DesEde3);
        }
        match oid.strip_prefix(AES_PREFIX)? {
            [0x02] => Some(Cipher::Aes128),
            [0x16] => Some(Cipher::Aes192),
            [0x2a] => Some(Cipher::Aes256),
            _ => None,
        }
    }

    fn from_name(name: &str) -> Option<Cipher> {
        match name.to_ascii_uppercase().as_str() {
            "AES-128-CBC" => Some(Cipher::Aes128),
            "AES-192-CBC" => Some(Cipher::Aes192),
            "AES-256-CBC" => Some(Cipher::Aes256),
            "DES-EDE3-CBC" => Some(Cipher::DesEde3),
            _ => None,
        }
    }
}

struct Pbes2 {
    salt: Vec<u8>,
    iterations: u64,
    prf: Prf,
    cipher: Cipher,
    iv: Vec<u8>,
    data: Vec<u8>,
}

fn encrypted_info(der: &[u8]) -> Result<Pbes2, KeyError> {
    let unsupported = || KeyError::Unsupported("an unknown scheme".to_owned());
    let (info, _) = expect(der, SEQUENCE).ok_or(KeyError::NotPem)?;
    let (algorithm, rest) = expect(info, SEQUENCE).ok_or(KeyError::NotPem)?;
    let (data, _) = expect(rest, OCTET_STRING).ok_or(KeyError::NotPem)?;
    let (scheme, params) = expect(algorithm, OID).ok_or(KeyError::NotPem)?;
    if scheme != PBES2 {
        return Err(KeyError::Unsupported("PBES1".to_owned()));
    }
    let (params, _) = expect(params, SEQUENCE).ok_or_else(unsupported)?;
    let (kdf, rest) = expect(params, SEQUENCE).ok_or_else(unsupported)?;
    let (encryption, _) = expect(rest, SEQUENCE).ok_or_else(unsupported)?;
    let (kdf_oid, kdf_params) = expect(kdf, OID).ok_or_else(unsupported)?;
    if kdf_oid != PBKDF2 {
        return Err(KeyError::Unsupported(
            "a key derivation other than PBKDF2".to_owned(),
        ));
    }
    let (kdf_params, _) = expect(kdf_params, SEQUENCE).ok_or_else(unsupported)?;
    let (salt, rest) = expect(kdf_params, OCTET_STRING).ok_or_else(unsupported)?;
    let (iterations, mut rest) = expect(rest, INTEGER).ok_or_else(unsupported)?;
    let iterations = unsigned(iterations)
        .filter(|count| (1..=MAX_ITERATIONS).contains(count))
        .ok_or_else(unsupported)?;
    if let Some((INTEGER, _, after)) = tlv(rest) {
        rest = after;
    }
    let prf = match expect(rest, SEQUENCE) {
        None => Prf::Sha1,
        Some((prf, _)) => {
            let (oid, _) = expect(prf, OID).ok_or_else(unsupported)?;
            match oid.strip_prefix(HMAC_PREFIX) {
                Some([0x07]) => Prf::Sha1,
                Some([0x08]) => Prf::Sha224,
                Some([0x09]) => Prf::Sha256,
                Some([0x0a]) => Prf::Sha384,
                Some([0x0b]) => Prf::Sha512,
                _ => {
                    return Err(KeyError::Unsupported(
                        "a PBKDF2 hash other than SHA-1 or SHA-2".to_owned(),
                    ));
                }
            }
        }
    };
    let (cipher_oid, cipher_params) = expect(encryption, OID).ok_or_else(unsupported)?;
    let cipher = Cipher::from_oid(cipher_oid)
        .ok_or_else(|| KeyError::Unsupported("a cipher other than AES-CBC".to_owned()))?;
    let (iv, _) = expect(cipher_params, OCTET_STRING).ok_or_else(unsupported)?;
    if iv.len() != cipher.block_length() {
        return Err(unsupported());
    }
    Ok(Pbes2 {
        salt: salt.to_vec(),
        iterations,
        prf,
        cipher,
        iv: iv.to_vec(),
        data: data.to_vec(),
    })
}

fn pbkdf2<M: Mac + KeyInit + Clone>(password: &[u8], salt: &[u8], rounds: u64, out: &mut [u8]) {
    let prf = <M as KeyInit>::new_from_slice(password).expect("HMAC takes keys of any length");
    for (index, chunk) in out.chunks_mut(M::output_size()).enumerate() {
        let mut mac = prf.clone();
        mac.update(salt);
        mac.update(&(index as u32 + 1).to_be_bytes());
        let mut block = mac.finalize().into_bytes();
        let mut sum = block.clone();
        for _ in 1..rounds {
            let mut mac = prf.clone();
            mac.update(&block);
            block = mac.finalize().into_bytes();
            for (total, byte) in sum.iter_mut().zip(block.iter()) {
                *total ^= byte;
            }
        }
        chunk.copy_from_slice(&sum[..chunk.len()]);
    }
}

fn derive(prf: Prf, password: &[u8], salt: &[u8], rounds: u64, length: usize) -> Vec<u8> {
    let mut key = vec![0; length];
    match prf {
        Prf::Sha1 => pbkdf2::<Hmac<Sha1>>(password, salt, rounds, &mut key),
        Prf::Sha224 => pbkdf2::<Hmac<Sha224>>(password, salt, rounds, &mut key),
        Prf::Sha256 => pbkdf2::<Hmac<Sha256>>(password, salt, rounds, &mut key),
        Prf::Sha384 => pbkdf2::<Hmac<Sha384>>(password, salt, rounds, &mut key),
        Prf::Sha512 => pbkdf2::<Hmac<Sha512>>(password, salt, rounds, &mut key),
    }
    key
}

fn cbc_decrypt<C: BlockDecrypt + KeyInit>(key: &[u8], iv: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    let cipher = C::new_from_slice(key).ok()?;
    let size = C::block_size();
    if data.is_empty() || !data.len().is_multiple_of(size) || iv.len() != size {
        return None;
    }
    let mut previous = iv.to_vec();
    let mut plain = Vec::with_capacity(data.len());
    for chunk in data.chunks(size) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        plain.extend(block.iter().zip(previous.iter()).map(|(a, b)| a ^ b));
        previous = chunk.to_vec();
    }
    let pad = *plain.last()? as usize;
    if pad == 0
        || pad > size
        || plain[plain.len() - pad..]
            .iter()
            .any(|byte| *byte as usize != pad)
    {
        return None;
    }
    plain.truncate(plain.len() - pad);
    Some(plain)
}

fn cbc_encrypt<C: BlockEncrypt + KeyInit>(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    let cipher = C::new_from_slice(key).expect("the key has the length of the cipher");
    let size = C::block_size();
    let pad = size - data.len() % size;
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat_n(pad as u8, pad));
    let mut previous = iv.to_vec();
    let mut out = Vec::with_capacity(padded.len());
    for chunk in padded.chunks(size) {
        let mixed: Vec<u8> = chunk
            .iter()
            .zip(previous.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        let mut block = GenericArray::clone_from_slice(&mixed);
        cipher.encrypt_block(&mut block);
        out.extend_from_slice(&block);
        previous = block.to_vec();
    }
    out
}

fn decrypt(cipher: Cipher, key: &[u8], iv: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    match cipher {
        Cipher::Aes128 => cbc_decrypt::<aes::Aes128>(key, iv, data),
        Cipher::Aes192 => cbc_decrypt::<aes::Aes192>(key, iv, data),
        Cipher::Aes256 => cbc_decrypt::<aes::Aes256>(key, iv, data),
        Cipher::DesEde3 => cbc_decrypt::<des::TdesEde3>(key, iv, data),
    }
}

fn bytes_to_key(password: &[u8], salt: &[u8], length: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(length + 16);
    let mut previous: Vec<u8> = Vec::new();
    while key.len() < length {
        let mut digest = Md5::new();
        digest.update(&previous);
        digest.update(password);
        digest.update(salt);
        previous = digest.finalize().to_vec();
        key.extend_from_slice(&previous);
    }
    key.truncate(length);
    key
}

fn legacy(block: &Block, passphrase: Option<&str>) -> Result<Option<Vec<u8>>, KeyError> {
    let encrypted = block
        .headers
        .iter()
        .any(|(key, value)| key == "Proc-Type" && value.contains("ENCRYPTED"));
    if !encrypted {
        return Ok(None);
    }
    let info = block
        .headers
        .iter()
        .find(|(key, _)| key == "DEK-Info")
        .map(|(_, value)| value.as_str())
        .ok_or(KeyError::NotPem)?;
    let (name, iv) = info.split_once(',').ok_or(KeyError::NotPem)?;
    let cipher = Cipher::from_name(name.trim())
        .ok_or_else(|| KeyError::Unsupported(name.trim().to_owned()))?;
    let iv = hex::decode(iv.trim()).map_err(|_| KeyError::NotPem)?;
    if iv.len() != cipher.block_length() {
        return Err(KeyError::NotPem);
    }
    let passphrase = passphrase
        .filter(|passphrase| !passphrase.is_empty())
        .ok_or(KeyError::NeedsPassphrase)?;
    let key = bytes_to_key(passphrase.as_bytes(), &iv[..8], cipher.key_length());
    decrypt(cipher, &key, &iv, &block.der)
        .map(Some)
        .ok_or(KeyError::WrongPassphrase)
}

pub fn private_key(text: &str, passphrase: Option<&str>) -> Result<RsaPrivateKey, KeyError> {
    let block = block(text)?;
    match block.label.as_str() {
        "RSA PRIVATE KEY" => {
            let decrypted = legacy(&block, passphrase)?;
            let der = decrypted.as_deref().unwrap_or(&block.der);
            RsaPrivateKey::from_pkcs1_der(der).map_err(|_| match decrypted {
                Some(_) => KeyError::WrongPassphrase,
                None => KeyError::NotPem,
            })
        }
        "PRIVATE KEY" => RsaPrivateKey::from_pkcs8_der(&block.der).map_err(|_| KeyError::NotRsa),
        "ENCRYPTED PRIVATE KEY" => {
            let info = encrypted_info(&block.der)?;
            let passphrase = passphrase
                .filter(|passphrase| !passphrase.is_empty())
                .ok_or(KeyError::NeedsPassphrase)?;
            let key = derive(
                info.prf,
                passphrase.as_bytes(),
                &info.salt,
                info.iterations,
                info.cipher.key_length(),
            );
            let plain = decrypt(info.cipher, &key, &info.iv, &info.data)
                .ok_or(KeyError::WrongPassphrase)?;
            RsaPrivateKey::from_pkcs8_der(&plain).map_err(|_| {
                match expect(&plain, SEQUENCE).is_some_and(|(_, rest)| rest.is_empty()) {
                    true => KeyError::NotRsa,
                    false => KeyError::WrongPassphrase,
                }
            })
        }
        "EC PRIVATE KEY" | "DSA PRIVATE KEY" | "OPENSSH PRIVATE KEY" => Err(KeyError::NotRsa),
        _ => Err(KeyError::NotPem),
    }
}

fn armor(label: &str, der: &[u8]) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(der);
    let mut out = format!("-----BEGIN {label}-----\n");
    for line in encoded.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        out.push('\n');
    }
    out.push_str(&format!("-----END {label}-----\n"));
    out
}

fn sequence(parts: &[Vec<u8>]) -> Vec<u8> {
    let body: Vec<u8> = parts.concat();
    with_length(SEQUENCE, &body)
}

fn with_length(tag: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    let length = body.len();
    if length < 0x80 {
        out.push(length as u8);
    } else {
        let bytes: Vec<u8> = length
            .to_be_bytes()
            .into_iter()
            .skip_while(|byte| *byte == 0)
            .collect();
        out.push(0x80 | bytes.len() as u8);
        out.extend(bytes);
    }
    out.extend_from_slice(body);
    out
}

pub fn encrypt_pkcs8(der: &[u8], passphrase: &str, salt: &[u8], iv: &[u8], rounds: u32) -> String {
    let key = derive(Prf::Sha256, passphrase.as_bytes(), salt, rounds as u64, 32);
    let data = cbc_encrypt::<aes::Aes256>(&key, iv, der);
    let mut aes256 = AES_PREFIX.to_vec();
    aes256.push(0x2a);
    let mut hmac_sha256 = HMAC_PREFIX.to_vec();
    hmac_sha256.push(0x09);
    let kdf = sequence(&[
        with_length(OID, PBKDF2),
        sequence(&[
            with_length(OCTET_STRING, salt),
            with_length(INTEGER, &rounds.to_be_bytes()),
            sequence(&[with_length(OID, &hmac_sha256), vec![0x05, 0x00]]),
        ]),
    ]);
    let scheme = sequence(&[with_length(OID, &aes256), with_length(OCTET_STRING, iv)]);
    let algorithm = sequence(&[with_length(OID, PBES2), sequence(&[kdf, scheme])]);
    let info = sequence(&[algorithm, with_length(OCTET_STRING, &data)]);
    armor("ENCRYPTED PRIVATE KEY", &info)
}

pub fn public_key(key: &RsaPrivateKey) -> String {
    use rsa::pkcs8::EncodePublicKey;
    let der = key
        .to_public_key()
        .to_public_key_der()
        .expect("an RSA public key can be encoded");
    armor("PUBLIC KEY", der.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::pkcs8::EncodePrivateKey;

    fn key() -> RsaPrivateKey {
        crate::testing::rsa_key()
    }

    #[test]
    fn plain_keys_are_read_in_both_layouts() {
        let key = key();
        let pkcs8 = key.to_pkcs8_pem(Default::default()).unwrap();
        assert_eq!(private_key(&pkcs8, None).unwrap(), key);
        let pkcs1 = key.to_pkcs1_pem(Default::default()).unwrap();
        assert_eq!(private_key(&pkcs1, Some("ignored")).unwrap(), key);
        assert_eq!(private_key("garbage", None).unwrap_err(), KeyError::NotPem);
        assert_eq!(
            private_key(
                "-----BEGIN EC PRIVATE KEY-----\nAA==\n-----END EC PRIVATE KEY-----\n",
                None
            )
            .unwrap_err(),
            KeyError::NotRsa
        );
        assert!(public_key(&key).starts_with("-----BEGIN PUBLIC KEY-----\nMI"));
    }

    #[test]
    fn encrypted_keys_need_their_passphrase() {
        let key = key();
        let der = key.to_pkcs8_der().unwrap();
        let encrypted = encrypt_pkcs8(der.as_bytes(), "secret", &[3; 16], &[9; 16], 2048);
        assert_eq!(private_key(&encrypted, Some("secret")).unwrap(), key);
        assert_eq!(
            private_key(&encrypted, None).unwrap_err(),
            KeyError::NeedsPassphrase
        );
        assert_eq!(
            private_key(&encrypted, Some("wrong")).unwrap_err(),
            KeyError::WrongPassphrase
        );
    }

    #[test]
    fn pbkdf2_matches_the_rfc_6070_vectors() {
        let mut out = [0; 20];
        pbkdf2::<Hmac<Sha1>>(b"password", b"salt", 2, &mut out);
        assert_eq!(hex::encode(out), "ea6c014dc72d6f8ccd1ed92ace1d41f0d8de8957");
        let mut long = [0; 25];
        pbkdf2::<Hmac<Sha1>>(
            b"passwordPASSWORDpassword",
            b"saltSALTsaltSALTsaltSALTsaltSALTsalt",
            4096,
            &mut long,
        );
        assert_eq!(
            hex::encode(long),
            "3d2eec4fe41c849b80c8d83662c0e44a8b291a964cf2f07038"
        );
    }

    #[test]
    fn legacy_keys_use_the_openssl_key_derivation() {
        let key = key();
        let der = key.to_pkcs1_der().unwrap();
        let iv = [5u8; 16];
        let secret = bytes_to_key(b"secret", &iv[..8], 32);
        let data = cbc_encrypt::<aes::Aes256>(&secret, &iv, der.as_bytes());
        let body = armor("RSA PRIVATE KEY", &data);
        let pem = body.replacen(
            "-----\n",
            &format!(
                "-----\nProc-Type: 4,ENCRYPTED\nDEK-Info: AES-256-CBC,{}\n\n",
                hex::encode_upper(iv)
            ),
            1,
        );
        assert_eq!(private_key(&pem, Some("secret")).unwrap(), key);
        assert_eq!(
            private_key(&pem, Some("other")).unwrap_err(),
            KeyError::WrongPassphrase
        );
        assert_eq!(
            private_key(&pem, None).unwrap_err(),
            KeyError::NeedsPassphrase
        );
    }

    fn openssl(dir: &std::path::Path, args: &[&str]) -> Option<String> {
        let output = std::process::Command::new("openssl")
            .args(args)
            .current_dir(dir)
            .output()
            .ok()?;
        assert!(
            output.status.success(),
            "openssl {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Some(String::from_utf8(output.stdout).unwrap())
    }

    #[test]
    fn keys_written_by_openssl_are_read_and_its_verify_accepts_the_signatures() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        if openssl(path, &["version"]).is_none() {
            eprintln!("openssl is not installed; the interoperability check is skipped");
            return;
        }
        openssl(path, &["genrsa", "-out", "plain.pem", "2048"]).unwrap();
        let plain = std::fs::read_to_string(path.join("plain.pem")).unwrap();
        let key = private_key(&plain, None).unwrap();
        let cases: &[&[&str]] = &[
            &[
                "pkcs8",
                "-topk8",
                "-in",
                "plain.pem",
                "-v2",
                "aes-256-cbc",
                "-passout",
                "pass:sesame",
            ],
            &[
                "pkcs8",
                "-topk8",
                "-in",
                "plain.pem",
                "-v2",
                "aes-128-cbc",
                "-v2prf",
                "hmacWithSHA1",
                "-passout",
                "pass:sesame",
            ],
            &[
                "pkcs8",
                "-topk8",
                "-in",
                "plain.pem",
                "-v2",
                "des3",
                "-v2prf",
                "hmacWithSHA512",
                "-passout",
                "pass:sesame",
            ],
            &[
                "rsa",
                "-in",
                "plain.pem",
                "-aes128",
                "-traditional",
                "-passout",
                "pass:sesame",
            ],
            &[
                "rsa",
                "-in",
                "plain.pem",
                "-des3",
                "-traditional",
                "-passout",
                "pass:sesame",
            ],
            &["rsa", "-in", "plain.pem", "-traditional"],
        ];
        for args in cases {
            let encrypted = openssl(path, args).unwrap();
            assert_eq!(
                private_key(&encrypted, Some("sesame")).unwrap(),
                key,
                "{args:?}"
            );
            if encrypted.contains("ENCRYPTED") {
                assert_eq!(
                    private_key(&encrypted, Some("other")).unwrap_err(),
                    KeyError::WrongPassphrase,
                    "{args:?}"
                );
            }
        }
        let scrypt = openssl(
            path,
            &[
                "pkcs8",
                "-topk8",
                "-in",
                "plain.pem",
                "-scrypt",
                "-passout",
                "pass:sesame",
            ],
        )
        .unwrap();
        assert!(matches!(
            private_key(&scrypt, Some("sesame")).unwrap_err(),
            KeyError::Unsupported(_)
        ));
        let signing = crate::signing::SigningKey::new(key, "openssl.rsa.pub".to_owned());
        std::fs::write(path.join("public.pem"), &signing.public_key).unwrap();
        std::fs::write(path.join("data"), b"APKINDEX").unwrap();
        std::fs::write(path.join("signature"), signing.sign(b"APKINDEX")).unwrap();
        let verified = openssl(
            path,
            &[
                "dgst",
                "-sha1",
                "-verify",
                "public.pem",
                "-signature",
                "signature",
                "data",
            ],
        )
        .unwrap();
        assert_eq!(verified.trim(), "Verified OK");
    }
}
