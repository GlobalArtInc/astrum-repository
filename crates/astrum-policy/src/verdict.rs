use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::exception::{Exception, Rule};
use crate::license::{self, Expression};
use crate::policy::{
    LicenseList, LicenseRule, Malicious, Mode, Policy, Severity, UnknownLicense, Unscanned,
};

pub const MAX_FINDING_REASONS: usize = 200;
pub const MAX_REASON_TEXT: usize = 256;

const HIDDEN: [(char, char); 19] = [
    ('\u{00AD}', '\u{00AD}'),
    ('\u{0600}', '\u{0605}'),
    ('\u{061C}', '\u{061C}'),
    ('\u{06DD}', '\u{06DD}'),
    ('\u{070F}', '\u{070F}'),
    ('\u{0890}', '\u{0891}'),
    ('\u{08E2}', '\u{08E2}'),
    ('\u{180E}', '\u{180E}'),
    ('\u{200B}', '\u{200F}'),
    ('\u{2028}', '\u{202E}'),
    ('\u{2060}', '\u{206F}'),
    ('\u{FEFF}', '\u{FEFF}'),
    ('\u{FFF9}', '\u{FFFB}'),
    ('\u{110BD}', '\u{110BD}'),
    ('\u{110CD}', '\u{110CD}'),
    ('\u{13430}', '\u{1343F}'),
    ('\u{1BCA0}', '\u{1BCA3}'),
    ('\u{1D173}', '\u{1D17A}'),
    ('\u{E0000}', '\u{E007F}'),
];

fn unsafe_in_text(ch: char) -> bool {
    ch.is_control()
        || HIDDEN
            .iter()
            .any(|(first, last)| (*first..=*last).contains(&ch))
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

pub fn rank(severity: Severity) -> u8 {
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
pub struct Peer {
    pub namespace: String,
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Target {
    pub format: String,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub digest: Option<String>,
    pub peers: Vec<Peer>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LicenseFact {
    pub state: license::State,
    pub expression: Option<String>,
    pub ids: Vec<String>,
}

impl LicenseFact {
    pub fn stored(state: &str, expression: Option<String>, ids: Vec<String>) -> Self {
        let state = license::State::parse(state).unwrap_or_else(|| {
            tracing::warn!(
                state,
                "a stored license state is not known to this version, so the license counts as unknown"
            );
            license::State::Unknown
        });
        Self {
            state,
            expression,
            ids,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Breach {
    NotAllowed,
    Denied,
    Unknown,
    Undeclared,
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
    License {
        #[serde(skip_serializing_if = "Option::is_none")]
        expression: Option<String>,
        breach: Breach,
        blocking: bool,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        exceptions: Vec<Applied>,
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
    pub license_state: Option<license::State>,
    pub reasons: Vec<Reason>,
    pub policy_revision: i64,
}

pub struct Input<'a> {
    pub policy: &'a Policy,
    pub exceptions: &'a [Exception],
    pub target: &'a Target,
    pub scan: Scan,
    pub findings: &'a [Finding],
    pub license: Option<&'a LicenseFact>,
    pub now: DateTime<Utc>,
}

pub fn failed(revision: i64, message: impl Into<String>) -> Verdict {
    Verdict {
        blocked: true,
        state: State::Error,
        counts: Counts::default(),
        max_severity: None,
        license_state: None,
        reasons: vec![Reason::PolicyError {
            message: clean(&message.into()),
        }],
        policy_revision: revision,
    }
}

fn applies(exception: &Exception, rule: Rule, policy: &str, now: DateTime<Utc>) -> bool {
    exception.rule == rule
        && exception
            .policy
            .as_deref()
            .is_none_or(|name| name == policy)
        && exception.expires_at.is_none_or(|at| at > now)
}

fn same(wanted: &Option<String>, actual: &str) -> bool {
    wanted.as_deref().is_none_or(|text| text == actual)
}

pub fn signature_exception<'a>(
    exceptions: &'a [Exception],
    policy: &str,
    target: &Target,
    now: DateTime<Utc>,
) -> Option<&'a Exception> {
    exceptions.iter().find(|exception| {
        applies(exception, Rule::Signature, policy, now)
            && exception.policy.is_some()
            && exception.advisory.is_none()
            && exception.license.is_none()
            && (exception.name.is_some() || exception.digest.is_some())
            && exception
                .digest
                .as_deref()
                .is_none_or(|wanted| target.digest.as_deref() == Some(wanted))
            && same(&exception.format, &target.format)
            && same(&exception.namespace, &target.namespace)
            && same(&exception.name, &target.name)
            && same(&exception.version, &target.version)
    })
}

fn digest_matches(exception: &Exception, target: &Target) -> bool {
    exception
        .digest
        .as_deref()
        .is_none_or(|wanted| target.digest.as_deref() == Some(wanted))
}

fn named(exception: &Exception, namespace: &str, name: &str, version: &str) -> bool {
    same(&exception.namespace, namespace)
        && same(&exception.name, name)
        && same(&exception.version, version)
}

fn names(exception: &Exception, target: &Target) -> bool {
    digest_matches(exception, target)
        && same(&exception.format, &target.format)
        && named(exception, &target.namespace, &target.name, &target.version)
}

fn names_or_peer(exception: &Exception, target: &Target) -> bool {
    digest_matches(exception, target)
        && same(&exception.format, &target.format)
        && (named(exception, &target.namespace, &target.name, &target.version)
            || target
                .peers
                .iter()
                .any(|peer| named(exception, &peer.namespace, &peer.name, &peer.version)))
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
    advisory && names_or_peer(exception, target)
}

fn covers_license(exception: &Exception, target: &Target, fact: Option<&LicenseFact>) -> bool {
    if exception.advisory.is_some()
        || (exception.license.is_none() && exception.name.is_none() && exception.digest.is_none())
    {
        return false;
    }
    let license = exception.license.as_deref().is_none_or(|wanted| {
        fact.is_some_and(|fact| fact.ids.iter().any(|id| license::same_id(id, wanted)))
    });
    license && names(exception, target)
}

pub fn exception_for<'a>(
    exceptions: &'a [Exception],
    policy: &str,
    target: &Target,
    finding: &Finding,
    now: DateTime<Utc>,
) -> Option<&'a Exception> {
    exceptions
        .iter()
        .filter(|exception| applies(exception, Rule::Vulnerability, policy, now))
        .find(|exception| covers(exception, target, finding))
}

fn reaches(finding: &Finding, rule: &crate::policy::VulnerabilityRule) -> bool {
    (finding.malicious && rule.malicious == Malicious::Block)
        || rule
            .severity
            .is_some_and(|threshold| rank(finding.severity) >= rank(threshold))
}

struct Part {
    blocked: bool,
    state: State,
    would_block: bool,
    reasons: Vec<Reason>,
}

impl Part {
    fn weight(&self) -> u8 {
        match (self.state, self.blocked) {
            (State::Block, _) => 4,
            (_, true) => 3,
            (State::Report, _) => 2,
            (State::Unscanned, _) => 1,
            _ => 0,
        }
    }
}

fn judge_vulnerabilities(input: &Input) -> Option<(Part, Counts, Option<Severity>)> {
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
        let excepted = exception_for(
            input.exceptions,
            &input.policy.name,
            input.target,
            finding,
            input.now,
        );
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
    let part = Part {
        blocked,
        state,
        would_block: rule.mode == Mode::Report && would_block,
        reasons,
    };
    Some((part, counts, max))
}

type Outcome = Option<(Breach, bool)>;

fn unjudged(setting: UnknownLicense, breach: Breach) -> Outcome {
    match setting {
        UnknownLicense::Allow => None,
        UnknownLicense::Report => Some((breach, false)),
        UnknownLicense::Block => Some((breach, true)),
    }
}

fn outcome(
    rule: &LicenseRule,
    state: license::State,
    expression: Option<&Expression>,
    ids: &[String],
) -> Outcome {
    match (state, expression) {
        (license::State::Spdx | license::State::Mapped, Some(expression)) => match rule.list {
            LicenseList::Allow => {
                (!license::satisfies(expression, ids)).then_some((Breach::NotAllowed, true))
            }
            LicenseList::Deny => {
                license::violates(expression, ids).then_some((Breach::Denied, true))
            }
        },
        (license::State::Unknown, Some(expression)) => match rule.list {
            LicenseList::Allow if license::satisfies(expression, ids) => None,
            LicenseList::Deny if license::violates(expression, ids) => Some((Breach::Denied, true)),
            _ => unjudged(rule.unknown, Breach::Unknown),
        },
        (license::State::None, _) => unjudged(rule.unknown, Breach::Undeclared),
        _ => unjudged(rule.unknown, Breach::Unknown),
    }
}

fn adjusted(rule: &LicenseRule, excepted: &[&Exception]) -> Vec<String> {
    let wanted = excepted
        .iter()
        .filter_map(|exception| exception.license.as_deref());
    match rule.list {
        LicenseList::Allow => rule
            .ids
            .iter()
            .map(String::as_str)
            .chain(wanted)
            .map(str::to_owned)
            .collect(),
        LicenseList::Deny => rule
            .ids
            .iter()
            .filter(|id| !wanted.clone().any(|wanted| license::same_id(id, wanted)))
            .cloned()
            .collect(),
    }
}

fn covered(expression: &Expression, excepted: &[&Exception]) -> bool {
    expression.ids().iter().all(|id| {
        excepted.iter().any(|exception| {
            exception
                .license
                .as_deref()
                .is_some_and(|wanted| license::same_id(id, wanted))
        })
    })
}

fn outlives(candidate: &Exception, known: &Exception) -> bool {
    match (candidate.expires_at, known.expires_at) {
        (None, Some(_)) => true,
        (Some(candidate), Some(known)) => candidate > known,
        _ => false,
    }
}

fn longest_lived(exceptions: Vec<&Exception>) -> Vec<&Exception> {
    let mut kept: Vec<&Exception> = Vec::new();
    for exception in exceptions {
        let wanted = exception.license.as_deref().unwrap_or_default();
        let same = kept.iter_mut().find(|known| {
            known
                .license
                .as_deref()
                .is_some_and(|id| license::same_id(id, wanted))
        });
        match same {
            Some(known) => {
                if outlives(exception, known) {
                    *known = exception;
                }
            }
            None => kept.push(exception),
        }
    }
    kept
}

struct Resolved<'a> {
    remaining: Outcome,
    used: Vec<&'a Exception>,
}

fn resolve<'a>(
    rule: &LicenseRule,
    state: license::State,
    expression: Option<&Expression>,
    first: (Breach, bool),
    applicable: &[&'a Exception],
) -> Resolved<'a> {
    if let Some(exception) = applicable
        .iter()
        .find(|exception| exception.license.is_none())
    {
        return Resolved {
            remaining: None,
            used: vec![*exception],
        };
    }
    let named = longest_lived(
        applicable
            .iter()
            .filter(|exception| exception.license.is_some())
            .copied()
            .collect(),
    );
    let Some(expression) = expression.filter(|_| !named.is_empty()) else {
        return Resolved {
            remaining: Some(first),
            used: Vec::new(),
        };
    };
    let again = outcome(rule, state, Some(expression), &adjusted(rule, &named));
    let settled = match again {
        None => true,
        Some((Breach::Unknown, _)) => covered(expression, &named),
        Some(_) => false,
    };
    let used = if settled || again != Some(first) {
        named
    } else {
        Vec::new()
    };
    Resolved {
        remaining: if settled { None } else { again },
        used,
    }
}

fn judge_licenses(input: &Input) -> Option<(Part, license::State)> {
    let rule = &input.policy.licenses;
    if rule.mode == Mode::Off {
        return None;
    }
    let fact = input.license;
    let state = fact.map_or(license::State::None, |fact| fact.state);
    let expression = fact
        .and_then(|fact| fact.expression.as_deref())
        .and_then(|text| license::normalise(text).expression);
    let mut part = Part {
        blocked: false,
        state: State::Pass,
        would_block: false,
        reasons: Vec::new(),
    };
    if let Some(first) = outcome(rule, state, expression.as_ref(), &rule.ids) {
        let applicable: Vec<&Exception> = input
            .exceptions
            .iter()
            .filter(|exception| {
                applies(exception, Rule::License, &input.policy.name, input.now)
                    && covers_license(exception, input.target, fact)
            })
            .collect();
        let resolved = resolve(rule, state, expression.as_ref(), first, &applicable);
        let (found, hard) = resolved.remaining.unwrap_or(first);
        let counts = resolved.remaining.is_some();
        let blocking = counts && hard && rule.mode == Mode::Block;
        let used = &resolved.used[..resolved.used.len().min(MAX_FINDING_REASONS)];
        part.reasons.push(Reason::License {
            expression: fact.and_then(|fact| fact.expression.as_deref()).map(clean),
            breach: found,
            blocking,
            exceptions: used
                .iter()
                .map(|exception| Applied {
                    id: exception.id,
                    reason: clean(&exception.reason),
                })
                .collect(),
        });
        for exception in used {
            part.reasons.push(Reason::Exception {
                id: exception.id,
                reason: clean(&exception.reason),
                findings: 1,
            });
        }
        if counts {
            part.blocked = blocking;
            part.would_block = hard && rule.mode == Mode::Report;
            part.state = if blocking {
                State::Block
            } else {
                State::Report
            };
        }
    }
    Some((part, state))
}

fn assemble(
    parts: Vec<Part>,
    counts: Counts,
    max_severity: Option<Severity>,
    license_state: Option<license::State>,
    policy_revision: i64,
) -> Verdict {
    let blocked = parts.iter().any(|part| part.blocked);
    let would_block = parts.iter().any(|part| part.would_block);
    let state = parts
        .iter()
        .max_by_key(|part| part.weight())
        .map_or(State::Pass, |part| part.state);
    let mut reasons: Vec<Reason> = parts.into_iter().flat_map(|part| part.reasons).collect();
    if would_block {
        reasons.push(Reason::WouldBlock);
    }
    Verdict {
        blocked,
        state,
        counts,
        max_severity,
        license_state,
        reasons,
        policy_revision,
    }
}

pub fn evaluate_license(input: &Input) -> Option<Verdict> {
    let (part, state) = judge_licenses(input)?;
    Some(assemble(
        vec![part],
        Counts::default(),
        None,
        Some(state),
        input.policy.revision,
    ))
}

pub fn evaluate_vulnerabilities(input: &Input) -> Option<Verdict> {
    let (part, counts, max_severity) = judge_vulnerabilities(input)?;
    Some(assemble(
        vec![part],
        counts,
        max_severity,
        None,
        input.policy.revision,
    ))
}

pub fn evaluate(input: &Input) -> Option<Verdict> {
    let mut parts: Vec<Part> = Vec::new();
    let (counts, max_severity) = match judge_vulnerabilities(input) {
        Some((part, counts, max)) => {
            parts.push(part);
            (counts, max)
        }
        None => (Counts::default(), None),
    };
    let license_state = match judge_licenses(input) {
        Some((part, state)) => {
            parts.push(part);
            Some(state)
        }
        None => None,
    };
    if parts.is_empty() {
        return None;
    }
    Some(assemble(
        parts,
        counts,
        max_severity,
        license_state,
        input.policy.revision,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{LicenseRule, SignatureRule, VulnerabilityRule};
    use chrono::Duration;
    use serde_json::json;

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
            peers: Vec::new(),
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
            license: None,
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
    fn an_exception_by_name_covers_a_target_through_any_of_its_peers() {
        let block = policy(Mode::Block, Some(Severity::Low));
        let found = [finding("A", Severity::High)];
        let peer = |name: &str, version: &str| Peer {
            namespace: String::new(),
            name: name.to_owned(),
            version: version.to_owned(),
        };
        let image = Target {
            format: "docker".to_owned(),
            digest: Some("sha256:x".to_owned()),
            peers: vec![peer("app", "1.0"), peer("tool", "2.0")],
            ..Target::default()
        };
        let blocked = |name: &str, version: Option<&str>| {
            let mut by_name = exception(None);
            by_name.name = Some(name.to_owned());
            by_name.version = version.map(str::to_owned);
            evaluate(&Input {
                policy: &block,
                exceptions: &[by_name],
                target: &image,
                scan: Scan::Scanned,
                findings: &found,
                license: None,
                now: Utc::now(),
            })
            .unwrap()
            .blocked
        };
        assert!(!blocked("tool", None));
        assert!(!blocked("app", Some("1.0")));
        assert!(blocked("app", Some("2.0")));
        assert!(blocked("other", None));
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
            license: None,
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

    fn signature_exception_for(
        digest: Option<&str>,
        name: Option<&str>,
        edit: impl FnOnce(&mut Exception),
    ) -> Exception {
        let mut held = exception(None);
        held.rule = Rule::Signature;
        held.policy = Some("base".to_owned());
        held.digest = digest.map(str::to_owned);
        held.name = name.map(str::to_owned);
        edit(&mut held);
        held
    }

    #[test]
    fn a_signature_exception_covers_only_the_image_it_names_for_its_own_policy() {
        let image = Target {
            format: "docker".to_owned(),
            namespace: String::new(),
            name: "app".to_owned(),
            version: "1.0".to_owned(),
            digest: Some("sha256:a".to_owned()),
            peers: Vec::new(),
        };
        let covers =
            |held: Exception| signature_exception(&[held], "base", &image, Utc::now()).is_some();
        assert!(covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |_| {}
        )));
        assert!(covers(signature_exception_for(None, Some("app"), |_| {})));
        assert!(covers(signature_exception_for(
            Some("sha256:a"),
            Some("app"),
            |held| held.version = Some("1.0".to_owned())
        )));
        assert!(!covers(signature_exception_for(
            Some("sha256:b"),
            None,
            |_| {}
        )));
        assert!(!covers(signature_exception_for(
            None,
            Some("other"),
            |_| {}
        )));
        assert!(!covers(signature_exception_for(
            None,
            Some("app"),
            |held| held.version = Some("2.0".to_owned())
        )));
        assert!(!covers(signature_exception_for(None, None, |held| {
            held.format = Some("docker".to_owned())
        })));
        assert!(!covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |held| { held.rule = Rule::Vulnerability }
        )));
        assert!(!covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |held| { held.advisory = Some("A".to_owned()) }
        )));
        assert!(!covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |held| { held.policy = Some("other".to_owned()) }
        )));
        assert!(!covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |held| { held.policy = None }
        )));
        assert!(!covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |held| { held.expires_at = Some(Utc::now() - Duration::seconds(1)) }
        )));
        assert!(covers(signature_exception_for(
            Some("sha256:a"),
            None,
            |held| {
                held.policy = Some("base".to_owned());
                held.expires_at = Some(Utc::now() + Duration::hours(1));
            }
        )));
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

    fn licensed(mode: Mode, list: LicenseList, ids: &[&str]) -> Policy {
        let mut policy = policy(Mode::Off, None);
        policy.licenses = LicenseRule {
            mode,
            list,
            ids: ids.iter().map(|id| (*id).to_owned()).collect(),
            unknown: UnknownLicense::Report,
        };
        policy
    }

    fn fact(expression: &str) -> LicenseFact {
        let normalised = license::normalise(expression);
        LicenseFact {
            state: normalised.state,
            expression: Some(normalised.text),
            ids: normalised.ids,
        }
    }

    fn undeclared(state: license::State) -> LicenseFact {
        LicenseFact {
            state,
            expression: None,
            ids: Vec::new(),
        }
    }

    fn judge_license(
        policy: &Policy,
        fact: Option<&LicenseFact>,
        exceptions: &[Exception],
    ) -> Verdict {
        evaluate(&Input {
            policy,
            exceptions,
            target: &target(),
            scan: Scan::Scanned,
            findings: &[],
            license: fact,
            now: Utc::now(),
        })
        .unwrap()
    }

    fn license_exception(license: Option<&str>) -> Exception {
        Exception {
            rule: Rule::License,
            license: license.map(str::to_owned),
            ..exception(None)
        }
    }

    fn license_reason(verdict: &Verdict) -> Option<(&Option<String>, Breach, bool)> {
        verdict.reasons.iter().find_map(|reason| match reason {
            Reason::License {
                expression,
                breach,
                blocking,
                ..
            } => Some((expression, *breach, *blocking)),
            _ => None,
        })
    }

    #[test]
    fn an_allow_list_needs_an_or_branch_whose_terms_are_all_allowed_and_ids_meet_their_equivalents()
    {
        let cases: [(&str, &[&str], bool); 6] = [
            ("MIT OR GPL-3.0-only", &["MIT"], true),
            ("MIT AND GPL-3.0-only", &["MIT"], false),
            ("MIT AND GPL-3.0-only", &["MIT", "GPL-3.0-only"], true),
            ("GPL-2.0", &["GPL-2.0-only"], true),
            ("GPL-3.0-only", &["gpl-3.0"], true),
            ("Apache-2.0", &["MIT"], false),
        ];
        for (expression, allowed, passes) in cases {
            let policy = licensed(Mode::Block, LicenseList::Allow, allowed);
            let verdict = judge_license(&policy, Some(&fact(expression)), &[]);
            assert_eq!(verdict.blocked, !passes, "{expression} against {allowed:?}");
            assert_eq!(verdict.license_state, Some(license::State::Spdx));
            match passes {
                true => assert!(verdict.reasons.is_empty(), "{expression}"),
                false => assert_eq!(
                    license_reason(&verdict),
                    Some((&Some(expression.to_owned()), Breach::NotAllowed, true))
                ),
            }
        }
    }

    #[test]
    fn a_deny_list_fails_when_every_or_branch_holds_a_denied_id() {
        let cases: [(&str, &[&str], bool); 5] = [
            ("MIT AND GPL-3.0-only", &["GPL-3.0-only"], true),
            ("MIT OR GPL-3.0-only", &["GPL-3.0-only"], false),
            ("GPL-3.0 OR GPL-3.0-only", &["GPL-3.0-only"], true),
            ("GPL-3.0", &["GPL-3.0-only"], true),
            ("MIT", &["GPL-3.0-only"], false),
        ];
        for (expression, denied, fails) in cases {
            let policy = licensed(Mode::Block, LicenseList::Deny, denied);
            let verdict = judge_license(&policy, Some(&fact(expression)), &[]);
            assert_eq!(verdict.blocked, fails, "{expression} against {denied:?}");
            if fails {
                assert_eq!(
                    license_reason(&verdict).map(|(_, breach, blocking)| (breach, blocking)),
                    Some((Breach::Denied, true))
                );
            }
        }
    }

    #[test]
    fn unknown_none_and_a_missing_row_follow_the_unknown_setting() {
        let facts = [
            (
                Some(undeclared(license::State::Unknown)),
                license::State::Unknown,
            ),
            (Some(undeclared(license::State::None)), license::State::None),
            (None, license::State::None),
        ];
        for (fact, state) in facts {
            for (setting, mode, blocked, shown) in [
                (UnknownLicense::Allow, Mode::Block, false, State::Pass),
                (UnknownLicense::Report, Mode::Block, false, State::Report),
                (UnknownLicense::Block, Mode::Block, true, State::Block),
                (UnknownLicense::Block, Mode::Report, false, State::Report),
            ] {
                let mut policy = licensed(mode, LicenseList::Deny, &["GPL-3.0-only"]);
                policy.licenses.unknown = setting;
                let verdict = judge_license(&policy, fact.as_ref(), &[]);
                let label = format!("{state:?} {setting:?} {mode:?}");
                assert_eq!(
                    (verdict.blocked, verdict.state),
                    (blocked, shown),
                    "{label}"
                );
                assert_eq!(verdict.license_state, Some(state), "{label}");
                let wanted = match state {
                    license::State::None => Breach::Undeclared,
                    _ => Breach::Unknown,
                };
                if setting != UnknownLicense::Allow {
                    assert_eq!(license_reason(&verdict).unwrap().1, wanted, "{label}");
                }
            }
        }
    }

    #[test]
    fn report_mode_never_blocks_and_says_it_would_and_off_judges_nothing() {
        let report = licensed(Mode::Report, LicenseList::Deny, &["GPL-3.0-only"]);
        let verdict = judge_license(&report, Some(&fact("GPL-3.0-only")), &[]);
        assert_eq!((verdict.blocked, verdict.state), (false, State::Report));
        assert!(!license_reason(&verdict).unwrap().2);
        assert!(verdict.reasons.contains(&Reason::WouldBlock));
        let clean = judge_license(&report, Some(&fact("MIT")), &[]);
        assert_eq!((clean.blocked, clean.state), (false, State::Pass));

        let off = licensed(Mode::Off, LicenseList::Deny, &["GPL-3.0-only"]);
        let input = Input {
            policy: &off,
            exceptions: &[],
            target: &target(),
            scan: Scan::Scanned,
            findings: &[],
            license: Some(&fact("GPL-3.0-only")),
            now: Utc::now(),
        };
        assert!(evaluate(&input).is_none());
        let vulnerable = policy(Mode::Block, Some(Severity::High));
        let verdict = judge(&vulnerable, Scan::Scanned, &[], &[]).unwrap();
        assert_eq!(verdict.license_state, None);
    }

    #[test]
    fn the_license_reason_names_the_expression_and_loses_control_characters() {
        let policy = licensed(Mode::Block, LicenseList::Deny, &["GPL-3.0-only"]);
        let mut dirty = fact("MIT AND GPL-3.0-only");
        dirty.expression = Some("MIT AND GPL-3.0-only\n\t".to_owned());
        let verdict = judge_license(&policy, Some(&dirty), &[]);
        let reasons = serde_json::to_value(&verdict.reasons).unwrap();
        assert_eq!(
            reasons,
            json!([{
                "kind": "license",
                "expression": "MIT AND GPL-3.0-only",
                "breach": "denied",
                "blocking": true
            }])
        );
        let none = judge_license(&licensed(Mode::Block, LicenseList::Deny, &[]), None, &[]);
        let reasons = serde_json::to_value(&none.reasons).unwrap();
        assert_eq!(
            reasons,
            json!([{"kind": "license", "breach": "undeclared", "blocking": false}])
        );
    }

    #[test]
    fn license_exceptions_lift_by_id_or_by_component_only_while_valid_and_for_the_rule() {
        let policy = licensed(Mode::Block, LicenseList::Deny, &["GPL-3.0-only"]);
        let found = fact("MIT AND GPL-3.0-only");
        let blocked =
            |exceptions: &[Exception]| judge_license(&policy, Some(&found), exceptions).blocked;
        assert!(blocked(&[]));

        for lifting in ["GPL-3.0-only", "gpl-3.0-only", "GPL-3.0"] {
            assert!(!blocked(&[license_exception(Some(lifting))]), "{lifting}");
        }
        for other in ["MIT", "Apache-2.0"] {
            assert!(blocked(&[license_exception(Some(other))]), "{other}");
        }

        let mut by_component = license_exception(None);
        by_component.name = Some("lib".to_owned());
        by_component.version = Some("1.0.0".to_owned());
        assert!(!blocked(&[by_component.clone()]));
        by_component.version = Some("2.0.0".to_owned());
        assert!(blocked(&[by_component]));
        assert!(blocked(&[license_exception(None)]));

        let mut narrowed = license_exception(Some("GPL-3.0-only"));
        narrowed.name = Some("other".to_owned());
        assert!(blocked(&[narrowed]));

        let mut expired = license_exception(Some("GPL-3.0-only"));
        expired.expires_at = Some(Utc::now() - Duration::seconds(1));
        assert!(blocked(&[expired]));
        let mut future = license_exception(Some("GPL-3.0-only"));
        future.expires_at = Some(Utc::now() + Duration::hours(1));
        assert!(!blocked(&[future]));

        let mut other_policy = license_exception(Some("GPL-3.0-only"));
        other_policy.policy = Some("other".to_owned());
        assert!(blocked(&[other_policy]));
        let mut other_rule = license_exception(Some("GPL-3.0-only"));
        other_rule.rule = Rule::Vulnerability;
        assert!(blocked(&[other_rule]));
        let mut advisory = license_exception(Some("GPL-3.0-only"));
        advisory.advisory = Some("GHSA-1".to_owned());
        assert!(blocked(&[advisory]));

        let held = license_exception(Some("GPL-3.0-only"));
        let verdict = judge_license(&policy, Some(&found), std::slice::from_ref(&held));
        assert_eq!((verdict.blocked, verdict.state), (false, State::Pass));
        assert!(!license_reason(&verdict).unwrap().2);
        assert!(verdict.reasons.iter().any(|reason| matches!(reason,
            Reason::Exception { id, findings: 1, .. } if *id == held.id)));

        let own = fact("LicenseRef-Own");
        let mut unknown = licensed(Mode::Block, LicenseList::Deny, &[]);
        unknown.licenses.unknown = UnknownLicense::Block;
        assert!(judge_license(&unknown, Some(&own), &[]).blocked);
        let lifted = judge_license(
            &unknown,
            Some(&own),
            &[license_exception(Some("licenseref-own"))],
        );
        assert!(!lifted.blocked);
        let unrelated = judge_license(&unknown, Some(&own), &[license_exception(Some("MIT"))]);
        assert!(unrelated.blocked);
    }

    #[test]
    fn an_id_exception_makes_the_id_acceptable_and_the_expression_is_judged_again() {
        let excepting = |policy: &Policy, expression: &str, ids: &[&str]| {
            let held: Vec<Exception> = ids.iter().map(|id| license_exception(Some(id))).collect();
            judge_license(policy, Some(&fact(expression)), &held)
        };
        let deny = licensed(
            Mode::Block,
            LicenseList::Deny,
            &["GPL-3.0-only", "AGPL-3.0-only"],
        );
        let both = "GPL-3.0-only AND AGPL-3.0-only";
        assert!(excepting(&deny, both, &[]).blocked);
        assert!(excepting(&deny, both, &["GPL-3.0-only"]).blocked);
        assert!(excepting(&deny, both, &["agpl-3.0-only"]).blocked);
        assert!(excepting(&deny, both, &["GPL-3.0-only", "MIT"]).blocked);
        let lifted = excepting(&deny, both, &["GPL-3.0-only", "AGPL-3.0-only"]);
        assert_eq!((lifted.blocked, lifted.state), (false, State::Pass));
        let cited = lifted
            .reasons
            .iter()
            .filter(|reason| matches!(reason, Reason::Exception { findings: 1, .. }))
            .count();
        assert_eq!(cited, 2);
        assert!(!excepting(&deny, "GPL-3.0-only OR MIT", &[]).blocked);
        assert!(excepting(&deny, "GPL-3.0-only OR AGPL-3.0-only", &[]).blocked);
        assert!(!excepting(&deny, "GPL-3.0-only OR AGPL-3.0-only", &["GPL-3.0-only"]).blocked);

        let allow = licensed(Mode::Block, LicenseList::Allow, &["MIT"]);
        let three = "MIT AND ISC AND BSD-3-Clause";
        assert!(excepting(&allow, three, &["ISC"]).blocked);
        assert!(excepting(&allow, three, &["ISC", "Apache-2.0"]).blocked);
        assert!(!excepting(&allow, three, &["isc", "BSD-3-Clause"]).blocked);
        assert!(excepting(&allow, "ISC OR Apache-2.0", &["BSD-3-Clause"]).blocked);
        assert!(!excepting(&allow, "ISC OR Apache-2.0", &["ISC"]).blocked);
        assert!(!excepting(&allow, "GPL-3.0 AND MIT", &["GPL-3.0-only"]).blocked);

        let blocked = excepting(&allow, three, &["ISC"]);
        assert_eq!(
            license_reason(&blocked).map(|(_, breach, blocking)| (breach, blocking)),
            Some((Breach::NotAllowed, true))
        );
        assert!(
            !blocked
                .reasons
                .iter()
                .any(|reason| matches!(reason, Reason::Exception { .. }))
        );
    }

    #[test]
    fn an_unknown_license_that_parsed_is_judged_by_the_list_before_the_unknown_setting() {
        type Case = (
            LicenseList,
            &'static [&'static str],
            UnknownLicense,
            &'static str,
            Option<(Breach, bool)>,
        );
        let cases: [Case; 12] = [
            (
                LicenseList::Deny,
                &["GPL-3.0-only"],
                UnknownLicense::Allow,
                "GPL-3.0-only AND Foo-Bar",
                Some((Breach::Denied, true)),
            ),
            (
                LicenseList::Deny,
                &["GPL-3.0-only"],
                UnknownLicense::Allow,
                "GPL-3.0-only OR Foo-Bar",
                None,
            ),
            (
                LicenseList::Deny,
                &["GPL-3.0-only"],
                UnknownLicense::Block,
                "GPL-3.0-only OR Foo-Bar",
                Some((Breach::Unknown, true)),
            ),
            (
                LicenseList::Allow,
                &["MIT"],
                UnknownLicense::Block,
                "MIT OR Foo-Bar",
                None,
            ),
            (
                LicenseList::Allow,
                &["MIT"],
                UnknownLicense::Block,
                "MIT AND Foo-Bar",
                Some((Breach::Unknown, true)),
            ),
            (
                LicenseList::Allow,
                &["MIT"],
                UnknownLicense::Allow,
                "Foo-Bar",
                None,
            ),
            (
                LicenseList::Allow,
                &["MIT"],
                UnknownLicense::Block,
                "Foo-Bar",
                Some((Breach::Unknown, true)),
            ),
            (
                LicenseList::Deny,
                &["GPL-3.0-only"],
                UnknownLicense::Report,
                "Foo-Bar",
                Some((Breach::Unknown, false)),
            ),
            (
                LicenseList::Deny,
                &["GPL-3.0-only"],
                UnknownLicense::Block,
                "Foo-Bar",
                Some((Breach::Unknown, true)),
            ),
            (
                LicenseList::Allow,
                &["LicenseRef-Own"],
                UnknownLicense::Block,
                "LicenseRef-Own",
                None,
            ),
            (
                LicenseList::Deny,
                &["LicenseRef-Bad"],
                UnknownLicense::Allow,
                "LicenseRef-Bad",
                Some((Breach::Denied, true)),
            ),
            (
                LicenseList::Deny,
                &["LicenseRef-Bad"],
                UnknownLicense::Allow,
                "LicenseRef-Own",
                None,
            ),
        ];
        for (list, ids, setting, expression, expected) in cases {
            let mut policy = licensed(Mode::Block, list, ids);
            policy.licenses.unknown = setting;
            let found = fact(expression);
            assert_eq!(found.state, license::State::Unknown, "{expression}");
            let verdict = judge_license(&policy, Some(&found), &[]);
            let label = format!("{expression} under {list:?} {ids:?} with {setting:?}");
            match expected {
                None => assert!(verdict.reasons.is_empty(), "{label}"),
                Some((breach, blocked)) => {
                    assert_eq!(verdict.blocked, blocked, "{label}");
                    assert_eq!(
                        license_reason(&verdict).map(|(_, found, _)| found),
                        Some(breach),
                        "{label}"
                    );
                }
            }
        }
    }

    #[test]
    fn an_id_exception_lifts_an_unknown_term_only_when_every_id_is_covered() {
        let mut policy = licensed(Mode::Block, LicenseList::Deny, &["GPL-3.0-only"]);
        policy.licenses.unknown = UnknownLicense::Block;
        let found = fact("GPL-3.0-only AND Foo-Bar");
        let with = |policy: &Policy, ids: &[&str]| {
            let held: Vec<Exception> = ids.iter().map(|id| license_exception(Some(id))).collect();
            judge_license(policy, Some(&found), &held)
        };
        for partial in [
            &[][..],
            &["GPL-3.0-only"],
            &["Foo-Bar"],
            &["GPL-3.0-only", "MIT"],
        ] {
            assert!(with(&policy, partial).blocked, "{partial:?}");
        }
        assert!(!with(&policy, &["GPL-3.0-only", "foo-bar"]).blocked);

        let partial = with(&policy, &["GPL-3.0-only"]);
        assert_eq!(
            license_reason(&partial).map(|(_, breach, blocking)| (breach, blocking)),
            Some((Breach::Unknown, true))
        );
        assert!(
            partial
                .reasons
                .iter()
                .any(|reason| matches!(reason, Reason::Exception { .. }))
        );

        let mut by_component = license_exception(None);
        by_component.name = Some("lib".to_owned());
        let lifted = judge_license(&policy, Some(&found), &[by_component]);
        assert!(!lifted.blocked);

        policy.licenses.unknown = UnknownLicense::Report;
        let softened = with(&policy, &["GPL-3.0-only"]);
        assert_eq!((softened.blocked, softened.state), (false, State::Report));
        assert!(with(&policy, &[]).blocked);

        let mut allow = licensed(Mode::Block, LicenseList::Allow, &["MIT"]);
        allow.licenses.unknown = UnknownLicense::Block;
        let mixed = fact("MIT AND Foo-Bar");
        assert!(judge_license(&allow, Some(&mixed), &[]).blocked);
        let held = [license_exception(Some("Foo-Bar"))];
        assert!(!judge_license(&allow, Some(&mixed), &held).blocked);
    }

    #[test]
    fn many_exceptions_for_one_id_are_cited_once_through_the_longest_lived() {
        let policy = licensed(Mode::Block, LicenseList::Deny, &["GPL-3.0-only"]);
        let found = fact("GPL-3.0-only");
        let mut held: Vec<Exception> = (1..=30)
            .map(|hours| {
                let mut each = license_exception(Some("GPL-3.0-only"));
                each.expires_at = Some(Utc::now() + Duration::hours(hours));
                each
            })
            .collect();
        let longest = held[29].id;
        let verdict = judge_license(&policy, Some(&found), &held);
        assert!(!verdict.blocked);
        let cited: Vec<Uuid> = verdict
            .reasons
            .iter()
            .filter_map(|reason| match reason {
                Reason::Exception { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(cited, [longest]);
        let permanent = license_exception(Some("gpl-3.0-only"));
        let forever = permanent.id;
        held.insert(3, permanent);
        let verdict = judge_license(&policy, Some(&found), &held);
        let cited: Vec<Uuid> = verdict
            .reasons
            .iter()
            .filter_map(|reason| match reason {
                Reason::License { exceptions, .. } => Some(exceptions.iter().map(|each| each.id)),
                _ => None,
            })
            .flatten()
            .collect();
        assert_eq!(cited, [forever]);
    }

    #[test]
    fn a_blocking_rule_blocks_whatever_the_other_says_and_every_failing_rule_is_listed() {
        let mut both = licensed(Mode::Report, LicenseList::Deny, &["GPL-3.0-only"]);
        both.vulnerabilities = VulnerabilityRule {
            mode: Mode::Block,
            severity: Some(Severity::High),
            ..VulnerabilityRule::default()
        };
        let found = fact("GPL-3.0-only");
        let verdict = evaluate(&Input {
            policy: &both,
            exceptions: &[],
            target: &target(),
            scan: Scan::Scanned,
            findings: &[finding("A", Severity::Critical)],
            license: Some(&found),
            now: Utc::now(),
        })
        .unwrap();
        assert_eq!((verdict.blocked, verdict.state), (true, State::Block));
        assert_eq!(verdict.counts.critical, 1);
        assert!(
            verdict
                .reasons
                .iter()
                .any(|reason| matches!(reason, Reason::Vulnerability { blocking: true, .. }))
        );
        assert_eq!(
            license_reason(&verdict).map(|(_, breach, blocking)| (breach, blocking)),
            Some((Breach::Denied, false))
        );

        let mut licence_blocks = both.clone();
        licence_blocks.licenses.mode = Mode::Block;
        let verdict = evaluate(&Input {
            policy: &licence_blocks,
            exceptions: &[],
            target: &target(),
            scan: Scan::Scanned,
            findings: &[],
            license: Some(&found),
            now: Utc::now(),
        })
        .unwrap();
        assert_eq!((verdict.blocked, verdict.state), (true, State::Block));
        assert_eq!(
            license_reason(&verdict).map(|(_, breach, blocking)| (breach, blocking)),
            Some((Breach::Denied, true))
        );
        assert!(
            !verdict
                .reasons
                .iter()
                .any(|reason| matches!(reason, Reason::Vulnerability { .. }))
        );

        let mut waiting = both.clone();
        waiting.vulnerabilities.unscanned = Unscanned::Block;
        let verdict = evaluate(&Input {
            policy: &waiting,
            exceptions: &[],
            target: &target(),
            scan: Scan::Missing,
            findings: &[],
            license: Some(&found),
            now: Utc::now(),
        })
        .unwrap();
        assert_eq!((verdict.blocked, verdict.state), (true, State::Unscanned));
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

    const FORMAT_CHARACTERS: [(u32, u32); 21] = [
        (0x00AD, 0x00AD),
        (0x0600, 0x0605),
        (0x061C, 0x061C),
        (0x06DD, 0x06DD),
        (0x070F, 0x070F),
        (0x0890, 0x0891),
        (0x08E2, 0x08E2),
        (0x180E, 0x180E),
        (0x200B, 0x200F),
        (0x202A, 0x202E),
        (0x2060, 0x2064),
        (0x2066, 0x206F),
        (0xFEFF, 0xFEFF),
        (0xFFF9, 0xFFFB),
        (0x110BD, 0x110BD),
        (0x110CD, 0x110CD),
        (0x13430, 0x1343F),
        (0x1BCA0, 0x1BCA3),
        (0x1D173, 0x1D17A),
        (0xE0001, 0xE0001),
        (0xE0020, 0xE007F),
    ];

    #[test]
    fn reason_text_loses_every_unicode_format_character() {
        for (first, last) in FORMAT_CHARACTERS {
            for code in first..=last {
                let character = char::from_u32(code).unwrap();
                assert_eq!(
                    clean(&format!("a{character}b")),
                    "ab",
                    "U+{code:04X} is a format character"
                );
            }
        }
    }

    #[test]
    fn reason_text_keeps_the_neighbours_of_each_hidden_range() {
        for (first, last) in HIDDEN {
            for edge in [first, last] {
                assert_eq!(clean(&format!("a{edge}b")), "ab", "U+{:04X}", edge as u32);
            }
            let before = char::from_u32(first as u32 - 1).unwrap();
            let after = char::from_u32(last as u32 + 1).unwrap();
            for neighbour in [before, after] {
                assert_eq!(
                    clean(&format!("a{neighbour}b")),
                    format!("a{neighbour}b"),
                    "U+{:04X} next to U+{:04X}..U+{:04X}",
                    neighbour as u32,
                    first as u32,
                    last as u32
                );
            }
        }
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
        let mut open = finding("GHSA-2", Severity::Critical);
        open.advisory = "GH\nSA-\u{200B}2".to_owned();
        open.aliases = vec![
            "CVE-\u{202E}2\r".to_owned(),
            format!("CVE-{}", "7".repeat(300)),
        ];
        let verdict = judge(
            &block,
            Scan::Scanned,
            &[hit, open],
            std::slice::from_ref(&held),
        )
        .unwrap();
        let texts = serde_json::to_string(&verdict.reasons).unwrap();
        assert!(!texts.contains("\\n") && !texts.contains("\\r") && !texts.contains("202e"));
        let mut listed_open = 0;
        for reason in &verdict.reasons {
            match reason {
                Reason::Vulnerability {
                    advisory,
                    aliases,
                    blocking: true,
                    ..
                } => {
                    listed_open += 1;
                    assert_eq!(advisory, "GHSA-2");
                    assert_eq!(aliases[0], "CVE-2");
                    assert_eq!(aliases[1].chars().count(), MAX_REASON_TEXT);
                }
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
        assert_eq!(listed_open, 1);
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
