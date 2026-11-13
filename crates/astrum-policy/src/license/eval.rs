use super::parse::Expression;
use super::spdx;

pub fn satisfies(expression: &Expression, allow: &[String]) -> bool {
    let allowed: Vec<String> = allow.iter().map(|id| resolved(id)).collect();
    allows(expression, &allowed)
}

pub fn violates(expression: &Expression, deny: &[String]) -> bool {
    let denied: Vec<String> = deny.iter().map(|id| resolved(id)).collect();
    denies(expression, &denied)
}

pub fn same_id(left: &str, right: &str) -> bool {
    resolved(left) == resolved(right)
}

fn allows(expression: &Expression, allowed: &[String]) -> bool {
    match expression {
        Expression::Id(id) | Expression::With(id, _) => allowed.contains(&resolved(id)),
        Expression::And(terms) => terms.iter().all(|term| allows(term, allowed)),
        Expression::Or(terms) => terms.iter().any(|term| allows(term, allowed)),
    }
}

fn denies(expression: &Expression, denied: &[String]) -> bool {
    match expression {
        Expression::Id(id) | Expression::With(id, _) => denied.contains(&resolved(id)),
        Expression::And(terms) => terms.iter().any(|term| denies(term, denied)),
        Expression::Or(terms) => terms.iter().all(|term| denies(term, denied)),
    }
}

fn resolved(id: &str) -> String {
    match spdx::lookup(id) {
        Some(entry) if entry.deprecated && !entry.equivalent.is_empty() => {
            entry.equivalent.to_ascii_lowercase()
        }
        _ => id.to_ascii_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::normalise;

    fn expression(text: &str) -> Expression {
        normalise(text)
            .expression
            .expect("the test expression parses")
    }

    fn list(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    #[test]
    fn satisfies_needs_an_or_branch_whose_and_terms_are_all_allowed() {
        let cases: [(&str, &[&str], bool); 9] = [
            ("MIT", &["MIT"], true),
            ("MIT", &["Apache-2.0"], false),
            ("MIT OR Apache-2.0", &["Apache-2.0"], true),
            ("MIT OR Apache-2.0", &["BSD-3-Clause"], false),
            ("MIT AND BSD-3-Clause", &["MIT"], false),
            ("MIT AND BSD-3-Clause", &["MIT", "BSD-3-Clause"], true),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["Apache-2.0", "BSD-3-Clause"],
                true,
            ),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["Apache-2.0"],
                false,
            ),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["BSD-3-Clause"],
                false,
            ),
        ];
        for (text, allow, expected) in cases {
            assert_eq!(
                satisfies(&expression(text), &list(allow)),
                expected,
                "{text} with {allow:?}"
            );
        }
    }

    #[test]
    fn violates_needs_every_or_branch_to_hold_a_denied_id() {
        let cases: [(&str, &[&str], bool); 9] = [
            ("MIT", &["MIT"], true),
            ("MIT", &["Apache-2.0"], false),
            ("MIT OR Apache-2.0", &["MIT"], false),
            ("MIT OR Apache-2.0", &["MIT", "Apache-2.0"], true),
            ("MIT AND BSD-3-Clause", &["BSD-3-Clause"], true),
            ("MIT AND BSD-3-Clause", &["Apache-2.0"], false),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["BSD-3-Clause"],
                true,
            ),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["Apache-2.0"],
                false,
            ),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["MIT", "Apache-2.0"],
                true,
            ),
        ];
        for (text, deny, expected) in cases {
            assert_eq!(
                violates(&expression(text), &list(deny)),
                expected,
                "{text} with {deny:?}"
            );
        }
    }

    #[test]
    fn with_exceptions_do_not_count_for_either_list() {
        let text = "GPL-2.0-only WITH Classpath-exception-2.0";
        assert!(satisfies(&expression(text), &list(&["GPL-2.0-only"])));
        assert!(!satisfies(
            &expression(text),
            &list(&["Classpath-exception-2.0"])
        ));
        assert!(violates(&expression(text), &list(&["GPL-2.0-only"])));
        assert!(!violates(
            &expression(text),
            &list(&["Classpath-exception-2.0"])
        ));
    }

    #[test]
    fn ids_compare_case_insensitively_and_exactly_without_prefixes() {
        assert!(satisfies(&expression("MIT"), &list(&["mit"])));
        assert!(satisfies(
            &expression("GPL-3.0-only"),
            &list(&["gpl-3.0-ONLY"])
        ));
        assert!(!satisfies(
            &expression("GPL-3.0-or-later"),
            &list(&["GPL-3.0-only"])
        ));
        assert!(!satisfies(
            &expression("GPL-3.0-only"),
            &list(&["GPL-3.0-or-later"])
        ));
        assert!(!violates(
            &expression("GPL-3.0-or-later"),
            &list(&["GPL-3.0-only"])
        ));
        assert!(violates(
            &expression("GPL-3.0-or-later"),
            &list(&["gpl-3.0-OR-later"])
        ));
    }

    #[test]
    fn deprecated_ids_compare_through_their_equivalent() {
        assert!(violates(&expression("GPL-3.0"), &list(&["GPL-3.0-only"])));
        assert!(violates(&expression("GPL-3.0-only"), &list(&["GPL-3.0"])));
        assert!(violates(
            &expression("GPL-2.0-with-classpath-exception"),
            &list(&["GPL-2.0-only"])
        ));
        assert!(!violates(&expression("GPL-3.0+"), &list(&["GPL-3.0-only"])));
        assert!(satisfies(
            &expression("GPL-3.0+"),
            &list(&["GPL-3.0-or-later"])
        ));
        assert!(satisfies(
            &expression("GPL-3.0-or-later"),
            &list(&["GPL-3.0+"])
        ));
        assert!(satisfies(&expression("GPL-3.0-only"), &list(&["GPL-3.0"])));
    }

    #[test]
    fn same_id_ignores_case_and_follows_deprecated_equivalents() {
        assert!(same_id("mit", "MIT"));
        assert!(same_id("GPL-3.0", "GPL-3.0-only"));
        assert!(same_id("gpl-3.0+", "GPL-3.0-or-later"));
        assert!(!same_id("GPL-3.0-only", "GPL-3.0-or-later"));
        assert!(!same_id("MIT", "ISC"));
    }
}
