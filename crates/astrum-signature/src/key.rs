use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ring::agreement::{
    self, ECDH_P256, ECDH_P384, EphemeralPrivateKey, UnparsedPublicKey as PeerPublicKey,
    agree_ephemeral,
};
use ring::rand::SystemRandom;
use ring::signature::{
    ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1, ED25519, RSA_PKCS1_2048_8192_SHA256,
    RSA_PSS_2048_8192_SHA256, UnparsedPublicKey, VerificationAlgorithm,
};
use sha2::{Digest, Sha256};
use x509_parser::asn1_rs::{Oid, Tag};
use x509_parser::prelude::FromDer;
use x509_parser::public_key::RSAPublicKey;
use x509_parser::x509::SubjectPublicKeyInfo;

use super::{MAX_KEY_PEM_BYTES, quote};

const PEM_LABEL: &str = "PUBLIC KEY";
const OID_EC_PUBLIC_KEY: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];
const OID_CURVE_P256: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07];
const OID_CURVE_P384: &[u8] = &[0x2b, 0x81, 0x04, 0x00, 0x22];
const OID_CURVE_P521: &[u8] = &[0x2b, 0x81, 0x04, 0x00, 0x23];
const OID_ED25519: &[u8] = &[0x2b, 0x65, 0x70];
const OID_RSA: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];
const OID_RSA_PSS: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0a];
const OID_DSA: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x38, 0x04, 0x01];
const TAG_INTEGER: u8 = 0x02;
const TAG_BIT_STRING: u8 = 0x03;
const TAG_NULL: u8 = 0x05;
const TAG_OID: u8 = 0x06;
const TAG_SEQUENCE: u8 = 0x30;
const P256_POINT_LEN: usize = 65;
const P384_POINT_LEN: usize = 97;
const ED25519_KEY_LEN: usize = 32;
const RSA_MIN_BITS: usize = 2048;
const RSA_MAX_BITS: usize = 8192;
const RSA_MAX_EXPONENT: u64 = (1 << 33) - 1;
const ED25519_SMALL_ORDER_Y: [&str; 7] = [
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0100000000000000000000000000000000000000000000000000000000000000",
    "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
    "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05",
    "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a",
    "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
    "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    EcdsaP256,
    EcdsaP384,
    Ed25519,
    Rsa,
}

impl KeyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EcdsaP256 => "ecdsa-p256",
            Self::EcdsaP384 => "ecdsa-p384",
            Self::Ed25519 => "ed25519",
            Self::Rsa => "rsa",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyError {
    TooLarge,
    NotPem,
    WrongLabel(String),
    MultipleBlocks,
    TrailingData,
    Base64,
    Der,
    NotCanonical,
    Algorithm(String),
    Curve(String),
    CurveParameters,
    NotOnCurve,
    Random,
    WeakKey,
    KeyData,
    RsaSize(usize),
    RsaExponent,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(f, "the key text is larger than {MAX_KEY_PEM_BYTES} bytes"),
            Self::NotPem => f.write_str("the text is not a single complete PEM block"),
            Self::WrongLabel(label) => {
                write!(f, "the PEM block is {label}, expected \"{PEM_LABEL}\"")
            }
            Self::MultipleBlocks => f.write_str("the text holds more than one PEM block"),
            Self::TrailingData => f.write_str("there are extra bytes after the key"),
            Self::Base64 => f.write_str("the PEM body is not valid base64"),
            Self::Der => f.write_str("the block is not a valid SubjectPublicKeyInfo"),
            Self::NotCanonical => f.write_str("the key is not in canonical DER form"),
            Self::Algorithm(name) => write!(f, "unsupported key algorithm {name}"),
            Self::Curve(name) => write!(f, "unsupported elliptic curve {name}"),
            Self::CurveParameters => {
                f.write_str("unsupported curve parameters, only named curves are accepted")
            }
            Self::NotOnCurve => f.write_str("the elliptic curve point is not on the curve"),
            Self::Random => f.write_str("the system random generator failed"),
            Self::WeakKey => f.write_str("the Ed25519 key is a point of small order"),
            Self::KeyData => f.write_str("the key data has the wrong length or encoding"),
            Self::RsaSize(bits) => write!(
                f,
                "RSA keys must have {RSA_MIN_BITS} to {RSA_MAX_BITS} bits, this one has {bits}"
            ),
            Self::RsaExponent => f.write_str("the RSA public exponent is not accepted"),
        }
    }
}

impl std::error::Error for KeyError {}

#[derive(Debug, Clone)]
pub struct PublicKey {
    kind: KeyKind,
    material: Vec<u8>,
    fingerprint: String,
}

struct Parsed {
    kind: KeyKind,
    material: Vec<u8>,
    canonical: Vec<u8>,
}

impl PublicKey {
    pub fn from_pem(pem: &str) -> Result<Self, KeyError> {
        Self::from_der(&pem_to_der(pem)?)
    }

    pub fn kind(&self) -> KeyKind {
        self.kind
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub(super) fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let check = |algorithm: &'static dyn VerificationAlgorithm| {
            UnparsedPublicKey::new(algorithm, &self.material)
                .verify(message, signature)
                .is_ok()
        };
        match self.kind {
            KeyKind::EcdsaP256 => check(&ECDSA_P256_SHA256_ASN1),
            KeyKind::EcdsaP384 => check(&ECDSA_P384_SHA384_ASN1),
            KeyKind::Ed25519 => check(&ED25519),
            KeyKind::Rsa => check(&RSA_PKCS1_2048_8192_SHA256) || check(&RSA_PSS_2048_8192_SHA256),
        }
    }

    fn from_der(der: &[u8]) -> Result<Self, KeyError> {
        let (rest, spki) = SubjectPublicKeyInfo::from_der(der).map_err(|_| KeyError::Der)?;
        if !rest.is_empty() {
            return Err(KeyError::TrailingData);
        }
        if spki.subject_public_key.unused_bits != 0 {
            return Err(KeyError::Der);
        }
        let key_bits = spki.subject_public_key.data.as_ref();
        let parsed = match spki.algorithm.algorithm.as_bytes() {
            OID_EC_PUBLIC_KEY => ec_key(&spki, key_bits)?,
            OID_ED25519 => ed25519_key(&spki, key_bits)?,
            OID_RSA => rsa_key(&spki, key_bits)?,
            OID_DSA => return Err(KeyError::Algorithm("DSA".to_owned())),
            OID_RSA_PSS => return Err(KeyError::Algorithm("RSASSA-PSS".to_owned())),
            _ => {
                let name = spki.algorithm.algorithm.to_id_string();
                return Err(KeyError::Algorithm(quote(&name)));
            }
        };
        if parsed.canonical != der {
            return Err(KeyError::NotCanonical);
        }
        Ok(Self {
            kind: parsed.kind,
            material: parsed.material,
            fingerprint: hex::encode(Sha256::digest(der)),
        })
    }
}

fn pem_to_der(pem: &str) -> Result<Vec<u8>, KeyError> {
    if pem.len() > MAX_KEY_PEM_BYTES {
        return Err(KeyError::TooLarge);
    }
    let mut lines = pem.lines().map(str::trim).filter(|line| !line.is_empty());
    let label = lines
        .next()
        .and_then(|line| line.strip_prefix("-----BEGIN "))
        .and_then(|rest| rest.strip_suffix("-----"))
        .ok_or(KeyError::NotPem)?;
    let mut body = String::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if let Some(end) = line
            .strip_prefix("-----END ")
            .and_then(|rest| rest.strip_suffix("-----"))
        {
            if end != label {
                return Err(KeyError::NotPem);
            }
            closed = true;
            break;
        }
        if line.starts_with("-----") {
            return Err(KeyError::NotPem);
        }
        body.push_str(line);
    }
    if !closed {
        return Err(KeyError::NotPem);
    }
    if label != PEM_LABEL {
        return Err(KeyError::WrongLabel(quote(label)));
    }
    if let Some(extra) = lines.next() {
        return Err(if extra.starts_with("-----BEGIN ") {
            KeyError::MultipleBlocks
        } else {
            KeyError::TrailingData
        });
    }
    if body.is_empty() {
        return Err(KeyError::NotPem);
    }
    STANDARD.decode(body).map_err(|_| KeyError::Base64)
}

fn ec_key(spki: &SubjectPublicKeyInfo, point: &[u8]) -> Result<Parsed, KeyError> {
    let parameters = spki
        .algorithm
        .parameters
        .as_ref()
        .filter(|parameters| parameters.tag() == Tag::Oid)
        .ok_or(KeyError::CurveParameters)?;
    let curve = Oid::try_from(parameters.clone()).map_err(|_| KeyError::CurveParameters)?;
    let (kind, length, ecdh) = match curve.as_bytes() {
        OID_CURVE_P256 => (KeyKind::EcdsaP256, P256_POINT_LEN, &ECDH_P256),
        OID_CURVE_P384 => (KeyKind::EcdsaP384, P384_POINT_LEN, &ECDH_P384),
        OID_CURVE_P521 => return Err(KeyError::Curve("secp521r1 (P-521)".to_owned())),
        _ => return Err(KeyError::Curve(quote(&curve.to_id_string()))),
    };
    if point.len() != length || point.first() != Some(&4) {
        return Err(KeyError::KeyData);
    }
    check_on_curve(ecdh, point)?;
    let algorithm = [
        der(TAG_OID, OID_EC_PUBLIC_KEY),
        der(TAG_OID, curve.as_bytes()),
    ]
    .concat();
    Ok(Parsed {
        kind,
        material: point.to_vec(),
        canonical: encode_spki(&algorithm, point),
    })
}

fn check_on_curve(algorithm: &'static agreement::Algorithm, point: &[u8]) -> Result<(), KeyError> {
    let private = EphemeralPrivateKey::generate(algorithm, &SystemRandom::new())
        .map_err(|_| KeyError::Random)?;
    agree_ephemeral(private, &PeerPublicKey::new(algorithm, point), |_| ())
        .map_err(|_| KeyError::NotOnCurve)
}

fn ed25519_key(spki: &SubjectPublicKeyInfo, key: &[u8]) -> Result<Parsed, KeyError> {
    if spki.algorithm.parameters.is_some() || key.len() != ED25519_KEY_LEN {
        return Err(KeyError::KeyData);
    }
    if has_small_order(key) {
        return Err(KeyError::WeakKey);
    }
    Ok(Parsed {
        kind: KeyKind::Ed25519,
        material: key.to_vec(),
        canonical: encode_spki(&der(TAG_OID, OID_ED25519), key),
    })
}

fn has_small_order(key: &[u8]) -> bool {
    let mut y = key.to_vec();
    if let Some(last) = y.last_mut() {
        *last &= 0x7f;
    }
    ED25519_SMALL_ORDER_Y.contains(&hex::encode(y).as_str())
}

fn rsa_key(spki: &SubjectPublicKeyInfo, key: &[u8]) -> Result<Parsed, KeyError> {
    match spki.algorithm.parameters.as_ref() {
        Some(parameters) if parameters.tag() == Tag::Null => {}
        Some(_) => return Err(KeyError::Der),
        None => return Err(KeyError::NotCanonical),
    }
    let (rest, rsa) = RSAPublicKey::from_der(key).map_err(|_| KeyError::Der)?;
    if !rest.is_empty() {
        return Err(KeyError::TrailingData);
    }
    let bits = modulus_bits(rsa.modulus).ok_or(KeyError::Der)?;
    if !(RSA_MIN_BITS..=RSA_MAX_BITS).contains(&bits) {
        return Err(KeyError::RsaSize(bits));
    }
    let exponent = rsa.try_exponent().map_err(|_| KeyError::RsaExponent)?;
    if exponent < 3 || exponent % 2 == 0 || exponent > RSA_MAX_EXPONENT {
        return Err(KeyError::RsaExponent);
    }
    let public = der(
        TAG_SEQUENCE,
        &[
            der(TAG_INTEGER, rsa.modulus),
            der(TAG_INTEGER, &unsigned_integer(exponent)),
        ]
        .concat(),
    );
    let algorithm = [der(TAG_OID, OID_RSA), der(TAG_NULL, &[])].concat();
    Ok(Parsed {
        kind: KeyKind::Rsa,
        canonical: encode_spki(&algorithm, &public),
        material: public,
    })
}

fn modulus_bits(modulus: &[u8]) -> Option<usize> {
    let magnitude = match modulus.split_first()? {
        (0, rest) => match rest.first() {
            Some(first) if first & 0x80 != 0 => rest,
            _ => return None,
        },
        (first, _) if first & 0x80 != 0 => return None,
        _ => modulus,
    };
    let first = *magnitude.first()?;
    let total = magnitude.len().checked_mul(8)?;
    total.checked_sub(first.leading_zeros() as usize)
}

fn unsigned_integer(value: u64) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    let start = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len() - 1);
    let mut content = Vec::with_capacity(bytes.len() - start + 1);
    if bytes[start] & 0x80 != 0 {
        content.push(0);
    }
    content.extend_from_slice(&bytes[start..]);
    content
}

fn der(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(content.len().saturating_add(10));
    encoded.push(tag);
    match u8::try_from(content.len()) {
        Ok(short) if short < 0x80 => encoded.push(short),
        _ => {
            let bytes = content.len().to_be_bytes();
            let start = bytes
                .iter()
                .position(|byte| *byte != 0)
                .unwrap_or(bytes.len() - 1);
            encoded.push(0x80 | (bytes.len() - start) as u8);
            encoded.extend_from_slice(&bytes[start..]);
        }
    }
    encoded.extend_from_slice(content);
    encoded
}

fn encode_spki(algorithm: &[u8], key: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(key.len().saturating_add(1));
    bits.push(0);
    bits.extend_from_slice(key);
    der(
        TAG_SEQUENCE,
        &[der(TAG_SEQUENCE, algorithm), der(TAG_BIT_STRING, &bits)].concat(),
    )
}

#[cfg(test)]
#[path = "key_tests.rs"]
mod tests;
