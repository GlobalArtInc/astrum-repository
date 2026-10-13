use std::collections::BTreeMap;

use crate::Level;

pub const ROOT: &str = "ROOT";
const MAX_NAME: usize = 200;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Directives {
    pub root: Option<Level>,
    pub loggers: Vec<(String, Level)>,
    pub other: Vec<String>,
}

pub fn parse(filter: &str) -> Directives {
    let mut directives = Directives::default();
    for directive in filter.split(',').map(str::trim).filter(|d| !d.is_empty()) {
        match directive.split_once('=') {
            None => match directive.parse::<Level>() {
                Ok(level) => directives.root = Some(level),
                Err(_) => directives.other.push(directive.to_owned()),
            },
            Some((target, level)) => match level.parse::<Level>() {
                Ok(level) if valid_target(target) => {
                    directives.loggers.retain(|(name, _)| name != target);
                    directives.loggers.push((target.to_owned(), level));
                }
                _ => directives.other.push(directive.to_owned()),
            },
        }
    }
    directives
}

pub fn compose(base: &Directives, overrides: &BTreeMap<String, Level>) -> String {
    let mut parts = Vec::new();
    if let Some(root) = overrides.get(ROOT).copied().or(base.root) {
        parts.push(root.directive().to_owned());
    }
    for (name, level) in &base.loggers {
        if !overrides.contains_key(name) {
            parts.push(format!("{name}={}", level.directive()));
        }
    }
    parts.extend(base.other.iter().cloned());
    for (name, level) in overrides {
        if name != ROOT {
            parts.push(format!("{name}={}", level.directive()));
        }
    }
    parts.join(",")
}

pub fn valid_name(name: &str) -> bool {
    name == ROOT || valid_target(name)
}

fn valid_target(target: &str) -> bool {
    !target.is_empty()
        && target.len() <= MAX_NAME
        && target.split("::").all(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_the_configured_filter() {
        let parsed = parse("info, sqlx=warn ,astrum_api::routes=debug,tower_http[span]=trace");
        assert_eq!(parsed.root, Some(Level::Info));
        assert_eq!(
            parsed.loggers,
            vec![
                ("sqlx".to_owned(), Level::Warn),
                ("astrum_api::routes".to_owned(), Level::Debug)
            ]
        );
        assert_eq!(parsed.other, vec!["tower_http[span]=trace".to_owned()]);
    }

    #[test]
    fn overrides_replace_configured_levels() {
        let base = parse("info,sqlx=warn");
        let overrides = BTreeMap::from([
            (ROOT.to_owned(), Level::Debug),
            ("sqlx".to_owned(), Level::Error),
            ("astrum_content".to_owned(), Level::Trace),
        ]);
        assert_eq!(
            compose(&base, &overrides),
            "debug,astrum_content=trace,sqlx=error"
        );
        assert_eq!(compose(&base, &BTreeMap::new()), "info,sqlx=warn");
    }

    #[test]
    fn accepts_module_paths_only() {
        assert!(valid_name("ROOT"));
        assert!(valid_name("astrum_api::routes::status"));
        assert!(!valid_name("astrum-api"));
        assert!(!valid_name("a=b"));
        assert!(!valid_name("::a"));
        assert!(!valid_name(""));
        assert!(!valid_name("1abc"));
    }
}
