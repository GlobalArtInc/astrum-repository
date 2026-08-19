use std::collections::HashSet;

use serde_json::{Value, json};

use crate::archive::{self, ArchiveError};
use crate::package::{NOARCH, Package};

pub const FILE: &str = "APKINDEX.tar.gz";
pub const INDEX: &str = "APKINDEX";
pub const DESCRIPTION: &str = "DESCRIPTION";
const INDEX_LIMIT: u64 = 512 << 20;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexEntry {
    pub checksum: String,
    pub name: String,
    pub version: String,
    pub arch: String,
    pub size: u64,
    pub installed_size: u64,
    pub description: String,
    pub url: String,
    pub license: String,
    pub origin: Option<String>,
    pub maintainer: Option<String>,
    pub build_time: Option<u64>,
    pub commit: Option<String>,
    pub provider_priority: Option<u64>,
    pub depends: Vec<String>,
    pub provides: Vec<String>,
    pub install_if: Vec<String>,
    pub data_hash: Option<String>,
}

fn words(values: Vec<&str>) -> Vec<String> {
    values
        .iter()
        .flat_map(|value| value.split_whitespace())
        .map(str::to_owned)
        .collect()
}

fn present(value: Option<&str>) -> Option<String> {
    value.filter(|value| !value.is_empty()).map(str::to_owned)
}

fn number(value: Option<&str>) -> Option<u64> {
    value.and_then(|value| value.trim().parse().ok())
}

fn single_line(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

impl IndexEntry {
    pub fn from_package(package: &Package, size: u64) -> IndexEntry {
        let info = &package.info;
        IndexEntry {
            checksum: package.checksum.clone(),
            name: info.name().to_owned(),
            version: info.version().to_owned(),
            arch: info.arch().to_owned(),
            size,
            installed_size: number(info.first("size")).unwrap_or(0),
            description: info.first("pkgdesc").unwrap_or_default().to_owned(),
            url: info.first("url").unwrap_or_default().to_owned(),
            license: info.first("license").unwrap_or_default().to_owned(),
            origin: present(info.first("origin")),
            maintainer: present(info.first("maintainer")),
            build_time: number(info.first("builddate")).filter(|time| *time > 0),
            commit: present(info.first("commit")),
            provider_priority: number(info.first("provider_priority")).filter(|value| *value > 0),
            depends: words(info.all("depend")),
            provides: words(info.all("provides")),
            install_if: words(info.all("install_if")),
            data_hash: present(info.first("datahash")),
        }
    }

    pub fn attributes(&self) -> Value {
        let mut attributes = json!({
            "checksum": self.checksum,
            "name": self.name,
            "version": self.version,
            "architecture": self.arch,
            "installedSize": self.installed_size,
            "description": self.description,
            "url": self.url,
            "license": self.license,
            "origin": self.origin,
            "maintainer": self.maintainer,
            "buildTime": self.build_time,
            "commit": self.commit,
            "providerPriority": self.provider_priority,
            "depends": self.depends,
            "provides": self.provides,
            "installIf": self.install_if,
            "dataHash": self.data_hash,
        });
        if let Some(map) = attributes.as_object_mut() {
            map.retain(|_, value| match value {
                Value::Null => false,
                Value::Array(items) => !items.is_empty(),
                _ => true,
            });
        }
        attributes
    }

    pub fn from_attributes(value: &Value, size: u64) -> Option<IndexEntry> {
        let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
        let list = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        Some(IndexEntry {
            checksum: text("checksum")?,
            name: text("name")?,
            version: text("version")?,
            arch: text("architecture").unwrap_or_else(|| NOARCH.to_owned()),
            size,
            installed_size: value
                .get("installedSize")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            description: text("description").unwrap_or_default(),
            url: text("url").unwrap_or_default(),
            license: text("license").unwrap_or_default(),
            origin: text("origin"),
            maintainer: text("maintainer"),
            build_time: value.get("buildTime").and_then(Value::as_u64),
            commit: text("commit"),
            provider_priority: value.get("providerPriority").and_then(Value::as_u64),
            depends: list("depends"),
            provides: list("provides"),
            install_if: list("installIf"),
            data_hash: text("dataHash"),
        })
    }

    pub fn render(&self, out: &mut String) {
        let mut line = |key: char, value: &str| {
            out.push(key);
            out.push(':');
            out.push_str(&single_line(value));
            out.push('\n');
        };
        line('C', &self.checksum);
        line('P', &self.name);
        line('V', &self.version);
        line('A', &self.arch);
        line('S', &self.size.to_string());
        line('I', &self.installed_size.to_string());
        line('T', &self.description);
        line('U', &self.url);
        line('L', &self.license);
        if let Some(origin) = &self.origin {
            line('o', origin);
        }
        if let Some(maintainer) = &self.maintainer {
            line('m', maintainer);
        }
        if let Some(time) = self.build_time {
            line('t', &time.to_string());
        }
        if let Some(commit) = &self.commit {
            line('c', commit);
        }
        if let Some(priority) = self.provider_priority {
            line('k', &priority.to_string());
        }
        for (key, values) in [
            ('D', &self.depends),
            ('p', &self.provides),
            ('i', &self.install_if),
        ] {
            if !values.is_empty() {
                line(key, &values.join(" "));
            }
        }
        out.push('\n');
    }
}

pub fn render(entries: &[IndexEntry]) -> String {
    let mut out = String::new();
    for entry in entries {
        entry.render(&mut out);
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub text: String,
    pub name: String,
    pub version: String,
    pub arch: String,
}

pub fn records(index: &str) -> Vec<Record> {
    let mut found = Vec::new();
    for block in index.split("\n\n") {
        let lines: Vec<&str> = block.lines().filter(|line| !line.is_empty()).collect();
        if lines.is_empty() {
            continue;
        }
        let field = |key: &str| {
            lines
                .iter()
                .find_map(|line| line.strip_prefix(key))
                .unwrap_or_default()
                .to_owned()
        };
        let name = field("P:");
        if name.is_empty() {
            continue;
        }
        let mut text = lines.join("\n");
        text.push_str("\n\n");
        found.push(Record {
            version: field("V:"),
            arch: field("A:"),
            name,
            text,
        });
    }
    found
}

pub fn merge<'a>(indexes: impl IntoIterator<Item = &'a str>) -> (String, usize) {
    let mut seen = HashSet::new();
    let mut out = String::new();
    let mut count = 0;
    for index in indexes {
        for record in records(index) {
            if seen.insert((record.name, record.version, record.arch)) {
                out.push_str(&record.text);
                count += 1;
            }
        }
    }
    (out, count)
}

pub fn archive(description: &str, index: &str, mtime: u64) -> Vec<u8> {
    let tar = archive::tar(
        &[
            (DESCRIPTION, description.as_bytes()),
            (INDEX, index.as_bytes()),
        ],
        mtime,
        false,
    )
    .expect("the index file names are valid tar paths");
    archive::gzip(&tar)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Read {
    pub description: String,
    pub index: String,
    pub signatures: Vec<String>,
}

pub fn read(file: &[u8]) -> Result<Read, ArchiveError> {
    let split = archive::split(file, INDEX_LIMIT)?;
    let mut rest = split.rest;
    let mut description = String::new();
    let mut index = None;
    while !rest.is_empty() && index.is_none() {
        let member = archive::member(rest, INDEX_LIMIT)?;
        rest = &rest[member.bytes.len()..];
        for entry in archive::entries(&member.content, |name| name == INDEX || name == DESCRIPTION)?
        {
            match entry.name.as_str() {
                INDEX => index = Some(String::from_utf8_lossy(&entry.data).into_owned()),
                DESCRIPTION => description = String::from_utf8_lossy(&entry.data).trim().to_owned(),
                _ => {}
            }
        }
    }
    let index =
        index.ok_or_else(|| ArchiveError::Tar("the archive holds no APKINDEX".to_owned()))?;
    Ok(Read {
        description,
        index,
        signatures: split.names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package;
    use crate::testing;

    fn entry() -> IndexEntry {
        let built = testing::build(&testing::Spec {
            depends: vec!["so:libc.musl-x86_64.so.1".to_owned(), "busybox".to_owned()],
            provides: vec!["cmd:hello=1.0-r0".to_owned()],
            ..testing::Spec::new("hello", "1.0-r0", "x86_64")
        });
        package::read(&built.bytes)
            .unwrap()
            .entry(built.bytes.len() as u64)
    }

    #[test]
    fn entries_render_in_the_order_of_apk_index() {
        let entry = entry();
        let mut text = String::new();
        entry.render(&mut text);
        let keys: String = text
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| &line[..1])
            .collect();
        assert_eq!(keys, "CPVASITULomtcDp");
        assert!(text.contains("\nT:hello for the tests\n"), "{text}");
        assert!(
            text.contains("\nD:so:libc.musl-x86_64.so.1 busybox\n"),
            "{text}"
        );
        assert!(text.ends_with("\np:cmd:hello=1.0-r0\n\n"), "{text}");
        assert_eq!(
            IndexEntry::from_attributes(&entry.attributes(), entry.size),
            Some(entry)
        );
    }

    #[test]
    fn indexes_merge_with_the_first_record_winning() {
        let one = "C:Q1a\nP:a\nV:1-r0\nA:x86_64\n\nC:Q1b\nP:b\nV:1-r0\nA:x86_64\n\n";
        let two = "C:Q1other\nP:a\nV:1-r0\nA:x86_64\n\nC:Q1c\nP:c\nV:2-r0\nA:x86_64\n";
        let (merged, count) = merge([one, two]);
        assert_eq!(count, 3);
        assert_eq!(
            merged,
            "C:Q1a\nP:a\nV:1-r0\nA:x86_64\n\nC:Q1b\nP:b\nV:1-r0\nA:x86_64\n\nC:Q1c\nP:c\nV:2-r0\nA:x86_64\n\n"
        );
    }

    #[test]
    fn index_archives_round_trip_with_their_signatures() {
        let index = "C:Q1a\nP:a\nV:1-r0\nA:x86_64\n\n";
        let unsigned = archive("v3.20/main", index, 1_700_000_000);
        let signature = archive::signature_member(".SIGN.RSA.repo.rsa.pub", &[7; 256], 0);
        let signed = [signature, unsigned.clone()].concat();
        let read = read(&signed).unwrap();
        assert_eq!(read.description, "v3.20/main");
        assert_eq!(read.index, index);
        assert_eq!(read.signatures, [".SIGN.RSA.repo.rsa.pub"]);
        assert!(super::read(&unsigned).unwrap().signatures.is_empty());
        let names: Vec<String> = archive::entries(
            &archive::member(&unsigned, 1 << 20).unwrap().content,
            |_| false,
        )
        .unwrap()
        .into_iter()
        .map(|entry| entry.name)
        .collect();
        assert_eq!(names, [DESCRIPTION, INDEX]);
    }
}
