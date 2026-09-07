use std::cmp::Ordering;

pub const WIDTH: usize = 72;
pub const EXDENT: usize = 8;

pub const SOURCE_FIELDS: [&str; 15] = [
    "Package",
    "Version",
    "Priority",
    "Depends",
    "Imports",
    "LinkingTo",
    "Suggests",
    "Enhances",
    "License",
    "License_is_FOSS",
    "License_restricts_use",
    "OS_type",
    "Archs",
    "MD5sum",
    "NeedsCompilation",
];

pub const BINARY_FIELDS: [&str; 14] = [
    "Package",
    "Version",
    "Priority",
    "Depends",
    "Imports",
    "LinkingTo",
    "Suggests",
    "Enhances",
    "License",
    "License_is_FOSS",
    "License_restricts_use",
    "OS_type",
    "Archs",
    "MD5sum",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub fields: Vec<(String, String)>,
    pub raw: String,
}

impl Record {
    pub fn get(&self, tag: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == tag)
            .map(|(_, value)| value.as_str())
    }
}

fn finish(records: &mut Vec<Record>, current: &mut Record, raw: &mut Vec<String>) {
    if !current.fields.is_empty() {
        current.raw = raw.join("\n");
        records.push(std::mem::take(current));
    }
    raw.clear();
}

pub fn parse(text: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut current = Record::default();
    let mut raw: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.trim().is_empty() {
            finish(&mut records, &mut current, &mut raw);
            continue;
        }
        if line.starts_with([' ', '\t']) {
            let Some((_, value)) = current.fields.last_mut() else {
                continue;
            };
            raw.push(line.to_owned());
            let piece = match line.trim() {
                "." => "",
                piece => piece,
            };
            if !value.is_empty() {
                value.push('\n');
            }
            value.push_str(piece);
            continue;
        }
        let Some((tag, value)) = line.split_once(':') else {
            continue;
        };
        if tag.is_empty() || tag.contains(char::is_whitespace) {
            continue;
        }
        raw.push(line.to_owned());
        current
            .fields
            .push((tag.to_owned(), value.trim().to_owned()));
    }
    finish(&mut records, &mut current, &mut raw);
    records
}

fn sentence_end(word: &str) -> bool {
    word.trim_end_matches([')', '"', '\''])
        .ends_with(['.', '?', '!'])
}

pub fn field(tag: &str, value: &str) -> String {
    let text = format!("{tag}: {value}");
    let pieces: Vec<&str> = text.split([' ', '\t', '\n']).collect();
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut length = 0;
    let mut has_word = false;
    let mut gap = 1;
    for (index, word) in pieces.iter().enumerate() {
        if word.is_empty() {
            continue;
        }
        let width = word.chars().count();
        if has_word && length + gap + width > WIDTH - 1 {
            lines.push(std::mem::take(&mut line));
            line.push_str(&" ".repeat(EXDENT));
            length = EXDENT;
            has_word = false;
        }
        if has_word {
            line.push_str(&" ".repeat(gap));
            length += gap;
        }
        line.push_str(word);
        length += width;
        has_word = true;
        let doubled = pieces.get(index + 1).is_some_and(|next| next.is_empty());
        gap = if doubled && sentence_end(word) { 2 } else { 1 };
    }
    lines.push(line);
    lines.join("\n")
}

pub fn valid_package(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'.')
}

pub fn version_parts(version: &str) -> Option<Vec<u64>> {
    let parts: Vec<&str> = version.split(['.', '-']).collect();
    if parts.len() < 2
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    parts.iter().map(|part| part.parse().ok()).collect()
}

pub fn valid_version(version: &str) -> bool {
    version_parts(version).is_some()
}

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    match (version_parts(left), version_parts(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => left.cmp(right),
    }
}

pub fn decode(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => bytes.iter().map(|byte| char::from(*byte)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_are_read_like_read_dcf() {
        let records = parse(
            "Package: a  \nImports:\n    utils,   \n\tstats\nDepends:   R (>= 3.0)\t\nTitle: x\n  .\n  y\n#comment: z\nNote:\nLast: w\n\n\nPackage: b\r\nVersion: 1.0\r\n",
        );
        assert_eq!(records.len(), 2);
        let first = &records[0];
        assert_eq!(first.get("Package"), Some("a"));
        assert_eq!(first.get("Imports"), Some("utils,\nstats"));
        assert_eq!(first.get("Depends"), Some("R (>= 3.0)"));
        assert_eq!(first.get("Title"), Some("x\n\ny"));
        assert_eq!(first.get("#comment"), Some("z"));
        assert_eq!(first.get("Note"), Some(""));
        assert_eq!(first.get("Last"), Some("w"));
        assert!(first.raw.starts_with("Package: a  \nImports:\n    utils,"));
        assert_eq!(records[1].get("Version"), Some("1.0"));
        assert_eq!(records[1].raw, "Package: b\nVersion: 1.0");
    }

    #[test]
    fn long_values_wrap_like_write_dcf() {
        assert_eq!(
            field(
                "Depends",
                "R (>= 4.0.0), methods, stats, utils, graphics, grDevices,\ndatasets, tools, parallel, splines, grid"
            ),
            "Depends: R (>= 4.0.0), methods, stats, utils, graphics, grDevices,\n        datasets, tools, parallel, splines, grid"
        );
        assert_eq!(
            field("Imports", "utils,\nstats (>= 3.0),\ntools"),
            "Imports: utils, stats (>= 3.0), tools"
        );
        assert_eq!(field("Imports", ""), "Imports:");
        let x = "x".repeat(60);
        let y = "y".repeat(60);
        let fits = format!("{x} {y} {}", "z".repeat(63));
        assert_eq!(
            field("Depends", &fits),
            format!("Depends: {x}\n        {y}\n        {}", "z".repeat(63))
        );
        let long = format!("{} y", "x".repeat(80));
        assert_eq!(
            field("Depends", &long),
            format!("Depends:\n        {}\n        y", "x".repeat(80))
        );
        assert_eq!(
            field("Depends", "a.  b. c?  d!   e"),
            "Depends: a.  b. c?  d!  e"
        );
        assert_eq!(field("Depends", "a.\nb"), "Depends: a. b");
    }

    #[test]
    fn names_and_versions_follow_writing_r_extensions() {
        for good in ["ab", "data.table", "R6", "A3"] {
            assert!(valid_package(good), "{good}");
        }
        for bad in ["a", "1ab", "ab.", "a_b", "ab-c", "äb"] {
            assert!(!valid_package(bad), "{bad}");
        }
        for good in ["1.0", "1.0-1", "0.2.1", "2.0-0", "1.10.0.9000"] {
            assert!(valid_version(good), "{good}");
        }
        for bad in ["1", "1..0", "1.0a", "-1.0", "1.0-", ""] {
            assert!(!valid_version(bad), "{bad}");
        }
        assert_eq!(compare_versions("1.10", "1.9"), Ordering::Greater);
        assert_eq!(compare_versions("2.0-0", "2.0"), Ordering::Greater);
        assert_eq!(compare_versions("1.0-1", "1.0.1"), Ordering::Equal);
        assert_eq!(compare_versions("1.10-1", "1.10"), Ordering::Greater);
    }

    #[test]
    fn latin1_descriptions_are_decoded() {
        assert_eq!(decode(b"Author: J\xf6rg"), "Author: Jörg");
        assert_eq!(decode("Author: Jörg".as_bytes()), "Author: Jörg");
    }
}
