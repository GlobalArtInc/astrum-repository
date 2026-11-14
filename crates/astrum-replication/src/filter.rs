use regex::{Regex, RegexBuilder};

pub const MAX_FILTER_BYTES: usize = 10 * 1024;
pub const FILTER_SIZE_LIMIT: usize = 1 << 20;

fn build(pattern: &str) -> Result<Regex, String> {
    RegexBuilder::new(pattern)
        .size_limit(FILTER_SIZE_LIMIT)
        .build()
        .map_err(|error| match error {
            regex::Error::Syntax(text) => format!(
                "is not a valid regular expression: {}",
                text.lines().last().unwrap_or("syntax error").trim()
            ),
            regex::Error::CompiledTooBig(_) => "is too complex".to_owned(),
            _ => "is not a valid regular expression".to_owned(),
        })
}

pub fn compile(pattern: &str) -> Result<Regex, String> {
    if pattern.len() > MAX_FILTER_BYTES {
        return Err(format!("must be at most {MAX_FILTER_BYTES} bytes"));
    }
    build(pattern)?;
    build(&format!(r"\A(?:{pattern})\z"))
}

pub fn subject(namespace: &str, name: &str) -> String {
    let doubled = namespace == "/"
        || name
            .strip_prefix(namespace)
            .is_some_and(|rest| rest.starts_with('/'));
    match namespace.is_empty() || doubled {
        true => name.to_owned(),
        false => format!("{namespace}/{name}"),
    }
}

pub fn component_name(namespace: &str, name: &str) -> String {
    subject(namespace, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_subject_never_repeats_the_namespace() {
        assert_eq!(subject("/docs", "/docs/a.txt"), "/docs/a.txt");
        assert_eq!(subject("/", "/top.txt"), "/top.txt");
        assert_eq!(subject("com.acme", "lib"), "com.acme/lib");
        assert_eq!(subject("acme", "ui"), "acme/ui");
        assert_eq!(subject("acme", "acme-ui"), "acme/acme-ui");
        assert_eq!(subject("", "app"), "app");
        assert_eq!(subject("/docs", "/docsx/a.txt"), "/docs//docsx/a.txt");
    }

    #[test]
    fn a_filter_matches_the_whole_subject() {
        let wanted = compile("/docs/.*\\.txt").unwrap();
        assert!(wanted.is_match(&subject("/docs", "/docs/a.txt")));
        assert!(!wanted.is_match(&subject("/other", "/other/a.txt")));
        let group = compile("com\\.acme/lib-.*").unwrap();
        assert!(group.is_match(&subject("com.acme", "lib-core")));
        assert!(!group.is_match(&subject("org.acme", "lib-core")));
    }
}
