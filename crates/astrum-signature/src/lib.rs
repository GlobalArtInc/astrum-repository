mod bundle;
mod json;
mod key;
mod simple;

#[cfg(test)]
mod tests;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub use bundle::verify_bundle;
pub use key::{KeyError, KeyKind, PublicKey};
pub use simple::verify_simple_signing;

pub const MAX_KEY_PEM_BYTES: usize = 16 * 1024;
pub const MAX_SIGNATURE_BYTES: usize = 16 * 1024;
pub const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;
pub const MAX_BUNDLE_BYTES: usize = 1024 * 1024;

const MAX_ECHOED_CHARS: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Claim {
    ImageSignature,
    Attestation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Verified { key: String, claim: Claim },
    Invalid(String),
    Unsupported(String),
}

fn quote(text: &str) -> String {
    let mut cleaned = String::with_capacity(MAX_ECHOED_CHARS + 5);
    for (index, c) in text.chars().enumerate() {
        if index == MAX_ECHOED_CHARS {
            cleaned.push_str("...");
            break;
        }
        if c == '"' || c == '\\' {
            cleaned.push('\\');
        }
        cleaned.push(if c.is_ascii_graphic() || c == ' ' {
            c
        } else {
            '?'
        });
    }
    format!("\"{cleaned}\"")
}

fn manifest_hex(manifest_digest: &str) -> Result<&str, Outcome> {
    match manifest_digest.strip_prefix("sha256:") {
        Some(hex)
            if hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) =>
        {
            Ok(hex)
        }
        _ => Err(Outcome::Invalid(
            "manifest digest is not sha256: followed by 64 lowercase hex digits".to_owned(),
        )),
    }
}

fn decode_base64(encoded: &str, max_decoded: usize, what: &str) -> Result<Vec<u8>, String> {
    let max_encoded = max_decoded.div_ceil(3).saturating_mul(4);
    if encoded.len() > max_encoded {
        return Err(format!("{what} is larger than {max_decoded} bytes"));
    }
    if encoded.is_empty() {
        return Err(format!("{what} is empty"));
    }
    let decoded = STANDARD
        .decode(encoded)
        .map_err(|_| format!("{what} is not valid base64"))?;
    if decoded.len() > max_decoded {
        return Err(format!("{what} is larger than {max_decoded} bytes"));
    }
    Ok(decoded)
}

fn first_verifying<'k>(
    keys: &'k [PublicKey],
    message: &[u8],
    signatures: &[Vec<u8>],
) -> Option<&'k PublicKey> {
    keys.iter()
        .find(|key| signatures.iter().any(|sig| key.verify(message, sig)))
}
