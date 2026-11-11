use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::exception::{Exception, Rule};
use crate::policy::{Malicious, Mode, Policy, Severity, Unscanned};

pub const MAX_FINDING_REASONS: usize = 200;
pub const MAX_REASON_TEXT: usize = 256;

fn unsafe_in_text(ch: char) -> bool {
    ch.is_control()
        || matches!(
            ch,
            '\u{061C}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
                | '\u{2028}'
                | '\u{2029}'
        )
}

pub fn clean(text: &str) -> String {
    let mut kept = text.chars().filter(|ch| !unsafe_in_text(*ch));
    let mut out: String = kept.by_ref().take(MAX_REASON_TEXT).collect();
    if kept.next().is_some() {
        out = out.chars().take(MAX_REASON_TEXT - 1).collect();
        out.push('\u{2026}');
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Pass,
    Report,
    Block,
    Unscanned,
    Error,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Pass => "pass",
            State::Report => "report",
            State::Block => "block",
            State::Unscanned => "unscanned",
            State::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
    Missing,
    Scanned,
    Matching,
    Incomplete,
    Unsupported,
    Failed,
    NoData,
    Unrecognised,
}

impl Scan {
    pub fn parse(state: Option<&str>) -> Self {
        match state {
            None => Scan::Missing,
            Some("scanned") => Scan::Scanned,
            Some("incomplete") => Scan::Incomplete,
            Some("unsupported") => Scan::Unsupported,
            Some("failed") => Scan::Failed,
            Some("no-data") => Scan::NoData,
            Some(_) => Scan::Unrecognised,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Scan::Missing => "missing",
            Scan::Scanned => "scanned",
            Scan::Matching => "matching",
            Scan::Incomplete => "incomplete",
            Scan::Unsupported => "unsupported",
            Scan::Failed => "failed",
            Scan::NoData => "no-data",
            Scan::Unrecognised => "unrecognised",
        }
    }
}

pub fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "critical",
        Severity::High => "high",
        Severity::Medium => "medium",
        Severity::Low => "low",
        Severity::Unknown => "unknown",
    }
}

pub fn parse_severity(text: &str) -> Severity {
    match text {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        "low" => Severity::Low,
        _ => Severity::Unknown,
    }
}

fn rank(severity: Severity) -> u8 {
    match severity {
        Severity::Critical => 4,
        Severity::High => 3,
        Severity::Medium => 2,
        Severity::Low => 1,
        Severity::Unknown => 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub advisory: String,
    pub aliases: Vec<String>,
    pub severity: Severity,
    pub malicious: bool,
    pub package: String,
    pub fixed: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Target {
    pub format: String,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub digest: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
    pub unknown: i32,
}

impl Counts {
    fn add(&mut self, severity: Severity) {
        let slot = match severity {
            Severity::Critical => &mut self.critical,
            Severity::High => &mut self.high,
            Severity::Medium => &mut self.medium,
            Severity::Low => &mut self.low,
            Severity::Unknown => &mut self.unknown,
        };
        *slot += 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Applied {
    pub id: Uuid,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Reason {
    Vulnerability {
        advisory: String,
        aliases: Vec<String>,
        severity: Severity,
        malicious: bool,
        package: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        fixed: Option<String>,
        blocking: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        exception: Option<Applied>,
    },
    Exception {
        id: Uuid,
        reason: String,
        findings: usize,
    },
    Unscanned {
        scan: String,
    },
    WouldBlock,
    PolicyError {
        message: String,
    },
    Truncated {
        omitted: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub blocked: bool,
    pub state: State,
    pub counts: Counts,
    pub max_severity: Option<Severity>,
    pub reasons: Vec<Reason>,
    pub policy_revision: i64,
}

pub struct Input<'a> {
    pub policy: &'a Policy,
    pub exceptions: &'a [Exception],
    pub target: &'a Target,
    pub scan: Scan,
    pub findings: &'a [Finding],
    pub now: DateTime<Utc>,
}

pub fn failed(revision: i64, message: impl Into<String>) -> Verdict {
    Verdict {
        blocked: true,
        state: State::Error,
        counts: Counts::default(),
        max_severity: None,
        reasons: vec![Reason::PolicyError {
            message: clean(&message.into()),
        }],
        policy_revision: revision,
    }
}

fn applies(exception: &Exception, policy: &str, now: DateTime<Utc>) -> bool {
    exception.rule == Rule::Vulnerability
        && exception
            .policy
            .as_deref()
            .is_none_or(|name| name == policy)
        && exception.expires_at.is_none_or(|at| at > now)
}

fn same(wanted: &Option<String>, actual: &str) -> bool {
    wanted.as_deref().is_none_or(|text| text == actual)
}

fn covers(exception: &Exception, target: &Target, finding: &Finding) -> bool {
    if exception.advisory.is_none() && exception.name.is_none() && exception.digest.is_none() {
        return false;
    }
    let advisory = exception.advisory.as_deref().is_none_or(|wanted| {
        wanted.eq_ignore_ascii_case(&finding.advisory)
            || finding
                .aliases
                .iter()
                .any(|alias| wanted.eq_ignore_ascii_case(alias))
    });
    let digest = exception
        .digest
        .as_deref()
        .is_none_or(|wanted| target.digest.as_deref() == Some(wanted));
    advisory
        && digest
        && same(&exception.format, &target.format)
        && same(&exception.namespace, &target.namespace)
        && same(&exception.name, &target.name)
        && same(&exception.version, &target.version)
}

fn reaches(finding: &Finding, rule: &crate::policy::VulnerabilityRule) -> bool {
    (finding.malicious && rule.malicious == Malicious::Block)
        || rule
            .severity
            .is_some_and(|threshold| rank(finding.severity) >= rank(threshold))
}

pub fn evaluate(input: &Input) -> Option<Verdict> {
    let rule = &input.policy.vulnerabilities;
    if rule.mode == Mode::Off {
        return None;
    }
    let mut counts = Counts::default();
    let mut max: Option<Severity> = None;
    let mut listed: Vec<(&Finding, bool, Option<&Exception>)> = Vec::new();
    let mut used: Vec<(&Exception, usize)> = Vec::new();
    for finding in input.findings {
        counts.add(finding.severity);
        if max.is_none_or(|top| rank(finding.severity) > rank(top)) {
            max = Some(finding.severity);
        }
        let excepted = input
            .exceptions
            .iter()
            .filter(|exception| applies(exception, &input.policy.name, input.now))
            .find(|exception| covers(exception, input.target, finding));
        let blocking = excepted.is_none() && reaches(finding, rule);
        if let Some(exception) = excepted {
            match used.iter_mut().find(|(known, _)| known.id == exception.id) {
                Some((_, findings)) => *findings += 1,
                None => used.push((exception, 1)),
            }
        }
        if blocking || excepted.is_some() {
            listed.push((finding, blocking, excepted));
        }
    }
    let by_findings = listed.iter().any(|(_, blocking, _)| *blocking);
    listed.sort_by(|a, b| {
        (!a.1, rank(b.0.severity), &a.0.advisory, &a.0.package).cmp(&(
            !b.1,
            rank(a.0.severity),
            &b.0.advisory,
            &b.0.package,
        ))
    });
    let omitted = listed.len().saturating_sub(MAX_FINDING_REASONS);
    let mut reasons: Vec<Reason> = listed
        .into_iter()
        .take(MAX_FINDING_REASONS)
        .map(|(finding, blocking, exception)| Reason::Vulnerability {
            advisory: clean(&finding.advisory),
            aliases: finding.aliases.iter().map(|alias| clean(alias)).collect(),
            severity: finding.severity,
            malicious: finding.malicious,
            package: clean(&finding.package),
            fixed: finding.fixed.as_deref().map(clean),
            blocking,
            exception: exception.map(|exception| Applied {
                id: exception.id,
                reason: clean(&exception.reason),
            }),
        })
        .collect();
    if omitted > 0 {
        reasons.push(Reason::Truncated { omitted });
    }
    for (exception, findings) in used {
        reasons.push(Reason::Exception {
            id: exception.id,
            reason: clean(&exception.reason),
            findings,
        });
    }
    let unscanned = input.scan != Scan::Scanned;
    if unscanned {
        reasons.push(Reason::Unscanned {
            scan: input.scan.as_str().to_owned(),
        });
    }
    let by_scan = unscanned && rule.unscanned == Unscanned::Block;
    let would_block = by_findings || by_scan;
    let (blocked, state) = match rule.mode {
        Mode::Block if by_findings => (true, State::Block),
        Mode::Block if unscanned => (by_scan, State::Unscanned),
        Mode::Block => (false, State::Pass),
        _ if would_block => (false, State::Report),
        _ if unscanned => (false, State::Unscanned),
        _ => (false, State::Pass),
    };
    if rule.mode == Mode::Report && would_block {
        reasons.push(Reason::WouldBlock);
    }
    Some(Verdict {
        blocked,
        state,
        counts,
        max_severity: max,
        reasons,
        policy_revision: input.policy.revision,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{LicenseRule, SignatureRule, VulnerabilityRule};
    use chrono::Duration;

    fn policy(mode: Mode, severity: Option<Severity>) -> Policy {
        Policy {
            name: "base".to_owned(),
            notes: String::new(),
            vulnerabilities: VulnerabilityRule {
                mode,
                severity,
                ..VulnerabilityRule::default()
            },
            licenses: LicenseRule::default(),
            signatures: SignatureRule::default(),
            revision: 3,
        }
    }

    fn finding(advisory: &str, severity: Severity) -> Finding {
        Finding {
            advisory: advisory.to_owned(),
            aliases: vec![format!("CVE-{advisory}")],
            severity,
            malicious: false,
            package: "npm/lib@1.0.0".to_owned(),
            fixed: Some("1.0.1".to_owned()),
        }
    }

    fn target() -> Target {
        Target {
            format: "npm".to_owned(),
            namespace: String::new(),
            name: "lib".to_owned(),
            version: "1.0.0".to_owned(),
            digest: None,
        }
    }

    fn exception(advisory: Option<&str>) -> Exception {
        Exception {
            id: Uuid::new_v4(),
            policy: None,
            rule: Rule::Vulnerability,
            advisory: advisory.map(str::to_owned),
            license: None,
            format: None,
            namespace: None,
            name: None,
            version: None,
            digest: None,
            reason: "accepted".to_owned(),
            expires_at: None,
            created_by: "tester".to_owned(),
            created_at: Utc::now(),
        }
    }

    fn judge(
        policy: &Policy,
        scan: Scan,
        findings: &[Finding],
        exceptions: &[Exception],
    ) -> Option<Verdict> {
        evaluate(&Input {
            policy,
            exceptions,
            target: &target(),
            scan,
            findings,
            now: Utc::now(),
        })
    }

    const LEVELS: [Severity; 5] = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Unknown,
    ];

    #[test]
    fn a_finding_blocks_when_it_reaches_the_threshold() {
        for (index, threshold) in LEVELS.into_iter().enumerate() {
            for (found_index, found) in LEVELS.into_iter().enumerate() {
                let verdict = judge(
                    &policy(Mode::Block, Some(threshold)),
                    Scan::Scanned,
                    &[finding("A", found)],
                    &[],
                )
                .unwrap();
                assert_eq!(
                    verdict.blocked,
                    found_index <= index,
                    "{found:?} against {threshold:?}"
                );
            }
        }
    }

    #[test]
    fn without_a_threshold_only_malicious_packages_block_and_threshold_mode_defers_to_severity() {
        let mut malicious = finding("MAL-1", Severity::Critical);
        malicious.malicious = true;
        let plain = finding("A", Severity::Critical);
        let none = policy(Mode::Block, None);
        assert!(!judge(&none, Scan::Scanned, &[plain], &[]).unwrap().blocked);
        assert!(
            judge(&none, Scan::Scanned, &[malicious.clone()], &[])
                .unwrap()
                .blocked
        );
        let mut deferred = policy(Mode::Block, Some(Severity::Critical));
        deferred.vulnerabilities.malicious = Malicious::Threshold;
        malicious.severity = Severity::High;
        assert!(
            !judge(&deferred, Scan::Scanned, &[malicious], &[])
                .unwrap()
                .blocked
        );
    }

    #[test]
    fn a_package_that_was_not_scanned_follows_the_unscanned_setting() {
        for scan in [
            Scan::Missing,
            Scan::Incomplete,
            Scan::Unsupported,
            Scan::Failed,
            Scan::NoData,
            Scan::Unrecognised,
        ] {
            let mut allow = policy(Mode::Block, Some(Severity::High));
            let verdict = judge(&allow, scan, &[], &[]).unwrap();
            assert_eq!((verdict.blocked, verdict.state), (false, State::Unscanned));
            allow.vulnerabilities.unscanned = Unscanned::Block;
            let verdict = judge(&allow, scan, &[], &[]).unwrap();
            assert_eq!((verdict.blocked, verdict.state), (true, State::Unscanned));
        }
        let mut strict = policy(Mode::Block, Some(Severity::High));
        strict.vulnerabilities.unscanned = Unscanned::Block;
        let verdict = judge(&strict, Scan::Scanned, &[], &[]).unwrap();
        assert_eq!((verdict.blocked, verdict.state), (false, State::Pass));
        assert!(verdict.reasons.is_empty());
    }

    #[test]
    fn off_stores_nothing_and_report_never_blocks_but_says_it_would() {
        let found = [finding("A", Severity::Critical)];
        assert!(judge(&policy(Mode::Off, None), Scan::Scanned, &found, &[]).is_none());
        let verdict = judge(
            &policy(Mode::Report, Some(Severity::High)),
            Scan::Scanned,
            &found,
            &[],
        )
        .unwrap();
        assert_eq!((verdict.blocked, verdict.state), (false, State::Report));
        assert!(verdict.reasons.contains(&Reason::WouldBlock));
        let clean = judge(
            &policy(Mode::Report, Some(Severity::High)),
            Scan::Scanned,
            &[finding("B", Severity::Low)],
            &[],
        )
        .unwrap();
        assert_eq!((clean.blocked, clean.state), (false, State::Pass));
        assert!(!clean.reasons.contains(&Reason::WouldBlock));
        assert_eq!(clean.counts.low, 1);
        assert_eq!(clean.max_severity, Some(Severity::Low));
        assert_eq!(clean.policy_revision, 3);
    }

    #[test]
    fn exceptions_match_by_advisory_alias_matcher_and_digest_only_while_valid() {
        let block = policy(Mode::Block, Some(Severity::Low));
        let found = [finding("A", Severity::High)];
        let blocked = |exceptions: &[Exception]| {
            judge(&block, Scan::Scanned, &found, exceptions)
                .unwrap()
                .blocked
        };
        assert!(blocked(&[]));
        assert!(!blocked(&[exception(Some("A"))]));
        assert!(!blocked(&[exception(Some("cve-A"))]));
        assert!(blocked(&[exception(Some("B"))]));

        let mut by_name = exception(None);
        by_name.name = Some("lib".to_owned());
        by_name.version = Some("1.0.0".to_owned());
        assert!(!blocked(&[by_name.clone()]));
        by_name.version = Some("2.0.0".to_owned());
        assert!(blocked(&[by_name]));

        let mut only_format = exception(None);
        only_format.format = Some("npm".to_owned());
        assert!(blocked(&[only_format]));

        let mut by_digest = exception(None);
        by_digest.digest = Some("sha256:x".to_owned());
        assert!(blocked(&[by_digest.clone()]));
        let image = Target {
            digest: Some("sha256:x".to_owned()),
            ..target()
        };
        let verdict = evaluate(&Input {
            policy: &block,
            exceptions: &[by_digest],
            target: &image,
            scan: Scan::Scanned,
            findings: &found,
            now: Utc::now(),
        })
        .unwrap();
        assert!(!verdict.blocked);

        let mut expired = exception(Some("A"));
        expired.expires_at = Some(Utc::now() - Duration::seconds(1));
        assert!(blocked(&[expired]));
        let mut future = exception(Some("A"));
        future.expires_at = Some(Utc::now() + Duration::hours(1));
        assert!(!blocked(&[future]));

        let mut other_policy = exception(Some("A"));
        other_policy.policy = Some("other".to_owned());
        assert!(blocked(&[other_policy]));
        let mut own_policy = exception(Some("A"));
        own_policy.policy = Some("base".to_owned());
        assert!(!blocked(&[own_policy]));
        let mut other_rule = exception(Some("A"));
        other_rule.rule = Rule::License;
        assert!(blocked(&[other_rule]));
    }

    #[test]
    fn an_excepted_finding_is_listed_with_its_exception_and_still_counted() {
        let block = policy(Mode::Block, Some(Severity::Low));
        let held = exception(Some("A"));
        let verdict = judge(
            &block,
            Scan::Scanned,
            &[finding("A", Severity::High), finding("B", Severity::Medium)],
            std::slice::from_ref(&held),
        )
        .unwrap();
        assert!(verdict.blocked);
        assert_eq!((verdict.counts.high, verdict.counts.medium), (1, 1));
        let excepted = verdict.reasons.iter().any(|reason| {
            matches!(reason, Reason::Vulnerability { advisory, blocking: false, exception: Some(applied), .. }
                if advisory == "A" && applied.id == held.id && applied.reason == "accepted")
        });
        assert!(excepted);
        assert!(verdict.reasons.iter().any(|reason| matches!(reason,
            Reason::Exception { id, findings: 1, .. } if *id == held.id)));
    }

    #[test]
    fn a_policy_that_cannot_be_read_blocks_with_the_error_state() {
        let verdict = failed(0, "the policy is missing");
        assert!(verdict.blocked);
        assert_eq!(verdict.state, State::Error);
        let text = serde_json::to_value(&verdict.reasons).unwrap();
        assert_eq!(text[0]["kind"], "policy-error");
    }

    #[test]
    fn the_findings_listed_are_capped_and_the_cut_is_said() {
        let block = policy(Mode::Block, Some(Severity::Low));
        let many: Vec<Finding> = (0..MAX_FINDING_REASONS + 5)
            .map(|index| finding(&format!("A{index:04}"), Severity::High))
            .collect();
        let verdict = judge(&block, Scan::Scanned, &many, &[]).unwrap();
        assert_eq!(verdict.reasons.len(), MAX_FINDING_REASONS + 1);
        assert_eq!(
            verdict.reasons.last(),
            Some(&Reason::Truncated { omitted: 5 })
        );
        assert_eq!(verdict.counts.high as usize, many.len());
    }

    #[test]
    fn reason_text_loses_control_and_bidi_characters() {
        let dirty = "a\nb\tc\r\u{0}d\u{7f}e\u{85}f\u{061C}g\u{200E}h\u{200F}i\u{202A}j\u{202E}k\u{2066}l\u{2069}m\u{2028}n\u{2029}o";
        assert_eq!(clean(dirty), "abcdefghijklmno");
        assert_eq!(
            clean("plain \u{43f}\u{440}\u{438}\u{432}\u{435}\u{442}"),
            "plain \u{43f}\u{440}\u{438}\u{432}\u{435}\u{442}"
        );
    }

    #[test]
    fn reason_text_is_cut_at_a_character_count_with_an_ellipsis() {
        let exact = "x".repeat(MAX_REASON_TEXT);
        assert_eq!(clean(&exact), exact);
        let long = "y".repeat(MAX_REASON_TEXT + 1);
        let cut = clean(&long);
        assert_eq!(cut.chars().count(), MAX_REASON_TEXT);
        assert_eq!(cut, format!("{}\u{2026}", "y".repeat(MAX_REASON_TEXT - 1)));
        let wide = "\u{1F600}".repeat(MAX_REASON_TEXT + 40);
        let cut = clean(&wide);
        assert_eq!(cut.chars().count(), MAX_REASON_TEXT);
        assert!(cut.ends_with('\u{2026}'));
        assert!(cut.starts_with("\u{1F600}"));
        let hidden = format!("{}\n\u{202E}", "z".repeat(MAX_REASON_TEXT));
        assert_eq!(clean(&hidden), "z".repeat(MAX_REASON_TEXT));
    }

    #[test]
    fn every_text_a_verdict_carries_is_cleaned() {
        let block = policy(Mode::Block, Some(Severity::Low));
        let mut held = exception(Some("A"));
        held.reason = format!("line\nbreak {}", "r".repeat(400));
        let mut hit = finding("A", Severity::High);
        hit.package = "npm/li\u{202E}b@1\n".to_owned();
        hit.fixed = Some(format!("1.0\r\n{}", "9".repeat(300)));
        let verdict = judge(&block, Scan::Scanned, &[hit], std::slice::from_ref(&held)).unwrap();
        let texts = serde_json::to_string(&verdict.reasons).unwrap();
        assert!(!texts.contains("\\n") && !texts.contains("\\r") && !texts.contains("202e"));
        for reason in &verdict.reasons {
            match reason {
                Reason::Vulnerability {
                    package,
                    fixed,
                    exception: Some(applied),
                    ..
                } => {
                    assert_eq!(package, "npm/lib@1");
                    assert_eq!(fixed.as_ref().unwrap().chars().count(), MAX_REASON_TEXT);
                    assert_eq!(applied.reason.chars().count(), MAX_REASON_TEXT);
                }
                Reason::Exception { reason, .. } => {
                    assert_eq!(reason.chars().count(), MAX_REASON_TEXT)
                }
                _ => {}
            }
        }
        let broken = failed(1, format!("bad\n{}", "m".repeat(500)));
        match &broken.reasons[0] {
            Reason::PolicyError { message } => {
                assert_eq!(message.chars().count(), MAX_REASON_TEXT);
                assert!(!message.contains('\n'));
            }
            other => panic!("unexpected reason {other:?}"),
        }
    }
}
