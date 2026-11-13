use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;

use astrum_security::ssrf::{Guard, SsrfProtection};
use regex::{Regex, RegexBuilder};
use url::Url;

pub const MAX_NAME: usize = 100;
pub const MAX_URL: usize = 2048;
pub const MAX_TOKEN: usize = 1024;
pub const MAX_FILTER_BYTES: usize = 10 * 1024;
pub const FILTER_SIZE_LIMIT: usize = 1 << 20;
pub const MAX_REMOTE_REPOSITORY: usize = astrum_repo::routing::MAX_NAME;

const LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);
const METADATA_V6: Ipv6Addr = Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254);

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

fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(v6)),
        v4 => v4,
    }
}

fn private_range(ip: IpAddr) -> bool {
    match canonical(ip) {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            v4.is_private() || (a == 100 && (64..128).contains(&b))
        }
        IpAddr::V6(v6) => (v6.segments()[0] & 0xfe00) == 0xfc00 && v6 != METADATA_V6,
    }
}

pub fn address_permitted(
    guard: &Guard,
    ip: IpAddr,
    host_allowed: bool,
    allow_private_network: bool,
) -> bool {
    guard.ip_allowed(ip, host_allowed) || (allow_private_network && private_range(ip))
}

pub async fn refused_destination(
    settings: &SsrfProtection,
    url: &Url,
    allow_private_network: bool,
) -> Option<String> {
    if !allow_private_network {
        return astrum_security::ssrf::refused_destination(settings, url.as_str()).await;
    }
    let guard = Guard::new(settings);
    let (addresses, host_allowed): (Vec<IpAddr>, bool) = match url.host()? {
        url::Host::Ipv4(ip) => (vec![IpAddr::V4(ip)], false),
        url::Host::Ipv6(ip) => (vec![IpAddr::V6(ip)], false),
        url::Host::Domain(domain) => {
            let port = url.port_or_known_default().unwrap_or(80);
            let found =
                tokio::time::timeout(LOOKUP_TIMEOUT, tokio::net::lookup_host((domain, port)))
                    .await
                    .ok()?
                    .ok()?;
            (
                found.map(|found| found.ip()).collect(),
                guard.host_allowed(domain),
            )
        }
    };
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|ip| address_permitted(&guard, *ip, host_allowed, true))
    {
        return None;
    }
    let mut shown: Vec<String> = addresses
        .iter()
        .map(|ip| canonical(*ip).to_string())
        .collect();
    shown.sort();
    shown.dedup();
    Some(format!(
        "Host resolves only to addresses a remote may not use even with private networks allowed (loopback, link-local, unspecified, multicast, broadcast): {}",
        shown.join(", ")
    ))
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

    #[test]
    fn private_ranges_open_only_with_the_flag() {
        let guard = Guard::new(&SsrfProtection::default());
        let ip = |text: &str| text.parse::<IpAddr>().unwrap();
        for private in [
            "10.1.2.3",
            "192.168.0.9",
            "172.16.0.1",
            "100.64.0.1",
            "fd12::1",
        ] {
            assert!(
                !address_permitted(&guard, ip(private), false, false),
                "{private}"
            );
            assert!(
                address_permitted(&guard, ip(private), false, true),
                "{private}"
            );
        }
        for refused in [
            "127.0.0.1",
            "::1",
            "169.254.1.1",
            "169.254.169.254",
            "fe80::1",
            "fd00:ec2::254",
            "0.0.0.0",
            "224.0.0.1",
            "255.255.255.255",
            "::ffff:127.0.0.1",
        ] {
            assert!(
                !address_permitted(&guard, ip(refused), false, true),
                "{refused}"
            );
        }
        assert!(address_permitted(&guard, ip("8.8.8.8"), false, false));
    }
}
