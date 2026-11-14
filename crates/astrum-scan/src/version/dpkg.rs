use std::cmp::Ordering;

use super::{cmp_digits, digits};

struct Version<'a> {
    epoch: &'a str,
    upstream: &'a str,
    revision: &'a str,
}

pub(super) fn compare(left: &str, right: &str) -> Option<Ordering> {
    let left = parse(left)?;
    let right = parse(right)?;
    Some(
        cmp_digits(left.epoch, right.epoch)
            .then_with(|| verrevcmp(left.upstream.as_bytes(), right.upstream.as_bytes()))
            .then_with(|| verrevcmp(left.revision.as_bytes(), right.revision.as_bytes())),
    )
}

fn parse(text: &str) -> Option<Version<'_>> {
    let (epoch, rest) = match text.split_once(':') {
        Some((epoch, rest)) if digits(epoch) => (epoch, rest),
        Some(_) => return None,
        None => ("0", text),
    };
    let (upstream, revision) = match rest.rsplit_once('-') {
        Some((upstream, revision)) if !revision.is_empty() => (upstream, revision),
        Some(_) => return None,
        None => (rest, ""),
    };
    let upstream_ok = !upstream.is_empty()
        && upstream.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'~' | b'-' | b':')
        });
    let revision_ok = revision
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'~'));
    (upstream_ok && revision_ok).then_some(Version {
        epoch,
        upstream,
        revision,
    })
}

fn order(byte: Option<u8>) -> i32 {
    match byte {
        None => 0,
        Some(byte) if byte.is_ascii_digit() => 0,
        Some(byte) if byte.is_ascii_alphabetic() => i32::from(byte),
        Some(b'~') => -1,
        Some(byte) => i32::from(byte) + 256,
    }
}

fn verrevcmp(left: &[u8], right: &[u8]) -> Ordering {
    let (mut i, mut j) = (0, 0);
    while i < left.len() || j < right.len() {
        let mut first_diff = 0;
        while (i < left.len() && !left[i].is_ascii_digit())
            || (j < right.len() && !right[j].is_ascii_digit())
        {
            let a = order(left.get(i).copied());
            let b = order(right.get(j).copied());
            if a != b {
                return a.cmp(&b);
            }
            i = (i + 1).min(left.len());
            j = (j + 1).min(right.len());
        }
        while i < left.len() && left[i] == b'0' {
            i += 1;
        }
        while j < right.len() && right[j] == b'0' {
            j += 1;
        }
        while i < left.len()
            && j < right.len()
            && left[i].is_ascii_digit()
            && right[j].is_ascii_digit()
        {
            if first_diff == 0 {
                first_diff = i32::from(left[i]) - i32::from(right[j]);
            }
            i += 1;
            j += 1;
        }
        if i < left.len() && left[i].is_ascii_digit() {
            return Ordering::Greater;
        }
        if j < right.len() && right[j].is_ascii_digit() {
            return Ordering::Less;
        }
        if first_diff != 0 {
            return first_diff.cmp(&0);
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use crate::version::{Ecosystem, assert_chain, assert_same, compare};

    #[test]
    fn policy_examples_and_tilde() {
        for ecosystem in [Ecosystem::Debian, Ecosystem::Ubuntu] {
            assert_chain(
                ecosystem,
                &[
                    "1.0~~", "1.0~~a", "1.0~", "1.0", "1.0a", "1.0+", "1.0.1", "1.1", "2.0",
                    "10.0", "1:0.5",
                ],
            );
            assert_chain(ecosystem, &["1.0~rc1", "1.0"]);
            assert_chain(ecosystem, &["1.0+dfsg", "1.0.1"]);
        }
    }

    #[test]
    fn epochs_and_revisions() {
        assert_chain(
            Ecosystem::Debian,
            &[
                "1.0-1~bpo1",
                "1.0-1",
                "1.0-1ubuntu1",
                "1.0-2",
                "1.0-10",
                "2:0.1-1",
                "2:0.1-2",
            ],
        );
        assert_chain(Ecosystem::Ubuntu, &["1.0-1", "1.0-2"]);
    }

    #[test]
    fn empty_revision_equals_zero() {
        assert_same(
            Ecosystem::Debian,
            &[("1.0", "1.0-0"), ("0:1.0", "1.0"), ("1.0-1", "1.0-01")],
        );
    }

    #[test]
    fn malformed_versions_are_unparseable() {
        for bad in [
            "", ":1.0", "x:1.0", "1.0-", "-1", "1:", "1.0_1", "1.0-1_2", "1.0 1",
        ] {
            assert_eq!(compare(Ecosystem::Debian, bad, "1.0"), None, "{bad}");
        }
    }
}
