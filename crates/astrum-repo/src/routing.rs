use std::collections::HashMap;

use astrum_db::entity::routing_rule;
use regex::RegexSet;
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder};
use serde::{Deserialize, Serialize};

use crate::config::{Problem, Problems};
use crate::store::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Mode {
    Allow,
    Block,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Allow => "ALLOW",
            Mode::Block => "BLOCK",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "ALLOW" => Some(Mode::Allow),
            "BLOCK" => Some(Mode::Block),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingRule {
    pub name: String,
    pub description: String,
    pub mode: Mode,
    pub matchers: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub mode: Option<String>,
    pub matchers: Option<Vec<String>>,
}

pub const MAX_NAME: usize = 200;
const MAX_PATTERN: usize = 10_000;

pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric() || first == '-')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

pub fn check_matcher(pattern: &str) -> Result<(), String> {
    if pattern.trim().is_empty() {
        return Err("must not be empty".to_owned());
    }
    if pattern.len() > MAX_PATTERN {
        return Err(format!("must be at most {MAX_PATTERN} characters"));
    }
    regex::Regex::new(&anchored(pattern))
        .map(|_| ())
        .map_err(|error| match error {
            regex::Error::Syntax(text) => format!(
                "is not a valid regular expression: {}",
                text.lines().last().unwrap_or("syntax error").trim()
            ),
            regex::Error::CompiledTooBig(_) => "is too complex".to_owned(),
            _ => "is not a valid regular expression".to_owned(),
        })
}

pub fn validate(request: RuleRequest) -> Result<RoutingRule, Vec<Problem>> {
    let mut problems = Problems::default();
    let name = request.name.unwrap_or_default().trim().to_owned();
    if name.is_empty() {
        problems.add("name", "must not be empty");
    } else if name.len() > MAX_NAME {
        problems.add("name", format!("must be at most {MAX_NAME} characters"));
    } else if !valid_name(&name) {
        problems.add(
            "name",
            "may contain only letters, digits, underscores, hyphens and dots, and cannot start with an underscore or a dot",
        );
    }
    let mode = match request.mode.as_deref().map(str::trim) {
        None | Some("") => {
            problems.add("mode", "A non-empty value must be specified");
            None
        }
        Some(value) => {
            let parsed = Mode::parse(&value.to_ascii_uppercase());
            if parsed.is_none() {
                problems.add("mode", "must be ALLOW or BLOCK");
            }
            parsed
        }
    };
    let matchers: Vec<String> = request.matchers.unwrap_or_default();
    if matchers.is_empty() {
        problems.add("matchers", "At least one rule must be specified");
    }
    for (index, pattern) in matchers.iter().enumerate() {
        if let Err(message) = check_matcher(pattern) {
            problems.add(&format!("matchers[{index}]"), message);
        }
    }
    problems.finish()?;
    Ok(RoutingRule {
        name,
        description: request.description.unwrap_or_default().trim().to_owned(),
        mode: mode.expect("checked above"),
        matchers,
    })
}

fn anchored(pattern: &str) -> String {
    format!("^(?:{pattern})$")
}

#[derive(Clone, Debug)]
pub struct Matcher {
    mode: Mode,
    set: RegexSet,
}

impl Matcher {
    pub fn new(mode: Mode, matchers: &[String]) -> Result<Self, regex::Error> {
        Ok(Self {
            mode,
            set: RegexSet::new(matchers.iter().map(|pattern| anchored(pattern)))?,
        })
    }

    pub fn allows(&self, path: &str) -> bool {
        let hit = self.set.is_match(path);
        match self.mode {
            Mode::Allow => hit,
            Mode::Block => !hit,
        }
    }
}

impl RoutingRule {
    pub fn matcher(&self) -> Result<Matcher, regex::Error> {
        Matcher::new(self.mode, &self.matchers)
    }

    pub fn allows(&self, path: &str) -> bool {
        self.matcher().is_ok_and(|matcher| matcher.allows(path))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MatcherResult {
    pub matches: bool,
    pub error: Option<String>,
}

pub fn evaluate(mode: Mode, matchers: &[String], path: &str) -> (bool, Vec<MatcherResult>) {
    let results: Vec<MatcherResult> = matchers
        .iter()
        .map(|pattern| match check_matcher(pattern) {
            Ok(()) => MatcherResult {
                matches: regex::Regex::new(&anchored(pattern))
                    .is_ok_and(|regex| regex.is_match(path)),
                error: None,
            },
            Err(message) => MatcherResult {
                matches: false,
                error: Some(message),
            },
        })
        .collect();
    let hit = results.iter().any(|result| result.matches);
    let allowed = match mode {
        Mode::Allow => hit,
        Mode::Block => !hit,
    };
    (allowed, results)
}

pub fn parse_mode(value: &str) -> Option<Mode> {
    Mode::parse(&value.trim().to_ascii_uppercase())
}

pub fn request_path(path: &str) -> String {
    match path.starts_with('/') {
        true => path.to_owned(),
        false => format!("/{path}"),
    }
}

fn read(row: routing_rule::Model) -> RoutingRule {
    RoutingRule {
        mode: Mode::parse(&row.mode).unwrap_or(Mode::Block),
        matchers: serde_json::from_value(row.matchers).unwrap_or_default(),
        name: row.name,
        description: row.description,
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<RoutingRule>, Error> {
    Ok(routing_rule::Entity::find()
        .order_by_asc(routing_rule::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(read)
        .collect())
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<RoutingRule>, Error> {
    Ok(routing_rule::Entity::find_by_id(name.to_owned())
        .one(db)
        .await?
        .map(read))
}

pub async fn matchers(db: &impl ConnectionTrait) -> Result<HashMap<String, Matcher>, Error> {
    let mut compiled = HashMap::new();
    for rule in list(db).await? {
        match rule.matcher() {
            Ok(matcher) => {
                compiled.insert(rule.name, matcher);
            }
            Err(error) => {
                tracing::warn!(rule = rule.name, %error, "a stored routing rule does not compile; it blocks everything");
                compiled.insert(
                    rule.name,
                    Matcher::new(Mode::Allow, &[]).expect("an empty set compiles"),
                );
            }
        }
    }
    Ok(compiled)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(mode: &str, matchers: &[&str]) -> RuleRequest {
        RuleRequest {
            name: Some("r-1".to_owned()),
            description: None,
            mode: Some(mode.to_owned()),
            matchers: Some(matchers.iter().map(|m| (*m).to_owned()).collect()),
        }
    }

    #[test]
    fn block_and_allow_match_the_whole_path() {
        let block = validate(rule("BLOCK", &[".*/internal/.*", "^/secret"])).unwrap();
        assert!(!block.allows("/com/internal/a.jar"));
        assert!(!block.allows("/secret"));
        assert!(block.allows("/secret/x"));
        assert!(block.allows("/com/example/a.jar"));
        let allow = validate(rule("ALLOW", &["/com/example/.*"])).unwrap();
        assert!(allow.allows("/com/example/a.jar"));
        assert!(!allow.allows("/org/example/a.jar"));
        assert!(!allow.allows("/x/com/example/a.jar"));
    }

    #[test]
    fn invalid_rules_are_refused_field_by_field() {
        let problems = validate(RuleRequest {
            name: Some("_bad".to_owned()),
            description: None,
            mode: Some("DENY".to_owned()),
            matchers: Some(vec!["(".to_owned(), " ".to_owned()]),
        })
        .unwrap_err();
        let ids: Vec<&str> = problems.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "PARAMETER name",
                "PARAMETER mode",
                "PARAMETER matchers[0]",
                "PARAMETER matchers[1]"
            ]
        );
        let empty = validate(RuleRequest::default()).unwrap_err();
        assert_eq!(empty.len(), 3);
        assert_eq!(empty[2].message, "At least one rule must be specified");
    }

    #[test]
    fn evaluation_reports_each_matcher() {
        let matchers = vec!["/com/.*".to_owned(), "(".to_owned(), ".*\\.jar".to_owned()];
        let (allowed, results) = evaluate(Mode::Block, &matchers, "/com/a.jar");
        assert!(!allowed);
        assert!(results[0].matches);
        assert!(results[1].error.is_some());
        assert!(results[2].matches);
        let (allowed, _) = evaluate(Mode::Allow, &matchers[1..2], "/com/a.jar");
        assert!(!allowed);
        let (allowed, results) = evaluate(Mode::Block, &["com/.*".to_owned()], "/com/a.jar");
        assert!(allowed, "matchers match the whole path");
        assert!(!results[0].matches);
    }

    #[test]
    fn names_follow_the_documented_pattern() {
        for good in ["a", "-a", "a.b_c-d", "9"] {
            assert!(valid_name(good), "{good}");
        }
        for bad in ["", "_a", ".a", "a b", "a/b"] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn paths_get_a_leading_slash() {
        assert_eq!(request_path("a/b"), "/a/b");
        assert_eq!(request_path("/a"), "/a");
        assert_eq!(request_path(""), "/");
    }
}
