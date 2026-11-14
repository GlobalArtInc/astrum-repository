use serde::{Deserialize, Serialize};

use super::records::{self, Parsed};

const INSTALLED: [&str; 3] = ["install", "ok", "installed"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub source_version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Default)]
struct Record {
    seen: bool,
    package: String,
    version: String,
    source: String,
    arch: String,
    status: String,
}

fn installed(status: &str) -> bool {
    status.split_ascii_whitespace().eq(INSTALLED)
}

fn split_source(value: &str) -> (Option<String>, Option<String>) {
    let (name, version) = match value.split_once('(') {
        Some((name, rest)) => (
            name.trim(),
            rest.split_once(')')
                .map(|(version, _)| version.trim())
                .filter(|version| !version.is_empty()),
        ),
        None => (value.trim(), None),
    };
    (
        records::optional(name.to_owned()),
        version.map(str::to_owned).and_then(records::optional),
    )
}

fn finish(record: &mut Record, parsed: &mut Parsed<DpkgPackage>, room: usize) {
    let finished = std::mem::take(record);
    if !finished.seen || !installed(&finished.status) {
        return;
    }
    if !records::token(&finished.package) || !records::token(&finished.version) {
        parsed.malformed += 1;
        return;
    }
    if parsed.packages.len() >= room {
        parsed.truncated = true;
        return;
    }
    let (source, source_version) = split_source(&finished.source);
    parsed.packages.push(DpkgPackage {
        name: finished.package,
        version: finished.version,
        source,
        source_version,
        arch: records::optional(finished.arch),
    });
}

pub fn parse(bytes: &[u8], room: usize) -> Parsed<DpkgPackage> {
    let mut parsed = Parsed::default();
    let mut record = Record::default();
    for line in records::lines(bytes) {
        if line.trim_ascii().is_empty() {
            finish(&mut record, &mut parsed, room);
            continue;
        }
        if parsed.truncated {
            break;
        }
        if line[0] == b' ' || line[0] == b'\t' {
            continue;
        }
        let Some(colon) = line.iter().position(|byte| *byte == b':') else {
            continue;
        };
        record.seen = true;
        let key = line[..colon].trim_ascii();
        let value = records::field(&line[colon + 1..]);
        let is = |name: &str| key.eq_ignore_ascii_case(name.as_bytes());
        if is("Package") {
            record.package = value;
        } else if is("Version") {
            record.version = value;
        } else if is("Source") {
            record.source = value;
        } else if is("Architecture") {
            record.arch = value;
        } else if is("Status") {
            record.status = value;
        }
    }
    finish(&mut record, &mut parsed, room);
    parsed
}

#[cfg(test)]
mod tests;
