use std::cmp::Ordering;

use super::{cmp_digits, digits};

struct Semver<'a> {
    core: [&'a str; 3],
    pre: Vec<&'a str>,
}

pub(super) fn compare(left: &str, right: &str) -> Option<Ordering> {
    let left = parse(left)?;
    let right = parse(right)?;
    for (a, b) in left.core.iter().zip(&right.core) {
        let ordering = cmp_digits(a, b);
        if ordering != Ordering::Equal {
            return Some(ordering);
        }
    }
    Some(cmp_pre(&left.pre, &right.pre))
}

pub(super) fn compare_go(left: &str, right: &str) -> Option<Ordering> {
    compare(strip_optional_v(left), strip_optional_v(right))
}

fn strip_optional_v(text: &str) -> &str {
    text.strip_prefix('v').unwrap_or(text)
}

fn parse(text: &str) -> Option<Semver<'_>> {
    let rest = match text.split_once('+') {
        Some((rest, build)) if identifiers(build) => rest,
        Some(_) => return None,
        None => text,
    };
    let (core, pre): (&str, Vec<&str>) = match rest.split_once('-') {
        Some((core, pre)) if prerelease(pre) => (core, pre.split('.').collect()),
        Some(_) => return None,
        None => (rest, Vec::new()),
    };
    let mut parts = core.split('.');
    let core = [
        number(parts.next()?)?,
        number(parts.next()?)?,
        number(parts.next()?)?,
    ];
    if parts.next().is_some() {
        return None;
    }
    Some(Semver { core, pre })
}

fn identifiers(text: &str) -> bool {
    text.split('.').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

fn prerelease(text: &str) -> bool {
    identifiers(text)
        && text
            .split('.')
            .all(|part| !(digits(part) && part.len() > 1 && part.starts_with('0')))
}

fn number(text: &str) -> Option<&str> {
    (digits(text) && (text == "0" || !text.starts_with('0'))).then_some(text)
}

fn cmp_pre(left: &[&str], right: &[&str]) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => {
            for (a, b) in left.iter().zip(right) {
                let ordering = cmp_identifier(a, b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            left.len().cmp(&right.len())
        }
    }
}

fn cmp_identifier(left: &str, right: &str) -> Ordering {
    match (digits(left), digits(right)) {
        (true, true) => cmp_digits(left, right),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => left.cmp(right),
    }
}

#[cfg(test)]
mod tests {
    use crate::version::{Ecosystem, assert_chain, assert_same, compare};

    #[test]
    fn precedence_follows_semver_org() {
        assert_chain(
            Ecosystem::Npm,
            &[
                "1.0.0-alpha",
                "1.0.0-alpha.1",
                "1.0.0-alpha.beta",
                "1.0.0-beta",
                "1.0.0-beta.2",
                "1.0.0-beta.11",
                "1.0.0-rc.1",
                "1.0.0",
                "2.0.0",
                "2.1.0",
                "2.1.1",
            ],
        );
        assert_chain(
            Ecosystem::Npm,
            &[
                "1.0.0-1",
                "1.0.0-9",
                "1.0.0-10",
                "1.0.0-a",
                "1.0.0-a.b",
                "1.0.0-b",
                "1.0.0",
            ],
        );
        assert_chain(
            Ecosystem::CratesIo,
            &[
                "0.0.1", "0.1.0", "0.9.99", "1.0.0-0", "1.0.0", "1.0.1", "1.9.0", "1.10.0", "2.0.0",
            ],
        );
    }

    #[test]
    fn build_metadata_is_ignored() {
        assert_same(
            Ecosystem::Npm,
            &[
                ("1.0.0+20130313144700", "1.0.0"),
                ("1.0.0-beta+exp.sha.5114f85", "1.0.0-beta"),
                ("1.0.0+a", "1.0.0+b"),
            ],
        );
    }

    #[test]
    fn go_versions_take_a_v_and_pseudo_versions() {
        assert_chain(
            Ecosystem::Go,
            &[
                "v0.0.0-20191109021931-daf7d2dfca0c",
                "v1.0.0-alpha",
                "v1.0.0-alpha.1",
                "v1.0.0-alpha.beta",
                "v1.0.0-beta",
                "v1.0.0-beta.2",
                "v1.0.0-beta.11",
                "v1.0.0-rc.1",
                "v1.0.0",
                "v1.2.3-0.20191109021931-daf7d2dfca0c",
                "v1.2.3",
                "v1.9.0",
                "v1.10.0",
                "v2.0.0+incompatible",
            ],
        );
        assert_same(
            Ecosystem::Go,
            &[
                ("v2.0.0+incompatible", "v2.0.0"),
                ("v1.0.0+build.1", "v1.0.0"),
            ],
        );
        assert_same(Ecosystem::Go, &[("1.2.3", "v1.2.3"), ("v1.2.3", "1.2.3")]);
        for bad in ["v1.2", "v01.2.3", "vv1.2.3", "v"] {
            assert_eq!(compare(Ecosystem::Go, bad, "v1.0.0"), None, "{bad}");
        }
    }

    #[test]
    fn malformed_semver_is_unparseable() {
        for bad in [
            "1",
            "1.0",
            "1.0.0.0",
            "01.0.0",
            "1.0.00",
            "a.b.c",
            "1.0.0-",
            "1.0.0-01",
            "1.0.0-a..b",
            "1.0.0+",
            "1.0.0+a..b",
            "1.0.0-a+b+c",
            "v1.0.0",
            " 1.0.0",
        ] {
            assert_eq!(compare(Ecosystem::Npm, bad, "1.0.0"), None, "{bad}");
        }
    }
}
