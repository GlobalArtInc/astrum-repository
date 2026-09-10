use std::io::Read;

use serde_json::{Map, Value};

use crate::version;

pub const MAX_PUBSPEC: u64 = 1024 * 1024;
const MAX_INFLATED: u64 = 1024 * 1024 * 1024;
const MAX_NAME: usize = 64;

const RESERVED: &[&str] = &[
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "for",
    "function",
    "get",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "library",
    "mixin",
    "new",
    "null",
    "operator",
    "part",
    "rethrow",
    "return",
    "set",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("{0}")]
    Invalid(String),
    #[error("the archive cannot be read: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pubspec {
    pub name: String,
    pub version: String,
    pub json: Value,
}

impl Pubspec {
    pub fn description(&self) -> Option<&str> {
        self.json.get("description").and_then(Value::as_str)
    }

    pub fn sdk(&self) -> Option<&str> {
        self.json
            .get("environment")
            .and_then(|environment| environment.get("sdk"))
            .and_then(Value::as_str)
    }
}

pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    name.len() <= MAX_NAME
        && chars
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|next| next.is_ascii_alphanumeric() || next == '_')
}

pub fn valid_new_name(name: &str) -> Result<(), String> {
    if !valid_name(name) {
        return Err(format!(
            "\"{name}\" is not a valid package name: use letters, digits and underscores, start with a letter or an underscore, and stay within {MAX_NAME} characters"
        ));
    }
    if name.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(format!(
            "\"{name}\" is not a valid package name: package names are lower case"
        ));
    }
    if RESERVED.contains(&name) {
        return Err(format!(
            "\"{name}\" is not a valid package name: it is a reserved word of Dart"
        ));
    }
    Ok(())
}

fn key_text(key: serde_norway::Value) -> String {
    match key {
        serde_norway::Value::String(text) => text,
        serde_norway::Value::Null => "null".to_owned(),
        serde_norway::Value::Bool(value) => value.to_string(),
        serde_norway::Value::Number(number) => number.to_string(),
        other => serde_json::to_string(&to_json(other)).unwrap_or_default(),
    }
}

pub fn to_json(value: serde_norway::Value) -> Value {
    match value {
        serde_norway::Value::Null => Value::Null,
        serde_norway::Value::Bool(value) => Value::Bool(value),
        serde_norway::Value::Number(number) => {
            if let Some(value) = number.as_u64() {
                Value::from(value)
            } else if let Some(value) = number.as_i64() {
                Value::from(value)
            } else {
                number
                    .as_f64()
                    .and_then(serde_json::Number::from_f64)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(number.to_string()))
            }
        }
        serde_norway::Value::String(text) => Value::String(text),
        serde_norway::Value::Sequence(items) => {
            Value::Array(items.into_iter().map(to_json).collect())
        }
        serde_norway::Value::Mapping(mapping) => {
            let mut map = Map::new();
            for (key, value) in mapping {
                map.insert(key_text(key), to_json(value));
            }
            Value::Object(map)
        }
        serde_norway::Value::Tagged(tagged) => to_json(tagged.value),
    }
}

pub fn parse_pubspec(text: &[u8]) -> Result<Pubspec, ArchiveError> {
    let text = std::str::from_utf8(text)
        .map_err(|_| ArchiveError::Invalid("pubspec.yaml is not UTF-8 text".to_owned()))?;
    let yaml: serde_norway::Value = serde_norway::from_str(text)
        .map_err(|error| ArchiveError::Invalid(format!("pubspec.yaml cannot be read: {error}")))?;
    let json = to_json(yaml);
    if !json.is_object() {
        return Err(ArchiveError::Invalid(
            "pubspec.yaml does not hold a mapping".to_owned(),
        ));
    }
    let name = match json.get("name") {
        Some(Value::String(name)) => name.clone(),
        _ => {
            return Err(ArchiveError::Invalid(
                "pubspec.yaml has no \"name\" field".to_owned(),
            ));
        }
    };
    let version = match json.get("version") {
        Some(Value::String(version)) => version.clone(),
        Some(Value::Number(number)) => number.to_string(),
        _ => {
            return Err(ArchiveError::Invalid(
                "pubspec.yaml has no \"version\" field".to_owned(),
            ));
        }
    };
    if !valid_name(&name) {
        return Err(ArchiveError::Invalid(format!(
            "\"{name}\" in pubspec.yaml is not a valid package name"
        )));
    }
    if !version::valid(&version) {
        return Err(ArchiveError::Invalid(format!(
            "\"{version}\" in pubspec.yaml is not a valid version; use a semantic version such as 1.2.3"
        )));
    }
    Ok(Pubspec {
        name,
        version,
        json,
    })
}

fn is_pubspec(path: &str) -> bool {
    path.trim_start_matches("./") == "pubspec.yaml"
}

pub fn pubspec_yaml(archive: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    if !archive.starts_with(&[0x1f, 0x8b]) {
        return Err(ArchiveError::Invalid(
            "the package is not a gzip-compressed tar archive".to_owned(),
        ));
    }
    let inflated = flate2::read::GzDecoder::new(archive).take(MAX_INFLATED);
    let mut tar = tar::Archive::new(inflated);
    for entry in tar.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path()?.to_string_lossy().replace('\\', "/");
        if !is_pubspec(&path) {
            continue;
        }
        if entry.size() > MAX_PUBSPEC {
            return Err(ArchiveError::Invalid(format!(
                "pubspec.yaml is larger than {MAX_PUBSPEC} bytes"
            )));
        }
        let mut data = Vec::new();
        (&mut entry).take(MAX_PUBSPEC).read_to_end(&mut data)?;
        return Ok(data);
    }
    Err(ArchiveError::Invalid(
        "the archive holds no pubspec.yaml at its top".to_owned(),
    ))
}

pub fn read(archive: &[u8]) -> Result<Pubspec, ArchiveError> {
    parse_pubspec(&pubspec_yaml(archive)?)
}

pub fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (name, data) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, *data)
            .expect("a test archive can be written");
    }
    builder
        .into_inner()
        .and_then(|encoder| encoder.finish())
        .expect("a test archive can be finished")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_pubspec_at_the_top_is_read_and_turned_into_json() {
        let archive = build(&[
            ("lib/hello.dart", b"void main() {}\n"),
            (
                "pubspec.yaml",
                b"name: hello\nversion: 1.2.0-dev.1\ndescription: Says hello.\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\ndependencies:\n  path: ^1.8.0\n  other:\n    hosted: https://example.org/repository/pub\n    version: ^2.0.0\nflutter:\n  plugin:\n    platforms:\n      android: {package: a.b, pluginClass: C}\n",
            ),
        ]);
        let pubspec = read(&archive).unwrap();
        assert_eq!(pubspec.name, "hello");
        assert_eq!(pubspec.version, "1.2.0-dev.1");
        assert_eq!(pubspec.description(), Some("Says hello."));
        assert_eq!(pubspec.sdk(), Some(">=3.0.0 <4.0.0"));
        assert_eq!(
            pubspec.json["dependencies"],
            json!({"path": "^1.8.0", "other": {"hosted": "https://example.org/repository/pub", "version": "^2.0.0"}})
        );
        assert_eq!(
            pubspec.json["flutter"]["plugin"]["platforms"]["android"]["pluginClass"],
            "C"
        );
    }

    #[test]
    fn packages_without_a_valid_pubspec_are_refused() {
        let nested = build(&[("hello/pubspec.yaml", b"name: hello\nversion: 1.0.0\n")]);
        assert!(
            read(&nested)
                .unwrap_err()
                .to_string()
                .contains("no pubspec.yaml")
        );
        assert!(
            read(b"PK\x03\x04")
                .unwrap_err()
                .to_string()
                .contains("gzip")
        );
        for (text, message) in [
            ("version: 1.0.0\n", "no \"name\""),
            ("name: hello\n", "no \"version\""),
            (
                "name: hello-world\nversion: 1.0.0\n",
                "not a valid package name",
            ),
            ("name: hello\nversion: 1.0\n", "not a valid version"),
            ("- a\n- b\n", "does not hold a mapping"),
            ("name: [\n", "cannot be read"),
        ] {
            let error = parse_pubspec(text.as_bytes()).unwrap_err().to_string();
            assert!(error.contains(message), "{text}: {error}");
        }
        let dotted = build(&[("./pubspec.yaml", b"name: hello\nversion: 1.0.0\n")]);
        assert_eq!(read(&dotted).unwrap().name, "hello");
    }

    #[test]
    fn new_package_names_follow_the_rules_of_the_dart_client() {
        assert!(valid_new_name("http_client").is_ok());
        assert!(valid_new_name("_private").is_ok());
        assert!(valid_new_name("Hello").unwrap_err().contains("lower case"));
        assert!(valid_new_name("class").unwrap_err().contains("reserved"));
        assert!(valid_new_name("1st").is_err());
        assert!(valid_new_name(&"a".repeat(65)).is_err());
        assert!(valid_name("Hello"));
    }

    #[test]
    fn yaml_keys_that_are_not_text_become_text() {
        let pubspec =
            parse_pubspec(b"name: x\nversion: 1.0.0\nextra:\n  1: one\n  true: yes\n").unwrap();
        assert_eq!(pubspec.json["extra"], json!({"1": "one", "true": "yes"}));
    }
}
