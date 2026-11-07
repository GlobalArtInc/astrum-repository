use std::cmp::Ordering;

use super::{Ecosystem, compare};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeKind {
    Ecosystem,
    Semver,
    Git,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event<'a> {
    Introduced(&'a str),
    Fixed(&'a str),
    LastAffected(&'a str),
    Limit(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range<'a> {
    pub kind: RangeKind,
    pub events: &'a [Event<'a>],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    pub affected: bool,
    pub fixed: Option<String>,
    pub skipped: usize,
}

const MAX_EVENTS: usize = 4096;

struct Span<'a> {
    kind: RangeKind,
    start: Option<&'a str>,
    end: Option<&'a str>,
    inclusive: bool,
    fix: bool,
}

struct Point<'a> {
    group: usize,
    text: &'a str,
    at: Option<&'a str>,
    event: Event<'a>,
}

struct Open<'a> {
    start: Option<&'a str>,
    group: usize,
}

pub fn affected(
    ecosystem: Ecosystem,
    version: &str,
    ranges: &[Range<'_>],
    versions: &[&str],
) -> Option<String> {
    let verdict = affected_detailed(ecosystem, version, ranges, versions);
    if verdict.affected {
        Some(verdict.fixed.unwrap_or_default())
    } else {
        None
    }
}

pub fn affected_detailed(
    ecosystem: Ecosystem,
    version: &str,
    ranges: &[Range<'_>],
    versions: &[&str],
) -> Verdict {
    if compare(ecosystem, version, version).is_none() {
        return Verdict::default();
    }
    let mut skipped = versions
        .iter()
        .filter(|listed| compare(ecosystem, listed, listed).is_none())
        .count();
    let mut spans = Vec::new();
    for range in ranges {
        let (found, dropped) = intervals(ecosystem, range);
        skipped += dropped;
        spans.extend(found);
    }
    let mut matched = versions
        .iter()
        .any(|listed| compare(ecosystem, version, listed) == Some(Ordering::Equal));
    let mut largest: Option<&str> = None;
    for span in &spans {
        if !covers(ecosystem, version, span) {
            continue;
        }
        matched = true;
        if span.fix
            && let Some(end) = span.end
            && largest.is_none_or(|best| {
                order(ecosystem, span.kind, end, best) == Some(Ordering::Greater)
            })
        {
            largest = Some(end);
        }
    }
    if !matched {
        return Verdict {
            affected: false,
            fixed: None,
            skipped,
        };
    }
    let fixed = largest
        .filter(|end| {
            !spans.iter().any(|span| covers(ecosystem, end, span))
                && !versions
                    .iter()
                    .any(|listed| compare(ecosystem, end, listed) == Some(Ordering::Equal))
        })
        .map(str::to_owned);
    Verdict {
        affected: true,
        fixed,
        skipped,
    }
}

fn order(ecosystem: Ecosystem, kind: RangeKind, left: &str, right: &str) -> Option<Ordering> {
    let comparator = match (kind, ecosystem) {
        (RangeKind::Semver, Ecosystem::Go) => Ecosystem::Go,
        (RangeKind::Semver, _) => Ecosystem::CratesIo,
        (RangeKind::Ecosystem | RangeKind::Git, _) => ecosystem,
    };
    compare(comparator, left, right)
}

fn is_beginning(ecosystem: Ecosystem, text: &str) -> bool {
    text == "0" || (ecosystem == Ecosystem::Go && text == "v0")
}

fn intervals<'a>(ecosystem: Ecosystem, range: &Range<'a>) -> (Vec<Span<'a>>, usize) {
    if range.kind == RangeKind::Git {
        return (Vec::new(), 0);
    }
    if range.events.len() > MAX_EVENTS {
        return (Vec::new(), 1);
    }
    let mut skipped = 0;
    let mut usable = Vec::with_capacity(range.events.len());
    let mut group_of = Vec::with_capacity(range.events.len());
    let mut closers: Vec<Option<usize>> = vec![None];
    let mut introduced = vec![0usize];
    let mut usable_introduced = vec![0usize];
    for (index, event) in range.events.iter().enumerate() {
        let (text, closer) = match *event {
            Event::Introduced(text) => (text, false),
            Event::Fixed(text) | Event::LastAffected(text) | Event::Limit(text) => (text, true),
        };
        let ok = (!closer && is_beginning(ecosystem, text))
            || order(ecosystem, range.kind, text, text).is_some();
        usable.push(ok);
        if !ok {
            skipped += 1;
        }
        let group = closers.len() - 1;
        group_of.push(group);
        if closer {
            closers[group] = Some(index);
            closers.push(None);
            introduced.push(0);
            usable_introduced.push(0);
        } else {
            introduced[group] += 1;
            if ok {
                usable_introduced[group] += 1;
            }
        }
    }

    let mut dropped = vec![false; closers.len()];
    let mut neutral = vec![false; range.events.len()];
    for (group, closer) in closers.iter().enumerate() {
        let Some(closer) = *closer else { continue };
        if !usable[closer] && usable_introduced[group] > 0 {
            dropped[group] = true;
            skipped += 1;
        } else if usable[closer] && introduced[group] > 0 && usable_introduced[group] == 0 {
            neutral[closer] = true;
            skipped += 1;
        }
    }

    let mut points = Vec::with_capacity(range.events.len());
    for (index, event) in range.events.iter().enumerate() {
        if !usable[index] || neutral[index] {
            continue;
        }
        let text = match *event {
            Event::Introduced(text)
            | Event::Fixed(text)
            | Event::LastAffected(text)
            | Event::Limit(text) => text,
        };
        let at = if matches!(event, Event::Introduced(_)) && is_beginning(ecosystem, text) {
            None
        } else {
            Some(text)
        };
        points.push(Point {
            group: group_of[index],
            text,
            at,
            event: *event,
        });
    }

    for next in 1..points.len() {
        let mut position = next;
        while position > 0
            && compare_points(
                ecosystem,
                range.kind,
                &points[position - 1],
                &points[position],
            ) == Ordering::Greater
        {
            points.swap(position - 1, position);
            position -= 1;
        }
    }

    let mut spans = Vec::new();
    let mut open: Option<Open<'a>> = None;
    let mut pending: Option<(Event<'a>, usize)> = None;
    for point in points {
        let previous = pending.take();
        match point.event {
            Event::Introduced(_) => {
                let replace = open
                    .as_ref()
                    .is_none_or(|current| dropped[current.group] && current.group != point.group);
                if replace {
                    open = None;
                    match previous {
                        Some((Event::LastAffected(version), group))
                            if group == point.group
                                && same(ecosystem, range.kind, point.at, version) =>
                        {
                            spans.push(Span {
                                kind: range.kind,
                                start: point.at,
                                end: Some(version),
                                inclusive: true,
                                fix: false,
                            });
                        }
                        Some((Event::Fixed(version) | Event::Limit(version), group))
                            if group == point.group
                                && same(ecosystem, range.kind, point.at, version) => {}
                        _ => {
                            open = Some(Open {
                                start: point.at,
                                group: point.group,
                            });
                        }
                    }
                }
            }
            closer => match open.take() {
                Some(current) => {
                    if !dropped[current.group] {
                        spans.push(Span {
                            kind: range.kind,
                            start: current.start,
                            end: Some(point.text),
                            inclusive: matches!(closer, Event::LastAffected(_)),
                            fix: matches!(closer, Event::Fixed(_)),
                        });
                    }
                }
                None => pending = Some((closer, point.group)),
            },
        }
    }
    if let Some(current) = open
        && !dropped[current.group]
    {
        spans.push(Span {
            kind: range.kind,
            start: current.start,
            end: None,
            inclusive: false,
            fix: false,
        });
    }
    (spans, skipped)
}

fn rank(point: &Point<'_>) -> u8 {
    if matches!(point.event, Event::Introduced(_)) {
        1
    } else {
        0
    }
}

fn compare_points(
    ecosystem: Ecosystem,
    kind: RangeKind,
    left: &Point<'_>,
    right: &Point<'_>,
) -> Ordering {
    cmp_at(ecosystem, kind, left.at, right.at).then(rank(left).cmp(&rank(right)))
}

fn cmp_at(
    ecosystem: Ecosystem,
    kind: RangeKind,
    left: Option<&str>,
    right: Option<&str>,
) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => order(ecosystem, kind, left, right).unwrap_or(Ordering::Equal),
    }
}

fn same(ecosystem: Ecosystem, kind: RangeKind, at: Option<&str>, version: &str) -> bool {
    at.is_some_and(|at| order(ecosystem, kind, at, version) == Some(Ordering::Equal))
}

fn covers(ecosystem: Ecosystem, version: &str, span: &Span<'_>) -> bool {
    let above_start = span.start.is_none_or(|start| {
        matches!(
            order(ecosystem, span.kind, version, start),
            Some(Ordering::Greater | Ordering::Equal)
        )
    });
    let below_end = span
        .end
        .is_none_or(|end| match order(ecosystem, span.kind, version, end) {
            Some(Ordering::Less) => true,
            Some(Ordering::Equal) => span.inclusive,
            _ => false,
        });
    above_start && below_end
}

#[cfg(test)]
mod tests {
    use crate::version::{
        Ecosystem, Event, Range, RangeKind, Verdict, affected, affected_detailed,
    };

    fn ecosystem_range<'a>(events: &'a [Event<'a>]) -> Range<'a> {
        Range {
            kind: RangeKind::Ecosystem,
            events,
        }
    }

    fn semver_range<'a>(events: &'a [Event<'a>]) -> Range<'a> {
        Range {
            kind: RangeKind::Semver,
            events,
        }
    }

    #[test]
    fn introduced_zero_with_fixed() {
        let events = [Event::Introduced("0"), Event::Fixed("1.2.0")];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "0.0.1", &ranges, &[]),
            Some("1.2.0".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.1.0", &ranges, &[]),
            Some("1.2.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Npm, "1.2.0", &ranges, &[]), None);
        assert_eq!(affected(Ecosystem::Npm, "1.3.0", &ranges, &[]), None);
    }

    #[test]
    fn last_affected_is_inclusive_and_has_no_fix() {
        let events = [Event::Introduced("1.0.0"), Event::LastAffected("1.5.0")];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(affected(Ecosystem::Npm, "0.9.0", &ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Some(String::new())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.5.0", &ranges, &[]),
            Some(String::new())
        );
        assert_eq!(affected(Ecosystem::Npm, "1.5.1", &ranges, &[]), None);
    }

    #[test]
    fn several_intervals_in_any_event_order() {
        let events = [
            Event::Fixed("2.3.0"),
            Event::Introduced("2.0.0"),
            Event::Fixed("1.0.0"),
            Event::Introduced("0"),
        ];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "0.5.0", &ranges, &[]),
            Some("1.0.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Npm, "1.5.0", &ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Npm, "2.1.0", &ranges, &[]),
            Some("2.3.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Npm, "2.3.0", &ranges, &[]), None);
    }

    #[test]
    fn fix_then_reintroduction_at_the_same_version() {
        let events = [
            Event::Introduced("0"),
            Event::Fixed("1.0.0"),
            Event::Introduced("1.0.0"),
            Event::Fixed("2.0.0"),
        ];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "0.9.0", &ranges, &[]),
            Some(String::new())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Npm, "2.0.0", &ranges, &[]), None);
    }

    #[test]
    fn last_affected_then_reintroduction_at_the_same_version() {
        let events = [
            Event::Introduced("0"),
            Event::LastAffected("1.0.0"),
            Event::Introduced("1.0.0"),
            Event::Fixed("2.0.0"),
        ];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "0.5.0", &ranges, &[]),
            Some(String::new())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.5.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Npm, "2.0.0", &ranges, &[]), None);
    }

    #[test]
    fn single_version_range_and_open_range() {
        let single = [Event::Introduced("1.0.0"), Event::LastAffected("1.0.0")];
        let ranges = [ecosystem_range(&single)];
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Some(String::new())
        );
        assert_eq!(affected(Ecosystem::Npm, "1.0.1", &ranges, &[]), None);
        let open = [Event::Introduced("1.0.0")];
        let ranges = [ecosystem_range(&open)];
        assert_eq!(
            affected(Ecosystem::Npm, "9.9.9", &ranges, &[]),
            Some(String::new())
        );
    }

    #[test]
    fn limit_bounds_the_range_without_a_fix() {
        let events = [Event::Introduced("1.0.0"), Event::Limit("3.0.0")];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(affected(Ecosystem::Npm, "0.5.0", &ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Npm, "2.0.0", &ranges, &[]),
            Some(String::new())
        );
        assert_eq!(affected(Ecosystem::Npm, "3.0.0", &ranges, &[]), None);
    }

    #[test]
    fn explicit_versions_match_by_precedence() {
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.2", &[], &["1.0.0", "1.0.2"]),
            Some(String::new())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.1", &[], &["1.0.0", "1.0.2"]),
            None
        );
        assert_eq!(
            affected(Ecosystem::PyPi, "1.0", &[], &["1.0.0"]),
            Some(String::new())
        );
    }

    #[test]
    fn overlapping_ranges_report_the_largest_fix() {
        let wide = [Event::Introduced("0"), Event::Fixed("3.0.0")];
        let narrow = [Event::Introduced("1.0.0"), Event::Fixed("2.0.0")];
        let ranges = [ecosystem_range(&wide), ecosystem_range(&narrow)];
        assert_eq!(
            affected(Ecosystem::Npm, "1.5.0", &ranges, &[]),
            Some("3.0.0".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "2.5.0", &ranges, &[]),
            Some("3.0.0".to_owned())
        );
    }

    #[test]
    fn smallest_fix_inside_another_matched_interval_is_not_returned() {
        let first = [Event::Introduced("0"), Event::Fixed("2.0.0")];
        let second = [Event::Introduced("1.0.0"), Event::Fixed("4.0.0")];
        let ranges = [ecosystem_range(&first), ecosystem_range(&second)];
        assert_eq!(
            affected(Ecosystem::Npm, "1.5.0", &ranges, &[]),
            Some("4.0.0".to_owned())
        );
    }

    #[test]
    fn largest_fix_covered_by_another_span_reports_no_fix() {
        let first = [Event::Introduced("0"), Event::Fixed("2.0.0")];
        let second = [Event::Introduced("1.0.0"), Event::Fixed("3.0.0")];
        let third = [Event::Introduced("2.8.0"), Event::Fixed("3.5.0")];
        let ranges = [
            ecosystem_range(&first),
            ecosystem_range(&second),
            ecosystem_range(&third),
        ];
        assert_eq!(
            affected(Ecosystem::Npm, "1.5.0", &ranges, &[]),
            Some(String::new())
        );
    }

    #[test]
    fn debian_ranges_use_dpkg_order() {
        let events = [Event::Introduced("0"), Event::Fixed("1.0-2")];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Debian, "1.0-1", &ranges, &[]),
            Some("1.0-2".to_owned())
        );
        assert_eq!(affected(Ecosystem::Debian, "1.0-2", &ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Debian, "1.0~rc1", &ranges, &[]),
            Some("1.0-2".to_owned())
        );
    }

    #[test]
    fn go_ranges_accept_the_v_prefix_on_either_side() {
        let plain = [Event::Introduced("0"), Event::Fixed("1.20.5")];
        let prefixed = [Event::Introduced("0"), Event::Fixed("v1.20.5")];
        let mixed = [Event::Introduced("v1.0.0"), Event::Fixed("1.20.5")];
        let both = [Event::Introduced("v0.0.0"), Event::Fixed("v1.20.5")];
        let plain_ranges = [ecosystem_range(&plain)];
        assert_eq!(
            affected(Ecosystem::Go, "v1.20.0", &plain_ranges, &[]),
            Some("1.20.5".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Go, "1.20.0", &plain_ranges, &[]),
            Some("1.20.5".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Go, "v1.20.0", &[ecosystem_range(&prefixed)], &[]),
            Some("v1.20.5".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Go, "v1.20.0", &[ecosystem_range(&mixed)], &[]),
            Some("1.20.5".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Go, "v1.20.0", &[ecosystem_range(&both)], &[]),
            Some("v1.20.5".to_owned())
        );
        assert_eq!(affected(Ecosystem::Go, "v1.20.5", &plain_ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Go, "vv1.20.0", &plain_ranges, &[]),
            None
        );
    }

    #[test]
    fn semver_ranges_use_the_semver_comparator() {
        let events = [Event::Introduced("0"), Event::Fixed("2.0.0")];
        let ranges = [semver_range(&events)];
        assert_eq!(
            affected(Ecosystem::Maven, "1.5.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Debian, "1.0.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Debian, "1.0-1", &ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Go, "v1.5.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
    }

    #[test]
    fn unparseable_inputs_are_never_affected() {
        let events = [Event::Introduced("0"), Event::Fixed("2.0.0")];
        let ranges = [ecosystem_range(&events)];
        for version in ["", "1.0", "x", "1.0.0-"] {
            assert_eq!(
                affected(Ecosystem::Npm, version, &ranges, &["x"]),
                None,
                "{version}"
            );
        }
        assert_eq!(affected(Ecosystem::Npm, "x", &[], &["x"]), None);
    }

    #[test]
    fn bad_closer_drops_only_its_interval() {
        let events = [
            Event::Introduced("0"),
            Event::Fixed("bogus"),
            Event::Introduced("3.0.0"),
            Event::Fixed("4.0.0"),
        ];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(affected(Ecosystem::Npm, "1.0.0", &ranges, &[]), None);
        assert_eq!(
            affected(Ecosystem::Npm, "3.5.0", &ranges, &[]),
            Some("4.0.0".to_owned())
        );
        assert_eq!(
            affected_detailed(Ecosystem::Npm, "3.5.0", &ranges, &[]),
            Verdict {
                affected: true,
                fixed: Some("4.0.0".to_owned()),
                skipped: 2,
            }
        );
    }

    #[test]
    fn bad_last_affected_never_opens_an_interval_to_infinity() {
        let events = [
            Event::Introduced("0"),
            Event::Fixed("1.0.0"),
            Event::Introduced("2.0.0"),
            Event::LastAffected("bogus"),
        ];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "0.5.0", &ranges, &[]),
            Some("1.0.0".to_owned())
        );
        assert_eq!(affected(Ecosystem::Npm, "2.5.0", &ranges, &[]), None);
        assert_eq!(affected(Ecosystem::Npm, "5.0.0", &ranges, &[]), None);
    }

    #[test]
    fn go_v0_marks_the_beginning() {
        let events = [Event::Introduced("v0"), Event::Fixed("v1.20.5")];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Go, "v1.20.0", &ranges, &[]),
            Some("v1.20.5".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Go, "v0.0.1", &ranges, &[]),
            Some("v1.20.5".to_owned())
        );
        assert_eq!(affected(Ecosystem::Go, "v1.20.5", &ranges, &[]), None);
        let other = [Event::Introduced("v0"), Event::Fixed("2.0.0")];
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &[ecosystem_range(&other)], &[]),
            None
        );
    }

    #[test]
    fn orphan_closer_does_not_pair_with_the_next_introduced() {
        let tails = [
            Event::Fixed("1.0.0"),
            Event::Limit("1.0.0"),
            Event::LastAffected("1.0.0"),
        ];
        for orphan in tails {
            let events = [
                Event::Introduced("bogus"),
                orphan,
                Event::Introduced("1.0.0"),
                Event::Fixed("2.0.0"),
            ];
            let ranges = [ecosystem_range(&events)];
            assert_eq!(
                affected_detailed(Ecosystem::Npm, "1.5.0", &ranges, &[]),
                Verdict {
                    affected: true,
                    fixed: Some("2.0.0".to_owned()),
                    skipped: 2,
                },
                "{orphan:?}"
            );
            assert_eq!(affected(Ecosystem::Npm, "0.9.0", &ranges, &[]), None);
            assert_eq!(affected(Ecosystem::Npm, "2.0.0", &ranges, &[]), None);
        }
    }

    #[test]
    fn orphan_closer_does_not_close_a_later_interval() {
        let events = [
            Event::Introduced("bogus"),
            Event::Fixed("1.0.0"),
            Event::Introduced("0"),
            Event::Fixed("5.0.0"),
        ];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "2.0.0", &ranges, &[]),
            Some("5.0.0".to_owned())
        );
    }

    #[test]
    fn second_introduced_inherits_the_dropped_state() {
        let events = [
            Event::Introduced("1.0.0"),
            Event::Introduced("2.0.0"),
            Event::Fixed("bogus"),
        ];
        let ranges = [ecosystem_range(&events)];
        for version in ["1.5.0", "2.5.0", "9.0.0"] {
            assert_eq!(
                affected_detailed(Ecosystem::Npm, version, &ranges, &[]),
                Verdict {
                    affected: false,
                    fixed: None,
                    skipped: 2,
                },
                "{version}"
            );
        }
    }

    #[test]
    fn fixed_listed_in_versions_is_covered() {
        let events = [Event::Introduced("0"), Event::Fixed("2.0.0")];
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Some("2.0.0".to_owned())
        );
        assert_eq!(
            affected(Ecosystem::Npm, "1.0.0", &ranges, &["2.0.0"]),
            Some(String::new())
        );
    }

    #[test]
    fn unparseable_listed_versions_are_skipped() {
        assert_eq!(
            affected_detailed(Ecosystem::Npm, "1.0.0", &[], &["bogus", "1.0.0", "x"]),
            Verdict {
                affected: true,
                fixed: None,
                skipped: 2,
            }
        );
        assert_eq!(
            affected_detailed(Ecosystem::Npm, "1.0.0", &[], &["bogus"]),
            Verdict {
                affected: false,
                fixed: None,
                skipped: 1,
            }
        );
    }

    #[test]
    fn oversized_range_is_skipped_and_the_limit_itself_is_evaluated() {
        let mut events = vec![Event::Introduced("0"), Event::Fixed("2.0.0")];
        events.resize(4096, Event::Introduced("3.0.0"));
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected_detailed(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Verdict {
                affected: true,
                fixed: Some("2.0.0".to_owned()),
                skipped: 0,
            }
        );
        events.push(Event::Introduced("3.0.0"));
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected_detailed(Ecosystem::Npm, "1.0.0", &ranges, &[]),
            Verdict {
                affected: false,
                fixed: None,
                skipped: 1,
            }
        );
    }

    #[test]
    fn many_events_are_ordered_without_panics() {
        let texts: Vec<String> = (0..52).map(|n| format!("1.{n}.0")).collect();
        let mut events = Vec::new();
        for i in (0..13).rev() {
            events.push(Event::Introduced(&texts[4 * i]));
        }
        for i in 0..13 {
            events.push(Event::Fixed(&texts[4 * i + 2]));
        }
        events.push(Event::Introduced("bogus"));
        events.push(Event::Fixed("bogus"));
        let ranges = [ecosystem_range(&events)];
        assert_eq!(
            affected_detailed(Ecosystem::Npm, "1.5.0", &ranges, &[]),
            Verdict {
                affected: true,
                fixed: Some("1.6.0".to_owned()),
                skipped: 2,
            }
        );
        assert_eq!(affected(Ecosystem::Npm, "1.7.0", &ranges, &[]), None);
    }

    #[test]
    fn git_ranges_and_empty_input_match_nothing() {
        let events = [Event::Introduced("0"), Event::Fixed("9.9.9")];
        let git = [Range {
            kind: RangeKind::Git,
            events: &events,
        }];
        assert_eq!(affected(Ecosystem::Npm, "1.0.0", &git, &[]), None);
        assert_eq!(affected(Ecosystem::Npm, "1.0.0", &[], &[]), None);
    }

    struct Xorshift(u64);

    impl Xorshift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, bound: u64) -> u64 {
            self.next() % bound
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Kind {
        Introduced,
        Fixed,
        LastAffected,
        Limit,
    }

    struct Model {
        start: u32,
        end: Option<u32>,
        inclusive: bool,
        fix: bool,
    }

    fn model_spans(events: &[(Kind, u32)]) -> Vec<Model> {
        let group = |index: usize| {
            events[..index]
                .iter()
                .filter(|(kind, _)| *kind != Kind::Introduced)
                .count()
        };
        let mut order: Vec<usize> = (0..events.len()).collect();
        order.sort_by_key(|&index| {
            let (kind, version) = events[index];
            (version, u8::from(kind == Kind::Introduced))
        });
        let mut spans = Vec::new();
        let mut open: Option<u32> = None;
        let mut pending: Option<(Kind, u32, usize)> = None;
        for index in order {
            let (kind, version) = events[index];
            let previous = pending.take();
            if kind == Kind::Introduced {
                if open.is_some() {
                    continue;
                }
                match previous {
                    Some((closer, at, owner)) if at == version && owner == group(index) => {
                        if closer == Kind::LastAffected {
                            spans.push(Model {
                                start: version,
                                end: Some(version),
                                inclusive: true,
                                fix: false,
                            });
                        }
                    }
                    _ => open = Some(version),
                }
            } else if let Some(start) = open.take() {
                spans.push(Model {
                    start,
                    end: Some(version),
                    inclusive: kind == Kind::LastAffected,
                    fix: kind == Kind::Fixed,
                });
            } else {
                pending = Some((kind, version, group(index)));
            }
        }
        if let Some(start) = open {
            spans.push(Model {
                start,
                end: None,
                inclusive: false,
                fix: false,
            });
        }
        spans
    }

    fn model_covers(span: &Model, doubled: u32) -> bool {
        doubled >= span.start * 2
            && span
                .end
                .is_none_or(|end| doubled < end * 2 || (doubled == end * 2 && span.inclusive))
    }

    fn model_verdict(cases: &[Vec<(Kind, u32)>], doubled: u32) -> Verdict {
        let spans: Vec<Model> = cases
            .iter()
            .flat_map(|events| model_spans(events))
            .collect();
        let covering: Vec<&Model> = spans
            .iter()
            .filter(|span| model_covers(span, doubled))
            .collect();
        let largest = covering
            .iter()
            .filter(|span| span.fix)
            .filter_map(|span| span.end)
            .max();
        let fixed = largest
            .filter(|end| !spans.iter().any(|span| model_covers(span, end * 2)))
            .map(|end| format!("{end}.0.0"));
        Verdict {
            affected: !covering.is_empty(),
            fixed,
            skipped: 0,
        }
    }

    fn probe(doubled: u32) -> String {
        if doubled.is_multiple_of(2) {
            format!("{}.0.0", doubled / 2)
        } else {
            format!("{}.5.0", doubled / 2)
        }
    }

    #[test]
    fn random_valid_ranges_match_the_reference_model() {
        let mut random = Xorshift(0x9E37_79B9_7F4A_7C15);
        let mut hit = 0;
        let mut with_fix = 0;
        let mut miss = 0;
        for case in 0..2000 {
            let mut cases = Vec::new();
            for _ in 0..1 + random.below(6) {
                let mut events = Vec::new();
                for _ in 0..1 + random.below(8) {
                    let kind = match random.below(4) {
                        0 | 1 => Kind::Introduced,
                        2 => Kind::Fixed,
                        _ if random.below(2) == 0 => Kind::LastAffected,
                        _ => Kind::Limit,
                    };
                    events.push((kind, random.below(6) as u32));
                }
                cases.push(events);
            }
            let texts: Vec<Vec<String>> = cases
                .iter()
                .map(|events| events.iter().map(|(_, v)| format!("{v}.0.0")).collect())
                .collect();
            let built: Vec<Vec<Event<'_>>> = cases
                .iter()
                .zip(&texts)
                .map(|(events, texts)| {
                    events
                        .iter()
                        .zip(texts)
                        .map(|((kind, _), text)| match kind {
                            Kind::Introduced => Event::Introduced(text),
                            Kind::Fixed => Event::Fixed(text),
                            Kind::LastAffected => Event::LastAffected(text),
                            Kind::Limit => Event::Limit(text),
                        })
                        .collect()
                })
                .collect();
            let ranges: Vec<Range<'_>> =
                built.iter().map(|events| ecosystem_range(events)).collect();
            for doubled in 0..12 {
                let text = probe(doubled);
                let expected = model_verdict(&cases, doubled);
                let actual = affected_detailed(Ecosystem::Npm, &text, &ranges, &[]);
                match (actual.affected, actual.fixed.is_some()) {
                    (false, _) => miss += 1,
                    (true, false) => hit += 1,
                    (true, true) => with_fix += 1,
                }
                assert_eq!(actual, expected, "case {case} probe {text}: {:?}", built);
            }
        }
        assert!(
            hit > 1000 && with_fix > 500 && miss > 1000,
            "{hit} {with_fix} {miss}"
        );
    }
}
