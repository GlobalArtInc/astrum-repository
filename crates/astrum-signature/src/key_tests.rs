use super::*;
use crate::tests::{ED25519_HEADER, key, read, text, variants};

const RSA_STRUCTURE_BYTES: usize = 32;

fn block(label: &str, body: &str) -> String {
    format!("-----BEGIN {label}-----\n{body}\n-----END {label}-----\n")
}

fn der_of(name: &str) -> Vec<u8> {
    let body: String = text(&format!("{name}.pub"))
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    STANDARD.decode(body).unwrap()
}

fn parse(der: &[u8]) -> Result<PublicKey, KeyError> {
    PublicKey::from_pem(&block("PUBLIC KEY", &STANDARD.encode(der)))
}

fn long_form_outer_length(der: &[u8]) -> Vec<u8> {
    let header = if der[1] < 0x80 {
        2
    } else {
        2 + usize::from(der[1] & 0x7f)
    };
    let content = &der[header..];
    let mut changed = vec![der[0], 0x84];
    changed.extend_from_slice(&u32::try_from(content.len()).unwrap().to_be_bytes());
    changed.extend_from_slice(content);
    changed
}

#[test]
fn every_supported_algorithm_parses_with_a_stable_fingerprint() {
    let cases = [
        (
            "p256",
            KeyKind::EcdsaP256,
            "ff8f5285d0ae9034cb885d53664f0abe71f44c65b7135d91843cd6e27e0ad13e",
        ),
        (
            "p384",
            KeyKind::EcdsaP384,
            "316925608ae69412edea33fd1fdc362868186b55f68132b1ed550ce9a0a6a9cd",
        ),
        (
            "ed25519",
            KeyKind::Ed25519,
            "90e1add1439c939795cec70f38171706a0904ec2d58c605ad90e5eeff3d98984",
        ),
        (
            "rsa2048",
            KeyKind::Rsa,
            "a3979c5c2ec890668b75e092ea920d0e1dd53251b1a8578b1c1472958051df93",
        ),
    ];
    for (name, kind, fingerprint) in cases {
        let parsed = key(name);
        assert_eq!(parsed.kind(), kind, "{name}");
        assert_eq!(parsed.fingerprint(), fingerprint, "{name}");
    }
    assert_eq!(KeyKind::EcdsaP384.as_str(), "ecdsa-p384");
}

#[test]
fn keys_outside_the_supported_set_are_refused() {
    let p256 = text("p256.pub");
    let mut trailing = der_of("p256");
    trailing.push(0);
    let cases = [
        (
            text("p521.pub"),
            KeyError::Curve("secp521r1 (P-521)".to_owned()),
        ),
        (text("dsa.pub"), KeyError::Algorithm("DSA".to_owned())),
        (text("rsa1024.pub"), KeyError::RsaSize(1024)),
        (
            block("PRIVATE KEY", "AAAA"),
            KeyError::WrongLabel("\"PRIVATE KEY\"".to_owned()),
        ),
        (
            text("certificate.pem"),
            KeyError::WrongLabel("\"CERTIFICATE\"".to_owned()),
        ),
        (format!("{p256}{p256}"), KeyError::MultipleBlocks),
        (format!("{p256}garbage"), KeyError::TrailingData),
        (
            format!("{p256}{}", " ".repeat(MAX_KEY_PEM_BYTES)),
            KeyError::TooLarge,
        ),
        (
            block("PUBLIC KEY", &STANDARD.encode(&trailing)),
            KeyError::TrailingData,
        ),
        (block("PUBLIC KEY", "!!!!"), KeyError::Base64),
        (block("PUBLIC KEY", "AAAA"), KeyError::Der),
        (String::new(), KeyError::NotPem),
        (
            p256.replace("END PUBLIC KEY", "END OTHER"),
            KeyError::NotPem,
        ),
        (
            p256.replace("-----END PUBLIC KEY-----", ""),
            KeyError::NotPem,
        ),
    ];
    for (input, expected) in cases {
        let shown: String = input.chars().take(40).collect();
        assert_eq!(
            PublicKey::from_pem(&input).unwrap_err(),
            expected,
            "{shown}"
        );
    }
}

#[test]
fn points_off_the_curve_are_refused() {
    for (name, point_len) in [("p256", P256_POINT_LEN), ("p384", P384_POINT_LEN)] {
        let original = der_of(name);
        assert!(parse(&original).is_ok(), "{name}");
        let start = original.len() - point_len;
        let mut flipped = original.clone();
        *flipped.last_mut().unwrap() ^= 1;
        let mut zeros = original.clone();
        zeros[start + 1..].fill(0);
        let mut ones = original.clone();
        ones[start + 1..].fill(0xff);
        for (case, der) in [("flipped", flipped), ("zeros", zeros), ("0xff", ones)] {
            assert_eq!(
                parse(&der).unwrap_err(),
                KeyError::NotOnCurve,
                "{name} {case}"
            );
        }
    }
}

#[test]
fn ed25519_keys_of_small_order_are_refused() {
    let normal = der_of("ed25519");
    assert_eq!(normal[..ED25519_HEADER.len()], ED25519_HEADER);
    assert!(parse(&normal).is_ok());
    let canonical = [
        "0100000000000000000000000000000000000000000000000000000000000000",
        "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000080",
        "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05",
        "26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc85",
        "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a",
        "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac03fa",
    ];
    let non_canonical = [
        "0100000000000000000000000000000000000000000000000000000000000080",
        "ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
        "edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f",
        "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    ];
    for encoding in canonical.into_iter().chain(non_canonical) {
        let mut der = ED25519_HEADER.to_vec();
        der.extend_from_slice(&hex::decode(encoding).unwrap());
        assert_eq!(parse(&der).unwrap_err(), KeyError::WeakKey, "{encoding}");
    }
}

#[test]
fn only_the_canonical_encoding_of_a_key_is_accepted() {
    let scans = [
        ("p256", der_of("p256").len()),
        ("ed25519", ED25519_HEADER.len()),
        ("rsa2048", RSA_STRUCTURE_BYTES),
    ];
    for (name, scanned) in scans {
        let original = der_of(name);
        let fingerprint = parse(&original).unwrap().fingerprint().to_owned();
        for index in 0..scanned {
            for value in 0..=u8::MAX {
                if value == original[index] {
                    continue;
                }
                let mut changed = original.clone();
                changed[index] = value;
                if let Ok(accepted) = parse(&changed) {
                    assert_eq!(
                        accepted.fingerprint(),
                        fingerprint,
                        "{name} {index} {value}"
                    );
                }
            }
        }
    }
}

#[test]
fn encodings_with_extra_or_mistyped_parts_are_refused() {
    let p256 = der_of("p256");
    let point = &p256[p256.len() - P256_POINT_LEN..];
    let ec = der(TAG_OID, OID_EC_PUBLIC_KEY);
    let curve = der(TAG_OID, OID_CURVE_P256);
    let null = der(TAG_NULL, &[]);
    let spki = |algorithm: &[u8]| encode_spki(algorithm, point);
    let mut bits = vec![0];
    bits.extend_from_slice(point);
    let fourth = der(
        TAG_SEQUENCE,
        &[
            der(TAG_SEQUENCE, &[ec.clone(), curve.clone()].concat()),
            der(TAG_BIT_STRING, &bits),
            null.clone(),
        ]
        .concat(),
    );
    assert!(parse(&spki(&[ec.clone(), curve.clone()].concat())).is_ok());
    let rsa = der_of("rsa2048");
    let rsa_bits = rsa[19..].to_vec();
    let absent = der(
        TAG_SEQUENCE,
        &[der(TAG_SEQUENCE, &der(TAG_OID, OID_RSA)), rsa_bits.clone()].concat(),
    );
    let modulus = RSAPublicKey::from_der(&rsa_bits[5..])
        .unwrap()
        .1
        .modulus
        .to_vec();
    let padded_exponent = der(
        TAG_SEQUENCE,
        &[der(TAG_INTEGER, &modulus), der(TAG_INTEGER, &[0, 1, 0, 1])].concat(),
    );
    let rsa_algorithm = [der(TAG_OID, OID_RSA), null.clone()].concat();
    let cases = [
        (
            "extra algorithm element",
            spki(&[ec.clone(), curve.clone(), null.clone()].concat()),
            None,
        ),
        ("fourth element", fourth, None),
        ("long form length", long_form_outer_length(&p256), None),
        (
            "octet string curve",
            spki(&[ec.clone(), der(0x04, OID_CURVE_P256)].concat()),
            Some(KeyError::CurveParameters),
        ),
        (
            "integer curve",
            spki(&[ec.clone(), der(TAG_INTEGER, OID_CURVE_P256)].concat()),
            Some(KeyError::CurveParameters),
        ),
        (
            "null curve",
            spki(&[ec.clone(), null.clone()].concat()),
            Some(KeyError::CurveParameters),
        ),
        (
            "explicit curve",
            spki(&[ec.clone(), der(TAG_SEQUENCE, &[2, 1, 1])].concat()),
            Some(KeyError::CurveParameters),
        ),
        ("no curve", spki(&ec), Some(KeyError::CurveParameters)),
        ("rsa without null", absent, Some(KeyError::NotCanonical)),
        ("rsa long form length", long_form_outer_length(&rsa), None),
        (
            "rsa padded exponent",
            encode_spki(&rsa_algorithm, &padded_exponent),
            None,
        ),
    ];
    for (name, der, expected) in cases {
        let error = parse(&der).unwrap_err();
        if let Some(expected) = expected {
            assert_eq!(error, expected, "{name}");
        }
    }
    assert_eq!(
        KeyError::CurveParameters.to_string(),
        "unsupported curve parameters, only named curves are accepted"
    );
}

#[test]
fn damaged_key_text_never_panics() {
    let (mut accepted, mut refused) = (0, 0);
    for name in ["p256", "p384", "ed25519", "rsa2048"] {
        for changed in variants(&read(&format!("{name}.pub"))) {
            let Ok(changed) = String::from_utf8(changed) else {
                continue;
            };
            match PublicKey::from_pem(&changed) {
                Ok(parsed) => {
                    assert_eq!(parsed.fingerprint().len(), 64);
                    accepted += 1;
                }
                Err(_) => refused += 1,
            }
        }
    }
    assert!(accepted > 0 && refused > 100);
}

#[test]
fn modulus_size_is_measured_in_bits() {
    assert_eq!(modulus_bits(&[0, 0x80, 1]), Some(16));
    assert_eq!(modulus_bits(&[0x7f, 1]), Some(15));
    assert_eq!(modulus_bits(&[0, 0x7f]), None);
    assert_eq!(modulus_bits(&[0x80]), None);
    assert_eq!(modulus_bits(&[]), None);
}
