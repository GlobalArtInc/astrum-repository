use std::collections::HashSet;

use crate::dcf::{self, BINARY_FIELDS, Record, SOURCE_FIELDS};
use crate::rds;

pub const PLAIN: &str = "PACKAGES";
pub const GZ: &str = "PACKAGES.gz";
pub const RDS: &str = "PACKAGES.rds";
pub const FILES: [&str; 3] = [PLAIN, GZ, RDS];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Source,
    Windows,
    Mac,
}

impl Kind {
    pub fn fields(self) -> &'static [&'static str] {
        match self {
            Kind::Source => &SOURCE_FIELDS,
            Kind::Windows | Kind::Mac => &BINARY_FIELDS,
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Kind::Source => &[".tar.gz", ".tar.bz2", ".tar.xz"],
            Kind::Windows => &[".zip"],
            Kind::Mac => &[".tgz"],
        }
    }

    pub fn of_directory(directory: &str) -> Kind {
        if directory.contains("/bin/windows/") {
            Kind::Windows
        } else if directory.contains("/bin/macosx/") {
            Kind::Mac
        } else {
            Kind::Source
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Plain,
    Gz,
    Rds,
}

impl File {
    pub fn parse(name: &str) -> Option<File> {
        match name {
            PLAIN => Some(File::Plain),
            GZ => Some(File::Gz),
            RDS => Some(File::Rds),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            File::Plain => PLAIN,
            File::Gz => GZ,
            File::Rds => RDS,
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            File::Plain => "text/plain",
            File::Gz => "application/x-gzip",
            File::Rds => "application/octet-stream",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub fields: Vec<(String, String)>,
    pub raw: Option<String>,
}

impl Entry {
    pub fn get(&self, tag: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == tag)
            .map(|(_, value)| value.as_str())
    }

    pub fn from_record(record: Record) -> Entry {
        Entry {
            fields: record.fields,
            raw: Some(record.raw),
        }
    }

    pub fn text(&self) -> String {
        match &self.raw {
            Some(raw) => raw.clone(),
            None => self
                .fields
                .iter()
                .map(|(tag, value)| dcf::field(tag, value))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    pub kind: Kind,
    pub entries: Vec<Entry>,
}

pub struct Files {
    pub plain: Vec<u8>,
    pub gz: Vec<u8>,
    pub rds: Vec<u8>,
}

impl Files {
    pub fn get(&self, file: File) -> &[u8] {
        match file {
            File::Plain => &self.plain,
            File::Gz => &self.gz,
            File::Rds => &self.rds,
        }
    }
}

impl Index {
    pub fn plain(&self) -> Vec<u8> {
        let mut text = self
            .entries
            .iter()
            .map(Entry::text)
            .collect::<Vec<_>>()
            .join("\n\n");
        if !text.is_empty() {
            text.push('\n');
        }
        text.into_bytes()
    }

    pub fn columns(&self) -> Vec<&str> {
        let mut columns: Vec<&str> = self.kind.fields().to_vec();
        for entry in &self.entries {
            for (tag, _) in &entry.fields {
                if !columns.contains(&tag.as_str()) {
                    columns.push(tag);
                }
            }
        }
        columns
    }

    pub fn rds(&self) -> Vec<u8> {
        let columns = self.columns();
        let mut cells = Vec::with_capacity(columns.len() * self.entries.len());
        for column in &columns {
            for entry in &self.entries {
                cells.push(entry.get(column));
            }
        }
        let row_names = (!self.entries.is_empty()).then(|| {
            self.entries
                .iter()
                .map(|entry| entry.get("Package").unwrap_or_default())
                .collect()
        });
        rds::gzip(&rds::serialize(&rds::Matrix {
            rows: self.entries.len(),
            cells,
            row_names,
            column_names: columns,
        }))
    }

    pub fn files(&self) -> Files {
        let plain = self.plain();
        Files {
            gz: rds::gzip(&plain),
            rds: self.rds(),
            plain,
        }
    }
}

pub fn hosted_entry(
    kind: Kind,
    description: &serde_json::Map<String, serde_json::Value>,
    md5: &str,
) -> Entry {
    let mut fields = Vec::new();
    for tag in kind.fields() {
        let value = match *tag {
            "MD5sum" => Some(md5.to_owned()),
            tag => description
                .get(tag)
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
        };
        if let Some(value) = value {
            fields.push(((*tag).to_owned(), value));
        }
    }
    Entry { fields, raw: None }
}

pub fn merge(kind: Kind, members: Vec<Vec<Record>>) -> Index {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut entries = Vec::new();
    for records in members {
        for record in records {
            let Some(package) = record.get("Package").map(str::to_owned) else {
                continue;
            };
            let version = record.get("Version").unwrap_or_default().to_owned();
            if seen.insert((package, version)) {
                entries.push(Entry::from_record(record));
            }
        }
    }
    Index { kind, entries }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn description(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn hosted_entries_keep_the_fields_of_write_packages() {
        let source = hosted_entry(
            Kind::Source,
            &description(json!({
                "Package": "astrumhello",
                "Version": "1.1.0",
                "Title": "Say Hello",
                "Depends": "R (>= 3.5.0)",
                "Imports": "utils",
                "Suggests": "testthat",
                "License": "MIT + file LICENSE",
                "NeedsCompilation": "no",
                "Packaged": "2026-10-18 06:34:16 UTC; root"
            })),
            "b0f6f81a0a210873c88e7b64a708054f",
        );
        let other = hosted_entry(
            Kind::Source,
            &description(json!({
                "Package": "astrumdep",
                "Version": "0.2.1",
                "Imports": "astrumhello (>= 1.1.0)",
                "LinkingTo": "Rcpp",
                "License": "GPL-2 | GPL-3",
                "NeedsCompilation": "no"
            })),
            "2fd3e11746e7f3c0fc3dfab7680f2268",
        );
        let index = Index {
            kind: Kind::Source,
            entries: vec![other, source],
        };
        assert_eq!(
            String::from_utf8(index.plain()).unwrap(),
            "Package: astrumdep\nVersion: 0.2.1\nImports: astrumhello (>= 1.1.0)\nLinkingTo: Rcpp\nLicense: GPL-2 | GPL-3\nMD5sum: 2fd3e11746e7f3c0fc3dfab7680f2268\nNeedsCompilation: no\n\nPackage: astrumhello\nVersion: 1.1.0\nDepends: R (>= 3.5.0)\nImports: utils\nSuggests: testthat\nLicense: MIT + file LICENSE\nMD5sum: b0f6f81a0a210873c88e7b64a708054f\nNeedsCompilation: no\n"
        );
        assert_eq!(index.columns(), SOURCE_FIELDS.to_vec());
        let binary = hosted_entry(
            Kind::Windows,
            &description(
                json!({"Package": "a1", "Version": "1.0", "NeedsCompilation": "yes", "Built": "R 4.4.1; ; 2026-01-01; windows"}),
            ),
            "00",
        );
        assert_eq!(binary.text(), "Package: a1\nVersion: 1.0\nMD5sum: 00");
    }

    #[test]
    fn groups_keep_the_first_entry_of_a_package_version() {
        let first =
            dcf::parse("Package: a1\nVersion: 1.0\nPath: sub\n\nPackage: b1\nVersion: 2.0\n");
        let second = dcf::parse(
            "Package: a1\nVersion: 1.0\nMD5sum: other\n\nPackage: a1\nVersion: 0.9\nDepends: R (>= 3.0),\n        methods\n",
        );
        let merged = merge(Kind::Source, vec![first, second]);
        let versions: Vec<(&str, &str)> = merged
            .entries
            .iter()
            .map(|entry| (entry.get("Package").unwrap(), entry.get("Version").unwrap()))
            .collect();
        assert_eq!(versions, [("a1", "1.0"), ("b1", "2.0"), ("a1", "0.9")]);
        assert_eq!(merged.entries[0].get("Path"), Some("sub"));
        assert!(
            String::from_utf8(merged.plain())
                .unwrap()
                .ends_with("Package: a1\nVersion: 0.9\nDepends: R (>= 3.0),\n        methods\n")
        );
        let columns = merged.columns();
        assert_eq!(columns.last(), Some(&"Path"));
        assert_eq!(
            merged.entries[2].get("Depends"),
            Some("R (>= 3.0),\nmethods")
        );
    }

    #[test]
    fn an_empty_index_is_empty_text() {
        let index = Index {
            kind: Kind::Mac,
            entries: vec![],
        };
        let files = index.files();
        assert!(files.plain.is_empty());
        assert_eq!(&files.gz[..2], &[0x1f, 0x8b]);
        assert_eq!(index.columns().len(), 14);
    }
}
