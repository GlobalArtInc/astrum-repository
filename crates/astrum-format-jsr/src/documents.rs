use std::collections::HashSet;

use serde_json::{Map, Value, json};

use crate::archive::Unpacked;
use crate::names::{Package, parse_version};

pub const MAX_LISTING: usize = 8 * 1024 * 1024;

pub type Versions = Vec<(String, Value)>;

fn yanked(value: &Value) -> bool {
    value
        .get("yanked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn latest(versions: &[(String, Value)]) -> Option<String> {
    versions
        .iter()
        .filter(|(_, value)| !yanked(value))
        .filter_map(|(text, _)| parse_version(text).map(|parsed| (parsed, text)))
        .filter(|(parsed, _)| parsed.pre.is_empty())
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, text)| text.clone())
}

pub fn listing(package: &Package, versions: &[(String, Value)]) -> Value {
    let mut document = Map::new();
    document.insert("scope".to_owned(), json!(package.scope));
    document.insert("name".to_owned(), json!(package.name));
    if let Some(latest) = latest(versions) {
        document.insert("latest".to_owned(), json!(latest));
    }
    let listed: Map<String, Value> = versions
        .iter()
        .map(|(version, value)| (version.clone(), value.clone()))
        .collect();
    document.insert("versions".to_owned(), Value::Object(listed));
    Value::Object(document)
}

pub fn parse_listing(bytes: &[u8]) -> Result<Versions, String> {
    let document: Value = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    let versions = document
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| "the document has no versions object".to_owned())?;
    Ok(versions
        .iter()
        .filter(|(version, _)| parse_version(version).is_some())
        .map(|(version, value)| {
            let value = match value {
                Value::Object(_) => value.clone(),
                _ => json!({}),
            };
            (version.clone(), value)
        })
        .collect())
}

pub fn merge(lists: Vec<Versions>) -> Versions {
    let mut merged: Versions = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for list in lists {
        for (version, value) in list {
            if seen.insert(version.clone()) {
                merged.push((version, value));
            }
        }
    }
    merged
}

pub fn version_meta(files: &Unpacked, exports: &Map<String, Value>) -> Value {
    let manifest: Map<String, Value> = files
        .files
        .iter()
        .map(|entry| {
            (
                format!("/{}", entry.path),
                json!({"size": entry.size, "checksum": format!("sha256-{}", entry.sha256)}),
            )
        })
        .collect();
    json!({"manifest": manifest, "exports": exports})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn versions(list: &[(&str, bool)]) -> Versions {
        list.iter()
            .map(|(version, gone)| {
                let value = if *gone {
                    json!({"yanked": true})
                } else {
                    json!({})
                };
                ((*version).to_owned(), value)
            })
            .collect()
    }

    #[test]
    fn latest_skips_prereleases_and_yanked_versions() {
        let list = versions(&[
            ("1.0.0", false),
            ("2.0.0-rc.1", false),
            ("1.5.0", false),
            ("1.9.0", true),
            ("1.10.0+build", false),
        ]);
        assert_eq!(latest(&list).as_deref(), Some("1.10.0+build"));
        assert_eq!(latest(&versions(&[("1.0.0-beta", false)])), None);
        assert_eq!(latest(&[]), None);
    }

    #[test]
    fn a_listing_names_the_scope_without_the_at_sign_and_omits_a_missing_latest() {
        let package = Package::new("acme", "hello").unwrap();
        let document = listing(&package, &versions(&[("1.0.0", false)]));
        assert_eq!(
            document,
            json!({"scope": "acme", "name": "hello", "latest": "1.0.0", "versions": {"1.0.0": {}}})
        );
        let pre = listing(&package, &versions(&[("1.0.0-rc.1", false)]));
        assert!(pre.get("latest").is_none());
    }

    #[test]
    fn merged_listings_keep_the_first_member_per_version() {
        let first = versions(&[("1.0.0", true), ("2.0.0", false)]);
        let second = versions(&[("1.0.0", false), ("3.0.0", false)]);
        let merged = merge(vec![first, second]);
        let names: Vec<&str> = merged.iter().map(|(version, _)| version.as_str()).collect();
        assert_eq!(names, ["1.0.0", "2.0.0", "3.0.0"]);
        assert!(yanked(&merged[0].1));
    }

    #[test]
    fn a_remote_listing_is_parsed_and_unknown_versions_are_dropped() {
        let parsed = parse_listing(
            br#"{"scope":"std","name":"path","latest":"1.0.0","versions":{"1.0.0":{},"nonsense":{},"0.9.0":{"yanked":true}}}"#,
        )
        .unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parse_listing(b"<html>").is_err());
        assert!(parse_listing(b"{}").is_err());
    }
}
