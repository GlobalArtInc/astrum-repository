use std::cmp::Ordering;
use std::io::Read;

use serde_json::{Map, Value, json};

pub const RUBY: &str = "ruby";
pub const ANY: &str = ">= 0";
pub const GEMS_PREFIX: &str = "/gems/";
pub const QUICK_PREFIX: &str = "/quick/Marshal.4.8/";
const MAX_METADATA: u64 = 32 * 1024 * 1024;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GemError {
    #[error("Not a gem file: {0}")]
    NotGem(String),
    #[error("{0}")]
    Invalid(String),
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        && name.chars().any(|c| c.is_ascii_alphabetic())
        && !name.starts_with('.')
        && !name.starts_with('-')
}

pub fn valid_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 320
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        && !name.starts_with('.')
}

pub fn valid_version(version: &str) -> bool {
    let version = version.trim();
    if version.is_empty() || version.len() > 128 {
        return false;
    }
    let (release, pre) = match version.split_once('-') {
        Some((release, pre)) => (release, Some(pre)),
        None => (version, None),
    };
    let mut parts = release.split('.');
    let first = parts.next().unwrap_or_default();
    if first.is_empty() || !first.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if !parts.all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric())) {
        return false;
    }
    pre.is_none_or(|pre| {
        pre.split('.').all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
    })
}

pub fn valid_platform(platform: &str) -> bool {
    !platform.is_empty()
        && platform.len() <= 64
        && platform
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

pub fn normalize_platform(platform: Option<&str>) -> String {
    match platform.map(str::trim).filter(|platform| !platform.is_empty()) {
        Some(platform) => platform.to_owned(),
        None => RUBY.to_owned(),
    }
}

pub fn is_prerelease(version: &str) -> bool {
    version.chars().any(|c| c.is_ascii_alphabetic())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    Number(String),
    Text(String),
}

fn segments(version: &str) -> Vec<Segment> {
    let text = version.trim().replace('-', ".pre.");
    let mut found = Vec::new();
    let mut current = String::new();
    let mut digits = false;
    for c in text.chars() {
        let kind = if c.is_ascii_digit() {
            Some(true)
        } else if c.is_ascii_alphabetic() {
            Some(false)
        } else {
            None
        };
        match kind {
            Some(is_digit) if !current.is_empty() && is_digit == digits => current.push(c),
            Some(is_digit) => {
                if !current.is_empty() {
                    found.push(segment(&current, digits));
                }
                current = c.to_string();
                digits = is_digit;
            }
            None => {
                if !current.is_empty() {
                    found.push(segment(&current, digits));
                    current.clear();
                }
            }
        }
    }
    if !current.is_empty() {
        found.push(segment(&current, digits));
    }
    found
}

fn segment(text: &str, digits: bool) -> Segment {
    match digits {
        true => {
            let trimmed = text.trim_start_matches('0');
            Segment::Number(if trimmed.is_empty() { "0" } else { trimmed }.to_owned())
        }
        false => Segment::Text(text.to_owned()),
    }
}

fn zero(segment: &Segment) -> bool {
    *segment == Segment::Number("0".to_owned())
}

fn canonical(version: &str) -> Vec<Segment> {
    let all = segments(version);
    let split = all
        .iter()
        .position(|segment| matches!(segment, Segment::Text(_)))
        .unwrap_or(all.len());
    let mut numbers = all[..split].to_vec();
    while numbers.last().is_some_and(zero) {
        numbers.pop();
    }
    let mut rest = all[split..].to_vec();
    while rest.last().is_some_and(zero) {
        rest.pop();
    }
    numbers.extend(rest);
    numbers
}

pub fn compare(left: &str, right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }
    let lhs = canonical(left);
    let rhs = canonical(right);
    let zero = Segment::Number("0".to_owned());
    for index in 0..lhs.len().max(rhs.len()) {
        let l = lhs.get(index).unwrap_or(&zero);
        let r = rhs.get(index).unwrap_or(&zero);
        if l == r {
            continue;
        }
        return match (l, r) {
            (Segment::Text(_), Segment::Number(_)) => Ordering::Less,
            (Segment::Number(_), Segment::Text(_)) => Ordering::Greater,
            (Segment::Number(a), Segment::Number(b)) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
            (Segment::Text(a), Segment::Text(b)) => a.cmp(b),
        };
    }
    Ordering::Equal
}

pub fn split_requirement(requirement: &str) -> Result<(String, String), GemError> {
    let text = requirement.trim();
    let operators = ["!=", ">=", "<=", "~>", "=", ">", "<"];
    let (operator, rest) = operators
        .iter()
        .find_map(|operator| text.strip_prefix(operator).map(|rest| (*operator, rest)))
        .unwrap_or(("=", text));
    let version = rest.trim();
    if version.is_empty() || version.contains(char::is_whitespace) {
        return Err(GemError::Invalid(format!("Invalid requirement: {requirement}")));
    }
    Ok((operator.to_owned(), version.to_owned()))
}

pub fn normalize_requirement(requirement: &str) -> Result<String, GemError> {
    let (operator, version) = split_requirement(requirement)?;
    Ok(format!("{operator} {version}"))
}

pub fn is_any(requirements: &[String]) -> bool {
    requirements.is_empty() || requirements == [ANY]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub kind: String,
    pub requirements: Vec<String>,
}

impl Dependency {
    pub fn new(name: &str, kind: &str, requirements: Vec<String>) -> Self {
        Self {
            name: name.to_owned(),
            kind: kind.to_owned(),
            requirements: match requirements.is_empty() {
                true => vec![ANY.to_owned()],
                false => requirements,
            },
        }
    }

    pub fn runtime(&self) -> bool {
        self.kind != "development"
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub name: String,
    pub version: String,
    pub platform: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub email: Vec<String>,
    pub homepage: Option<String>,
    pub licenses: Vec<String>,
    pub metadata: Vec<(String, String)>,
    pub dependencies: Vec<Dependency>,
    pub required_ruby_version: Vec<String>,
    pub required_rubygems_version: Vec<String>,
    pub rubygems_version: Option<String>,
    pub specification_version: i64,
    pub date: Option<String>,
}

pub fn is_ruby(platform: &str) -> bool {
    platform.trim().is_empty() || platform.trim() == RUBY
}

pub fn version_and_platform(version: &str, platform: &str) -> String {
    match is_ruby(platform) {
        true => version.to_owned(),
        false => format!("{version}-{platform}"),
    }
}

pub fn full_name(name: &str, version: &str, platform: &str) -> String {
    format!("{name}-{}", version_and_platform(version, platform))
}

pub fn gem_path(full_name: &str) -> String {
    format!("{GEMS_PREFIX}{full_name}.gem")
}

impl Spec {
    pub fn full_name(&self) -> String {
        full_name(&self.name, &self.version, &self.platform)
    }

    pub fn gem_path(&self) -> String {
        gem_path(&self.full_name())
    }

    pub fn version_and_platform(&self) -> String {
        version_and_platform(&self.version, &self.platform)
    }

    pub fn runtime_dependencies(&self) -> Vec<Dependency> {
        self.dependencies
            .iter()
            .filter(|dependency| dependency.runtime())
            .cloned()
            .collect()
    }

    pub fn attributes(&self) -> Value {
        let mut out = Map::new();
        out.insert("name".to_owned(), json!(self.name));
        out.insert("version".to_owned(), json!(self.version));
        out.insert("platform".to_owned(), json!(self.platform));
        for (key, value) in [
            ("summary", &self.summary),
            ("description", &self.description),
        ] {
            if let Some(value) = value.as_deref().filter(|value| !value.trim().is_empty()) {
                out.insert(key.to_owned(), json!(value));
            }
        }
        if !self.authors.is_empty() {
            out.insert("authors".to_owned(), json!(self.authors.join(", ")));
        }
        if !self.email.is_empty() {
            out.insert("email".to_owned(), json!(self.email));
        }
        if let Some(homepage) = self.homepage.as_deref().filter(|value| !value.trim().is_empty()) {
            out.insert("homepage".to_owned(), json!(homepage));
        }
        if !self.licenses.is_empty() {
            out.insert("licenses".to_owned(), json!(self.licenses));
        }
        if !self.metadata.is_empty() {
            let metadata: Map<String, Value> = self
                .metadata
                .iter()
                .map(|(key, value)| (key.clone(), json!(value)))
                .collect();
            out.insert("metadata".to_owned(), Value::Object(metadata));
        }
        out.insert(
            "dependencies".to_owned(),
            Value::Array(
                self.dependencies
                    .iter()
                    .map(|dependency| {
                        json!({"name": dependency.name, "type": dependency.kind, "requirements": dependency.requirements})
                    })
                    .collect(),
            ),
        );
        out.insert("required_ruby_version".to_owned(), json!(self.required_ruby_version));
        out.insert(
            "required_rubygems_version".to_owned(),
            json!(self.required_rubygems_version),
        );
        if let Some(version) = self.rubygems_version.as_deref().filter(|value| !value.trim().is_empty()) {
            out.insert("rubygems_version".to_owned(), json!(version));
        }
        out.insert(
            "specification_version".to_owned(),
            json!(self.specification_version),
        );
        if let Some(date) = &self.date {
            out.insert("date".to_owned(), json!(date));
        }
        Value::Object(out)
    }

    pub fn from_attributes(attributes: &Value) -> Option<Spec> {
        let text = |key: &str| attributes.get(key).and_then(Value::as_str).map(str::to_owned);
        let strings = |key: &str| -> Vec<String> {
            match attributes.get(key) {
                Some(Value::Array(items)) => items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect(),
                Some(Value::String(text)) if !text.trim().is_empty() => vec![text.clone()],
                _ => Vec::new(),
            }
        };
        let dependencies = attributes
            .get("dependencies")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?;
                        let kind = item.get("type").and_then(Value::as_str).unwrap_or("runtime");
                        let requirements = item
                            .get("requirements")
                            .and_then(Value::as_array)
                            .map(|list| {
                                list.iter()
                                    .filter_map(|value| value.as_str().map(str::to_owned))
                                    .collect()
                            })
                            .unwrap_or_default();
                        Some(Dependency::new(name, kind, requirements))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let metadata = attributes
            .get("metadata")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())))
                    .collect()
            })
            .unwrap_or_default();
        Some(Spec {
            name: text("name")?,
            version: text("version")?,
            platform: normalize_platform(text("platform").as_deref()),
            summary: text("summary"),
            description: text("description"),
            authors: match attributes.get("authors") {
                Some(Value::String(text)) => text.split(", ").map(str::to_owned).collect(),
                _ => strings("authors"),
            },
            email: strings("email"),
            homepage: text("homepage"),
            licenses: strings("licenses"),
            metadata,
            dependencies,
            required_ruby_version: with_any(strings("required_ruby_version")),
            required_rubygems_version: with_any(strings("required_rubygems_version")),
            rubygems_version: text("rubygems_version"),
            specification_version: attributes
                .get("specification_version")
                .and_then(Value::as_i64)
                .unwrap_or(4),
            date: text("date"),
        })
    }
}

fn with_any(requirements: Vec<String>) -> Vec<String> {
    match requirements.is_empty() {
        true => vec![ANY.to_owned()],
        false => requirements,
    }
}

fn plain(value: serde_norway::Value) -> serde_norway::Value {
    use serde_norway::Value as Yaml;
    match value {
        Yaml::Tagged(tagged) => {
            let tag = tagged.tag.to_string();
            let inner = plain(tagged.value);
            match (tag.as_str(), inner) {
                ("!binary", Yaml::String(encoded)) => {
                    use base64::Engine;
                    let compact: String = encoded.split_whitespace().collect();
                    match base64::engine::general_purpose::STANDARD.decode(compact) {
                        Ok(bytes) => Yaml::String(String::from_utf8_lossy(&bytes).into_owned()),
                        Err(_) => Yaml::String(encoded),
                    }
                }
                (_, inner) => inner,
            }
        }
        Yaml::Sequence(items) => Yaml::Sequence(items.into_iter().map(plain).collect()),
        Yaml::Mapping(map) => Yaml::Mapping(map.into_iter().map(|(key, value)| (plain(key), plain(value))).collect()),
        other => other,
    }
}

fn yaml_text(value: Option<&serde_norway::Value>) -> Option<String> {
    use serde_norway::Value as Yaml;
    match value? {
        Yaml::String(text) => Some(text.clone()),
        Yaml::Number(number) => Some(number.to_string()),
        Yaml::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn yaml_strings(value: Option<&serde_norway::Value>) -> Vec<String> {
    use serde_norway::Value as Yaml;
    match value {
        Some(Yaml::Sequence(items)) => items
            .iter()
            .filter_map(|item| yaml_text(Some(item)))
            .filter(|text| !text.trim().is_empty())
            .collect(),
        other => yaml_text(other)
            .filter(|text| !text.trim().is_empty())
            .into_iter()
            .collect(),
    }
}

fn yaml_version(value: Option<&serde_norway::Value>) -> Option<String> {
    match value? {
        serde_norway::Value::Mapping(map) => yaml_text(map.get("version")),
        other => yaml_text(Some(other)),
    }
}

fn yaml_requirements(value: Option<&serde_norway::Value>) -> Result<Vec<String>, GemError> {
    use serde_norway::Value as Yaml;
    let requirements = match value {
        Some(Yaml::Mapping(map)) => map.get("requirements"),
        other => other,
    };
    let mut found = Vec::new();
    match requirements {
        Some(Yaml::Sequence(items)) => {
            for item in items {
                match item {
                    Yaml::Sequence(pair) if pair.len() == 2 => {
                        if let (Some(operator), Some(version)) =
                            (yaml_text(pair.first()), yaml_version(pair.get(1)))
                        {
                            found.push(format!("{} {}", operator.trim(), version.trim()));
                        }
                    }
                    Yaml::String(text) if !text.trim().is_empty() => {
                        found.push(normalize_requirement(text)?)
                    }
                    _ => {}
                }
            }
        }
        Some(Yaml::String(text)) if !text.trim().is_empty() => {
            for part in text.split(',') {
                found.push(normalize_requirement(part)?);
            }
        }
        _ => {}
    }
    Ok(found)
}

fn yaml_platform(value: Option<&serde_norway::Value>) -> String {
    match value {
        Some(serde_norway::Value::Mapping(map)) => {
            let parts: Vec<String> = ["cpu", "os", "version"]
                .iter()
                .filter_map(|key| yaml_text(map.get(*key)))
                .filter(|part| !part.trim().is_empty())
                .collect();
            normalize_platform(Some(&parts.join("-")))
        }
        other => normalize_platform(yaml_text(other).as_deref()),
    }
}

pub fn parse_spec(yaml: &str) -> Result<Spec, GemError> {
    let document: serde_norway::Value = serde_norway::from_str(yaml)
        .map_err(|error| GemError::Invalid(format!("Invalid gem specification: {error}")))?;
    let document = plain(document);
    let serde_norway::Value::Mapping(spec) = document else {
        return Err(GemError::Invalid(
            "Invalid gem specification: not a mapping".to_owned(),
        ));
    };
    let name = yaml_text(spec.get("name")).unwrap_or_default();
    if !valid_name(&name) {
        return Err(GemError::Invalid(format!("Invalid gem name: {name}")));
    }
    let version = yaml_version(spec.get("version")).unwrap_or_default();
    if !valid_version(&version) {
        return Err(GemError::Invalid(format!("Invalid gem version: {version}")));
    }
    let platform = yaml_platform(spec.get("platform"));
    if !valid_platform(&platform) {
        return Err(GemError::Invalid(format!("Invalid gem platform: {platform}")));
    }
    let mut licenses = yaml_strings(spec.get("licenses"));
    if licenses.is_empty() {
        licenses = yaml_strings(spec.get("license"));
    }
    let mut dependencies = Vec::new();
    if let Some(serde_norway::Value::Sequence(items)) = spec.get("dependencies") {
        for item in items {
            let serde_norway::Value::Mapping(dependency) = item else {
                continue;
            };
            let Some(name) = yaml_text(dependency.get("name")) else {
                continue;
            };
            let requirement = dependency
                .get("requirement")
                .or_else(|| dependency.get("version_requirements"));
            let kind = yaml_text(dependency.get("type"))
                .map(|kind| kind.trim_start_matches(':').to_owned())
                .unwrap_or_else(|| "runtime".to_owned());
            dependencies.push(Dependency::new(&name, &kind, yaml_requirements(requirement)?));
        }
    }
    let metadata = match spec.get("metadata") {
        Some(serde_norway::Value::Mapping(map)) => map
            .iter()
            .filter_map(|(key, value)| Some((yaml_text(Some(key))?, yaml_text(Some(value))?)))
            .collect(),
        _ => Vec::new(),
    };
    let specification_version = yaml_text(spec.get("specification_version"))
        .and_then(|text| text.trim().parse::<i64>().ok())
        .filter(|value| (-999..=999).contains(value))
        .unwrap_or(4);
    let date = yaml_text(spec.get("date")).and_then(|text| {
        let bytes = text.as_bytes();
        (text.len() >= 10
            && bytes[..4].iter().all(u8::is_ascii_digit)
            && bytes[4] == b'-'
            && bytes[7] == b'-')
            .then(|| text[..10].to_owned())
    });
    Ok(Spec {
        name,
        version: version.trim().to_owned(),
        platform,
        summary: yaml_text(spec.get("summary")),
        description: yaml_text(spec.get("description")),
        authors: yaml_strings(spec.get("authors")),
        email: yaml_strings(spec.get("email")),
        homepage: yaml_text(spec.get("homepage")),
        licenses,
        metadata,
        dependencies,
        required_ruby_version: with_any(yaml_requirements(spec.get("required_ruby_version"))?),
        required_rubygems_version: with_any(yaml_requirements(
            spec.get("required_rubygems_version"),
        )?),
        rubygems_version: yaml_text(spec.get("rubygems_version")),
        specification_version,
        date,
    })
}

pub fn read_gem(data: impl Read) -> Result<Spec, GemError> {
    let mut archive = tar::Archive::new(data);
    let entries = archive
        .entries()
        .map_err(|error| GemError::NotGem(error.to_string()))?;
    let mut spec = None;
    let mut payload = false;
    for entry in entries {
        let entry = entry.map_err(|error| GemError::NotGem(error.to_string()))?;
        let path = entry
            .path()
            .map_err(|error| GemError::NotGem(error.to_string()))?
            .to_string_lossy()
            .into_owned();
        match path.as_str() {
            "metadata.gz" if spec.is_none() => {
                let mut yaml = String::new();
                flate2::read::GzDecoder::new(entry)
                    .take(MAX_METADATA + 1)
                    .read_to_string(&mut yaml)
                    .map_err(|error| GemError::NotGem(error.to_string()))?;
                if yaml.len() as u64 > MAX_METADATA {
                    return Err(GemError::Invalid(
                        "Gem specification is larger than 32 MB".to_owned(),
                    ));
                }
                spec = Some(parse_spec(&yaml)?);
            }
            "data.tar.gz" => payload = true,
            _ => {}
        }
        if spec.is_some() && payload {
            break;
        }
    }
    let spec = spec.ok_or_else(|| GemError::Invalid("Not a gem file: metadata.gz is missing".to_owned()))?;
    if !payload {
        return Err(GemError::Invalid(
            "Not a gem file: data.tar.gz is missing".to_owned(),
        ));
    }
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const SPEC: &str = "--- !ruby/object:Gem::Specification\nname: hola\nversion: !ruby/object:Gem::Version\n  version: 1.2.0\nplatform: ruby\nauthors:\n- Jane Doe\nbindir: bin\ncert_chain: []\ndate: 2026-10-13 00:00:00.000000000 Z\ndependencies:\n- !ruby/object:Gem::Dependency\n  name: rake\n  requirement: !ruby/object:Gem::Requirement\n    requirements:\n    - - \">=\"\n      - !ruby/object:Gem::Version\n        version: '12'\n    - - \"<\"\n      - !ruby/object:Gem::Version\n        version: '14'\n  type: :runtime\n  prerelease: false\n  version_requirements: !ruby/object:Gem::Requirement\n    requirements:\n    - - \">=\"\n      - !ruby/object:Gem::Version\n        version: '12'\n- !ruby/object:Gem::Dependency\n  name: rspec\n  requirement: !ruby/object:Gem::Requirement\n    requirements:\n    - - \"~>\"\n      - !ruby/object:Gem::Version\n        version: '3.0'\n  type: :development\n  prerelease: false\ndescription: A simple hello world gem\nemail: jane@example.org\nexecutables: []\nextensions: []\nextra_rdoc_files: []\nfiles:\n- lib/hola.rb\nhomepage: https://example.org/hola\nlicenses:\n- MIT\nmetadata:\n  source_code_uri: https://example.org/hola/src\npost_install_message:\nrdoc_options: []\nrequire_paths:\n- lib\nrequired_ruby_version: !ruby/object:Gem::Requirement\n  requirements:\n  - - \">=\"\n    - !ruby/object:Gem::Version\n      version: '2.7'\nrequired_rubygems_version: !ruby/object:Gem::Requirement\n  requirements:\n  - - \">=\"\n    - !ruby/object:Gem::Version\n      version: '0'\nrequirements: []\nrubygems_version: 3.6.7\nsigning_key:\nspecification_version: 4\nsummary: Hola!\ntest_files: []\n";

    #[test]
    fn specifications_are_read_from_their_yaml() {
        let spec = parse_spec(SPEC).unwrap();
        assert_eq!(spec.name, "hola");
        assert_eq!(spec.version, "1.2.0");
        assert_eq!(spec.platform, "ruby");
        assert_eq!(spec.date.as_deref(), Some("2026-10-13"));
        assert_eq!(spec.dependencies.len(), 2);
        assert_eq!(spec.dependencies[0].requirements, [">= 12", "< 14"]);
        assert_eq!(spec.runtime_dependencies().len(), 1);
        assert_eq!(spec.required_ruby_version, [">= 2.7"]);
        assert_eq!(spec.metadata[0].0, "source_code_uri");
        assert_eq!(Spec::from_attributes(&spec.attributes()).unwrap(), spec);
        assert!(parse_spec("--- []").is_err());
    }

    #[test]
    fn versions_compare_like_rubygems() {
        for (a, b, expected) in [
            ("1.0", "1.0.0", Ordering::Equal),
            ("1.0.a", "1.0", Ordering::Less),
            ("1.0.0-rc1", "1.0.0", Ordering::Less),
            ("1.10", "1.9", Ordering::Greater),
            ("2.0.0.pre", "2.0.0.beta", Ordering::Greater),
        ] {
            assert_eq!(compare(a, b), expected, "{a} {b}");
        }
        assert!(valid_version("1.0.0-rc.1") && valid_version("3.2.1.beta2") && !valid_version("x1"));
        assert!(is_prerelease("1.0.rc1") && !is_prerelease("1.0"));
        assert!(valid_name("rails") && !valid_name("123") && !valid_name("-x"));
        assert_eq!(normalize_requirement("~>3.0").unwrap(), "~> 3.0");
        assert_eq!(normalize_requirement("1.0").unwrap(), "= 1.0");
        assert_eq!(full_name("nokogiri", "1.16.0", "x86_64-linux"), "nokogiri-1.16.0-x86_64-linux");
    }
}
