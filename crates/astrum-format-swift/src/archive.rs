use std::collections::BTreeMap;
use std::io::Read;

const MAX_MANIFEST: usize = 1024 * 1024;
const MAC_RESOURCES: &str = "__MACOSX";

pub const MANIFEST: &str = "Package.swift";

pub fn is_manifest(file: &str) -> bool {
    file == MANIFEST || variant_version(file).is_some()
}

pub fn variant_version(file: &str) -> Option<&str> {
    let version = file
        .strip_prefix("Package@swift-")?
        .strip_suffix(".swift")?;
    swift_version(version).then_some(version)
}

pub fn swift_version(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    (1..=3).contains(&parts.len())
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= 9 && part.bytes().all(|c| c.is_ascii_digit())
        })
}

pub fn tools_version(manifest: &str) -> Option<String> {
    let first = manifest.trim_start_matches('\u{feff}').lines().next()?;
    let rest = first.trim_start().strip_prefix("//")?.trim_start();
    let lower = rest.to_ascii_lowercase();
    let rest = &rest[lower.find("swift-tools-version")? + "swift-tools-version".len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let version: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let version = version.trim_end_matches('.').to_owned();
    (!version.is_empty()).then_some(version)
}

fn parts(version: &str) -> Vec<u64> {
    let mut parts: Vec<u64> = version
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect();
    parts.resize(3, 0);
    parts
}

pub fn resolve<'a>(variants: impl IntoIterator<Item = &'a String>, swift: &str) -> Option<String> {
    if !swift_version(swift) {
        return None;
    }
    let wanted = parts(swift);
    variants
        .into_iter()
        .find(|file| variant_version(file).is_some_and(|version| parts(version) == wanted))
        .cloned()
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Source {
    pub manifests: BTreeMap<String, String>,
}

impl Source {
    pub fn tools_versions(&self) -> BTreeMap<String, String> {
        self.manifests
            .iter()
            .map(|(file, content)| {
                let version = tools_version(content)
                    .or_else(|| variant_version(file).map(str::to_owned))
                    .unwrap_or_default();
                (file.clone(), version)
            })
            .collect()
    }
}

fn u16_at(data: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?) as usize)
}

fn u32_at(data: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?) as usize)
}

struct Entry {
    name: String,
    method: usize,
    compressed: usize,
    local: usize,
}

fn entries(data: &[u8]) -> Result<Vec<Entry>, String> {
    let bad = || "the source archive is not a zip archive".to_owned();
    if data.len() < 22 {
        return Err(bad());
    }
    let search = data.len().saturating_sub(65_557);
    let end = (search..=data.len() - 22)
        .rev()
        .find(|&at| data[at..at + 4] == [0x50, 0x4b, 0x05, 0x06])
        .ok_or_else(bad)?;
    let count = u16_at(data, end + 10).ok_or_else(bad)?;
    let mut at = u32_at(data, end + 16).ok_or_else(bad)?;
    let mut found = Vec::with_capacity(count);
    for _ in 0..count {
        if data.get(at..at + 4) != Some(&[0x50, 0x4b, 0x01, 0x02][..]) {
            return Err(bad());
        }
        let method = u16_at(data, at + 10).ok_or_else(bad)?;
        let compressed = u32_at(data, at + 20).ok_or_else(bad)?;
        let name_length = u16_at(data, at + 28).ok_or_else(bad)?;
        let extra = u16_at(data, at + 30).ok_or_else(bad)?;
        let comment = u16_at(data, at + 32).ok_or_else(bad)?;
        let local = u32_at(data, at + 42).ok_or_else(bad)?;
        let name =
            String::from_utf8_lossy(data.get(at + 46..at + 46 + name_length).ok_or_else(bad)?)
                .replace('\\', "/");
        found.push(Entry {
            name,
            method,
            compressed,
            local,
        });
        at += 46 + name_length + extra + comment;
    }
    Ok(found)
}

fn content(data: &[u8], entry: &Entry) -> Result<String, String> {
    let bad = || format!("{} cannot be read from the source archive", entry.name);
    if data.get(entry.local..entry.local + 4) != Some(&[0x50, 0x4b, 0x03, 0x04][..]) {
        return Err(bad());
    }
    let name_length = u16_at(data, entry.local + 26).ok_or_else(bad)?;
    let extra = u16_at(data, entry.local + 28).ok_or_else(bad)?;
    let start = entry.local + 30 + name_length + extra;
    let raw = data.get(start..start + entry.compressed).ok_or_else(bad)?;
    let mut text = Vec::new();
    match entry.method {
        0 => text.extend_from_slice(&raw[..raw.len().min(MAX_MANIFEST + 1)]),
        8 => {
            flate2::read::DeflateDecoder::new(raw)
                .take(MAX_MANIFEST as u64 + 1)
                .read_to_end(&mut text)
                .map_err(|_| bad())?;
        }
        _ => {
            return Err(format!(
                "{} uses a compression the server does not read",
                entry.name
            ));
        }
    }
    if text.len() > MAX_MANIFEST {
        return Err(format!(
            "{} is larger than {MAX_MANIFEST} bytes",
            entry.name
        ));
    }
    String::from_utf8(text).map_err(|_| format!("{} is not UTF-8 text", entry.name))
}

pub fn read(data: &[u8]) -> Result<Source, String> {
    let entries = entries(data)?;
    let mut roots = std::collections::BTreeSet::new();
    let mut candidates: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for entry in &entries {
        let name = entry.name.trim_start_matches('/');
        if name.is_empty()
            || name == MAC_RESOURCES
            || name.starts_with(&format!("{MAC_RESOURCES}/"))
        {
            continue;
        }
        if name == ".." || name.contains("../") {
            return Err(format!(
                "source archive contains an entry outside of the package: {name}"
            ));
        }
        let Some((first, rest)) = name.split_once('/') else {
            roots.insert(match name.ends_with('/') {
                true => name.trim_end_matches('/').to_owned(),
                false => String::new(),
            });
            continue;
        };
        roots.insert(first.to_owned());
        if !name.ends_with('/') && is_manifest(rest) {
            candidates.entry(first.to_owned()).or_default().push(entry);
        }
    }
    if roots.is_empty() {
        return Err("the source archive is empty".to_owned());
    }
    let mut source = Source::default();
    for (root, found) in candidates {
        if !found
            .iter()
            .any(|entry| entry.name.ends_with(&format!("/{MANIFEST}")))
        {
            continue;
        }
        for entry in found {
            let file = entry.name[root.len() + 1..].to_owned();
            source.manifests.insert(file, content(data, entry)?);
        }
        return Ok(source);
    }
    Err("the source archive has no Package.swift in its top directory".to_owned())
}

#[cfg(test)]
pub mod testing {
    pub fn zip(files: &[(&str, &str)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, text) in files {
            let offset = out.len() as u32;
            let crc = crc32(text.as_bytes());
            let header = |signature: &[u8]| {
                let mut part = signature.to_vec();
                part.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                part.extend_from_slice(&crc.to_le_bytes());
                part.extend_from_slice(&(text.len() as u32).to_le_bytes());
                part.extend_from_slice(&(text.len() as u32).to_le_bytes());
                part.extend_from_slice(&(name.len() as u16).to_le_bytes());
                part.extend_from_slice(&[0, 0]);
                part
            };
            out.extend_from_slice(&header(&[0x50, 0x4b, 0x03, 0x04]));
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(text.as_bytes());
            let mut entry = vec![0x50, 0x4b, 0x01, 0x02, 20, 0];
            entry.extend_from_slice(&header(&[])[..]);
            entry.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            entry.extend_from_slice(&offset.to_le_bytes());
            entry.extend_from_slice(name.as_bytes());
            central.extend_from_slice(&entry);
        }
        let start = out.len() as u32;
        out.extend_from_slice(&central);
        out.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(central.len() as u32).to_le_bytes());
        out.extend_from_slice(&start.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for byte in data {
            crc ^= *byte as u32;
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    0xedb8_8320 ^ (crc >> 1)
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifests_are_read_from_the_top_directory() {
        let data = testing::zip(&[
            (
                "LinkedList/Package.swift",
                "// swift-tools-version:5.7\nimport PackageDescription\n",
            ),
            (
                "LinkedList/Package@swift-5.9.swift",
                "// swift-tools-version: 5.9\n",
            ),
            (
                "LinkedList/Sources/LinkedList/List.swift",
                "struct List {}\n",
            ),
        ]);
        let source = read(&data).unwrap();
        assert_eq!(source.manifests.len(), 2);
        let versions = source.tools_versions();
        assert_eq!(versions["Package.swift"], "5.7");
        assert_eq!(versions["Package@swift-5.9.swift"], "5.9");
        assert!(read(&testing::zip(&[("x/README.md", "x")])).is_err());
        assert!(read(b"not a zip").is_err());
    }

    #[test]
    fn manifest_variants_resolve_by_swift_version() {
        let variants = [
            "Package@swift-5.9.swift".to_owned(),
            "Package@swift-6.swift".to_owned(),
        ];
        assert_eq!(
            resolve(&variants, "6.0").as_deref(),
            Some("Package@swift-6.swift")
        );
        assert_eq!(
            resolve(&variants, "5.9.0").as_deref(),
            Some("Package@swift-5.9.swift")
        );
        assert_eq!(resolve(&variants, "5.8"), None);
        assert!(is_manifest("Package.swift") && !is_manifest("Package@swift-x.swift"));
    }
}
