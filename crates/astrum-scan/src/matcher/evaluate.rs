use serde_json::Value;

use crate::version::{Ecosystem, Event, Range, RangeKind, affected_detailed, compare};

const EVENT_KEYS: [&str; 4] = ["introduced", "fixed", "last_affected", "limit"];

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub advisory_id: String,
    pub ranges: Value,
    pub versions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub advisory_id: String,
    pub fixed: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Evaluation {
    pub found: Vec<Found>,
    pub skipped: u64,
}

struct Parsed<'a> {
    kind: RangeKind,
    events: Vec<Event<'a>>,
}

pub fn resolve(ecosystem: &str, version: &str) -> Option<Ecosystem> {
    let known = Ecosystem::from_osv(ecosystem)?;
    compare(known, version, version)?;
    Some(known)
}

fn parse_ranges(value: &Value) -> Option<Vec<Parsed<'_>>> {
    value
        .as_array()?
        .iter()
        .map(|range| {
            let kind = match range.get("type")?.as_str()? {
                "ECOSYSTEM" => RangeKind::Ecosystem,
                "SEMVER" => RangeKind::Semver,
                "GIT" => RangeKind::Git,
                _ => return None,
            };
            let events = range
                .get("events")?
                .as_array()?
                .iter()
                .map(event)
                .collect::<Option<Vec<_>>>()?;
            Some(Parsed { kind, events })
        })
        .collect()
}

fn event(value: &Value) -> Option<Event<'_>> {
    let object = value.as_object()?;
    let mut keys = EVENT_KEYS.iter().filter(|key| object.contains_key(**key));
    let key = keys.next()?;
    if keys.next().is_some() || object.len() != 1 {
        return None;
    }
    let text = object.get(*key)?.as_str()?;
    match *key {
        "introduced" => Some(Event::Introduced(text)),
        "fixed" => Some(Event::Fixed(text)),
        "last_affected" => Some(Event::LastAffected(text)),
        _ => Some(Event::Limit(text)),
    }
}

pub fn evaluate(ecosystem: Ecosystem, version: &str, candidates: &[Candidate]) -> Evaluation {
    let mut evaluation = Evaluation::default();
    for candidate in candidates {
        let Some(parsed) = parse_ranges(&candidate.ranges) else {
            evaluation.skipped += 1;
            continue;
        };
        let ranges: Vec<Range<'_>> = parsed
            .iter()
            .map(|range| Range {
                kind: range.kind,
                events: &range.events,
            })
            .collect();
        let listed: Vec<&str> = candidate.versions.iter().map(String::as_str).collect();
        let verdict = affected_detailed(ecosystem, version, &ranges, &listed);
        if verdict.affected {
            evaluation.found.push(Found {
                advisory_id: candidate.advisory_id.clone(),
                fixed: verdict.fixed.filter(|fixed| !fixed.is_empty()),
            });
        } else if verdict.skipped > 0 {
            evaluation.skipped += 1;
        }
    }
    evaluation
        .found
        .sort_by(|a, b| a.advisory_id.cmp(&b.advisory_id));
    evaluation
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn candidate(id: &str, ranges: Value, versions: &[&str]) -> Candidate {
        Candidate {
            advisory_id: id.to_owned(),
            ranges,
            versions: versions.iter().map(|v| (*v).to_owned()).collect(),
        }
    }

    fn ranges(kind: &str, events: Value) -> Value {
        json!([{"type": kind, "events": events}])
    }

    fn ids(evaluation: &Evaluation) -> Vec<&str> {
        evaluation
            .found
            .iter()
            .map(|found| found.advisory_id.as_str())
            .collect()
    }

    #[test]
    fn npm_ranges_match_between_introduced_and_fixed_and_report_the_fix() {
        let candidates = [
            candidate(
                "A",
                ranges(
                    "SEMVER",
                    json!([{"introduced": "1.0.0"}, {"fixed": "1.4.2"}]),
                ),
                &[],
            ),
            candidate(
                "B",
                ranges(
                    "SEMVER",
                    json!([{"introduced": "0"}, {"last_affected": "0.9.0"}]),
                ),
                &[],
            ),
        ];
        let inside = evaluate(Ecosystem::Npm, "1.3.0", &candidates);
        assert_eq!(ids(&inside), ["A"]);
        assert_eq!(inside.found[0].fixed.as_deref(), Some("1.4.2"));
        assert!(
            evaluate(Ecosystem::Npm, "1.4.2", &candidates)
                .found
                .is_empty()
        );
        assert!(
            evaluate(Ecosystem::Npm, "0.9.1", &candidates)
                .found
                .is_empty()
        );
        let last = evaluate(Ecosystem::Npm, "0.9.0", &candidates);
        assert_eq!(ids(&last), ["B"]);
        assert_eq!(last.found[0].fixed, None);
    }

    #[test]
    fn maven_matches_by_range_and_by_the_explicit_version_list() {
        let candidates = [
            candidate(
                "R",
                ranges(
                    "ECOSYSTEM",
                    json!([{"introduced": "2.0"}, {"fixed": "2.0.5"}]),
                ),
                &[],
            ),
            candidate("L", json!([]), &["3.1", "3.2.RELEASE"]),
        ];
        assert_eq!(
            ids(&evaluate(Ecosystem::Maven, "2.0.4", &candidates)),
            ["R"]
        );
        assert!(
            evaluate(Ecosystem::Maven, "2.0.5", &candidates)
                .found
                .is_empty()
        );
        assert_eq!(ids(&evaluate(Ecosystem::Maven, "3.1", &candidates)), ["L"]);
        assert!(
            evaluate(Ecosystem::Maven, "3.3", &candidates)
                .found
                .is_empty()
        );
    }

    #[test]
    fn malformed_ranges_skip_the_advisory_and_never_match_everything() {
        let candidates = [
            candidate("NOT-AN-ARRAY", json!({"type": "SEMVER"}), &["1.0.0"]),
            candidate(
                "BAD-TYPE",
                ranges("BOGUS", json!([{"introduced": "0"}])),
                &[],
            ),
            candidate(
                "BAD-EVENT",
                ranges("SEMVER", json!([{"introduced": 0}])),
                &[],
            ),
            candidate(
                "TWO-KEYS",
                ranges("SEMVER", json!([{"introduced": "0", "fixed": "1.0.0"}])),
                &[],
            ),
            candidate(
                "UNORDERED",
                ranges(
                    "SEMVER",
                    json!([{"introduced": "not a version"}, {"fixed": "also not"}]),
                ),
                &[],
            ),
        ];
        let evaluation = evaluate(Ecosystem::Npm, "1.0.0", &candidates);
        assert!(evaluation.found.is_empty());
        assert_eq!(evaluation.skipped, 5);
    }

    #[test]
    fn an_unparseable_version_or_ecosystem_resolves_to_nothing() {
        assert!(resolve("npm", "1.0.0").is_some());
        assert!(resolve("npm", "").is_none());
        assert!(resolve("npm", "not.a.version").is_none());
        assert!(resolve("RubyGems", "1.0.0").is_none());
    }
}
