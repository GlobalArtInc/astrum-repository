use std::cmp::Ordering;

pub const UNKNOWN: &str = "unknown";
pub const BOX_SUFFIX: &str = ".box";
pub const ARCHITECTURES: &[&str] = &[
    "amd64", "arm64", "i386", "arm", "ppc64le", "ppc64", "mips64le", "mips64", "mipsle", "mips",
    "s390x", UNKNOWN,
];
const MAX_NAME: usize = 128;
const MAX_VERSION: usize = 128;
const MAX_PROVIDER: usize = 64;

fn token(text: &str, max: usize, dots: bool) -> bool {
    !text.is_empty()
        && text.len() <= max
        && text.starts_with(|c: char| c.is_ascii_alphanumeric())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || (dots && c == '.'))
}

pub fn valid_name(name: &str) -> bool {
    token(name, MAX_NAME, true)
}

pub fn valid_provider(provider: &str) -> bool {
    token(provider, MAX_PROVIDER, false)
}

pub fn valid_architecture(architecture: &str) -> bool {
    token(architecture, MAX_PROVIDER, false)
}

pub fn known_architecture(architecture: &str) -> bool {
    ARCHITECTURES.contains(&architecture)
}

pub fn valid_version(version: &str) -> bool {
    if version.is_empty() || version.len() > MAX_VERSION {
        return false;
    }
    let (release, pre) = match version.split_once('-') {
        Some((release, pre)) => (release, Some(pre)),
        None => (version, None),
    };
    let mut parts = release.split('.');
    let leading = parts
        .next()
        .is_some_and(|first| !first.is_empty() && first.bytes().all(|b| b.is_ascii_digit()));
    leading
        && parts.all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric()))
        && pre.is_none_or(|pre| {
            pre.split('.').all(|part| {
                !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        })
}

pub fn is_prerelease(version: &str) -> bool {
    version.chars().any(|c| c.is_ascii_alphabetic())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    Number(String),
    Text(String),
}

fn flush(current: &mut String, digits: bool, found: &mut Vec<Segment>) {
    if current.is_empty() {
        return;
    }
    let taken = std::mem::take(current);
    found.push(match digits {
        true => Segment::Number(taken.trim_start_matches('0').to_owned()),
        false => Segment::Text(taken),
    });
}

fn segments(version: &str) -> Vec<Segment> {
    let spelled = version.replace('-', ".pre.");
    let mut found: Vec<Segment> = Vec::new();
    let mut current = String::new();
    let mut digits = false;
    for c in spelled.chars() {
        if c.is_ascii_digit() || c.is_ascii_alphabetic() {
            if !current.is_empty() && c.is_ascii_digit() != digits {
                flush(&mut current, digits, &mut found);
            }
            digits = c.is_ascii_digit();
            current.push(c);
        } else {
            flush(&mut current, digits, &mut found);
        }
    }
    flush(&mut current, digits, &mut found);
    let split = found
        .iter()
        .position(|segment| matches!(segment, Segment::Text(_)))
        .unwrap_or(found.len());
    let zero = Segment::Number(String::new());
    let mut release = found[..split].to_vec();
    while release.last() == Some(&zero) {
        release.pop();
    }
    let mut rest = found[split..].to_vec();
    while rest.last() == Some(&zero) {
        rest.pop();
    }
    release.extend(rest);
    release
}

pub fn compare(left: &str, right: &str) -> Ordering {
    let (a, b) = (segments(left), segments(right));
    let zero = Segment::Number(String::new());
    for index in 0..a.len().max(b.len()) {
        let x = a.get(index).unwrap_or(&zero);
        let y = b.get(index).unwrap_or(&zero);
        let ordering = match (x, y) {
            (Segment::Number(x), Segment::Number(y)) => {
                x.len().cmp(&y.len()).then_with(|| x.cmp(y))
            }
            (Segment::Text(x), Segment::Text(y)) => x.cmp(y),
            (Segment::Text(_), Segment::Number(_)) => Ordering::Less,
            (Segment::Number(_), Segment::Text(_)) => Ordering::Greater,
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.cmp(right)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoxName {
    pub org: String,
    pub name: String,
}

impl BoxName {
    pub fn new(org: &str, name: &str) -> Option<Self> {
        (valid_name(org) && valid_name(name)).then(|| Self {
            org: org.to_owned(),
            name: name.to_owned(),
        })
    }

    pub fn tag(&self) -> String {
        format!("{}/{}", self.org, self.name)
    }

    pub fn path(&self) -> String {
        format!("/{}/{}", self.org, self.name)
    }

    pub fn metadata_path(&self) -> String {
        format!("/{}/{}/metadata.json", self.org, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoxFile {
    pub boxed: BoxName,
    pub version: String,
    pub provider: String,
    pub architecture: String,
}

impl BoxFile {
    pub fn new(boxed: BoxName, version: &str, provider: &str, architecture: &str) -> Option<Self> {
        (valid_version(version) && valid_provider(provider) && valid_architecture(architecture))
            .then(|| Self {
                boxed,
                version: version.to_owned(),
                provider: provider.to_owned(),
                architecture: architecture.to_owned(),
            })
    }

    pub fn parse(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        match parts.as_slice() {
            [org, name, version, provider, file] => {
                let architecture = file.strip_suffix(BOX_SUFFIX)?;
                Self::new(BoxName::new(org, name)?, version, provider, architecture)
            }
            _ => None,
        }
    }

    pub fn path(&self) -> String {
        format!(
            "/{}/{}/{}/{}/{}{BOX_SUFFIX}",
            self.boxed.org, self.boxed.name, self.version, self.provider, self.architecture
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_follow_the_rubygems_grammar() {
        for good in [
            "1",
            "1.0.0",
            "20240101.0.0",
            "1.1.0.beta1",
            "2.0.0-rc1",
            "1.0.0-pre.2",
        ] {
            assert!(valid_version(good), "{good}");
        }
        for bad in [
            "",
            "v1.0.0",
            "1..0",
            "1.0.",
            "1.0_1",
            "1.0.0-",
            "-1",
            "1.0.0+build",
            "a",
        ] {
            assert!(!valid_version(bad), "{bad}");
        }
        assert!(is_prerelease("1.1.0.beta1"));
        assert!(is_prerelease("2.0.0-rc1"));
        assert!(!is_prerelease("2.0.0"));
    }

    #[test]
    fn versions_order_like_rubygems() {
        let mut versions = vec![
            "1.10.0",
            "1.2.0",
            "1.2.0.rc1",
            "1.2.0-rc2",
            "2",
            "1.2",
            "0.9.9",
            "1.2.0.a",
        ];
        versions.sort_by(|a, b| compare(a, b));
        assert_eq!(
            versions,
            [
                "0.9.9",
                "1.2.0.a",
                "1.2.0-rc2",
                "1.2.0.rc1",
                "1.2",
                "1.2.0",
                "1.10.0",
                "2"
            ]
        );
        assert_eq!(segments("1.0.a"), segments("1.a"));
        assert_eq!(segments("1.2.0"), segments("1.2"));
        assert_eq!(compare("01.0", "1.0"), Ordering::Less);
        assert_eq!(compare("1.0.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn names_providers_and_architectures_are_path_safe() {
        assert!(valid_name("ubuntu-24.04"));
        assert!(valid_name("acme_corp"));
        assert!(!valid_name(".hidden"));
        assert!(!valid_name("a/b"));
        assert!(!valid_name("a b"));
        assert!(valid_provider("vmware_desktop"));
        assert!(!valid_provider("virtual.box"));
        assert!(known_architecture("arm64"));
        assert!(!known_architecture("x86_64"));
        assert!(valid_architecture("x86_64"));
    }

    #[test]
    fn box_paths_round_trip() {
        let file = BoxFile::parse("/acme/base/1.0.0/virtualbox/amd64.box").unwrap();
        assert_eq!(file.boxed.tag(), "acme/base");
        assert_eq!(file.version, "1.0.0");
        assert_eq!(file.provider, "virtualbox");
        assert_eq!(file.architecture, "amd64");
        assert_eq!(file.path(), "/acme/base/1.0.0/virtualbox/amd64.box");
        for other in [
            "/acme/base/1.0.0/virtualbox/amd64",
            "/acme/base/latest/virtualbox/amd64.box",
            "/acme/base/1.0.0/virtualbox/.box",
            "/acme/base/1.0.0/amd64.box",
        ] {
            assert!(BoxFile::parse(other).is_none(), "{other}");
        }
        assert_eq!(
            BoxName::new("acme", "base").unwrap().metadata_path(),
            "/acme/base/metadata.json"
        );
    }
}
