use std::collections::HashSet;
use std::fmt::Write;

use rand::TryRngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::password::argon2;

pub const ALPHABET: &[u8; 31] = b"abcdefghjkmnpqrstuvwxyz23456789";
pub const GROUP_LENGTH: usize = 5;
pub const CODE_LENGTH: usize = 2 * GROUP_LENGTH;
pub const SET_SIZE: usize = 10;

const UNBIASED_BELOW: u8 = 248;
const SALT_DOMAIN: &[u8] = b"astrum/recovery-code/v1";
const SALT_LENGTH: usize = 16;
const HASH_LENGTH: usize = 32;

fn symbol(byte: u8) -> Option<u8> {
    (byte < UNBIASED_BELOW).then(|| ALPHABET[usize::from(byte) % ALPHABET.len()])
}

fn random_code() -> String {
    let mut code = String::with_capacity(CODE_LENGTH + 1);
    let mut pool = [0u8; 32];
    let mut taken = 0;
    while taken < CODE_LENGTH {
        OsRng
            .try_fill_bytes(&mut pool)
            .expect("the operating system provides random bytes");
        for byte in pool {
            let Some(symbol) = symbol(byte) else {
                continue;
            };
            if taken == GROUP_LENGTH {
                code.push('-');
            }
            code.push(char::from(symbol));
            taken += 1;
            if taken == CODE_LENGTH {
                break;
            }
        }
    }
    code
}

pub fn generate_set() -> Vec<String> {
    let mut seen = HashSet::with_capacity(SET_SIZE);
    let mut codes = Vec::with_capacity(SET_SIZE);
    while codes.len() < SET_SIZE {
        let code = random_code();
        if seen.insert(code.clone()) {
            codes.push(code);
        }
    }
    codes
}

pub fn normalise(input: &str) -> Option<String> {
    let mut out = String::with_capacity(CODE_LENGTH);
    for character in input.chars() {
        if character == '-' || character.is_whitespace() {
            continue;
        }
        let lower = character.to_ascii_lowercase();
        if !lower.is_ascii() || !ALPHABET.contains(&(lower as u8)) || out.len() == CODE_LENGTH {
            return None;
        }
        out.push(lower);
    }
    (out.len() == CODE_LENGTH).then_some(out)
}

fn salt(context: &str) -> [u8; SALT_LENGTH] {
    let digest = Sha256::new()
        .chain_update(SALT_DOMAIN)
        .chain_update([0u8])
        .chain_update(context.as_bytes())
        .finalize();
    let mut salt = [0u8; SALT_LENGTH];
    salt.copy_from_slice(&digest[..SALT_LENGTH]);
    salt
}

pub fn hash(context: &str, input: &str) -> Option<String> {
    let normalised = normalise(input)?;
    let mut output = [0u8; HASH_LENGTH];
    argon2()
        .hash_password_into(normalised.as_bytes(), &salt(context), &mut output)
        .expect("argon2 accepts a short password, a sixteen byte salt and a 32 byte output");
    let mut hex = String::with_capacity(2 * HASH_LENGTH);
    for byte in output {
        write!(hex, "{byte:02x}").expect("writing to a string cannot fail");
    }
    Some(hex)
}

pub fn matches(context: &str, stored_hash: &str, input: &str) -> bool {
    match hash(context, input) {
        Some(candidate) => candidate.as_bytes().ct_eq(stored_hash.as_bytes()).into(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_the_alphabet_and_the_grouped_shape() {
        for _ in 0..200 {
            for code in generate_set() {
                assert_eq!(code.len(), CODE_LENGTH + 1);
                assert_eq!(code.as_bytes()[GROUP_LENGTH], b'-');
                assert!(
                    code.bytes()
                        .filter(|b| *b != b'-')
                        .all(|b| ALPHABET.contains(&b))
                );
                assert_eq!(normalise(&code).unwrap().len(), CODE_LENGTH);
            }
        }
    }

    #[test]
    fn the_alphabet_has_no_ambiguous_characters() {
        for banned in b"0o1il" {
            assert!(!ALPHABET.contains(banned), "{}", char::from(*banned));
        }
        assert_eq!(ALPHABET.len(), 31);
        assert_eq!(ALPHABET.iter().collect::<HashSet<_>>().len(), 31);
    }

    #[test]
    fn a_set_has_ten_distinct_codes_every_time() {
        for _ in 0..2000 {
            let set = generate_set();
            assert_eq!(set.len(), SET_SIZE);
            assert_eq!(set.iter().collect::<HashSet<_>>().len(), SET_SIZE);
        }
    }

    #[test]
    fn normalise_accepts_case_space_and_hyphen_variants_only() {
        let expected = Some("abcde23456".to_owned());
        for input in [
            "abcde-23456",
            "ABCDE-23456",
            "AbCdE 23456",
            "  abcde23456  ",
            "abcde--23456",
            "a-b-c-d-e-2-3-4-5-6",
            "abcde\t23456\n",
        ] {
            assert_eq!(normalise(input), expected, "{input:?}");
        }
        for input in [
            "",
            "-",
            "abcde-2345",
            "abcde-234567",
            "abcde-2345l",
            "abcde-2345\u{0}",
            "abcde-2345é",
            "ａｂｃｄｅ-23456",
            "İbcde-23456",
            "abcde-23450",
            &"a".repeat(100_000),
        ] {
            assert_eq!(normalise(input), None, "{input:?}");
        }
    }

    #[test]
    fn only_bytes_below_248_make_a_symbol_and_each_symbol_is_equally_likely() {
        let mut counts = std::collections::HashMap::new();
        for byte in 0..=255u8 {
            match symbol(byte) {
                Some(symbol) => *counts.entry(symbol).or_insert(0u32) += 1,
                None => assert!(byte >= 248, "{byte}"),
            }
        }
        assert_eq!((248..=255u8).filter_map(symbol).count(), 0);
        assert_eq!(counts.len(), ALPHABET.len());
        for letter in ALPHABET {
            assert_eq!(counts[letter], 8, "{}", char::from(*letter));
        }
    }

    #[test]
    fn normalise_refuses_lookalikes_controls_and_other_separators() {
        for input in [
            "abcde-2345\u{212A}",
            "abcde2345\u{0}",
            "\u{0}abcde23456",
            "abcde_23456",
            "abcde.23456",
            "abcde/23456",
        ] {
            assert_eq!(normalise(input), None, "{input:?}");
        }
    }

    #[test]
    fn a_hash_matches_the_reference_argon2id_vectors() {
        let vectors = [
            (
                "user-1",
                "af9c8f890808fe21cb0fe8428c76391acaae3e09f6cd5a1b4e3d2a5d8683e547",
            ),
            (
                "user-2",
                "80c30b0ec81920bcd209d89e531574dcf21ce94dc8a771c24e78db6c821881ae",
            ),
            (
                "",
                "a61b39a890af444f9d3d2b5afe7d641ad47397f1abf155c4a0a7623a8a352217",
            ),
        ];
        for (context, expected) in vectors {
            assert_eq!(hash(context, "abcde-23456").as_deref(), Some(expected));
        }
    }

    #[test]
    fn a_hash_is_deterministic_per_context_and_verifies() {
        let code = "abcde-23456";
        let stored = hash("user-1", code).unwrap();
        assert_eq!(stored.len(), 64);
        assert!(
            stored
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
        assert_eq!(hash("user-1", "ABCDE 23456").unwrap(), stored);
        assert_ne!(hash("user-2", code).unwrap(), stored);
        assert_ne!(hash("user-1", "abcde-23457").unwrap(), stored);
        assert!(matches("user-1", &stored, code));
        assert!(matches("user-1", &stored, "  ABCDE23456"));
        assert!(!matches("user-2", &stored, code));
        assert!(!matches("user-1", &stored, "abcde-23457"));
        assert!(!matches("user-1", &stored, "abcde-2345"));
        assert!(!matches("user-1", &stored, ""));
        assert!(!matches("user-1", "", code));
        assert!(!matches("user-1", &stored[..63], code));
        assert!(!matches("user-1", &stored.to_uppercase(), code));
    }

    #[test]
    fn an_invalid_code_has_no_hash() {
        for input in [
            "",
            "not a code",
            "abcde-2345",
            "abcde-234567",
            "abcde-2345l",
        ] {
            assert_eq!(hash("user-1", input), None, "{input:?}");
        }
    }

    #[test]
    fn generated_codes_verify_against_their_own_hash() {
        for code in generate_set() {
            assert!(matches("user-1", &hash("user-1", &code).unwrap(), &code));
        }
    }
}
