use astrum_db::entity::trusted_key;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, QueryOrder, SqlErr};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x509_parser::pem::Pem;
use x509_parser::prelude::FromDer;
use x509_parser::public_key::{PublicKey, RSAPublicKey};
use x509_parser::x509::SubjectPublicKeyInfo;

use crate::Error;

pub const ECDSA_P256: &str = "ecdsa-p256";
pub const ECDSA_P384: &str = "ecdsa-p384";
pub const ED25519: &str = "ed25519";
pub const RSA: &str = "rsa";
pub const MAX_PEM_BYTES: usize = 8192;
pub const MIN_RSA_BITS: usize = 2048;
pub const MAX_RSA_BITS: usize = 8192;
pub const MIN_RSA_EXPONENT: u64 = 65537;
pub const MAX_RSA_EXPONENT: u64 = 1 << 32;

const OID_EC_PUBLIC_KEY: &str = "1.2.840.10045.2.1";
const OID_CURVE_P256: &str = "1.2.840.10045.3.1.7";
const OID_CURVE_P384: &str = "1.3.132.0.34";
const OID_ED25519: &str = "1.3.101.112";
const OID_RSA: &str = "1.2.840.113549.1.1.1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedKey {
    pub algorithm: &'static str,
    pub fingerprint: String,
    pub public_key: String,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

pub fn parse_public_key(pem: &str) -> Result<ParsedKey, Error> {
    if pem.len() > MAX_PEM_BYTES {
        return Err(invalid(format!(
            "the public key must be at most {MAX_PEM_BYTES} bytes long"
        )));
    }
    if pem.contains("PRIVATE KEY") {
        return Err(invalid(
            "a private key cannot be stored as a trusted key; give the public key",
        ));
    }
    let mut blocks = Pem::iter_from_buffer(pem.as_bytes());
    let block = match blocks.next() {
        Some(Ok(block)) => block,
        Some(Err(error)) => return Err(invalid(format!("the PEM text is not valid: {error}"))),
        None => return Err(invalid("the public key must be PEM text")),
    };
    if blocks.next().is_some() {
        return Err(invalid("give exactly one public key"));
    }
    if end_label(pem) != Some(block.label.as_str()) {
        return Err(invalid(
            "the PEM BEGIN and END lines must name the same block",
        ));
    }
    if block.label != "PUBLIC KEY" {
        return Err(invalid(format!(
            "the PEM block is \"{}\"; give a \"PUBLIC KEY\" block (SubjectPublicKeyInfo)",
            block.label
        )));
    }
    let (rest, info) = SubjectPublicKeyInfo::from_der(&block.contents)
        .map_err(|_| invalid("the public key is not a valid SubjectPublicKeyInfo"))?;
    if !rest.is_empty() {
        return Err(invalid("the public key has bytes after the key"));
    }
    let algorithm = algorithm_of(&info)?;
    let der = block.contents.as_slice();
    let encoded = STANDARD.encode(der);
    let mut public_key = String::from("-----BEGIN PUBLIC KEY-----\n");
    for line in encoded.as_bytes().chunks(64) {
        public_key.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        public_key.push('\n');
    }
    public_key.push_str("-----END PUBLIC KEY-----\n");
    Ok(ParsedKey {
        algorithm,
        fingerprint: hex::encode(Sha256::digest(der)),
        public_key,
    })
}

fn end_label(pem: &str) -> Option<&str> {
    pem.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("-----END "))
        .and_then(|rest| rest.strip_suffix("-----"))
}

fn modulus_bits(modulus: &[u8]) -> Option<usize> {
    if modulus.first().is_some_and(|byte| byte & 0x80 != 0) {
        return None;
    }
    let start = modulus.iter().position(|byte| *byte != 0)?;
    let significant = &modulus[start..];
    Some(significant.len() * 8 - significant[0].leading_zeros() as usize)
}

fn check_rsa(key: &RSAPublicKey<'_>) -> Result<(), Error> {
    let bits =
        modulus_bits(key.modulus).ok_or_else(|| invalid("the RSA public key is not valid"))?;
    if bits < MIN_RSA_BITS {
        return Err(invalid(format!(
            "an RSA key must have at least {MIN_RSA_BITS} bits"
        )));
    }
    if bits > MAX_RSA_BITS {
        return Err(invalid(format!(
            "an RSA key must have at most {MAX_RSA_BITS} bits"
        )));
    }
    if key.modulus.last().is_some_and(|byte| byte & 1 == 0) {
        return Err(invalid("an RSA modulus must be odd"));
    }
    let exponent = key
        .try_exponent()
        .map_err(|_| invalid("the RSA public exponent is not valid"))?;
    if exponent % 2 == 0 || !(MIN_RSA_EXPONENT..=MAX_RSA_EXPONENT).contains(&exponent) {
        return Err(invalid(format!(
            "an RSA public exponent must be odd and between {MIN_RSA_EXPONENT} and {MAX_RSA_EXPONENT}"
        )));
    }
    Ok(())
}

fn algorithm_of(info: &SubjectPublicKeyInfo<'_>) -> Result<&'static str, Error> {
    let oid = info.algorithm.algorithm.to_id_string();
    let point = info.subject_public_key.data.as_ref();
    match oid.as_str() {
        OID_EC_PUBLIC_KEY => {
            let curve = info
                .algorithm
                .parameters
                .as_ref()
                .and_then(|parameters| parameters.as_oid().ok())
                .map(|curve| curve.to_id_string());
            let (name, length) = match curve.as_deref() {
                Some(OID_CURVE_P256) => (ECDSA_P256, 65),
                Some(OID_CURVE_P384) => (ECDSA_P384, 97),
                _ => return Err(unsupported()),
            };
            if point.len() != length || point.first() != Some(&4) {
                return Err(invalid(
                    "the elliptic curve point must be uncompressed and match the curve",
                ));
            }
            Ok(name)
        }
        OID_ED25519 => match point.len() {
            32 => Ok(ED25519),
            _ => Err(invalid("an Ed25519 public key is 32 bytes long")),
        },
        OID_RSA => match info.parsed() {
            Ok(PublicKey::RSA(key)) => check_rsa(&key).map(|()| RSA),
            _ => Err(invalid("the RSA public key is not valid")),
        },
        _ => Err(unsupported()),
    }
}

fn unsupported() -> Error {
    invalid(format!(
        "the key algorithm is not supported; use {ECDSA_P256}, {ECDSA_P384}, {ED25519} or {RSA}"
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedKey {
    pub name: String,
    pub algorithm: String,
    pub public_key: String,
    pub fingerprint: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

impl TrustedKey {
    fn from_row(row: trusted_key::Model) -> Self {
        Self {
            name: row.name,
            algorithm: row.algorithm,
            public_key: row.public_key,
            fingerprint: row.fingerprint,
            notes: row.notes,
            created_at: row.created_at.with_timezone(&Utc),
        }
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<TrustedKey>, Error> {
    Ok(trusted_key::Entity::find()
        .order_by_asc(trusted_key::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(TrustedKey::from_row)
        .collect())
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<TrustedKey>, Error> {
    Ok(trusted_key::Entity::find_by_id(name.to_owned())
        .one(db)
        .await?
        .map(TrustedKey::from_row))
}

pub async fn create(db: &impl ConnectionTrait, key: &TrustedKey) -> Result<(), Error> {
    if key.public_key.contains("PRIVATE KEY") {
        return Err(Error::Invalid(
            "a private key cannot be stored as a trusted key".to_owned(),
        ));
    }
    let row = trusted_key::ActiveModel {
        name: ActiveValue::Set(key.name.clone()),
        algorithm: ActiveValue::Set(key.algorithm.clone()),
        public_key: ActiveValue::Set(key.public_key.clone()),
        fingerprint: ActiveValue::Set(key.fingerprint.clone()),
        notes: ActiveValue::Set(key.notes.clone()),
        created_at: ActiveValue::Set(key.created_at.fixed_offset()),
    };
    trusted_key::Entity::insert(row)
        .exec(db)
        .await
        .map_err(|error| match error.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => Error::Duplicate(format!(
                "the trusted key \"{}\" or its fingerprint",
                key.name
            )),
            _ => Error::Database(error),
        })?;
    Ok(())
}

pub async fn delete(db: &impl ConnectionTrait, name: &str) -> Result<bool, Error> {
    let deleted = trusted_key::Entity::delete_by_id(name.to_owned())
        .exec(db)
        .await?;
    Ok(deleted.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P256: &str = "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEjlbYB6ZMqNQZi+NyFytE68YWZvvD\njMVTs80I9GOxS4MNCnsT0S5/H7Y/0u0pII+w1TvGjJLD1qCKvJb5FhDpLw==\n-----END PUBLIC KEY-----\n";
    const P384: &str = "-----BEGIN PUBLIC KEY-----\nMHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEaJzCiv8oWh0vCWTJ45x8e5qCET81e29y\nnTPaKgPZavUkdKBfkAIbZhnnzRAW03dQ0Vmw41WIpUXtVGukF6X7CA0GnBIhSErY\nXHUF9WUZ3YrC1VeYPKRKy17oQVy+NokL\n-----END PUBLIC KEY-----\n";
    const ED25519_KEY: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAJE3dInZfUxV36BT4nDCDzkPU0/gziiEq+fMBP78UtCo=\n-----END PUBLIC KEY-----\n";
    const RSA_2048: &str = "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA23+Rt4nAMYWurjilMc5N\n41BvaxChEvP9+krrepKCiPoL8FHzovK43WnIb6ljEU7OEjbDTINGyNeSuCNOpHaD\nBHeNpi0XPGxCmuHLITJY7xfmqvHuZf6sta9K41TcKdHYmBhwu033A2EfZVpsKb5l\ncJ274CyKA16kfUiozeAiSxZ4vrXFUPTWL+iFHk7DCVTfQb09SjTKN3kDPhkeA27Y\nmM3bxCK+Rm/vkqvlt37akKtXIJaqhsn9WSmMwr0ZveeFC/i329PjCsiwV4H3egEl\n6fiaSB7wpugFIZJ0HkdqP47OqTYRV4tUdAASbd5FWu05phwU1ly2i7QzAT0yY5Hy\nrwIDAQAB\n-----END PUBLIC KEY-----\n";
    const RSA_1024: &str = "-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgIz1akZhyY8JRIOJUIA9Zw0jT\n1q2KTkbrHjqLeoXYJtnGaoCzc5SDEgDdzCa+81k0A7R4Ff1bUrCwz5lnZvRMnEF6\nzF84xRxVl0vyhrKnZ9IVZIOSaV00Dc5yl9Xaoym+6JjlgNsp/c6lA3DxoOl8lVTX\nR4UqticodfNpJCjYawIDAQAB\n-----END PUBLIC KEY-----\n";

    #[test]
    fn the_algorithm_and_the_fingerprint_come_from_the_key() {
        for (pem, algorithm) in [
            (P256, ECDSA_P256),
            (P384, ECDSA_P384),
            (ED25519_KEY, ED25519),
            (RSA_2048, RSA),
        ] {
            let parsed = parse_public_key(pem).unwrap();
            assert_eq!(parsed.algorithm, algorithm);
            assert_eq!(parsed.fingerprint.len(), 64);
            assert_eq!(parsed.public_key, pem);
        }
        let padded = format!("\n  \n{P256}\n");
        assert_eq!(
            parse_public_key(&padded).unwrap(),
            parse_public_key(P256).unwrap()
        );
    }

    #[test]
    fn private_keys_and_unusable_input_are_refused() {
        let private = "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n";
        let two = format!("{P256}{ED25519_KEY}");
        let certificate = "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n";
        let oversized = "A".repeat(MAX_PEM_BYTES + 1);
        for text in [
            private,
            "plain text",
            "",
            two.as_str(),
            certificate,
            RSA_1024,
            oversized.as_str(),
        ] {
            let error = parse_public_key(text).unwrap_err();
            assert!(matches!(error, Error::Invalid(_)), "{text}: {error}");
        }
    }

    fn der(tag: u8, content: &[u8]) -> Vec<u8> {
        let mut out = vec![tag];
        match content.len() {
            length if length < 128 => out.push(length as u8),
            length if length < 256 => out.extend([0x81, length as u8]),
            length => out.extend([0x82, (length >> 8) as u8, length as u8]),
        }
        out.extend(content);
        out
    }

    fn integer(value: &[u8]) -> Vec<u8> {
        let mut bytes = value.to_vec();
        if bytes[0] & 0x80 != 0 {
            bytes.insert(0, 0);
        }
        der(0x02, &bytes)
    }

    fn rsa_pem(modulus: &[u8], exponent: &[u8]) -> String {
        let numbers = der(0x30, &[integer(modulus), integer(exponent)].concat());
        let algorithm = der(
            0x30,
            &[
                0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00,
            ],
        );
        let bits = der(0x03, &[&[0u8][..], &numbers].concat());
        let info = der(0x30, &[algorithm, bits].concat());
        let encoded = STANDARD.encode(info);
        let lines: Vec<&str> = encoded
            .as_bytes()
            .chunks(64)
            .map(|line| std::str::from_utf8(line).unwrap())
            .collect();
        format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            lines.join("\n")
        )
    }

    fn modulus(bits: usize) -> Vec<u8> {
        let mut bytes = vec![0xab; bits / 8];
        bytes[0] = 0xc0;
        bytes
    }

    #[test]
    fn rsa_keys_that_can_never_verify_are_refused() {
        let exponent = [0x01, 0x00, 0x01];
        assert_eq!(
            parse_public_key(&rsa_pem(&modulus(2048), &exponent))
                .unwrap()
                .algorithm,
            RSA
        );
        assert_eq!(
            parse_public_key(&rsa_pem(&modulus(8192), &exponent))
                .unwrap()
                .algorithm,
            RSA
        );
        let mut even = modulus(2048);
        *even.last_mut().unwrap() = 0xac;
        let cases = [
            ("too large", rsa_pem(&modulus(8200), &exponent)),
            ("even modulus", rsa_pem(&even, &exponent)),
            (
                "even exponent",
                rsa_pem(&modulus(2048), &[0x01, 0x00, 0x00]),
            ),
            ("small exponent", rsa_pem(&modulus(2048), &[0x03])),
            (
                "huge exponent",
                rsa_pem(&modulus(2048), &[0x01, 0x00, 0x00, 0x00, 0x01]),
            ),
        ];
        for (label, pem) in cases {
            let error = parse_public_key(&pem).unwrap_err();
            assert!(matches!(error, Error::Invalid(_)), "{label}: {error}");
        }
    }

    #[test]
    fn mismatched_pem_labels_are_refused() {
        let swapped = P256.replace("END PUBLIC KEY", "END CERTIFICATE");
        let error = parse_public_key(&swapped).unwrap_err();
        assert!(error.to_string().contains("same block"), "{error}");
    }
}
