use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    numbers: [u64; 4],
    parts: usize,
    pre: Vec<String>,
    metadata: Option<String>,
}

fn number(text: &str) -> Option<u64> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn identifier(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

impl Version {
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim();
        if text.is_empty() || text.len() > 64 {
            return None;
        }
        let (rest, metadata) = match text.split_once('+') {
            Some((rest, metadata)) => {
                if !metadata.split('.').all(identifier) {
                    return None;
                }
                (rest, Some(metadata.to_owned()))
            }
            None => (text, None),
        };
        let (release, pre) = match rest.split_once('-') {
            Some((release, pre)) => {
                let labels: Vec<String> = pre.split('.').map(str::to_owned).collect();
                if !labels.iter().all(|label| identifier(label)) {
                    return None;
                }
                (release, labels)
            }
            None => (rest, Vec::new()),
        };
        let pieces: Vec<&str> = release.split('.').collect();
        if pieces.is_empty() || pieces.len() > 4 {
            return None;
        }
        let mut numbers = [0u64; 4];
        for (slot, piece) in numbers.iter_mut().zip(&pieces) {
            *slot = number(piece)?;
        }
        Some(Version {
            numbers,
            parts: pieces.len(),
            pre,
            metadata,
        })
    }

    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    pub fn is_semver2(&self) -> bool {
        self.pre.len() > 1 || self.metadata.is_some()
    }

    pub fn normalized(&self) -> String {
        let [major, minor, patch, revision] = self.numbers;
        let mut text = format!("{major}.{minor}.{patch}");
        if revision != 0 {
            text.push_str(&format!(".{revision}"));
        }
        if !self.pre.is_empty() {
            text.push('-');
            text.push_str(&self.pre.join("."));
        }
        text
    }

    pub fn full(&self) -> String {
        match &self.metadata {
            Some(metadata) => format!("{}+{metadata}", self.normalized()),
            None => self.normalized(),
        }
    }

    pub fn lower(&self) -> String {
        self.normalized().to_ascii_lowercase()
    }

    pub fn has_four_parts(&self) -> bool {
        self.parts == 4
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.normalized())
    }
}

fn compare_label(left: &str, right: &str) -> Ordering {
    match (number(left), number(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()),
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.numbers.cmp(&other.numbers).then_with(|| {
            match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => {
                    for (left, right) in self.pre.iter().zip(&other.pre) {
                        let ordering = compare_label(left, right);
                        if ordering != Ordering::Equal {
                            return ordering;
                        }
                    }
                    self.pre.len().cmp(&other.pre.len())
                }
            }
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn compare_text(left: &str, right: &str) -> Ordering {
    match (Version::parse(left), Version::parse(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()),
    }
}

pub fn same(left: &str, right: &str) -> bool {
    match (Version::parse(left), Version::parse(right)) {
        (Some(left), Some(right)) => left.cmp(&right) == Ordering::Equal,
        _ => left.eq_ignore_ascii_case(right),
    }
}

pub fn range(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() {
        return "(, )".to_owned();
    }
    match Version::parse(text) {
        Some(version) => format!("[{}, )", version.full()),
        None => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_normalised_like_the_gallery_does() {
        for (given, normalized) in [
            ("1.0", "1.0.0"),
            ("1.0.0.0", "1.0.0"),
            ("01.002.3", "1.2.3"),
            ("1.2.3.4", "1.2.3.4"),
            ("1.0.0-Beta.1", "1.0.0-Beta.1"),
            ("1.0.0+meta", "1.0.0"),
            ("2", "2.0.0"),
        ] {
            assert_eq!(Version::parse(given).unwrap().normalized(), normalized);
        }
        assert_eq!(Version::parse("1.0.0+git.1").unwrap().full(), "1.0.0+git.1");
        assert_eq!(Version::parse("1.0.0-RC").unwrap().lower(), "1.0.0-rc");
        for bad in [
            "",
            "a.b",
            "1..2",
            "1.2.3.4.5",
            "1.0-",
            "1.0-be ta",
            "1.0+",
            "-1",
        ] {
            assert!(Version::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn versions_compare_by_numbers_then_labels() {
        let mut sorted = vec![
            "1.0.0",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-alpha",
            "0.9",
            "1.0.0.1",
            "1.0.0-Beta",
            "10.0.0",
        ];
        sorted.sort_by(|left, right| compare_text(left, right));
        assert_eq!(
            sorted,
            vec![
                "0.9",
                "1.0.0-alpha",
                "1.0.0-Beta",
                "1.0.0-beta.2",
                "1.0.0-beta.11",
                "1.0.0",
                "1.0.0.1",
                "10.0.0"
            ]
        );
        assert!(same("1.0", "1.0.0.0"));
        assert!(same("1.0.0-RC", "1.0.0-rc"));
    }

    #[test]
    fn semver2_and_prerelease_are_recognised() {
        assert!(Version::parse("1.0.0-beta.1").unwrap().is_semver2());
        assert!(Version::parse("1.0.0+meta").unwrap().is_semver2());
        assert!(!Version::parse("1.0.0-beta1").unwrap().is_semver2());
        assert!(Version::parse("1.0.0-beta1").unwrap().is_prerelease());
        assert!(!Version::parse("1.0.0").unwrap().is_prerelease());
    }

    #[test]
    fn plain_versions_become_minimum_ranges() {
        assert_eq!(range("13.0.1"), "[13.0.1, )");
        assert_eq!(range("[1.0, 2.0)"), "[1.0, 2.0)");
        assert_eq!(range(""), "(, )");
    }
}
