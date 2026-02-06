use std::cmp::Ordering;

pub const SNAPSHOT: &str = "SNAPSHOT";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timestamped<'a> {
    pub prefix: &'a str,
    pub timestamp: &'a str,
    pub build: u32,
}

impl Timestamped<'_> {
    pub fn base_version(&self) -> String {
        format!("{}{SNAPSHOT}", self.prefix)
    }
}

impl PartialOrd for Timestamped<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamped<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(other.timestamp)
            .then(self.build.cmp(&other.build))
    }
}

pub fn timestamped(version: &str) -> Option<Timestamped<'_>> {
    let (rest, build) = version.rsplit_once('-')?;
    if build.is_empty() || !build.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let build: u32 = build.parse().ok()?;
    let split = rest.len().checked_sub(15)?;
    let (prefix, timestamp) = rest.split_at_checked(split)?;
    if !is_timestamp(timestamp) || !(prefix.is_empty() || prefix.ends_with('-')) {
        return None;
    }
    Some(Timestamped {
        prefix,
        timestamp,
        build,
    })
}

pub fn is_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 15
        && bytes[8] == b'.'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 8 || byte.is_ascii_digit())
}

pub fn is_snapshot(version: &str) -> bool {
    version.ends_with(SNAPSHOT) || timestamped(version).is_some()
}

pub fn base_version(version: &str) -> String {
    match timestamped(version) {
        Some(found) => found.base_version(),
        None => version.to_owned(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Item {
    Int(String),
    Text(String),
    List(Vec<Item>),
}

const QUALIFIERS: [&str; 7] = ["alpha", "beta", "milestone", "rc", "snapshot", "", "sp"];
const RELEASE_INDEX: &str = "5";

impl Item {
    fn number(digits: &str) -> Item {
        let trimmed = digits.trim_start_matches('0');
        Item::Int(if trimmed.is_empty() { "0" } else { trimmed }.to_owned())
    }

    fn text(value: &str, followed_by_digit: bool) -> Item {
        let value = match (followed_by_digit, value) {
            (true, "a") => "alpha",
            (true, "b") => "beta",
            (true, "m") => "milestone",
            (_, "ga" | "final" | "release") => "",
            (_, "cr") => "rc",
            (_, other) => other,
        };
        Item::Text(value.to_owned())
    }

    fn parsed(is_digit: bool, value: &str) -> Item {
        if is_digit {
            Item::number(value)
        } else {
            Item::text(value, false)
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Item::Int(value) => value == "0",
            Item::Text(value) => value.is_empty(),
            Item::List(items) => items.is_empty(),
        }
    }

    fn compare(&self, other: Option<&Item>) -> Ordering {
        match (self, other) {
            (Item::Int(value), None) => {
                if value == "0" {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            }
            (Item::Int(left), Some(Item::Int(right))) => {
                left.len().cmp(&right.len()).then_with(|| left.cmp(right))
            }
            (Item::Int(_), Some(_)) => Ordering::Greater,
            (Item::Text(value), None) => qualifier(value).as_str().cmp(RELEASE_INDEX),
            (Item::Text(_), Some(Item::Int(_))) => Ordering::Less,
            (Item::Text(left), Some(Item::Text(right))) => qualifier(left).cmp(&qualifier(right)),
            (Item::Text(_), Some(Item::List(_))) => Ordering::Less,
            (Item::List(items), None) => items
                .iter()
                .map(|item| item.compare(None))
                .find(|ordering| *ordering != Ordering::Equal)
                .unwrap_or(Ordering::Equal),
            (Item::List(_), Some(Item::Int(_))) => Ordering::Less,
            (Item::List(_), Some(Item::Text(_))) => Ordering::Greater,
            (Item::List(left), Some(Item::List(right))) => compare_lists(left, right),
        }
    }
}

fn qualifier(value: &str) -> String {
    match QUALIFIERS.iter().position(|known| *known == value) {
        Some(index) => index.to_string(),
        None => format!("{}-{value}", QUALIFIERS.len()),
    }
}

fn compare_lists(left: &[Item], right: &[Item]) -> Ordering {
    let length = left.len().max(right.len());
    for index in 0..length {
        let ordering = match (left.get(index), right.get(index)) {
            (None, None) => Ordering::Equal,
            (None, Some(right)) => right.compare(None).reverse(),
            (Some(left), right) => left.compare(right),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn normalize(items: &mut Vec<Item>) {
    let mut index = items.len();
    while index > 0 {
        index -= 1;
        if items[index].is_null() {
            items.remove(index);
        } else if !matches!(items[index], Item::List(_)) {
            break;
        }
    }
}

fn parse(version: &str) -> Vec<Item> {
    let lowered = version.to_lowercase();
    let chars: Vec<char> = lowered.chars().collect();
    let mut levels: Vec<Vec<Item>> = vec![Vec::new()];
    let mut is_digit = false;
    let mut start = 0;
    let text = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
    for (index, &c) in chars.iter().enumerate() {
        let current = levels.len() - 1;
        if c == '.' || c == '-' {
            let item = if index == start {
                Item::Int("0".to_owned())
            } else {
                Item::parsed(is_digit, &text(start, index))
            };
            levels[current].push(item);
            start = index + 1;
            if c == '-' {
                levels.push(Vec::new());
            }
        } else if c.is_ascii_digit() {
            if !is_digit && index > start {
                levels[current].push(Item::text(&text(start, index), true));
                start = index;
                levels.push(Vec::new());
            }
            is_digit = true;
        } else {
            if is_digit && index > start {
                levels[current].push(Item::number(&text(start, index)));
                start = index;
                levels.push(Vec::new());
            }
            is_digit = false;
        }
    }
    if chars.len() > start {
        let current = levels.len() - 1;
        levels[current].push(Item::parsed(is_digit, &text(start, chars.len())));
    }
    let mut child: Option<Vec<Item>> = None;
    while let Some(mut level) = levels.pop() {
        if let Some(nested) = child.take() {
            level.push(Item::List(nested));
        }
        normalize(&mut level);
        child = Some(level);
    }
    child.unwrap_or_default()
}

pub fn compare(left: &str, right: &str) -> Ordering {
    compare_lists(&parse(left), &parse(right))
}

pub fn sort(versions: &mut [String]) {
    versions.sort_by(|left, right| compare(left, right).then_with(|| left.cmp(right)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ordered(versions: &[&str]) {
        for pair in versions.windows(2) {
            assert_eq!(
                compare(pair[0], pair[1]),
                Ordering::Less,
                "{} < {}",
                pair[0],
                pair[1]
            );
            assert_eq!(compare(pair[1], pair[0]), Ordering::Greater);
        }
    }

    #[test]
    fn versions_follow_the_maven_order() {
        ordered(&[
            "1-alpha-1",
            "1-alpha-2",
            "1-beta",
            "1-milestone",
            "1-rc",
            "1-SNAPSHOT",
            "1",
            "1-sp",
            "1-abc",
            "1-1",
            "1.1",
            "1.2",
            "1.10",
            "2.0-RC1",
            "2.0",
            "10.0",
        ]);
        ordered(&["1.0-alpha1", "1.0-beta1", "1.0-SNAPSHOT", "1.0", "1.0.1"]);
        ordered(&["3.8.9", "3.9.0-rc-1", "3.9.0", "3.9.11"]);
    }

    #[test]
    fn equal_spellings_compare_equal() {
        for (left, right) in [
            ("1", "1.0"),
            ("1", "1.0.0"),
            ("1", "1-ga"),
            ("1", "1-final"),
            ("1.0-RC1", "1.0-cr1"),
            ("1a1", "1-alpha-1"),
            ("1.0.0", "1-0"),
            ("1.0-alpha1", "1-alpha1"),
            ("01", "1"),
        ] {
            assert_eq!(compare(left, right), Ordering::Equal, "{left} = {right}");
        }
    }

    #[test]
    fn large_numbers_do_not_overflow() {
        ordered(&["1.99999999999999999999", "1.100000000000000000000"]);
    }

    #[test]
    fn snapshots_are_recognised() {
        assert!(is_snapshot("1.0-SNAPSHOT"));
        assert!(is_snapshot("1.0-20261012.101010-3"));
        assert!(is_snapshot("20261012.101010-3"));
        assert!(!is_snapshot("1.0"));
        assert!(!is_snapshot("1.0-20261012.10101-3"));
        assert!(!is_snapshot("1.020261012.101010-3"));
        let found = timestamped("1.0-20261012.101010-3").unwrap();
        assert_eq!(found.prefix, "1.0-");
        assert_eq!(found.timestamp, "20261012.101010");
        assert_eq!(found.build, 3);
        assert_eq!(base_version("1.0-20261012.101010-3"), "1.0-SNAPSHOT");
        assert_eq!(base_version("1.0"), "1.0");
        assert!(timestamped("1.0-20261012.101010-12").unwrap() > found);
    }
}
