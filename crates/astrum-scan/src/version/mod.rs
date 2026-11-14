mod apk;
mod dpkg;
mod maven;
mod nuget;
mod pep440;
mod range;
mod semver;

use std::cmp::Ordering;

pub use range::{Event, Range, RangeKind, Verdict, affected, affected_detailed};

pub const MAX_LEN: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ecosystem {
    Maven,
    Npm,
    PyPi,
    Go,
    CratesIo,
    NuGet,
    Debian,
    Ubuntu,
    Alpine,
}

impl Ecosystem {
    pub fn from_osv(name: &str) -> Option<Ecosystem> {
        match name {
            "Maven" => Some(Ecosystem::Maven),
            "npm" => Some(Ecosystem::Npm),
            "PyPI" => Some(Ecosystem::PyPi),
            "Go" => Some(Ecosystem::Go),
            "crates.io" => Some(Ecosystem::CratesIo),
            "NuGet" => Some(Ecosystem::NuGet),
            _ => match name.split_once(':') {
                Some(("Debian", release)) if !release.is_empty() => Some(Ecosystem::Debian),
                Some(("Ubuntu", release)) if !release.is_empty() => Some(Ecosystem::Ubuntu),
                Some(("Alpine", release)) if !release.is_empty() => Some(Ecosystem::Alpine),
                _ => None,
            },
        }
    }
}

pub fn compare(ecosystem: Ecosystem, left: &str, right: &str) -> Option<Ordering> {
    if !within_limit(left) || !within_limit(right) {
        return None;
    }
    match ecosystem {
        Ecosystem::Maven => maven::compare(left, right),
        Ecosystem::Npm | Ecosystem::CratesIo => semver::compare(left, right),
        Ecosystem::Go => semver::compare_go(left, right),
        Ecosystem::PyPi => pep440::compare(left, right),
        Ecosystem::NuGet => nuget::compare(left, right),
        Ecosystem::Debian | Ecosystem::Ubuntu => dpkg::compare(left, right),
        Ecosystem::Alpine => apk::compare(left, right),
    }
}

fn within_limit(text: &str) -> bool {
    !text.is_empty() && text.len() <= MAX_LEN
}

fn digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn cmp_digits(left: &str, right: &str) -> Ordering {
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

#[cfg(test)]
fn assert_chain(ecosystem: Ecosystem, chain: &[&str]) {
    for (index, low) in chain.iter().enumerate() {
        for high in &chain[index + 1..] {
            assert_eq!(
                compare(ecosystem, low, high),
                Some(Ordering::Less),
                "{ecosystem:?}: {low} < {high}"
            );
            assert_eq!(
                compare(ecosystem, high, low),
                Some(Ordering::Greater),
                "{ecosystem:?}: {high} > {low}"
            );
        }
    }
}

#[cfg(test)]
fn assert_same(ecosystem: Ecosystem, pairs: &[(&str, &str)]) {
    for (left, right) in pairs {
        assert_eq!(
            compare(ecosystem, left, right),
            Some(Ordering::Equal),
            "{ecosystem:?}: {left} = {right}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Ecosystem; 9] = [
        Ecosystem::Maven,
        Ecosystem::Npm,
        Ecosystem::PyPi,
        Ecosystem::Go,
        Ecosystem::CratesIo,
        Ecosystem::NuGet,
        Ecosystem::Debian,
        Ecosystem::Ubuntu,
        Ecosystem::Alpine,
    ];

    #[test]
    fn osv_names_map_to_ecosystems() {
        for (name, ecosystem) in [
            ("Maven", Ecosystem::Maven),
            ("npm", Ecosystem::Npm),
            ("PyPI", Ecosystem::PyPi),
            ("Go", Ecosystem::Go),
            ("crates.io", Ecosystem::CratesIo),
            ("NuGet", Ecosystem::NuGet),
            ("Debian:11", Ecosystem::Debian),
            ("Ubuntu:22.04:LTS", Ecosystem::Ubuntu),
            ("Alpine:v3.18", Ecosystem::Alpine),
        ] {
            assert_eq!(Ecosystem::from_osv(name), Some(ecosystem), "{name}");
        }
        for name in ["", "pypi", "Debian", "Debian:", "Rocky:9", "Packagist"] {
            assert_eq!(Ecosystem::from_osv(name), None, "{name}");
        }
    }

    #[test]
    fn empty_and_oversized_inputs_are_unparseable() {
        let long = "1".repeat(MAX_LEN + 1);
        for ecosystem in ALL {
            assert_eq!(compare(ecosystem, "", "1"), None, "{ecosystem:?}");
            assert_eq!(compare(ecosystem, "1", ""), None, "{ecosystem:?}");
            assert_eq!(compare(ecosystem, &long, "1"), None, "{ecosystem:?}");
        }
    }

    #[test]
    fn length_limit_is_exactly_256_bytes() {
        let accepted = format!("1.0.0-{}", "a".repeat(MAX_LEN - 6));
        let rejected = format!("1.0.0-{}", "a".repeat(MAX_LEN - 5));
        assert_eq!(accepted.len(), MAX_LEN);
        assert_eq!(rejected.len(), MAX_LEN + 1);
        assert_eq!(
            compare(Ecosystem::Npm, &accepted, "1.0.0"),
            Some(Ordering::Less)
        );
        assert_eq!(compare(Ecosystem::Npm, &rejected, "1.0.0"), None);
        let accepted = "1".repeat(MAX_LEN);
        assert_eq!(
            compare(Ecosystem::Maven, &accepted, "1"),
            Some(Ordering::Greater)
        );
        assert_eq!(
            compare(Ecosystem::Maven, &format!("{accepted}1"), "1"),
            None
        );
    }

    #[test]
    fn huge_numbers_compare_without_overflow() {
        let big = "99999999999999999999999999";
        for (ecosystem, left, right) in [
            (Ecosystem::Npm, format!("{big}.0.0"), "1.0.0"),
            (Ecosystem::CratesIo, format!("{big}.0.0"), "1.0.0"),
            (Ecosystem::Go, format!("v{big}.0.0"), "v1.0.0"),
            (Ecosystem::PyPi, format!("{big}.0"), "1.0"),
            (Ecosystem::Maven, format!("{big}.0"), "1.0"),
            (Ecosystem::Debian, format!("{big}.0"), "1.0"),
            (Ecosystem::Alpine, format!("{big}.0"), "1.0"),
        ] {
            assert_eq!(
                compare(ecosystem, &left, right),
                Some(Ordering::Greater),
                "{ecosystem:?}: {left}"
            );
        }
    }

    #[test]
    fn odd_inputs_never_panic() {
        let long = "1.".repeat(200);
        let odd = [
            "",
            " ",
            "0",
            "~",
            "-",
            "+",
            ".",
            "!",
            ":",
            "_",
            "v",
            "-r",
            "é",
            "1é",
            "日本",
            "\u{0}",
            "1.0\n",
            "1!2!3",
            "1:2:3",
            "1..2",
            "--1",
            "1+",
            "+1",
            "0000",
            "1.0.0-",
            "1.0_",
            long.as_str(),
        ];
        for ecosystem in ALL {
            for left in odd {
                for right in odd {
                    let events = [Event::Introduced("0"), Event::Fixed(right)];
                    let ranges = [Range {
                        kind: RangeKind::Ecosystem,
                        events: &events,
                    }];
                    let _ = compare(ecosystem, left, right);
                    let _ = affected(ecosystem, left, &ranges, &[right]);
                }
            }
        }
    }
}
