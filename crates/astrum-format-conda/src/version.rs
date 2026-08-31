use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Part {
    Dev,
    Text(String),
    Number(u128),
    Post,
}

const ZERO: Part = Part::Number(0);

fn parts(component: &str) -> Vec<Part> {
    let mut found = Vec::new();
    let mut rest = component;
    while let Some(first) = rest.chars().next() {
        let digits = first.is_ascii_digit();
        let end = rest
            .find(|c: char| c.is_ascii_digit() != digits)
            .unwrap_or(rest.len());
        let (run, tail) = rest.split_at(end);
        found.push(if digits {
            Part::Number(run.parse().unwrap_or(u128::MAX))
        } else {
            match run {
                "dev" => Part::Dev,
                "post" => Part::Post,
                _ => Part::Text(run.to_owned()),
            }
        });
        rest = tail;
    }
    if !matches!(found.first(), Some(Part::Number(_))) {
        found.insert(0, ZERO);
    }
    found
}

fn components(text: &str) -> Vec<Vec<Part>> {
    let (body, underscore) = match text.strip_suffix('_') {
        Some(body) => (body, true),
        None => (text, false),
    };
    let mut found: Vec<Vec<Part>> = body.split(['.', '_', '-']).map(parts).collect();
    if underscore && let Some(last) = found.last_mut() {
        last.push(Part::Text("_".to_owned()));
    }
    found
}

fn compare_component(left: &[Part], right: &[Part]) -> Ordering {
    for at in 0..left.len().max(right.len()) {
        let ordering = left
            .get(at)
            .unwrap_or(&ZERO)
            .cmp(right.get(at).unwrap_or(&ZERO));
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn compare_components(left: &[Vec<Part>], right: &[Vec<Part>]) -> Ordering {
    let zero = vec![ZERO];
    for at in 0..left.len().max(right.len()) {
        let ordering = compare_component(
            left.get(at).unwrap_or(&zero),
            right.get(at).unwrap_or(&zero),
        );
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

/// A conda version as conda orders it: an optional epoch before `!`, the version, and an optional
/// local version after `+`, each split into components at `.` and `_` and each component into runs
/// of digits and letters (with `dev` lowest, `post` highest and numbers above other text).
#[derive(Clone, Debug)]
pub struct Version {
    epoch: u128,
    main: Vec<Vec<Part>>,
    local: Vec<Vec<Part>>,
}

impl Version {
    pub fn parse(text: &str) -> Version {
        let lowered = text.trim().to_ascii_lowercase();
        let (epoch, rest) = match lowered.split_once('!') {
            Some((epoch, rest))
                if epoch.chars().all(|c| c.is_ascii_digit()) && !epoch.is_empty() =>
            {
                (epoch.parse().unwrap_or(u128::MAX), rest.to_owned())
            }
            _ => (0, lowered),
        };
        let (main, local) = match rest.split_once('+') {
            Some((main, local)) => (components(main), components(local)),
            None => (components(&rest), Vec::new()),
        };
        Version { epoch, main, local }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.epoch
            .cmp(&other.epoch)
            .then_with(|| compare_components(&self.main, &other.main))
            .then_with(|| compare_components(&self.local, &other.local))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text)
    }

    #[test]
    fn versions_order_like_conda() {
        let ordered = [
            "0.4.1.rc",
            "0.4.1",
            "0.5a1",
            "0.5b3",
            "0.5C1",
            "0.5",
            "0.9.6",
            "0.960923",
            "1.0",
            "1.1dev1",
            "1.1_",
            "1.1a1",
            "1.1.0dev1",
            "1.1.a1",
            "1.1.0rc1",
            "1.1.0",
            "1.1.0post1",
            "1.1post1",
            "1996.07.12",
            "1!0.4.1",
            "1!3.1.1.6",
            "2!0.4.1",
        ];
        for pair in ordered.windows(2) {
            assert!(v(pair[0]) < v(pair[1]), "{} < {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn equal_spellings_compare_equal() {
        assert_eq!(v("0.4.1.rc"), v("0.4.1.RC"));
        assert_eq!(v("1.1.0dev1"), v("1.1.dev1"));
        assert_eq!(v("1.1.0"), v("1.1"));
        assert_eq!(v("1.1.0post1"), v("1.1.post1"));
        assert!(v("1.0+local.2") > v("1.0+local.1"));
        assert!(v("1.10") > v("1.9"));
        assert!(v("2.0") > v("2.0rc1"));
    }
}
