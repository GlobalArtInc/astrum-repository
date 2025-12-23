use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use hmac::{Hmac, Mac};
use md5::Md5;
use rand::RngCore;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use subtle::ConstantTimeEq;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verified {
    Match,
    MatchButOutdated,
    Mismatch,
}

impl Verified {
    pub fn matched(self) -> bool {
        self != Verified::Mismatch
    }
}

const SHIRO_PREFIX: &str = "$shiro1$";
const MAX_ITERATIONS: u32 = 10_000_000;

fn argon2() -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())
}

pub fn hash(password: &str) -> String {
    let mut salt = [0u8; 16];
    rand::rng().fill_bytes(&mut salt);
    let salt = SaltString::encode_b64(&salt).expect("sixteen bytes make a valid salt");
    argon2()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 accepts any password with the default parameters")
        .to_string()
}

pub fn is_known_hash(stored: &str) -> bool {
    stored.starts_with(SHIRO_PREFIX)
        || pbkdf2_parts(stored).is_some()
        || legacy_hex(stored).is_some()
        || stored.starts_with("$argon2")
}

fn outdated(matched: bool) -> Verified {
    match matched {
        true => Verified::MatchButOutdated,
        false => Verified::Mismatch,
    }
}

pub fn verify(stored: &str, password: &str) -> Verified {
    if stored.starts_with(SHIRO_PREFIX) {
        return outdated(verify_shiro(stored, password));
    }
    if let Some(parts) = pbkdf2_parts(stored) {
        return outdated(parts.verify(password));
    }
    if let Some(kind) = legacy_hex(stored) {
        return outdated(verify_hex(kind, stored, password));
    }
    let Ok(parsed) = PasswordHash::new(stored) else {
        return Verified::Mismatch;
    };
    if argon2()
        .verify_password(password.as_bytes(), &parsed)
        .is_err()
    {
        return Verified::Mismatch;
    }
    let wanted = Params::default();
    let current = parsed.algorithm.as_str() == "argon2id"
        && Params::try_from(&parsed).is_ok_and(|params| {
            params.m_cost() == wanted.m_cost()
                && params.t_cost() == wanted.t_cost()
                && params.p_cost() == wanted.p_cost()
        });
    match current {
        true => Verified::Match,
        false => Verified::MatchButOutdated,
    }
}

fn verify_shiro(stored: &str, password: &str) -> bool {
    let parts: Vec<&str> = stored.trim_start_matches('$').split('$').collect();
    let [_, algorithm, iterations, salt, expected] = parts.as_slice() else {
        return false;
    };
    if *algorithm != "SHA-512" {
        return false;
    }
    let (Ok(iterations), Ok(salt), Ok(expected)) = (
        iterations.parse::<u32>(),
        STANDARD.decode(salt),
        STANDARD.decode(expected),
    ) else {
        return false;
    };
    if iterations == 0 || iterations > MAX_ITERATIONS {
        return false;
    }
    let mut digest = Sha512::new();
    digest.update(&salt);
    digest.update(password.as_bytes());
    let mut value = digest.finalize();
    for _ in 1..iterations {
        value = Sha512::digest(value);
    }
    value.as_slice().ct_eq(&expected).into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prf {
    Sha1,
    Sha256,
    Sha512,
}

struct Pbkdf2 {
    prf: Prf,
    iterations: u32,
    salt: Vec<u8>,
    hash: Vec<u8>,
}

fn adapted_b64(text: &str) -> Option<Vec<u8>> {
    let text = text.replace('.', "+");
    STANDARD
        .decode(&text)
        .or_else(|_| STANDARD_NO_PAD.decode(text.trim_end_matches('=')))
        .ok()
}

fn pbkdf2_parts(stored: &str) -> Option<Pbkdf2> {
    let parts: Vec<&str> = stored.split('$').collect();
    let (algorithm, parameters, salt, hash) = match parts.as_slice() {
        ["", algorithm, parameters, salt, hash] => (*algorithm, *parameters, *salt, *hash),
        _ => return None,
    };
    let prf = match algorithm.to_ascii_lowercase().as_str() {
        "pbkdf2" | "pbkdf2-sha1" | "pbkdf2withhmacsha1" => Prf::Sha1,
        "pbkdf2-sha256" | "pbkdf2withhmacsha256" => Prf::Sha256,
        "pbkdf2-sha512" | "pbkdf2withhmacsha512" => Prf::Sha512,
        _ => return None,
    };
    let mut iterations = parameters.parse::<u32>().ok();
    for parameter in parameters.split(',') {
        if let Some((name, value)) = parameter.split_once('=')
            && matches!(name, "i" | "iterations" | "rounds" | "key_iteration")
        {
            iterations = value.parse().ok();
        }
    }
    let iterations = iterations.filter(|count| (1..=MAX_ITERATIONS).contains(count))?;
    let salt = adapted_b64(salt).filter(|salt| !salt.is_empty())?;
    let hash = adapted_b64(hash).filter(|hash| hash.len() >= 16)?;
    Some(Pbkdf2 {
        prf,
        iterations,
        salt,
        hash,
    })
}

fn pbkdf2<M: Mac + hmac::digest::KeyInit + Clone>(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    out: &mut [u8],
) {
    let prf = <M as Mac>::new_from_slice(password).expect("HMAC takes a key of any length");
    for (index, chunk) in out.chunks_mut(M::output_size()).enumerate() {
        let mut mac = prf.clone();
        mac.update(salt);
        mac.update(&(index as u32 + 1).to_be_bytes());
        let mut block = mac.finalize().into_bytes();
        let mut sum = block.clone();
        for _ in 1..iterations {
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

impl Pbkdf2 {
    fn verify(&self, password: &str) -> bool {
        let mut derived = vec![0u8; self.hash.len()];
        let bytes = password.as_bytes();
        match self.prf {
            Prf::Sha1 => pbkdf2::<Hmac<Sha1>>(bytes, &self.salt, self.iterations, &mut derived),
            Prf::Sha256 => pbkdf2::<Hmac<Sha256>>(bytes, &self.salt, self.iterations, &mut derived),
            Prf::Sha512 => pbkdf2::<Hmac<Sha512>>(bytes, &self.salt, self.iterations, &mut derived),
        }
        derived.ct_eq(&self.hash).into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LegacyHex {
    Md5,
    Sha1,
}

fn legacy_hex(stored: &str) -> Option<LegacyHex> {
    if !stored.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    match stored.len() {
        32 => Some(LegacyHex::Md5),
        40 => Some(LegacyHex::Sha1),
        _ => None,
    }
}

fn verify_hex(kind: LegacyHex, stored: &str, password: &str) -> bool {
    let digest = match kind {
        LegacyHex::Md5 => Md5::digest(password.as_bytes()).to_vec(),
        LegacyHex::Sha1 => Sha1::digest(password.as_bytes()).to_vec(),
    };
    let Ok(expected) = hex_decode(stored) else {
        return false;
    };
    digest.ct_eq(&expected).into()
}

fn hex_decode(text: &str) -> Result<Vec<u8>, ()> {
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).map_err(|_| ()))
        .collect()
}

pub fn random(length: usize) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| ALPHABET[(rng.next_u32() as usize) % ALPHABET.len()] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shiro(password: &str, salt: &[u8], iterations: u32) -> String {
        let mut digest = Sha512::new();
        digest.update(salt);
        digest.update(password.as_bytes());
        let mut value = digest.finalize();
        for _ in 1..iterations {
            value = Sha512::digest(value);
        }
        format!(
            "$shiro1$SHA-512${iterations}${}${}",
            STANDARD.encode(salt),
            STANDARD.encode(value)
        )
    }

    #[test]
    fn new_hashes_verify_and_are_salted() {
        let first = hash("correct horse");
        let second = hash("correct horse");
        assert!(first.starts_with("$argon2id$"));
        assert_ne!(first, second);
        assert_eq!(verify(&first, "correct horse"), Verified::Match);
        assert_eq!(verify(&first, "correct horsE"), Verified::Mismatch);
    }

    #[test]
    fn hashes_from_the_old_server_verify_and_ask_to_be_replaced() {
        let stored = shiro("admin123", b"0123456789abcdef", 1024);
        assert_eq!(verify(&stored, "admin123"), Verified::MatchButOutdated);
        assert_eq!(verify(&stored, "admin124"), Verified::Mismatch);
        assert!(verify(&stored, "admin123").matched());
    }

    #[test]
    fn hashes_up_to_ten_million_rounds_are_read() {
        let stored = shiro("admin123", b"salt", 50_000);
        assert_eq!(verify(&stored, "admin123"), Verified::MatchButOutdated);
        assert!(is_known_hash(&stored));
    }

    fn pbkdf2_hash(
        prf: Prf,
        password: &str,
        salt: &[u8],
        iterations: u32,
        length: usize,
    ) -> Vec<u8> {
        let mut out = vec![0u8; length];
        match prf {
            Prf::Sha1 => pbkdf2::<Hmac<Sha1>>(password.as_bytes(), salt, iterations, &mut out),
            Prf::Sha256 => pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, iterations, &mut out),
            Prf::Sha512 => pbkdf2::<Hmac<Sha512>>(password.as_bytes(), salt, iterations, &mut out),
        }
        out
    }

    #[test]
    fn pbkdf2_follows_rfc_6070() {
        assert_eq!(
            pbkdf2_hash(Prf::Sha1, "password", b"salt", 4096, 20),
            [
                0x4b, 0x00, 0x79, 0x01, 0xb7, 0x65, 0x48, 0x9a, 0xbe, 0xad, 0x49, 0xd9, 0x26, 0xf7,
                0x21, 0xd0, 0x65, 0xa4, 0x29, 0xc1
            ]
        );
    }

    #[test]
    fn pbkdf2_phc_hashes_verify_in_every_spelling() {
        let salt = b"0123456789abcdef";
        let sha256 = pbkdf2_hash(Prf::Sha256, "s3cret", salt, 10_000, 32);
        let sha512 = pbkdf2_hash(Prf::Sha512, "s3cret", salt, 1000, 64);
        let sha1 = pbkdf2_hash(Prf::Sha1, "s3cret", salt, 1024, 20);
        let spellings = [
            format!(
                "$pbkdf2-sha256$i=10000,l=32${}${}",
                STANDARD_NO_PAD.encode(salt),
                STANDARD_NO_PAD.encode(&sha256)
            ),
            format!(
                "$pbkdf2-sha256$10000${}${}",
                STANDARD.encode(salt).replace('+', "."),
                STANDARD.encode(&sha256).replace('+', ".")
            ),
            format!(
                "$PBKDF2WithHmacSHA256$key_iteration=10000,key_len=256${}${}",
                STANDARD.encode(salt),
                STANDARD.encode(&sha256)
            ),
            format!(
                "$pbkdf2-sha512$i=1000${}${}",
                STANDARD.encode(salt),
                STANDARD.encode(&sha512)
            ),
            format!(
                "$pbkdf2$i=1024${}${}",
                STANDARD.encode(salt),
                STANDARD.encode(&sha1)
            ),
        ];
        for stored in &spellings {
            assert!(is_known_hash(stored), "{stored}");
            assert_eq!(
                verify(stored, "s3cret"),
                Verified::MatchButOutdated,
                "{stored}"
            );
            assert_eq!(verify(stored, "s3cres"), Verified::Mismatch, "{stored}");
        }
    }

    #[test]
    fn legacy_unsalted_hex_hashes_verify() {
        let sha1 = "5baa61e4c9b93f3f0682250b6cf8331b7ee68fd8";
        let md5 = "5F4DCC3B5AA765D61D8327DEB882CF99";
        assert_eq!(verify(sha1, "password"), Verified::MatchButOutdated);
        assert_eq!(verify(md5, "password"), Verified::MatchButOutdated);
        assert_eq!(verify(sha1, "Password"), Verified::Mismatch);
        assert!(is_known_hash(md5));
        assert!(!is_known_hash("unused"));
        assert!(!is_known_hash("5baa61e4"));
    }

    #[test]
    fn broken_hashes_never_match() {
        for stored in [
            "",
            "plain",
            "$shiro1$SHA-512$x$a$b",
            "$shiro1$MD5$1024$YQ==$YQ==",
            "$shiro1$SHA-512$0$YQ==$YQ==",
            "$argon2id$v=19$m=19456,t=2,p=1$bad",
            "$pbkdf2-sha256$i=0$YWJj$YWJjZGVmZ2hpamtsbW5vcA",
            "$pbkdf2-md4$i=10$YWJj$YWJjZGVmZ2hpamtsbW5vcA",
            "zzzz61e4c9b93f3f0682250b6cf8331b7ee68fd8",
        ] {
            assert_eq!(verify(stored, ""), Verified::Mismatch, "{stored:?}");
        }
    }

    #[test]
    fn random_passwords_avoid_lookalikes() {
        let value = random(24);
        assert_eq!(value.len(), 24);
        assert!(!value.contains(['0', 'O', 'o', '1', 'l', 'I']));
        assert_ne!(random(24), value);
    }
}
