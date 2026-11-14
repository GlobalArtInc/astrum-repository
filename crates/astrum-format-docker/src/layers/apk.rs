use serde::{Deserialize, Serialize};

use super::limits::Limits;
use super::records::{self, Budget, Parsed, Truncation};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApkPackage {
    pub name: String,
    pub version: String,
    pub origin: Option<String>,
    pub arch: Option<String>,
}

impl ApkPackage {
    pub fn cost(&self) -> usize {
        self.name.len()
            + self.version.len()
            + self.origin.as_ref().map_or(0, String::len)
            + self.arch.as_ref().map_or(0, String::len)
    }
}

#[derive(Default)]
struct Record {
    seen: bool,
    too_long: bool,
    name: String,
    version: String,
    origin: String,
    arch: String,
}

fn build(record: Record) -> Option<ApkPackage> {
    if record.too_long || !records::token(&record.name) || !records::token(&record.version) {
        return None;
    }
    Some(ApkPackage {
        name: record.name,
        version: record.version,
        origin: records::optional(record.origin).ok()?,
        arch: records::optional(record.arch).ok()?,
    })
}

fn finish(
    record: &mut Record,
    parsed: &mut Parsed<ApkPackage>,
    limits: &Limits,
    budget: &mut Budget,
) {
    let finished = std::mem::take(record);
    if !finished.seen {
        return;
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

pub fn parse(bytes: &[u8], limits: &Limits, budget: &mut Budget) -> Parsed<ApkPackage> {
    let mut parsed = Parsed::default();
    let mut record = Record::default();
    for line in records::lines(bytes) {
        if line.trim_ascii().is_empty() {
            finish(&mut record, &mut parsed, limits, budget);
            continue;
        }
        if parsed.truncated.is_some() {
            break;
        }
        record.seen = true;
        if line.len() < 2 || line[1] != b':' {
            continue;
        }
        let value = &line[2..];
        match line[0] {
            b'P' => records::assign(&mut record.name, &mut record.too_long, value),
            b'V' => records::assign(&mut record.version, &mut record.too_long, value),
            b'o' => records::assign(&mut record.origin, &mut record.too_long, value),
            b'A' => records::assign(&mut record.arch, &mut record.too_long, value),
            _ => {}
        }
    }
    finish(&mut record, &mut parsed, limits, budget);
    parsed
}

#[cfg(test)]
mod tests;
