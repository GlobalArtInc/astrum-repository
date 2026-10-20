use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::names::{self, BoxFile, BoxName, UNKNOWN};

pub const ACTIVE: &str = "active";
const PREFERRED: &str = "amd64";
const CHECKSUM_TYPES: &[&str] = &["md5", "sha1", "sha256", "sha384", "sha512"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Provider {
    pub name: String,
    pub architecture: String,
    pub default_architecture: Option<bool>,
    pub checksum_type: Option<String>,
    pub checksum: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub status: Option<String>,
    pub description: Option<String>,
    pub providers: Vec<Provider>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub boxed: BoxName,
    pub description: Option<String>,
    pub releases: Vec<Release>,
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

impl Provider {
    fn file(&self, boxed: &BoxName, version: &str) -> BoxFile {
        BoxFile {
            boxed: boxed.clone(),
            version: version.to_owned(),
            provider: self.name.clone(),
            architecture: self.architecture.clone(),
        }
    }

    fn render(&self, repository_url: &str, boxed: &BoxName, version: &str) -> Value {
        let mut object = Map::new();
        object.insert("name".into(), json!(self.name));
        object.insert(
            "url".into(),
            json!(format!(
                "{repository_url}{}",
                self.file(boxed, version).path()
            )),
        );
        if let (Some(kind), Some(checksum)) = (&self.checksum_type, &self.checksum) {
            object.insert("checksum_type".into(), json!(kind));
            object.insert("checksum".into(), json!(checksum));
        }
        object.insert("architecture".into(), json!(self.architecture));
        object.insert(
            "default_architecture".into(),
            json!(self.default_architecture.unwrap_or(false)),
        );
        Value::Object(object)
    }

    fn parse(value: &Value, base: Option<&url::Url>) -> Option<Provider> {
        let name = text(value.get("name"))?;
        if !names::valid_provider(&name) {
            return None;
        }
        let architecture = text(value.get("architecture")).unwrap_or_else(|| UNKNOWN.to_owned());
        if !names::valid_architecture(&architecture) {
            return None;
        }
        let checksum_type = text(value.get("checksum_type"))
            .map(|kind| kind.to_ascii_lowercase())
            .filter(|kind| CHECKSUM_TYPES.contains(&kind.as_str()));
        let checksum = text(value.get("checksum"))
            .filter(|checksum| checksum.chars().all(|c| c.is_ascii_hexdigit()))
            .map(|checksum| checksum.to_ascii_lowercase());
        let (checksum_type, checksum) = match (checksum_type, checksum) {
            (Some(kind), Some(checksum)) => (Some(kind), Some(checksum)),
            _ => (None, None),
        };
        let url = text(value.get("url")).and_then(|address| {
            let resolved = match base {
                Some(base) => base.join(&address).ok(),
                None => url::Url::parse(&address).ok(),
            }?;
            matches!(resolved.scheme(), "http" | "https").then(|| resolved.to_string())
        });
        Some(Provider {
            name,
            architecture,
            default_architecture: value.get("default_architecture").and_then(Value::as_bool),
            checksum_type,
            checksum,
            url,
        })
    }
}

impl Release {
    fn render(&self, repository_url: &str, boxed: &BoxName) -> Value {
        let mut object = Map::new();
        object.insert("version".into(), json!(self.version));
        object.insert(
            "status".into(),
            json!(self.status.as_deref().unwrap_or(ACTIVE)),
        );
        if let Some(description) = &self.description {
            object.insert("description".into(), json!(description));
        }
        object.insert(
            "providers".into(),
            Value::Array(
                self.providers
                    .iter()
                    .map(|provider| provider.render(repository_url, boxed, &self.version))
                    .collect(),
            ),
        );
        Value::Object(object)
    }

    fn has(&self, name: &str, architecture: &str) -> bool {
        self.providers
            .iter()
            .any(|provider| provider.name == name && provider.architecture == architecture)
    }

    fn settle(&mut self) {
        let names: Vec<String> = self
            .providers
            .iter()
            .map(|provider| provider.name.clone())
            .collect();
        for name in names {
            let indices: Vec<usize> = (0..self.providers.len())
                .filter(|index| self.providers[*index].name == name)
                .collect();
            let chosen = indices
                .iter()
                .copied()
                .find(|index| self.providers[*index].default_architecture == Some(true))
                .or_else(|| {
                    [PREFERRED, UNKNOWN].iter().find_map(|wanted| {
                        indices
                            .iter()
                            .copied()
                            .find(|index| self.providers[*index].architecture == *wanted)
                    })
                })
                .or_else(|| indices.first().copied());
            for index in indices {
                self.providers[index].default_architecture = Some(Some(index) == chosen);
            }
        }
    }
}

impl Catalog {
    pub fn new(boxed: BoxName) -> Self {
        Self {
            boxed,
            description: None,
            releases: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.releases.is_empty()
    }

    pub fn find(&self, file: &BoxFile) -> Option<&Provider> {
        self.releases
            .iter()
            .find(|release| release.version == file.version)?
            .providers
            .iter()
            .find(|provider| {
                provider.name == file.provider && provider.architecture == file.architecture
            })
    }

    pub fn add(&mut self, version: &str, description: Option<String>, provider: Provider) {
        let index = match self
            .releases
            .iter()
            .position(|release| release.version == version)
        {
            Some(index) => index,
            None => {
                self.releases.push(Release {
                    version: version.to_owned(),
                    ..Release::default()
                });
                self.releases.len() - 1
            }
        };
        let release = &mut self.releases[index];
        if release.description.is_none() {
            release.description = description;
        }
        if !release.has(&provider.name, &provider.architecture) {
            release.providers.push(provider);
        }
    }

    pub fn finish(&mut self) {
        self.releases
            .retain(|release| !release.providers.is_empty());
        self.releases
            .sort_by(|a, b| names::compare(&b.version, &a.version));
        for release in &mut self.releases {
            release.providers.sort_by(|a, b| {
                a.name
                    .cmp(&b.name)
                    .then_with(|| a.architecture.cmp(&b.architecture))
            });
            release.settle();
        }
        if self.description.is_none() {
            self.description = self
                .releases
                .iter()
                .find_map(|release| release.description.clone());
        }
    }

    pub fn retain(&mut self, keep: impl Fn(&BoxFile) -> bool) {
        let boxed = self.boxed.clone();
        for release in &mut self.releases {
            let version = release.version.clone();
            release
                .providers
                .retain(|provider| keep(&provider.file(&boxed, &version)));
        }
        self.finish();
    }

    pub fn render(&self, repository_url: &str) -> Value {
        json!({
            "name": self.boxed.tag(),
            "description": self.description.clone().unwrap_or_default(),
            "versions": self
                .releases
                .iter()
                .map(|release| release.render(repository_url, &self.boxed))
                .collect::<Vec<_>>(),
        })
    }

    pub fn parse(boxed: BoxName, document: &Value, base: Option<&url::Url>) -> Catalog {
        let mut catalog = Catalog::new(boxed);
        catalog.description =
            text(document.get("description")).or_else(|| text(document.get("short_description")));
        for value in document
            .get("versions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(version) = text(value.get("version")).filter(|v| names::valid_version(v))
            else {
                continue;
            };
            if catalog
                .releases
                .iter()
                .any(|release| release.version == version)
            {
                continue;
            }
            let mut release = Release {
                version,
                status: text(value.get("status")),
                description: text(value.get("description"))
                    .or_else(|| text(value.get("description_markdown"))),
                providers: Vec::new(),
            };
            for provider in value
                .get("providers")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|provider| Provider::parse(provider, base))
            {
                if provider.url.is_some() && !release.has(&provider.name, &provider.architecture) {
                    release.providers.push(provider);
                }
            }
            catalog.releases.push(release);
        }
        catalog.finish();
        catalog
    }

    pub fn merge(boxed: &BoxName, catalogs: Vec<Catalog>) -> Option<Catalog> {
        let mut merged: Option<Catalog> = None;
        for catalog in catalogs.into_iter().filter(|catalog| !catalog.is_empty()) {
            let target = merged.get_or_insert_with(|| Catalog::new(boxed.clone()));
            if target.description.is_none() {
                target.description = catalog.description.clone();
            }
            let mut statuses: BTreeMap<String, Option<String>> = BTreeMap::new();
            for release in catalog.releases {
                statuses.insert(release.version.clone(), release.status.clone());
                for provider in release.providers {
                    target.add(&release.version, release.description.clone(), provider);
                }
            }
            for release in &mut target.releases {
                if release.status.is_none()
                    && let Some(status) = statuses.get(&release.version)
                {
                    release.status = status.clone();
                }
            }
        }
        let mut merged = merged?;
        merged.finish();
        Some(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed() -> BoxName {
        BoxName::new("acme", "base").unwrap()
    }

    fn provider(name: &str, architecture: &str) -> Provider {
        Provider {
            name: name.to_owned(),
            architecture: architecture.to_owned(),
            checksum_type: Some("sha256".to_owned()),
            checksum: Some("ab".repeat(32)),
            ..Provider::default()
        }
    }

    #[test]
    fn catalogs_render_box_metadata_with_urls_of_this_repository() {
        let mut catalog = Catalog::new(boxed());
        catalog.add("1.0.0", None, provider("virtualbox", "arm64"));
        catalog.add("1.0.0", None, provider("virtualbox", "amd64"));
        catalog.add(
            "1.10.0",
            Some("Faster".to_owned()),
            provider("libvirt", "arm64"),
        );
        catalog.add("1.2.0", None, provider("virtualbox", "i386"));
        catalog.add("1.2.0", None, provider("virtualbox", "unknown"));
        catalog.finish();
        let document = catalog.render("https://repo.example/repository/boxes");
        assert_eq!(document["name"], "acme/base");
        assert_eq!(document["description"], "Faster");
        let versions: Vec<&str> = document["versions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|version| version["version"].as_str().unwrap())
            .collect();
        assert_eq!(versions, ["1.10.0", "1.2.0", "1.0.0"]);
        assert_eq!(document["versions"][0]["status"], "active");
        assert_eq!(document["versions"][0]["description"], "Faster");
        assert_eq!(
            document["versions"][0]["providers"][0],
            json!({
                "name": "libvirt",
                "url": "https://repo.example/repository/boxes/acme/base/1.10.0/libvirt/arm64.box",
                "checksum_type": "sha256",
                "checksum": "ab".repeat(32),
                "architecture": "arm64",
                "default_architecture": true,
            })
        );
        let defaults = |index: usize| -> Vec<(String, bool)> {
            document["versions"][index]["providers"]
                .as_array()
                .unwrap()
                .iter()
                .map(|provider| {
                    (
                        provider["architecture"].as_str().unwrap().to_owned(),
                        provider["default_architecture"].as_bool().unwrap(),
                    )
                })
                .collect()
        };
        assert_eq!(
            defaults(1),
            [("i386".to_owned(), false), ("unknown".to_owned(), true)]
        );
        assert_eq!(
            defaults(2),
            [("amd64".to_owned(), true), ("arm64".to_owned(), false)]
        );
    }

    #[test]
    fn remote_metadata_parses_and_resolves_its_urls() {
        let base = url::Url::parse("https://boxes.example/acme/base").unwrap();
        let remote = json!({
            "name": "acme/base",
            "description": "",
            "short_description": "A base box",
            "versions": [
                {"version": "2.0.0", "status": "active", "providers": [
                    {"name": "virtualbox", "architecture": "amd64", "default_architecture": false, "checksum_type": "SHA256", "checksum": "AB".repeat(32), "url": "https://cdn.example/a.box"},
                    {"name": "virtualbox", "architecture": "arm64", "default_architecture": true, "checksum_type": "none", "checksum": "", "url": "https://cdn.example/b.box"},
                    {"name": "virtualbox", "architecture": "arm64", "url": "https://cdn.example/again.box"},
                    {"name": "hyperv", "architecture": "amd64"},
                    {"name": "bad/name", "url": "https://cdn.example/c.box"},
                    {"name": "ftp", "url": "ftp://cdn.example/c.box"}
                ]},
                {"version": "1.0.0", "providers": [{"name": "libvirt", "url": "/files/legacy.box", "checksum_type": "sha1", "checksum": "0a".repeat(20)}]},
                {"version": "latest", "providers": [{"name": "libvirt", "url": "https://cdn.example/d.box"}]},
                {"version": "2.0.0", "providers": [{"name": "qemu", "url": "https://cdn.example/e.box"}]},
                {"version": "0.1.0", "providers": []}
            ]
        });
        let parsed = Catalog::parse(boxed(), &remote, Some(&base));
        assert_eq!(parsed.description.as_deref(), Some("A base box"));
        assert_eq!(parsed.releases.len(), 2);
        let newest = &parsed.releases[0];
        assert_eq!(newest.version, "2.0.0");
        assert_eq!(newest.providers.len(), 2);
        assert_eq!(
            newest.providers[0].checksum.as_deref(),
            Some("ab".repeat(32).as_str())
        );
        assert_eq!(newest.providers[0].default_architecture, Some(false));
        assert_eq!(newest.providers[1].checksum_type, None);
        assert_eq!(newest.providers[1].default_architecture, Some(true));
        let legacy = &parsed.releases[1].providers[0];
        assert_eq!(legacy.architecture, "unknown");
        assert_eq!(legacy.default_architecture, Some(true));
        assert_eq!(
            legacy.url.as_deref(),
            Some("https://boxes.example/files/legacy.box")
        );
    }

    #[test]
    fn groups_merge_versions_first_member_first() {
        let mut own = Catalog::new(boxed());
        own.add(
            "1.0.0",
            Some("Own".to_owned()),
            provider("virtualbox", "amd64"),
        );
        own.finish();
        let mut remote = Catalog::new(boxed());
        remote.description = Some("Remote".to_owned());
        remote.add(
            "1.0.0",
            None,
            Provider {
                checksum: Some("cd".repeat(32)),
                ..provider("virtualbox", "amd64")
            },
        );
        remote.add(
            "1.0.0",
            None,
            Provider {
                default_architecture: Some(true),
                ..provider("virtualbox", "arm64")
            },
        );
        remote.add("0.9.0", None, provider("libvirt", "amd64"));
        let merged = Catalog::merge(&boxed(), vec![Catalog::new(boxed()), own, remote]).unwrap();
        assert_eq!(merged.description.as_deref(), Some("Own"));
        assert_eq!(merged.releases.len(), 2);
        let first = &merged.releases[0];
        assert_eq!(first.providers.len(), 2);
        assert_eq!(
            first.providers[0].checksum.as_deref(),
            Some("ab".repeat(32).as_str())
        );
        let defaults: Vec<bool> = first
            .providers
            .iter()
            .map(|provider| provider.default_architecture.unwrap())
            .collect();
        assert_eq!(defaults, [true, false]);
        assert!(Catalog::merge(&boxed(), vec![Catalog::new(boxed())]).is_none());
    }

    #[test]
    fn retained_entries_drop_empty_versions() {
        let mut catalog = Catalog::new(boxed());
        catalog.add("1.0.0", None, provider("virtualbox", "amd64"));
        catalog.add("1.0.0", None, provider("virtualbox", "arm64"));
        catalog.add("2.0.0", None, provider("virtualbox", "amd64"));
        catalog.finish();
        catalog.retain(|file| file.architecture == "arm64");
        assert_eq!(catalog.releases.len(), 1);
        assert_eq!(
            catalog.releases[0].providers[0].default_architecture,
            Some(true)
        );
    }
}
