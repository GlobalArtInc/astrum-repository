use std::cmp::Ordering;

pub const MAX_NAME: usize = 64;
pub fn valid_owner(owner: &str) -> bool {
    !owner.is_empty() && owner.len() <= MAX_NAME && owner.chars().all(|c| c.is_ascii_alphanumeric())
}
pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    name.len() <= MAX_NAME
        && chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

pub fn is_prerelease(version: &str) -> bool {
    semver::Version::parse(version).is_ok_and(|parsed| !parsed.pre.is_empty())
}
pub fn newest_first(a: &str, b: &str) -> Ordering {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(a), Ok(b)) => b.cmp_precedence(&a).then_with(|| b.cmp(&a)),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}
pub fn highest<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let mut all: Vec<&str> = versions.into_iter().collect();
    all.sort_by(|a, b| newest_first(a, b));
    all.iter()
        .find(|version| valid_version(version) && !is_prerelease(version))
        .or_else(|| all.first())
        .copied()
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Module {
    pub owner: String,
    pub name: String,
}

impl Module {
    pub fn new(owner: &str, name: &str) -> Option<Self> {
        (valid_owner(owner) && valid_name(name)).then(|| Self {
            owner: owner.to_ascii_lowercase(),
            name: name.to_owned(),
        })
    }
    pub fn parse(slug: &str) -> Option<Self> {
        let (owner, name) = slug.split_once(['-', '/'])?;
        Self::new(owner, name)
    }

    pub fn slug(&self) -> String {
        format!("{}-{}", self.owner, self.name)
    }

    pub fn release_slug(&self, version: &str) -> String {
        format!("{}-{version}", self.slug())
    }

    pub fn file_name(&self, version: &str) -> String {
        format!("{}.tar.gz", self.release_slug(version))
    }

    pub fn artifact_path(&self, version: &str) -> String {
        format!(
            "/{}/{}/{version}/{}",
            self.owner,
            self.name,
            self.file_name(version)
        )
    }

    pub fn uri(&self) -> String {
        format!("/v3/modules/{}", self.slug())
    }

    pub fn release_uri(&self, version: &str) -> String {
        format!("/v3/releases/{}", self.release_slug(version))
    }

    pub fn file_uri(&self, version: &str) -> String {
        format!("/v3/files/{}", self.file_name(version))
    }

    pub fn cached_module(&self) -> String {
        format!("{}.json", self.uri())
    }

    pub fn cached_releases(&self) -> String {
        format!("{}/releases.json", self.uri())
    }

    pub fn cached_release(&self, version: &str) -> String {
        format!("{}.json", self.release_uri(version))
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.owner, self.name)
    }
}
pub fn parse_release(slug: &str) -> Option<(Module, String)> {
    let (owner, rest) = slug.split_once('-')?;
    let (name, version) = rest.split_once('-')?;
    let module = Module::new(owner, name)?;
    valid_version(version).then(|| (module, version.to_owned()))
}

pub fn parse_file_name(file: &str) -> Option<(Module, String)> {
    parse_release(file.strip_suffix(".tar.gz")?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_forge() {
        for good in ["stdlib", "a", "my_mod2", "x1_y"] {
            assert!(valid_name(good), "{good}");
        }
        for bad in ["", "Stdlib", "1abc", "_abc", "a-b", "a.b", &"a".repeat(65)] {
            assert!(!valid_name(bad), "{bad}");
        }
        assert!(valid_owner("puppetlabs") && valid_owner("MiamiOH") && valid_owner("42"));
        assert!(!valid_owner("") && !valid_owner("a_b") && !valid_owner("a-b"));
    }

    #[test]
    fn slugs_take_both_separators_and_keep_owners_in_lower_case() {
        let module = Module::parse("PuppetLabs/stdlib").unwrap();
        assert_eq!(module.slug(), "puppetlabs-stdlib");
        assert_eq!(Module::parse("puppetlabs-stdlib").unwrap(), module);
        assert_eq!(
            module.artifact_path("9.0.0"),
            "/puppetlabs/stdlib/9.0.0/puppetlabs-stdlib-9.0.0.tar.gz"
        );
        assert_eq!(
            module.file_uri("9.0.0"),
            "/v3/files/puppetlabs-stdlib-9.0.0.tar.gz"
        );
        assert_eq!(
            module.release_uri("9.0.0"),
            "/v3/releases/puppetlabs-stdlib-9.0.0"
        );
        for bad in [
            "stdlib",
            "puppetlabs-",
            "puppet-labs-stdlib",
            "a_b-c",
            "a-B",
        ] {
            assert!(Module::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn release_slugs_and_file_names_split_at_the_first_two_dashes() {
        let (module, version) = parse_release("acme-tools-1.2.0-rc.1").unwrap();
        assert_eq!(module.to_string(), "acme-tools");
        assert_eq!(version, "1.2.0-rc.1");
        let (module, version) = parse_file_name("acme-tools-1.2.0+build.5.tar.gz").unwrap();
        assert_eq!(module.name, "tools");
        assert_eq!(version, "1.2.0+build.5");
        for bad in [
            "acme-tools",
            "acme-tools-1.0",
            "acme-Tools-1.0.0",
            "acme_x-tools-1.0.0",
        ] {
            assert!(parse_release(bad).is_none(), "{bad}");
        }
        assert!(parse_file_name("acme-tools-1.0.0.zip").is_none());
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
    }
}
