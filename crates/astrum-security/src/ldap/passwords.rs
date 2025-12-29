use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use md5::Md5;
use sha1::Sha1;
use sha2::digest::{Digest, DynDigest};
use sha2::{Sha256, Sha384, Sha512};
use subtle::ConstantTimeEq;

pub fn matches(password: &str, stored: &[u8]) -> bool {
    if password.is_empty() || stored.is_empty() {
        return false;
    }
    let candidate = password.as_bytes();
    let Ok(value) = std::str::from_utf8(stored) else {
        return bool::from(candidate.ct_eq(stored));
    };
    let Some((scheme, encoded)) = value
        .strip_prefix('{')
        .and_then(|rest| rest.split_once('}'))
    else {
        return bool::from(candidate.ct_eq(stored));
    };
    match scheme.to_ascii_uppercase().as_str() {
        "CLEARTEXT" | "PLAIN" => bool::from(candidate.ct_eq(encoded.as_bytes())),
        "SHA" | "SHA1" => hashed(Box::new(Sha1::new()), 20, false, candidate, encoded),
        "SSHA" | "SSHA1" => hashed(Box::new(Sha1::new()), 20, true, candidate, encoded),
        "SHA256" | "SHA-256" => hashed(Box::new(Sha256::new()), 32, false, candidate, encoded),
        "SSHA256" | "SSHA-256" => hashed(Box::new(Sha256::new()), 32, true, candidate, encoded),
        "SHA384" | "SHA-384" => hashed(Box::new(Sha384::new()), 48, false, candidate, encoded),
        "SSHA384" | "SSHA-384" => hashed(Box::new(Sha384::new()), 48, true, candidate, encoded),
        "SHA512" | "SHA-512" => hashed(Box::new(Sha512::new()), 64, false, candidate, encoded),
        "SSHA512" | "SSHA-512" => hashed(Box::new(Sha512::new()), 64, true, candidate, encoded),
        "MD5" => hashed(Box::new(Md5::new()), 16, false, candidate, encoded),
        "SMD5" => hashed(Box::new(Md5::new()), 16, true, candidate, encoded),
        _ => false,
    }
}

fn hashed(
    mut digest: Box<dyn DynDigest>,
    length: usize,
    salted: bool,
    candidate: &[u8],
    encoded: &str,
) -> bool {
    let Ok(decoded) = STANDARD.decode(encoded.trim()) else {
        return false;
    };
    let fits = match salted {
        true => decoded.len() > length,
        false => decoded.len() == length,
    };
    if !fits {
        return false;
    }
    digest.update(candidate);
    if salted {
        digest.update(&decoded[length..]);
    }
    let computed = digest.finalize();
    bool::from(computed.as_ref().ct_eq(&decoded[..length]))
}

pub fn ssha(password: &str, salt: &[u8]) -> String {
    let mut digest = Sha1::new();
    Digest::update(&mut digest, password.as_bytes());
    Digest::update(&mut digest, salt);
    let mut bytes = digest.finalize().to_vec();
    bytes.extend_from_slice(salt);
    format!("{{SSHA}}{}", STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_hash<D: Digest>(password: &str) -> String {
        STANDARD.encode(D::digest(password.as_bytes()))
    }

    #[test]
    fn plain_and_hashed_values_match() {
        assert!(matches("secret", b"secret"));
        assert!(!matches("secret", b"Secret"));
        assert!(matches("secret", b"{CLEARTEXT}secret"));
        assert!(matches(
            "secret",
            format!("{{SHA}}{}", plain_hash::<Sha1>("secret")).as_bytes()
        ));
        assert!(matches(
            "secret",
            format!("{{sha256}}{}", plain_hash::<Sha256>("secret")).as_bytes()
        ));
        assert!(matches(
            "secret",
            format!("{{SHA512}}{}", plain_hash::<Sha512>("secret")).as_bytes()
        ));
        assert!(matches(
            "secret",
            format!("{{MD5}}{}", plain_hash::<Md5>("secret")).as_bytes()
        ));
        assert!(matches(
            "bob-secret",
            ssha("bob-secret", b"salty").as_bytes()
        ));
        assert!(!matches("wrong", ssha("bob-secret", b"salty").as_bytes()));
    }

    #[test]
    fn empty_or_unknown_values_never_match() {
        assert!(!matches("", b""));
        assert!(!matches("", b"{CLEARTEXT}"));
        assert!(!matches("x", b""));
        assert!(!matches("x", b"{CRYPT}abc"));
        assert!(!matches("x", b"{SSHA}not base64!"));
        assert!(!matches("x", b"{SSHA}AAAA"));
    }
}
