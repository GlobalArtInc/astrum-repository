use std::cmp::Ordering;

struct Parts<'a> {
    epoch: &'a str,
    version: &'a str,
    release: Option<&'a str>,
}

fn parts(full: &str) -> Parts<'_> {
    let (epoch, rest) = match full.split_once(':') {
        Some((epoch, rest)) if epoch.bytes().all(|byte| byte.is_ascii_digit()) => (epoch, rest),
        _ => ("0", full),
    };
    let (version, release) = match rest.rsplit_once('-') {
        Some((version, release)) => (version, Some(release)),
        None => (rest, None),
    };
    Parts {
        epoch: if epoch.is_empty() { "0" } else { epoch },
        version,
        release,
    }
}

fn numbers(a: &[u8], b: &[u8]) -> Ordering {
    let trim = |digits: &[u8]| {
        let start = digits
            .iter()
            .position(|digit| *digit != b'0')
            .unwrap_or(digits.len());
        digits[start..].to_vec()
    };
    let (a, b) = (trim(a), trim(b));
    a.len().cmp(&b.len()).then_with(|| a.cmp(&b))
}

fn take(text: &[u8], at: &mut usize, numeric: bool) -> usize {
    let start = *at;
    while *at < text.len()
        && match numeric {
            true => text[*at].is_ascii_digit(),
            false => text[*at].is_ascii_alphabetic(),
        }
    {
        *at += 1;
    }
    start
}

pub fn segments(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        let (gap_a, gap_b) = (i, j);
        while i < a.len() && !a[i].is_ascii_alphanumeric() {
            i += 1;
        }
        while j < b.len() && !b[j].is_ascii_alphanumeric() {
            j += 1;
        }
        if i >= a.len() || j >= b.len() {
            break;
        }
        if i - gap_a != j - gap_b {
            return (i - gap_a).cmp(&(j - gap_b));
        }
        let numeric = a[i].is_ascii_digit();
        let start_a = take(a, &mut i, numeric);
        let start_b = take(b, &mut j, numeric);
        let (left, right) = (&a[start_a..i], &b[start_b..j]);
        if right.is_empty() {
            return match numeric {
                true => Ordering::Greater,
                false => Ordering::Less,
            };
        }
        let order = match numeric {
            true => numbers(left, right),
            false => left.cmp(right),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    let (rest_a, rest_b) = (&a[i..], &b[j..]);
    if rest_a.is_empty() && rest_b.is_empty() {
        return Ordering::Equal;
    }
    let older = (rest_a.is_empty() && !rest_b[0].is_ascii_alphabetic())
        || rest_a.first().is_some_and(u8::is_ascii_alphabetic);
    match older {
        true => Ordering::Less,
        false => Ordering::Greater,
    }
}

pub fn compare(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let (a, b) = (parts(a), parts(b));
    segments(a.epoch, b.epoch)
        .then_with(|| segments(a.version, b.version))
        .then_with(|| match (a.release, b.release) {
            (Some(left), Some(right)) => segments(left, right),
            _ => Ordering::Equal,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascending(versions: &[&str]) {
        for pair in versions.windows(2) {
            assert_eq!(compare(pair[0], pair[1]), Ordering::Less, "{pair:?}");
            assert_eq!(compare(pair[1], pair[0]), Ordering::Greater, "{pair:?}");
        }
    }

    #[test]
    fn versions_order_like_the_vercmp_documentation() {
        ascending(&[
            "1.0a", "1.0b", "1.0beta", "1.0p", "1.0pre", "1.0rc", "1.0", "1.0.a", "1.0.1",
        ]);
        ascending(&["1", "1.0", "1.1", "1.1.1", "1.2", "2.0", "3.0.0"]);
        assert_eq!(compare("1:1.0-1", "2.0-1"), Ordering::Greater);
        assert_eq!(compare("2:1.0-1", "1:3.6-1"), Ordering::Greater);
        assert_eq!(compare("1.5-1", "1.5"), Ordering::Equal);
        assert_eq!(compare("1.5-1", "1.5-2"), Ordering::Less);
        assert_eq!(compare("1", "2"), Ordering::Less);
        assert_eq!(compare("2.0-1", "1.7-6"), Ordering::Greater);
        assert_eq!(compare("2.0", "2.0-13"), Ordering::Equal);
        assert_eq!(compare("4.34", "1:001"), Ordering::Less);
        assert_eq!(compare("1.010", "1.10"), Ordering::Equal);
        assert_eq!(compare("1.0-10", "1.0-9"), Ordering::Greater);
        assert_eq!(compare("0:1.0-1", "1.0-1"), Ordering::Equal);
    }
}
