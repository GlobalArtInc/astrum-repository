use std::cmp::Ordering;

use super::{cmp_digits, digits};

const PRE: [(&str, u8); 8] = [
    ("alpha", 0),
    ("a", 0),
    ("beta", 1),
    ("b", 1),
    ("preview", 2),
    ("pre", 2),
    ("rc", 2),
    ("c", 2),
];
const POST: [(&str, u8); 3] = [("post", 0), ("rev", 0), ("r", 0)];
const DEV: [(&str, u8); 1] = [("dev", 0)];

struct Version {
    epoch: String,
    release: Vec<String>,
    pre: Option<(u8, String)>,
    post: Option<String>,
    dev: Option<String>,
    local: Option<Vec<String>>,
}

pub(super) fn compare(left: &str, right: &str) -> Option<Ordering> {
    let left = parse(left)?;
    let right = parse(right)?;
    Some(order(&left, &right))
}

fn parse(text: &str) -> Option<Version> {
    let lowered = text.trim().to_ascii_lowercase();
    let text = lowered.strip_prefix('v').unwrap_or(&lowered);
    let (public, local) = match text.split_once('+') {
        Some((public, local)) => (public, Some(segments(local)?)),
        None => (text, None),
    };
    let (epoch, rest) = match public.split_once('!') {
        Some((epoch, rest)) if digits(epoch) => (epoch.to_owned(), rest),
        Some(_) => return None,
        None => ("0".to_owned(), public),
    };
    let (release, rest) = release(rest)?;
    let (pre, rest) = match tagged(rest, &PRE) {
        Some((rank, number, rest)) => (Some((rank, number)), rest),
        None => (None, rest),
    };
    let (post, rest) = match tagged(rest, &POST) {
        Some((_, number, rest)) => (Some(number), rest),
        None if rest
            .strip_prefix('-')
            .is_some_and(|after| after.starts_with(|c: char| c.is_ascii_digit())) =>
        {
            let (number, rest) = number_after(rest);
            (Some(number), rest)
        }
        None => (None, rest),
    };
    let (dev, rest) = match tagged(rest, &DEV) {
        Some((_, number, rest)) => (Some(number), rest),
        None => (None, rest),
    };
    rest.is_empty().then_some(Version {
        epoch,
        release,
        pre,
        post,
        dev,
        local,
    })
}

fn release(text: &str) -> Option<(Vec<String>, &str)> {
    let mut parts = Vec::new();
    let mut rest = text;
    loop {
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        if end == 0 {
            return None;
        }
        parts.push(rest[..end].to_owned());
        rest = &rest[end..];
        match rest.strip_prefix('.') {
            Some(next) if next.starts_with(|c: char| c.is_ascii_digit()) => rest = next,
            _ => return Some((parts, rest)),
        }
    }
}

fn tagged<'a>(text: &'a str, words: &[(&str, u8)]) -> Option<(u8, String, &'a str)> {
    let body = text.strip_prefix(['-', '_', '.']).unwrap_or(text);
    let (rank, after) = words
        .iter()
        .find_map(|&(word, rank)| body.strip_prefix(word).map(|after| (rank, after)))?;
    let (number, rest) = number_after(after);
    Some((rank, number, rest))
}

fn number_after(text: &str) -> (String, &str) {
    let body = text.strip_prefix(['-', '_', '.']).unwrap_or(text);
    let end = body
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(body.len());
    if end == 0 {
        ("0".to_owned(), text)
    } else {
        (body[..end].to_owned(), &body[end..])
    }
}

fn segments(text: &str) -> Option<Vec<String>> {
    let parts: Vec<String> = text.split(['.', '-', '_']).map(str::to_owned).collect();
    parts
        .iter()
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        .then_some(parts)
}

fn order(left: &Version, right: &Version) -> Ordering {
    cmp_digits(&left.epoch, &right.epoch)
        .then_with(|| cmp_release(&left.release, &right.release))
        .then_with(|| cmp_pre(left, right))
        .then_with(|| cmp_post(left, right))
        .then_with(|| cmp_dev(left, right))
        .then_with(|| cmp_local(left.local.as_deref(), right.local.as_deref()))
}

fn component(list: &[String], index: usize) -> &str {
    list.get(index).map_or("0", String::as_str)
}

fn cmp_release(left: &[String], right: &[String]) -> Ordering {
    for index in 0..left.len().max(right.len()) {
        let ordering = cmp_digits(component(left, index), component(right, index));
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn pre_class(version: &Version) -> i8 {
    match (&version.pre, &version.post, &version.dev) {
        (Some(_), _, _) => 0,
        (None, None, Some(_)) => -1,
        (None, _, _) => 1,
    }
}

fn cmp_pre(left: &Version, right: &Version) -> Ordering {
    pre_class(left)
        .cmp(&pre_class(right))
        .then_with(|| match (&left.pre, &right.pre) {
            (Some((rank, number)), Some((other_rank, other))) => {
                rank.cmp(other_rank).then_with(|| cmp_digits(number, other))
            }
            _ => Ordering::Equal,
        })
}

fn cmp_post(left: &Version, right: &Version) -> Ordering {
    match (&left.post, &right.post) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => cmp_digits(left, right),
    }
}

fn cmp_dev(left: &Version, right: &Version) -> Ordering {
    match (&left.dev, &right.dev) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => cmp_digits(left, right),
    }
}

fn cmp_local(left: Option<&[String]>, right: Option<&[String]>) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => {
            for (a, b) in left.iter().zip(right) {
                let ordering = match (digits(a), digits(b)) {
                    (true, true) => cmp_digits(a, b),
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    (false, false) => a.cmp(b),
                };
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            left.len().cmp(&right.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::version::{Ecosystem, assert_chain, assert_same, compare};

    #[test]
    fn ordering_examples_from_the_spec() {
        assert_chain(
            Ecosystem::PyPi,
            &[
                "1.0.dev456",
                "1.0a1",
                "1.0a2.dev456",
                "1.0a12.dev456",
                "1.0a12",
                "1.0b1.dev456",
                "1.0b2",
                "1.0b2.post345.dev456",
                "1.0b2.post345",
                "1.0rc1.dev456",
                "1.0rc1",
                "1.0",
                "1.0+abc.5",
                "1.0+abc.7",
                "1.0+5",
                "1.0.post456.dev34",
                "1.0.post456",
                "1.0.15",
                "1.1.dev1",
            ],
        );
    }

    #[test]
    fn epochs_and_plain_releases() {
        assert_chain(
            Ecosystem::PyPi,
            &[
                "0.9",
                "1.0.dev1",
                "1.0a1",
                "1.0",
                "1.0.post1",
                "1.1",
                "2.0",
                "10.0",
            ],
        );
        assert_chain(Ecosystem::PyPi, &["1.0", "1!0.5", "2!0.1"]);
    }

    #[test]
    fn spelling_variants_are_equal() {
        assert_same(
            Ecosystem::PyPi,
            &[
                ("1.0alpha1", "1.0a1"),
                ("1.0-a1", "1.0a1"),
                ("1.0c1", "1.0rc1"),
                ("1.0pre1", "1.0rc1"),
                ("1.0preview1", "1.0rc1"),
                ("1.0-1", "1.0.post1"),
                ("1.0rev2", "1.0post2"),
                ("1.0-r3", "1.0.post3"),
                ("v1.0", "1.0"),
                ("1.0.0", "1.0"),
                ("01.1", "1.1"),
                ("1.0a", "1.0a0"),
                ("1.0+LOCAL", "1.0+local"),
            ],
        );
    }

    #[test]
    fn malformed_versions_are_unparseable() {
        for bad in [
            "1.0a1a2",
            "1.0 1",
            "1..0",
            "a1",
            "1.0.x",
            "1.0!",
            "1!",
            "1!2!3",
            "1.0-",
            "1.0+",
            "1.0+a-b_c.",
        ] {
            assert_eq!(compare(Ecosystem::PyPi, bad, "1.0"), None, "{bad}");
        }
        assert_eq!(compare(Ecosystem::PyPi, "", "1.0"), None);
    }
}
