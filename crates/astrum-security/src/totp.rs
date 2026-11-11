use hmac::{Hmac, Mac};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use rand::TryRngCore;
use rand::rngs::OsRng;
use sha1::Sha1;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::Zeroize;

pub const SECRET_BYTES: usize = 20;
pub const DIGITS: usize = 6;
pub const PERIOD_SECONDS: u64 = 30;

const MODULUS: u32 = 1_000_000;
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const BASE32_SECRET_CHARS: usize = 32;
const LABEL_ENCODE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TotpError {
    #[error("the text is not valid base32")]
    InvalidEncoding,
    #[error("a TOTP secret has exactly {SECRET_BYTES} bytes")]
    WrongLength,
}

pub struct TotpSecret {
    bytes: [u8; SECRET_BYTES],
}

impl TotpSecret {
    pub fn generate() -> Self {
        let mut bytes = [0u8; SECRET_BYTES];
        OsRng
            .try_fill_bytes(&mut bytes)
            .expect("the operating system provides random bytes");
        Self { bytes }
    }

    pub fn from_bytes(bytes: [u8; SECRET_BYTES]) -> Self {
        Self { bytes }
    }

    pub fn from_base32(text: &str) -> Result<Self, TotpError> {
        if text.contains('=') {
            return Err(TotpError::InvalidEncoding);
        }
        if text.len() != BASE32_SECRET_CHARS {
            return Err(TotpError::WrongLength);
        }
        let mut decoded = base32_decode(text)?;
        let mut bytes = [0u8; SECRET_BYTES];
        let result = if decoded.len() == SECRET_BYTES {
            bytes.copy_from_slice(&decoded);
            Ok(Self { bytes })
        } else {
            Err(TotpError::WrongLength)
        };
        decoded.zeroize();
        bytes.zeroize();
        result
    }

    pub fn to_base32(&self) -> String {
        base32_encode(&self.bytes)
    }
}

impl Drop for TotpSecret {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl std::fmt::Debug for TotpSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TotpSecret(<redacted>)")
    }
}

pub fn step_at(unix_seconds: u64) -> u64 {
    unix_seconds / PERIOD_SECONDS
}

#[cfg(test)]
thread_local! {
    pub(crate) static HMAC_CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn truncated(secret: &TotpSecret, step: u64) -> u32 {
    #[cfg(test)]
    HMAC_CALLS.with(|calls| calls.set(calls.get() + 1));
    let mut mac = <Hmac<Sha1> as Mac>::new_from_slice(&secret.bytes)
        .expect("HMAC accepts keys of any length");
    mac.update(&step.to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = usize::from(digest[digest.len() - 1] & 0x0f);
    u32::from_be_bytes([
        digest[offset] & 0x7f,
        digest[offset + 1],
        digest[offset + 2],
        digest[offset + 3],
    ])
}

fn digits_at(secret: &TotpSecret, step: u64) -> [u8; DIGITS] {
    let mut value = truncated(secret, step) % MODULUS;
    let mut digits = [b'0'; DIGITS];
    for slot in digits.iter_mut().rev() {
        *slot = b'0' + (value % 10) as u8;
        value /= 10;
    }
    digits
}

pub fn code_at(secret: &TotpSecret, step: u64) -> String {
    digits_at(secret, step)
        .iter()
        .map(|d| char::from(*d))
        .collect()
}

fn parse_code(code: &str) -> Option<[u8; DIGITS]> {
    let bytes = code.as_bytes();
    if bytes.len() != DIGITS || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut digits = [0u8; DIGITS];
    digits.copy_from_slice(bytes);
    Some(digits)
}

pub fn matching_step(
    secret: &TotpSecret,
    code: &str,
    now_unix_seconds: u64,
    window: u8,
) -> Option<u64> {
    let candidate = parse_code(code)?;
    let current = i128::from(step_at(now_unix_seconds));
    let window = i128::from(window);
    let mut matched = Choice::from(0);
    let mut found = 0u64;
    for offset in -window..=window {
        let Ok(step) = u64::try_from(current + offset) else {
            continue;
        };
        let expected = digits_at(secret, step);
        let equal = expected[..].ct_eq(&candidate[..]);
        found = u64::conditional_select(&found, &step, equal);
        matched |= equal;
    }
    bool::from(matched).then_some(found)
}

pub fn otpauth_uri(issuer: &str, account: &str, secret: &TotpSecret) -> String {
    let issuer = utf8_percent_encode(issuer, LABEL_ENCODE);
    let account = utf8_percent_encode(account, LABEL_ENCODE);
    format!(
        "otpauth://totp/{issuer}:{account}?secret={secret}&issuer={issuer}&algorithm=SHA1&digits={DIGITS}&period={PERIOD_SECONDS}",
        secret = secret.to_base32(),
    )
}

fn base32_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity((bytes.len() * 8).div_ceil(5));
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for byte in bytes {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(char::from(
                BASE32_ALPHABET[((buffer >> bits) & 0x1f) as usize],
            ));
        }
        buffer &= (1 << bits) - 1;
    }
    if bits > 0 {
        out.push(char::from(
            BASE32_ALPHABET[((buffer << (5 - bits)) & 0x1f) as usize],
        ));
    }
    out
}

fn base32_decode(text: &str) -> Result<Vec<u8>, TotpError> {
    let trimmed = text.trim_end_matches('=');
    if matches!(trimmed.len() % 8, 1 | 3 | 6) {
        return Err(TotpError::InvalidEncoding);
    }
    let mut out = Vec::with_capacity(trimmed.len() * 5 / 8);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for byte in trimmed.bytes() {
        let value = match byte.to_ascii_uppercase() {
            letter @ b'A'..=b'Z' => letter - b'A',
            digit @ b'2'..=b'7' => digit - b'2' + 26,
            _ => {
                out.zeroize();
                return Err(TotpError::InvalidEncoding);
            }
        };
        buffer = (buffer << 5) | u32::from(value);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    if buffer != 0 {
        out.zeroize();
        return Err(TotpError::InvalidEncoding);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_KEY: [u8; SECRET_BYTES] = *b"12345678901234567890";

    fn rfc_secret() -> TotpSecret {
        TotpSecret::from_bytes(RFC_KEY)
    }

    #[test]
    fn rfc_6238_sha1_vectors() {
        let vectors: [(u64, u32); 6] = [
            (59, 94_287_082),
            (1_111_111_109, 7_081_804),
            (1_111_111_111, 14_050_471),
            (1_234_567_890, 89_005_924),
            (2_000_000_000, 69_279_037),
            (20_000_000_000, 65_353_130),
        ];
        let secret = rfc_secret();
        for (time, eight) in vectors {
            let step = step_at(time);
            assert_eq!(truncated(&secret, step) % 100_000_000, eight, "T={time}");
            assert_eq!(
                code_at(&secret, step),
                format!("{:06}", eight % MODULUS),
                "T={time}"
            );
        }
    }

    #[test]
    fn window_of_one_accepts_neighbours_and_refuses_two_away() {
        let secret = rfc_secret();
        let now = 1_111_111_111;
        let current = step_at(now);
        for (offset, expected) in [
            (-2i64, None),
            (-1, Some(current - 1)),
            (0, Some(current)),
            (1, Some(current + 1)),
            (2, None),
        ] {
            let step = current.checked_add_signed(offset).unwrap();
            let code = code_at(&secret, step);
            assert_eq!(
                matching_step(&secret, &code, now, 1),
                expected,
                "offset {offset}"
            );
        }
    }

    #[test]
    fn the_matched_step_is_returned_so_the_caller_can_refuse_a_replay() {
        let secret = rfc_secret();
        let now = 1_234_567_890;
        let code = code_at(&secret, step_at(now) - 1);
        let first = matching_step(&secret, &code, now, 1);
        assert_eq!(first, Some(step_at(now) - 1));
        assert_eq!(
            matching_step(&secret, &code, now - PERIOD_SECONDS, 1),
            first
        );
        assert_eq!(matching_step(&secret, &code, now + PERIOD_SECONDS, 1), None);
    }

    #[test]
    fn malformed_codes_are_refused_without_panicking() {
        let secret = rfc_secret();
        let valid = code_at(&secret, step_at(59));
        let hostile = [
            "",
            " ",
            &valid[..5],
            &format!("{valid}0"),
            &format!(" {valid}"),
            &format!("{valid} "),
            "12345a",
            "-12345",
            "+12345",
            "１２３４５６",
            "١٢٣٤٥٦",
            "12345\u{0}",
            "\u{1F600}\u{1F600}",
            &"9".repeat(100_000),
        ];
        for code in hostile {
            assert_eq!(matching_step(&secret, code, 59, 1), None, "{code:?}");
        }
        assert_eq!(matching_step(&secret, &valid, 59, 1), Some(1));
    }

    #[test]
    fn the_window_near_time_zero_does_not_underflow() {
        let secret = rfc_secret();
        let code = code_at(&secret, 0);
        assert_eq!(matching_step(&secret, &code, 0, 255), Some(0));
        let late = code_at(&secret, u64::MAX / PERIOD_SECONDS);
        assert_eq!(
            matching_step(&secret, &late, u64::MAX, 255),
            Some(u64::MAX / PERIOD_SECONDS)
        );
    }

    #[test]
    fn base32_round_trips_with_and_without_padding_and_lowercase() {
        for length in 0..=24usize {
            let bytes: Vec<u8> = (0..length as u8)
                .map(|b| b.wrapping_mul(37).wrapping_add(11))
                .collect();
            let text = base32_encode(&bytes);
            assert!(!text.contains('='));
            assert_eq!(base32_decode(&text).unwrap(), bytes);
            let mut padded = text.clone();
            while !padded.len().is_multiple_of(8) {
                padded.push('=');
            }
            assert_eq!(base32_decode(&padded).unwrap(), bytes);
            assert_eq!(base32_decode(&text.to_lowercase()).unwrap(), bytes);
        }
        assert_eq!(base32_encode(b"foobar"), "MZXW6YTBOI");
    }

    #[test]
    fn base32_refuses_foreign_characters_and_impossible_lengths() {
        for text in [
            "A",
            "ABC",
            "ABCDEF",
            "AB1D",
            "AB D",
            "AB=D",
            "ÀB",
            "ABCDEFGH\u{0}",
        ] {
            assert_eq!(
                base32_decode(text),
                Err(TotpError::InvalidEncoding),
                "{text:?}"
            );
        }
    }

    #[test]
    fn base32_refuses_non_canonical_trailing_bits() {
        assert_eq!(base32_decode("AA"), Ok(vec![0]));
        assert_eq!(base32_decode("AB"), Err(TotpError::InvalidEncoding));
        assert_eq!(base32_decode("AAAC"), Err(TotpError::InvalidEncoding));
        assert_eq!(base32_decode("MY"), Ok(b"f".to_vec()));
        assert_eq!(base32_decode("MZ"), Err(TotpError::InvalidEncoding));
    }

    #[test]
    fn steps_that_give_the_same_code_return_the_largest_one() {
        let secret = rfc_secret();
        assert_eq!(code_at(&secret, 910_737), "911617");
        assert_eq!(code_at(&secret, 910_738), "911617");
        for now_step in [910_737u64, 910_738, 910_739] {
            let now = now_step * PERIOD_SECONDS + 5;
            assert_eq!(
                matching_step(&secret, "911617", now, 1),
                Some(910_738),
                "now at step {now_step}"
            );
        }
    }

    #[test]
    fn a_secret_survives_base32_and_wrong_sizes_are_refused() {
        let secret = TotpSecret::generate();
        let text = secret.to_base32();
        assert_eq!(text.len(), 32);
        let again = TotpSecret::from_base32(&text.to_lowercase()).unwrap();
        assert_eq!(again.bytes, secret.bytes);
        for padded in [
            format!("{text}======"),
            format!("{text}="),
            format!("{}======", &text[..26]),
            format!("={text}"),
        ] {
            assert_eq!(
                TotpSecret::from_base32(&padded).err(),
                Some(TotpError::InvalidEncoding),
                "{padded}"
            );
        }
        assert_eq!(
            TotpSecret::from_base32(&text[..31]).err(),
            Some(TotpError::WrongLength)
        );
        assert_eq!(
            TotpSecret::from_base32(&format!("{text}A")).err(),
            Some(TotpError::WrongLength)
        );
        assert_eq!(
            TotpSecret::from_base32("").err(),
            Some(TotpError::WrongLength)
        );
        assert_eq!(
            TotpSecret::from_base32(&"1".repeat(32)).err(),
            Some(TotpError::InvalidEncoding)
        );
        assert_ne!(TotpSecret::generate().bytes, TotpSecret::generate().bytes);
    }

    #[test]
    fn debug_does_not_print_the_secret() {
        let secret = rfc_secret();
        let shown = format!("{secret:?}");
        assert_eq!(shown, "TotpSecret(<redacted>)");
        assert!(!shown.contains(&secret.to_base32()));
        let error = TotpSecret::from_base32(&secret.to_base32()[..5]).unwrap_err();
        assert!(!format!("{error:?} {error}").contains(&secret.to_base32()[..5]));
    }

    #[test]
    fn otpauth_uri_has_the_exact_shape() {
        let secret = rfc_secret();
        assert_eq!(
            otpauth_uri("Astrum (repo.example.com)", "alice", &secret),
            "otpauth://totp/Astrum%20%28repo.example.com%29:alice?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Astrum%20%28repo.example.com%29&algorithm=SHA1&digits=6&period=30"
        );
    }

    #[test]
    fn otpauth_uri_escapes_issuer_and_account() {
        let secret = rfc_secret();
        let uri = otpauth_uri("A&B:C", "bob@example.com?x=1 #y/é", &secret);
        assert!(uri.starts_with(
            "otpauth://totp/A%26B%3AC:bob%40example.com%3Fx%3D1%20%23y%2F%C3%A9?secret="
        ));
        assert!(uri.contains("&issuer=A%26B%3AC&algorithm=SHA1"));
        assert_eq!(uri.matches('?').count(), 1);
        assert_eq!(uri.matches(':').count(), 2);
    }
}
