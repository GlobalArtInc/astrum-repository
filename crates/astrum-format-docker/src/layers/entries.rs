use std::io::{self, Read};

use tar::{Header, PaxExtensions};

use super::limits::Limits;

const BLOCK: usize = 512;
const OLD_EXTENDED: usize = 482;
const EXTENDED: usize = 504;
const OPAQUE: &str = ".wh..wh..opq";
const WHITEOUT: &str = ".wh.";
const RESERVED: &str = ".wh..wh.";
const SPARSE: &str = "GNU.sparse.";
const SPARSE_NAME: &str = "GNU.sparse.name";

const REGULAR: u8 = b'0';
const REGULAR_OLD: u8 = 0;
const LINK: u8 = b'1';
const SYMLINK: u8 = b'2';
const DIRECTORY: u8 = b'5';
const GNU_SPARSE: u8 = b'S';
const LONG_NAME: u8 = b'L';
const LONG_LINK: u8 = b'K';
const PAX_LOCAL: u8 = b'x';
const PAX_GLOBAL: u8 = b'g';
const CONTINUOUS: u8 = b'7';
const CHAR: u8 = b'3';
const BLOCK_DEVICE: u8 = b'4';
const FIFO: u8 = b'6';

pub enum Item<'a> {
    File {
        path: String,
        size: u64,
        body: &'a mut dyn Read,
    },
    Link {
        path: String,
    },
    Node {
        path: String,
    },
    Whiteout {
        path: String,
    },
    Opaque {
        dir: String,
    },
    Sparse {
        path: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Entries,
    Malformed,
    Metadata,
    Pax,
}

#[derive(Default)]
struct Pax {
    path: Option<Vec<u8>>,
    size: Option<u64>,
    sparse: bool,
    sparse_name: Option<Vec<u8>>,
}

#[derive(Default)]
struct Pending {
    long: Option<Vec<u8>>,
    pax: Pax,
}

enum Block {
    End,
    Full,
}

fn read_block(reader: &mut impl Read, block: &mut [u8; BLOCK]) -> io::Result<Block> {
    let mut filled = 0;
    while filled < BLOCK {
        match reader.read(&mut block[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    match filled {
        0 => Ok(Block::End),
        BLOCK => Ok(Block::Full),
        _ => Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "a tar header is cut short",
        )),
    }
}

fn skip(reader: &mut impl Read, length: u64) -> io::Result<()> {
    let copied = io::copy(&mut reader.by_ref().take(length), &mut io::sink())?;
    match copied == length {
        true => Ok(()),
        false => Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "the archive ended inside an entry",
        )),
    }
}

fn padding(size: u64) -> u64 {
    (BLOCK as u64 - size % BLOCK as u64) % BLOCK as u64
}

fn until_nul(bytes: &[u8]) -> &[u8] {
    bytes.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn checksum(block: &[u8; BLOCK]) -> io::Result<()> {
    let (mut unsigned, mut signed) = (0i64, 0i64);
    for (index, byte) in block.iter().enumerate() {
        let byte = if (148..156).contains(&index) {
            b' '
        } else {
            *byte
        };
        unsigned += i64::from(byte);
        signed += i64::from(byte as i8);
    }
    let stored = i64::from(Header::from_byte_slice(block).cksum()?);
    match stored == unsigned || stored == signed {
        true => Ok(()),
        false => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "archive header checksum mismatch",
        )),
    }
}

fn header_name(block: &[u8; BLOCK]) -> Vec<u8> {
    let name = until_nul(&block[..100]);
    let prefix = match &block[257..263] == b"ustar\0" {
        true if &block[508..512] == b"tar\0" => until_nul(&block[345..345 + 131]),
        true => until_nul(&block[345..345 + 155]),
        false => &[],
    };
    match prefix.is_empty() {
        true => name.to_vec(),
        false => [prefix, b"/", name].concat(),
    }
}

fn read_meta(reader: &mut impl Read, size: u64, cap: u64) -> io::Result<Option<Vec<u8>>> {
    if size > cap {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    reader.by_ref().take(size).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != size {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    skip(reader, padding(size))?;
    Ok(Some(bytes))
}

fn parse_pax(bytes: &[u8]) -> Option<Pax> {
    let mut pax = Pax::default();
    for record in PaxExtensions::new(bytes) {
        let record = record.ok()?;
        let key = record.key().ok()?;
        let value = record.value_bytes();
        match key {
            "path" => pax.path = (!value.is_empty()).then(|| value.to_vec()),
            "size" => {
                pax.size = match value.is_empty() {
                    true => None,
                    false => Some(std::str::from_utf8(value).ok()?.parse().ok()?),
                }
            }
            SPARSE_NAME => {
                pax.sparse = true;
                pax.sparse_name = (!value.is_empty()).then(|| value.to_vec());
            }
            other if other.starts_with(SPARSE) => pax.sparse = true,
            _ => {}
        }
    }
    Some(pax)
}

pub fn clean(raw: &[u8], max: usize) -> Option<String> {
    if raw.len() > max {
        return None;
    }
    let text = std::str::from_utf8(raw).ok()?;
    if text.contains('\0') {
        return None;
    }
    let mut parts: Vec<&str> = Vec::new();
    for part in text.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn header_only(flag: u8) -> bool {
    matches!(
        flag,
        LINK | SYMLINK | CHAR | BLOCK_DEVICE | DIRECTORY | FIFO
    )
}

fn skip_extensions(reader: &mut impl Read, limit: u64) -> io::Result<Option<Stop>> {
    let mut block = [0u8; BLOCK];
    for _ in 0..limit {
        if matches!(read_block(reader, &mut block)?, Block::End) {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        if block[EXTENDED] == 0 {
            return Ok(None);
        }
    }
    Ok(Some(Stop::Entries))
}

enum Marker {
    Opaque(String),
    Whiteout(String),
}

fn marker(path: &str) -> Option<Marker> {
    let (dir, base) = path.rsplit_once('/').unwrap_or(("", path));
    if base == OPAQUE {
        return Some(Marker::Opaque(dir.to_owned()));
    }
    let target = base.strip_prefix(WHITEOUT)?;
    if base.starts_with(RESERVED) || target.is_empty() {
        return None;
    }
    Some(Marker::Whiteout(match dir.is_empty() {
        true => target.to_owned(),
        false => format!("{dir}/{target}"),
    }))
}

struct Fields {
    flag: u8,
    size: u64,
    name: Vec<u8>,
    old_extended: bool,
}

fn fields(block: &[u8; BLOCK]) -> io::Result<Fields> {
    checksum(block)?;
    Ok(Fields {
        flag: block[156],
        size: Header::from_byte_slice(block).entry_size()?,
        name: header_name(block),
        old_extended: block[OLD_EXTENDED] > 0,
    })
}

fn resolved_name(entry: &Fields, current: &Pending) -> Vec<u8> {
    let mut name = entry.name.clone();
    if let Some(path) = &current.pax.path {
        name = path.clone();
    }
    if let Some(long) = &current.long {
        let cut = until_nul(long);
        if !cut.is_empty() {
            name = cut.to_vec();
        }
    }
    name
}

pub fn walk<R: Read>(
    mut reader: R,
    limits: &Limits,
    mut each: impl FnMut(Item<'_>) -> io::Result<()>,
) -> io::Result<Option<Stop>> {
    let mut count = 0u64;
    let mut pending = Pending::default();
    let mut block = [0u8; BLOCK];
    loop {
        if matches!(read_block(&mut reader, &mut block)?, Block::End) {
            return Ok(None);
        }
        if block.iter().all(|byte| *byte == 0) {
            return Ok(match read_block(&mut reader, &mut block)? {
                Block::End => None,
                Block::Full if block.iter().all(|byte| *byte == 0) => None,
                Block::Full => Some(Stop::Malformed),
            });
        }
        count += 1;
        if count > limits.max_entries {
            return Ok(Some(Stop::Entries));
        }
        let entry = fields(&block)?;
        match entry.flag {
            LONG_NAME => {
                match read_meta(&mut reader, entry.size, limits.max_meta)? {
                    Some(bytes) => pending.long = Some(bytes),
                    None => return Ok(Some(Stop::Metadata)),
                }
                continue;
            }
            PAX_LOCAL => {
                let Some(bytes) = read_meta(&mut reader, entry.size, limits.max_meta)? else {
                    return Ok(Some(Stop::Metadata));
                };
                let Some(pax) = parse_pax(&bytes) else {
                    return Ok(Some(Stop::Pax));
                };
                pending.pax = pax;
                continue;
            }
            PAX_GLOBAL => {
                skip(&mut reader, entry.size)?;
                skip(&mut reader, padding(entry.size))?;
                pending = Pending::default();
                continue;
            }
            LONG_LINK => {
                skip(&mut reader, entry.size)?;
                skip(&mut reader, padding(entry.size))?;
                continue;
            }
            _ => {}
        }
        let current = std::mem::take(&mut pending);
        let named = resolved_name(&entry, &current);
        let path = clean(&named, limits.max_name);
        let directory =
            entry.flag == DIRECTORY || (entry.flag == REGULAR_OLD && named.ends_with(b"/"));
        let only = header_only(entry.flag) || directory;
        let size = match only {
            true => 0,
            false => current.pax.size.unwrap_or(entry.size),
        };
        if entry.flag == GNU_SPARSE
            && entry.old_extended
            && let Some(stop) = skip_extensions(&mut reader, limits.max_entries)?
        {
            return Ok(Some(stop));
        }
        let sparse = entry.flag == GNU_SPARSE || current.pax.sparse;
        let kind = path.as_deref().and_then(marker);
        match (path, kind) {
            (Some(_), Some(Marker::Opaque(dir))) => each(Item::Opaque { dir })?,
            (Some(_), Some(Marker::Whiteout(path))) => each(Item::Whiteout { path })?,
            (Some(path), None) if sparse => {
                each(Item::Sparse { path })?;
                let renamed = current
                    .pax
                    .sparse_name
                    .as_deref()
                    .and_then(|name| clean(name, limits.max_name));
                if let Some(path) = renamed {
                    each(Item::Sparse { path })?;
                }
            }
            (Some(_), None) if directory => {}
            (Some(path), None) if matches!(entry.flag, LINK | SYMLINK) => {
                each(Item::Link { path })?
            }
            (Some(path), None) if matches!(entry.flag, CHAR | BLOCK_DEVICE | FIFO) => {
                each(Item::Node { path })?
            }
            (Some(path), None) if matches!(entry.flag, REGULAR | REGULAR_OLD | CONTINUOUS) => {
                let mut body = reader.by_ref().take(size);
                each(Item::File {
                    path,
                    size,
                    body: &mut body,
                })?;
                let rest = body.limit();
                skip(&mut reader, rest)?;
                skip(&mut reader, padding(size))?;
                continue;
            }
            (Some(path), None) => each(Item::Node { path })?,
            (None, _) => {}
        }
        if !only {
            skip(&mut reader, size)?;
            skip(&mut reader, padding(size))?;
        }
    }
}

#[cfg(test)]
mod tests;
