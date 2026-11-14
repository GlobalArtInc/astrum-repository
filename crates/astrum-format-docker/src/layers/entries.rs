use std::borrow::Cow;
use std::io::{self, Read};

use tar::{Archive, EntryType, PaxExtensions};

use super::limits::Limits;

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
    Link,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Entries,
    PaxSize,
}

#[derive(Default)]
struct Pending {
    long: Option<Vec<u8>>,
    pax_path: Option<Vec<u8>>,
    pax_size: Option<u64>,
    sparse: bool,
    unusable: bool,
}

fn meta(entry: &mut dyn Read, size: u64, cap: u64) -> io::Result<Option<Vec<u8>>> {
    if size > cap {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    entry.take(size).read_to_end(&mut bytes)?;
    match bytes.len() as u64 == size {
        true => Ok(Some(bytes)),
        false => Err(io::ErrorKind::UnexpectedEof.into()),
    }
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

fn name_of<'a>(pending: &'a Pending, header: Cow<'a, [u8]>) -> Option<Cow<'a, [u8]>> {
    if pending.unusable {
        return None;
    }
    if let Some(long) = &pending.long {
        let trimmed = long.strip_suffix(&[0]).unwrap_or(long);
        return Some(Cow::Borrowed(trimmed));
    }
    if let Some(path) = &pending.pax_path {
        return Some(Cow::Borrowed(path));
    }
    Some(header)
}

fn classify<'a>(path: String, size: u64, body: &'a mut dyn Read) -> Option<Item<'a>> {
    let (dir, base) = path.rsplit_once('/').unwrap_or(("", path.as_str()));
    if base == OPAQUE {
        return Some(Item::Opaque {
            dir: dir.to_owned(),
        });
    }
    if base.starts_with(RESERVED) {
        return None;
    }
    if let Some(target) = base.strip_prefix(WHITEOUT) {
        if target.is_empty() {
            return None;
        }
        let removed = match dir.is_empty() {
            true => target.to_owned(),
            false => format!("{dir}/{target}"),
        };
        return Some(Item::Whiteout { path: removed });
    }
    Some(Item::File { path, size, body })
}

pub fn walk<R: Read>(
    reader: R,
    limits: &Limits,
    mut each: impl FnMut(Item<'_>) -> io::Result<()>,
) -> io::Result<Option<Stop>> {
    let mut archive = Archive::new(reader);
    let mut count = 0u64;
    let mut pending = Pending::default();
    for entry in archive.entries()?.raw(true) {
        let mut entry = entry?;
        count += 1;
        if count > limits.max_entries {
            return Ok(Some(Stop::Entries));
        }
        let kind = entry.header().entry_type();
        let size = entry.header().entry_size()?;
        let recognised = entry.header().as_gnu().is_some() || entry.header().as_ustar().is_some();
        if recognised && kind.is_gnu_longname() {
            match meta(&mut entry, size, limits.max_meta)? {
                Some(bytes) => pending.long = Some(bytes),
                None => pending.unusable = true,
            }
            continue;
        }
        if recognised && kind.is_pax_local_extensions() {
            match meta(&mut entry, size, limits.max_meta)? {
                Some(bytes) => read_pax(&bytes, &mut pending),
                None => pending.unusable = true,
            }
            continue;
        }
        if recognised && (kind.is_gnu_longlink() || kind.is_pax_global_extensions()) {
            continue;
        }
        let current = std::mem::take(&mut pending);
        if current.pax_size.is_some_and(|pax| pax != size) {
            return Ok(Some(Stop::PaxSize));
        }
        let regular = matches!(kind, EntryType::Regular | EntryType::Continuous);
        let link = matches!(kind, EntryType::Link | EntryType::Symlink);
        if !(regular || link) || current.sparse {
            continue;
        }
        let name = {
            let header_name = entry.path_bytes();
            name_of(&current, header_name).and_then(|raw| normalise(&raw, limits.max_name))
        };
        let Some(path) = name else {
            continue;
        };
        if link {
            each(Item::Link)?;
            continue;
        }
        if let Some(item) = classify(path, size, &mut entry) {
            each(item)?;
        }
    }
    Ok(None)
}
