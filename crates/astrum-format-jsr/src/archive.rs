use std::cell::Cell;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::rc::Rc;

use flate2::bufread::GzDecoder;
use sha2::{Digest, Sha256};
use tar::EntryType;

pub const MAX_COMPRESSED: usize = 20 * 1024 * 1024;
pub const MAX_EXPANDED: u64 = 100 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 2000;
pub const MAX_PATH: usize = 512;

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
const SHOWN: usize = 80;

#[derive(Debug, PartialEq, Eq)]
pub enum ArchiveError {
    Invalid(String),
    TooLarge(String),
    Storage(String),
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

fn shown(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .take(SHOWN)
        .collect()
}

#[derive(Debug)]
pub struct Unpacked {
    pub files: Vec<Entry>,
    spool: File,
}

#[derive(Debug)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
    offset: u64,
}

impl Unpacked {
    pub fn find(&self, path: &str) -> Option<&Entry> {
        self.files.iter().find(|entry| entry.path == path)
    }

    pub fn open(&self, entry: &Entry) -> io::Result<File> {
        let mut file = self.spool.try_clone()?;
        file.seek(SeekFrom::Start(entry.offset))?;
        Ok(file)
    }

    pub fn read_all(&self, entry: &Entry, limit: u64) -> io::Result<Option<Vec<u8>>> {
        if entry.size > limit {
            return Ok(None);
        }
        let mut bytes = Vec::with_capacity(entry.size as usize);
        self.open(entry)?.take(entry.size).read_to_end(&mut bytes)?;
        Ok(Some(bytes))
    }
}

struct Capped<R> {
    inner: R,
    seen: u64,
    limit: u64,
    exceeded: Rc<Cell<bool>>,
}

impl<R: Read> Read for Capped<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let count = self.inner.read(buf)?;
        self.seen += count as u64;
        if self.seen > self.limit {
            self.exceeded.set(true);
            return Err(io::Error::other("the archive expands beyond its limit"));
        }
        Ok(count)
    }
}

struct Hashing<'a> {
    file: &'a mut File,
    hash: Sha256,
    size: u64,
    failed: bool,
}

impl Write for Hashing<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.file.write(buf).inspect_err(|_| self.failed = true)?;
        self.hash.update(&buf[..written]);
        self.size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

pub fn clean_path(raw: &[u8]) -> Result<String, ArchiveError> {
    if raw.len() > MAX_PATH {
        return Err(invalid(format!(
            "the path {}... is longer than {MAX_PATH} bytes",
            shown(raw)
        )));
    }
    let text = std::str::from_utf8(raw)
        .map_err(|_| invalid(format!("the path {} is not UTF-8", shown(raw))))?;
    if text.contains('\0') || text.contains('\\') || text.chars().any(char::is_control) {
        return Err(invalid(format!(
            "the path {} holds a NUL, backslash or control character",
            shown(raw)
        )));
    }
    let stripped = text
        .strip_prefix('/')
        .or_else(|| text.strip_prefix("./"))
        .unwrap_or(text);
    if stripped.is_empty() {
        return Err(invalid("an entry has an empty path"));
    }
    for segment in stripped.split('/') {
        let bytes = segment.as_bytes();
        if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            return Err(invalid(format!(
                "the path {} has a segment that names a drive",
                shown(text.as_bytes())
            )));
        }
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(invalid(format!(
                "the path {} is absolute, empty-segmented or leaves the package",
                shown(text.as_bytes())
            )));
        }
    }
    Ok(stripped.to_owned())
}

fn kind_name(kind: EntryType) -> &'static str {
    match kind {
        EntryType::Symlink => "a symbolic link",
        EntryType::Link => "a hard link",
        EntryType::Char => "a character device",
        EntryType::Block => "a block device",
        EntryType::Fifo => "a fifo",
        EntryType::GNUSparse => "a sparse file",
        _ => "not a regular file",
    }
}

fn read_failure(error: io::Error, exceeded: &Cell<bool>) -> ArchiveError {
    if exceeded.get() {
        return ArchiveError::TooLarge(format!(
            "the archive expands to more than {MAX_EXPANDED} bytes"
        ));
    }
    invalid(format!("the archive cannot be read: {error}"))
}

pub fn unpack(compressed: &[u8], spool_dir: Option<&Path>) -> Result<Unpacked, ArchiveError> {
    if compressed.len() > MAX_COMPRESSED {
        return Err(ArchiveError::TooLarge(format!(
            "the archive is larger than {MAX_COMPRESSED} bytes"
        )));
    }
    if !compressed.starts_with(&GZIP_MAGIC) {
        return Err(invalid("the body is not a gzip-compressed tar archive"));
    }
    let exceeded = Rc::new(Cell::new(false));
    let capped = Capped {
        inner: GzDecoder::new(compressed),
        seen: 0,
        limit: MAX_EXPANDED,
        exceeded: exceeded.clone(),
    };
    let mut archive = tar::Archive::new(capped);
    let entries = archive
        .entries()
        .map_err(|error| read_failure(error, &exceeded))?;
    let mut spool = match spool_dir {
        Some(dir) => tempfile::tempfile_in(dir),
        None => tempfile::tempfile(),
    }
    .map_err(|error| ArchiveError::Storage(format!("the archive cannot be unpacked: {error}")))?;
    let mut files: Vec<Entry> = Vec::new();
    let mut names: HashSet<String> = HashSet::new();
    let mut count = 0usize;
    let mut total = 0u64;
    for entry in entries {
        let mut entry = entry.map_err(|error| read_failure(error, &exceeded))?;
        count += 1;
        if count > MAX_ENTRIES {
            return Err(invalid(format!(
                "the archive has more than {MAX_ENTRIES} entries"
            )));
        }
        let kind = entry.header().entry_type();
        match kind {
            EntryType::Directory | EntryType::XGlobalHeader => continue,
            EntryType::Regular => {}
            other => {
                return Err(invalid(format!(
                    "the entry {} is {}; only regular files are accepted",
                    shown(&entry.path_bytes()),
                    kind_name(other)
                )));
            }
        }
        let path = clean_path(&entry.path_bytes())?;
        if !names.insert(path.clone()) {
            return Err(invalid(format!("the archive lists {path} twice")));
        }
        let declared = entry.size();
        if total.saturating_add(declared) > MAX_EXPANDED {
            return Err(ArchiveError::TooLarge(format!(
                "the archive expands to more than {MAX_EXPANDED} bytes"
            )));
        }
        let mut sink = Hashing {
            file: &mut spool,
            hash: Sha256::new(),
            size: 0,
            failed: false,
        };
        if let Err(error) = io::copy(&mut entry, &mut sink) {
            return Err(match sink.failed {
                true => ArchiveError::Storage(format!("the archive cannot be unpacked: {error}")),
                false => read_failure(error, &exceeded),
            });
        }
        let (size, digest) = (sink.size, sink.hash.finalize());
        if size != declared {
            return Err(invalid(format!("the entry {path} is cut short")));
        }
        files.push(Entry {
            path,
            size,
            sha256: hex::encode(digest),
            offset: total,
        });
        total += size;
    }
    let mut capped = archive.into_inner();
    io::copy(&mut capped, &mut io::sink()).map_err(|error| read_failure(error, &exceeded))?;
    if !capped.inner.get_ref().is_empty() {
        return Err(invalid(
            "the body holds more than one gzip stream or data after the archive",
        ));
    }
    Ok(Unpacked { files, spool })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;

    fn header(path: &[u8], kind: EntryType, size: u64) -> tar::Header {
        let mut header = tar::Header::new_gnu();
        header.as_old_mut().name[..path.len()].copy_from_slice(path);
        header.set_entry_type(kind);
        header.set_size(size);
        header.set_mode(0o644);
        header.set_cksum();
        header
    }

    fn pack(build: impl FnOnce(&mut tar::Builder<Vec<u8>>)) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        build(&mut builder);
        gzip(&builder.into_inner().unwrap())
    }

    fn gzip(raw: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(raw).unwrap();
        encoder.finish().unwrap()
    }

    fn regular(builder: &mut tar::Builder<Vec<u8>>, path: &[u8], data: &[u8]) {
        builder
            .append(&header(path, EntryType::Regular, data.len() as u64), data)
            .unwrap();
    }

    fn refused(archive: &[u8]) -> ArchiveError {
        unpack(archive, None).unwrap_err()
    }

    fn message(error: ArchiveError) -> String {
        match error {
            ArchiveError::Invalid(message)
            | ArchiveError::TooLarge(message)
            | ArchiveError::Storage(message) => message,
        }
    }

    #[test]
    fn regular_files_are_read_with_their_digests_and_directories_are_skipped() {
        let archive = pack(|builder| {
            builder
                .append(&header(b"sub/", EntryType::Directory, 0), &[][..])
                .unwrap();
            regular(builder, b"/mod.ts", b"export const a = 1;\n");
            regular(builder, b"./sub/b.ts", b"");
        });
        let unpacked = unpack(&archive, None).unwrap();
        let paths: Vec<&str> = unpacked.files.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["mod.ts", "sub/b.ts"]);
        let first = unpacked.find("mod.ts").unwrap();
        assert_eq!(first.size, 20);
        assert_eq!(
            first.sha256,
            hex::encode(Sha256::digest(b"export const a = 1;\n"))
        );
        assert_eq!(
            unpacked.read_all(first, 100).unwrap().unwrap(),
            b"export const a = 1;\n"
        );
        assert_eq!(unpacked.read_all(first, 5).unwrap(), None);
        assert_eq!(
            unpacked
                .read_all(unpacked.find("sub/b.ts").unwrap(), 10)
                .unwrap(),
            Some(Vec::new())
        );
        let later = unpacked.find("sub/b.ts").unwrap();
        assert_eq!(later.offset, 20);
    }

    #[test]
    fn links_devices_and_fifos_are_refused() {
        for kind in [
            EntryType::Symlink,
            EntryType::Link,
            EntryType::Char,
            EntryType::Block,
            EntryType::Fifo,
        ] {
            let archive = pack(|builder| {
                let mut link = header(b"passwd", kind, 0);
                if matches!(kind, EntryType::Symlink | EntryType::Link) {
                    link.set_link_name("/etc/passwd").unwrap();
                    link.set_cksum();
                }
                builder.append(&link, &[][..]).unwrap();
            });
            let text = message(refused(&archive));
            assert!(text.contains("only regular files"), "{kind:?}: {text}");
        }
    }

    #[test]
    fn paths_that_leave_the_package_are_refused() {
        let bad: [&[u8]; 12] = [
            b"../x.ts",
            b"a/../../x.ts",
            b"a/./b.ts",
            b"//etc/passwd",
            b"a//b.ts",
            b"C:/x.ts",
            b"sub/c:x.ts",
            b"a\\b.ts",
            b"a/b/",
            b"/",
            b"./",
            b"\xff\xfe.ts",
        ];
        for path in bad {
            let archive = pack(|builder| regular(builder, path, b"x"));
            assert!(
                matches!(refused(&archive), ArchiveError::Invalid(_)),
                "{}",
                String::from_utf8_lossy(path)
            );
        }
        let long = "a".repeat(MAX_PATH + 1);
        let archive = pack(|builder| {
            let mut entry = tar::Header::new_gnu();
            entry.set_size(1);
            entry.set_entry_type(EntryType::Regular);
            builder.append_data(&mut entry, &long, &b"x"[..]).unwrap();
        });
        assert!(message(refused(&archive)).contains("longer than"));
    }

    #[test]
    fn a_path_with_a_nul_is_refused() {
        assert!(clean_path(b"a\0b.ts").is_err());
        assert_eq!(clean_path(b"/a/b.ts").unwrap(), "a/b.ts");
        assert_eq!(clean_path(b"./a.ts").unwrap(), "a.ts");
        assert!(clean_path(b"/./a.ts").is_err());
        assert_eq!(clean_path(b"ab:c.ts").unwrap(), "ab:c.ts");
        assert!(clean_path(b"a:b.ts").is_err());
    }

    #[test]
    fn duplicate_entries_are_refused() {
        let archive = pack(|builder| {
            regular(builder, b"mod.ts", b"1");
            regular(builder, b"/mod.ts", b"2");
        });
        assert!(message(refused(&archive)).contains("twice"));
    }

    #[test]
    fn the_entry_count_is_capped() {
        let build = |count: usize| {
            pack(|builder| {
                for number in 0..count {
                    regular(builder, format!("f{number}.ts").as_bytes(), b"");
                }
            })
        };
        assert_eq!(
            unpack(&build(MAX_ENTRIES), None).unwrap().files.len(),
            MAX_ENTRIES
        );
        assert!(message(refused(&build(MAX_ENTRIES + 1))).contains("more than 2000 entries"));
    }

    #[test]
    fn a_decompression_bomb_stops_at_the_expanded_limit() {
        let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
        let mut entry = header(b"zeros.bin", EntryType::Regular, 0);
        entry.set_size(MAX_EXPANDED);
        entry.set_cksum();
        builder
            .append(&entry, std::io::repeat(0).take(MAX_EXPANDED))
            .unwrap();
        let mut second = header(b"more.bin", EntryType::Regular, 0);
        second.set_size(1024);
        second.set_cksum();
        builder
            .append(&second, std::io::repeat(0).take(1024))
            .unwrap();
        let archive = builder.into_inner().unwrap().finish().unwrap();
        assert!(archive.len() < MAX_COMPRESSED);
        assert!(matches!(refused(&archive), ArchiveError::TooLarge(_)));
    }

    #[test]
    fn a_header_that_promises_more_than_the_limit_is_refused_before_reading() {
        let mut raw = Vec::new();
        let mut entry = header(b"big.bin", EntryType::Regular, 0);
        entry.set_size(MAX_EXPANDED + 1);
        entry.set_cksum();
        raw.extend_from_slice(entry.as_bytes());
        let archive = gzip(&raw);
        assert!(matches!(refused(&archive), ArchiveError::TooLarge(_)));
    }

    #[test]
    fn the_compressed_size_is_capped_and_the_body_must_be_gzip() {
        let big = vec![0u8; MAX_COMPRESSED + 1];
        assert!(matches!(refused(&big), ArchiveError::TooLarge(_)));
        assert!(matches!(
            refused(b"not an archive"),
            ArchiveError::Invalid(_)
        ));
        let truncated = {
            let archive = pack(|builder| regular(builder, b"mod.ts", &[7u8; 4096]));
            archive[..archive.len() / 2].to_vec()
        };
        assert!(matches!(refused(&truncated), ArchiveError::Invalid(_)));
    }

    fn huge_extension_archive(kind: EntryType) -> Vec<u8> {
        let size = MAX_EXPANDED + 1024 * 1024;
        let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
        let mut extension = header(b"extension", kind, 0);
        extension.set_size(size);
        extension.set_cksum();
        builder
            .append(&extension, std::io::repeat(0).take(size))
            .unwrap();
        let file = header(b"mod.ts", EntryType::Regular, 1);
        builder.append(&file, &b"x"[..]).unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn a_huge_extension_header_with_a_small_file_entry_stops_at_the_expanded_limit() {
        for kind in [EntryType::XHeader, EntryType::GNULongName] {
            let archive = huge_extension_archive(kind);
            assert!(archive.len() < MAX_COMPRESSED);
            assert!(
                matches!(refused(&archive), ArchiveError::TooLarge(_)),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn a_second_gzip_stream_or_trailing_data_is_refused() {
        let first = pack(|builder| regular(builder, b"mod.ts", b"1"));
        let second = pack(|builder| regular(builder, b"other.ts", b"2"));
        let mut joined = first.clone();
        joined.extend_from_slice(&second);
        assert!(message(refused(&joined)).contains("more than one gzip stream"));
        let mut padded = first.clone();
        padded.extend_from_slice(b"trailing");
        assert!(matches!(refused(&padded), ArchiveError::Invalid(_)));
        assert!(unpack(&first, None).is_ok());
    }
}
