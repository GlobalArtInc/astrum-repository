use serde_json::Value;

use crate::verdict::{State, clean};

pub const UNSCANNED: &str = "the component has not been scanned yet";
pub const UNREADABLE: &str = "the policy cannot be read";

const FALLBACK: &str = "the component is blocked";

pub fn blocks(state: &str, blocked: bool) -> bool {
    let known = [State::Pass, State::Report, State::Unscanned]
        .into_iter()
        .any(|known| known.as_str() == state);
    !known || blocked
}

fn text<'a>(item: &'a Value, key: &str) -> Option<&'a str> {
    item.get(key).and_then(Value::as_str)
}

fn finding(item: &Value) -> Option<String> {
    if item.get("blocking").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let mut line = format!(
        "{}: {}",
        clean(text(item, "advisory")?),
        clean(text(item, "severity").unwrap_or("unknown"))
    );
    if item.get("malicious").and_then(Value::as_bool) == Some(true) {
        line.push_str(", malicious");
    }
    if let Some(package) = text(item, "package") {
        line.push_str(" in ");
        line.push_str(&clean(package));
    }
    if let Some(fixed) = text(item, "fixed") {
        line.push_str(", fixed in ");
        line.push_str(&clean(fixed));
    }
    Some(line)
}

fn license(item: &Value) -> Option<String> {
    if item.get("blocking").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let expression = text(item, "expression").map(clean);
    let breach = text(item, "breach");
    Some(match (breach, expression) {
        (Some("undeclared"), _) => "no license is declared".to_owned(),
        (Some("not-allowed"), Some(expression)) => {
            format!("license {expression}: not on the allow list")
        }
        (Some("denied"), Some(expression)) => format!("license {expression}: on the deny list"),
        (Some("unknown"), Some(expression)) => format!("license {expression}: not recognised"),
        (Some("unknown"), None) => "the license is not recognised".to_owned(),
        (_, Some(expression)) => format!("license {expression}: breaks the license rule"),
        (_, None) => "the license breaks the license rule".to_owned(),
    })
}

fn problem(item: &Value) -> Option<String> {
    text(item, "message").map(clean)
}

fn unscanned(item: &Value) -> Option<String> {
    Some(format!(
        "the component has not been scanned (scan: {})",
        clean(text(item, "scan")?)
    ))
}

fn collect(items: &[Value], kind: &str, line: fn(&Value) -> Option<String>) -> Vec<String> {
    items
        .iter()
        .filter(|item| text(item, "kind") == Some(kind))
        .filter_map(line)
        .collect()
}

pub fn details(reasons: &Value) -> String {
    let items: &[Value] = reasons.as_array().map_or(&[], Vec::as_slice);
    let mut lines = collect(items, "vulnerability", finding);
    lines.extend(collect(items, "license", license));
    if lines.is_empty() {
        lines = collect(items, "policy-error", problem);
    }
    if lines.is_empty() {
        lines = collect(items, "unscanned", unscanned);
    }
    if lines.is_empty() {
        lines.push(FALLBACK.to_owned());
    }
    lines.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_the_known_states_pass_and_only_when_the_flag_agrees() {
        for state in ["pass", "report", "unscanned"] {
            assert!(!blocks(state, false), "{state}");
            assert!(blocks(state, true), "{state}");
        }
        for state in ["block", "error", "", "Pass", "from-the-future"] {
            assert!(blocks(state, false), "{state}");
            assert!(blocks(state, true), "{state}");
        }
    }

    #[test]
    fn the_text_lists_blocking_findings_and_leaves_out_excepted_ones() {
        let reasons = json!([
            {"kind": "vulnerability", "advisory": "GHSA-1", "severity": "high",
             "malicious": false, "package": "npm/lib@1.0.0", "fixed": "1.0.1", "blocking": true},
            {"kind": "vulnerability", "advisory": "GHSA-2", "severity": "critical",
             "malicious": false, "package": "npm/lib@1.0.0", "blocking": false,
             "exception": {"id": "x", "reason": "accepted"}},
            {"kind": "vulnerability", "advisory": "MAL-3", "severity": "critical",
             "malicious": true, "package": "npm/evil@2", "blocking": true},
            {"kind": "exception", "id": "x", "reason": "accepted", "findings": 1},
            {"kind": "unscanned", "scan": "failed"}
        ]);
        assert_eq!(
            details(&reasons),
            "GHSA-1: high in npm/lib@1.0.0, fixed in 1.0.1; MAL-3: critical, malicious in npm/evil@2"
        );
    }

    #[test]
    fn without_a_blocking_finding_the_text_names_the_error_or_the_scan_state() {
        let broken =
            json!([{"kind": "policy-error", "message": "the content policy \"p\" does not exist"}]);
        assert_eq!(details(&broken), "the content policy \"p\" does not exist");
        let waiting = json!([{"kind": "unscanned", "scan": "missing"}]);
        assert_eq!(
            details(&waiting),
            "the component has not been scanned (scan: missing)"
        );
        for nothing in [
            json!([]),
            json!(null),
            json!({"kind": "vulnerability"}),
            json!([42]),
        ] {
            assert_eq!(details(&nothing), "the component is blocked");
        }
    }

    #[test]
    fn license_failures_join_the_findings_and_only_the_blocking_ones_are_named() {
        let reasons = json!([
            {"kind": "vulnerability", "advisory": "GHSA-1", "severity": "high",
             "malicious": false, "package": "npm/lib@1.0.0", "blocking": true},
            {"kind": "license", "expression": "MIT AND GPL-3.0-only", "breach": "denied",
             "blocking": true},
            {"kind": "license", "expression": "ISC", "breach": "not-allowed", "blocking": true},
            {"kind": "license", "expression": "LicenseRef-Own", "breach": "unknown",
             "blocking": true},
            {"kind": "license", "expression": "MIT", "breach": "denied", "blocking": false,
             "exception": {"id": "x", "reason": "accepted"}},
            {"kind": "license", "breach": "undeclared", "blocking": false}
        ]);
        assert_eq!(
            details(&reasons),
            "GHSA-1: high in npm/lib@1.0.0; license MIT AND GPL-3.0-only: on the deny list; \
             license ISC: not on the allow list; license LicenseRef-Own: not recognised"
        );
        let lone = json!([
            {"kind": "license", "breach": "undeclared", "blocking": true},
            {"kind": "unscanned", "scan": "missing"}
        ]);
        assert_eq!(details(&lone), "no license is declared");
        let hidden = json!([
            {"kind": "license", "expression": "GPL\n-3.0\u{202E}", "breach": "denied",
             "blocking": true}
        ]);
        assert_eq!(details(&hidden), "license GPL-3.0: on the deny list");
    }

    #[test]
    fn the_text_is_cleaned_again_whatever_the_stored_reasons_hold() {
        let dirty = json!([{"kind": "vulnerability", "advisory": "GH\nSA\u{202E}-1\u{200B}",
            "severity": "high", "package": "npm/a\r@1", "blocking": true}]);
        assert_eq!(details(&dirty), "GHSA-1: high in npm/a@1");
        let wide = json!([{"kind": "policy-error", "message": "\u{1F600}\u{200B}".repeat(400)}]);
        assert_eq!(
            details(&wide).chars().count(),
            crate::verdict::MAX_REASON_TEXT
        );
    }
}
