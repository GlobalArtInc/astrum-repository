use std::net::IpAddr;

use astrum_security::ssrf::cleartext_permitted;
use regex::{Regex, RegexBuilder};
use url::{Host, Url};

pub const MAX_NAME: usize = 100;
pub const MAX_URL: usize = 2048;
pub const MAX_TOKEN: usize = 1024;
pub const MAX_FILTER_BYTES: usize = 10 * 1024;
pub const FILTER_SIZE_LIMIT: usize = 1 << 20;
pub const MAX_REMOTE_REPOSITORY: usize = astrum_repo::routing::MAX_NAME;

pub fn name(value: Option<&str>) -> Result<String, String> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if value.chars().count() > MAX_NAME {
        return Err(format!("must be at most {MAX_NAME} characters"));
    }
    if value.chars().any(char::is_control) {
        return Err("must not contain control characters".to_owned());
    }
    Ok(value.to_owned())
}

pub fn token(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if value.len() > MAX_TOKEN {
        return Err(format!("must be at most {MAX_TOKEN} characters"));
    }
    if value
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || !c.is_ascii())
    {
        return Err("must not contain spaces, control or non-ASCII characters".to_owned());
    }
    Ok(())
}

pub fn remote_url(value: Option<&str>, allow_private_network: bool) -> Result<Url, String> {
    const SHAPE: &str = "must be an address such as https://astrum.example.com:8443, with no path, user name, query or fragment";
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if value.len() > MAX_URL {
        return Err(format!("must be at most {MAX_URL} characters"));
    }
    if value
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '@' | '\\' | '?' | '#'))
    {
        return Err(SHAPE.to_owned());
    }
    let url = Url::parse(value).map_err(|_| SHAPE.to_owned())?;
    match url.scheme() {
        "https" => {}
        "http" if allow_private_network => {}
        "http" => {
            return Err("must use https unless private network addresses are allowed".to_owned());
        }
        _ => return Err("must be an https:// address".to_owned()),
    }
    let literal = match url.host() {
        Some(Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
        _ => None,
    };
    if url.scheme() == "http" && literal.is_some_and(|ip| !cleartext_permitted(ip)) {
        return Err("an http:// address must name a private network address, so that the token never crosses the internet in clear; use https for other hosts".to_owned());
    }
    if url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(SHAPE.to_owned());
    }
    Ok(url)
}

pub fn normalized(url: &Url) -> String {
    let port = url
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
    format!(
        "{}://{}{port}",
        url.scheme(),
        url.host_str().unwrap_or_default()
    )
}

pub fn remote_repository(value: Option<&str>) -> Result<String, String> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if value.len() > MAX_REMOTE_REPOSITORY {
        return Err(format!(
            "must be at most {MAX_REMOTE_REPOSITORY} characters"
        ));
    }
    if !astrum_repo::routing::valid_name(value) {
        return Err("may hold only letters, digits, '-', '_' and '.', and must start with a letter, a digit or '-'".to_owned());
    }
    if astrum_repo::store::reserved_name(value) {
        return Err(format!(
            "\"{value}\" is reserved for the replication protocol and cannot name a repository"
        ));
    }
    Ok(value.to_owned())
}

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

pub fn filter(pattern: &str) -> Result<Regex, String> {
    if pattern.len() > MAX_FILTER_BYTES {
        return Err(format!("must be at most {MAX_FILTER_BYTES} bytes"));
    }
    build(pattern)?;
    build(&format!(r"\A(?:{pattern})\z"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_remote_repository_cannot_carry_a_reserved_name() {
        for reserved in ["remotes", "Remotes", "CAPABILITIES"] {
            assert!(
                remote_repository(Some(reserved))
                    .unwrap_err()
                    .contains("reserved"),
                "{reserved}"
            );
        }
        assert_eq!(remote_repository(Some("remotes-2")).unwrap(), "remotes-2");
    }

    #[test]
    fn clear_text_needs_a_private_address() {
        for public in ["http://203.0.113.7:8081", "http://[2001:db8::1]:8081"] {
            let error = remote_url(Some(public), true).unwrap_err();
            assert!(error.contains("private network"), "{public}: {error}");
        }
        for private in [
            "http://10.0.0.5:8081",
            "http://192.168.1.2",
            "http://[fd12::1]:80",
            "http://repo.internal:8081",
        ] {
            assert!(remote_url(Some(private), true).is_ok(), "{private}");
        }
        assert!(remote_url(Some("https://203.0.113.7"), false).is_ok());
    }

    #[test]
    fn a_filter_must_match_the_whole_text() {
        let regex = filter("lib-.*").unwrap();
        assert!(regex.is_match("lib-core"));
        assert!(!regex.is_match("my-lib-core"));
        let alternatives = filter("a|b").unwrap();
        assert!(alternatives.is_match("b"));
        assert!(!alternatives.is_match("bc"));
    }

    #[test]
    fn a_pathological_filter_is_refused_as_too_complex() {
        assert!(
            filter("(a{1000}){1000}")
                .unwrap_err()
                .contains("too complex")
        );
        assert!(filter(&"a".repeat(MAX_FILTER_BYTES + 1)).is_err());
    }

    #[test]
    fn a_filter_cannot_break_out_of_its_group() {
        assert!(filter("a)|(b").is_err());
    }
}
