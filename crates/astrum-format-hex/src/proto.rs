use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use prost::Message;

pub const MAX_RESOURCE: u64 = 512 * 1024 * 1024;

#[derive(Clone, PartialEq, Message)]
pub struct Signed {
    #[prost(bytes = "vec", required, tag = "1")]
    pub payload: Vec<u8>,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub signature: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Names {
    #[prost(message, repeated, tag = "1")]
    pub packages: Vec<NamedPackage>,
    #[prost(string, required, tag = "2")]
    pub repository: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct NamedPackage {
    #[prost(string, required, tag = "1")]
    pub name: String,
    #[prost(message, optional, tag = "3")]
    pub updated_at: Option<Timestamp>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Message)]
pub struct Timestamp {
    #[prost(int64, required, tag = "1")]
    pub seconds: i64,
    #[prost(int32, required, tag = "2")]
    pub nanos: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Versions {
    #[prost(message, repeated, tag = "1")]
    pub packages: Vec<VersionedPackage>,
    #[prost(string, required, tag = "2")]
    pub repository: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct VersionedPackage {
    #[prost(string, required, tag = "1")]
    pub name: String,
    #[prost(string, repeated, tag = "2")]
    pub versions: Vec<String>,
    #[prost(int32, repeated, packed = "true", tag = "3")]
    pub retired: Vec<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Package {
    #[prost(message, repeated, tag = "1")]
    pub releases: Vec<Release>,
    #[prost(string, required, tag = "2")]
    pub name: String,
    #[prost(string, required, tag = "3")]
    pub repository: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Release {
    #[prost(string, required, tag = "1")]
    pub version: String,
    #[prost(bytes = "vec", required, tag = "2")]
    pub inner_checksum: Vec<u8>,
    #[prost(message, repeated, tag = "3")]
    pub dependencies: Vec<Dependency>,
    #[prost(message, optional, tag = "4")]
    pub retired: Option<RetirementStatus>,
    #[prost(bytes = "vec", optional, tag = "5")]
    pub outer_checksum: Option<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
pub struct RetirementStatus {
    #[prost(enumeration = "RetirementReason", required, tag = "1")]
    pub reason: i32,
    #[prost(string, optional, tag = "2")]
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum RetirementReason {
    RetiredOther = 0,
    RetiredInvalid = 1,
    RetiredSecurity = 2,
    RetiredDeprecated = 3,
    RetiredRenamed = 4,
}

impl RetirementReason {
    pub const ALL: [RetirementReason; 5] = [
        RetirementReason::RetiredOther,
        RetirementReason::RetiredInvalid,
        RetirementReason::RetiredSecurity,
        RetirementReason::RetiredDeprecated,
        RetirementReason::RetiredRenamed,
    ];

    pub fn word(self) -> &'static str {
        match self {
            RetirementReason::RetiredOther => "other",
            RetirementReason::RetiredInvalid => "invalid",
            RetirementReason::RetiredSecurity => "security",
            RetirementReason::RetiredDeprecated => "deprecated",
            RetirementReason::RetiredRenamed => "renamed",
        }
    }

    pub fn from_word(word: &str) -> Option<RetirementReason> {
        Self::ALL
            .into_iter()
            .find(|reason| reason.word().eq_ignore_ascii_case(word.trim()))
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Dependency {
    #[prost(string, required, tag = "1")]
    pub package: String,
    #[prost(string, required, tag = "2")]
    pub requirement: String,
    #[prost(bool, optional, tag = "3")]
    pub optional: Option<bool>,
    #[prost(string, optional, tag = "4")]
    pub app: Option<String>,
    #[prost(string, optional, tag = "5")]
    pub repository: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("the resource is not gzip-compressed: {0}")]
    Gzip(std::io::Error),
    #[error("the resource is larger than {MAX_RESOURCE} bytes")]
    TooLarge,
    #[error("the resource is not a signed message: {0}")]
    Decode(#[from] prost::DecodeError),
}

pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .expect("writing to memory does not fail");
    encoder.finish().expect("writing to memory does not fail")
}

pub fn gunzip(data: &[u8]) -> Result<Vec<u8>, ResourceError> {
    let mut plain = Vec::new();
    GzDecoder::new(data)
        .take(MAX_RESOURCE + 1)
        .read_to_end(&mut plain)
        .map_err(ResourceError::Gzip)?;
    if plain.len() as u64 > MAX_RESOURCE {
        return Err(ResourceError::TooLarge);
    }
    Ok(plain)
}

pub fn open(resource: &[u8]) -> Result<Signed, ResourceError> {
    Ok(Signed::decode(gunzip(resource)?.as_slice())?)
}

pub fn wrap(payload: Vec<u8>, signature: Vec<u8>) -> Vec<u8> {
    gzip(
        &Signed {
            payload,
            signature: Some(signature),
        }
        .encode_to_vec(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_use_the_field_numbers_of_the_registry() {
        let names = Names {
            packages: vec![NamedPackage {
                name: "a".to_owned(),
                updated_at: Some(Timestamp {
                    seconds: 1,
                    nanos: 0,
                }),
            }],
            repository: "r".to_owned(),
        };
        assert_eq!(
            names.encode_to_vec(),
            [10, 9, 10, 1, b'a', 26, 4, 8, 1, 16, 0, 18, 1, b'r']
        );
        let versions = Versions {
            packages: vec![VersionedPackage {
                name: "a".to_owned(),
                versions: vec!["1.0.0".to_owned(), "2.0.0".to_owned()],
                retired: vec![1],
            }],
            repository: "r".to_owned(),
        };
        let encoded = versions.encode_to_vec();
        assert!(encoded.ends_with(&[26, 1, 1, 18, 1, b'r']), "{encoded:?}");
        let release = Release {
            version: "1.0.0".to_owned(),
            inner_checksum: vec![1],
            dependencies: vec![Dependency {
                package: "b".to_owned(),
                requirement: "~> 1.0".to_owned(),
                optional: Some(true),
                app: None,
                repository: None,
            }],
            retired: Some(RetirementStatus {
                reason: RetirementReason::RetiredSecurity as i32,
                message: None,
            }),
            outer_checksum: Some(vec![2]),
        };
        let encoded = release.encode_to_vec();
        assert_eq!(&encoded[..9], &[10, 5, b'1', b'.', b'0', b'.', b'0', 18, 1]);
        assert!(encoded.ends_with(&[34, 2, 8, 2, 42, 1, 2]), "{encoded:?}");
        assert_eq!(Release::decode(encoded.as_slice()).unwrap(), release);
    }

    #[test]
    fn signed_resources_are_gzipped_messages() {
        let wrapped = wrap(b"payload".to_vec(), b"signature".to_vec());
        assert_eq!(&wrapped[..2], &[0x1f, 0x8b]);
        let opened = open(&wrapped).unwrap();
        assert_eq!(opened.payload, b"payload");
        assert_eq!(opened.signature.as_deref(), Some(&b"signature"[..]));
        assert!(open(b"plain").is_err());
        assert_eq!(
            RetirementReason::from_word("Security"),
            Some(RetirementReason::RetiredSecurity)
        );
        assert_eq!(RetirementReason::RetiredRenamed.word(), "renamed");
        assert!(RetirementReason::from_word("bad").is_none());
    }
}
