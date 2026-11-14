use astrum_content::inventory::LicenseFact;
use serde_json::Value;

const MAX_FREE_TEXT_CHARS: usize = 100;
const CLASSIFIER_PREFIX: &str = "License ::";

const CLASSIFIERS: &[(&str, &str)] = &[
    ("License :: OSI Approved :: MIT License", "MIT"),
    (
        "License :: OSI Approved :: Apache Software License",
        "Apache-2.0",
    ),
    (
        "License :: OSI Approved :: GNU General Public License v2 (GPLv2)",
        "GPL-2.0-only",
    ),
    (
        "License :: OSI Approved :: GNU General Public License v2 or later (GPLv2+)",
        "GPL-2.0-or-later",
    ),
    (
        "License :: OSI Approved :: GNU General Public License v3 (GPLv3)",
        "GPL-3.0-only",
    ),
    (
        "License :: OSI Approved :: GNU General Public License v3 or later (GPLv3+)",
        "GPL-3.0-or-later",
    ),
    (
        "License :: OSI Approved :: GNU Lesser General Public License v2 (LGPLv2)",
        "LGPL-2.0-only",
    ),
    (
        "License :: OSI Approved :: GNU Lesser General Public License v2 or later (LGPLv2+)",
        "LGPL-2.0-or-later",
    ),
    (
        "License :: OSI Approved :: GNU Lesser General Public License v2.1 (LGPLv2.1)",
        "LGPL-2.1-only",
    ),
    (
        "License :: OSI Approved :: GNU Lesser General Public License v2.1 or later (LGPLv2.1+)",
        "LGPL-2.1-or-later",
    ),
    (
        "License :: OSI Approved :: GNU Lesser General Public License v3 (LGPLv3)",
        "LGPL-3.0-only",
    ),
    (
        "License :: OSI Approved :: GNU Lesser General Public License v3 or later (LGPLv3+)",
        "LGPL-3.0-or-later",
    ),
    (
        "License :: OSI Approved :: GNU Affero General Public License v3",
        "AGPL-3.0-only",
    ),
    (
        "License :: OSI Approved :: GNU Affero General Public License v3 or later (AGPLv3+)",
        "AGPL-3.0-or-later",
    ),
    (
        "License :: OSI Approved :: Mozilla Public License 2.0 (MPL 2.0)",
        "MPL-2.0",
    ),
    ("License :: OSI Approved :: ISC License (ISCL)", "ISC"),
    (
        "License :: OSI Approved :: Python Software Foundation License",
        "PSF-2.0",
    ),
    (
        "License :: OSI Approved :: The Unlicense (Unlicense)",
        "Unlicense",
    ),
    (
        "License :: OSI Approved :: Eclipse Public License 1.0 (EPL-1.0)",
        "EPL-1.0",
    ),
    (
        "License :: OSI Approved :: Eclipse Public License 2.0 (EPL-2.0)",
        "EPL-2.0",
    ),
];

pub fn fact(attributes: &Value) -> Option<LicenseFact> {
    let pypi = &attributes["pypi"];
    if let Some(expression) = text(&pypi["license_expression"]) {
        return Some(fact_of(expression, "pypi.license_expression"));
    }
    if let Some(free) = text(&pypi["license"]) {
        let short = free.chars().count() <= MAX_FREE_TEXT_CHARS;
        if short && !free.contains(['\n', '\r']) {
            return Some(fact_of(free, "pypi.license"));
        }
    }
    let licences: Vec<&str> = pypi["classifiers"]
        .as_array()?
        .iter()
        .filter_map(text)
        .filter(|classifier| classifier.starts_with(CLASSIFIER_PREFIX))
        .collect();
    if licences.is_empty() {
        return None;
    }
    let mapped: Option<Vec<&str>> = licences
        .iter()
        .map(|classifier| {
            CLASSIFIERS
                .iter()
                .find(|(known, _)| known == classifier)
                .map(|(_, id)| *id)
        })
        .collect();
    let declared = match mapped {
        Some(ids) => {
            let mut unique: Vec<&str> = Vec::new();
            for id in ids {
                if !unique.contains(&id) {
                    unique.push(id);
                }
            }
            unique.join(" OR ")
        }
        None => licences.join(" OR "),
    };
    Some(fact_of(&declared, "pypi.classifier"))
}

fn text(value: &Value) -> Option<&str> {
    value
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn fact_of(declared: &str, source: &str) -> LicenseFact {
    LicenseFact {
        declared: declared.to_owned(),
        source: source.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn declared(pypi: Value) -> Option<(String, String)> {
        fact(&json!({ "pypi": pypi })).map(|fact| (fact.declared, fact.source))
    }

    #[test]
    fn a_licence_expression_wins_over_the_free_text_and_the_classifiers() {
        assert_eq!(
            declared(json!({
                "license_expression": "MIT OR Apache-2.0",
                "license": "BSD",
                "classifiers": ["License :: OSI Approved :: GNU General Public License v3 (GPLv3)"]
            })),
            Some((
                "MIT OR Apache-2.0".to_owned(),
                "pypi.license_expression".to_owned()
            ))
        );
    }

    #[test]
    fn free_text_is_used_only_when_it_is_short_and_on_one_line() {
        assert_eq!(
            declared(json!({"license": "MIT License"})),
            Some(("MIT License".to_owned(), "pypi.license".to_owned()))
        );
        let whole_text = format!("{}\n", "x".repeat(MAX_FREE_TEXT_CHARS + 1));
        assert_eq!(
            declared(json!({
                "license": whole_text,
                "classifiers": ["License :: OSI Approved :: ISC License (ISCL)"]
            })),
            Some(("ISC".to_owned(), "pypi.classifier".to_owned()))
        );
    }

    #[test]
    fn classifiers_map_to_spdx_ids_and_several_are_alternatives() {
        assert_eq!(
            declared(
                json!({"classifiers": ["Programming Language :: Python :: 3", "License :: OSI Approved :: MIT License"]})
            ),
            Some(("MIT".to_owned(), "pypi.classifier".to_owned()))
        );
        assert_eq!(
            declared(json!({"classifiers": [
                "License :: OSI Approved :: Apache Software License",
                "License :: OSI Approved :: GNU General Public License v2 or later (GPLv2+)"
            ]})),
            Some((
                "Apache-2.0 OR GPL-2.0-or-later".to_owned(),
                "pypi.classifier".to_owned()
            ))
        );
    }

    #[test]
    fn an_ambiguous_classifier_keeps_the_declared_text_and_names_no_id() {
        assert_eq!(
            declared(
                json!({"classifiers": ["License :: OSI Approved :: MIT License", "License :: OSI Approved :: BSD License"]})
            ),
            Some((
                "License :: OSI Approved :: MIT License OR License :: OSI Approved :: BSD License"
                    .to_owned(),
                "pypi.classifier".to_owned()
            ))
        );
    }

    #[test]
    fn a_package_without_any_licence_has_no_fact() {
        assert_eq!(declared(json!({"name": "probe"})), None);
        assert_eq!(
            declared(json!({"license": "  ", "classifiers": ["Topic :: Utilities"]})),
            None
        );
    }
}
