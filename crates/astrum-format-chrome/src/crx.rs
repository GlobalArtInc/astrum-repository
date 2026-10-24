use std::cmp::Ordering;

use astrum_content::zip::{self, Source, ZipError};
use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::manifest::{self, Manifest};

const MAGIC: &[u8; 4] = b"Cr24";
const PREAMBLE: u64 = 12;
const MAX_HEADER: u64 = 1024 * 1024;
const ID_BYTES: usize = 16;
const RSA_PROOF: u64 = 2;
const ECDSA_PROOF: u64 = 3;
const SIGNED_HEADER_DATA: u64 = 10000;
const PUBLIC_KEY: u64 = 1;
const CRX_ID: u64 = 1;

#[derive(Debug, thiserror::Error)]
pub enum CrxError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Store(#[from] astrum_blob::Error),
}

fn invalid(message: impl Into<String>) -> CrxError {
    CrxError::Invalid(message.into())
}

fn archive_error(error: ZipError) -> CrxError {
    match error {
        ZipError::Store(error) => CrxError::Store(error),
        other => invalid(format!(
            "the archive of the CRX package cannot be read: {other}"
        )),
    }
}

pub fn id_of(bytes: &[u8]) -> String {
    bytes
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0f])
        .map(|nibble| char::from(b'a' + nibble))
        .collect()
}

pub fn id_of_key(public_key: &[u8]) -> String {
    id_of(&Sha256::digest(public_key)[..ID_BYTES])
}

pub fn valid_id(id: &str) -> bool {
    id.len() == ID_BYTES * 2 && id.bytes().all(|byte| (b'a'..=b'p').contains(&byte))
}

pub fn parse_version(version: &str) -> Option<Vec<u32>> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    parts
        .into_iter()
        .map(|part| {
            let digits =
                !part.is_empty() && part.len() <= 5 && part.bytes().all(|b| b.is_ascii_digit());
            let canonical = part == "0" || !part.starts_with('0');
            match digits && canonical {
                true => part.parse::<u32>().ok().filter(|value| *value <= 65535),
                false => None,
            }
        })
        .collect()
}

pub fn valid_version(version: &str) -> bool {
    parse_version(version).is_some()
}

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    let (Some(left), Some(right)) = (parse_version(left), parse_version(right)) else {
        return left.cmp(right);
    };
    (0..4)
        .map(|at| {
            left.get(at)
                .copied()
                .unwrap_or(0)
                .cmp(&right.get(at).copied().unwrap_or(0))
        })
        .find(|order| order.is_ne())
        .unwrap_or(Ordering::Equal)
}

pub fn artifact_path(id: &str, version: &str) -> String {
    format!("/{id}/{version}/{id}-{version}.crx")
}

#[derive(Debug)]
enum Field<'a> {
    Varint,
    Bytes(&'a [u8]),
    Fixed,
}

fn varint(bytes: &[u8], at: &mut usize) -> Result<u64, &'static str> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *bytes.get(*at).ok_or("a number is cut off")?;
        *at += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err("a number is too long")
}

fn fields(bytes: &[u8]) -> Result<Vec<(u64, Field<'_>)>, &'static str> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let key = varint(bytes, &mut at)?;
        let number = key >> 3;
        let field = match key & 7 {
            0 => {
                varint(bytes, &mut at)?;
                Field::Varint
            }
            1 | 5 => {
                let width = if key & 7 == 1 { 8 } else { 4 };
                at = at
                    .checked_add(width)
                    .filter(|end| *end <= bytes.len())
                    .ok_or("a field is cut off")?;
                Field::Fixed
            }
            2 => {
                let length =
                    usize::try_from(varint(bytes, &mut at)?).map_err(|_| "a field is too long")?;
                let end = at
                    .checked_add(length)
                    .filter(|end| *end <= bytes.len())
                    .ok_or("a field is cut off")?;
                let data = &bytes[at..end];
                at = end;
                Field::Bytes(data)
            }
            _ => return Err("a field has an unknown wire type"),
        };
        if number == 0 {
            return Err("a field has the number 0");
        }
        found.push((number, field));
    }
    Ok(found)
}

fn bytes_of<'a>(found: &[(u64, Field<'a>)], wanted: u64) -> Option<&'a [u8]> {
    found.iter().rev().find_map(|(number, field)| match field {
        Field::Bytes(data) if *number == wanted => Some(*data),
        _ => None,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub id: String,
    pub rsa_keys: usize,
    pub ecdsa_keys: usize,
}

pub fn parse_header(header: &[u8]) -> Result<Header, CrxError> {
    let broken = |reason: &str| invalid(format!("the CRX header is broken: {reason}"));
    let found = fields(header).map_err(broken)?;
    let signed = bytes_of(&found, SIGNED_HEADER_DATA)
        .ok_or_else(|| broken("it has no signed header data"))?;
    let signed_fields = fields(signed).map_err(broken)?;
    let crx_id = bytes_of(&signed_fields, CRX_ID).ok_or_else(|| broken("it names no id"))?;
    if crx_id.len() != ID_BYTES {
        return Err(broken("its id is not 16 bytes long"));
    }
    let id = id_of(crx_id);
    let mut rsa_keys = 0;
    let mut ecdsa_keys = 0;
    let mut owned = false;
    for (number, field) in &found {
        let (Field::Bytes(proof), true) = (field, *number == RSA_PROOF || *number == ECDSA_PROOF)
        else {
            continue;
        };
        let proof_fields = fields(proof).map_err(broken)?;
        let key = bytes_of(&proof_fields, PUBLIC_KEY).unwrap_or_default();
        match *number {
            RSA_PROOF => {
                rsa_keys += 1;
                owned |= !key.is_empty() && id_of_key(key) == id;
            }
            _ => ecdsa_keys += 1,
        }
    }
    if !owned {
        return Err(invalid(format!(
            "the CRX header names the id {id}, but none of its RSA keys belongs to that id"
        )));
    }
    Ok(Header {
        id,
        rsa_keys,
        ecdsa_keys,
    })
}

struct Shifted<'a> {
    inner: &'a dyn Source,
    start: u64,
}

#[async_trait]
impl Source for Shifted<'_> {
    fn size(&self) -> u64 {
        self.inner.size().saturating_sub(self.start)
    }

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ZipError> {
        self.inner
            .read(self.start.saturating_add(offset), length)
            .await
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub id: String,
    pub manifest: Manifest,
}

fn u32_at(bytes: &[u8], at: usize) -> u64 {
    let mut word = [0u8; 4];
    word.copy_from_slice(&bytes[at..at + 4]);
    u64::from(u32::from_le_bytes(word))
}

pub async fn inspect(source: &dyn Source) -> Result<Package, CrxError> {
    let size = source.size();
    let preamble = match size >= PREAMBLE {
        true => source.read(0, PREAMBLE).await.map_err(archive_error)?,
        false => Vec::new(),
    };
    if preamble.len() < PREAMBLE as usize || &preamble[..4] != MAGIC {
        return Err(invalid(
            "the file is not a CRX package: it does not start with Cr24",
        ));
    }
    match u32_at(&preamble, 4) {
        3 => {}
        2 => {
            return Err(invalid(
                "the file is a CRX2 package, which Chromium browsers no longer install; pack the extension again as CRX3",
            ));
        }
        other => {
            return Err(invalid(format!(
                "the CRX format version {other} is not supported; only CRX3 is"
            )));
        }
    }
    let header_length = u32_at(&preamble, 8);
    if header_length > MAX_HEADER {
        return Err(invalid("the CRX header is too large"));
    }
    if PREAMBLE + header_length > size {
        return Err(invalid("the CRX header is cut off"));
    }
    let header = source
        .read(PREAMBLE, header_length)
        .await
        .map_err(archive_error)?;
    let parsed = parse_header(&header)?;
    let archive = Shifted {
        inner: source,
        start: PREAMBLE + header_length,
    };
    let manifest = match manifest::read(&archive).await {
        Err(CrxError::Invalid(message)) => match manifest::read(source).await {
            Ok(manifest) => manifest,
            Err(_) => return Err(CrxError::Invalid(message)),
        },
        other => other?,
    };
    Ok(Package {
        id: parsed.id,
        manifest,
    })
}

pub(crate) async fn entries(source: &dyn Source) -> Result<Vec<zip::Entry>, CrxError> {
    zip::entries(source).await.map_err(archive_error)
}

pub(crate) async fn entry(
    source: &dyn Source,
    entry: &zip::Entry,
    limit: u64,
) -> Result<Vec<u8>, CrxError> {
    zip::read(source, entry, limit).await.map_err(archive_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use astrum_content::zip::Memory;

    #[test]
    fn ids_are_written_with_the_letters_a_to_p() {
        assert_eq!(id_of(&[0x00, 0x1f, 0xff]), "aabppp");
        assert!(valid_id(&"a".repeat(32)));
        assert!(valid_id("abcdefghijklmnopabcdefghijklmnop"));
        assert!(!valid_id("abcdefghijklmnopabcdefghijklmnoq"));
        assert!(!valid_id(&"a".repeat(31)));
        assert!(!valid_id(&"A".repeat(32)));
    }

    #[test]
    fn versions_follow_the_extension_rules() {
        assert!(valid_version("1"));
        assert!(valid_version("1.2.3.4"));
        assert!(valid_version("0.65535"));
        assert!(!valid_version("1.2.3.4.5"));
        assert!(!valid_version("1.02"));
        assert!(!valid_version("65536"));
        assert!(!valid_version("1.0-beta"));
        assert!(!valid_version(""));
        assert!(!valid_version("1..2"));
        assert_eq!(compare_versions("1.10", "1.9"), Ordering::Greater);
        assert_eq!(compare_versions("1.0.0.0", "1"), Ordering::Equal);
        assert_eq!(compare_versions("2", "10"), Ordering::Less);
    }

    #[test]
    fn headers_name_the_id_of_their_key() {
        let key = b"an example public key";
        let header = testing::header(key, &[key.as_slice()]);
        let parsed = parse_header(&header).unwrap();
        assert_eq!(parsed.id, id_of_key(key));
        assert_eq!(parsed.rsa_keys, 1);
        let foreign = testing::header(key, &[b"another key".as_slice()]);
        let error = parse_header(&foreign).unwrap_err().to_string();
        assert!(error.contains("none of its RSA keys"), "{error}");
        assert!(parse_header(b"\x0a\x05abc").is_err());
        assert!(parse_header(&[]).is_err());
    }

    #[tokio::test]
    async fn packages_are_read_and_broken_ones_refused() {
        let manifest = r#"{"manifest_version": 3, "name": "Acme", "version": "1.2.3"}"#;
        let package = testing::crx(b"key one", manifest, &[]);
        let found = inspect(&Memory(package.clone())).await.unwrap();
        assert_eq!(found.id, id_of_key(b"key one"));
        assert_eq!(found.manifest.version, "1.2.3");
        assert_eq!(found.manifest.name, "Acme");

        let message =
            |bytes: Vec<u8>| async move { inspect(&Memory(bytes)).await.unwrap_err().to_string() };
        assert!(
            message(b"PK\x03\x04 not a crx".to_vec())
                .await
                .contains("Cr24")
        );
        let mut crx2 = package.clone();
        crx2[4] = 2;
        assert!(message(crx2).await.contains("CRX2"));
        let mut cut = package.clone();
        cut.truncate(20);
        assert!(message(cut).await.contains("cut off"));
        let start = 12 + u32_at(&package, 8) as usize;
        let mut absolute = package.clone();
        let end = absolute.len() - 22;
        let directory = start + u32_at(&absolute, end + 16) as usize;
        let mut at = directory;
        while at < end {
            let local = u32_at(&absolute, at + 42) as u32 + start as u32;
            absolute[at + 42..at + 46].copy_from_slice(&local.to_le_bytes());
            let lengths = [28, 30, 32].map(|offset| {
                u16::from_le_bytes([absolute[at + offset], absolute[at + offset + 1]]) as usize
            });
            at += 46 + lengths.iter().sum::<usize>();
        }
        absolute[end + 16..end + 20].copy_from_slice(&(directory as u32).to_le_bytes());
        let found = inspect(&Memory(absolute)).await.unwrap();
        assert_eq!(found.manifest.version, "1.2.3");

        let mut no_zip = package[..start].to_vec();
        no_zip.extend_from_slice(b"not a zip archive at all");
        assert!(message(no_zip).await.contains("archive of the CRX package"));
    }
}
