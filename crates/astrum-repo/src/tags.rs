use std::iter::Peekable;
use std::str::Chars;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::Problems;

pub const MAX_ENTRIES: usize = 32;
pub const MAX_ENTRY_LENGTH: usize = 128;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TagRules {
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub exceptions: Vec<String>,
}

impl TagRules {
    pub fn check(&self, problems: &mut Problems) {
        for (field, entries) in self.lists() {
            if entries.len() > MAX_ENTRIES {
                problems.add(field, format!("must have at most {MAX_ENTRIES} entries"));
            }
            for (index, entry) in entries.iter().enumerate() {
                if let Err(message) = compile(entry) {
                    problems.add(&format!("{field}[{index}]"), message);
                }
            }
        }
    }

    pub fn matcher(&self) -> Result<TagMatcher, TagError> {
        Ok(TagMatcher {
            patterns: compile_all("immutableTags.patterns", &self.patterns)?,
            exceptions: compile_all("immutableTags.exceptions", &self.exceptions)?,
        })
    }

    fn lists(&self) -> [(&'static str, &Vec<String>); 2] {
        [
            ("immutableTags.patterns", &self.patterns),
            ("immutableTags.exceptions", &self.exceptions),
        ]
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{field}: {message}")]
pub struct TagError {
    pub field: String,
    pub message: String,
}

fn compile_all(field: &str, entries: &[String]) -> Result<Vec<(String, Regex)>, TagError> {
    if entries.len() > MAX_ENTRIES {
        return Err(TagError {
            field: field.to_owned(),
            message: format!("must have at most {MAX_ENTRIES} entries"),
        });
    }
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            compile(entry)
                .map(|regex| (entry.clone(), regex))
                .map_err(|message| TagError {
                    field: format!("{field}[{index}]"),
                    message,
                })
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct TagMatcher {
    patterns: Vec<(String, Regex)>,
    exceptions: Vec<(String, Regex)>,
}

impl TagMatcher {
    pub fn reason(&self, tag: &str) -> Option<String> {
        if is_referrer_tag(tag) {
            return None;
        }
        let (pattern, _) = self
            .patterns
            .iter()
            .find(|(_, regex)| regex.is_match(tag))?;
        let excepted = self.exceptions.iter().any(|(_, regex)| regex.is_match(tag));
        (!excepted).then(|| format!("rule {pattern}"))
    }
}

fn is_referrer_tag(tag: &str) -> bool {
    let Some(digest) = tag.strip_prefix("sha256-") else {
        return false;
    };
    let digest = [".sig", ".att", ".sbom"]
        .into_iter()
        .find_map(|suffix| digest.strip_suffix(suffix))
        .unwrap_or(digest);
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_tag_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-')
}

fn not_allowed(character: char) -> String {
    format!("must not contain \"{character}\": use tag characters and *?[]")
}

fn compile(pattern: &str) -> Result<Regex, String> {
    if pattern.is_empty() {
        return Err("must not be empty".to_owned());
    }
    let mut source = String::from("(?s)^");
    let mut chars = pattern.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '*' => source.push_str(".*"),
            '?' => source.push('.'),
            '[' => source.push_str(&class(&mut chars)?),
            ']' => return Err("has a ] without a [".to_owned()),
            character if is_tag_character(character) => {
                source.push_str(&regex::escape(&character.to_string()))
            }
            character => return Err(not_allowed(character)),
        }
    }
    if pattern.chars().count() > MAX_ENTRY_LENGTH {
        return Err(format!(
            "must be at most {MAX_ENTRY_LENGTH} characters long"
        ));
    }
    source.push('$');
    Regex::new(&source).map_err(|error| error.to_string())
}

fn class(chars: &mut Peekable<Chars<'_>>) -> Result<String, String> {
    let mut source = String::from("[");
    loop {
        let first = chars
            .next()
            .ok_or_else(|| "has a [ without a closing ]".to_owned())?;
        if first == ']' {
            break;
        }
        if !is_tag_character(first) {
            return Err(not_allowed(first));
        }
        if chars.peek() != Some(&'-') {
            source.push_str(&regex::escape(&first.to_string()));
            continue;
        }
        chars.next();
        match chars.peek().copied() {
            None | Some(']') => {
                source.push_str(&regex::escape(&first.to_string()));
                source.push_str("\\-");
            }
            Some(last) => {
                chars.next();
                if !is_tag_character(last) {
                    return Err(not_allowed(last));
                }
                if last < first {
                    return Err("has an invalid range in []".to_owned());
                }
                source.push_str(&regex::escape(&first.to_string()));
                source.push('-');
                source.push_str(&regex::escape(&last.to_string()));
            }
        }
    }
    if source == "[" {
        return Err("has an empty []".to_owned());
    }
    source.push(']');
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher(patterns: &[&str], exceptions: &[&str]) -> TagMatcher {
        TagRules {
            patterns: patterns
                .iter()
                .map(|pattern| (*pattern).to_owned())
                .collect(),
            exceptions: exceptions
                .iter()
                .map(|pattern| (*pattern).to_owned())
                .collect(),
        }
        .matcher()
        .unwrap()
    }

    fn refused(pattern: &str) -> bool {
        TagRules {
            patterns: vec![pattern.to_owned()],
            exceptions: Vec::new(),
        }
        .matcher()
        .is_err()
    }

    #[test]
    fn star_matches_the_whole_tag_and_not_latest() {
        let rules = matcher(&["v*"], &[]);
        assert_eq!(rules.reason("v1.2"), Some("rule v*".to_owned()));
        assert_eq!(rules.reason("latest"), None);
    }

    #[test]
    fn classes_and_question_marks_match_one_character_each() {
        let rules = matcher(&["[0-9]*"], &[]);
        assert!(rules.reason("1.0").is_some());
        assert!(rules.reason("v1").is_none());
        let rules = matcher(&["v?"], &[]);
        assert!(rules.reason("v1").is_some());
        assert!(rules.reason("v").is_none());
        assert!(rules.reason("v12").is_none());
    }

    #[test]
    fn an_exception_wins_over_a_pattern() {
        let rules = matcher(&["v*"], &["v*-rc*"]);
        assert!(rules.reason("v1.0").is_some());
        assert!(rules.reason("v1.0-rc1").is_none());
    }

    #[test]
    fn referrer_tags_are_always_exempt() {
        let digest = "0123456789abcdef".repeat(4);
        let rules = matcher(&["*"], &[]);
        for tag in [
            format!("sha256-{digest}"),
            format!("sha256-{digest}.sig"),
            format!("sha256-{digest}.att"),
            format!("sha256-{digest}.sbom"),
        ] {
            assert_eq!(rules.reason(&tag), None, "{tag}");
        }
        for tag in [
            format!("sha256-{digest}.dsse"),
            format!("sha256-{digest}.sigx"),
            format!("sha256-{digest}.sig.sig"),
            format!("sha256-{digest}.SIG"),
            format!("sha256-{}", &digest[1..]),
            format!("sha256-{}.sig", &digest[1..]),
            format!("sha256-{digest}0"),
            format!("sha256-{digest}0.sig"),
            format!("sha256-{}", digest.to_uppercase()),
            format!("sha256-{}.sig", digest.to_uppercase()),
            format!("sha512-{digest}{digest}"),
            format!("x-sha256-{digest}"),
        ] {
            assert!(rules.reason(&tag).is_some(), "{tag}");
        }
    }

    #[test]
    fn patterns_are_anchored_over_the_whole_tag() {
        let rules = matcher(&["v1"], &[]);
        assert!(rules.reason("v1").is_some());
        assert!(rules.reason("av1").is_none());
        assert!(rules.reason("v12").is_none());
    }

    #[test]
    fn dots_in_patterns_are_literal() {
        let rules = matcher(&["v1.2"], &[]);
        assert!(rules.reason("v1.2").is_some());
        assert!(rules.reason("v1x2").is_none());
    }

    #[test]
    fn characters_outside_the_allowed_set_are_refused() {
        for pattern in ["v+", "(a)", "a|b", "a\\b", "^a", "a$", "a{1}", "a!", "a b"] {
            assert!(refused(pattern), "{pattern}");
        }
        let error = TagRules {
            patterns: vec!["é".repeat(100)],
            exceptions: Vec::new(),
        }
        .matcher()
        .unwrap_err();
        assert!(
            error.message.starts_with("must not contain \"é\""),
            "{}",
            error.message
        );
    }

    #[test]
    fn malformed_classes_are_refused() {
        for pattern in ["[", "[]", "[a-", "[z-a]", "]"] {
            assert!(refused(pattern), "{pattern}");
        }
    }

    #[test]
    fn empty_patterns_mean_off() {
        let rules = TagRules::default().matcher().unwrap();
        assert_eq!(rules.reason("v1.2"), None);
    }
}
