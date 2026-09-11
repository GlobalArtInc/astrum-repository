use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Part {
    Number(u64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Vec<Part>,
    build: Vec<Part>,
}

fn identifiers(text: &str) -> Option<Vec<Part>> {
    text.split('.')
        .map(|part| {
            if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return None;
            }
            Some(match part.parse::<u64>() {
                Ok(number) if part.chars().all(|c| c.is_ascii_digit()) => Part::Number(number),
                _ => Part::Text(part.to_owned()),
            })
        })
        .collect()
}

fn number(text: &str) -> Option<u64> {
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

impl Version {
    pub fn parse(text: &str) -> Option<Version> {
        if text.len() > 128 {
            return None;
        }
        let (rest, build) = match text.split_once('+') {
            Some((rest, build)) => (rest, identifiers(build)?),
            None => (text, Vec::new()),
        };
        let (core, pre) = match rest.split_once('-') {
            Some((core, pre)) => (core, identifiers(pre)?),
            None => (rest, Vec::new()),
        };
        let mut numbers = core.split('.');
        let major = number(numbers.next()?)?;
        let minor = number(numbers.next()?)?;
        let patch = number(numbers.next()?)?;
        if numbers.next().is_some() {
            return None;
        }
        Some(Version {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }

    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }
}

fn compare_parts(left: &[Part], right: &[Part]) -> Ordering {
    for (a, b) in left.iter().zip(right) {
        let order = match (a, b) {
            (Part::Number(a), Part::Number(b)) => a.cmp(b),
            (Part::Number(_), Part::Text(_)) => Ordering::Less,
            (Part::Text(_), Part::Number(_)) => Ordering::Greater,
            (Part::Text(a), Part::Text(b)) => a.cmp(b),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    left.len().cmp(&right.len())
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => compare_parts(&self.pre, &other.pre),
            })
            .then_with(|| match (self.build.is_empty(), other.build.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => compare_parts(&self.build, &other.build),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn valid(text: &str) -> bool {
    Version::parse(text).is_some()
}

pub fn is_prerelease(text: &str) -> bool {
    Version::parse(text).is_some_and(|version| version.is_prerelease())
}

pub fn compare(left: &str, right: &str) -> Ordering {
    match (Version::parse(left), Version::parse(right)) {
        (Some(a), Some(b)) => a.cmp(&b).then_with(|| left.cmp(right)),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => left.cmp(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_like_semantic_versions() {
        for good in [
            "0.0.1",
            "1.2.3",
            "1.0.0-dev.1",
            "2.0.0-nullsafety.0",
            "1.0.0+1",
            "1.0.0-beta+exp.sha.5114f85",
            "10.20.30",
        ] {
            assert!(valid(good), "{good}");
        }
        for bad in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "v1.2.3",
            "1.2.x",
            "1.2.3-",
            "1.2.3+",
            "1.2.3-a..b",
            "1.2.3 ",
            "01a.2.3",
        ] {
            assert!(!valid(bad), "{bad}");
        }
        assert!(is_prerelease("1.0.0-dev.1"));
        assert!(is_prerelease("1.0.0-dev+2"));
        assert!(!is_prerelease("1.0.0+build-1"));
    }

    #[test]
    fn versions_sort_like_the_dart_client() {
        let mut versions = vec![
            "1.0.0",
            "1.0.0+1",
            "1.0.0-beta.11",
            "1.0.0-alpha",
            "1.0.0-beta.2",
            "0.9.10",
            "1.0.0-alpha.1",
            "0.9.9",
            "1.0.0+0.1",
            "1.0.0-rc.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
        ];
        versions.sort_by(|a, b| compare(a, b));
        assert_eq!(
            versions,
            [
                "0.9.9",
                "0.9.10",
                "1.0.0-alpha",
                "1.0.0-alpha.1",
                "1.0.0-alpha.beta",
                "1.0.0-beta",
                "1.0.0-beta.2",
                "1.0.0-beta.11",
                "1.0.0-rc.1",
                "1.0.0",
                "1.0.0+0.1",
                "1.0.0+1",
            ]
        );
    }
}
