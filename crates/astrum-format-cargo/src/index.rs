use std::collections::{BTreeMap, HashSet};

use serde::Deserialize;
use serde_json::{Map, Value, json};

pub const MAX_NAME: usize = 64;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PublishError {
    #[error("the publish request is cut short")]
    Truncated,
    #[error("the publish metadata is not valid JSON: {0}")]
    Metadata(String),
    #[error(
        "`{0}` is not a valid crate name: use letters, digits, `-` and `_`, start with a letter, at most 64 characters"
    )]
    Name(String),
    #[error("`{0}` is not a valid semantic version")]
    Version(String),
    #[error("the .crate file is not a gzip archive")]
    NotGzip,
}

pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    name.len() <= MAX_NAME
        && first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

pub fn prefix(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    match lower.len() {
        0 => String::new(),
        1 => "1".to_owned(),
        2 => "2".to_owned(),
        3 => format!("3/{}", &lower[..1]),
        _ => format!("{}/{}", &lower[..2], &lower[2..4]),
    }
}

pub fn index_path(name: &str) -> String {
    format!("/{}/{}", prefix(name), name.to_ascii_lowercase())
}

pub fn name_of_index_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let name = match parts.as_slice() {
        [size, name] if (*size == "1" || *size == "2") && name.len().to_string() == *size => name,
        ["3", first, name] if name.len() == 3 && name.get(..1) == Some(*first) => name,
        [a, b, name]
            if name.len() >= 4 && name.get(..2) == Some(*a) && name.get(2..4) == Some(*b) =>
        {
            name
        }
        _ => return None,
    };
    (valid_name(name) && name.to_ascii_lowercase() == *name).then(|| (*name).to_owned())
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PublishDependency {
    pub name: String,
    #[serde(default)]
    pub version_req: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "yes")]
    pub default_features: bool,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub registry: Option<String>,
    #[serde(default)]
    pub explicit_name_in_toml: Option<String>,
}

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub vers: String,
    #[serde(default)]
    pub deps: Vec<PublishDependency>,
    #[serde(default)]
    pub features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub readme: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub license_file: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub links: Option<String>,
    #[serde(default)]
    pub rust_version: Option<String>,
}

pub struct Publish {
    pub metadata: Metadata,
    pub archive: bytes::Bytes,
}

fn length_at(body: &[u8], at: usize) -> Result<usize, PublishError> {
    let bytes: [u8; 4] = body
        .get(at..at + 4)
        .and_then(|slice| slice.try_into().ok())
        .ok_or(PublishError::Truncated)?;
    Ok(u32::from_le_bytes(bytes) as usize)
}

pub fn parse_publish(body: bytes::Bytes) -> Result<Publish, PublishError> {
    let json_length = length_at(&body, 0)?;
    let json_end = 4usize
        .checked_add(json_length)
        .ok_or(PublishError::Truncated)?;
    let json = body.get(4..json_end).ok_or(PublishError::Truncated)?;
    let metadata: Metadata =
        serde_json::from_slice(json).map_err(|error| PublishError::Metadata(error.to_string()))?;
    let crate_length = length_at(&body, json_end)?;
    let start = json_end + 4;
    let end = start
        .checked_add(crate_length)
        .ok_or(PublishError::Truncated)?;
    if end > body.len() {
        return Err(PublishError::Truncated);
    }
    if !valid_name(&metadata.name) {
        return Err(PublishError::Name(metadata.name));
    }
    if !valid_version(&metadata.vers) {
        return Err(PublishError::Version(metadata.vers));
    }
    let archive = body.slice(start..end);
    if archive.len() < 2 || archive[0] != 0x1f || archive[1] != 0x8b {
        return Err(PublishError::NotGzip);
    }
    Ok(Publish { metadata, archive })
}

fn uses_new_syntax(values: &[String]) -> bool {
    values
        .iter()
        .any(|value| value.starts_with("dep:") || value.contains("?/"))
}

pub fn entry(metadata: &Metadata, checksum: &str) -> Value {
    let deps: Vec<Value> = metadata
        .deps
        .iter()
        .map(|dependency| {
            let mut out = Map::new();
            let (name, package) = match &dependency.explicit_name_in_toml {
                Some(renamed) if !renamed.is_empty() => {
                    (renamed.clone(), Some(dependency.name.clone()))
                }
                _ => (dependency.name.clone(), None),
            };
            out.insert("name".to_owned(), json!(name));
            out.insert("req".to_owned(), json!(dependency.version_req));
            out.insert("features".to_owned(), json!(dependency.features));
            out.insert("optional".to_owned(), json!(dependency.optional));
            out.insert(
                "default_features".to_owned(),
                json!(dependency.default_features),
            );
            out.insert("target".to_owned(), json!(dependency.target));
            out.insert(
                "kind".to_owned(),
                json!(
                    dependency
                        .kind
                        .clone()
                        .unwrap_or_else(|| "normal".to_owned())
                ),
            );
            out.insert("registry".to_owned(), json!(dependency.registry));
            if let Some(package) = package {
                out.insert("package".to_owned(), json!(package));
            }
            Value::Object(out)
        })
        .collect();
    let (features2, features): (BTreeMap<_, _>, BTreeMap<_, _>) = metadata
        .features
        .iter()
        .map(|(key, values)| (key.clone(), values.clone()))
        .partition(|(_, values)| uses_new_syntax(values));
    let mut out = Map::new();
    out.insert("name".to_owned(), json!(metadata.name));
    out.insert("vers".to_owned(), json!(metadata.vers));
    out.insert("deps".to_owned(), Value::Array(deps));
    out.insert("cksum".to_owned(), json!(checksum));
    out.insert("features".to_owned(), json!(features));
    out.insert("yanked".to_owned(), json!(false));
    if let Some(links) = metadata.links.as_ref().filter(|links| !links.is_empty()) {
        out.insert("links".to_owned(), json!(links));
    }
    if !features2.is_empty() {
        out.insert("features2".to_owned(), json!(features2));
        out.insert("v".to_owned(), json!(2));
    }
    if let Some(rust_version) = metadata
        .rust_version
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        out.insert("rust_version".to_owned(), json!(rust_version));
    }
    Value::Object(out)
}

pub fn render(entries: &[Value]) -> String {
    let mut text = String::new();
    for entry in entries {
        text.push_str(&entry.to_string());
        text.push('\n');
    }
    text
}

pub fn parse(text: &str) -> Vec<Value> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|entry| entry.get("vers").and_then(Value::as_str).is_some())
        .collect()
}

pub fn merge(listings: Vec<Vec<Value>>) -> Vec<Value> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    for listing in listings {
        for entry in listing {
            let version = entry
                .get("vers")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if seen.insert(version) {
                merged.push(entry);
            }
        }
    }
    merged
}

pub fn checksum_of(entries: &[Value], version: &str) -> Option<String> {
    entries
        .iter()
        .find(|entry| entry.get("vers").and_then(Value::as_str) == Some(version))
        .and_then(|entry| entry.get("cksum").and_then(Value::as_str))
        .map(str::to_owned)
}

pub fn download_url(template: &str, name: &str, version: &str, checksum: Option<&str>) -> String {
    let markers = [
        "{crate}",
        "{version}",
        "{prefix}",
        "{lowerprefix}",
        "{sha256-checksum}",
    ];
    if !markers.iter().any(|marker| template.contains(marker)) {
        return format!(
            "{}/{name}/{version}/download",
            template.trim_end_matches('/')
        );
    }
    let original_prefix = match name.len() {
        1 => "1".to_owned(),
        2 => "2".to_owned(),
        3 => format!("3/{}", &name[..1]),
        _ => format!("{}/{}", &name[..2], &name[2..4]),
    };
    template
        .replace("{crate}", name)
        .replace("{version}", version)
        .replace("{lowerprefix}", &prefix(name))
        .replace("{prefix}", &original_prefix)
        .replace("{sha256-checksum}", checksum.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn publish_body(metadata: &Value, archive: &[u8]) -> bytes::Bytes {
        let json = serde_json::to_vec(metadata).unwrap();
        let mut body = Vec::new();
        body.extend_from_slice(&(json.len() as u32).to_le_bytes());
        body.extend_from_slice(&json);
        body.extend_from_slice(&(archive.len() as u32).to_le_bytes());
        body.extend_from_slice(archive);
        bytes::Bytes::from(body)
    }

    #[test]
    fn index_paths_follow_the_name_length() {
        assert_eq!(index_path("a"), "/1/a");
        assert_eq!(index_path("ab"), "/2/ab");
        assert_eq!(index_path("abc"), "/3/a/abc");
        assert_eq!(index_path("Serde"), "/se/rd/serde");
        assert_eq!(name_of_index_path("/se/rd/serde").as_deref(), Some("serde"));
        assert_eq!(name_of_index_path("/3/a/abc").as_deref(), Some("abc"));
        assert_eq!(name_of_index_path("/1/a").as_deref(), Some("a"));
        assert_eq!(name_of_index_path("/2/ab").as_deref(), Some("ab"));
        for bad in [
            "/se/rd/Serde",
            "/se/xx/serde",
            "/3/b/abc",
            "/1/ab",
            "/config.json",
            "/a/b/c/d",
        ] {
            assert_eq!(name_of_index_path(bad), None, "{bad}");
        }
    }

    #[test]
    fn names_and_versions_are_checked() {
        assert!(valid_name("astrum-demo_1"));
        assert!(!valid_name("1demo"));
        assert!(!valid_name("demo.x"));
        assert!(!valid_name(&"a".repeat(65)));
        assert!(valid_version("1.0.0-beta.1+build"));
        assert!(!valid_version("1.0"));
    }

    #[test]
    fn publish_bodies_give_the_metadata_and_the_archive() {
        let metadata = json!({
            "name": "demo", "vers": "0.1.0",
            "deps": [{"name": "serde", "version_req": "^1", "features": ["derive"], "optional": false,
                      "default_features": true, "target": null, "kind": "normal",
                      "registry": "https://github.com/rust-lang/crates.io-index",
                      "explicit_name_in_toml": "serde_renamed"}],
            "features": {"default": ["std"], "std": [], "json": ["dep:serde_json"]},
            "rust_version": "1.70"
        });
        let parsed = parse_publish(publish_body(&metadata, &[0x1f, 0x8b, 1, 2])).unwrap();
        assert_eq!(parsed.metadata.name, "demo");
        assert_eq!(parsed.archive.len(), 4);
        let line = entry(&parsed.metadata, "abc");
        assert_eq!(line["deps"][0]["name"], "serde_renamed");
        assert_eq!(line["deps"][0]["package"], "serde");
        assert_eq!(line["deps"][0]["req"], "^1");
        assert_eq!(line["features"], json!({"default": ["std"], "std": []}));
        assert_eq!(line["features2"], json!({"json": ["dep:serde_json"]}));
        assert_eq!(line["v"], 2);
        assert_eq!(line["rust_version"], "1.70");
        assert_eq!(line["yanked"], false);
        assert_eq!(
            parse_publish(publish_body(
                &json!({"name": "1x", "vers": "1.0.0"}),
                &[0x1f, 0x8b]
            ))
            .err(),
            Some(PublishError::Name("1x".to_owned()))
        );
        assert_eq!(
            parse_publish(publish_body(&json!({"name": "x", "vers": "1.0.0"}), b"PK")).err(),
            Some(PublishError::NotGzip)
        );
        let mut cut = publish_body(&metadata, &[0x1f, 0x8b, 1, 2]).to_vec();
        cut.truncate(cut.len() - 1);
        assert_eq!(
            parse_publish(bytes::Bytes::from(cut)).err(),
            Some(PublishError::Truncated)
        );
    }

    #[test]
    fn listings_merge_by_version_and_download_templates_expand() {
        let first = parse("{\"name\":\"a\",\"vers\":\"1.0.0\",\"cksum\":\"x\"}\n");
        let second = parse(
            "{\"name\":\"a\",\"vers\":\"1.0.0\",\"cksum\":\"y\"}\n{\"name\":\"a\",\"vers\":\"2.0.0\",\"cksum\":\"z\"}\nnot json\n",
        );
        let merged = merge(vec![first, second]);
        assert_eq!(merged.len(), 2);
        assert_eq!(checksum_of(&merged, "1.0.0").as_deref(), Some("x"));
        assert_eq!(
            download_url("https://static.crates.io/crates", "serde", "1.0.0", None),
            "https://static.crates.io/crates/serde/1.0.0/download"
        );
        assert_eq!(
            download_url(
                "https://h/{lowerprefix}/{crate}/{crate}-{version}.crate?c={sha256-checksum}",
                "Serde",
                "1.0.0",
                Some("ab")
            ),
            "https://h/se/rd/Serde/Serde-1.0.0.crate?c=ab"
        );
    }
}
