use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::proto::{
    Dependency, NamedPackage, Names, Package, Release, RetirementReason, RetirementStatus,
    Timestamp, VersionedPackage, Versions,
};
use crate::tarball::Requirement;

pub const PUBLIC_REPOSITORY: &str = "hexpm";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retirement {
    pub reason: RetirementReason,
    pub message: Option<String>,
}

impl Retirement {
    pub fn attributes(&self) -> Value {
        match &self.message {
            Some(message) => json!({"reason": self.reason.word(), "message": message}),
            None => json!({"reason": self.reason.word()}),
        }
    }

    pub fn from_attributes(value: &Value) -> Option<Retirement> {
        Some(Retirement {
            reason: RetirementReason::from_word(value.get("reason")?.as_str()?)?,
            message: value
                .get("message")
                .and_then(Value::as_str)
                .filter(|message| !message.is_empty())
                .map(str::to_owned),
        })
    }

    fn status(&self) -> RetirementStatus {
        RetirementStatus {
            reason: self.reason as i32,
            message: self.message.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub version: String,
    pub inner_checksum: Vec<u8>,
    pub outer_checksum: Vec<u8>,
    pub requirements: Vec<Requirement>,
    pub retired: Option<Retirement>,
    pub updated_at: i64,
}

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    match (semver::Version::parse(left), semver::Version::parse(right)) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        (Ok(_), Err(_)) => Ordering::Greater,
        (Err(_), Ok(_)) => Ordering::Less,
        (Err(_), Err(_)) => left.cmp(right),
    }
}

fn dependency(repository: &str, requirement: &Requirement) -> Dependency {
    let from = requirement
        .repository
        .as_deref()
        .unwrap_or(PUBLIC_REPOSITORY);
    Dependency {
        package: requirement.name.clone(),
        requirement: requirement.requirement.clone(),
        optional: requirement.optional.then_some(true),
        app: requirement
            .app
            .clone()
            .filter(|app| *app != requirement.name),
        repository: (from != repository).then(|| from.to_owned()),
    }
}

fn release(repository: &str, record: &Record) -> Release {
    Release {
        version: record.version.clone(),
        inner_checksum: record.inner_checksum.clone(),
        dependencies: record
            .requirements
            .iter()
            .map(|requirement| dependency(repository, requirement))
            .collect(),
        retired: record.retired.as_ref().map(Retirement::status),
        outer_checksum: Some(record.outer_checksum.clone()),
    }
}

fn by_name(records: &[Record]) -> BTreeMap<&str, Vec<&Record>> {
    let mut grouped: BTreeMap<&str, Vec<&Record>> = BTreeMap::new();
    for record in records {
        grouped.entry(&record.name).or_default().push(record);
    }
    for releases in grouped.values_mut() {
        releases.sort_by(|left, right| compare_versions(&left.version, &right.version));
        releases.dedup_by(|left, right| left.version == right.version);
    }
    grouped
}

pub fn package(repository: &str, name: &str, records: &[Record]) -> Package {
    let releases = by_name(records)
        .remove(name)
        .unwrap_or_default()
        .into_iter()
        .map(|record| release(repository, record))
        .collect();
    Package {
        releases,
        name: name.to_owned(),
        repository: repository.to_owned(),
    }
}

pub fn names(repository: &str, records: &[Record]) -> Names {
    Names {
        packages: by_name(records)
            .into_iter()
            .map(|(name, releases)| NamedPackage {
                name: name.to_owned(),
                updated_at: releases
                    .iter()
                    .map(|record| record.updated_at)
                    .max()
                    .map(|seconds| Timestamp { seconds, nanos: 0 }),
            })
            .collect(),
        repository: repository.to_owned(),
    }
}

pub fn versions(repository: &str, records: &[Record]) -> Versions {
    Versions {
        packages: by_name(records)
            .into_iter()
            .map(|(name, releases)| VersionedPackage {
                name: name.to_owned(),
                versions: releases
                    .iter()
                    .map(|record| record.version.clone())
                    .collect(),
                retired: releases
                    .iter()
                    .enumerate()
                    .filter(|(_, record)| record.retired.is_some())
                    .map(|(index, _)| index as i32)
                    .collect(),
            })
            .collect(),
        repository: repository.to_owned(),
    }
}

pub fn merge_names(repository: &str, members: Vec<Names>) -> Names {
    let mut merged: BTreeMap<String, Option<Timestamp>> = BTreeMap::new();
    for member in members {
        for package in member.packages {
            let slot = merged.entry(package.name).or_default();
            *slot = match (*slot, package.updated_at) {
                (Some(known), Some(other)) => Some(known.max(other)),
                (known, other) => known.or(other),
            };
        }
    }
    Names {
        packages: merged
            .into_iter()
            .map(|(name, updated_at)| NamedPackage { name, updated_at })
            .collect(),
        repository: repository.to_owned(),
    }
}

pub fn merge_versions(repository: &str, members: Vec<Versions>) -> Versions {
    let mut merged: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
    for member in members {
        for package in member.packages {
            let known = merged.entry(package.name).or_default();
            for (index, version) in package.versions.into_iter().enumerate() {
                if known.iter().any(|(seen, _)| *seen == version) {
                    continue;
                }
                let retired = package.retired.contains(&(index as i32));
                known.push((version, retired));
            }
        }
    }
    Versions {
        packages: merged
            .into_iter()
            .map(|(name, mut versions)| {
                versions.sort_by(|left, right| compare_versions(&left.0, &right.0));
                VersionedPackage {
                    name,
                    retired: versions
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, retired))| *retired)
                        .map(|(index, _)| index as i32)
                        .collect(),
                    versions: versions.into_iter().map(|(version, _)| version).collect(),
                }
            })
            .collect(),
        repository: repository.to_owned(),
    }
}

pub fn merge_package(repository: &str, name: &str, members: Vec<Package>) -> Package {
    let mut releases: Vec<Release> = Vec::new();
    for member in members {
        for release in member.releases {
            if !releases
                .iter()
                .any(|known| known.version == release.version)
            {
                releases.push(release);
            }
        }
    }
    releases.sort_by(|left, right| compare_versions(&left.version, &right.version));
    Package {
        releases,
        name: name.to_owned(),
        repository: repository.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(name: &str, version: &str, updated_at: i64) -> Record {
        Record {
            name: name.to_owned(),
            version: version.to_owned(),
            inner_checksum: vec![1; 32],
            outer_checksum: vec![2; 32],
            requirements: Vec::new(),
            retired: None,
            updated_at,
        }
    }

    #[test]
    fn hosted_records_become_the_three_resources() {
        let mut beta = record("hello", "1.0.0-beta.1", 5);
        beta.retired = Some(Retirement {
            reason: RetirementReason::RetiredInvalid,
            message: Some("broken".to_owned()),
        });
        let mut first = record("hello", "1.0.0", 7);
        first.requirements = vec![
            Requirement {
                name: "jason".to_owned(),
                requirement: "~> 1.4".to_owned(),
                optional: false,
                app: None,
                repository: Some("hexpm".to_owned()),
            },
            Requirement {
                name: "tools".to_owned(),
                requirement: "~> 0.1".to_owned(),
                optional: true,
                app: Some("my_tools".to_owned()),
                repository: Some("hex-local".to_owned()),
            },
            Requirement {
                name: "plain".to_owned(),
                requirement: ">= 0.0.0".to_owned(),
                optional: false,
                app: Some("plain".to_owned()),
                repository: None,
            },
        ];
        let records = vec![
            record("zeta", "0.1.0", 3),
            first,
            record("hello", "10.0.0", 6),
            record("hello", "2.0.0", 9),
            beta,
        ];
        let package = package("hex-local", "hello", &records);
        assert_eq!(package.repository, "hex-local");
        let ordered: Vec<&str> = package
            .releases
            .iter()
            .map(|release| release.version.as_str())
            .collect();
        assert_eq!(ordered, ["1.0.0-beta.1", "1.0.0", "2.0.0", "10.0.0"]);
        assert_eq!(
            package.releases[0].retired,
            Some(RetirementStatus {
                reason: 1,
                message: Some("broken".to_owned())
            })
        );
        let dependencies = &package.releases[1].dependencies;
        assert_eq!(dependencies[0].repository.as_deref(), Some("hexpm"));
        assert_eq!(dependencies[0].optional, None);
        assert_eq!(dependencies[1].repository, None);
        assert_eq!(dependencies[1].optional, Some(true));
        assert_eq!(dependencies[1].app.as_deref(), Some("my_tools"));
        assert_eq!(dependencies[2].repository.as_deref(), Some("hexpm"));
        assert_eq!(dependencies[2].app, None);
        assert_eq!(package.releases[1].outer_checksum, Some(vec![2; 32]));

        let names = names("hex-local", &records);
        assert_eq!(
            names
                .packages
                .iter()
                .map(|package| (package.name.as_str(), package.updated_at.unwrap().seconds))
                .collect::<Vec<_>>(),
            [("hello", 9), ("zeta", 3)]
        );
        let versions = versions("hex-local", &records);
        assert_eq!(versions.packages[0].versions[3], "10.0.0");
        assert_eq!(versions.packages[0].retired, [0]);
        assert!(package_is_empty(&records));
    }

    fn package_is_empty(records: &[Record]) -> bool {
        package("r", "missing", records).releases.is_empty()
    }

    #[test]
    fn groups_merge_with_the_first_member_winning() {
        let one = Versions {
            packages: vec![VersionedPackage {
                name: "a".to_owned(),
                versions: vec!["1.0.0".to_owned(), "1.1.0".to_owned()],
                retired: vec![1],
            }],
            repository: "one".to_owned(),
        };
        let two = Versions {
            packages: vec![
                VersionedPackage {
                    name: "a".to_owned(),
                    versions: vec!["0.9.0".to_owned(), "1.1.0".to_owned()],
                    retired: vec![],
                },
                VersionedPackage {
                    name: "b".to_owned(),
                    versions: vec!["2.0.0".to_owned()],
                    retired: vec![0],
                },
            ],
            repository: "two".to_owned(),
        };
        let merged = merge_versions("all", vec![one, two]);
        assert_eq!(merged.repository, "all");
        assert_eq!(merged.packages[0].versions, ["0.9.0", "1.0.0", "1.1.0"]);
        assert_eq!(merged.packages[0].retired, [2]);
        assert_eq!(merged.packages[1].retired, [0]);

        let names = merge_names(
            "all",
            vec![
                Names {
                    packages: vec![NamedPackage {
                        name: "b".to_owned(),
                        updated_at: Some(Timestamp {
                            seconds: 1,
                            nanos: 0,
                        }),
                    }],
                    repository: "one".to_owned(),
                },
                Names {
                    packages: vec![
                        NamedPackage {
                            name: "a".to_owned(),
                            updated_at: None,
                        },
                        NamedPackage {
                            name: "b".to_owned(),
                            updated_at: Some(Timestamp {
                                seconds: 4,
                                nanos: 0,
                            }),
                        },
                    ],
                    repository: "two".to_owned(),
                },
            ],
        );
        assert_eq!(names.packages[0].name, "a");
        assert_eq!(names.packages[1].updated_at.unwrap().seconds, 4);

        let first = package("one", "a", &[record("a", "1.0.0", 1)]);
        let mut other = record("a", "1.0.0", 1);
        other.inner_checksum = vec![9; 32];
        let second = package("two", "a", &[other, record("a", "0.1.0", 1)]);
        let merged = merge_package("all", "a", vec![first, second]);
        assert_eq!(merged.repository, "all");
        assert_eq!(merged.releases.len(), 2);
        assert_eq!(merged.releases[0].version, "0.1.0");
        assert_eq!(merged.releases[1].inner_checksum, vec![1; 32]);
        assert_eq!(
            Retirement::from_attributes(
                &Retirement {
                    reason: RetirementReason::RetiredRenamed,
                    message: Some("see b".to_owned())
                }
                .attributes()
            )
            .unwrap()
            .message
            .as_deref(),
            Some("see b")
        );
    }
}
