use std::cmp::Ordering;

pub const MAX_NAME: usize = 255;
const MAX_PART: usize = 18;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Wanted {
    Latest,
    Exact(String),
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    matches!(parts.len(), 2 | 3)
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= MAX_PART && part.bytes().all(|b| b.is_ascii_digit())
        })
}

pub fn path_version(segment: &str) -> Option<String> {
    let dotted = segment.replace('_', ".");
    valid_version(&dotted).then_some(dotted)
}

pub fn wanted(segment: &str) -> Option<Wanted> {
    match segment {
        "latest" => Some(Wanted::Latest),
        other => path_version(other).map(Wanted::Exact),
    }
}

fn numbers(version: &str) -> [u64; 3] {
    let mut numbers = [0; 3];
    for (slot, part) in numbers.iter_mut().zip(version.split('.')) {
        *slot = part.parse().unwrap_or(0);
    }
    numbers
}

pub fn compare(a: &str, b: &str) -> Ordering {
    numbers(a).cmp(&numbers(b)).then_with(|| a.cmp(b))
}

pub fn newest_first(versions: &mut [String]) {
    versions.sort_by(|a, b| compare(b, a));
}

pub fn highest<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    versions.into_iter().max_by(|a, b| compare(a, b))
}

pub fn version_of_url(address: &str) -> Option<String> {
    let path = address.split(['?', '#']).next().unwrap_or_default();
    let (_, tail) = path.rsplit_once("/versions/")?;
    path_version(tail.trim_end_matches('/'))
}

pub fn tarball_name(name: &str, version: &str) -> String {
    format!("{name}-{version}.tar.gz")
}

pub fn tarball_path(name: &str, version: &str) -> String {
    format!(
        "/cookbooks/{name}/{version}/{}",
        tarball_name(name, version)
    )
}

pub fn folder_path(name: &str) -> String {
    format!("/cookbooks/{name}/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_versions_follow_the_supermarket() {
        for name in [
            "apt",
            "build-essential",
            "chef_client_updater",
            "SysinternalsBginfo",
            "301",
        ] {
            assert!(valid_name(name), "{name}");
        }
        for name in ["", "a b", "a/b", "a.b", "ü"] {
            assert!(!valid_name(name), "{name}");
        }
        assert!(valid_version("1.2.3"));
        assert!(valid_version("0.10"));
        for version in ["1", "1.2.3.4", "1.2.x", "1.2.3-rc.1", "1..2", ""] {
            assert!(!valid_version(version), "{version}");
        }
        assert_eq!(path_version("1_2_3").as_deref(), Some("1.2.3"));
        assert_eq!(path_version("7.4.0").as_deref(), Some("7.4.0"));
        assert_eq!(wanted("latest"), Some(Wanted::Latest));
        assert_eq!(wanted("1_0"), Some(Wanted::Exact("1.0".to_owned())));
        assert_eq!(wanted("newest"), None);
    }

    #[test]
    fn versions_compare_by_number() {
        let mut versions: Vec<String> = ["1.10.0", "1.2.0", "1.9", "2.0.0", "1.2.10"]
            .iter()
            .map(|version| (*version).to_owned())
            .collect();
        newest_first(&mut versions);
        assert_eq!(versions, ["2.0.0", "1.10.0", "1.9", "1.2.10", "1.2.0"]);
        assert_eq!(highest(["0.9.0", "0.10.0"]), Some("0.10.0"));
        assert_eq!(highest([]), None);
    }

    #[test]
    fn version_addresses_give_their_version() {
        assert_eq!(
            version_of_url("https://supermarket.example/api/v1/cookbooks/apt/versions/7_4_0")
                .as_deref(),
            Some("7.4.0")
        );
        assert_eq!(
            version_of_url("https://supermarket.example/api/v1/cookbooks/apt/versions/8.0.0/")
                .as_deref(),
            Some("8.0.0")
        );
        assert_eq!(
            version_of_url("https://supermarket.example/api/v1/cookbooks/apt"),
            None
        );
        assert_eq!(
            tarball_path("apt", "1.0.0"),
            "/cookbooks/apt/1.0.0/apt-1.0.0.tar.gz"
        );
    }
}
