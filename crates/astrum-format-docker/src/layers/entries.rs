use std::io::{self, Read};

use tar::{EntryType, Header, PaxExtensions};

use super::limits::Limits;

const BLOCK: usize = 512;
const EXTENDED_FLAG: usize = 504;
const OPAQUE: &str = ".wh..wh..opq";
const WHITEOUT: &str = ".wh.";
const RESERVED: &str = ".wh..wh.";
const SPARSE: &str = "GNU.sparse.";

pub enum Item<'a> {
    File {
        path: String,
        size: u64,
        body: &'a mut dyn Read,
    },
    Whiteout {
        path: String,
    },
    Opaque {
        dir: String,
    },
    Link {
        path: String,
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
}

#[derive(Default)]
struct Pending {
    long: Option<Vec<u8>>,
    pax_path: Option<Vec<u8>>,
    pax_size: Option<u64>,
    sparse: bool,
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

fn checksum(block: &[u8; BLOCK], header: &Header) -> io::Result<()> {
    let sum = block[..148]
        .iter()
        .chain(&block[156..])
        .map(|byte| u32::from(*byte))
        .sum::<u32>()
        + 8 * 32;
    match header.cksum()? == sum {
        true => Ok(()),
        false => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "archive header checksum mismatch",
        )),
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

fn read_pax(bytes: &[u8], pending: &mut Pending) {
    for record in PaxExtensions::new(bytes).flatten() {
        let Ok(key) = record.key() else {
            continue;
        };
        match key {
            "path" => pending.pax_path = Some(record.value_bytes().to_vec()),
            "size" => {
                pending.pax_size = Some(
                    record
                        .value()
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(u64::MAX),
                )
            }
            other if other.starts_with(SPARSE) => pending.sparse = true,
            _ => {}
        }
    }
}

pub fn normalise(raw: &[u8], max: usize) -> Option<String> {
    if raw.len() > max {
        return None;
    }
    let text = std::str::from_utf8(raw).ok()?;
    if text.contains('\0') {
        return None;
    }
    let mut parts = Vec::new();
    for part in text.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            other => parts.push(other),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn classify(path: String, size: u64, body: &mut dyn Read) -> Item<'_> {
    let (dir, base) = path.rsplit_once('/').unwrap_or(("", path.as_str()));
    if base == OPAQUE {
        return Item::Opaque {
            dir: dir.to_owned(),
        };
    }
    if let Some(target) = base.strip_prefix(WHITEOUT)
        && !base.starts_with(RESERVED)
        && !target.is_empty()
    {
        let removed = match dir.is_empty() {
            true => target.to_owned(),
            false => format!("{dir}/{target}"),
        };
        return Item::Whiteout { path: removed };
    }
    Item::File { path, size, body }
}

struct Fields {
    kind: EntryType,
    plain_zero: bool,
    size: u64,
    name: Vec<u8>,
    recognised: bool,
    extended: bool,
}

fn fields(block: &[u8; BLOCK]) -> io::Result<Fields> {
    let header = Header::from_byte_slice(block);
    checksum(block, header)?;
    Ok(Fields {
        kind: header.entry_type(),
        plain_zero: block[156] == 0,
        size: header.entry_size()?,
        name: header.path_bytes().into_owned(),
        recognised: header.as_gnu().is_some() || header.as_ustar().is_some(),
        extended: header.as_gnu().is_some_and(|gnu| gnu.is_extended()),
    })
}

fn header_only(kind: EntryType) -> bool {
    matches!(
        kind,
        EntryType::Link
            | EntryType::Symlink
            | EntryType::Char
            | EntryType::Block
            | EntryType::Directory
            | EntryType::Fifo
    )
}

fn skip_extensions(reader: &mut impl Read, limit: u64) -> io::Result<Option<Stop>> {
    let mut block = [0u8; BLOCK];
    for _ in 0..limit {
        if matches!(read_block(reader, &mut block)?, Block::End) {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        if block[EXTENDED_FLAG] != 1 {
            return Ok(None);
        }
    }
    Ok(Some(Stop::Entries))
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
        if entry.recognised && entry.kind.is_gnu_longname() {
            match read_meta(&mut reader, entry.size, limits.max_meta)? {
                Some(bytes) => pending.long = Some(bytes),
                None => return Ok(Some(Stop::Metadata)),
            }
            continue;
        }
        if entry.recognised && entry.kind.is_pax_local_extensions() {
            match read_meta(&mut reader, entry.size, limits.max_meta)? {
                Some(bytes) => read_pax(&bytes, &mut pending),
                None => return Ok(Some(Stop::Metadata)),
            }
            continue;
        }
        if entry.recognised
            && (entry.kind.is_gnu_longlink() || entry.kind.is_pax_global_extensions())
        {
            skip(&mut reader, entry.size)?;
            skip(&mut reader, padding(entry.size))?;
            continue;
        }
        let current = std::mem::take(&mut pending);
        let named = match (&current.long, &current.pax_path) {
            (Some(long), _) => long.strip_suffix(&[0]).unwrap_or(long).to_vec(),
            (None, Some(path)) => path.clone(),
            (None, None) => entry.name.clone(),
        };
        let path = normalise(&named, limits.max_name);
        if header_only(entry.kind) {
            if let Some(path) = path
                && matches!(entry.kind, EntryType::Link | EntryType::Symlink)
            {
                each(Item::Link { path })?;
            }
            continue;
        }
        let size = current.pax_size.unwrap_or(entry.size);
        if entry.kind == EntryType::GNUSparse
            && entry.extended
            && let Some(stop) = skip_extensions(&mut reader, limits.max_entries)?
        {
            return Ok(Some(stop));
        }
        let sparse = entry.kind == EntryType::GNUSparse || current.sparse;
        let directory = entry.plain_zero && named.ends_with(b"/");
        let regular = matches!(entry.kind, EntryType::Regular | EntryType::Continuous);
        match path {
            Some(path) if sparse => each(Item::Sparse { path })?,
            Some(path) if regular && !directory => {
                let mut body = reader.by_ref().take(size);
                each(classify(path, size, &mut body))?;
                let rest = body.limit();
                skip(&mut reader, rest)?;
                skip(&mut reader, padding(size))?;
                continue;
            }
            _ => {}
        }
        skip(&mut reader, size)?;
        skip(&mut reader, padding(size))?;
    }
}
