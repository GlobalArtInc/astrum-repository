use std::collections::BTreeMap;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde_json::{Map, Value, json};

use crate::addon::addon_path;
use crate::version::compare;

const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'@');

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub version: String,
    pub link: String,
    pub hash: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

pub type Updates = BTreeMap<String, Vec<Update>>;

pub fn encoded_path(path: &str) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn link(base: &str, id: &str, version: &str) -> String {
    format!(
        "{}{}",
        base.trim_end_matches('/'),
        encoded_path(&addon_path(id, version))
    )
}

pub fn sha256_hash(hex: &str) -> Option<String> {
    (hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| format!("sha256:{}", hex.to_ascii_lowercase()))
}

impl Update {
    fn json(&self) -> Value {
        let mut entry = Map::new();
        entry.insert("version".into(), json!(self.version));
        entry.insert("update_link".into(), json!(self.link));
        if let Some(hash) = &self.hash {
            entry.insert("update_hash".into(), json!(hash));
        }
        let mut gecko = Map::new();
        if let Some(min) = &self.min {
            gecko.insert("strict_min_version".into(), json!(min));
        }
        if let Some(max) = &self.max {
            gecko.insert("strict_max_version".into(), json!(max));
        }
        if !gecko.is_empty() {
            entry.insert(
                "browser_specific_settings".into(),
                json!({ "gecko": gecko }),
            );
        }
        Value::Object(entry)
    }
}

pub fn sort(updates: &mut Updates) {
    for list in updates.values_mut() {
        list.sort_by(|left, right| compare(&left.version, &right.version));
        list.dedup_by(|later, earlier| later.version == earlier.version);
    }
}

pub fn document(updates: &Updates) -> Value {
    let addons: Map<String, Value> = updates
        .iter()
        .map(|(id, list)| {
            let entries: Vec<Value> = list.iter().map(Update::json).collect();
            (id.clone(), json!({ "updates": entries }))
        })
        .collect();
    json!({ "addons": addons })
}

pub fn merge(lists: Vec<Updates>) -> Updates {
    let mut merged = Updates::new();
    for list in lists {
        for (id, updates) in list {
            if !updates.is_empty() {
                merged.entry(id).or_insert(updates);
            }
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(version: &str) -> Update {
        Update {
            version: version.to_owned(),
            link: format!("https://r.test/{version}.xpi"),
            hash: None,
            min: None,
            max: None,
        }
    }

    #[test]
    fn links_encode_braces_and_keep_the_at_sign() {
        assert_eq!(
            link("https://r.test/repository/ff/", "name@example.org", "1.0+b"),
            "https://r.test/repository/ff/name@example.org/1.0%2Bb/name-1.0_b.xpi"
        );
        assert_eq!(
            link(
                "https://r.test/repository/ff",
                "{ec8030f7-c20a-464f-9b0e-13a3a9e97384}",
                "2"
            ),
            "https://r.test/repository/ff/%7Bec8030f7-c20a-464f-9b0e-13a3a9e97384%7D/2/ec8030f7-c20a-464f-9b0e-13a3a9e97384-2.xpi"
        );
        assert_eq!(
            sha256_hash(&"AB".repeat(32)),
            Some(format!("sha256:{}", "ab".repeat(32)))
        );
        assert_eq!(sha256_hash("abc"), None);
    }

    #[test]
    fn documents_hold_every_version_in_order() {
        let mut updates = Updates::new();
        updates.insert(
            "a@b".to_owned(),
            vec![
                Update {
                    hash: Some("sha256:00".to_owned()),
                    min: Some("115.0".to_owned()),
                    ..update("1.10")
                },
                update("1.9"),
                update("1.9"),
            ],
        );
        sort(&mut updates);
        let document = document(&updates);
        let listed = document["addons"]["a@b"]["updates"].as_array().unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0]["version"], "1.9");
        assert!(listed[0].get("browser_specific_settings").is_none());
        assert!(listed[0].get("update_hash").is_none());
        assert_eq!(listed[1]["update_hash"], "sha256:00");
        assert_eq!(
            listed[1]["browser_specific_settings"]["gecko"]["strict_min_version"],
            "115.0"
        );
        assert!(
            listed[1]["browser_specific_settings"]["gecko"]
                .get("strict_max_version")
                .is_none()
        );
    }

    #[test]
    fn the_first_member_that_knows_an_add_on_wins() {
        let first = Updates::from([
            ("a@b".to_owned(), vec![update("1")]),
            ("c@d".to_owned(), vec![]),
        ]);
        let second = Updates::from([
            ("a@b".to_owned(), vec![update("2")]),
            ("c@d".to_owned(), vec![update("3")]),
        ]);
        let merged = merge(vec![first, second]);
        assert_eq!(merged["a@b"], vec![update("1")]);
        assert_eq!(merged["c@d"], vec![update("3")]);
    }
}
