use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Arc;

use astrum_blob::{BlobId, BlobStore};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

const EOCD: u64 = 0x0605_4b50;
const EOCD64: u64 = 0x0606_4b50;
const EOCD64_LOCATOR: u64 = 0x0706_4b50;
const CENTRAL: u64 = 0x0201_4b50;
const LOCAL: u64 = 0x0403_4b50;
const MAX_COMMENT: u64 = 65_535;
const MAX_DIRECTORY: u64 = 64 * 1024 * 1024;
const MAX_METADATA: u64 = 16 * 1024 * 1024;
const MAX_TAR_PREFIX: u64 = 32 * 1024 * 1024;
const MAX_TAR_INFLATED: u64 = 512 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("the archive cannot be read: {0}")]
    Io(#[from] io::Error),
    #[error("the archive is not valid: {0}")]
    Invalid(&'static str),
    #[error("the blob store failed: {0}")]
    Store(#[from] astrum_blob::Error),
}

#[async_trait]
pub trait Source: Send + Sync {
    fn size(&self) -> u64;

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ArchiveError>;
}

pub struct LocalFile {
    pub path: PathBuf,
    pub size: u64,
}

#[async_trait]
impl Source for LocalFile {
    fn size(&self) -> u64 {
        self.size
    }

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ArchiveError> {
        let mut file = tokio::fs::File::open(&self.path).await?;
        file.seek(io::SeekFrom::Start(offset)).await?;
        let mut buffer = Vec::with_capacity(length as usize);
        file.take(length).read_to_end(&mut buffer).await?;
        Ok(buffer)
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Wheel,
    Egg,
    ZipSdist,
    TarSdist,
}

impl Kind {
    pub fn of(filename: &str) -> Option<Kind> {
        let lower = filename.to_ascii_lowercase();
        if lower.ends_with(".whl") {
            Some(Kind::Wheel)
        } else if lower.ends_with(".egg") {
            Some(Kind::Egg)
        } else if lower.ends_with(".zip") {
            Some(Kind::ZipSdist)
        } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            Some(Kind::TarSdist)
        } else {
            None
        }
    }

    fn wants(self, entry: &str) -> bool {
        let parts: Vec<&str> = entry.split('/').collect();
        match (self, parts.as_slice()) {
            (Kind::Wheel, [directory, "METADATA"]) => directory.ends_with(".dist-info"),
            (Kind::Egg, ["EGG-INFO", "PKG-INFO"]) => true,
            (Kind::ZipSdist | Kind::TarSdist, [directory, "PKG-INFO"]) => !directory.is_empty(),
            _ => false,
        }
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

async fn zip_entry(source: &dyn Source, kind: Kind) -> Result<Option<Vec<u8>>, ArchiveError> {
    let directory = central_directory(source).await?;
    let Some(entry) = entries(&directory)
        .into_iter()
        .find(|entry| kind.wants(&entry.name))
    else {
        return Ok(None);
    };
    if entry.uncompressed > MAX_METADATA || entry.compressed > MAX_METADATA {
        return Err(ArchiveError::Invalid("the metadata file is too large"));
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
                .take(MAX_METADATA)
                .read_to_end(&mut inflated)?;
            Ok(Some(inflated))
        }
        _ => Err(ArchiveError::Invalid("unsupported compression")),
    }
}

fn tar_entry(prefix: &[u8]) -> Result<Option<Vec<u8>>, ArchiveError> {
    let inflated = flate2::read::GzDecoder::new(prefix).take(MAX_TAR_INFLATED);
    let mut archive = tar::Archive::new(inflated);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().replace('\\', "/");
        let path = path.trim_start_matches("./");
        if Kind::TarSdist.wants(path) && entry.header().entry_type().is_file() {
            let mut data = Vec::new();
            (&mut entry).take(MAX_METADATA).read_to_end(&mut data)?;
            return Ok(Some(data));
        }
    }
    Ok(None)
}

async fn sniff(source: &dyn Source) -> Result<Option<Kind>, ArchiveError> {
    let head = source.read(0, 4.min(source.size())).await?;
    Ok(match head.as_slice() {
        [b'P', b'K', 3, 4] => Some(Kind::ZipSdist),
        [0x1f, 0x8b, ..] => Some(Kind::TarSdist),
        _ => None,
    })
}

pub async fn metadata(source: &dyn Source, filename: &str) -> Result<Option<String>, ArchiveError> {
    let kind = match Kind::of(filename) {
        Some(kind) => kind,
        None => match sniff(source).await? {
            Some(kind) => kind,
            None => return Ok(None),
        },
    };
    let found = match kind {
        Kind::TarSdist => {
            let prefix = source.read(0, source.size().min(MAX_TAR_PREFIX)).await?;
            tokio::task::spawn_blocking(move || tar_entry(&prefix))
                .await
                .map_err(io::Error::other)?
        }
        Kind::ZipSdist => match zip_entry(source, Kind::ZipSdist).await? {
            Some(found) => Ok(Some(found)),
            None => zip_entry(source, Kind::Wheel).await,
        },
        zip => zip_entry(source, zip).await,
    };
    match found {
        Ok(found) => Ok(found.map(|bytes| String::from_utf8_lossy(&bytes).into_owned())),
        Err(ArchiveError::Io(error)) if kind == Kind::TarSdist => {
            tracing::debug!(%error, filename, "the sdist ended before its PKG-INFO");
            Ok(None)
        }
        Err(error) => Err(error),
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
            out.extend_from_slice(&3u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(b"xyz");
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

    pub fn tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for (name, data) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *data).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    #[tokio::test]
    async fn wheels_give_their_metadata() {
        let wheel = zip(&[
            ("probe/__init__.py", b"VALUE = 1\n", true),
            ("probe-1.0.dist-info/WHEEL", b"Wheel-Version: 1.0\n", false),
            (
                "probe-1.0.dist-info/METADATA",
                b"Metadata-Version: 2.1\nName: probe\nVersion: 1.0\n",
                true,
            ),
        ]);
        let found = metadata(&Memory(wheel.clone()), "probe-1.0-py3-none-any.whl")
            .await
            .unwrap();
        assert_eq!(
            found.as_deref(),
            Some("Metadata-Version: 2.1\nName: probe\nVersion: 1.0\n")
        );
        let stored = zip(&[("probe-1.0.dist-info/METADATA", b"Name: probe\n", false)]);
        assert_eq!(
            metadata(&Memory(stored), "probe-1.0-py3-none-any.whl")
                .await
                .unwrap()
                .as_deref(),
            Some("Name: probe\n")
        );
        let nothing = zip(&[("probe/__init__.py", b"", false)]);
        assert_eq!(metadata(&Memory(nothing), "x.whl").await.unwrap(), None);
        assert!(
            metadata(&Memory(b"not a zip".to_vec()), "x.whl")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn sdists_give_their_pkg_info() {
        let sdist = tar_gz(&[
            ("probe-1.0/setup.py", b"print()\n"),
            ("probe-1.0/src/PKG-INFO", b"wrong level\n"),
            ("probe-1.0/PKG-INFO", b"Name: probe\nVersion: 1.0\n"),
        ]);
        assert_eq!(
            metadata(&Memory(sdist.clone()), "probe-1.0.tar.gz")
                .await
                .unwrap()
                .as_deref(),
            Some("Name: probe\nVersion: 1.0\n")
        );
        assert_eq!(
            metadata(&Memory(sdist), "upload.bin")
                .await
                .unwrap()
                .as_deref(),
            Some("Name: probe\nVersion: 1.0\n")
        );
        let zipped = zip(&[("probe-1.0/PKG-INFO", b"Name: probe\n", true)]);
        assert_eq!(
            metadata(&Memory(zipped), "probe-1.0.zip")
                .await
                .unwrap()
                .as_deref(),
            Some("Name: probe\n")
        );
        assert_eq!(
            metadata(&Memory(b"plain".to_vec()), "notes.txt")
                .await
                .unwrap(),
            None
        );
    }
}
