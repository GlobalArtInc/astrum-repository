use serde::{Deserialize, Serialize};

use super::records::{self, Parsed};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApkPackage {
    pub name: String,
    pub version: String,
    pub origin: Option<String>,
    pub arch: Option<String>,
}

#[derive(Default)]
struct Record {
    seen: bool,
    name: String,
    version: String,
    origin: String,
    arch: String,
}

fn finish(record: &mut Record, parsed: &mut Parsed<ApkPackage>, room: usize) {
    let finished = std::mem::take(record);
    if !finished.seen {
        return;
    }
    if !records::token(&finished.name) || !records::token(&finished.version) {
        parsed.malformed += 1;
        return;
    }
    if parsed.packages.len() >= room {
        parsed.truncated = true;
        return;
    }
    parsed.packages.push(ApkPackage {
        name: finished.name,
        version: finished.version,
        origin: records::optional(finished.origin),
        arch: records::optional(finished.arch),
    });
}

pub fn parse(bytes: &[u8], room: usize) -> Parsed<ApkPackage> {
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
        if line.len() < 2 || line[1] != b':' {
            record.seen = true;
            continue;
        }
        record.seen = true;
        let value = records::field(&line[2..]);
        match line[0] {
            b'P' => record.name = value,
            b'V' => record.version = value,
            b'o' => record.origin = value,
            b'A' => record.arch = value,
            _ => {}
        }
    }
    finish(&mut record, &mut parsed, room);
    parsed
}

#[cfg(test)]
mod tests;
