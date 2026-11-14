use std::cmp::Ordering;

struct Version<'a> {
    numbers: [u64; 4],
    pre: Vec<&'a str>,
}

pub(super) fn compare(left: &str, right: &str) -> Option<Ordering> {
    let left = parse(left)?;
    let right = parse(right)?;
    Some(order(&left, &right))
}

fn number(text: &str) -> Option<u64> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn identifier(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn parse(text: &str) -> Option<Version<'_>> {
    let text = text.trim();
    let (rest, metadata) = match text.split_once('+') {
        Some((rest, metadata)) => (rest, Some(metadata)),
        None => (text, None),
    };
    if metadata.is_some_and(|metadata| !metadata.split('.').all(identifier)) {
        return None;
    }
    let (release, pre) = match rest.split_once('-') {
        Some((release, pre)) => {
            let labels: Vec<&str> = pre.split('.').collect();
            if !labels.iter().all(|label| identifier(label)) {
                return None;
            }
            (release, labels)
        }
        None => (rest, Vec::new()),
    };
    let pieces: Vec<&str> = release.split('.').collect();
    if pieces.len() > 4 {
        return None;
    }
    let mut numbers = [0u64; 4];
    for (slot, piece) in numbers.iter_mut().zip(&pieces) {
        *slot = number(piece)?;
    }
    Some(Version { numbers, pre })
}

fn compare_label(left: &str, right: &str) -> Ordering {
    match (number(left), number(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()),
    }
}

fn order(left: &Version, right: &Version) -> Ordering {
    left.numbers.cmp(&right.numbers).then_with(|| {
        match (left.pre.is_empty(), right.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                for (a, b) in left.pre.iter().zip(&right.pre) {
                    let ordering = compare_label(a, b);
                    if ordering != Ordering::Equal {
                        return ordering;
                    }
                }
                left.pre.len().cmp(&right.pre.len())
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::version::{Ecosystem, assert_chain, assert_same, compare};

    #[test]
    fn numbers_then_labels_like_the_gallery() {
        assert_chain(
            Ecosystem::NuGet,
            &[
                "0.9",
                "1.0.0-alpha",
                "1.0.0-Beta",
                "1.0.0-beta.2",
                "1.0.0-beta.11",
                "1.0.0",
                "1.0.0.1",
                "10.0.0",
            ],
        );
        assert_chain(
            Ecosystem::NuGet,
            &[
                "1.0.0-1",
                "1.0.0-10",
                "1.0.0-a",
                "1.0.0-a.1",
                "1.0.0-b",
                "1.0.0",
            ],
        );
        assert_chain(Ecosystem::NuGet, &["1.2.3", "1.2.3.1", "1.2.4", "2"]);
    }

    #[test]
    fn normalised_spellings_and_metadata_are_equal() {
        assert_same(
            Ecosystem::NuGet,
            &[
                ("1.0", "1.0.0.0"),
                ("1.0.0-RC", "1.0.0-rc"),
                ("1.0.0+a", "1.0.0+b"),
                ("01.002.3", "1.2.3"),
                ("2", "2.0.0"),
            ],
        );
    }

    #[test]
    fn malformed_versions_are_unparseable() {
        for bad in [
            "",
            "a.b",
            "1..2",
            "1.2.3.4.5",
            "1.0-",
            "1.0-be ta",
            "1.0+",
            "-1",
            "99999999999999999999.0",
        ] {
            assert_eq!(compare(Ecosystem::NuGet, bad, "1.0"), None, "{bad}");
        }
    }
}
