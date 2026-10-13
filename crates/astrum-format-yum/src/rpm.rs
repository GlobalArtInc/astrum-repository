use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

use crate::version;

pub const LEAD_SIZE: usize = 96;
const LEAD_MAGIC: [u8; 4] = [0xED, 0xAB, 0xEE, 0xDB];
const HEADER_MAGIC: [u8; 4] = [0x8E, 0xAD, 0xE8, 0x01];
const MAX_ENTRIES: u32 = 0xffff;
const MAX_DATA: u32 = 64 * 1024 * 1024;
const CHANGELOG_LIMIT: usize = 10;

pub const TYPE_CHAR: u32 = 1;
pub const TYPE_INT8: u32 = 2;
pub const TYPE_INT16: u32 = 3;
pub const TYPE_INT32: u32 = 4;
pub const TYPE_INT64: u32 = 5;
pub const TYPE_STRING: u32 = 6;
pub const TYPE_STRING_ARRAY: u32 = 8;
pub const TYPE_I18NSTRING: u32 = 9;

pub mod tag {
    pub const NAME: u32 = 1000;
    pub const VERSION: u32 = 1001;
    pub const RELEASE: u32 = 1002;
    pub const EPOCH: u32 = 1003;
    pub const SUMMARY: u32 = 1004;
    pub const DESCRIPTION: u32 = 1005;
    pub const BUILDTIME: u32 = 1006;
    pub const BUILDHOST: u32 = 1007;
    pub const SIZE: u32 = 1009;
    pub const VENDOR: u32 = 1011;
    pub const LICENSE: u32 = 1014;
    pub const PACKAGER: u32 = 1015;
    pub const GROUP: u32 = 1016;
    pub const URL: u32 = 1020;
    pub const OS: u32 = 1021;
    pub const ARCH: u32 = 1022;
    pub const OLDFILENAMES: u32 = 1027;
    pub const FILESIZES: u32 = 1028;
    pub const FILEMODES: u32 = 1030;
    pub const FILEFLAGS: u32 = 1037;
    pub const SOURCERPM: u32 = 1044;
    pub const ARCHIVESIZE: u32 = 1046;
    pub const PROVIDENAME: u32 = 1047;
    pub const REQUIREFLAGS: u32 = 1048;
    pub const REQUIRENAME: u32 = 1049;
    pub const REQUIREVERSION: u32 = 1050;
    pub const NOSOURCE: u32 = 1051;
    pub const NOPATCH: u32 = 1052;
    pub const CONFLICTFLAGS: u32 = 1053;
    pub const CONFLICTNAME: u32 = 1054;
    pub const CONFLICTVERSION: u32 = 1055;
    pub const CHANGELOGTIME: u32 = 1080;
    pub const CHANGELOGNAME: u32 = 1081;
    pub const CHANGELOGTEXT: u32 = 1082;
    pub const OBSOLETENAME: u32 = 1090;
    pub const SOURCEPACKAGE: u32 = 1106;
    pub const PROVIDEFLAGS: u32 = 1112;
    pub const PROVIDEVERSION: u32 = 1113;
    pub const OBSOLETEFLAGS: u32 = 1114;
    pub const OBSOLETEVERSION: u32 = 1115;
    pub const DIRINDEXES: u32 = 1116;
    pub const BASENAMES: u32 = 1117;
    pub const DIRNAMES: u32 = 1118;
    pub const PAYLOADFORMAT: u32 = 1124;
    pub const PAYLOADCOMPRESSOR: u32 = 1125;
    pub const LONGARCHIVESIZE: u32 = 271;
    pub const LONGSIZE: u32 = 5009;
    pub const RECOMMENDNAME: u32 = 5046;
    pub const RECOMMENDVERSION: u32 = 5047;
    pub const RECOMMENDFLAGS: u32 = 5048;
    pub const SUGGESTNAME: u32 = 5049;
    pub const SUGGESTVERSION: u32 = 5050;
    pub const SUGGESTFLAGS: u32 = 5051;
    pub const SUPPLEMENTNAME: u32 = 5052;
    pub const SUPPLEMENTVERSION: u32 = 5053;
    pub const SUPPLEMENTFLAGS: u32 = 5054;
    pub const ENHANCENAME: u32 = 5055;
    pub const ENHANCEVERSION: u32 = 5056;
    pub const ENHANCEFLAGS: u32 = 5057;
    pub const SIG_SIZE: u32 = 1000;
    pub const SIG_PAYLOADSIZE: u32 = 1007;
    pub const SIG_LONGARCHIVESIZE: u32 = 271;
}

const SENSE_LESS: u64 = 1 << 1;
const SENSE_GREATER: u64 = 1 << 2;
const SENSE_EQUAL: u64 = 1 << 3;
const SENSE_POSTTRANS: u64 = 1 << 5;
const SENSE_PREREQ: u64 = 1 << 6;
const SENSE_PRETRANS: u64 = 1 << 7;
const SENSE_SCRIPT_PRE: u64 = 1 << 9;
const SENSE_SCRIPT_POST: u64 = 1 << 10;
const SENSE_RPMLIB: u64 = 1 << 24;
const PRE_MASK: u64 =
    SENSE_PREREQ | SENSE_PRETRANS | SENSE_POSTTRANS | SENSE_SCRIPT_PRE | SENSE_SCRIPT_POST;

const FILE_TYPE_MASK: u64 = 0o170000;
const FILE_TYPE_DIRECTORY: u64 = 0o040000;
const FILE_FLAG_GHOST: u64 = 1 << 6;
const LIBC: &str = "libc.so.6";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RpmError {
    #[error("the file is not an RPM package")]
    NotRpm,
    #[error("the RPM package is truncated")]
    Truncated,
    #[error("{0}")]
    Invalid(String),
}

fn invalid(message: impl Into<String>) -> RpmError {
    RpmError::Invalid(message.into())
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    kind: u32,
    offset: usize,
    count: usize,
}

#[derive(Debug)]
pub struct Header {
    entries: HashMap<u32, Entry>,
    data: Vec<u8>,
    entry_count: usize,
}

fn be32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

impl Header {
    pub fn size_of_intro(intro: &[u8]) -> Result<usize, RpmError> {
        if intro.len() < 16 {
            return Err(RpmError::Truncated);
        }
        if intro[..4] != HEADER_MAGIC {
            return Err(invalid("the RPM header has a bad magic number"));
        }
        let count = be32(intro, 8);
        let length = be32(intro, 12);
        if count > MAX_ENTRIES {
            return Err(invalid(format!(
                "the RPM header has an invalid number of entries: {count}"
            )));
        }
        if length > MAX_DATA {
            return Err(invalid(format!(
                "the RPM header has an invalid data size: {length}"
            )));
        }
        Ok(16 + 16 * count as usize + length as usize)
    }

    pub fn parse(bytes: &[u8]) -> Result<Header, RpmError> {
        let size = Self::size_of_intro(bytes)?;
        if bytes.len() < size {
            return Err(RpmError::Truncated);
        }
        let count = be32(bytes, 8) as usize;
        let length = be32(bytes, 12) as usize;
        let index = &bytes[16..16 + count * 16];
        let data = bytes[16 + count * 16..size].to_vec();
        let mut entries = HashMap::new();
        for at in (0..count).map(|i| i * 16) {
            let tag = be32(index, at);
            let entry = Entry {
                kind: be32(index, at + 4),
                offset: be32(index, at + 8) as usize,
                count: be32(index, at + 12) as usize,
            };
            if entry.offset > length {
                return Err(invalid(format!(
                    "the RPM header entry {tag} points outside the header"
                )));
            }
            entries.entry(tag).or_insert(entry);
        }
        Ok(Header {
            entries,
            data,
            entry_count: count,
        })
    }

    pub fn size(&self) -> usize {
        16 + 16 * self.entry_count + self.data.len()
    }

    pub fn has(&self, tag: u32) -> bool {
        self.entries.contains_key(&tag)
    }

    pub fn strings(&self, tag: u32) -> Result<Vec<String>, RpmError> {
        let Some(entry) = self.entries.get(&tag) else {
            return Ok(Vec::new());
        };
        let count = match entry.kind {
            TYPE_STRING => 1,
            TYPE_STRING_ARRAY | TYPE_I18NSTRING => entry.count,
            _ => 0,
        };
        let mut values = Vec::with_capacity(count.min(4096));
        let mut position = entry.offset;
        for _ in 0..count {
            let end = self.data[position.min(self.data.len())..]
                .iter()
                .position(|byte| *byte == 0)
                .map(|found| position + found)
                .ok_or_else(|| {
                    invalid(format!("an unterminated string in RPM header entry {tag}"))
                })?;
            values.push(String::from_utf8_lossy(&self.data[position..end]).into_owned());
            position = end + 1;
        }
        Ok(values)
    }

    pub fn string(&self, tag: u32) -> Result<Option<String>, RpmError> {
        Ok(self.strings(tag)?.into_iter().next())
    }

    pub fn numbers(&self, tag: u32) -> Result<Vec<u64>, RpmError> {
        let Some(entry) = self.entries.get(&tag) else {
            return Ok(Vec::new());
        };
        let width = match entry.kind {
            TYPE_CHAR | TYPE_INT8 => 1,
            TYPE_INT16 => 2,
            TYPE_INT32 => 4,
            TYPE_INT64 => 8,
            _ => return Ok(Vec::new()),
        };
        let end = entry
            .count
            .checked_mul(width)
            .and_then(|length| length.checked_add(entry.offset))
            .filter(|end| *end <= self.data.len())
            .ok_or_else(|| invalid(format!("RPM header entry {tag} points outside the header")))?;
        Ok(self.data[entry.offset..end]
            .chunks(width)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0u64, |value, byte| (value << 8) | u64::from(*byte))
            })
            .collect())
    }

    pub fn number(&self, tag: u32) -> Result<Option<u64>, RpmError> {
        Ok(self.numbers(tag)?.into_iter().next())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub name: String,
    pub flags: Option<&'static str>,
    pub epoch: Option<String>,
    pub version: Option<String>,
    pub release: Option<String>,
    pub pre: bool,
}

impl Dependency {
    fn of(name: &str, sense: u64, evr: Option<&str>, pre: bool) -> Self {
        let flags = flags(sense);
        match (flags, evr.filter(|evr| !evr.is_empty())) {
            (Some(flags), Some(evr)) => {
                let (epoch, version, release) = version::split_evr(evr);
                Dependency {
                    name: name.to_owned(),
                    flags: Some(flags),
                    epoch: Some(epoch),
                    version: Some(version),
                    release,
                    pre,
                }
            }
            _ => Dependency {
                name: name.to_owned(),
                flags: None,
                epoch: None,
                version: None,
                release: None,
                pre,
            },
        }
    }

    fn with_pre(&self, pre: bool) -> Self {
        Dependency {
            pre,
            ..self.clone()
        }
    }
}

fn flags(sense: u64) -> Option<&'static str> {
    match sense & (SENSE_LESS | SENSE_GREATER | SENSE_EQUAL) {
        SENSE_LESS => Some("LT"),
        SENSE_GREATER => Some("GT"),
        SENSE_EQUAL => Some("EQ"),
        value if value == SENSE_LESS | SENSE_EQUAL => Some("LE"),
        value if value == SENSE_GREATER | SENSE_EQUAL => Some("GE"),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpmFile {
    pub path: String,
    pub kind: &'static str,
}

impl RpmFile {
    pub fn is_primary(&self) -> bool {
        is_primary_path(&self.path)
    }
}

fn is_primary_path(path: &str) -> bool {
    path.starts_with("/etc/") || path.contains("bin/") || path == "/usr/lib/sendmail"
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changelog {
    pub author: String,
    pub date: u64,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub epoch: String,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub packager: Option<String>,
    pub url: Option<String>,
    pub build_time: u64,
    pub installed_size: u64,
    pub archive_size: u64,
    pub license: Option<String>,
    pub vendor: Option<String>,
    pub group: Option<String>,
    pub build_host: Option<String>,
    pub source_rpm: Option<String>,
    pub header_start: u64,
    pub header_end: u64,
    pub provides: Vec<Dependency>,
    pub requires: Vec<Dependency>,
    pub conflicts: Vec<Dependency>,
    pub obsoletes: Vec<Dependency>,
    pub recommends: Vec<Dependency>,
    pub suggests: Vec<Dependency>,
    pub supplements: Vec<Dependency>,
    pub enhances: Vec<Dependency>,
    pub files: Vec<RpmFile>,
    pub changelogs: Vec<Changelog>,
}

impl Package {
    pub fn version_release(&self) -> String {
        format!("{}-{}", self.version, self.release)
    }

    pub fn component_version(&self) -> String {
        match self.epoch.as_str() {
            "0" => self.version_release(),
            epoch => format!("{epoch}:{}", self.version_release()),
        }
    }

    pub fn file_name(&self) -> String {
        format!("{}-{}.{}.rpm", self.name, self.version_release(), self.arch)
    }

    pub fn attributes(&self) -> Value {
        let mut attributes = Map::new();
        attributes.insert("name".to_owned(), json!(self.name));
        attributes.insert("epoch".to_owned(), json!(self.epoch));
        attributes.insert("version".to_owned(), json!(self.version));
        attributes.insert("release".to_owned(), json!(self.release));
        attributes.insert("architecture".to_owned(), json!(self.arch));
        for (key, value) in [
            ("summary", &self.summary),
            ("description", &self.description),
            ("license", &self.license),
            ("url", &self.url),
            ("vendor", &self.vendor),
            ("group", &self.group),
            ("sourceRpm", &self.source_rpm),
        ] {
            if let Some(value) = value.as_deref().filter(|value| !value.is_empty()) {
                attributes.insert(key.to_owned(), json!(value));
            }
        }
        if self.build_time > 0 {
            attributes.insert("buildTime".to_owned(), json!(self.build_time));
        }
        Value::Object(attributes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileName {
    pub name: String,
    pub version: String,
    pub release: String,
    pub arch: String,
}

impl FileName {
    pub fn parse(file: &str) -> Option<FileName> {
        let stem = file.strip_suffix(".rpm")?;
        let (rest, arch) = stem.rsplit_once('.')?;
        let (rest, release) = rest.rsplit_once('-')?;
        let (name, version) = rest.rsplit_once('-')?;
        if name.is_empty()
            || version.is_empty()
            || release.is_empty()
            || arch.is_empty()
            || arch.contains('-')
        {
            return None;
        }
        Some(FileName {
            name: name.to_owned(),
            version: version.to_owned(),
            release: release.to_owned(),
            arch: arch.to_owned(),
        })
    }

    pub fn version_release(&self) -> String {
        format!("{}-{}", self.version, self.release)
    }
}

pub fn needed(prefix: &[u8]) -> Result<Option<usize>, RpmError> {
    if prefix.len() < 4 {
        return Ok(Some(LEAD_SIZE + 16));
    }
    if prefix[..4] != LEAD_MAGIC {
        return Err(RpmError::NotRpm);
    }
    if prefix.len() < LEAD_SIZE + 16 {
        return Ok(Some(LEAD_SIZE + 16));
    }
    let signature = Header::size_of_intro(&prefix[LEAD_SIZE..])?;
    let start = LEAD_SIZE + signature + (8 - signature % 8) % 8;
    if prefix.len() < start + 16 {
        return Ok(Some(start + 16));
    }
    let header = Header::size_of_intro(&prefix[start..])?;
    match prefix.len() < start + header {
        true => Ok(Some(start + header)),
        false => Ok(None),
    }
}

pub fn parse(bytes: &[u8]) -> Result<Package, RpmError> {
    if bytes.len() < 4 || bytes[..4] != LEAD_MAGIC {
        return Err(RpmError::NotRpm);
    }
    if bytes.len() < LEAD_SIZE {
        return Err(RpmError::Truncated);
    }
    let signature = Header::parse(&bytes[LEAD_SIZE..])?;
    let size = signature.size();
    let start = LEAD_SIZE + size + (8 - size % 8) % 8;
    if bytes.len() < start {
        return Err(RpmError::Truncated);
    }
    let header = Header::parse(&bytes[start..])?;
    let end = start + header.size();
    build(&header, &signature, start as u64, end as u64)
}

fn required(header: &Header, tag: u32, what: &str) -> Result<String, RpmError> {
    header
        .string(tag)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid(format!("the RPM header has no {what}")))
}

fn build(
    header: &Header,
    signature: &Header,
    header_start: u64,
    header_end: u64,
) -> Result<Package, RpmError> {
    let files = files(header)?;
    let provides = dependencies(
        header,
        tag::PROVIDENAME,
        tag::PROVIDEFLAGS,
        tag::PROVIDEVERSION,
    )?;
    let requires = requires(header, &provides, &files)?;
    let arch = if !header.has(tag::SOURCERPM) || header.has(tag::SOURCEPACKAGE) {
        match header.has(tag::NOSOURCE) || header.has(tag::NOPATCH) {
            true => "nosrc".to_owned(),
            false => "src".to_owned(),
        }
    } else {
        required(header, tag::ARCH, "architecture")?
    };
    Ok(Package {
        name: required(header, tag::NAME, "name")?,
        version: required(header, tag::VERSION, "version")?,
        release: required(header, tag::RELEASE, "release")?,
        epoch: header
            .number(tag::EPOCH)?
            .map(|epoch| epoch.to_string())
            .unwrap_or_else(|| "0".to_owned()),
        arch,
        summary: header.string(tag::SUMMARY)?,
        description: header.string(tag::DESCRIPTION)?,
        packager: header.string(tag::PACKAGER)?,
        url: header.string(tag::URL)?,
        build_time: header.number(tag::BUILDTIME)?.unwrap_or(0),
        installed_size: match header.number(tag::LONGSIZE)? {
            Some(size) => size,
            None => header.number(tag::SIZE)?.unwrap_or(0),
        },
        archive_size: [
            header.number(tag::LONGARCHIVESIZE)?,
            header.number(tag::ARCHIVESIZE)?,
            signature.number(tag::SIG_LONGARCHIVESIZE)?,
            signature.number(tag::SIG_PAYLOADSIZE)?,
        ]
        .into_iter()
        .flatten()
        .next()
        .unwrap_or(0),
        license: header.string(tag::LICENSE)?,
        vendor: header.string(tag::VENDOR)?,
        group: header.string(tag::GROUP)?,
        build_host: header.string(tag::BUILDHOST)?,
        source_rpm: header.string(tag::SOURCERPM)?,
        header_start,
        header_end,
        provides,
        requires,
        conflicts: dependencies(
            header,
            tag::CONFLICTNAME,
            tag::CONFLICTFLAGS,
            tag::CONFLICTVERSION,
        )?,
        obsoletes: dependencies(
            header,
            tag::OBSOLETENAME,
            tag::OBSOLETEFLAGS,
            tag::OBSOLETEVERSION,
        )?,
        recommends: dependencies(
            header,
            tag::RECOMMENDNAME,
            tag::RECOMMENDFLAGS,
            tag::RECOMMENDVERSION,
        )?,
        suggests: dependencies(
            header,
            tag::SUGGESTNAME,
            tag::SUGGESTFLAGS,
            tag::SUGGESTVERSION,
        )?,
        supplements: dependencies(
            header,
            tag::SUPPLEMENTNAME,
            tag::SUPPLEMENTFLAGS,
            tag::SUPPLEMENTVERSION,
        )?,
        enhances: dependencies(
            header,
            tag::ENHANCENAME,
            tag::ENHANCEFLAGS,
            tag::ENHANCEVERSION,
        )?,
        files,
        changelogs: changelogs(header)?,
    })
}

fn dependencies(
    header: &Header,
    name_tag: u32,
    flags_tag: u32,
    version_tag: u32,
) -> Result<Vec<Dependency>, RpmError> {
    let names = header.strings(name_tag)?;
    let flags = header.numbers(flags_tag)?;
    let versions = header.strings(version_tag)?;
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let dependency = Dependency::of(
            name,
            flags.get(index).copied().unwrap_or(0),
            versions.get(index).map(String::as_str),
            false,
        );
        if seen.insert(dependency.clone()) {
            found.push(dependency);
        }
    }
    Ok(found)
}

fn requires(
    header: &Header,
    provides: &[Dependency],
    files: &[RpmFile],
) -> Result<Vec<Dependency>, RpmError> {
    let names = header.strings(tag::REQUIRENAME)?;
    let flags = header.numbers(tag::REQUIREFLAGS)?;
    let versions = header.strings(tag::REQUIREVERSION)?;
    let primary: HashSet<&str> = files
        .iter()
        .filter(|file| file.is_primary())
        .map(|file| file.path.as_str())
        .collect();
    let provided: HashSet<&Dependency> = provides.iter().collect();
    let mut requires: Vec<Dependency> = Vec::new();
    let mut libc: Option<Dependency> = None;
    for (index, name) in names.iter().enumerate() {
        let flag = flags.get(index).copied().unwrap_or(0);
        if name.starts_with("rpmlib(") || flag & SENSE_RPMLIB != 0 {
            continue;
        }
        let evr = versions.get(index).map(String::as_str);
        let dependency = Dependency::of(name, flag, evr, flag & PRE_MASK != 0);
        if name.starts_with(LIBC) {
            let newer = libc
                .as_ref()
                .is_none_or(|known| version::compare_libc(name, &known.name).is_gt());
            if newer {
                libc = Some(dependency);
            }
            continue;
        }
        if name.starts_with('/') && primary.contains(name.as_str()) {
            continue;
        }
        if provided.contains(&dependency.with_pre(false)) {
            continue;
        }
        if dependency.pre {
            let plain = dependency.with_pre(false);
            requires.retain(|known| *known != plain);
        } else if requires.contains(&dependency.with_pre(true)) {
            continue;
        }
        if !requires.contains(&dependency) {
            requires.push(dependency);
        }
    }
    if let Some(libc) = libc
        && !requires.contains(&libc)
    {
        requires.push(libc);
    }
    Ok(requires)
}

fn files(header: &Header) -> Result<Vec<RpmFile>, RpmError> {
    let base_names = header.strings(tag::BASENAMES)?;
    let paths = if base_names.is_empty() {
        header.strings(tag::OLDFILENAMES)?
    } else {
        let dir_names = header.strings(tag::DIRNAMES)?;
        let indexes = header.numbers(tag::DIRINDEXES)?;
        if indexes.len() < base_names.len() {
            return Err(invalid("the RPM header has incomplete file directories"));
        }
        base_names
            .iter()
            .zip(indexes)
            .map(|(base, index)| {
                dir_names
                    .get(index as usize)
                    .map(|dir| format!("{dir}{base}"))
                    .ok_or_else(|| invalid("the RPM header refers to a missing file directory"))
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let modes = header.numbers(tag::FILEMODES)?;
    let flags = header.numbers(tag::FILEFLAGS)?;
    Ok(paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let mode = modes.get(index).copied().unwrap_or(0);
            let flag = flags.get(index).copied().unwrap_or(0);
            let kind = if mode & FILE_TYPE_MASK == FILE_TYPE_DIRECTORY {
                "dir"
            } else if flag & FILE_FLAG_GHOST != 0 {
                "ghost"
            } else {
                ""
            };
            RpmFile { path, kind }
        })
        .collect())
}

fn changelogs(header: &Header) -> Result<Vec<Changelog>, RpmError> {
    let times = header.numbers(tag::CHANGELOGTIME)?;
    let authors = header.strings(tag::CHANGELOGNAME)?;
    let texts = header.strings(tag::CHANGELOGTEXT)?;
    let mut changelogs: Vec<Changelog> = times
        .into_iter()
        .zip(authors)
        .zip(texts)
        .take(CHANGELOG_LIMIT)
        .map(|((date, author), text)| Changelog { author, date, text })
        .collect();
    changelogs.reverse();
    Ok(changelogs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestRpm;

    #[test]
    fn packages_are_read_from_their_headers() {
        let data = TestRpm::new("hello", "1.2", "3.el9")
            .epoch(2)
            .file("/usr/bin/hello", 10)
            .file("/usr/share/doc/hello/README", 20)
            .directory("/usr/share/doc/hello")
            .ghost("/var/log/hello.log")
            .requires("/usr/bin/hello", 0, "")
            .requires("bash", 8 | 4, "5.0")
            .requires("libc.so.6()(64bit)", 0, "")
            .requires("libc.so.6(GLIBC_2.34)(64bit)", 0, "")
            .requires("coreutils", 1 << 9, "")
            .requires("coreutils", 0, "")
            .changelog(1_700_000_000, "Jane <jane@example.org> - 1.2-3", "- first")
            .changelog(1_600_000_000, "Jane <jane@example.org> - 1.1-1", "- older")
            .build();
        let package = parse(&data).unwrap();
        assert_eq!(package.name, "hello");
        assert_eq!(package.epoch, "2");
        assert_eq!(package.component_version(), "2:1.2-3.el9");
        assert_eq!(package.arch, "x86_64");
        assert_eq!(package.file_name(), "hello-1.2-3.el9.x86_64.rpm");
        assert_eq!(package.installed_size, 30);
        assert_eq!(package.header_start % 8, 0);
        assert!(package.header_end > package.header_start);
        assert_eq!(
            package
                .files
                .iter()
                .map(|file| (file.path.as_str(), file.kind))
                .collect::<Vec<_>>(),
            vec![
                ("/usr/bin/hello", ""),
                ("/usr/share/doc/hello/README", ""),
                ("/usr/share/doc/hello", "dir"),
                ("/var/log/hello.log", "ghost"),
            ]
        );
        let requires: Vec<_> = package
            .requires
            .iter()
            .map(|dependency| {
                (
                    dependency.name.as_str(),
                    dependency.flags,
                    dependency.version.as_deref(),
                    dependency.pre,
                )
            })
            .collect();
        assert_eq!(
            requires,
            vec![
                ("bash", Some("GE"), Some("5.0"), false),
                ("coreutils", None, None, true),
                ("libc.so.6(GLIBC_2.34)(64bit)", None, None, false),
            ]
        );
        assert_eq!(package.provides.len(), 2);
        assert_eq!(package.changelogs[0].text, "- older");
        assert_eq!(package.attributes()["architecture"], "x86_64");
    }

    #[test]
    fn source_packages_and_prefixes_are_recognised() {
        let data = TestRpm::new("hello", "1.2", "3").source().build();
        assert_eq!(parse(&data).unwrap().arch, "src");
        assert_eq!(needed(&data[..2]).unwrap(), Some(LEAD_SIZE + 16));
        let mut length = LEAD_SIZE + 16;
        while let Some(more) = needed(&data[..length]).unwrap() {
            assert!(more > length);
            length = more;
        }
        assert!(parse(&data[..length]).is_ok());
        assert_eq!(parse(b"PK\x03\x04 not an rpm"), Err(RpmError::NotRpm));
        assert_eq!(needed(b"PK\x03\x04"), Err(RpmError::NotRpm));
    }

    #[test]
    fn file_names_split_into_their_parts() {
        let name = FileName::parse("python3-libs-3.9.18-1.el9.x86_64.rpm").unwrap();
        assert_eq!(name.name, "python3-libs");
        assert_eq!(name.version_release(), "3.9.18-1.el9");
        assert_eq!(name.arch, "x86_64");
        assert!(FileName::parse("hello.rpm").is_none());
        assert!(FileName::parse("hello-1.0.x86_64.rpm").is_none());
    }
}
