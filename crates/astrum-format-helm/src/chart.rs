use std::collections::BTreeMap;
use std::io::Read;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};

pub const MAX_CHART_FILE: u64 = 4 * 1024 * 1024;
const MAX_INFLATED: u64 = 1024 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ChartError {
    #[error("the archive cannot be read: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Invalid(String),
}

fn or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Flag {
    Bool(bool),
    Text(String),
}

fn lenient_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<Flag>::deserialize(deserializer)? {
        Some(Flag::Bool(value)) => Some(value),
        Some(Flag::Text(text)) => match text.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        },
        None => None,
    })
}

fn blank(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|value| value.trim().is_empty())
}

fn not_true(value: &Option<bool>) -> bool {
    *value != Some(true)
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Maintainer {
    #[serde(default, skip_serializing_if = "blank")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    #[serde(default, skip_serializing_if = "blank")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub condition: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub enabled: Option<bool>,
    #[serde(
        default,
        rename = "import-values",
        skip_serializing_if = "Option::is_none"
    )]
    pub import_values: Option<Value>,
    #[serde(default, skip_serializing_if = "blank")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub repository: Option<String>,
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub annotations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub api_version: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub app_version: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub condition: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub created: Option<String>,
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub dependencies: Vec<Dependency>,
    #[serde(
        default,
        deserialize_with = "lenient_bool",
        skip_serializing_if = "not_true"
    )]
    pub deprecated: Option<bool>,
    #[serde(default, skip_serializing_if = "blank")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub digest: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub engine: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub home: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub icon: Option<String>,
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub kube_version: Option<String>,
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub maintainers: Vec<Maintainer>,
    #[serde(default, deserialize_with = "or_default")]
    pub name: String,
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub tags: Option<String>,
    #[serde(default, skip_serializing_if = "blank")]
    pub tiller_version: Option<String>,
    #[serde(default, rename = "type", skip_serializing_if = "blank")]
    pub kind: Option<String>,
    #[serde(
        default,
        deserialize_with = "or_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub urls: Vec<String>,
    #[serde(default, deserialize_with = "or_default")]
    pub version: String,
}

pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 250
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

pub fn is_valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 128
        && !version.starts_with('.')
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+'))
}

pub fn file_name(name: &str, version: &str) -> String {
    format!("{name}-{version}.tgz")
}

impl Chart {
    pub fn parse(yaml: &[u8]) -> Result<Chart, ChartError> {
        let chart: Chart = serde_norway::from_slice(yaml)
            .map_err(|error| ChartError::Invalid(format!("Chart.yaml cannot be read: {error}")))?;
        chart.checked()
    }

    pub fn checked(self) -> Result<Chart, ChartError> {
        if !is_valid_name(&self.name) {
            return Err(ChartError::Invalid(match self.name.is_empty() {
                true => "the chart has no name".to_owned(),
                false => format!("{} is not a valid chart name", self.name),
            }));
        }
        if !is_valid_version(&self.version) {
            return Err(ChartError::Invalid(match self.version.is_empty() {
                true => format!("the chart {} has no version", self.name),
                false => format!("{} is not a valid chart version", self.version),
            }));
        }
        Ok(self)
    }

    pub fn file_name(&self) -> String {
        file_name(&self.name, &self.version)
    }

    pub fn metadata(&self) -> Chart {
        Chart {
            created: None,
            digest: None,
            urls: Vec::new(),
            ..self.clone()
        }
    }

    pub fn attributes(&self) -> Value {
        json!({ "helm": serde_json::to_value(self.metadata()).unwrap_or(Value::Null) })
    }

    pub fn from_attributes(attributes: &Value) -> Option<Chart> {
        serde_json::from_value(attributes.get("helm")?.clone()).ok()
    }

    pub fn semver(&self) -> Option<semver::Version> {
        semver::Version::parse(self.version.trim_start_matches(['v', 'V'])).ok()
    }
}

fn is_chart_yaml(path: &str) -> bool {
    let parts: Vec<&str> = path
        .trim_start_matches("./")
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    matches!(parts.as_slice(), [top, "Chart.yaml"] if *top != "." && *top != "..")
}

pub fn chart_yaml(archive: &[u8]) -> Result<Vec<u8>, ChartError> {
    let inflated = flate2::read::GzDecoder::new(archive).take(MAX_INFLATED);
    let mut tar = tar::Archive::new(inflated);
    for entry in tar.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path()?.to_string_lossy().replace('\\', "/");
        if !is_chart_yaml(&path) {
            continue;
        }
        if entry.size() > MAX_CHART_FILE {
            return Err(ChartError::Invalid(format!(
                "{path} is larger than {MAX_CHART_FILE} bytes"
            )));
        }
        let mut data = Vec::new();
        (&mut entry).take(MAX_CHART_FILE).read_to_end(&mut data)?;
        return Ok(data);
    }
    Err(ChartError::Invalid(
        "the archive holds no Chart.yaml at the top of a chart folder".to_owned(),
    ))
}

pub fn read(archive: &[u8]) -> Result<Chart, ChartError> {
    if !archive.starts_with(&[0x1f, 0x8b]) {
        return Err(ChartError::Invalid(
            "the file is not a gzip-compressed chart archive".to_owned(),
        ));
    }
    let chart = Chart::parse(&chart_yaml(archive)?)?;
    Ok(chart.metadata())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for (name, data) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *data).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    const CHART: &str = "apiVersion: v2\nname: demo\ndescription: A demo chart\ntype: application\nversion: 1.10\nappVersion: 1.16\nkeywords:\n  - web\n  - 7\nmaintainers:\n  - name: Jane Doe\n    email: jane@example.org\nannotations:\n  category: Testing\n  count: 3\ndependencies:\n  - name: redis\n    version: ~17.0.0\n    repository: https://charts.example.org\n    condition: redis.enabled\n    import-values:\n      - child: default\n        parent: redis\n    enabled: true\ndeprecated: yes\nunknownField: kept out\n";

    #[test]
    fn chart_yaml_keeps_scalars_as_written() {
        let chart = Chart::parse(CHART.as_bytes()).unwrap();
        assert_eq!(chart.name, "demo");
        assert_eq!(chart.version, "1.10");
        assert_eq!(chart.app_version.as_deref(), Some("1.16"));
        assert_eq!(chart.keywords, vec!["web", "7"]);
        assert_eq!(chart.annotations["count"], "3");
        assert_eq!(chart.maintainers[0].name.as_deref(), Some("Jane Doe"));
        assert_eq!(chart.dependencies[0].version.as_deref(), Some("~17.0.0"));
        assert_eq!(chart.dependencies[0].enabled, Some(true));
        assert_eq!(
            chart.dependencies[0].import_values,
            Some(json!([{"child": "default", "parent": "redis"}]))
        );
        assert_eq!(chart.deprecated, Some(true));
        assert_eq!(chart.kind.as_deref(), Some("application"));
    }

    #[test]
    fn attributes_round_trip_without_index_fields() {
        let mut chart = Chart::parse(CHART.as_bytes()).unwrap();
        chart.digest = Some("abc".to_owned());
        chart.urls = vec!["demo-1.10.tgz".to_owned()];
        let attributes = chart.attributes();
        assert_eq!(attributes["helm"]["appVersion"], "1.16");
        assert_eq!(attributes["helm"]["keywords"], json!(["web", "7"]));
        assert!(attributes["helm"].get("digest").is_none());
        assert!(attributes["helm"].get("urls").is_none());
        assert!(attributes["helm"].get("unknownField").is_none());
        let back = Chart::from_attributes(&attributes).unwrap();
        assert_eq!(back, chart.metadata());
    }

    #[test]
    fn null_lists_and_empty_values_are_tolerated() {
        let chart =
            Chart::parse(b"name: x\nversion: 0.1.0\nkeywords:\nmaintainers: ~\ndescription: ''\n")
                .unwrap();
        assert!(chart.keywords.is_empty() && chart.maintainers.is_empty());
        let yaml = serde_norway::to_string(&chart).unwrap();
        assert_eq!(yaml, "name: x\nversion: 0.1.0\n");
    }

    #[test]
    fn names_and_versions_are_checked() {
        for (name, version) in [
            ("", "1.0.0"),
            ("a/b", "1.0.0"),
            ("..", "1.0.0"),
            ("a", ""),
            ("a", "../1"),
            ("a", "1.0 0"),
        ] {
            let chart = Chart {
                name: name.to_owned(),
                version: version.to_owned(),
                ..Default::default()
            };
            assert!(chart.checked().is_err(), "{name} {version}");
        }
        assert_eq!(file_name("demo", "1.0.0+build.1"), "demo-1.0.0+build.1.tgz");
    }

    #[test]
    fn the_chart_yaml_of_the_top_folder_is_read() {
        let data = archive(&[
            ("demo/charts/sub/Chart.yaml", b"name: sub\nversion: 9.9.9\n"),
            ("demo/values.yaml", b"replicas: 1\n"),
            ("demo/Chart.yaml", CHART.as_bytes()),
        ]);
        let chart = read(&data).unwrap();
        assert_eq!(
            (chart.name.as_str(), chart.version.as_str()),
            ("demo", "1.10")
        );
        assert!(read(b"not a chart").is_err());
        let empty = archive(&[("demo/values.yaml", b"a: 1\n")]);
        assert!(
            read(&empty)
                .unwrap_err()
                .to_string()
                .contains("no Chart.yaml")
        );
        let nameless = archive(&[("demo/Chart.yaml", b"version: 1.0.0\n")]);
        assert!(read(&nameless).unwrap_err().to_string().contains("no name"));
    }
}
