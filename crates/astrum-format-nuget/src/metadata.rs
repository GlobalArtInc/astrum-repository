use serde::{Deserialize, Serialize};

use crate::version::{self, Version};
use crate::xml::{self, Element};

pub const MAX_ID: usize = 100;
pub const SYMBOLS_TYPE: &str = "SymbolsPackage";
pub const DEFAULT_TYPE: &str = "Dependency";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NuspecError {
    #[error("the .nuspec file is not valid XML: {0}")]
    Xml(String),
    #[error("the .nuspec file has no metadata element")]
    NoMetadata,
    #[error("the package id is missing")]
    NoId,
    #[error("the package id {0} is not valid")]
    BadId(String),
    #[error("the package version is missing")]
    NoVersion,
    #[error("the package version {0} is not valid")]
    BadVersion(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub id: String,
    #[serde(default)]
    pub range: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_framework: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub id: String,
    pub version: String,
    pub normalized_version: String,
    #[serde(default)]
    pub original_version: String,
    #[serde(default)]
    pub is_prerelease: bool,
    #[serde(default, rename = "semVer2")]
    pub semver2: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owners: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_expression: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(default)]
    pub require_license_acceptance: bool,
    #[serde(default)]
    pub development_dependency: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_client_version: Option<String>,
    #[serde(default)]
    pub package_types: Vec<String>,
    #[serde(default)]
    pub dependency_groups: Vec<DependencyGroup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
}

pub fn valid_id(id: &str) -> bool {
    if id.is_empty() || id.chars().count() > MAX_ID {
        return false;
    }
    let mut previous_separator = true;
    for c in id.chars() {
        if c == '.' || c == '-' {
            if previous_separator {
                return false;
            }
            previous_separator = true;
        } else if c.is_alphanumeric() || c == '_' {
            previous_separator = false;
        } else {
            return false;
        }
    }
    !previous_separator
}

fn split_list(text: Option<String>, separators: &[char]) -> Vec<String> {
    text.map(|text| {
        text.split(|c: char| separators.contains(&c) || c.is_whitespace())
            .filter(|part| !part.is_empty())
            .map(str::to_owned)
            .collect()
    })
    .unwrap_or_default()
}

fn dependency(element: &Element) -> Option<Dependency> {
    let id = element.attribute("id")?.to_owned();
    Some(Dependency {
        id,
        range: element.attribute("version").unwrap_or_default().to_owned(),
    })
}

fn dependency_groups(metadata: &Element) -> Vec<DependencyGroup> {
    let Some(dependencies) = metadata.child("dependencies") else {
        return Vec::new();
    };
    let mut groups: Vec<DependencyGroup> = dependencies
        .children("group")
        .map(|group| DependencyGroup {
            target_framework: group.attribute("targetFramework").map(str::to_owned),
            dependencies: group
                .children("dependency")
                .filter_map(dependency)
                .collect(),
        })
        .collect();
    let loose: Vec<Dependency> = dependencies
        .children("dependency")
        .filter_map(dependency)
        .collect();
    if !loose.is_empty() {
        groups.insert(
            0,
            DependencyGroup {
                target_framework: None,
                dependencies: loose,
            },
        );
    }
    groups
}

pub fn parse_nuspec(bytes: &[u8]) -> Result<Metadata, NuspecError> {
    let root = xml::parse(bytes).map_err(|error| NuspecError::Xml(error.to_string()))?;
    let metadata = root.child("metadata").ok_or(NuspecError::NoMetadata)?;
    let id = metadata.text_of("id").ok_or(NuspecError::NoId)?;
    if !valid_id(&id) {
        return Err(NuspecError::BadId(id));
    }
    let original = metadata.text_of("version").ok_or(NuspecError::NoVersion)?;
    let parsed =
        Version::parse(&original).ok_or_else(|| NuspecError::BadVersion(original.clone()))?;
    let license = metadata.child("license");
    let license_expression = license
        .filter(|license| license.attribute("type") == Some("expression"))
        .and_then(Element::value);
    let mut package_types: Vec<String> = metadata
        .child("packageTypes")
        .map(|types| {
            types
                .children("packageType")
                .filter_map(|kind| kind.attribute("name").map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    if package_types.is_empty() {
        package_types.push(DEFAULT_TYPE.to_owned());
    }
    let repository = metadata.child("repository").map(|repository| Repository {
        kind: repository.attribute("type").map(str::to_owned),
        url: repository.attribute("url").map(str::to_owned),
        commit: repository.attribute("commit").map(str::to_owned),
        branch: repository.attribute("branch").map(str::to_owned),
    });
    let flag = |name: &str| {
        metadata
            .text_of(name)
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    };
    Ok(Metadata {
        id,
        version: parsed.full(),
        normalized_version: parsed.normalized(),
        original_version: original,
        is_prerelease: parsed.is_prerelease(),
        semver2: parsed.is_semver2(),
        title: metadata.text_of("title"),
        authors: metadata.text_of("authors"),
        owners: metadata.text_of("owners"),
        description: metadata.text_of("description"),
        summary: metadata.text_of("summary"),
        release_notes: metadata.text_of("releaseNotes"),
        copyright: metadata.text_of("copyright"),
        language: metadata.text_of("language"),
        tags: split_list(metadata.text_of("tags"), &[',', ';']),
        project_url: metadata.text_of("projectUrl"),
        license_url: metadata.text_of("licenseUrl"),
        license_expression,
        icon_url: metadata.text_of("iconUrl"),
        icon: metadata.text_of("icon"),
        readme: metadata.text_of("readme"),
        require_license_acceptance: flag("requireLicenseAcceptance"),
        development_dependency: flag("developmentDependency"),
        min_client_version: metadata.attribute("minClientVersion").map(str::to_owned),
        package_types,
        dependency_groups: dependency_groups(metadata),
        repository: repository.filter(|repository| repository != &Repository::default()),
    })
}

impl Metadata {
    pub fn parsed_version(&self) -> Option<Version> {
        Version::parse(&self.version).or_else(|| Version::parse(&self.normalized_version))
    }

    pub fn lower_id(&self) -> String {
        self.id.to_lowercase()
    }

    pub fn lower_version(&self) -> String {
        self.normalized_version.to_lowercase()
    }

    pub fn is_symbols(&self) -> bool {
        self.package_types
            .iter()
            .any(|kind| kind.eq_ignore_ascii_case(SYMBOLS_TYPE))
    }

    pub fn author_list(&self) -> Vec<String> {
        self.authors
            .as_deref()
            .map(|authors| {
                authors
                    .split(',')
                    .map(str::trim)
                    .filter(|author| !author.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn v2_dependencies(&self) -> String {
        let mut parts = Vec::new();
        for group in &self.dependency_groups {
            let framework = group.target_framework.clone().unwrap_or_default();
            if group.dependencies.is_empty() {
                parts.push(format!("::{framework}"));
            }
            for dependency in &group.dependencies {
                parts.push(format!(
                    "{}:{}:{framework}",
                    dependency.id, dependency.range
                ));
            }
        }
        parts.join("|")
    }

    pub fn ranges_normalized(&self) -> Vec<DependencyGroup> {
        self.dependency_groups
            .iter()
            .map(|group| DependencyGroup {
                target_framework: group.target_framework.clone(),
                dependencies: group
                    .dependencies
                    .iter()
                    .map(|dependency| Dependency {
                        id: dependency.id.clone(),
                        range: version::range(&dependency.range),
                    })
                    .collect(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const NUSPEC: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">
  <metadata minClientVersion="3.3.0">
    <id>Astrum.Probe</id>
    <version>1.02.0-Beta.1</version>
    <title>Probe</title>
    <authors>Astrum Team, Someone Else</authors>
    <requireLicenseAcceptance>true</requireLicenseAcceptance>
    <license type="expression">MIT</license>
    <licenseUrl>https://licenses.nuget.org/MIT</licenseUrl>
    <description>A probe &amp; more.</description>
    <tags>astrum probe,test</tags>
    <packageTypes><packageType name="Dependency" /></packageTypes>
    <repository type="git" url="https://example.invalid/probe.git" commit="abc" />
    <dependencies>
      <group targetFramework="net8.0">
        <dependency id="Newtonsoft.Json" version="13.0.1" exclude="Build" />
      </group>
      <group targetFramework="netstandard2.0" />
    </dependencies>
  </metadata>
</package>"#;

    #[test]
    fn a_nuspec_gives_its_metadata() {
        let metadata = parse_nuspec(NUSPEC.as_bytes()).unwrap();
        assert_eq!(metadata.id, "Astrum.Probe");
        assert_eq!(metadata.normalized_version, "1.2.0-Beta.1");
        assert_eq!(metadata.original_version, "1.02.0-Beta.1");
        assert_eq!(metadata.lower_version(), "1.2.0-beta.1");
        assert!(metadata.is_prerelease && metadata.semver2);
        assert_eq!(metadata.tags, vec!["astrum", "probe", "test"]);
        assert_eq!(metadata.author_list(), vec!["Astrum Team", "Someone Else"]);
        assert_eq!(metadata.license_expression.as_deref(), Some("MIT"));
        assert_eq!(metadata.description.as_deref(), Some("A probe & more."));
        assert_eq!(metadata.min_client_version.as_deref(), Some("3.3.0"));
        assert!(metadata.require_license_acceptance);
        assert_eq!(
            metadata.v2_dependencies(),
            "Newtonsoft.Json:13.0.1:net8.0|::netstandard2.0"
        );
        assert_eq!(
            metadata.ranges_normalized()[0].dependencies[0].range,
            "[13.0.1, )"
        );
        assert_eq!(
            metadata.repository.as_ref().unwrap().commit.as_deref(),
            Some("abc")
        );
        assert!(!metadata.is_symbols());
        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["semVer2"], true);
        let back: Metadata = serde_json::from_value(json).unwrap();
        assert_eq!(back, metadata);
    }

    #[test]
    fn loose_dependencies_and_symbol_packages_are_understood() {
        let nuspec = r#"<package><metadata><id>A</id><version>1.0</version>
            <packageTypes><packageType name="SymbolsPackage"/></packageTypes>
            <dependencies><dependency id="B" version="[1.0,2.0)"/></dependencies></metadata></package>"#;
        let metadata = parse_nuspec(nuspec.as_bytes()).unwrap();
        assert!(metadata.is_symbols());
        assert_eq!(metadata.v2_dependencies(), "B:[1.0,2.0):");
        assert_eq!(metadata.package_types, vec!["SymbolsPackage"]);
        let plain = parse_nuspec(
            b"<package><metadata><id>A</id><version>1.0</version></metadata></package>",
        )
        .unwrap();
        assert_eq!(plain.package_types, vec![DEFAULT_TYPE]);
        assert!(plain.dependency_groups.is_empty());
    }

    #[test]
    fn broken_nuspecs_say_what_is_wrong() {
        let cases: [(&str, NuspecError); 5] = [
            ("<package/>", NuspecError::NoMetadata),
            ("<package><metadata/></package>", NuspecError::NoId),
            (
                "<package><metadata><id>a b</id></metadata></package>",
                NuspecError::BadId("a b".to_owned()),
            ),
            (
                "<package><metadata><id>a</id></metadata></package>",
                NuspecError::NoVersion,
            ),
            (
                "<package><metadata><id>a</id><version>x</version></metadata></package>",
                NuspecError::BadVersion("x".to_owned()),
            ),
        ];
        for (nuspec, error) in cases {
            assert_eq!(parse_nuspec(nuspec.as_bytes()).unwrap_err(), error);
        }
        assert!(matches!(
            parse_nuspec(b"<package>"),
            Err(NuspecError::Xml(_))
        ));
    }

    #[test]
    fn ids_follow_the_gallery_rules() {
        for good in ["A", "Astrum.Probe", "my-lib_2", "Ünicode.Name"] {
            assert!(valid_id(good), "{good}");
        }
        for bad in ["", ".a", "a.", "a..b", "a b", "a/b", &"x".repeat(101)] {
            assert!(!valid_id(bad), "{bad}");
        }
    }
}
