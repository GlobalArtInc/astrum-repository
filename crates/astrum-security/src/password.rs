use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rand::RngCore;
use sha2::{Digest, Sha512};
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

pub fn verify(stored: &str, password: &str) -> Verified {
    if stored.starts_with(SHIRO_PREFIX) {
        return match verify_shiro(stored, password) {
            true => Verified::MatchButOutdated,
            false => Verified::Mismatch,
        };
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
    if iterations == 0 || iterations > 1_000_000 {
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
    fn broken_hashes_never_match() {
        for stored in [
            "",
            "plain",
            "$shiro1$SHA-512$x$a$b",
            "$shiro1$MD5$1024$YQ==$YQ==",
            "$shiro1$SHA-512$0$YQ==$YQ==",
            "$argon2id$v=19$m=19456,t=2,p=1$bad",
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
