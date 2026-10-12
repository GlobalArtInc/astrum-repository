use std::cmp::Ordering;

const RELEASE: u8 = 6;

#[derive(Debug, PartialEq, Eq)]
enum Item<'a> {
    Number(&'a str),
    Word(&'a str),
}

fn items(version: &str) -> Vec<Item<'_>> {
    let mut items = Vec::new();
    let mut start = None;
    let mut digits = false;
    for (index, c) in version.char_indices() {
        let separator = !c.is_alphanumeric();
        match start {
            Some(begin) if separator || c.is_ascii_digit() != digits => {
                items.push(item(&version[begin..index], digits));
                start = (!separator).then_some(index);
            }
            None if !separator => start = Some(index),
            _ => {}
        }
        if !separator {
            digits = c.is_ascii_digit();
        }
    }
    if let Some(begin) = start {
        items.push(item(&version[begin..], digits));
    }
    items
}

fn item(text: &str, digits: bool) -> Item<'_> {
    match digits {
        true => Item::Number(text),
        false => Item::Word(text),
    }
}

fn rank(word: &str) -> u8 {
    match word.to_ascii_lowercase().as_str() {
        "dev" => 0,
        "alpha" | "a" => 1,
        "beta" | "b" => 2,
        "milestone" | "m" => 3,
        "rc" | "cr" | "c" | "pre" | "preview" => 4,
        "snapshot" => 5,
        "ga" | "final" | "release" | "r" => RELEASE,
        "sp" | "post" | "patch" | "pl" => 7,
        _ => 5,
    }
}

fn numbers(left: &str, right: &str) -> Ordering {
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn words(left: &str, right: &str) -> Ordering {
    rank(left)
        .cmp(&rank(right))
        .then_with(|| left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()))
}

fn against_nothing(item: &Item<'_>) -> Ordering {
    match item {
        Item::Number(number) => numbers(number, "0"),
        Item::Word(word) => rank(word).cmp(&RELEASE),
    }
}

pub(super) fn compare(left: &str, right: &str) -> Ordering {
    let left_items = items(left);
    let right_items = items(right);
    let longest = left_items.len().max(right_items.len());
    for index in 0..longest {
        let ordering = match (left_items.get(index), right_items.get(index)) {
            (Some(Item::Number(a)), Some(Item::Number(b))) => numbers(a, b),
            (Some(Item::Word(a)), Some(Item::Word(b))) => words(a, b),
            (Some(Item::Number(_)), Some(Item::Word(_))) => Ordering::Greater,
            (Some(Item::Word(_)), Some(Item::Number(_))) => Ordering::Less,
            (Some(item), None) => against_nothing(item),
            (None, Some(item)) => against_nothing(item).reverse(),
            (None, None) => Ordering::Equal,
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.cmp(right)
}

pub(super) fn natural(left: &str, right: &str) -> Ordering {
    let mut a = left.chars().peekable();
    let mut b = right.chars().peekable();
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return left.cmp(right),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let mut first = String::new();
                while let Some(c) = a.peek().copied().filter(char::is_ascii_digit) {
                    first.push(c);
                    a.next();
                }
                let mut second = String::new();
                while let Some(c) = b.peek().copied().filter(char::is_ascii_digit) {
                    second.push(c);
                    b.next();
                }
                let ordering = numbers(&first, &second);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(x), Some(y)) => {
                let ordering = x.to_lowercase().cmp(y.to_lowercase());
                if ordering != Ordering::Equal {
                    return ordering;
                }
                a.next();
                b.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut versions: Vec<&str>) -> Vec<&str> {
        versions.sort_by(|a, b| compare(a, b));
        versions
    }

    #[test]
    fn numbers_compare_as_numbers() {
        assert_eq!(
            sorted(vec!["1.10.0", "1.9.0", "1.2", "10.0", "2.0"]),
            ["1.2", "1.9.0", "1.10.0", "2.0", "10.0"]
        );
        assert_eq!(compare("1.0", "1.0.0"), Ordering::Less);
        assert_eq!(compare("1.0.1", "1.0"), Ordering::Greater);
        assert_eq!(compare("1.01", "1.1"), Ordering::Less);
    }

    #[test]
    fn pre_releases_come_before_releases() {
        assert_eq!(
            sorted(vec![
                "1.0.0",
                "1.0.0-rc.1",
                "1.0.0-alpha",
                "1.0.0-beta.2",
                "1.0.0-beta.11",
                "0.9.9",
                "1.0.1"
            ]),
            [
                "0.9.9",
                "1.0.0-alpha",
                "1.0.0-beta.2",
                "1.0.0-beta.11",
                "1.0.0-rc.1",
                "1.0.0",
                "1.0.1"
            ]
        );
        assert_eq!(compare("1.0-SNAPSHOT", "1.0"), Ordering::Less);
        assert_eq!(compare("1.0-rc1", "1.0-SNAPSHOT"), Ordering::Less);
        assert_eq!(compare("2.0.0.RELEASE", "2.0.0"), Ordering::Greater);
        assert_eq!(compare("1.0.Final", "1.0"), Ordering::Greater);
    }

    #[test]
    fn python_style_versions_order_too() {
        assert_eq!(
            sorted(vec![
                "1.0.post1",
                "1.0",
                "1.0a1",
                "1.0b2",
                "1.0rc1",
                "1.0.dev3",
                "1.1"
            ]),
            [
                "1.0.dev3",
                "1.0a1",
                "1.0b2",
                "1.0rc1",
                "1.0",
                "1.0.post1",
                "1.1"
            ]
        );
    }

    #[test]
    fn names_sort_naturally() {
        let mut names = vec!["file10.txt", "File2.txt", "file1.txt", "a", "B"];
        names.sort_by(|a, b| natural(a, b));
        assert_eq!(names, ["a", "B", "file1.txt", "File2.txt", "file10.txt"]);
    }
}
