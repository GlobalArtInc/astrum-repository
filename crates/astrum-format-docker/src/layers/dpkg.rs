use serde::{Deserialize, Serialize};

use super::limits::Limits;
use super::records::{self, Budget, Parsed, Truncation};

const GONE: [&str; 2] = ["not-installed", "config-files"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Database {
    Status,
    Part,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub source_version: Option<String>,
    pub arch: Option<String>,
}

impl DpkgPackage {
    pub fn cost(&self) -> usize {
        self.name.len()
            + self.version.len()
            + self.source.as_ref().map_or(0, String::len)
            + self.source_version.as_ref().map_or(0, String::len)
            + self.arch.as_ref().map_or(0, String::len)
    }
}

#[derive(Default)]
struct Record {
    seen: bool,
    too_long: bool,
    status_too_long: bool,
    package: String,
    version: String,
    source: String,
    arch: String,
    status: Option<String>,
}

enum Presence {
    Installed,
    Gone,
    Unknown,
}

fn presence(record: &Record, database: Database) -> Presence {
    if record.status_too_long {
        return Presence::Unknown;
    }
    let Some(status) = &record.status else {
        return match database {
            Database::Status => Presence::Unknown,
            Database::Part => Presence::Installed,
        };
    };
    match status.split_ascii_whitespace().nth(2) {
        None => Presence::Unknown,
        Some(word) if GONE.contains(&word) => Presence::Gone,
        Some(_) => Presence::Installed,
    }
}

fn split_source(value: &str) -> Option<(Option<String>, Option<String>)> {
    if value.is_empty() {
        return Some((None, None));
    }
    let (name, version) = match value.split_once('(') {
        Some((name, rest)) => (name.trim(), rest.split_once(')')?.0.trim()),
        None => (value.trim(), ""),
    };
    if !records::token(name) {
        return None;
    }
    let version = records::optional(version.to_owned()).ok()?;
    Some((Some(name.to_owned()), version))
}

fn build(record: Record) -> Option<DpkgPackage> {
    if record.too_long || !records::token(&record.package) || !records::token(&record.version) {
        return None;
    }
    let (source, source_version) = split_source(&record.source)?;
    Some(DpkgPackage {
        name: record.package,
        version: record.version,
        source,
        source_version,
        arch: records::optional(record.arch).ok()?,
    })
}

fn finish(
    record: &mut Record,
    parsed: &mut Parsed<DpkgPackage>,
    limits: &Limits,
    budget: &mut Budget,
    database: Database,
) {
    let finished = std::mem::take(record);
    if !finished.seen {
        return;
    }
    match presence(&finished, database) {
        Presence::Gone => return,
        Presence::Unknown => {
            parsed.malformed += 1;
            return;
        }
        Presence::Installed => {}
    }
    let Some(package) = build(finished) else {
        parsed.malformed += 1;
        return;
    };
    if parsed.packages.len() >= limits.max_db_packages {
        parsed.truncated = Some(Truncation::Count);
        return;
    }
    if !budget.take(package.cost()) {
        parsed.truncated = Some(Truncation::Budget);
        return;
    }
    parsed.packages.push(package);
}

pub fn parse(
    bytes: &[u8],
    limits: &Limits,
    budget: &mut Budget,
    database: Database,
) -> Parsed<DpkgPackage> {
    let mut parsed = Parsed::default();
    let mut record = Record::default();
    for line in records::lines(bytes) {
        if line.trim_ascii().is_empty() {
            finish(&mut record, &mut parsed, limits, budget, database);
            continue;
        }
        if parsed.truncated.is_some() {
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
        let value = &line[colon + 1..];
        let is = |name: &str| key.eq_ignore_ascii_case(name.as_bytes());
        if is("Package") {
            records::assign(&mut record.package, &mut record.too_long, value);
        } else if is("Version") {
            records::assign(&mut record.version, &mut record.too_long, value);
        } else if is("Source") {
            records::assign(&mut record.source, &mut record.too_long, value);
        } else if is("Architecture") {
            records::assign(&mut record.arch, &mut record.too_long, value);
        } else if is("Status") {
            let mut status = String::new();
            records::assign(&mut status, &mut record.status_too_long, value);
            record.status = Some(status);
        }
    }
    finish(&mut record, &mut parsed, limits, budget, database);
    parsed
}

#[cfg(test)]
mod tests;
