use std::cmp::Ordering;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

use crate::version;
use crate::xml::{self, Element, XmlError, line};

pub const MODEL_VERSION: &str = "1.1.0";
const TIMESTAMP: &str = "%Y%m%d%H%M%S";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    pub group_id: Option<String>,
    pub artifact_id: Option<String>,
    pub version: Option<String>,
    pub versioning: Option<Versioning>,
    pub plugins: Vec<Plugin>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Versioning {
    pub latest: Option<String>,
    pub release: Option<String>,
    pub snapshot: Option<Snapshot>,
    pub versions: Vec<String>,
    pub last_updated: Option<String>,
    pub snapshot_versions: Vec<SnapshotVersion>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub timestamp: Option<String>,
    pub build_number: Option<u32>,
    pub local_copy: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SnapshotVersion {
    pub classifier: Option<String>,
    pub extension: String,
    pub value: String,
    pub updated: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plugin {
    pub name: Option<String>,
    pub prefix: String,
    pub artifact_id: String,
}

pub fn timestamp(time: DateTime<Utc>) -> String {
    time.format(TIMESTAMP).to_string()
}

pub fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(value, TIMESTAMP)
        .ok()
        .map(|naive| Utc.from_utc_datetime(&naive))
}

impl Versioning {
    fn is_empty(&self) -> bool {
        *self == Versioning::default()
    }
}

impl Snapshot {
    fn newer_than(&self, other: &Snapshot) -> bool {
        (self.timestamp.as_deref(), self.build_number)
            > (other.timestamp.as_deref(), other.build_number)
    }
}

impl SnapshotVersion {
    fn key(&self) -> (Option<&str>, &str) {
        (self.classifier.as_deref(), self.extension.as_str())
    }

    fn newer_than(&self, other: &SnapshotVersion) -> bool {
        let stamp = |value: &str| {
            version::timestamped(value).map(|found| (found.timestamp.to_owned(), found.build))
        };
        (self.updated.as_deref(), stamp(&self.value))
            > (other.updated.as_deref(), stamp(&other.value))
    }
}

pub fn parse(bytes: &[u8]) -> Result<Metadata, XmlError> {
    let root = xml::parse(bytes)?;
    if root.name != "metadata" {
        return Err(XmlError::Syntax(format!(
            "the root element is {}, not metadata",
            root.name
        )));
    }
    Ok(Metadata {
        group_id: root.text_of("groupId"),
        artifact_id: root.text_of("artifactId"),
        version: root.text_of("version"),
        versioning: root.child("versioning").map(versioning),
        plugins: root
            .child("plugins")
            .map(|plugins| {
                plugins
                    .children("plugin")
                    .filter_map(|plugin| {
                        Some(Plugin {
                            name: plugin.text_of("name"),
                            prefix: plugin.text_of("prefix")?,
                            artifact_id: plugin.text_of("artifactId")?,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

fn versioning(element: &Element) -> Versioning {
    Versioning {
        latest: element.text_of("latest"),
        release: element.text_of("release"),
        snapshot: element.child("snapshot").map(|snapshot| Snapshot {
            timestamp: snapshot.text_of("timestamp"),
            build_number: snapshot
                .text_of("buildNumber")
                .and_then(|number| number.parse().ok()),
            local_copy: snapshot.text_of("localCopy").as_deref() == Some("true"),
        }),
        versions: element
            .child("versions")
            .map(|versions| {
                versions
                    .children("version")
                    .filter_map(Element::value)
                    .collect()
            })
            .unwrap_or_default(),
        last_updated: element.text_of("lastUpdated"),
        snapshot_versions: element
            .child("snapshotVersions")
            .map(|versions| {
                versions
                    .children("snapshotVersion")
                    .filter_map(|entry| {
                        Some(SnapshotVersion {
                            classifier: entry.text_of("classifier"),
                            extension: entry.text_of("extension")?,
                            value: entry.text_of("value")?,
                            updated: entry.text_of("updated"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

pub fn write(metadata: &Metadata) -> String {
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata modelVersion=\"{MODEL_VERSION}\">\n"
    );
    line(&mut out, 1, "groupId", metadata.group_id.as_deref());
    line(&mut out, 1, "artifactId", metadata.artifact_id.as_deref());
    line(&mut out, 1, "version", metadata.version.as_deref());
    if let Some(versioning) = metadata
        .versioning
        .as_ref()
        .filter(|found| !found.is_empty())
    {
        out.push_str("  <versioning>\n");
        line(&mut out, 2, "latest", versioning.latest.as_deref());
        line(&mut out, 2, "release", versioning.release.as_deref());
        if let Some(snapshot) = &versioning.snapshot {
            out.push_str("    <snapshot>\n");
            line(&mut out, 3, "timestamp", snapshot.timestamp.as_deref());
            line(
                &mut out,
                3,
                "buildNumber",
                snapshot
                    .build_number
                    .map(|number| number.to_string())
                    .as_deref(),
            );
            line(
                &mut out,
                3,
                "localCopy",
                snapshot.local_copy.then_some("true"),
            );
            out.push_str("    </snapshot>\n");
        }
        if !versioning.versions.is_empty() {
            out.push_str("    <versions>\n");
            for version in &versioning.versions {
                line(&mut out, 3, "version", Some(version));
            }
            out.push_str("    </versions>\n");
        }
        line(
            &mut out,
            2,
            "lastUpdated",
            versioning.last_updated.as_deref(),
        );
        if !versioning.snapshot_versions.is_empty() {
            out.push_str("    <snapshotVersions>\n");
            for entry in &versioning.snapshot_versions {
                out.push_str("      <snapshotVersion>\n");
                line(&mut out, 4, "classifier", entry.classifier.as_deref());
                line(&mut out, 4, "extension", Some(&entry.extension));
                line(&mut out, 4, "value", Some(&entry.value));
                line(&mut out, 4, "updated", entry.updated.as_deref());
                out.push_str("      </snapshotVersion>\n");
            }
            out.push_str("    </snapshotVersions>\n");
        }
        out.push_str("  </versioning>\n");
    }
    if !metadata.plugins.is_empty() {
        out.push_str("  <plugins>\n");
        for plugin in &metadata.plugins {
            out.push_str("    <plugin>\n");
            line(&mut out, 3, "name", plugin.name.as_deref());
            line(&mut out, 3, "prefix", Some(&plugin.prefix));
            line(&mut out, 3, "artifactId", Some(&plugin.artifact_id));
            out.push_str("    </plugin>\n");
        }
        out.push_str("  </plugins>\n");
    }
    out.push_str("</metadata>\n");
    out
}

pub fn versions(mut versions: Vec<String>, last_updated: Option<String>) -> Versioning {
    version::sort(&mut versions);
    versions.dedup();
    Versioning {
        latest: versions.last().cloned(),
        release: versions
            .iter()
            .rev()
            .find(|candidate| !version::is_snapshot(candidate))
            .cloned(),
        versions,
        last_updated,
        ..Default::default()
    }
}

fn newest(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    }
}

fn highest(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => match version::compare(&left, &right) {
            Ordering::Less => Some(right),
            _ => Some(left),
        },
        (left, right) => left.or(right),
    }
}

pub fn merge(documents: impl IntoIterator<Item = Metadata>) -> Option<Metadata> {
    let mut documents = documents.into_iter();
    let mut merged = documents.next()?;
    for document in documents {
        merged.group_id = merged.group_id.or(document.group_id);
        merged.artifact_id = merged.artifact_id.or(document.artifact_id);
        merged.version = merged.version.or(document.version);
        for plugin in document.plugins {
            if !merged
                .plugins
                .iter()
                .any(|known| known.prefix == plugin.prefix)
            {
                merged.plugins.push(plugin);
            }
        }
        merged.versioning = match (merged.versioning.take(), document.versioning) {
            (Some(left), Some(right)) => Some(merge_versioning(left, right)),
            (left, right) => left.or(right),
        };
    }
    if let Some(versioning) = merged.versioning.as_mut() {
        let mut versions = std::mem::take(&mut versioning.versions);
        version::sort(&mut versions);
        versions.dedup();
        versioning.latest = versions.last().cloned().or(versioning.latest.take());
        versioning.release = versions
            .iter()
            .rev()
            .find(|candidate| !version::is_snapshot(candidate))
            .cloned()
            .or(versioning.release.take());
        versioning.versions = versions;
    }
    Some(merged)
}

fn merge_versioning(mut left: Versioning, right: Versioning) -> Versioning {
    left.latest = highest(left.latest, right.latest);
    left.release = highest(left.release, right.release);
    left.versions.extend(right.versions);
    left.last_updated = newest(left.last_updated, right.last_updated);
    left.snapshot = match (left.snapshot.take(), right.snapshot) {
        (Some(left), Some(right)) => Some(if right.newer_than(&left) { right } else { left }),
        (left, right) => left.or(right),
    };
    for entry in right.snapshot_versions {
        match left
            .snapshot_versions
            .iter_mut()
            .find(|known| known.key() == entry.key())
        {
            Some(known) => {
                if entry.newer_than(known) {
                    *known = entry;
                }
            }
            None => left.snapshot_versions.push(entry),
        }
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
  <groupId>org.example</groupId>
  <artifactId>lib</artifactId>
  <versioning>
    <release>9.9</release>
    <versions><version>9.9</version><version>1.0</version></versions>
    <lastUpdated>20200101000000</lastUpdated>
  </versioning>
</metadata>"#;

    const SNAPSHOTS: &str = r#"<metadata modelVersion="1.1.0">
  <groupId>org.example</groupId>
  <artifactId>lib</artifactId>
  <versioning>
    <latest>1.0-SNAPSHOT</latest>
    <versions><version>1.0-SNAPSHOT</version><version>10.0</version></versions>
    <lastUpdated>20261012101010</lastUpdated>
  </versioning>
</metadata>"#;

    #[test]
    fn metadata_round_trips() {
        let parsed = parse(RELEASES.as_bytes()).unwrap();
        assert_eq!(parsed.group_id.as_deref(), Some("org.example"));
        let versioning = parsed.versioning.as_ref().unwrap();
        assert_eq!(versioning.versions, ["9.9", "1.0"]);
        assert_eq!(versioning.release.as_deref(), Some("9.9"));
        assert_eq!(parse(write(&parsed).as_bytes()).unwrap(), parsed);
        assert!(parse(b"<project/>").is_err());
        assert!(parse(b"garbage <not xml").is_err());
    }

    #[test]
    fn generated_versions_name_latest_and_release() {
        let versioning = versions(
            vec![
                "1.10".to_owned(),
                "1.9".to_owned(),
                "2.0-SNAPSHOT".to_owned(),
                "1.9".to_owned(),
            ],
            Some("20261012101010".to_owned()),
        );
        assert_eq!(versioning.versions, ["1.9", "1.10", "2.0-SNAPSHOT"]);
        assert_eq!(versioning.latest.as_deref(), Some("2.0-SNAPSHOT"));
        assert_eq!(versioning.release.as_deref(), Some("1.10"));
        let written = write(&Metadata {
            group_id: Some("com.acme".to_owned()),
            artifact_id: Some("tool".to_owned()),
            versioning: Some(versions(
                vec!["2.1".to_owned()],
                Some("20261012144758".to_owned()),
            )),
            ..Default::default()
        });
        assert_eq!(
            written,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata modelVersion=\"1.1.0\">\n  <groupId>com.acme</groupId>\n  <artifactId>tool</artifactId>\n  <versioning>\n    <latest>2.1</latest>\n    <release>2.1</release>\n    <versions>\n      <version>2.1</version>\n    </versions>\n    <lastUpdated>20261012144758</lastUpdated>\n  </versioning>\n</metadata>\n"
        );
    }

    #[test]
    fn members_merge_into_one_document() {
        let merged = merge([
            parse(RELEASES.as_bytes()).unwrap(),
            parse(SNAPSHOTS.as_bytes()).unwrap(),
        ])
        .unwrap();
        let versioning = merged.versioning.unwrap();
        assert_eq!(versioning.versions, ["1.0-SNAPSHOT", "1.0", "9.9", "10.0"]);
        assert_eq!(versioning.latest.as_deref(), Some("10.0"));
        assert_eq!(versioning.release.as_deref(), Some("10.0"));
        assert_eq!(versioning.last_updated.as_deref(), Some("20261012101010"));
        assert!(merge(Vec::new()).is_none());
    }

    #[test]
    fn snapshot_metadata_keeps_the_newest_builds() {
        let one = Metadata {
            version: Some("1.0-SNAPSHOT".to_owned()),
            versioning: Some(Versioning {
                snapshot: Some(Snapshot {
                    timestamp: Some("20261012.101010".to_owned()),
                    build_number: Some(3),
                    local_copy: false,
                }),
                last_updated: Some("20261012101010".to_owned()),
                snapshot_versions: vec![
                    SnapshotVersion {
                        classifier: None,
                        extension: "jar".to_owned(),
                        value: "1.0-20261012.101010-3".to_owned(),
                        updated: Some("20261012101010".to_owned()),
                    },
                    SnapshotVersion {
                        classifier: Some("sources".to_owned()),
                        extension: "jar".to_owned(),
                        value: "1.0-20261012.101010-3".to_owned(),
                        updated: Some("20261012101010".to_owned()),
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut two = one.clone();
        let versioning = two.versioning.as_mut().unwrap();
        versioning.snapshot = Some(Snapshot {
            timestamp: Some("20261013.080000".to_owned()),
            build_number: Some(4),
            local_copy: false,
        });
        versioning.last_updated = Some("20261013080000".to_owned());
        versioning.snapshot_versions.truncate(1);
        versioning.snapshot_versions[0].value = "1.0-20261013.080000-4".to_owned();
        versioning.snapshot_versions[0].updated = Some("20261013080000".to_owned());
        let merged = merge([one.clone(), two]).unwrap().versioning.unwrap();
        assert_eq!(merged.snapshot.unwrap().build_number, Some(4));
        assert_eq!(merged.snapshot_versions.len(), 2);
        assert_eq!(merged.snapshot_versions[0].value, "1.0-20261013.080000-4");
        assert_eq!(merged.snapshot_versions[1].value, "1.0-20261012.101010-3");
        let written = write(&one);
        assert!(written.contains("<snapshot>\n      <timestamp>20261012.101010</timestamp>\n      <buildNumber>3</buildNumber>\n    </snapshot>"));
        assert_eq!(parse(written.as_bytes()).unwrap(), one);
    }

    #[test]
    fn plugins_are_kept_once_per_prefix() {
        let plugins = |prefix: &str, artifact: &str| Metadata {
            plugins: vec![Plugin {
                name: None,
                prefix: prefix.to_owned(),
                artifact_id: artifact.to_owned(),
            }],
            ..Default::default()
        };
        let merged = merge([
            plugins("hello", "hello-maven-plugin"),
            plugins("hello", "other-plugin"),
            plugins("bye", "bye-maven-plugin"),
        ])
        .unwrap();
        assert_eq!(merged.plugins.len(), 2);
        assert_eq!(merged.plugins[0].artifact_id, "hello-maven-plugin");
        let written = write(&merged);
        assert!(!written.contains("<versioning>"));
        assert_eq!(parse(written.as_bytes()).unwrap(), merged);
    }

    #[test]
    fn timestamps_use_the_maven_form() {
        let time = parse_timestamp("20261012101010").unwrap();
        assert_eq!(timestamp(time), "20261012101010");
        assert!(parse_timestamp("2026-10-12").is_none());
    }
}
