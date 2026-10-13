use std::cmp::Ordering;

pub fn compare(left: &str, right: &str) -> Ordering {
    let strip = |version: &str| version.split('+').next().unwrap_or_default().to_owned();
    let (a, b) = (strip(left), strip(right));
    let split = |version: &str| match version.split_once('-') {
        Some((core, pre)) => (core.to_owned(), Some(pre.to_owned())),
        None => (version.to_owned(), None),
    };
    let ((core_a, pre_a), (core_b, pre_b)) = (split(&a), split(&b));
    let core = identifiers(&core_a, &core_b, true);
    if core != Ordering::Equal {
        return core;
    }
    match (pre_a, pre_b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => identifiers(&a, &b, false),
    }
}

fn identifiers(a: &str, b: &str, numeric_only: bool) -> Ordering {
    let left: Vec<&str> = a.split('.').collect();
    let right: Vec<&str> = b.split('.').collect();
    for (x, y) in left.iter().zip(&right) {
        let found = identifier(x, y, numeric_only);
        if found != Ordering::Equal {
            return found;
        }
    }
    left.len().cmp(&right.len())
}

fn identifier(a: &str, b: &str, numeric_only: bool) -> Ordering {
    let numeric = |value: &str| !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit());
    match (numeric(a), numeric(b)) {
        (true, true) => {
            let trim = |value: &str| {
                let trimmed = value.trim_start_matches('0');
                if trimmed.is_empty() { "0" } else { trimmed }.to_owned()
            };
            let (x, y) = (trim(a), trim(b));
            x.len().cmp(&y.len()).then_with(|| x.cmp(&y))
        }
        (true, false) if !numeric_only => Ordering::Less,
        (false, true) if !numeric_only => Ordering::Greater,
        _ => a.cmp(b),
    }
}

pub fn sort(versions: &mut [String]) {
    versions.sort_by(|a, b| compare(a, b));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_follow_semantic_precedence() {
        let mut versions: Vec<String> = [
            "1.10.0",
            "1.2.0",
            "1.2.0-rc.1",
            "1.2.0-alpha",
            "1.2.0-rc.10",
            "1.2.0-rc.2",
            "0.9.0+build",
        ]
        .iter()
        .map(|v| v.to_string())
        .collect();
        sort(&mut versions);
        assert_eq!(
            versions,
            [
                "0.9.0+build",
                "1.2.0-alpha",
                "1.2.0-rc.1",
                "1.2.0-rc.2",
                "1.2.0-rc.10",
                "1.2.0",
                "1.10.0"
            ]
        );
        assert_eq!(compare("1.0.0+a", "1.0.0+b"), Ordering::Equal);
    }
}
