use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
struct Part<'a> {
    a: u64,
    b: &'a str,
    c: u64,
    d: &'a str,
}

fn number(text: &str) -> (u64, &str) {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let value = match end {
        0 => 0,
        _ => text[..end].parse().unwrap_or(u64::MAX),
    };
    (value, &text[end..])
}

fn string(text: &str) -> (&str, &str) {
    let end = text
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(text.len());
    (&text[..end], &text[end..])
}

fn part(text: &str) -> Part<'_> {
    if text == "*" {
        return Part {
            a: u64::MAX,
            b: "",
            c: 0,
            d: "",
        };
    }
    let (a, rest) = number(text);
    let (b, rest) = string(rest);
    if b == "+" {
        return Part {
            a: a.saturating_add(1),
            b: "pre",
            c: 0,
            d: "",
        };
    }
    let (c, d) = number(rest);
    Part { a, b, c, d }
}

fn strings(left: &str, right: &str) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => left.cmp(right),
    }
}

pub fn compare(left: &str, right: &str) -> Ordering {
    let mut lefts = left.split('.');
    let mut rights = right.split('.');
    loop {
        let (l, r) = match (lefts.next(), rights.next()) {
            (None, None) => return Ordering::Equal,
            (l, r) => (part(l.unwrap_or("0")), part(r.unwrap_or("0"))),
        };
        let order =
            l.a.cmp(&r.a)
                .then_with(|| strings(l.b, r.b))
                .then_with(|| l.c.cmp(&r.c))
                .then_with(|| strings(l.d, r.d));
        if order != Ordering::Equal {
            return order;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_like_the_toolkit_format() {
        let ordered = [
            "1.0pre1", "1.0pre2", "1.0", "1.0.1", "1.1a", "1.1pre", "1.1pre1a", "1.1pre1",
            "1.1pre10", "1.1", "1.10", "2.0b1", "2.0", "10.0",
        ];
        for pair in ordered.windows(2) {
            assert_eq!(compare(pair[0], pair[1]), Ordering::Less, "{pair:?}");
            assert_eq!(compare(pair[1], pair[0]), Ordering::Greater, "{pair:?}");
        }
        assert_eq!(compare("1.0", "1"), Ordering::Equal);
        assert_eq!(compare("1.0.0.0", "1"), Ordering::Equal);
        assert_eq!(compare("1.0+", "1.1pre"), Ordering::Equal);
        assert_eq!(compare("1.*", "1.999"), Ordering::Greater);
        assert_eq!(compare("1.99999999999999999999", "1.2"), Ordering::Greater);
    }
}
