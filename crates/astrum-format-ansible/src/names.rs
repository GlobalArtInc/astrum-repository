use std::cmp::Ordering;

pub const MAX_NAME: usize = 64;

/// A namespace or collection name as Galaxy publishes it: lower-case letters, digits and
/// underscores, starting with a letter, without two underscores in a row.
pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    name.len() <= MAX_NAME
        && chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.contains("__")
}

/// A name as it may appear in a path: what `valid_name` accepts, plus the names other Galaxy
/// servers already hold (a leading digit or underscore, two underscores).
pub fn path_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

pub fn is_prerelease(version: &str) -> bool {
    semver::Version::parse(version).is_ok_and(|parsed| !parsed.pre.is_empty())
}

/// Newest first by SemVer precedence; text that is not SemVer sorts last.
pub fn newest_first(a: &str, b: &str) -> Ordering {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(a), Ok(b)) => b.cmp_precedence(&a).then_with(|| b.cmp(&a)),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

/// The version Galaxy calls highest: the newest release, or the newest pre-release when there
/// is no release.
pub fn highest<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let mut all: Vec<&str> = versions.into_iter().collect();
    all.sort_by(|a, b| newest_first(a, b));
    all.iter()
        .find(|version| valid_version(version) && !is_prerelease(version))
        .or_else(|| all.first())
        .copied()
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Coll {
    pub namespace: String,
    pub name: String,
}

impl Coll {
    pub fn new(namespace: &str, name: &str) -> Option<Self> {
        (path_name(namespace) && path_name(name)).then(|| Self {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
        })
    }

    pub fn file_name(&self, version: &str) -> String {
        format!("{}-{}-{version}.tar.gz", self.namespace, self.name)
    }

    pub fn artifact_path(&self, version: &str) -> String {
        format!(
            "/{}/{}/{version}/{}",
            self.namespace,
            self.name,
            self.file_name(version)
        )
    }

    pub fn api_path(&self) -> String {
        format!("/api/v3/collections/{}/{}/", self.namespace, self.name)
    }

    pub fn cached_collection(&self) -> String {
        format!("{}collection.json", self.api_path())
    }

    pub fn cached_versions(&self) -> String {
        format!("{}versions.json", self.api_path())
    }

    pub fn cached_version(&self, version: &str) -> String {
        format!("{}versions/{version}.json", self.api_path())
    }
}

impl std::fmt::Display for Coll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.namespace, self.name)
    }
}

/// Splits `<namespace>-<name>-<version>.tar.gz`; names hold no `-`, so the version is the rest.
pub fn parse_file_name(file: &str) -> Option<(Coll, String)> {
    let stem = file.strip_suffix(".tar.gz")?;
    let (namespace, rest) = stem.split_once('-')?;
    let (name, version) = rest.split_once('-')?;
    let coll = Coll::new(namespace, name)?;
    valid_version(version).then(|| (coll, version.to_owned()))
}

/// Reads `<namespace>.<name>` as dependencies and the command line write it.
pub fn parse_fqcn(text: &str) -> Option<Coll> {
    let (namespace, name) = text.split_once('.')?;
    Coll::new(namespace, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_galaxy() {
        for good in ["community", "a", "my_ns2", "x1_y"] {
            assert!(valid_name(good), "{good}");
        }
        for bad in [
            "",
            "Community",
            "1abc",
            "_abc",
            "a__b",
            "a-b",
            "a.b",
            &"a".repeat(65),
        ] {
            assert!(!valid_name(bad), "{bad}");
        }
        assert!(path_name("1password") && path_name("_private") && !path_name("A"));
    }

    #[test]
    fn versions_sort_by_semver_precedence() {
        let mut versions = vec!["1.0.0", "1.10.0", "1.2.0", "2.0.0-rc.1", "1.0.0-beta.2"];
        versions.sort_by(|a, b| newest_first(a, b));
        assert_eq!(
            versions,
            ["2.0.0-rc.1", "1.10.0", "1.2.0", "1.0.0", "1.0.0-beta.2"]
        );
        assert_eq!(highest(["1.0.0", "2.0.0-rc.1", "1.2.0"]), Some("1.2.0"));
        assert_eq!(highest(["2.0.0-rc.1", "2.0.0-beta.1"]), Some("2.0.0-rc.1"));
        assert_eq!(highest([]), None);
        assert!(is_prerelease("1.0.0-beta.1") && !is_prerelease("1.0.0+build.5"));
        assert!(!valid_version("1.0") && !valid_version("01.0.0"));
    }

    #[test]
    fn file_names_split_at_the_first_two_dashes() {
        let (coll, version) = parse_file_name("my_ns-tools-1.2.0-rc.1.tar.gz").unwrap();
        assert_eq!(coll.to_string(), "my_ns.tools");
        assert_eq!(version, "1.2.0-rc.1");
        assert_eq!(
            coll.artifact_path(&version),
            "/my_ns/tools/1.2.0-rc.1/my_ns-tools-1.2.0-rc.1.tar.gz"
        );
        for bad in [
            "ns-name.tar.gz",
            "ns-name-1.0.tar.gz",
            "ns-name-1.0.0.zip",
            "NS-name-1.0.0.tar.gz",
        ] {
            assert!(parse_file_name(bad).is_none(), "{bad}");
        }
        assert_eq!(parse_fqcn("community.general").unwrap().name, "general");
        assert!(parse_fqcn("general").is_none());
    }
}
