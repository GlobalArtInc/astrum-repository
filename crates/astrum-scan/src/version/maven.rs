use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Item {
    Int(String),
    Text(String),
    List(Vec<Item>),
}

const QUALIFIERS: [&str; 7] = ["alpha", "beta", "milestone", "rc", "snapshot", "", "sp"];
const RELEASE_INDEX: &str = "5";

impl Item {
    fn number(digits: &str) -> Item {
        let trimmed = digits.trim_start_matches('0');
        Item::Int(if trimmed.is_empty() { "0" } else { trimmed }.to_owned())
    }

    fn text(value: &str, followed_by_digit: bool) -> Item {
        let value = match (followed_by_digit, value) {
            (true, "a") => "alpha",
            (true, "b") => "beta",
            (true, "m") => "milestone",
            (_, "ga" | "final" | "release") => "",
            (_, "cr") => "rc",
            (_, other) => other,
        };
        Item::Text(value.to_owned())
    }

    fn parsed(is_digit: bool, value: &str) -> Item {
        if is_digit {
            Item::number(value)
        } else {
            Item::text(value, false)
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Item::Int(value) => value == "0",
            Item::Text(value) => value.is_empty(),
            Item::List(items) => items.is_empty(),
        }
    }

    fn compare(&self, other: Option<&Item>) -> Ordering {
        match (self, other) {
            (Item::Int(value), None) => {
                if value == "0" {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            }
            (Item::Int(left), Some(Item::Int(right))) => {
                left.len().cmp(&right.len()).then_with(|| left.cmp(right))
            }
            (Item::Int(_), Some(_)) => Ordering::Greater,
            (Item::Text(value), None) => qualifier(value).as_str().cmp(RELEASE_INDEX),
            (Item::Text(_), Some(Item::Int(_))) => Ordering::Less,
            (Item::Text(left), Some(Item::Text(right))) => qualifier(left).cmp(&qualifier(right)),
            (Item::Text(_), Some(Item::List(_))) => Ordering::Less,
            (Item::List(items), None) => items
                .iter()
                .map(|item| item.compare(None))
                .find(|ordering| *ordering != Ordering::Equal)
                .unwrap_or(Ordering::Equal),
            (Item::List(_), Some(Item::Int(_))) => Ordering::Less,
            (Item::List(_), Some(Item::Text(_))) => Ordering::Greater,
            (Item::List(left), Some(Item::List(right))) => compare_lists(left, right),
        }
    }
}

fn qualifier(value: &str) -> String {
    match QUALIFIERS.iter().position(|known| *known == value) {
        Some(index) => index.to_string(),
        None => format!("{}-{value}", QUALIFIERS.len()),
    }
}

fn compare_lists(left: &[Item], right: &[Item]) -> Ordering {
    let length = left.len().max(right.len());
    for index in 0..length {
        let ordering = match (left.get(index), right.get(index)) {
            (None, None) => Ordering::Equal,
            (None, Some(right)) => right.compare(None).reverse(),
            (Some(left), right) => left.compare(right),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn normalize(items: &mut Vec<Item>) {
    let mut index = items.len();
    while index > 0 {
        index -= 1;
        if items[index].is_null() {
            items.remove(index);
        } else if !matches!(items[index], Item::List(_)) {
            break;
        }
    }
}

fn parse(version: &str) -> Vec<Item> {
    let lowered = version.to_lowercase();
    let chars: Vec<char> = lowered.chars().collect();
    let mut levels: Vec<Vec<Item>> = vec![Vec::new()];
    let mut is_digit = false;
    let mut start = 0;
    let text = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
    for (index, &c) in chars.iter().enumerate() {
        let current = levels.len() - 1;
        if c == '.' || c == '-' {
            let item = if index == start {
                Item::Int("0".to_owned())
            } else {
                Item::parsed(is_digit, &text(start, index))
            };
            levels[current].push(item);
            start = index + 1;
            if c == '-' {
                levels.push(Vec::new());
            }
        } else if c.is_ascii_digit() {
            if !is_digit && index > start {
                levels[current].push(Item::text(&text(start, index), true));
                start = index;
                levels.push(Vec::new());
            }
            is_digit = true;
        } else {
            if is_digit && index > start {
                levels[current].push(Item::number(&text(start, index)));
                start = index;
                levels.push(Vec::new());
            }
            is_digit = false;
        }
    }
    if chars.len() > start {
        let current = levels.len() - 1;
        levels[current].push(Item::parsed(is_digit, &text(start, chars.len())));
    }
    let mut child: Option<Vec<Item>> = None;
    while let Some(mut level) = levels.pop() {
        if let Some(nested) = child.take() {
            level.push(Item::List(nested));
        }
        normalize(&mut level);
        child = Some(level);
    }
    child.unwrap_or_default()
}

pub(super) fn compare(left: &str, right: &str) -> Option<Ordering> {
    Some(compare_lists(&parse(left), &parse(right)))
}

#[cfg(test)]
mod tests {
    use crate::version::{Ecosystem, assert_chain, assert_same, compare};

    #[test]
    fn qualifiers_follow_the_maven_order() {
        assert_chain(
            Ecosystem::Maven,
            &[
                "1-alpha-1",
                "1-alpha-2",
                "1-beta",
                "1-milestone",
                "1-rc",
                "1-SNAPSHOT",
                "1",
                "1-sp",
                "1-abc",
                "1-1",
                "1.1",
                "1.2",
                "1.10",
                "2.0-RC1",
                "2.0",
                "10.0",
            ],
        );
        assert_chain(
            Ecosystem::Maven,
            &["1.0-alpha1", "1.0-beta1", "1.0-SNAPSHOT", "1.0", "1.0.1"],
        );
        assert_chain(
            Ecosystem::Maven,
            &["3.8.9", "3.9.0-rc-1", "3.9.0", "3.9.11"],
        );
    }

    #[test]
    fn equal_spellings_compare_equal() {
        assert_same(
            Ecosystem::Maven,
            &[
                ("1", "1.0"),
                ("1", "1.0.0"),
                ("1", "1-ga"),
                ("1", "1-final"),
                ("1.0-RC1", "1.0-cr1"),
                ("1a1", "1-alpha-1"),
                ("1.0.0", "1-0"),
                ("1.0-alpha1", "1-alpha1"),
                ("01", "1"),
            ],
        );
    }

    #[test]
    fn large_numbers_do_not_overflow() {
        assert_chain(
            Ecosystem::Maven,
            &["1.99999999999999999999", "1.100000000000000000000"],
        );
    }

    #[test]
    fn empty_input_is_unparseable() {
        assert_eq!(compare(Ecosystem::Maven, "", ""), None);
    }
}
