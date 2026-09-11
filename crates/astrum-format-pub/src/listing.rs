use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::version;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entry {
    pub version: String,
    pub retracted: bool,
    pub sha256: Option<String>,
    pub published: Option<String>,
    pub pubspec: Value,
    pub archive_url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listing {
    pub name: String,
    pub discontinued: bool,
    pub replaced_by: Option<String>,
    pub advisories_updated: Option<String>,
    pub versions: Vec<Entry>,
}

pub fn archive_path(name: &str, version: &str) -> String {
    format!("/packages/{name}/versions/{version}.tar.gz")
}

pub fn archive_url(repository_url: &str, name: &str, version: &str) -> String {
    format!("{repository_url}{}", archive_path(name, version))
}

impl Entry {
    fn render(&self, repository_url: &str, name: &str) -> Value {
        let mut object = Map::new();
        object.insert("version".into(), json!(self.version));
        if self.retracted {
            object.insert("retracted".into(), json!(true));
        }
        object.insert(
            "archive_url".into(),
            json!(archive_url(repository_url, name, &self.version)),
        );
        if let Some(sha256) = &self.sha256 {
            object.insert("archive_sha256".into(), json!(sha256));
        }
        object.insert("pubspec".into(), self.pubspec.clone());
        if let Some(published) = &self.published {
            object.insert("published".into(), json!(published));
        }
        Value::Object(object)
    }

    fn parse(value: &Value) -> Option<Entry> {
        let version = value.get("version")?.as_str()?.to_owned();
        if !version::valid(&version) {
            return None;
        }
        Some(Entry {
            version,
            retracted: value
                .get("retracted")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            sha256: value
                .get("archive_sha256")
                .and_then(Value::as_str)
                .filter(|sha256| {
                    sha256.len() == 64 && sha256.chars().all(|c| c.is_ascii_hexdigit())
                })
                .map(str::to_ascii_lowercase),
            published: value
                .get("published")
                .and_then(Value::as_str)
                .map(str::to_owned),
            pubspec: value
                .get("pubspec")
                .filter(|pubspec| pubspec.is_object())
                .cloned()
                .unwrap_or_else(|| json!({})),
            archive_url: value
                .get("archive_url")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }
}

impl Listing {
    pub fn sort(&mut self) {
        self.versions
            .sort_by(|a, b| version::compare(&a.version, &b.version));
    }

    pub fn latest(&self) -> Option<&Entry> {
        let parsed: Vec<(&Entry, version::Version)> = self
            .versions
            .iter()
            .filter_map(|entry| Some((entry, version::Version::parse(&entry.version)?)))
            .collect();
        let any_live = parsed.iter().any(|(entry, _)| !entry.retracted);
        let any_stable = parsed
            .iter()
            .any(|(entry, version)| (!any_live || !entry.retracted) && !version.is_prerelease());
        let mut best: Option<&(&Entry, version::Version)> = None;
        for candidate in &parsed {
            let (entry, version) = candidate;
            if any_live && entry.retracted {
                continue;
            }
            if any_stable && version.is_prerelease() {
                continue;
            }
            if best.is_none_or(|(_, known)| version > known) {
                best = Some(candidate);
            }
        }
        best.map(|(entry, _)| *entry)
    }

    pub fn find(&self, version: &str) -> Option<&Entry> {
        self.versions.iter().find(|entry| entry.version == version)
    }

    pub fn render(&self, repository_url: &str) -> Value {
        let mut object = Map::new();
        object.insert("name".into(), json!(self.name));
        if self.discontinued {
            object.insert("isDiscontinued".into(), json!(true));
            if let Some(replaced_by) = &self.replaced_by {
                object.insert("replacedBy".into(), json!(replaced_by));
            }
        }
        if let Some(updated) = &self.advisories_updated {
            object.insert("advisoriesUpdated".into(), json!(updated));
        }
        if let Some(latest) = self.latest() {
            object.insert("latest".into(), latest.render(repository_url, &self.name));
        }
        object.insert(
            "versions".into(),
            Value::Array(
                self.versions
                    .iter()
                    .map(|entry| entry.render(repository_url, &self.name))
                    .collect(),
            ),
        );
        Value::Object(object)
    }

    pub fn render_version(&self, repository_url: &str, version: &str) -> Option<Value> {
        self.find(version)
            .map(|entry| entry.render(repository_url, &self.name))
    }

    pub fn parse(name: &str, document: &Value) -> Listing {
        let mut listing = Listing {
            name: name.to_owned(),
            discontinued: document
                .get("isDiscontinued")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            replaced_by: document
                .get("replacedBy")
                .and_then(Value::as_str)
                .map(str::to_owned),
            advisories_updated: document
                .get("advisoriesUpdated")
                .and_then(Value::as_str)
                .map(str::to_owned),
            versions: Vec::new(),
        };
        for value in document
            .get("versions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(entry) = Entry::parse(value)
                && listing.find(&entry.version).is_none()
            {
                listing.versions.push(entry);
            }
        }
        listing.sort();
        listing
    }

    pub fn merge(name: &str, listings: Vec<Listing>) -> Option<Listing> {
        let mut merged: Option<Listing> = None;
        let mut versions: BTreeMap<String, Entry> = BTreeMap::new();
        for listing in listings {
            if listing.versions.is_empty() {
                continue;
            }
            let target = merged.get_or_insert_with(|| Listing {
                name: name.to_owned(),
                discontinued: listing.discontinued,
                replaced_by: listing.replaced_by.clone(),
                ..Listing::default()
            });
            target.advisories_updated =
                match (target.advisories_updated.take(), listing.advisories_updated) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
            for entry in listing.versions {
                versions.entry(entry.version.clone()).or_insert(entry);
            }
        }
        let mut merged = merged?;
        merged.versions = versions.into_values().collect();
        merged.sort();
        Some(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(version: &str) -> Entry {
        Entry {
            version: version.to_owned(),
            pubspec: json!({"name": "hello", "version": version}),
            ..Entry::default()
        }
    }

    #[test]
    fn listings_render_the_v2_document_with_archive_urls_of_this_repository() {
        let mut listing = Listing {
            name: "hello".to_owned(),
            versions: vec![
                Entry {
                    sha256: Some("ab".repeat(32)),
                    published: Some("2026-10-18T08:00:00.000Z".to_owned()),
                    ..entry("1.1.0")
                },
                entry("1.0.0"),
                Entry {
                    retracted: true,
                    ..entry("1.2.0")
                },
                entry("2.0.0-dev.1"),
            ],
            ..Listing::default()
        };
        listing.sort();
        let document = listing.render("https://repo.example/repository/pub");
        assert_eq!(document["name"], "hello");
        assert_eq!(document["latest"]["version"], "1.1.0");
        assert_eq!(
            document["latest"]["archive_url"],
            "https://repo.example/repository/pub/packages/hello/versions/1.1.0.tar.gz"
        );
        assert_eq!(document["latest"]["archive_sha256"], "ab".repeat(32));
        assert_eq!(document["latest"]["published"], "2026-10-18T08:00:00.000Z");
        let versions: Vec<&str> = document["versions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["version"].as_str().unwrap())
            .collect();
        assert_eq!(versions, ["1.0.0", "1.1.0", "1.2.0", "2.0.0-dev.1"]);
        assert_eq!(document["versions"][2]["retracted"], true);
        assert!(document["versions"][0].get("retracted").is_none());
        assert!(document.get("isDiscontinued").is_none());
        listing.discontinued = true;
        listing.replaced_by = Some("hello2".to_owned());
        let document = listing.render("https://repo.example/repository/pub");
        assert_eq!(document["isDiscontinued"], true);
        assert_eq!(document["replacedBy"], "hello2");
    }

    #[test]
    fn the_latest_version_is_the_newest_live_release() {
        let only_pre = Listing {
            name: "x".to_owned(),
            versions: vec![entry("1.0.0-dev.1"), entry("1.0.0-dev.2")],
            ..Listing::default()
        };
        assert_eq!(only_pre.latest().unwrap().version, "1.0.0-dev.2");
        let all_retracted = Listing {
            name: "x".to_owned(),
            versions: vec![
                Entry {
                    retracted: true,
                    ..entry("1.0.0")
                },
                Entry {
                    retracted: true,
                    ..entry("0.9.0")
                },
            ],
            ..Listing::default()
        };
        assert_eq!(all_retracted.latest().unwrap().version, "1.0.0");
        assert!(Listing::default().latest().is_none());
    }

    #[test]
    fn remote_listings_parse_and_groups_merge_first_member_first() {
        let remote = json!({
            "name": "hello",
            "isDiscontinued": true,
            "replacedBy": "hi",
            "advisoriesUpdated": "2026-01-01T00:00:00.000Z",
            "latest": {"version": "1.0.0"},
            "versions": [
                {"version": "1.0.0", "archive_url": "https://pub.example/api/archives/hello-1.0.0.tar.gz", "archive_sha256": "AB".repeat(32), "pubspec": {"name": "hello"}},
                {"version": "0.1.0", "retracted": true, "archive_url": "https://pub.example/x.tar.gz", "pubspec": {"name": "hello"}},
                {"version": "not-a-version"},
                {"version": "1.0.0"}
            ]
        });
        let parsed = Listing::parse("hello", &remote);
        assert!(parsed.discontinued);
        assert_eq!(parsed.replaced_by.as_deref(), Some("hi"));
        assert_eq!(parsed.versions.len(), 2);
        assert_eq!(parsed.versions[0].version, "0.1.0");
        assert!(parsed.versions[0].retracted);
        assert_eq!(
            parsed.versions[1].sha256.as_deref(),
            Some("ab".repeat(32).as_str())
        );
        assert_eq!(
            parsed.versions[1].archive_url.as_deref(),
            Some("https://pub.example/api/archives/hello-1.0.0.tar.gz")
        );
        let own = Listing {
            name: "hello".to_owned(),
            versions: vec![Entry {
                sha256: Some("cd".repeat(32)),
                ..entry("1.0.0")
            }],
            ..Listing::default()
        };
        let merged = Listing::merge("hello", vec![Listing::default(), own, parsed]).unwrap();
        assert!(!merged.discontinued);
        assert_eq!(
            merged.advisories_updated.as_deref(),
            Some("2026-01-01T00:00:00.000Z")
        );
        assert_eq!(merged.versions.len(), 2);
        assert_eq!(
            merged.find("1.0.0").unwrap().sha256.as_deref(),
            Some("cd".repeat(32).as_str())
        );
        assert!(Listing::merge("hello", vec![Listing::default()]).is_none());
    }
}
