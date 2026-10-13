use std::cmp::Ordering;

pub fn split_evr(evr: &str) -> (String, String, Option<String>) {
    let (epoch, rest) = match evr.split_once(':') {
        Some((epoch, rest)) if !epoch.is_empty() && epoch.chars().all(|c| c.is_ascii_digit()) => {
            (epoch.to_owned(), rest)
        }
        _ => ("0".to_owned(), evr),
    };
    match rest.rsplit_once('-') {
        Some((version, release)) => (epoch, version.to_owned(), Some(release.to_owned())),
        None => (epoch, rest.to_owned(), None),
    }
}

fn separator(c: char) -> bool {
    !c.is_alphanumeric() && c != '~' && c != '^'
}

pub fn compare(first: &str, second: &str) -> Ordering {
    if first == second {
        return Ordering::Equal;
    }
    let a: Vec<char> = first.chars().collect();
    let b: Vec<char> = second.chars().collect();
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        while i < a.len() && separator(a[i]) {
            i += 1;
        }
        while j < b.len() && separator(b[j]) {
            j += 1;
        }
        let tilde_a = a.get(i) == Some(&'~');
        let tilde_b = b.get(j) == Some(&'~');
        if tilde_a || tilde_b {
            if !tilde_a {
                return Ordering::Greater;
            }
            if !tilde_b {
                return Ordering::Less;
            }
            i += 1;
            j += 1;
            continue;
        }
        let caret_a = a.get(i) == Some(&'^');
        let caret_b = b.get(j) == Some(&'^');
        if caret_a || caret_b {
            if i >= a.len() {
                return Ordering::Less;
            }
            if j >= b.len() {
                return Ordering::Greater;
            }
            if !caret_a {
                return Ordering::Greater;
            }
            if !caret_b {
                return Ordering::Less;
            }
            i += 1;
            j += 1;
            continue;
        }
        if i >= a.len() || j >= b.len() {
            break;
        }
        let (start_a, start_b) = (i, j);
        let numeric = a[i].is_ascii_digit();
        let keep = |c: char| match numeric {
            true => c.is_ascii_digit(),
            false => c.is_alphabetic(),
        };
        while i < a.len() && keep(a[i]) {
            i += 1;
        }
        while j < b.len() && keep(b[j]) {
            j += 1;
        }
        let segment_a: String = a[start_a..i].iter().collect();
        let segment_b: String = b[start_b..j].iter().collect();
        if segment_b.is_empty() {
            return match numeric {
                true => Ordering::Greater,
                false => Ordering::Less,
            };
        }
        let result = match numeric {
            true => {
                let x = segment_a.trim_start_matches('0');
                let y = segment_b.trim_start_matches('0');
                x.len().cmp(&y.len()).then_with(|| x.cmp(y))
            }
            false => segment_a.cmp(&segment_b),
        };
        if result != Ordering::Equal {
            return result;
        }
    }
    match (i >= a.len(), j >= b.len()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        _ => Ordering::Greater,
    }
}

fn glibc(name: &str) -> Option<&str> {
    let start = name.find("(GLIBC_")? + "(GLIBC_".len();
    let end = name[start..].find(')')? + start;
    Some(&name[start..end])
}

pub fn compare_libc(first: &str, second: &str) -> Ordering {
    match (glibc(first), glibc(second)) {
        (Some(a), Some(b)) => compare(a, b),
        (a, b) => a.is_some().cmp(&b.is_some()),
    }
}

pub fn is_prerelease(component_version: &str) -> bool {
    let rest = component_version
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or(component_version);
    let (version, release) = rest.rsplit_once('-').unwrap_or((rest, ""));
    version.contains('~') || release.starts_with("0.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_like_rpm() {
        for (a, b, expected) in [
            ("1.0", "1.0", Ordering::Equal),
            ("1.0", "2.0", Ordering::Less),
            ("2.0.1", "2.0", Ordering::Greater),
            ("1.0010", "1.9", Ordering::Greater),
            ("1.05", "1.5", Ordering::Equal),
            ("1.0a", "1.0", Ordering::Greater),
            ("1.0~rc1", "1.0", Ordering::Less),
            ("1.0~rc1", "1.0~rc2", Ordering::Less),
            ("1.0^", "1.0", Ordering::Greater),
            ("1.0^git1", "1.0.1", Ordering::Less),
            ("a", "1", Ordering::Less),
            ("1_0", "1.0", Ordering::Equal),
        ] {
            assert_eq!(compare(a, b), expected, "{a} vs {b}");
            assert_eq!(compare(b, a), expected.reverse(), "{b} vs {a}");
        }
    }

    #[test]
    fn evr_and_prereleases() {
        assert_eq!(
            split_evr("2:1.0-3"),
            ("2".to_owned(), "1.0".to_owned(), Some("3".to_owned()))
        );
        assert_eq!(split_evr("1.0"), ("0".to_owned(), "1.0".to_owned(), None));
        assert!(is_prerelease("1.0~beta-1"));
        assert!(is_prerelease("3:1.0-0.1.rc1"));
        assert!(!is_prerelease("1.0-1.el9"));
        assert_eq!(
            compare_libc(
                "libc.so.6(GLIBC_2.34)(64bit)",
                "libc.so.6(GLIBC_2.4)(64bit)"
            ),
            Ordering::Greater
        );
        assert_eq!(
            compare_libc("libc.so.6()(64bit)", "libc.so.6(GLIBC_2.4)(64bit)"),
            Ordering::Less
        );
    }
}
