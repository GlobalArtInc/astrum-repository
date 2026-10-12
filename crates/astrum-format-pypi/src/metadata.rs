use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

const SINGLE: &[(&str, &str)] = &[
    ("metadata-version", "metadata_version"),
    ("name", "name"),
    ("version", "version"),
    ("summary", "summary"),
    ("description", "description"),
    ("description-content-type", "description_content_type"),
    ("keywords", "keywords"),
    ("home-page", "home_page"),
    ("download-url", "download_url"),
    ("author", "author"),
    ("author-email", "author_email"),
    ("maintainer", "maintainer"),
    ("maintainer-email", "maintainer_email"),
    ("license", "license"),
    ("license-expression", "license_expression"),
    ("requires-python", "requires_python"),
];

const MULTIPLE: &[(&str, &str)] = &[
    ("classifier", "classifiers"),
    ("requires-dist", "requires_dist"),
    ("project-url", "project_urls"),
    ("platform", "platform"),
    ("provides-extra", "provides_extra"),
];

const ASSET_KEYS: &[&str] = &["name", "version", "summary", "requires_python"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoreMetadata {
    pub single: BTreeMap<&'static str, String>,
    pub multiple: BTreeMap<&'static str, Vec<String>>,
}

fn single_key(header: &str) -> Option<&'static str> {
    SINGLE
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(header))
        .map(|(_, key)| *key)
}

fn multiple_key(header: &str) -> Option<&'static str> {
    MULTIPLE
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(header))
        .map(|(_, key)| *key)
}

impl CoreMetadata {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.single
            .get(key)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
    }

    pub fn name(&self) -> Option<&str> {
        self.get("name")
    }

    pub fn version(&self) -> Option<&str> {
        self.get("version")
    }

    fn set(&mut self, header: &str, value: String) {
        if let Some(key) = single_key(header) {
            self.single.entry(key).or_insert(value);
        } else if let Some(key) = multiple_key(header) {
            self.multiple.entry(key).or_default().push(value);
        }
    }

    pub fn parse(text: &str) -> Self {
        let mut metadata = CoreMetadata::default();
        let mut current: Option<(String, String)> = None;
        let mut lines = text.split('\n');
        let mut body = None;
        for line in lines.by_ref() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.is_empty() {
                body = Some(());
                break;
            }
            if line.starts_with([' ', '\t']) {
                if let Some((_, value)) = &mut current {
                    let continued = line.trim_start();
                    value.push('\n');
                    value.push_str(continued.strip_prefix('|').unwrap_or(continued));
                }
                continue;
            }
            if let Some((header, value)) = current.take() {
                metadata.set(&header, value);
            }
            if let Some((header, value)) = line.split_once(':') {
                current = Some((header.trim().to_owned(), value.trim().to_owned()));
            }
        }
        if let Some((header, value)) = current.take() {
            metadata.set(&header, value);
        }
        if body.is_some() {
            let rest: Vec<&str> = lines
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .collect();
            let description = rest.join("\n");
            if !description.trim().is_empty() {
                metadata
                    .single
                    .insert("description", description.trim_end().to_owned());
            }
        }
        metadata
    }

    pub fn from_form<'a>(fields: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut metadata = CoreMetadata::default();
        for (field, value) in fields {
            if value.trim().is_empty() {
                continue;
            }
            if let Some((_, key)) = SINGLE.iter().find(|(_, key)| *key == field) {
                metadata
                    .single
                    .entry(key)
                    .or_insert_with(|| value.to_owned());
            } else if let Some((_, key)) = MULTIPLE.iter().find(|(_, key)| *key == field) {
                metadata
                    .multiple
                    .entry(key)
                    .or_default()
                    .push(value.to_owned());
            } else if field == "classifier" {
                metadata
                    .multiple
                    .entry("classifiers")
                    .or_default()
                    .push(value.to_owned());
            }
        }
        metadata
    }

    pub fn asset_attributes(&self) -> Value {
        let mut pypi = Map::new();
        for key in ASSET_KEYS {
            if let Some(value) = self.get(key) {
                pypi.insert((*key).to_owned(), json!(value));
            }
        }
        json!({ "pypi": pypi })
    }

    pub fn component_attributes(&self) -> Value {
        let mut pypi = Map::new();
        for (key, value) in &self.single {
            if !value.trim().is_empty() && *key != "metadata_version" {
                pypi.insert((*key).to_owned(), json!(value));
            }
        }
        for (key, values) in &self.multiple {
            pypi.insert((*key).to_owned(), json!(values));
        }
        json!({ "pypi": pypi })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PKG_INFO: &str = "Metadata-Version: 2.4\r\nName: Astrum_Probe.Pkg\r\nVersion: 0.2.0\r\nSummary: Probe package\r\nAuthor-email: Probe Author <probe@example.org>\r\nLicense: MIT\r\n  second licence line\r\nKeywords: astrum,probe\r\nClassifier: Programming Language :: Python :: 3\r\nClassifier: License :: OSI Approved :: MIT License\r\nRequires-Python: >=3.8\r\nDescription-Content-Type: text/markdown\r\nUnknown-Header: kept out\r\n\r\n# Probe\r\n\r\nLong description.\r\n";

    #[test]
    fn core_metadata_parses_headers_and_body() {
        let metadata = CoreMetadata::parse(PKG_INFO);
        assert_eq!(metadata.name(), Some("Astrum_Probe.Pkg"));
        assert_eq!(metadata.version(), Some("0.2.0"));
        assert_eq!(metadata.get("requires_python"), Some(">=3.8"));
        assert_eq!(metadata.get("license"), Some("MIT\nsecond licence line"));
        assert_eq!(
            metadata.get("description"),
            Some("# Probe\n\nLong description.")
        );
        assert_eq!(metadata.multiple["classifiers"].len(), 2);
        let asset = metadata.asset_attributes();
        assert_eq!(
            asset,
            json!({"pypi": {"name": "Astrum_Probe.Pkg", "version": "0.2.0", "summary": "Probe package", "requires_python": ">=3.8"}})
        );
        let component = metadata.component_attributes();
        assert_eq!(component["pypi"]["keywords"], "astrum,probe");
        assert_eq!(
            component["pypi"]["classifiers"][0],
            "Programming Language :: Python :: 3"
        );
        assert!(component["pypi"].get("metadata_version").is_none());
    }

    #[test]
    fn old_style_descriptions_in_headers_unfold() {
        let metadata = CoreMetadata::parse(
            "Metadata-Version: 1.1\nName: old\nVersion: 1.0\nDescription: first\n        |second\n        third\nPlatform: UNKNOWN\n",
        );
        assert_eq!(metadata.get("description"), Some("first\nsecond\nthird"));
        assert_eq!(metadata.multiple["platform"], ["UNKNOWN"]);
    }

    #[test]
    fn twine_forms_map_to_the_same_keys() {
        let metadata = CoreMetadata::from_form([
            ("name", "Astrum-Probe.Pkg"),
            ("version", "0.1.0"),
            ("summary", "Probe"),
            ("requires_python", ">=3.8"),
            ("classifiers", "A :: B"),
            ("classifiers", "C :: D"),
            ("license", ""),
            (":action", "file_upload"),
        ]);
        assert_eq!(metadata.name(), Some("Astrum-Probe.Pkg"));
        assert_eq!(metadata.multiple["classifiers"], ["A :: B", "C :: D"]);
        assert_eq!(metadata.get("license"), None);
        assert_eq!(
            metadata.asset_attributes()["pypi"]["requires_python"],
            ">=3.8"
        );
    }
}
