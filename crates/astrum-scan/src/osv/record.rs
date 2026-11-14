use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Datelike, SecondsFormat, Utc};
use serde_json::{Map, Value, json};

use super::{MAX_ECOSYSTEM_CHARS, normalise_name, valid_ecosystem};
use crate::cvss;
use crate::version::MAX_LEN;

pub const MAX_DOCUMENT_BYTES: usize = 256 * 1024;
const MAX_ID_CHARS: usize = 128;
const MAX_ALIASES: usize = 100;
const MAX_SUMMARY_CHARS: usize = 1024;
const MAX_NAME_CHARS: usize = 512;
const MAX_RANGES: usize = 256;
const MAX_EVENTS: usize = 4096;
const MAX_VERSIONS: usize = 100_000;
const KEPT_DETAILS_CHARS: usize = 8 * 1024;
const MIN_YEAR: i32 = 1000;
const MAX_YEAR: i32 = 9999;
const EVENT_KEYS: [&str; 4] = ["introduced", "fixed", "last_affected", "limit"];

#[derive(Clone, Debug, PartialEq)]
pub struct Affected {
    pub ecosystem: String,
    pub name: String,
    pub ranges: Vec<Value>,
    pub versions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub id: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub severity: &'static str,
    pub score: Option<f64>,
    pub vector: Option<String>,
    pub malicious: bool,
    pub published: Option<String>,
    pub modified: String,
    pub withdrawn: Option<String>,
    pub document: Value,
    pub affected: Vec<Affected>,
}

pub fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    !id.is_empty()
        && id.len() <= MAX_ID_CHARS
        && chars
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | ':' | '+' | '-' | '_'))
        && !id.contains("..")
}

pub fn entry_stem(name: &str) -> Option<&str> {
    let stem = name.strip_suffix(".json")?;
    valid_id(stem).then_some(stem)
}

fn timestamp(value: &Value) -> Option<DateTime<Utc>> {
    let parsed = DateTime::parse_from_rfc3339(value.as_str()?).ok()?;
    Some(parsed.with_timezone(&Utc))
}

fn stored_time(value: Option<&Value>) -> Result<Option<String>, &'static str> {
    let Some(at) = value.and_then(timestamp) else {
        return Ok(None);
    };
    if !(MIN_YEAR..=MAX_YEAR).contains(&at.year()) {
        return Err("a time of the record is outside the years the database stores");
    }
    Ok(Some(at.to_rfc3339_opts(SecondsFormat::Micros, true)))
}

fn has_nul(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains('\0'),
        Value::Array(items) => items.iter().any(has_nul),
        Value::Object(map) => map
            .iter()
            .any(|(key, item)| key.contains('\0') || has_nul(item)),
        _ => false,
    }
}

fn text(value: Option<&Value>, limit: usize) -> String {
    value
        .and_then(Value::as_str)
        .map(|text| text.chars().take(limit).collect())
        .unwrap_or_default()
}

fn text_severity(value: &str) -> Option<&'static str> {
    match value.to_ascii_lowercase().as_str() {
        "critical" => Some("critical"),
        "high" => Some("high"),
        "moderate" | "medium" => Some("medium"),
        "low" => Some("low"),
        _ => None,
    }
}

fn vector_score(record: &Value) -> Option<(f64, String)> {
    record
        .get("severity")?
        .as_array()?
        .iter()
        .filter(|entry| {
            entry
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "CVSS_V3")
        })
        .filter_map(|entry| {
            let vector = entry.get("score")?.as_str()?;
            Some((cvss::base_score(vector)?, vector.to_owned()))
        })
        .max_by(|left, right| left.0.total_cmp(&right.0))
}

fn event(value: &Value) -> Option<Value> {
    let object = value.as_object()?;
    let mut found = None;
    for key in EVENT_KEYS {
        if let Some(entry) = object.get(key) {
            let text = entry.as_str()?;
            if found.is_some() || text.is_empty() || text.len() > MAX_LEN {
                return None;
            }
            found = Some(json!({ key: text }));
        }
    }
    found
}

fn range(value: &Value) -> Option<Value> {
    let kind = value.get("type")?.as_str()?;
    if !matches!(kind, "ECOSYSTEM" | "SEMVER") {
        return None;
    }
    let listed = value.get("events")?.as_array()?;
    if listed.is_empty() || listed.len() > MAX_EVENTS {
        return None;
    }
    let events: Vec<Value> = listed.iter().filter_map(event).collect();
    if events.is_empty() {
        return None;
    }
    Some(json!({"type": kind, "events": events}))
}

fn affected(record: &Value) -> Vec<Affected> {
    let mut merged: BTreeMap<(String, String), (Affected, HashSet<String>)> = BTreeMap::new();
    let entries = record
        .get("affected")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for entry in entries {
        let Some(package) = entry.get("package") else {
            continue;
        };
        let (Some(ecosystem), Some(name)) = (
            package.get("ecosystem").and_then(Value::as_str),
            package.get("name").and_then(Value::as_str),
        ) else {
            continue;
        };
        if !valid_ecosystem(ecosystem)
            || ecosystem.len() > MAX_ECOSYSTEM_CHARS
            || name.is_empty()
            || name.chars().count() > MAX_NAME_CHARS
            || name.chars().any(char::is_control)
        {
            continue;
        }
        let name = normalise_name(ecosystem, name);
        let (slot, known_versions) = merged
            .entry((ecosystem.to_owned(), name.clone()))
            .or_insert_with(|| {
                (
                    Affected {
                        ecosystem: ecosystem.to_owned(),
                        name,
                        ranges: Vec::new(),
                        versions: Vec::new(),
                    },
                    HashSet::new(),
                )
            });
        let ranges = entry
            .get("ranges")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for found in ranges.iter().filter_map(range) {
            if slot.ranges.len() < MAX_RANGES && !slot.ranges.contains(&found) {
                slot.ranges.push(found);
            }
        }
        let versions = entry
            .get("versions")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for version in versions.iter().filter_map(Value::as_str) {
            if slot.versions.len() >= MAX_VERSIONS {
                break;
            }
            if !version.is_empty()
                && version.len() <= MAX_LEN
                && known_versions.insert(version.to_owned())
            {
                slot.versions.push(version.to_owned());
            }
        }
    }
    merged.into_values().map(|(slot, _)| slot).collect()
}

fn size(document: &Value) -> usize {
    serde_json::to_vec(document).map_or(usize::MAX, |bytes| bytes.len())
}

fn fit(mut document: Value, minimal: impl FnOnce() -> Value) -> Value {
    if size(&document) <= MAX_DOCUMENT_BYTES {
        return document;
    }
    if let Some(Value::String(details)) = document.get_mut("details") {
        *details = details.chars().take(KEPT_DETAILS_CHARS).collect();
    }
    for key in ["affected", "references", "credits", "database_specific"] {
        if size(&document) <= MAX_DOCUMENT_BYTES {
            return document;
        }
        if let Some(object) = document.as_object_mut() {
            object.remove(key);
        }
    }
    if size(&document) <= MAX_DOCUMENT_BYTES {
        document
    } else {
        minimal()
    }
}

pub fn parse(bytes: &[u8], stem: &str) -> Result<Record, &'static str> {
    let document: Value =
        serde_json::from_slice(bytes).map_err(|_| "the record is not valid JSON")?;
    let object = document.as_object().ok_or("the record is not an object")?;
    if has_nul(&document) {
        return Err("the record contains a NUL character");
    }
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or("the record has no id")?;
    if id != stem {
        return Err("the id of the record differs from its file name");
    }
    let id = id.to_owned();
    let modified =
        stored_time(object.get("modified"))?.ok_or("the record has no valid modified time")?;
    let published = stored_time(object.get("published"))?;
    let withdrawn = stored_time(object.get("withdrawn"))?;
    let mut aliases: Vec<String> = Vec::new();
    for alias in object
        .get("aliases")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
    {
        if aliases.len() < MAX_ALIASES
            && !alias.is_empty()
            && alias.chars().count() <= MAX_ID_CHARS
            && !alias.chars().any(char::is_control)
            && !aliases.iter().any(|known| known == alias)
        {
            aliases.push(alias.to_owned());
        }
    }
    let summary = text(object.get("summary"), MAX_SUMMARY_CHARS);
    let scored = vector_score(&document);
    let malicious = id.starts_with("MAL-");
    let from_text = object
        .get("database_specific")
        .and_then(|specific| specific.get("severity"))
        .and_then(Value::as_str)
        .and_then(text_severity);
    let severity = if malicious {
        "critical"
    } else if let Some(severity) = from_text {
        severity
    } else if let Some((score, _)) = &scored {
        cvss::Severity::from_score(*score).as_str()
    } else {
        "unknown"
    };
    let (score, vector) = match scored {
        Some((score, vector)) => (Some(score), Some(vector)),
        None => (None, None),
    };
    let affected = affected(&document);
    let document = fit(document, || {
        let mut minimal = Map::new();
        minimal.insert("id".to_owned(), id.clone().into());
        minimal.insert("modified".to_owned(), modified.clone().into());
        minimal.insert("summary".to_owned(), summary.clone().into());
        minimal.insert("aliases".to_owned(), aliases.clone().into());
        if let Some(published) = &published {
            minimal.insert("published".to_owned(), published.clone().into());
        }
        if let Some(withdrawn) = &withdrawn {
            minimal.insert("withdrawn".to_owned(), withdrawn.clone().into());
        }
        Value::Object(minimal)
    });
    Ok(Record {
        id,
        aliases,
        summary,
        severity,
        score,
        vector,
        malicious,
        published,
        modified,
        withdrawn,
        document,
        affected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_json(value: Value, stem: &str) -> Result<Record, &'static str> {
        parse(&serde_json::to_vec(&value).unwrap(), stem)
    }

    #[test]
    fn entry_names_must_be_flat_json_files() {
        assert_eq!(
            entry_stem("GHSA-1234-abcd-5678.json"),
            Some("GHSA-1234-abcd-5678")
        );
        assert_eq!(entry_stem("RHSA-2020:1234.json"), Some("RHSA-2020:1234"));
        for bad in [
            "../evil.json",
            "/etc/passwd.json",
            "a/b.json",
            "a\\b.json",
            "..json",
            ".hidden.json",
            "a..b.json",
            "readme.txt",
            ".json",
        ] {
            assert_eq!(entry_stem(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_record_is_read_leniently_and_names_are_normalised() {
        let record = parse_json(
            json!({
                "id": "PYSEC-1", "modified": "2024-01-02T03:04:05Z", "unknown": {"x": 1},
                "aliases": ["CVE-1", "CVE-1", 5],
                "summary": "s",
                "severity": [{"type": "CVSS_V3", "score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"}],
                "affected": [
                    {"package": {"ecosystem": "PyPI", "name": "Django_REST"},
                     "ranges": [{"type": "ECOSYSTEM", "events": [{"introduced": "0"}, {"fixed": "2.0"}, {"bad": 1}]},
                                {"type": "GIT", "events": [{"introduced": "0"}]}],
                     "versions": ["1.0", "1.0"]},
                    {"package": {"ecosystem": "PyPI", "name": "django-rest"},
                     "ranges": [{"type": "SEMVER", "events": [{"introduced": "3.0"}]}],
                     "versions": ["3.1"]},
                    {"package": {"ecosystem": "bad/eco", "name": "x"}},
                    {"package": {"name": "no-eco"}}
                ]
            }),
            "PYSEC-1",
        )
        .unwrap();
        assert_eq!(record.severity, "critical");
        assert_eq!(record.score, Some(9.8));
        assert_eq!(record.aliases, ["CVE-1"]);
        assert_eq!(record.affected.len(), 1);
        let package = &record.affected[0];
        assert_eq!(package.name, "django-rest");
        assert_eq!(package.ranges.len(), 2);
        assert_eq!(package.ranges[0]["events"].as_array().unwrap().len(), 2);
        assert_eq!(package.versions, ["1.0", "3.1"]);
    }

    #[test]
    fn severity_follows_text_then_cvss_then_unknown_and_malware_is_critical() {
        let base = |extra: Value, id: &str| {
            let mut record = json!({"id": id, "modified": "2024-01-01T00:00:00Z"});
            record
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            parse_json(record, id).unwrap()
        };
        let moderate = base(
            json!({"database_specific": {"severity": "MODERATE"},
                   "severity": [{"type": "CVSS_V3", "score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"}]}),
            "GHSA-1",
        );
        assert_eq!((moderate.severity, moderate.score), ("medium", Some(9.8)));
        let v4_only = base(
            json!({"severity": [{"type": "CVSS_V4", "score": "CVSS:4.0/AV:N/AC:L/AT:N/PR:N/UI:N/VC:H/VI:H/VA:H/SC:N/SI:N/SA:N"}]}),
            "GHSA-2",
        );
        assert_eq!((v4_only.severity, v4_only.score), ("unknown", None));
        let malware = base(json!({}), "MAL-2024-1");
        assert!(malware.malicious);
        assert_eq!(malware.severity, "critical");
    }

    #[test]
    fn bad_records_are_refused_with_a_reason() {
        assert!(parse(b"{not json", "A-1").is_err());
        assert!(parse(b"[1]", "A-1").is_err());
        assert!(parse_json(json!({"id": "A-1"}), "A-1").is_err());
        assert!(parse_json(json!({"id": "A-1", "modified": "yesterday"}), "A-1").is_err());
        assert!(
            parse_json(
                json!({"id": "A-2", "modified": "2024-01-01T00:00:00Z"}),
                "A-1"
            )
            .is_err()
        );
    }

    #[test]
    fn an_oversized_document_loses_details_and_then_bulk() {
        let record = parse_json(
            json!({"id": "A-1", "modified": "2024-01-01T00:00:00Z",
                   "details": "d".repeat(400 * 1024)}),
            "A-1",
        )
        .unwrap();
        assert!(size(&record.document) <= MAX_DOCUMENT_BYTES);
        assert_eq!(
            record.document["details"].as_str().unwrap().len(),
            KEPT_DETAILS_CHARS
        );
        let huge = parse_json(
            json!({"id": "A-1", "modified": "2024-01-01T00:00:00Z",
                   "credits": [{"name": "c".repeat(400 * 1024)}]}),
            "A-1",
        )
        .unwrap();
        assert!(huge.document.get("credits").is_none());
        assert_eq!(huge.document["id"], "A-1");
    }
}
