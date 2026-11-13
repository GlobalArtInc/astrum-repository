mod alias;
mod eval;
mod parse;
mod spdx;

pub use eval::{satisfies, violates};
pub use parse::Expression;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Spdx,
    Mapped,
    Unknown,
    None,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spdx => "spdx",
            Self::Mapped => "mapped",
            Self::Unknown => "unknown",
            Self::None => "none",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Normalised {
    pub expression: Option<Expression>,
    pub text: String,
    pub ids: Vec<String>,
    pub state: State,
    pub deprecated: bool,
}

pub fn normalise(input: &str) -> Normalised {
    let text = input.trim();
    if text.is_empty() {
        return describe(text, None, State::None);
    }
    match parse::parse(text) {
        Some(parsed)
            if parsed
                .expression
                .ids()
                .iter()
                .all(|id| spdx::lookup(id).is_some()) =>
        {
            let state = if parsed.mapped {
                State::Mapped
            } else {
                State::Spdx
            };
            describe(text, Some(parsed.expression), state)
        }
        parsed => match alias::lookup(text) {
            Some(id) => describe(text, Some(Expression::Id(id.to_owned())), State::Mapped),
            None => describe(text, parsed.map(|parsed| parsed.expression), State::Unknown),
        },
    }
}

fn describe(input: &str, expression: Option<Expression>, state: State) -> Normalised {
    let mut ids: Vec<String> = Vec::new();
    if let Some(expression) = &expression {
        for id in expression.ids() {
            if !ids.iter().any(|known| known.as_str() == id) {
                ids.push(id.to_owned());
            }
        }
    }
    let deprecated = ids
        .iter()
        .any(|id| spdx::lookup(id).is_some_and(|entry| entry.deprecated));
    let text = match &expression {
        Some(expression) => expression.to_string(),
        None => input.to_owned(),
    };
    Normalised {
        expression,
        text,
        ids,
        state,
        deprecated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested(levels: usize) -> String {
        format!("{}MIT{}", "(".repeat(levels), ")".repeat(levels))
    }

    #[test]
    fn valid_ids_are_spdx_and_come_back_in_their_canonical_spelling() {
        let cases = [
            ("MIT", "MIT"),
            ("Apache-2.0", "Apache-2.0"),
            ("BSD-3-Clause", "BSD-3-Clause"),
            ("GPL-3.0-or-later", "GPL-3.0-or-later"),
            ("mit", "MIT"),
            ("apache-2.0", "Apache-2.0"),
        ];
        for (input, text) in cases {
            let normalised = normalise(input);
            assert_eq!(normalised.state, State::Spdx, "{input}");
            assert_eq!(normalised.text, text, "{input}");
            assert!(!normalised.deprecated, "{input}");
        }
    }

    #[test]
    fn deprecated_ids_stay_as_written_and_are_flagged() {
        for input in [
            "GPL-2.0",
            "GPL-2.0+",
            "GPL-3.0+",
            "LGPL-2.1",
            "GPL-2.0-with-classpath-exception",
        ] {
            let normalised = normalise(input);
            assert_eq!(normalised.state, State::Spdx, "{input}");
            assert_eq!(normalised.text, input, "{input}");
            assert!(normalised.deprecated, "{input}");
        }
    }

    #[test]
    fn expressions_keep_their_operators_and_drop_redundant_parentheses() {
        let cases: [(&str, &str, &[&str]); 6] = [
            (
                "MIT OR Apache-2.0",
                "MIT OR Apache-2.0",
                &["MIT", "Apache-2.0"],
            ),
            (
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                "(MIT OR Apache-2.0) AND BSD-3-Clause",
                &["MIT", "Apache-2.0", "BSD-3-Clause"],
            ),
            (
                "GPL-2.0-only WITH Classpath-exception-2.0",
                "GPL-2.0-only WITH Classpath-exception-2.0",
                &["GPL-2.0-only"],
            ),
            (
                "MIT AND Apache-2.0 OR BSD-3-Clause",
                "MIT AND Apache-2.0 OR BSD-3-Clause",
                &["MIT", "Apache-2.0", "BSD-3-Clause"],
            ),
            (
                "((MIT)) OR apache-2.0",
                "MIT OR Apache-2.0",
                &["MIT", "Apache-2.0"],
            ),
            ("MIT OR MIT", "MIT OR MIT", &["MIT"]),
        ];
        for (input, text, ids) in cases {
            let normalised = normalise(input);
            assert_eq!(normalised.state, State::Spdx, "{input}");
            assert_eq!(normalised.text, text, "{input}");
            assert_eq!(normalised.ids, ids, "{input}");
        }
    }

    #[test]
    fn the_legacy_slash_reads_as_or_and_is_mapped() {
        let normalised = normalise("MIT/Apache-2.0");
        assert_eq!(normalised.state, State::Mapped);
        assert_eq!(normalised.text, "MIT OR Apache-2.0");
        assert_eq!(normalised.ids, ["MIT", "Apache-2.0"]);
    }

    #[test]
    fn plus_keeps_deprecated_ids_and_is_unknown_without_an_or_later_id() {
        let cases = [
            ("GPL-2.0+", State::Spdx, "GPL-2.0+", true),
            ("GPLv2+", State::Mapped, "GPL-2.0-or-later", false),
            ("MIT+", State::Unknown, "MIT+", false),
            ("Apache-2.0+", State::Unknown, "Apache-2.0+", false),
            ("GPL-2.0-only+", State::Unknown, "GPL-2.0-only+", false),
        ];
        for (input, state, text, deprecated) in cases {
            let normalised = normalise(input);
            assert_eq!(normalised.state, state, "{input}");
            assert_eq!(normalised.text, text, "{input}");
            assert_eq!(normalised.deprecated, deprecated, "{input}");
        }
    }

    #[test]
    fn licenseref_and_documentref_are_kept_and_count_as_unknown() {
        for input in [
            "LicenseRef-Custom",
            "DocumentRef-spdx-tool-1.2:LicenseRef-MIT-Style-2",
            "MIT OR LicenseRef-Custom",
        ] {
            let normalised = normalise(input);
            assert_eq!(normalised.state, State::Unknown, "{input}");
            assert!(normalised.expression.is_some(), "{input}");
        }
        assert_eq!(normalise("LicenseRef-Custom").ids, ["LicenseRef-Custom"]);
    }

    #[test]
    fn garbage_is_unknown_and_keeps_the_input_as_text() {
        for input in [
            "MIT OR",
            "(MIT",
            "MIT)",
            "OR MIT",
            "MIT MIT",
            "MIT; Apache",
            "https:",
            "<script>",
            "Ünlicense",
        ] {
            let normalised = normalise(input);
            assert_eq!(normalised.state, State::Unknown, "{input}");
            assert_eq!(normalised.expression, None, "{input}");
            assert_eq!(normalised.text, input, "{input}");
        }
    }

    #[test]
    fn empty_and_whitespace_only_input_has_state_none() {
        for input in ["", "   ", "\t\n"] {
            let normalised = normalise(input);
            assert_eq!(normalised.state, State::None, "{input:?}");
            assert_eq!(normalised.expression, None, "{input:?}");
            assert!(normalised.ids.is_empty(), "{input:?}");
            assert_eq!(normalised.text, "", "{input:?}");
        }
    }

    #[test]
    fn input_over_one_kibibyte_is_unknown() {
        let input = format!("MIT OR {}", "Apache-2.0 OR ".repeat(100));
        assert!(input.len() > parse::MAX_LENGTH);
        assert_eq!(normalise(&input).state, State::Unknown);
    }

    #[test]
    fn nesting_of_sixteen_levels_parses_and_seventeen_is_unknown() {
        assert_eq!(normalise(&nested(16)).state, State::Spdx);
        assert_eq!(normalise(&nested(16)).text, "MIT");
        assert_eq!(normalise(&nested(17)).state, State::Unknown);
        assert_eq!(normalise(&nested(17)).expression, None);
    }

    #[test]
    fn unclosed_parentheses_are_unknown_without_overflowing_the_stack() {
        assert_eq!(normalise(&"(".repeat(1000)).state, State::Unknown);
    }
}
