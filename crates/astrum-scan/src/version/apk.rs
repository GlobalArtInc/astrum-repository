use std::cmp::Ordering;

use super::{cmp_digits, digits};

const RELEASE: u8 = 4;

struct Version<'a> {
    numbers: Vec<&'a str>,
    letter: Option<char>,
    suffixes: Vec<(u8, &'a str)>,
    revision: &'a str,
}

pub(super) fn compare(left: &str, right: &str) -> Option<Ordering> {
    let left = parse(left)?;
    let right = parse(right)?;
    Some(
        cmp_numbers(&left.numbers, &right.numbers)
            .then_with(|| left.letter.cmp(&right.letter))
            .then_with(|| cmp_suffixes(&left.suffixes, &right.suffixes))
            .then_with(|| cmp_digits(left.revision, right.revision)),
    )
}

fn parse(text: &str) -> Option<Version<'_>> {
    let (body, revision) = match text.rsplit_once("-r") {
        Some((body, revision)) if digits(revision) => (body, revision),
        _ => (text, "0"),
    };
    let mut parts = body.split('_');
    let main = parts.next()?;
    let (numeric, letter) = match main.as_bytes().last() {
        Some(byte) if byte.is_ascii_lowercase() => {
            (&main[..main.len() - 1], Some(char::from(*byte)))
        }
        _ => (main, None),
    };
    let numbers: Vec<&str> = numeric.split('.').collect();
    if !numbers.iter().all(|number| digits(number)) {
        return None;
    }
    let mut suffixes = Vec::new();
    for part in parts {
        let end = part
            .find(|c: char| c.is_ascii_digit())
            .unwrap_or(part.len());
        let (name, number) = part.split_at(end);
        if !number.is_empty() && !digits(number) {
            return None;
        }
        let number = if number.is_empty() { "0" } else { number };
        suffixes.push((rank(name)?, number));
    }
    Some(Version {
        numbers,
        letter,
        suffixes,
        revision,
    })
}

fn rank(name: &str) -> Option<u8> {
    match name {
        "alpha" => Some(0),
        "beta" => Some(1),
        "pre" => Some(2),
        "rc" => Some(3),
        "cvs" => Some(5),
        "svn" => Some(6),
        "git" => Some(7),
        "hg" => Some(8),
        "p" => Some(9),
        _ => None,
    }
}

fn cmp_numbers(left: &[&str], right: &[&str]) -> Ordering {
    for (a, b) in left.iter().zip(right) {
        let ordering = cmp_digits(a, b);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

fn cmp_suffixes(left: &[(u8, &str)], right: &[(u8, &str)]) -> Ordering {
    for index in 0..left.len().max(right.len()) {
        let (left_rank, left_number) = left.get(index).copied().unwrap_or((RELEASE, "0"));
        let (right_rank, right_number) = right.get(index).copied().unwrap_or((RELEASE, "0"));
        let ordering = left_rank
            .cmp(&right_rank)
            .then_with(|| cmp_digits(left_number, right_number));
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use crate::version::{Ecosystem, assert_chain, assert_same, compare};

    #[test]
    fn suffix_order_from_alpha_to_p() {
        assert_chain(
            Ecosystem::Alpine,
            &[
                "1.0_alpha1",
                "1.0_alpha2",
                "1.0_beta1",
                "1.0_pre1",
                "1.0_rc1",
                "1.0_rc2",
                "1.0",
                "1.0_cvs1",
                "1.0_svn1",
                "1.0_git1",
                "1.0_hg1",
                "1.0_p1",
                "1.0_p2",
            ],
        );
    }

    #[test]
    fn numbers_letters_and_revisions() {
        assert_chain(Ecosystem::Alpine, &["1.0", "1.0a", "1.0b", "1.1"]);
        assert_chain(Ecosystem::Alpine, &["1.9", "1.10", "2.0"]);
        assert_chain(
            Ecosystem::Alpine,
            &["1.0", "1.0-r1", "1.0-r2", "1.0-r10", "1.0_p1-r1"],
        );
        assert_chain(Ecosystem::Alpine, &["1.0-r9", "1.0_p1"]);
        assert_chain(Ecosystem::Alpine, &["1.0_rc1", "1.0_rc1-r1", "1.0_rc2"]);
    }

    #[test]
    fn missing_revision_and_zero_revision_are_equal() {
        assert_same(
            Ecosystem::Alpine,
            &[
                ("1.0", "1.0-r0"),
                ("1.2.3", "1.2.3-r0"),
                ("1.0_p1", "1.0_p1-r0"),
            ],
        );
    }

    #[test]
    fn malformed_versions_are_unparseable() {
        for bad in [
            "", "a", "1.0-", "1.0-r", "1.0_", "1.0_x", "1.0_rc1x", "1.0_1", "1..0", "1.0A", "1.0 1",
        ] {
            assert_eq!(compare(Ecosystem::Alpine, bad, "1.0"), None, "{bad}");
        }
    }
}
