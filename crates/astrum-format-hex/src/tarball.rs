use std::io::Read;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::terms::{self, Term};

pub const FORMAT_VERSION: &str = "3";
pub const MAX_TARBALL: u64 = 64 * 1024 * 1024;
const MAX_METADATA: u64 = 1024 * 1024;
const MAX_NAME: usize = 255;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TarballError {
    #[error("the package tarball is not a tar file: {0}")]
    NotTar(String),
    #[error("the package tarball lacks {0}")]
    Missing(&'static str),
    #[error("the package tarball has version {0}; only version 3 is supported")]
    Version(String),
    #[error("CHECKSUM does not match the contents of the package tarball")]
    Checksum,
    #[error("metadata.config cannot be read: {0}")]
    Metadata(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requirement {
    pub name: String,
    pub requirement: String,
    pub optional: bool,
    pub app: Option<String>,
    pub repository: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub app: Option<String>,
    pub description: Option<String>,
    pub licenses: Vec<String>,
    pub build_tools: Vec<String>,
    pub elixir: Option<String>,
    pub links: Vec<(String, String)>,
    pub requirements: Vec<Requirement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tarball {
    pub metadata: Metadata,
    pub inner_checksum: [u8; 32],
    pub outer_checksum: [u8; 32],
}

pub fn valid_name(name: &str) -> bool {
    name.len() <= MAX_NAME
        && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    version.len() <= MAX_NAME && semver::Version::parse(version).is_ok()
}

pub fn file_name(name: &str, version: &str) -> String {
    format!("{name}-{version}.tar")
}

pub fn read(bytes: &[u8]) -> Result<Tarball, TarballError> {
    let mut version = None;
    let mut checksum = None;
    let mut metadata = None;
    let mut contents = None;
    let mut archive = tar::Archive::new(bytes);
    let entries = archive
        .entries()
        .map_err(|error| TarballError::NotTar(error.to_string()))?;
    for entry in entries {
        let mut entry = entry.map_err(|error| TarballError::NotTar(error.to_string()))?;
        let path = entry
            .path()
            .map_err(|error| TarballError::NotTar(error.to_string()))?
            .to_string_lossy()
            .into_owned();
        let name = path.trim_start_matches("./");
        let slot = match name {
            "VERSION" => &mut version,
            "CHECKSUM" => &mut checksum,
            "metadata.config" => &mut metadata,
            "contents.tar.gz" => &mut contents,
            _ => continue,
        };
        let limit = match name {
            "contents.tar.gz" => MAX_TARBALL,
            _ => MAX_METADATA,
        };
        let mut data = Vec::new();
        (&mut entry)
            .take(limit + 1)
            .read_to_end(&mut data)
            .map_err(|error| TarballError::NotTar(error.to_string()))?;
        if data.len() as u64 > limit {
            return Err(TarballError::NotTar(format!("{path} is too large")));
        }
        *slot = Some(data);
    }
    let version = version.ok_or(TarballError::Missing("VERSION"))?;
    let metadata = metadata.ok_or(TarballError::Missing("metadata.config"))?;
    let contents = contents.ok_or(TarballError::Missing("contents.tar.gz"))?;
    let version_text = String::from_utf8_lossy(&version).trim().to_owned();
    if version_text != FORMAT_VERSION {
        return Err(TarballError::Version(version_text));
    }
    let mut digest = Sha256::new();
    digest.update(&version);
    digest.update(&metadata);
    digest.update(&contents);
    let inner_checksum: [u8; 32] = digest.finalize().into();
    if let Some(checksum) = checksum {
        let given = String::from_utf8_lossy(&checksum)
            .trim()
            .to_ascii_lowercase();
        if given != hex::encode(inner_checksum) {
            return Err(TarballError::Checksum);
        }
    }
    let text = String::from_utf8(metadata)
        .map_err(|_| TarballError::Metadata("it is not UTF-8".to_owned()))?;
    Ok(Tarball {
        metadata: interpret(&text)?,
        inner_checksum,
        outer_checksum: Sha256::digest(bytes).into(),
    })
}

fn text_of(term: Option<&Term>) -> Option<String> {
    term.filter(|term| !term.is_undefined())
        .and_then(Term::text)
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn texts_of(term: Option<&Term>) -> Vec<String> {
    term.and_then(Term::items)
        .unwrap_or_default()
        .iter()
        .filter_map(|item| text_of(Some(item)))
        .collect()
}

fn requirement(name: Option<String>, fields: &Term) -> Result<Requirement, TarballError> {
    let name = name
        .or_else(|| text_of(fields.get("name").as_ref()))
        .ok_or_else(|| TarballError::Metadata("a requirement has no name".to_owned()))?;
    if !valid_name(&name) && !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(TarballError::Metadata(format!(
            "the requirement {name} is not a package name"
        )));
    }
    let requirement = text_of(fields.get("requirement").as_ref()).ok_or_else(|| {
        TarballError::Metadata(format!(
            "the requirement on {name} has no version requirement"
        ))
    })?;
    Ok(Requirement {
        app: text_of(fields.get("app").as_ref()).filter(|app| *app != name),
        optional: fields
            .get("optional")
            .and_then(|optional| optional.boolean())
            .unwrap_or(false),
        repository: text_of(fields.get("repository").as_ref()),
        requirement,
        name,
    })
}

fn requirements(term: Option<&Term>) -> Result<Vec<Requirement>, TarballError> {
    let Some(term) = term else {
        return Ok(Vec::new());
    };
    let items = term.items().unwrap_or_default();
    let mut found = Vec::new();
    for item in items {
        match &item {
            Term::Tuple(parts) if parts.len() == 2 && parts[0].text().is_some() => {
                found.push(requirement(parts[0].text(), &parts[1])?);
            }
            other if other.pairs().is_some() => found.push(requirement(None, other)?),
            _ => {
                return Err(TarballError::Metadata(
                    "requirements must be a list of named requirements".to_owned(),
                ));
            }
        }
    }
    found.sort_by(|left, right| left.name.cmp(&right.name));
    found.dedup_by(|left, right| left.name == right.name);
    Ok(found)
}

pub fn interpret(text: &str) -> Result<Metadata, TarballError> {
    let terms = terms::parse(text).map_err(|error| TarballError::Metadata(error.to_string()))?;
    let mut fields: Vec<(String, Term)> = Vec::new();
    for term in terms {
        match term {
            Term::Tuple(parts) if parts.len() == 2 => {
                let mut parts = parts.into_iter();
                let key = parts.next().and_then(|key| key.text());
                let value = parts.next();
                if let (Some(key), Some(value)) = (key, value) {
                    fields.push((key, value));
                }
            }
            _ => {
                return Err(TarballError::Metadata(
                    "every term must be a {Key, Value} tuple".to_owned(),
                ));
            }
        }
    }
    let field = |key: &str| {
        fields
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value)
    };
    let name = text_of(field("name")).ok_or(TarballError::Missing("a package name"))?;
    if !valid_name(&name) {
        return Err(TarballError::Metadata(format!(
            "{name} is not a valid package name (lowercase letters, digits and _, starting with a letter)"
        )));
    }
    let version = text_of(field("version")).ok_or(TarballError::Missing("a version"))?;
    if !valid_version(&version) {
        return Err(TarballError::Metadata(format!(
            "{version} is not a semantic version"
        )));
    }
    let links = field("links")
        .and_then(Term::pairs)
        .unwrap_or_default()
        .iter()
        .filter_map(|(key, value)| Some((key.text()?, value.text()?)))
        .collect();
    Ok(Metadata {
        app: text_of(field("app")).filter(|app| *app != name),
        description: text_of(field("description")),
        licenses: texts_of(field("licenses")),
        build_tools: texts_of(field("build_tools")),
        elixir: text_of(field("elixir")),
        links,
        requirements: requirements(field("requirements"))?,
        name,
        version,
    })
}

impl Requirement {
    pub fn attributes(&self) -> Value {
        let mut map = Map::new();
        map.insert("name".to_owned(), json!(self.name));
        map.insert("requirement".to_owned(), json!(self.requirement));
        map.insert("optional".to_owned(), json!(self.optional));
        if let Some(app) = &self.app {
            map.insert("app".to_owned(), json!(app));
        }
        if let Some(repository) = &self.repository {
            map.insert("repository".to_owned(), json!(repository));
        }
        Value::Object(map)
    }

    pub fn from_attributes(value: &Value) -> Option<Requirement> {
        Some(Requirement {
            name: value.get("name")?.as_str()?.to_owned(),
            requirement: value.get("requirement")?.as_str()?.to_owned(),
            optional: value
                .get("optional")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            app: value.get("app").and_then(Value::as_str).map(str::to_owned),
            repository: value
                .get("repository")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }
}

impl Metadata {
    pub fn app_name(&self) -> &str {
        self.app.as_deref().unwrap_or(&self.name)
    }

    pub fn summary(&self) -> Value {
        let mut map = Map::new();
        map.insert("app".to_owned(), json!(self.app_name()));
        if let Some(description) = &self.description {
            map.insert("description".to_owned(), json!(description));
        }
        map.insert("licenses".to_owned(), json!(self.licenses));
        map.insert("buildTools".to_owned(), json!(self.build_tools));
        Value::Object(map)
    }

    pub fn links_value(&self) -> Value {
        Value::Object(
            self.links
                .iter()
                .map(|(key, value)| (key.clone(), json!(value)))
                .collect(),
        )
    }
}

impl Tarball {
    pub fn attributes(&self) -> Value {
        let metadata = &self.metadata;
        let mut map = match metadata.summary() {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        map.insert("name".to_owned(), json!(metadata.name));
        map.insert("version".to_owned(), json!(metadata.version));
        if let Some(elixir) = &metadata.elixir {
            map.insert("elixir".to_owned(), json!(elixir));
        }
        map.insert("links".to_owned(), metadata.links_value());
        map.insert(
            "requirements".to_owned(),
            Value::Array(
                metadata
                    .requirements
                    .iter()
                    .map(Requirement::attributes)
                    .collect(),
            ),
        );
        map.insert(
            "innerChecksum".to_owned(),
            json!(hex::encode(self.inner_checksum)),
        );
        map.insert(
            "outerChecksum".to_owned(),
            json!(hex::encode(self.outer_checksum)),
        );
        Value::Object(map)
    }
}

pub fn split_file(file: &str) -> Option<(&str, &str)> {
    let stem = file.strip_suffix(".tar")?;
    let (name, version) = stem.split_once('-')?;
    (!name.is_empty()
        && !version.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !version.contains('/'))
    .then_some((name, version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Built, Spec};

    #[test]
    fn tarballs_carry_their_metadata_and_checksums() {
        let built: Built = Spec::new("hello", "1.2.0-rc.1")
            .requirement("jason", "~> 1.4", false)
            .optional("plug", "~> 1.0")
            .build();
        let read = read(&built.bytes).unwrap();
        assert_eq!(read.metadata.name, "hello");
        assert_eq!(read.metadata.version, "1.2.0-rc.1");
        assert_eq!(read.metadata.app_name(), "hello");
        assert_eq!(read.metadata.licenses, ["Apache-2.0"]);
        assert_eq!(read.metadata.build_tools, ["mix"]);
        assert_eq!(
            read.metadata.description.as_deref(),
            Some("The hello package — for the tests")
        );
        assert_eq!(
            read.metadata.links,
            [("GitHub".to_owned(), "https://example.org/hello".to_owned())]
        );
        assert_eq!(
            read.metadata.requirements,
            [
                Requirement {
                    name: "jason".to_owned(),
                    requirement: "~> 1.4".to_owned(),
                    optional: false,
                    app: None,
                    repository: Some("hexpm".to_owned()),
                },
                Requirement {
                    name: "plug".to_owned(),
                    requirement: "~> 1.0".to_owned(),
                    optional: true,
                    app: None,
                    repository: Some("hexpm".to_owned()),
                }
            ]
        );
        assert_eq!(read.inner_checksum, built.inner_checksum);
        assert_eq!(
            read.outer_checksum,
            <[u8; 32]>::from(Sha256::digest(&built.bytes))
        );
        let attributes = read.attributes();
        assert_eq!(
            attributes["innerChecksum"],
            hex::encode(built.inner_checksum)
        );
        assert_eq!(attributes["requirements"][1]["optional"], true);
        assert_eq!(
            Requirement::from_attributes(&attributes["requirements"][0]).unwrap(),
            read.metadata.requirements[0]
        );
    }

    #[test]
    fn broken_tarballs_are_refused_with_a_reason() {
        assert!(matches!(
            read(b"not a tar"),
            Err(TarballError::NotTar(_)) | Err(TarballError::Missing(_))
        ));
        let mut spec = Spec::new("hello", "1.0.0");
        spec.checksum = Some("00".repeat(32));
        assert_eq!(read(&spec.build().bytes), Err(TarballError::Checksum));
        let mut spec = Spec::new("hello", "1.0.0");
        spec.format_version = "2".to_owned();
        assert_eq!(
            read(&spec.build().bytes),
            Err(TarballError::Version("2".to_owned()))
        );
        let mut spec = Spec::new("hello", "1.0.0");
        spec.metadata =
            Some("{<<\"name\">>,<<\"Hello\">>}.\n{<<\"version\">>,<<\"1.0.0\">>}.\n".to_owned());
        assert!(matches!(
            read(&spec.build().bytes),
            Err(TarballError::Metadata(_))
        ));
        let mut spec = Spec::new("hello", "1.0.0");
        spec.metadata =
            Some("{<<\"name\">>,<<\"hello\">>}.\n{<<\"version\">>,<<\"1.0\">>}.\n".to_owned());
        assert!(matches!(
            read(&spec.build().bytes),
            Err(TarballError::Metadata(_))
        ));
        let mut spec = Spec::new("hello", "1.0.0");
        spec.metadata = Some("{<<\"version\">>,<<\"1.0.0\">>}.\n".to_owned());
        assert_eq!(
            read(&spec.build().bytes),
            Err(TarballError::Missing("a package name"))
        );
    }

    #[test]
    fn legacy_and_current_requirement_forms_are_read() {
        let legacy = "{<<\"name\">>,<<\"a\">>}.\n{<<\"version\">>,<<\"1.0.0\">>}.\n\
                      {<<\"requirements\">>,[[{<<\"name\">>,<<\"b\">>},{<<\"app\">>,<<\"bee\">>},{<<\"optional\">>,true},{<<\"requirement\">>,<<\">= 0.1.0\">>}]]}.\n";
        let current = "{<<\"name\">>,<<\"a\">>}.\n{<<\"version\">>,<<\"1.0.0\">>}.\n\
                       {<<\"requirements\">>,[{<<\"b\">>,[{<<\"app\">>,<<\"bee\">>},{<<\"optional\">>,true},{<<\"requirement\">>,<<\">= 0.1.0\">>}]}]}.\n";
        let map = "{<<\"name\">>,<<\"a\">>}.\n{<<\"version\">>,<<\"1.0.0\">>}.\n\
                   {<<\"requirements\">>,#{<<\"b\">> => #{<<\"app\">> => <<\"bee\">>, <<\"optional\">> => true, <<\"requirement\">> => <<\">= 0.1.0\">>}}}.\n";
        let wanted = vec![Requirement {
            name: "b".to_owned(),
            requirement: ">= 0.1.0".to_owned(),
            optional: true,
            app: Some("bee".to_owned()),
            repository: None,
        }];
        for text in [legacy, current, map] {
            assert_eq!(interpret(text).unwrap().requirements, wanted, "{text}");
        }
        let broken = "{<<\"name\">>,<<\"a\">>}.\n{<<\"version\">>,<<\"1.0.0\">>}.\n{<<\"requirements\">>,[<<\"b\">>]}.\n";
        assert!(interpret(broken).is_err());
        assert!(interpret("{<<\"name\">>,<<\"a\">>}.\n[].\n").is_err());
    }

    #[test]
    fn names_and_files_follow_the_registry_rules() {
        assert!(valid_name("hello_world2"));
        assert!(!valid_name("Hello"));
        assert!(!valid_name("_hello"));
        assert!(!valid_name("hello-world"));
        assert!(valid_version("1.0.0-rc.1+build.5"));
        assert!(!valid_version("1.0"));
        assert_eq!(file_name("a", "1.0.0"), "a-1.0.0.tar");
        assert_eq!(
            split_file("a_b-1.0.0-rc.1.tar"),
            Some(("a_b", "1.0.0-rc.1"))
        );
        assert_eq!(split_file("a-1.0.0.tar.gz"), None);
        assert_eq!(split_file("a.tar"), None);
        assert_eq!(split_file("-1.0.0.tar"), None);
    }
}
