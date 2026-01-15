use std::io::{self, Read, Write};
use std::sync::Arc;

use astrum_blob::{BlobId, BlobStore};
use async_trait::async_trait;
use futures_util::TryStreamExt;

const EOCD: u32 = 0x0605_4b50;
const EOCD64: u32 = 0x0606_4b50;
const EOCD64_LOCATOR: u32 = 0x0706_4b50;
const CENTRAL: u32 = 0x0201_4b50;
const LOCAL: u32 = 0x0403_4b50;
const MAX_COMMENT: u64 = 65_535;
const MAX_DIRECTORY: u64 = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ZipError {
    #[error("the archive cannot be read: {0}")]
    Io(#[from] io::Error),
    #[error("the file is not a valid zip archive: {0}")]
    Invalid(&'static str),
    #[error("the blob store failed: {0}")]
    Store(#[from] astrum_blob::Error),
}

#[async_trait]
pub trait Source: Send + Sync {
    fn size(&self) -> u64;

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ZipError>;
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

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ZipError> {
        let Some(content) = self.store.read_range(self.id, offset, length).await? else {
            return Err(ZipError::Invalid("the blob is missing"));
        };
        let mut buffer = Vec::with_capacity(length as usize);
        let mut stream = content.stream;
        while let Some(chunk) = stream.try_next().await? {
            buffer.extend_from_slice(&chunk);
        }
        Ok(buffer)
    }
}

pub struct Memory(pub Vec<u8>);

#[async_trait]
impl Source for Memory {
    fn size(&self) -> u64 {
        self.0.len() as u64
    }

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ZipError> {
        let start = (offset as usize).min(self.0.len());
        let end = start.saturating_add(length as usize).min(self.0.len());
        Ok(self.0[start..end].to_vec())
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub method: u64,
    pub compressed: u64,
    pub size: u64,
    offset: u64,
}

impl Entry {
    pub fn is_directory(&self) -> bool {
        self.name.ends_with('/')
    }
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
                entry.size = u64_at(data, field).unwrap_or(entry.size);
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

fn parse_directory(directory: &[u8]) -> Vec<Entry> {
    let mut found = Vec::new();
    let mut at = 0;
    while u32_at(directory, at) == Some(CENTRAL as u64) {
        let field = |offset: usize| u16_at(directory, at + offset);
        let (Some(method), Some(name_length), Some(extra_length), Some(comment_length)) =
            (field(10), field(28), field(30), field(32))
        else {
            break;
        };
        let (Some(compressed), Some(size), Some(offset)) = (
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
            size,
            offset,
        };
        zip64_extra(extra, &mut entry, (size, compressed, offset));
        found.push(entry);
        at = next;
    }
    found
}

pub async fn entries(source: &dyn Source) -> Result<Vec<Entry>, ZipError> {
    let size = source.size();
    if size < 22 {
        return Err(ZipError::Invalid("the file is too short"));
    }
    let tail_length = size.min(MAX_COMMENT + 22);
    let tail_start = size - tail_length;
    let tail = source.read(tail_start, tail_length).await?;
    let end = (0..tail.len().saturating_sub(21))
        .rev()
        .find(|at| u32_at(&tail, *at) == Some(EOCD as u64))
        .ok_or(ZipError::Invalid("no end of central directory"))?;
    let mut directory_size = u32_at(&tail, end + 12).unwrap_or_default();
    let mut directory_offset = u32_at(&tail, end + 16).unwrap_or_default();
    if (directory_offset == 0xFFFF_FFFF || directory_size == 0xFFFF_FFFF)
        && end >= 20
        && u32_at(&tail, end - 20) == Some(EOCD64_LOCATOR as u64)
    {
        let record_offset = u64_at(&tail, end - 20 + 8).unwrap_or_default();
        let record = source.read(record_offset, 56).await?;
        if u32_at(&record, 0) != Some(EOCD64 as u64) {
            return Err(ZipError::Invalid("broken zip64 end record"));
        }
        directory_size = u64_at(&record, 40).unwrap_or_default();
        directory_offset = u64_at(&record, 48).unwrap_or_default();
    }
    if directory_size > MAX_DIRECTORY || directory_offset.saturating_add(directory_size) > size {
        return Err(ZipError::Invalid("the central directory is out of bounds"));
    }
    let directory = source.read(directory_offset, directory_size).await?;
    Ok(parse_directory(&directory))
}

pub async fn read(source: &dyn Source, entry: &Entry, limit: u64) -> Result<Vec<u8>, ZipError> {
    if entry.size > limit || entry.compressed > limit.saturating_mul(1024).max(limit) {
        return Err(ZipError::Invalid("an entry of the archive is too large"));
    }
    let header = source.read(entry.offset, 30).await?;
    if u32_at(&header, 0) != Some(LOCAL as u64) {
        return Err(ZipError::Invalid("broken local header"));
    }
    let skip =
        30 + u16_at(&header, 26).unwrap_or_default() + u16_at(&header, 28).unwrap_or_default();
    let data = source.read(entry.offset + skip, entry.compressed).await?;
    match entry.method {
        0 => Ok(data),
        8 => {
            let mut inflated = Vec::with_capacity(entry.size.min(limit) as usize);
            flate2::read::DeflateDecoder::new(data.as_slice())
                .take(limit)
                .read_to_end(&mut inflated)?;
            Ok(inflated)
        }
        _ => Err(ZipError::Invalid("unsupported compression")),
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

pub fn write(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(data)
            .expect("writing into memory does not fail");
        let stored = encoder.finish().expect("writing into memory does not fail");
        let offset = out.len() as u32;
        let crc = crc32(data);
        out.extend_from_slice(&LOCAL.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0]);
        out.extend_from_slice(&8u16.to_le_bytes());
        out.extend_from_slice(&[0, 0, 0x21, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(stored.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&stored);
        central.extend_from_slice(&CENTRAL.to_le_bytes());
        central.extend_from_slice(&[20, 0, 20, 0, 0, 0]);
        central.extend_from_slice(&8u16.to_le_bytes());
        central.extend_from_slice(&[0, 0, 0x21, 0]);
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
    out.extend_from_slice(&EOCD.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&directory_offset.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn written_archives_list_and_read_back() {
        let archive = write(&[("a/go.mod", b"module a\n"), ("a/b.txt", b"hello")]);
        let source = Memory(archive);
        let listed = entries(&source).await.unwrap();
        let names: Vec<&str> = listed.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, ["a/go.mod", "a/b.txt"]);
        assert_eq!(
            read(&source, &listed[0], 1024).await.unwrap(),
            b"module a\n"
        );
        assert!(read(&source, &listed[1], 2).await.is_err());
        assert!(
            entries(&Memory(b"not a zip at all, sorry".to_vec()))
                .await
                .is_err()
        );
    }
}
