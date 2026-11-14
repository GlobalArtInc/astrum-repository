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

pub fn component_name(namespace: &str, name: &str) -> String {
    match namespace.is_empty() {
        true => name.to_owned(),
        false => format!("{namespace}/{name}"),
    }
}
