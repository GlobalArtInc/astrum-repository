use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value};

use crate::names;

pub type Document = Map<String, Value>;

pub const MAX_DOCUMENT: usize = 4 * 1024 * 1024;

const SOURCE_TYPES: &[&str] = &["archive", "git_repository", "local_path"];

pub fn parse(bytes: &[u8], what: &str) -> Result<Document, String> {
    match serde_json::from_slice::<Value>(bytes) {
        Ok(Value::Object(document)) => Ok(document),
        Ok(_) => Err(format!("{what} must be a JSON object")),
        Err(error) => Err(format!("{what} is not valid JSON: {error}")),
    }
}

pub fn render(document: &Document) -> Vec<u8> {
    let mut body = serde_json::to_vec_pretty(document).expect("a JSON object serializes");
    body.push(b'\n');
    body
}

fn text<'a>(document: &'a Document, key: &str, what: &str) -> Result<Option<&'a str>, String> {
    match document.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(format!("{what}: {key} must be a string")),
    }
}

fn texts(document: &Document, key: &str, what: &str) -> Result<Vec<String>, String> {
    match document.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::String(value) if !value.is_empty() => Ok(value.clone()),
                _ => Err(format!("{what}: {key} must hold non-empty strings")),
            })
            .collect(),
        Some(_) => Err(format!("{what}: {key} must be an array of strings")),
    }
}

pub fn validate_registry(bytes: &[u8]) -> Result<Document, String> {
    let what = "bazel_registry.json";
    let document = parse(bytes, what)?;
    texts(&document, "mirrors", what)?;
    text(&document, "module_base_path", what)?;
    Ok(document)
}

pub fn merge_registries(documents: &[Document]) -> Document {
    let mut mirrors: Vec<String> = Vec::new();
    let mut base = None;
    for document in documents {
        for mirror in texts(document, "mirrors", "").unwrap_or_default() {
            if !mirrors.contains(&mirror) {
                mirrors.push(mirror);
            }
        }
        if base.is_none() {
            base = text(document, "module_base_path", "")
                .ok()
                .flatten()
                .map(str::to_owned);
        }
    }
    let mut merged = Document::new();
    merged.insert(
        "mirrors".to_owned(),
        Value::Array(mirrors.into_iter().map(Value::String).collect()),
    );
    if let Some(base) = base {
        merged.insert("module_base_path".to_owned(), Value::String(base));
    }
    merged
}

pub fn validate_metadata(bytes: &[u8]) -> Result<Document, String> {
    let what = "metadata.json";
    let document = parse(bytes, what)?;
    text(&document, "homepage", what)?;
    text(&document, "deprecated", what)?;
    texts(&document, "repository", what)?;
    texts(&document, "versions", what)?;
    match document.get("maintainers") {
        None | Some(Value::Null) => {}
        Some(Value::Array(items)) if items.iter().all(Value::is_object) => {}
        Some(_) => return Err(format!("{what}: maintainers must be an array of objects")),
    }
    match document.get("yanked_versions") {
        None | Some(Value::Null) => {}
        Some(Value::Object(yanked)) if yanked.values().all(Value::is_string) => {
            if let Some(version) = yanked.keys().find(|version| !names::valid_version(version)) {
                return Err(format!(
                    "{what}: {version} is not a valid version in yanked_versions"
                ));
            }
        }
        Some(_) => {
            return Err(format!(
                "{what}: yanked_versions must map versions to the reasons they were yanked"
            ));
        }
    }
    Ok(document)
}

pub fn build_metadata(stored: Option<Document>, mut versions: Vec<String>) -> Document {
    let mut document = stored.unwrap_or_default();
    document.retain(|_, value| !value.is_null());
    names::sort_versions(&mut versions);
    versions.dedup();
    document.insert(
        "versions".to_owned(),
        Value::Array(versions.into_iter().map(Value::String).collect()),
    );
    if !document
        .get("yanked_versions")
        .is_some_and(Value::is_object)
    {
        document.insert("yanked_versions".to_owned(), Value::Object(Map::new()));
    }
    document
}

pub fn versions_of(document: &Document) -> Vec<String> {
    texts(document, "versions", "").unwrap_or_default()
}

fn blank(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.is_empty(),
        Value::Array(items) => items.is_empty(),
        Value::Object(fields) => fields.is_empty(),
        _ => false,
    }
}

pub fn merge_metadata(documents: Vec<Document>) -> Document {
    let mut versions: Vec<String> = Vec::new();
    let mut yanked = Map::new();
    let mut merged = Document::new();
    for document in documents {
        for (key, value) in document {
            match key.as_str() {
                "versions" => {
                    if let Value::Array(items) = value {
                        for item in items.into_iter().filter_map(|item| match item {
                            Value::String(version) => Some(version),
                            _ => None,
                        }) {
                            if !versions.contains(&item) {
                                versions.push(item);
                            }
                        }
                    }
                }
                "yanked_versions" => {
                    if let Value::Object(fields) = value {
                        for (version, reason) in fields {
                            yanked.entry(version).or_insert(reason);
                        }
                    }
                }
                _ if blank(&value) => {}
                _ => {
                    merged.entry(key).or_insert(value);
                }
            }
        }
    }
    names::sort_versions(&mut versions);
    merged.insert(
        "versions".to_owned(),
        Value::Array(versions.into_iter().map(Value::String).collect()),
    );
    merged.insert("yanked_versions".to_owned(), Value::Object(yanked));
    merged
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub patches: BTreeMap<String, String>,
    pub overlay: BTreeMap<String, String>,
}

fn listed(document: &Document, key: &str) -> Result<BTreeMap<String, String>, String> {
    let what = "source.json";
    match document.get(key) {
        None | Some(Value::Null) => Ok(BTreeMap::new()),
        Some(Value::Object(fields)) => fields
            .iter()
            .map(|(name, integrity)| {
                if !names::valid_relative(name) {
                    return Err(format!("{what}: {name} is not a valid file name in {key}"));
                }
                match integrity {
                    Value::String(value) if valid_integrity(value) => {
                        Ok((name.clone(), value.clone()))
                    }
                    _ => Err(format!(
                        "{what}: the integrity of {name} in {key} must look like sha256-<base64>"
                    )),
                }
            })
            .collect(),
        Some(_) => Err(format!(
            "{what}: {key} must map file names to their integrity"
        )),
    }
}

fn required<'a>(document: &'a Document, key: &str, kind: &str) -> Result<&'a str, String> {
    match text(document, key, "source.json")? {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(format!(
            "source.json: {key} is required for the {kind} type"
        )),
    }
}

pub fn validate_source(bytes: &[u8]) -> Result<Source, String> {
    let what = "source.json";
    let document = parse(bytes, what)?;
    let kind = text(&document, "type", what)?.unwrap_or("archive");
    if !SOURCE_TYPES.contains(&kind) {
        return Err(format!(
            "{what}: type {kind} is not one of {}",
            SOURCE_TYPES.join(", ")
        ));
    }
    match document.get("patch_strip") {
        None | Some(Value::Null) => {}
        Some(value) if value.as_u64().is_some() => {}
        Some(_) => {
            return Err(format!(
                "{what}: patch_strip must be a non-negative integer"
            ));
        }
    }
    text(&document, "strip_prefix", what)?;
    match kind {
        "archive" => {
            let address = required(&document, "url", kind)?;
            if url::Url::parse(address).is_err() {
                return Err(format!("{what}: {address} is not a URL"));
            }
            for mirror in texts(&document, "mirror_urls", what)? {
                if url::Url::parse(&mirror).is_err() {
                    return Err(format!("{what}: {mirror} in mirror_urls is not a URL"));
                }
            }
            if !valid_integrity(required(&document, "integrity", kind)?) {
                return Err(format!(
                    "{what}: the integrity must look like sha256-<base64>"
                ));
            }
            text(&document, "archive_type", what)?;
        }
        "git_repository" => {
            required(&document, "remote", kind)?;
            let commit = text(&document, "commit", what)?.is_some_and(|value| !value.is_empty());
            let tag = text(&document, "tag", what)?.is_some_and(|value| !value.is_empty());
            if !commit && !tag {
                return Err(format!("{what}: a git_repository source needs a commit"));
            }
        }
        _ => {
            required(&document, "path", kind)?;
        }
    }
    let (patches, overlay) = match kind {
        "archive" => (listed(&document, "patches")?, listed(&document, "overlay")?),
        "git_repository" => (listed(&document, "patches")?, BTreeMap::new()),
        _ => (BTreeMap::new(), BTreeMap::new()),
    };
    Ok(Source { patches, overlay })
}

fn valid_integrity(value: &str) -> bool {
    let Some((algorithm, digest)) = value.split_once('-') else {
        return false;
    };
    matches!(algorithm, "sha1" | "sha256" | "sha384" | "sha512")
        && !digest.is_empty()
        && STANDARD.decode(digest).is_ok()
}

#[derive(Clone, Copy, Debug)]
pub struct Digests<'a> {
    pub sha1: &'a str,
    pub sha256: &'a str,
    pub sha512: &'a str,
}

pub fn integrity_matches(integrity: &str, digests: Digests<'_>) -> Option<bool> {
    let (algorithm, encoded) = integrity.split_once('-')?;
    let expected = hex::encode(STANDARD.decode(encoded).ok()?);
    let stored = match algorithm {
        "sha1" => digests.sha1,
        "sha256" => digests.sha256,
        "sha512" => digests.sha512,
        _ => return None,
    };
    Some(stored.eq_ignore_ascii_case(&expected))
}

impl Source {
    pub fn integrity_of(&self, entry: &names::Entry) -> Option<&str> {
        match entry {
            names::Entry::Patch(name) => self.patches.get(name),
            names::Entry::Overlay(name) => self.overlay.get(name),
            _ => None,
        }
        .map(String::as_str)
    }

    pub fn entries(&self) -> Vec<(names::Entry, &str)> {
        self.patches
            .iter()
            .map(|(name, integrity)| (names::Entry::Patch(name.clone()), integrity.as_str()))
            .chain(
                self.overlay.iter().map(|(name, integrity)| {
                    (names::Entry::Overlay(name.clone()), integrity.as_str())
                }),
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use sha2::{Digest, Sha256, Sha512};

    use super::*;

    fn object(value: Value) -> Document {
        match value {
            Value::Object(document) => document,
            other => panic!("not an object: {other}"),
        }
    }

    fn sri(data: &[u8]) -> String {
        format!("sha256-{}", STANDARD.encode(Sha256::digest(data)))
    }

    #[test]
    fn registry_documents_hold_mirrors_and_a_base_path() {
        assert!(
            validate_registry(br#"{"mirrors": ["https://m.example/x"], "module_base_path": "m"}"#)
                .is_ok()
        );
        assert!(validate_registry(b"{}").is_ok());
        for bad in [
            &b"[]"[..],
            b"not json",
            br#"{"mirrors": "x"}"#,
            br#"{"mirrors": [1]}"#,
            br#"{"mirrors": [""]}"#,
            br#"{"module_base_path": 3}"#,
        ] {
            assert!(
                validate_registry(bad).is_err(),
                "{}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn registries_of_members_merge_mirrors_in_order() {
        let merged = merge_registries(&[
            object(json!({"mirrors": ["https://a/", "https://b/"]})),
            object(json!({"mirrors": ["https://b/", "https://c/"], "module_base_path": "base"})),
            object(json!({"module_base_path": "other"})),
        ]);
        assert_eq!(
            Value::Object(merged),
            json!({"mirrors": ["https://a/", "https://b/", "https://c/"], "module_base_path": "base"})
        );
        assert_eq!(Value::Object(merge_registries(&[])), json!({"mirrors": []}));
    }

    #[test]
    fn metadata_documents_are_checked_field_by_field() {
        let good = br#"{
            "homepage": "https://example.org",
            "maintainers": [{"name": "A", "github": "a"}],
            "repository": ["github:a/b"],
            "versions": ["1.0"],
            "yanked_versions": {"1.0": "broken"},
            "deprecated": "use b"
        }"#;
        assert!(validate_metadata(good).is_ok());
        for bad in [
            &br#"{"homepage": 1}"#[..],
            br#"{"maintainers": ["a"]}"#,
            br#"{"maintainers": {}}"#,
            br#"{"repository": "x"}"#,
            br#"{"versions": [1]}"#,
            br#"{"yanked_versions": []}"#,
            br#"{"yanked_versions": {"1.0": 3}}"#,
            br#"{"yanked_versions": {"1/0": "x"}}"#,
            br#"{"deprecated": true}"#,
        ] {
            assert!(
                validate_metadata(bad).is_err(),
                "{}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn generated_metadata_lists_the_stored_versions_in_order() {
        let stored = object(json!({
            "homepage": "https://example.org",
            "versions": ["9.9"],
            "yanked_versions": {"1.0.0": "bad"},
        }));
        let built = build_metadata(
            Some(stored),
            vec!["1.10.0".to_owned(), "1.2.0".to_owned(), "1.0.0".to_owned()],
        );
        assert_eq!(
            Value::Object(built),
            json!({
                "homepage": "https://example.org",
                "versions": ["1.0.0", "1.2.0", "1.10.0"],
                "yanked_versions": {"1.0.0": "bad"},
            })
        );
        assert_eq!(
            Value::Object(build_metadata(None, vec!["1.0".to_owned()])),
            json!({"versions": ["1.0"], "yanked_versions": {}})
        );
    }

    #[test]
    fn metadata_of_members_merges_versions_and_yanks_and_keeps_the_first_fields() {
        let merged = merge_metadata(vec![
            object(json!({
                "homepage": "",
                "maintainers": [],
                "versions": ["1.0", "2.0"],
                "yanked_versions": {"1.0": "first"},
            })),
            object(json!({
                "homepage": "https://second",
                "maintainers": [{"github": "x"}],
                "versions": ["1.5", "2.0"],
                "yanked_versions": {"1.0": "second", "1.5": "other"},
            })),
            object(json!({"homepage": "https://third", "deprecated": "old"})),
        ]);
        assert_eq!(
            Value::Object(merged),
            json!({
                "homepage": "https://second",
                "maintainers": [{"github": "x"}],
                "deprecated": "old",
                "versions": ["1.0", "1.5", "2.0"],
                "yanked_versions": {"1.0": "first", "1.5": "other"},
            })
        );
    }

    #[test]
    fn archive_sources_need_a_url_and_name_valid_patches() {
        let source = validate_source(
            br#"{
                "url": "https://example.org/a.tar.gz",
                "integrity": "sha256-vCg8381SalLDIBJ5zaS8KYZS76iYsQtNsIN9xRZSdW8=",
                "strip_prefix": "a-1.0",
                "patch_strip": 1,
                "patches": {"fix.patch": "sha256-vCg8381SalLDIBJ5zaS8KYZS76iYsQtNsIN9xRZSdW8="},
                "overlay": {"BUILD.bazel": "sha256-vCg8381SalLDIBJ5zaS8KYZS76iYsQtNsIN9xRZSdW8="}
            }"#,
        )
        .unwrap();
        assert_eq!(source.patches.len(), 1);
        assert_eq!(source.overlay.len(), 1);
        assert_eq!(source.entries().len(), 2);
        for bad in [
            &br#"{}"#[..],
            br#"{"url": 3}"#,
            br#"{"url": "https://e/a"}"#,
            br#"{"url": "no url", "integrity": "sha256-vCg8381SalLDIBJ5zaS8KYZS76iYsQtNsIN9xRZSdW8="}"#,
            br#"{"url": "https://e/a", "integrity": "md5-AAAA"}"#,
            br#"{"url": "https://e/a", "integrity": "sha256-%%%"}"#,
            br#"{"url": "https://e/a", "mirror_urls": ["x"]}"#,
            br#"{"url": "https://e/a", "patch_strip": -1}"#,
            br#"{"url": "https://e/a", "patch_strip": "1"}"#,
            br#"{"url": "https://e/a", "patches": []}"#,
            br#"{"url": "https://e/a", "patches": {"../x": "sha256-AAAA"}}"#,
            br#"{"url": "https://e/a", "patches": {"x": "nope"}}"#,
            br#"{"type": "zip", "url": "https://e/a"}"#,
            b"[]",
        ] {
            assert!(
                validate_source(bad).is_err(),
                "{}",
                String::from_utf8_lossy(bad)
            );
        }
    }

    #[test]
    fn git_and_local_sources_have_their_own_fields() {
        assert!(
            validate_source(
                br#"{"type": "git_repository", "remote": "https://e/r.git", "commit": "abc"}"#
            )
            .is_ok()
        );
        assert!(
            validate_source(br#"{"type": "git_repository", "remote": "git@e:r.git", "tag": "v1"}"#)
                .is_ok()
        );
        assert!(
            validate_source(br#"{"type": "git_repository", "remote": "https://e/r.git"}"#).is_err()
        );
        assert!(validate_source(br#"{"type": "git_repository", "commit": "abc"}"#).is_err());
        assert!(validate_source(br#"{"type": "local_path", "path": "../m"}"#).is_ok());
        assert!(validate_source(br#"{"type": "local_path"}"#).is_err());
        assert!(
            validate_source(
                br#"{"type": "archive", "url": "https://e/a", "integrity": "sha256-vCg8381SalLDIBJ5zaS8KYZS76iYsQtNsIN9xRZSdW8="}"#
            )
            .is_ok()
        );
        assert!(validate_source(br#"{"type": "archive", "url": "https://e/a"}"#).is_err());
    }

    #[test]
    fn integrity_is_compared_with_the_stored_digests() {
        let data = b"--- a\n+++ b\n";
        let sha256 = hex::encode(Sha256::digest(data));
        let sha512 = hex::encode(Sha512::digest(data));
        let digests = Digests {
            sha1: "",
            sha256: &sha256,
            sha512: &sha512,
        };
        assert_eq!(integrity_matches(&sri(data), digests), Some(true));
        assert_eq!(integrity_matches(&sri(b"other"), digests), Some(false));
        let long = format!("sha512-{}", STANDARD.encode(Sha512::digest(data)));
        assert_eq!(integrity_matches(&long, digests), Some(true));
        assert_eq!(integrity_matches("sha384-AAAA", digests), None);
        assert_eq!(integrity_matches("garbage", digests), None);
    }
}
