use serde::{Deserialize, Serialize};

use crate::inventory::purl;

use super::records;

const SCAN_LIMIT: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsRelease {
    pub path: String,
    pub id: String,
    pub version_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Apk,
    Dpkg,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Distro {
    pub family: Family,
    pub ecosystem: String,
    pub id: String,
    pub release: String,
}

fn unquote(value: &[u8]) -> Option<String> {
    let value = value.trim_ascii();
    let inner = match value {
        [first, inner @ .., last] if first == last && (*first == b'"' || *first == b'\'') => inner,
        other => other,
    };
    records::field(inner)
}

pub fn parse_os_release(path: &str, bytes: &[u8]) -> Option<OsRelease> {
    let mut id = None;
    let mut version_id = None;
    for line in records::lines(&bytes[..bytes.len().min(SCAN_LIMIT)]) {
        let line = line.trim_ascii();
        if line.first().is_none_or(|first| *first == b'#') {
            continue;
        }
        let Some(equals) = line.iter().position(|byte| *byte == b'=') else {
            continue;
        };
        let value = unquote(&line[equals + 1..]);
        match line[..equals].trim_ascii() {
            b"ID" => id = value,
            b"VERSION_ID" => version_id = value,
            _ => {}
        }
    }
    let id = id.filter(|id| records::token(id))?.to_ascii_lowercase();
    Some(OsRelease {
        path: path.to_owned(),
        id,
        version_id: version_id.filter(|version| records::token(version)),
    })
}

pub fn detect(os: &OsRelease) -> Option<Distro> {
    let release = os.version_id.as_deref()?;
    let ecosystem = purl::os_ecosystem(&os.id, release)?;
    let family = match os.id.as_str() {
        "alpine" => Family::Apk,
        _ => Family::Dpkg,
    };
    Some(Distro {
        family,
        ecosystem,
        id: os.id.clone(),
        release: release.to_owned(),
    })
}

#[cfg(test)]
mod tests;
