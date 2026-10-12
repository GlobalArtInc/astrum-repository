use std::net::{IpAddr, Ipv6Addr};

const ATOM_SYMBOLS: &str = "!#$%&'*+/=?^_`{|}~-";

pub fn identifier_problem(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        Some("must not be empty")
    } else if value.trim() != value {
        Some("must not start or end with spaces")
    } else if value.chars().any(char::is_control) {
        Some("must not contain control characters")
    } else if value.contains(':') {
        Some("must not contain ':'")
    } else {
        None
    }
}

pub fn email(value: &str) -> bool {
    let Some((local, domain)) = value.rsplit_once('@') else {
        return false;
    };
    local_part(local) && domain_part(domain)
}

fn local_part(local: &str) -> bool {
    if local.is_empty() || local.chars().count() > 64 {
        return false;
    }
    if let Some(quoted) = local
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return quoted
            .chars()
            .all(|c| c != '"' && c != '\\' && !c.is_control());
    }
    local.split('.').all(|atom| {
        !atom.is_empty()
            && atom.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || ATOM_SYMBOLS.contains(c)
                    || (!c.is_ascii() && !c.is_control() && !c.is_whitespace())
            })
    })
}

fn domain_part(domain: &str) -> bool {
    if let Some(literal) = domain
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        return literal.parse::<IpAddr>().is_ok()
            || literal
                .strip_prefix("IPv6:")
                .is_some_and(|v6| v6.parse::<Ipv6Addr>().is_ok());
    }
    !domain.is_empty()
        && domain.len() <= 255
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.chars().count() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emails_like_the_reference_server() {
        for good in [
            "a@b",
            "a@b.c",
            "a@b_c.d",
            "a+tag@b.c",
            "a@[1.2.3.4]",
            "\"a b\"@c.d",
            "ü@b.c",
            "a@ü.c",
            "a@1.2.3.4",
            "a!#$%&'*+/=?^_`{|}~@b.c",
            "first.last@example.org",
        ] {
            assert!(email(good), "{good}");
        }
        for bad in [
            "a b@c.d", "a@", "@b", "a@b..c", "a.@b.c", ".a@b.c", "a..b@c.d", "a@-b.c", "a@b.c ",
            "a@b.c.", "a@b-.c", "plain", "",
        ] {
            assert!(!email(bad), "{bad}");
        }
    }

    #[test]
    fn identifiers_reject_what_breaks_sign_in() {
        assert_eq!(identifier_problem("jane.doe@corp"), None);
        assert_eq!(identifier_problem("jane doe"), None);
        assert!(identifier_problem(" jane").is_some());
        assert!(identifier_problem("jane:doe").is_some());
        assert!(identifier_problem("jane\tdoe").is_some());
        assert!(identifier_problem("  ").is_some());
    }
}
