use std::collections::{BTreeMap, HashSet};
use std::io::Read;

use serde_json::Value;

use crate::archive::{self, TarWriter};
use crate::paths::Variant;

pub const DESC: &str = "desc";
pub const FILES: &str = "files";
const MEMBER_LIMIT: u64 = 256 << 20;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub file_name: String,
    pub name: String,
    pub base: Option<String>,
    pub version: String,
    pub description: String,
    pub groups: Vec<String>,
    pub compressed_size: u64,
    pub installed_size: u64,
    pub sha256: String,
    pub signature: Option<String>,
    pub url: String,
    pub licenses: Vec<String>,
    pub arch: String,
    pub build_date: u64,
    pub packager: String,
    pub replaces: Vec<String>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub depends: Vec<String>,
    pub opt_depends: Vec<String>,
    pub make_depends: Vec<String>,
    pub check_depends: Vec<String>,
}

fn many(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

fn single_line(value: &str) -> String {
    value.replace(['\n', '\r'], " ")
}

impl Entry {
    pub fn from_attributes(value: &Value) -> Option<Entry> {
        let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
        let list = |key: &str| -> Vec<String> {
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
        let number = |key: &str| value.get(key).and_then(Value::as_u64).unwrap_or(0);
        Some(Entry {
            name: text("name")?,
            version: text("version")?,
            arch: text("architecture")?,
            base: text("base"),
            description: text("description").unwrap_or_default(),
            groups: list("groups"),
            installed_size: number("installedSize"),
            url: text("url").unwrap_or_default(),
            licenses: list("license"),
            build_date: number("buildDate"),
            packager: text("packager").unwrap_or_default(),
            replaces: list("replaces"),
            conflicts: list("conflicts"),
            provides: list("provides"),
            depends: list("depends"),
            opt_depends: list("optDepends"),
            make_depends: list("makeDepends"),
            check_depends: list("checkDepends"),
            ..Entry::default()
        })
    }

    pub fn directory(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    pub fn desc(&self) -> String {
        let mut out = String::new();
        let mut section = |key: &str, values: &[&str]| {
            let values: Vec<&&str> = values.iter().filter(|value| !value.is_empty()).collect();
            if values.is_empty() {
                return;
            }
            out.push('%');
            out.push_str(key);
            out.push_str("%\n");
            for value in values {
                out.push_str(&single_line(value));
                out.push('\n');
            }
            out.push('\n');
        };
        let compressed = self.compressed_size.to_string();
        let installed = self.installed_size.to_string();
        let built = self.build_date.to_string();
        section("FILENAME", &[&self.file_name]);
        section("NAME", &[&self.name]);
        section("BASE", &[self.base.as_deref().unwrap_or_default()]);
        section("VERSION", &[&self.version]);
        section("DESC", &[&self.description]);
        section("GROUPS", &many(&self.groups));
        section("CSIZE", &[&compressed]);
        section("ISIZE", &[&installed]);
        section("SHA256SUM", &[&self.sha256]);
        section("PGPSIG", &[self.signature.as_deref().unwrap_or_default()]);
        section("URL", &[&self.url]);
        section("LICENSE", &many(&self.licenses));
        section("ARCH", &[&self.arch]);
        section("BUILDDATE", &[&built]);
        section("PACKAGER", &[&self.packager]);
        section("REPLACES", &many(&self.replaces));
        section("CONFLICTS", &many(&self.conflicts));
        section("PROVIDES", &many(&self.provides));
        section("DEPENDS", &many(&self.depends));
        section("OPTDEPENDS", &many(&self.opt_depends));
        section("MAKEDEPENDS", &many(&self.make_depends));
        section("CHECKDEPENDS", &many(&self.check_depends));
        out
    }
}

pub fn files_text(files: &[String]) -> String {
    let mut out = String::from("%FILES%\n");
    for file in files {
        out.push_str(&single_line(file));
        out.push('\n');
    }
    out
}

pub fn sections(text: &str) -> BTreeMap<String, Vec<String>> {
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.len() > 2 && line.starts_with('%') && line.ends_with('%') {
            let key = line[1..line.len() - 1].to_owned();
            found.entry(key.clone()).or_default();
            current = Some(key);
        } else if !line.is_empty()
            && let Some(key) = &current
        {
            found.entry(key.clone()).or_default().push(line.to_owned());
        }
    }
    found
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub directory: String,
    pub name: String,
    pub members: Vec<(String, Vec<u8>)>,
}

impl Record {
    pub fn member(&self, name: &str) -> Option<&[u8]> {
        self.members
            .iter()
            .find(|(member, _)| member == name)
            .map(|(_, data)| data.as_slice())
    }

    pub fn field(&self, key: &str) -> Option<String> {
        let desc = String::from_utf8_lossy(self.member(DESC)?).into_owned();
        sections(&desc).remove(key)?.into_iter().next()
    }
}

fn name_of(directory: &str) -> String {
    let mut parts = directory.rsplitn(3, '-');
    let (_, _, name) = (parts.next(), parts.next(), parts.next());
    name.unwrap_or(directory).to_owned()
}

pub fn archive(records: &[Record], variant: Variant, mtime: u64) -> std::io::Result<Vec<u8>> {
    let mut sorted: Vec<&Record> = records.iter().collect();
    sorted.sort_by(|a, b| a.directory.cmp(&b.directory));
    let mut writer = TarWriter::gzip(mtime);
    for record in sorted {
        writer.directory(&record.directory)?;
        for (member, data) in &record.members {
            if variant == Variant::Db && member == FILES {
                continue;
            }
            writer.file(&format!("{}/{member}", record.directory), data)?;
        }
    }
    writer.finish()
}

#[derive(Debug, thiserror::Error)]
#[error("the package database cannot be read: {0}")]
pub struct DbError(String);

pub fn read(file: &[u8]) -> Result<Vec<Record>, DbError> {
    let failed = |error: std::io::Error| DbError(error.to_string());
    let opened = archive::open(file).map_err(failed)?;
    if opened.incomplete {
        return Err(DbError("the xz stream ends early".to_owned()));
    }
    let mut tar = tar::Archive::new(opened.reader);
    let mut records: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    for entry in tar.entries().map_err(failed)? {
        let mut entry = entry.map_err(failed)?;
        if archive::is_extension(&entry) {
            continue;
        }
        let name = archive::entry_name(&entry);
        let Some((directory, member)) = name.trim_end_matches('/').split_once('/') else {
            if entry.header().entry_type().is_dir() {
                records
                    .entry(name.trim_end_matches('/').to_owned())
                    .or_default();
            }
            continue;
        };
        if member.is_empty() || member.contains('/') || !entry.header().entry_type().is_file() {
            continue;
        }
        let mut data = Vec::new();
        (&mut entry)
            .take(MEMBER_LIMIT)
            .read_to_end(&mut data)
            .map_err(failed)?;
        records
            .entry(directory.to_owned())
            .or_default()
            .push((member.to_owned(), data));
    }
    Ok(records
        .into_iter()
        .filter(|(_, members)| !members.is_empty())
        .map(|(directory, members)| {
            let mut record = Record {
                name: name_of(&directory),
                directory,
                members,
            };
            if let Some(name) = record.field("NAME") {
                record.name = name;
            }
            record
        })
        .collect())
}

pub fn merge(databases: Vec<Vec<Record>>) -> Vec<Record> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    for records in databases {
        for record in records {
            if seen.insert(record.name.clone()) {
                merged.push(record);
            }
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Entry {
        Entry {
            file_name: "hello-1.0-1-x86_64.pkg.tar.zst".to_owned(),
            name: "hello".to_owned(),
            base: Some("hello".to_owned()),
            version: "1.0-1".to_owned(),
            description: "Says hello".to_owned(),
            compressed_size: 1234,
            installed_size: 5678,
            sha256: "ab".repeat(32),
            signature: Some("iQ==".to_owned()),
            url: "https://example.org/hello".to_owned(),
            licenses: vec!["MIT".to_owned(), "Apache-2.0".to_owned()],
            arch: "x86_64".to_owned(),
            build_date: 1_700_000_000,
            packager: "Astrum Tests <tests@example.org>".to_owned(),
            depends: vec!["glibc".to_owned()],
            opt_depends: vec!["bash: for the scripts".to_owned()],
            ..Entry::default()
        }
    }

    #[test]
    fn desc_files_list_their_sections_in_order() {
        let text = entry().desc();
        let headers: Vec<&str> = text.lines().filter(|line| line.starts_with('%')).collect();
        assert_eq!(
            headers,
            [
                "%FILENAME%",
                "%NAME%",
                "%BASE%",
                "%VERSION%",
                "%DESC%",
                "%CSIZE%",
                "%ISIZE%",
                "%SHA256SUM%",
                "%PGPSIG%",
                "%URL%",
                "%LICENSE%",
                "%ARCH%",
                "%BUILDDATE%",
                "%PACKAGER%",
                "%DEPENDS%",
                "%OPTDEPENDS%"
            ]
        );
        assert!(
            text.starts_with("%FILENAME%\nhello-1.0-1-x86_64.pkg.tar.zst\n\n%NAME%\nhello\n\n")
        );
        assert!(text.contains("\n%LICENSE%\nMIT\nApache-2.0\n\n"));
        assert!(text.ends_with("%OPTDEPENDS%\nbash: for the scripts\n\n"));
        let parsed = sections(&text);
        assert_eq!(parsed["CSIZE"], ["1234"]);
        assert_eq!(parsed["LICENSE"], ["MIT", "Apache-2.0"]);
        assert_eq!(
            files_text(&["usr/".to_owned(), "usr/bin/hello".to_owned()]),
            "%FILES%\nusr/\nusr/bin/hello\n"
        );
    }

    fn record(name: &str, version: &str, files: bool) -> Record {
        let entry = Entry {
            name: name.to_owned(),
            version: version.to_owned(),
            ..entry()
        };
        let mut members = vec![(DESC.to_owned(), entry.desc().into_bytes())];
        if files {
            members.push((
                FILES.to_owned(),
                files_text(&["usr/".to_owned()]).into_bytes(),
            ));
        }
        Record {
            directory: entry.directory(),
            name: name.to_owned(),
            members,
        }
    }

    #[test]
    fn databases_round_trip_and_merge_by_package_name() {
        let records = vec![
            record("zeta", "1-1", true),
            record("alpha", "2:1.0-3", true),
        ];
        let db = archive(&records, Variant::Db, 1_700_000_000).unwrap();
        let read_back = read(&db).unwrap();
        assert_eq!(read_back.len(), 2);
        assert_eq!(read_back[0].directory, "alpha-2:1.0-3");
        assert_eq!(read_back[0].name, "alpha");
        assert!(read_back[0].member(FILES).is_none());
        let files = read(&archive(&records, Variant::Files, 0).unwrap()).unwrap();
        assert_eq!(files[1].member(FILES).unwrap(), b"%FILES%\nusr/\n");
        let merged = merge(vec![
            vec![record("alpha", "1-1", false)],
            read_back,
            vec![record("beta", "1-1", false)],
        ]);
        let names: Vec<(&str, &str)> = merged
            .iter()
            .map(|record| (record.name.as_str(), record.directory.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                ("alpha", "alpha-1-1"),
                ("zeta", "zeta-1-1"),
                ("beta", "beta-1-1")
            ]
        );
        assert_eq!(name_of("lib32-gcc-libs-14.2.1-1"), "lib32-gcc-libs");
        assert!(read(b"garbage").is_err());
    }
}
