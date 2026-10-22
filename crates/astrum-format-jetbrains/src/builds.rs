use std::cmp::Ordering;

const WILDCARD: u64 = u64::MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build(Vec<u64>);

fn without_product(text: &str) -> &str {
    match text.split_once('-') {
        Some((product, rest))
            if (1..=5).contains(&product.len())
                && product.chars().all(|c| c.is_ascii_alphabetic())
                && rest.starts_with(|c: char| c.is_ascii_digit()) =>
        {
            rest
        }
        _ => text,
    }
}

impl Build {
    pub fn parse(text: &str) -> Option<Build> {
        let number = without_product(text.trim());
        if number.is_empty() {
            return None;
        }
        let mut parts = Vec::new();
        for part in number.split('.') {
            if part == "*" || part.eq_ignore_ascii_case("snapshot") {
                parts.push(WILDCARD);
            } else if !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) {
                parts.push(part.parse().ok()?);
            } else {
                return None;
            }
        }
        Some(Build(parts))
    }

    pub fn compatible(&self, since: Option<&str>, until: Option<&str>) -> bool {
        if let Some(since) = since.and_then(Build::parse)
            && self < &since
        {
            return false;
        }
        if let Some(until) = until.and_then(Build::parse)
            && self > &until
        {
            return false;
        }
        true
    }
}

impl PartialOrd for Build {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Build {
    fn cmp(&self, other: &Self) -> Ordering {
        for (left, right) in self.0.iter().zip(&other.0) {
            match left.cmp(right) {
                Ordering::Equal => {}
                unequal => return unequal,
            }
        }
        self.0.len().cmp(&other.0.len())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
    Number(&'a str),
    Word(String),
}

fn tokens(version: &str) -> Vec<Token<'_>> {
    let mut found = Vec::new();
    let mut chars = version.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if c.is_ascii_digit() {
            let mut end = start + 1;
            while let Some(&(at, next)) = chars.peek() {
                if !next.is_ascii_digit() {
                    break;
                }
                end = at + 1;
                chars.next();
            }
            found.push(Token::Number(version[start..end].trim_start_matches('0')));
        } else if c.is_alphabetic() {
            let mut word = c.to_lowercase().to_string();
            while let Some(&(_, next)) = chars.peek() {
                if !next.is_alphabetic() {
                    break;
                }
                word.extend(next.to_lowercase());
                chars.next();
            }
            found.push(Token::Word(word));
        }
    }
    found
}

fn compare_tokens(left: &Token<'_>, right: &Token<'_>) -> Ordering {
    match (left, right) {
        (Token::Number(a), Token::Number(b)) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
        (Token::Word(a), Token::Word(b)) => a.cmp(b),
        (Token::Number(_), Token::Word(_)) => Ordering::Greater,
        (Token::Word(_), Token::Number(_)) => Ordering::Less,
    }
}

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    let (a, b) = (tokens(left), tokens(right));
    for (x, y) in a.iter().zip(&b) {
        match compare_tokens(x, y) {
            Ordering::Equal => {}
            unequal => return unequal,
        }
    }
    match a.len().cmp(&b.len()) {
        Ordering::Equal => left.cmp(right),
        Ordering::Greater => match a[b.len()] {
            Token::Word(_) => Ordering::Less,
            Token::Number(_) => Ordering::Greater,
        },
        Ordering::Less => match b[a.len()] {
            Token::Word(_) => Ordering::Greater,
            Token::Number(_) => Ordering::Less,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(text: &str) -> Build {
        Build::parse(text).unwrap()
    }

    #[test]
    fn builds_drop_the_product_code() {
        assert_eq!(build("IU-243.21565.193"), build("243.21565.193"));
        assert_eq!(build("GO-251.1"), build("251.1"));
        assert!(Build::parse("IU-").is_none());
        assert!(Build::parse("").is_none());
        assert!(Build::parse("243.abc").is_none());
        assert!(Build::parse("243..1").is_none());
    }

    #[test]
    fn builds_compare_component_wise_and_longer_is_greater() {
        assert!(build("243.21565.193") > build("243.9999"));
        assert!(build("243.21565.193") > build("243"));
        assert!(build("251.1") > build("243.99999"));
        assert!(build("243.1.1") < build("243.2"));
    }

    #[test]
    fn until_build_wildcards_cover_the_branch() {
        let ide = build("IU-243.21565.193");
        assert!(ide.compatible(Some("243.21565"), Some("243.*")));
        assert!(ide.compatible(Some("242.1"), Some("243.SNAPSHOT")));
        assert!(!ide.compatible(Some("243.21566"), None));
        assert!(!ide.compatible(None, Some("243")));
        assert!(!ide.compatible(None, Some("242.*")));
        assert!(ide.compatible(None, None));
        assert!(ide.compatible(Some("garbage"), Some("")));
    }

    #[test]
    fn versions_compare_numerically_and_pre_releases_come_first() {
        use Ordering::*;
        assert_eq!(compare_versions("1.10.0", "1.9.9"), Greater);
        assert_eq!(compare_versions("2.19.0", "2.19.0"), Equal);
        assert_eq!(compare_versions("1.0.1", "1.0"), Greater);
        assert_eq!(compare_versions("1.0-beta", "1.0"), Less);
        assert_eq!(compare_versions("1.0", "1.0-eap"), Greater);
        assert_eq!(compare_versions("243.21565.122", "243.9.1"), Greater);
        assert_eq!(compare_versions("1.0-alpha", "1.0-beta"), Less);
        assert_eq!(compare_versions("2024.1", "2023.3.5"), Greater);
    }
}
