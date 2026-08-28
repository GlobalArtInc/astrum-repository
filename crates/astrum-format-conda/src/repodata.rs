use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::paths::PackageType;
use crate::raw::{self, RawError, quote};
use crate::version::Version;

pub const REPODATA_VERSION: i64 = 1;
pub const CHANNELDATA_VERSION: i64 = 1;

/// The fields of `info/about.json` that `channeldata.json` repeats for each package.
pub const ABOUT_FIELDS: &[&str] = &[
    "description",
    "dev_url",
    "doc_url",
    "home",
    "license",
    "license_family",
    "source_url",
    "summary",
];

/// A `repodata.json` document whose records are kept as raw JSON.
#[derive(Clone, Debug, Default)]
pub struct Repodata<'a> {
    pub subdir: String,
    pub packages: BTreeMap<String, Cow<'a, [u8]>>,
    pub conda: BTreeMap<String, Cow<'a, [u8]>>,
    pub removed: BTreeSet<String>,
}

struct Listed<'r, 'a> {
    kind: PackageType,
    file: &'r String,
    record: &'r Cow<'a, [u8]>,
    identity: Option<(String, Version)>,
}

#[derive(Deserialize)]
struct Identity {
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
}

impl<'a> Repodata<'a> {
    pub fn empty(subdir: &str) -> Repodata<'static> {
        Repodata {
            subdir: subdir.to_owned(),
            ..Repodata::default()
        }
    }

    pub fn insert(&mut self, kind: PackageType, file: String, record: Cow<'a, [u8]>) {
        let map = match kind {
            PackageType::TarBz2 => &mut self.packages,
            PackageType::Conda => &mut self.conda,
        };
        map.entry(file).or_insert(record);
    }

    pub fn len(&self) -> usize {
        self.packages.len() + self.conda.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads a member's `repodata.json`; records that are not valid JSON objects are dropped.
    pub fn parse(subdir: &str, bytes: &'a [u8]) -> Result<Repodata<'a>, RawError> {
        let top = raw::members(bytes)?;
        let mut found = Repodata::empty(subdir);
        for kind in [PackageType::TarBz2, PackageType::Conda] {
            let Some(object) = raw::member(&top, kind.key()) else {
                continue;
            };
            for (file, record) in raw::members(object)? {
                if record.first() == Some(&b'{') && raw::valid(record) {
                    found.insert(kind, file, Cow::Borrowed(record));
                }
            }
        }
        if let Some(removed) = raw::member(&top, "removed")
            && let Ok(list) = serde_json::from_slice::<Vec<String>>(removed)
        {
            found.removed.extend(list);
        }
        Ok(found)
    }

    /// Adds the records of `other` whose file names this document does not list yet.
    pub fn absorb(&mut self, other: Repodata<'a>) {
        for (file, record) in other.packages {
            self.packages.entry(file).or_insert(record);
        }
        for (file, record) in other.conda {
            self.conda.entry(file).or_insert(record);
        }
        self.removed.extend(other.removed);
    }

    /// The trimmed index conda reads first: every build of the newest version of each package.
    pub fn current(&self) -> Repodata<'a> {
        let identities: Vec<Listed<'_, 'a>> = self
            .packages
            .iter()
            .map(|(file, record)| (PackageType::TarBz2, file, record))
            .chain(
                self.conda
                    .iter()
                    .map(|(file, record)| (PackageType::Conda, file, record)),
            )
            .map(|(kind, file, record)| Listed {
                kind,
                file,
                record,
                identity: serde_json::from_slice::<Identity>(record)
                    .ok()
                    .filter(|identity| !identity.name.is_empty())
                    .map(|identity| {
                        let version = Version::parse(&identity.version);
                        (identity.name, version)
                    }),
            })
            .collect();
        let mut newest: HashMap<&str, &Version> = HashMap::new();
        for listed in &identities {
            if let Some((name, version)) = &listed.identity {
                newest
                    .entry(name.as_str())
                    .and_modify(|best| {
                        if version > *best {
                            *best = version;
                        }
                    })
                    .or_insert(version);
            }
        }
        let mut current = Repodata::empty(&self.subdir);
        for listed in &identities {
            if let Some((name, version)) = &listed.identity
                && newest
                    .get(name.as_str())
                    .is_some_and(|best| *best == version)
            {
                current.insert(listed.kind, listed.file.clone(), listed.record.clone());
            }
        }
        current
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            128 + self
                .packages
                .iter()
                .chain(self.conda.iter())
                .map(|(file, record)| file.len() + record.len() + 4)
                .sum::<usize>(),
        );
        let write = |out: &mut Vec<u8>, map: &BTreeMap<String, Cow<'a, [u8]>>| {
            out.push(b'{');
            for (at, (file, record)) in map.iter().enumerate() {
                if at > 0 {
                    out.push(b',');
                }
                out.extend_from_slice(quote(file).as_bytes());
                out.push(b':');
                out.extend_from_slice(record);
            }
            out.push(b'}');
        };
        write!(
            out,
            "{{\"info\":{{\"subdir\":{}}},\"packages\":",
            quote(&self.subdir)
        )
        .expect("writing into memory does not fail");
        write(&mut out, &self.packages);
        out.extend_from_slice(b",\"packages.conda\":");
        write(&mut out, &self.conda);
        out.extend_from_slice(b",\"removed\":");
        out.extend_from_slice(
            serde_json::to_string(&self.removed)
                .expect("a list of strings always serializes")
                .as_bytes(),
        );
        write!(out, ",\"repodata_version\":{REPODATA_VERSION}}}")
            .expect("writing into memory does not fail");
        out
    }
}

/// The repodata record of a package: its `info/index.json` with the size and checksums of the file.
pub fn record(
    index: &Map<String, Value>,
    subdir: &str,
    size: i64,
    md5: &str,
    sha256: &str,
) -> Vec<u8> {
    let mut record = index.clone();
    record.insert("md5".to_owned(), json!(md5));
    record.insert("sha256".to_owned(), json!(sha256));
    record.insert("size".to_owned(), json!(size));
    record
        .entry("subdir".to_owned())
        .or_insert_with(|| json!(subdir));
    serde_json::to_vec(&Value::Object(record)).expect("a JSON object always serializes")
}

/// What `channeldata.json` says of one package of a hosted channel.
#[derive(Clone, Debug, Default)]
pub struct Described {
    pub name: String,
    pub version: String,
    pub subdir: String,
    pub timestamp: Option<i64>,
    pub index: Map<String, Value>,
    pub about: Option<Map<String, Value>>,
    pub run_exports: Option<Value>,
}

pub fn channeldata(packages: &[Described]) -> Vec<u8> {
    struct Entry<'p> {
        newest: &'p Described,
        newest_version: Version,
        subdirs: BTreeSet<&'p str>,
        timestamp: Option<i64>,
        run_exports: BTreeMap<&'p str, &'p Value>,
    }
    let mut entries: BTreeMap<&str, Entry> = BTreeMap::new();
    let mut subdirs = BTreeSet::new();
    for package in packages {
        subdirs.insert(package.subdir.as_str());
        let version = Version::parse(&package.version);
        let entry = entries
            .entry(package.name.as_str())
            .or_insert_with(|| Entry {
                newest: package,
                newest_version: version.clone(),
                subdirs: BTreeSet::new(),
                timestamp: None,
                run_exports: BTreeMap::new(),
            });
        entry.subdirs.insert(package.subdir.as_str());
        entry.timestamp = entry.timestamp.max(package.timestamp);
        if version > entry.newest_version {
            entry.newest = package;
            entry.newest_version = version;
        }
        if let Some(run_exports) = &package.run_exports
            && run_exports
                .as_object()
                .is_some_and(|object| !object.is_empty())
        {
            entry
                .run_exports
                .insert(package.version.as_str(), run_exports);
        }
    }
    let mut described = Map::new();
    for (name, entry) in entries {
        let mut fields = Map::new();
        for field in ABOUT_FIELDS {
            let value = entry
                .newest
                .about
                .as_ref()
                .and_then(|about| about.get(*field))
                .or_else(|| entry.newest.index.get(*field))
                .filter(|value| !value.is_null());
            if let Some(value) = value {
                fields.insert((*field).to_owned(), value.clone());
            }
        }
        fields.insert("run_exports".to_owned(), json!(entry.run_exports));
        fields.insert("subdirs".to_owned(), json!(entry.subdirs));
        if let Some(timestamp) = entry.timestamp {
            fields.insert("timestamp".to_owned(), json!(timestamp));
        }
        fields.insert("version".to_owned(), json!(entry.newest.version));
        described.insert(name.to_owned(), Value::Object(fields));
    }
    serde_json::to_vec(&json!({
        "channeldata_version": CHANNELDATA_VERSION,
        "packages": described,
        "subdirs": subdirs,
    }))
    .expect("a JSON object always serializes")
}

pub fn empty_channeldata() -> Vec<u8> {
    channeldata(&[])
}

/// Merges the `channeldata.json` of group members: the first member that lists a package describes
/// it, and its subdirectories are those of every member.
pub fn merge_channeldata(documents: &[Value]) -> Vec<u8> {
    let mut packages: Map<String, Value> = Map::new();
    let mut subdirs: BTreeSet<String> = BTreeSet::new();
    for document in documents {
        if let Some(list) = document.get("subdirs").and_then(Value::as_array) {
            subdirs.extend(list.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        let Some(listed) = document.get("packages").and_then(Value::as_object) else {
            continue;
        };
        for (name, entry) in listed {
            match packages.get_mut(name) {
                None => {
                    packages.insert(name.clone(), entry.clone());
                }
                Some(existing) => {
                    let mut union: BTreeSet<String> = BTreeSet::new();
                    for side in [&*existing, entry] {
                        if let Some(list) = side.get("subdirs").and_then(Value::as_array) {
                            union.extend(list.iter().filter_map(Value::as_str).map(str::to_owned));
                        }
                    }
                    if let Some(object) = existing.as_object_mut() {
                        object.insert("subdirs".to_owned(), json!(union));
                    }
                }
            }
        }
    }
    serde_json::to_vec(&json!({
        "channeldata_version": CHANNELDATA_VERSION,
        "packages": packages,
        "subdirs": subdirs,
    }))
    .expect("a JSON object always serializes")
}

pub fn bz2(data: &[u8]) -> Vec<u8> {
    let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
    encoder
        .write_all(data)
        .expect("compressing in memory does not fail");
    encoder
        .finish()
        .expect("compressing in memory does not fail")
}

pub fn zst(data: &[u8]) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(data, ruzstd::encoding::CompressionLevel::Fastest)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    fn record_of(name: &str, version: &str, build: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"name": name, "version": version, "build": build, "build_number": 0, "depends": []})).unwrap()
    }

    #[test]
    fn documents_round_trip_and_trim_to_the_newest_versions() {
        let mut repodata = Repodata::empty("linux-64");
        repodata.insert(
            PackageType::TarBz2,
            "a-1.0-0.tar.bz2".to_owned(),
            Cow::Owned(record_of("a", "1.0", "0")),
        );
        repodata.insert(
            PackageType::Conda,
            "a-1.10-0.conda".to_owned(),
            Cow::Owned(record_of("a", "1.10", "0")),
        );
        repodata.insert(
            PackageType::Conda,
            "a-1.10-1.conda".to_owned(),
            Cow::Owned(record_of("a", "1.10", "1")),
        );
        repodata.insert(
            PackageType::Conda,
            "a-1.9-0.conda".to_owned(),
            Cow::Owned(record_of("a", "1.9", "0")),
        );
        repodata.insert(
            PackageType::TarBz2,
            "b-2.0rc1-0.tar.bz2".to_owned(),
            Cow::Owned(record_of("b", "2.0rc1", "0")),
        );
        let bytes = repodata.to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["info"]["subdir"], "linux-64");
        assert_eq!(value["repodata_version"], 1);
        assert_eq!(value["removed"], json!([]));
        assert_eq!(value["packages"]["a-1.0-0.tar.bz2"]["version"], "1.0");
        assert_eq!(value["packages.conda"].as_object().unwrap().len(), 3);

        let parsed = Repodata::parse("linux-64", &bytes).unwrap();
        assert_eq!(parsed.len(), 5);
        let current = parsed.current();
        let files: Vec<&String> = current
            .packages
            .keys()
            .chain(current.conda.keys())
            .collect();
        assert_eq!(
            files,
            ["b-2.0rc1-0.tar.bz2", "a-1.10-0.conda", "a-1.10-1.conda"]
        );
    }

    #[test]
    fn members_merge_with_the_first_one_winning() {
        let first = br#"{"info":{"subdir":"noarch"},"packages":{"x-1-0.tar.bz2":{"name":"x","version":"1","from":"first"}},"packages.conda":{},"removed":["old-1-0.tar.bz2"]}"#;
        let second = br#"{"packages":{"x-1-0.tar.bz2":{"name":"x","version":"1","from":"second"},"y-2-0.tar.bz2":{"name":"y","version":"2"},"broken-1-0.tar.bz2":[1]},"packages.conda":{"y-2-0.conda":{"name":"y","version":"2"}}}"#;
        let mut merged = Repodata::parse("noarch", first).unwrap();
        merged.absorb(Repodata::parse("noarch", second).unwrap());
        let value: Value = serde_json::from_slice(&merged.to_bytes()).unwrap();
        assert_eq!(value["packages"]["x-1-0.tar.bz2"]["from"], "first");
        assert_eq!(value["packages"]["y-2-0.tar.bz2"]["name"], "y");
        assert!(value["packages"].get("broken-1-0.tar.bz2").is_none());
        assert_eq!(value["packages.conda"]["y-2-0.conda"]["version"], "2");
        assert_eq!(value["removed"], json!(["old-1-0.tar.bz2"]));
    }

    #[test]
    fn records_carry_checksums_and_channeldata_the_newest_version() {
        let index = json!({"name": "a", "version": "1.0", "build": "0", "license": "MIT"});
        let record: Value =
            serde_json::from_slice(&record(index.as_object().unwrap(), "noarch", 12, "m", "s"))
                .unwrap();
        assert_eq!(record["size"], 12);
        assert_eq!(record["md5"], "m");
        assert_eq!(record["sha256"], "s");
        assert_eq!(record["subdir"], "noarch");

        let described = |version: &str, subdir: &str, summary: &str| Described {
            name: "a".to_owned(),
            version: version.to_owned(),
            subdir: subdir.to_owned(),
            timestamp: Some(version.len() as i64),
            index: index.as_object().unwrap().clone(),
            about: Some(
                json!({"summary": summary, "home": "https://example.org"})
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
            run_exports: Some(json!({"weak": [format!("a >={version}")]})),
        };
        let document: Value = serde_json::from_slice(&channeldata(&[
            described("1.10", "linux-64", "newest"),
            described("1.9", "noarch", "older"),
        ]))
        .unwrap();
        let entry = &document["packages"]["a"];
        assert_eq!(entry["version"], "1.10");
        assert_eq!(entry["summary"], "newest");
        assert_eq!(entry["license"], "MIT");
        assert_eq!(entry["subdirs"], json!(["linux-64", "noarch"]));
        assert_eq!(entry["run_exports"]["1.9"]["weak"][0], "a >=1.9");
        assert_eq!(document["subdirs"], json!(["linux-64", "noarch"]));
        assert_eq!(document["channeldata_version"], 1);

        let merged: Value = serde_json::from_slice(&merge_channeldata(&[
            json!({"packages": {"a": {"version": "2", "subdirs": ["noarch"]}}, "subdirs": ["noarch"]}),
            json!({"packages": {"a": {"version": "1", "subdirs": ["linux-64"]}, "b": {"version": "3", "subdirs": ["linux-64"]}}, "subdirs": ["linux-64"]}),
        ]))
        .unwrap();
        assert_eq!(merged["packages"]["a"]["version"], "2");
        assert_eq!(
            merged["packages"]["a"]["subdirs"],
            json!(["linux-64", "noarch"])
        );
        assert_eq!(merged["packages"]["b"]["version"], "3");
        assert_eq!(merged["subdirs"], json!(["linux-64", "noarch"]));
    }

    #[test]
    fn compressed_variants_decompress_to_the_document() {
        let document = Repodata::empty("noarch").to_bytes();
        let mut out = Vec::new();
        bzip2::read::MultiBzDecoder::new(bz2(&document).as_slice())
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, document);
        let compressed = zst(&document);
        let mut out = Vec::new();
        ruzstd::decoding::StreamingDecoder::new(compressed.as_slice())
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, document);
    }
}
