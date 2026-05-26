use std::io::{self, Read};
use std::sync::Arc;

use astrum_blob::{BlobId, BlobStore};
use async_trait::async_trait;
use futures_util::TryStreamExt;

const EOCD: u64 = 0x0605_4b50;
const EOCD64: u64 = 0x0606_4b50;
const EOCD64_LOCATOR: u64 = 0x0706_4b50;
const CENTRAL: u64 = 0x0201_4b50;
const LOCAL: u64 = 0x0403_4b50;
const MAX_COMMENT: u64 = 65_535;
const MAX_DIRECTORY: u64 = 64 * 1024 * 1024;
const MAX_NUSPEC: u64 = 4 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("the package cannot be read: {0}")]
    Io(#[from] io::Error),
    #[error("the package is not a valid zip archive: {0}")]
    Invalid(&'static str),
    #[error("the blob store failed: {0}")]
    Store(#[from] astrum_blob::Error),
}

#[async_trait]
pub trait Source: Send + Sync {
    fn size(&self) -> u64;

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ArchiveError>;
}

pub struct Blob {
    pub store: Arc<dyn BlobStore>,
    pub id: BlobId,
    pub size: u64,
}

#[async_trait]
impl Source for Blob {
    fn size(&self) -> u64 {
        self.size
    }

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ArchiveError> {
        let Some(content) = self.store.read_range(self.id, offset, length).await? else {
            return Err(ArchiveError::Invalid("the blob is missing"));
        };
        let mut buffer = Vec::with_capacity(length as usize);
        let mut stream = content.stream;
        while let Some(chunk) = stream.try_next().await? {
            buffer.extend_from_slice(&chunk);
        }
        Ok(buffer)
    }
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as u64)
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as u64)
}

fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
}

struct Entry {
    name: String,
    method: u64,
    compressed: u64,
    uncompressed: u64,
    offset: u64,
}

fn zip64_extra(extra: &[u8], entry: &mut Entry, raw: (u64, u64, u64)) {
    let mut at = 0;
    while at + 4 <= extra.len() {
        let (Some(id), Some(length)) = (u16_at(extra, at), u16_at(extra, at + 2)) else {
            return;
        };
        let data = &extra[(at + 4).min(extra.len())..(at + 4 + length as usize).min(extra.len())];
        if id == 1 {
            let mut field = 0;
            if raw.0 == 0xFFFF_FFFF {
                entry.uncompressed = u64_at(data, field).unwrap_or(entry.uncompressed);
                field += 8;
            }
            if raw.1 == 0xFFFF_FFFF {
                entry.compressed = u64_at(data, field).unwrap_or(entry.compressed);
                field += 8;
            }
            if raw.2 == 0xFFFF_FFFF {
                entry.offset = u64_at(data, field).unwrap_or(entry.offset);
            }
            return;
        }
        at += 4 + length as usize;
    }
}

fn entries(directory: &[u8]) -> Vec<Entry> {
    let mut found = Vec::new();
    let mut at = 0;
    while u32_at(directory, at) == Some(CENTRAL) {
        let field = |offset: usize| u16_at(directory, at + offset);
        let (Some(method), Some(name_length), Some(extra_length), Some(comment_length)) =
            (field(10), field(28), field(30), field(32))
        else {
            break;
        };
        let (Some(compressed), Some(uncompressed), Some(offset)) = (
            u32_at(directory, at + 20),
            u32_at(directory, at + 24),
            u32_at(directory, at + 42),
        ) else {
            break;
        };
        let name_start = at + 46;
        let extra_start = name_start + name_length as usize;
        let next = extra_start + extra_length as usize + comment_length as usize;
        let (Some(name), Some(extra)) = (
            directory.get(name_start..extra_start),
            directory.get(extra_start..extra_start + extra_length as usize),
        ) else {
            break;
        };
        let mut entry = Entry {
            name: String::from_utf8_lossy(name).into_owned(),
            method,
            compressed,
            uncompressed,
            offset,
        };
        zip64_extra(extra, &mut entry, (uncompressed, compressed, offset));
        found.push(entry);
        at = next;
    }
    found
}

async fn central_directory(source: &dyn Source) -> Result<Vec<u8>, ArchiveError> {
    let size = source.size();
    if size < 22 {
        return Err(ArchiveError::Invalid("the file is too short"));
    }
    let tail_length = size.min(MAX_COMMENT + 22);
    let tail_start = size - tail_length;
    let tail = source.read(tail_start, tail_length).await?;
    let end = (0..tail.len().saturating_sub(21))
        .rev()
        .find(|at| u32_at(&tail, *at) == Some(EOCD))
        .ok_or(ArchiveError::Invalid("no end of central directory"))?;
    let mut directory_size = u32_at(&tail, end + 12).unwrap_or_default();
    let mut directory_offset = u32_at(&tail, end + 16).unwrap_or_default();
    if (directory_offset == 0xFFFF_FFFF || directory_size == 0xFFFF_FFFF)
        && end >= 20
        && u32_at(&tail, end - 20) == Some(EOCD64_LOCATOR)
    {
        let record_offset = u64_at(&tail, end - 20 + 8).unwrap_or_default();
        let record = source.read(record_offset, 56).await?;
        if u32_at(&record, 0) != Some(EOCD64) {
            return Err(ArchiveError::Invalid("broken zip64 end record"));
        }
        directory_size = u64_at(&record, 40).unwrap_or_default();
        directory_offset = u64_at(&record, 48).unwrap_or_default();
    }
    if directory_size > MAX_DIRECTORY || directory_offset.saturating_add(directory_size) > size {
        return Err(ArchiveError::Invalid(
            "the central directory is out of bounds",
        ));
    }
    source.read(directory_offset, directory_size).await
}

fn is_nuspec(name: &str) -> bool {
    !name.contains('/') && !name.contains('\\') && name.to_ascii_lowercase().ends_with(".nuspec")
}

pub async fn nuspec(source: &dyn Source) -> Result<Option<Vec<u8>>, ArchiveError> {
    let directory = central_directory(source).await?;
    let Some(entry) = entries(&directory)
        .into_iter()
        .find(|entry| is_nuspec(&entry.name))
    else {
        return Ok(None);
    };
    if entry.uncompressed > MAX_NUSPEC || entry.compressed > MAX_NUSPEC {
        return Err(ArchiveError::Invalid("the .nuspec file is too large"));
    }
    let header = source.read(entry.offset, 30).await?;
    if u32_at(&header, 0) != Some(LOCAL) {
        return Err(ArchiveError::Invalid("broken local header"));
    }
    let skip =
        30 + u16_at(&header, 26).unwrap_or_default() + u16_at(&header, 28).unwrap_or_default();
    let data = source.read(entry.offset + skip, entry.compressed).await?;
    match entry.method {
        0 => Ok(Some(data)),
        8 => {
            let mut inflated = Vec::with_capacity(entry.uncompressed as usize);
            flate2::read::DeflateDecoder::new(data.as_slice())
                .take(MAX_NUSPEC)
                .read_to_end(&mut inflated)?;
            Ok(Some(inflated))
        }
        _ => Err(ArchiveError::Invalid("unsupported compression")),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::Write;

    use super::*;

    pub struct Memory(pub Vec<u8>);

    #[async_trait]
    impl Source for Memory {
        fn size(&self) -> u64 {
            self.0.len() as u64
        }

        async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ArchiveError> {
            let start = (offset as usize).min(self.0.len());
            let end = (start + length as usize).min(self.0.len());
            Ok(self.0[start..end].to_vec())
        }
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for byte in data {
            crc ^= *byte as u32;
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    pub fn zip(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, data, deflate) in files {
            let stored: Vec<u8> = if *deflate {
                let mut encoder =
                    flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(data).unwrap();
                encoder.finish().unwrap()
            } else {
                data.to_vec()
            };
            let method: u16 = if *deflate { 8 } else { 0 };
            let offset = out.len() as u32;
            let crc = crc32(data);
            out.extend_from_slice(&(LOCAL as u32).to_le_bytes());
            out.extend_from_slice(&[20, 0, 0, 0]);
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&[0, 0, 0, 0]);
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(stored.len() as u32).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&stored);
            central.extend_from_slice(&(CENTRAL as u32).to_le_bytes());
            central.extend_from_slice(&[20, 0, 20, 0, 0, 0]);
            central.extend_from_slice(&method.to_le_bytes());
            central.extend_from_slice(&[0, 0, 0, 0]);
            central.extend_from_slice(&crc.to_le_bytes());
            central.extend_from_slice(&(stored.len() as u32).to_le_bytes());
            central.extend_from_slice(&(data.len() as u32).to_le_bytes());
            central.extend_from_slice(&(name.len() as u16).to_le_bytes());
            central.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            central.extend_from_slice(&offset.to_le_bytes());
            central.extend_from_slice(name.as_bytes());
        }
        let directory_offset = out.len() as u32;
        out.extend_from_slice(&central);
        out.extend_from_slice(&(EOCD as u32).to_le_bytes());
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(central.len() as u32).to_le_bytes());
        out.extend_from_slice(&directory_offset.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out
    }

    #[tokio::test]
    async fn the_root_nuspec_is_found() {
        let package = zip(&[
            ("_rels/.rels", b"<x/>", true),
            ("lib/net8.0/other.nuspec", b"wrong", false),
            ("Probe.nuspec", b"<package/>", true),
        ]);
        assert_eq!(
            nuspec(&Memory(package)).await.unwrap().as_deref(),
            Some(&b"<package/>"[..])
        );
        let stored = zip(&[("Probe.NuSpec", b"<p/>", false)]);
        assert_eq!(
            nuspec(&Memory(stored)).await.unwrap().as_deref(),
            Some(&b"<p/>"[..])
        );
        let none = zip(&[("lib/a.dll", b"", false)]);
        assert_eq!(nuspec(&Memory(none)).await.unwrap(), None);
        assert!(nuspec(&Memory(b"garbage".to_vec())).await.is_err());
        assert!(
            nuspec(&Memory(b"garbage that is long enough to look".to_vec()))
                .await
                .is_err()
        );
    }
}
