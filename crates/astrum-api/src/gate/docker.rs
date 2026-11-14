use std::time::Instant;

use astrum_content::{ContentError, Ctx};
use astrum_policy::policy::{Mode, Policy, Unscanned};
use astrum_policy::refusal;
use astrum_policy::subject;
use astrum_policy::verdict::{State, clean};
use astrum_scan::queue;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use serde_json::{Value, json};

use super::{
    Held, Judgement, MARK_BUDGET, Outcome, PolicyGate, Refusal, UNAVAILABLE, Unchecked, read,
    unreadable,
};

const DOCKER: &str = "docker";
const MAX_DENIED: usize = 400;
const MAX_LISTED: usize = 8;
const MAX_PARENTS: i64 = 64;
const MAX_TAGGED: i64 = 16;
const SCAN_REASON: &str = "image pull";
const BEING_SCANNED: &str = "the image is being scanned, retry";
const NOT_QUEUED: &str = "the scan of the image could not be queued, retry";
const NO_SCAN_DATA: &str = "no scan data for this image digest";
const IN_BLOCKED_INDEX: &str = "the image is part of a blocked index: ";
const BLOCKED: &str = "the image is blocked";

pub(super) fn judged_by_digest(ctx: &Ctx) -> bool {
    ctx.format_id() == DOCKER
}

fn cap(text: &str, max: usize) -> String {
    match text.chars().count() > max {
        true => {
            let mut cut: String = text.chars().take(max - 1).collect();
            cut.push('\u{2026}');
            cut
        }
        false => text.to_owned(),
    }
}

fn denied(policy: &str, details: &str) -> ContentError {
    match ContentError::blocked_by(&clean(policy), details) {
        ContentError::Blocked(text) => ContentError::Blocked(cap(&text, MAX_DENIED)),
        error => error,
    }
}

fn refuse(policy: &str, state: &str, details: &str) -> Judgement {
    Judgement::Refused(
        Outcome::Blocked,
        Refusal {
            policy: policy.to_owned(),
            state: clean(state),
            error: denied(policy, details),
        },
    )
}

fn text<'a>(item: &'a Value, key: &str) -> Option<&'a str> {
    item.get(key).and_then(Value::as_str)
}

fn advisories(items: &[Value]) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for item in items {
        if text(item, "kind") != Some("vulnerability")
            || item.get("blocking").and_then(Value::as_bool) != Some(true)
        {
            continue;
        }
        let Some(advisory) = text(item, "advisory") else {
            continue;
        };
        let id = clean(advisory);
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

fn first(items: &[Value], kind: &str, key: &str) -> Option<String> {
    items
        .iter()
        .find(|item| text(item, "kind") == Some(kind))
        .and_then(|item| text(item, key))
        .map(clean)
}

fn details(reasons: &Value) -> String {
    let items: &[Value] = reasons.as_array().map_or(&[], Vec::as_slice);
    let ids = advisories(items);
    if !ids.is_empty() {
        let shown = ids.iter().take(MAX_LISTED).cloned().collect::<Vec<_>>();
        let noun = match ids.len() {
            1 => "advisory",
            _ => "advisories",
        };
        let mut line = format!("{} blocking {noun}: {}", ids.len(), shown.join(", "));
        if ids.len() > shown.len() {
            line.push_str(&format!(" and {} more", ids.len() - shown.len()));
        }
        return line;
    }
    if let Some(message) = first(items, "policy-error", "message") {
        return message;
    }
    if let Some(scan) = first(items, "unscanned", "scan") {
        return format!("the image has not been scanned (scan: {scan})");
    }
    BLOCKED.to_owned()
}

async fn parents(
    db: &DatabaseConnection,
    repository: &str,
    digest: &str,
    policy: &str,
) -> Result<Vec<Held>, DbErr> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT v.policy, v.policy_revision, v.state, v.blocked, v.reasons \
             FROM docker_manifest_child c \
             JOIN policy_verdict v ON v.repository = c.repository AND v.subject = 'oci:' || c.parent \
             WHERE c.repository = $1 AND c.child = $2 AND v.policy = $3 \
             ORDER BY CASE WHEN v.blocked OR v.state NOT IN ('pass', 'report', 'unscanned') THEN 0 \
                           WHEN v.state = 'report' THEN 1 ELSE 2 END, c.parent \
             LIMIT $4",
            [
                repository.into(),
                digest.into(),
                policy.into(),
                MAX_PARENTS.into(),
            ],
        ))
        .await?;
    let mut held = Vec::with_capacity(rows.len());
    for row in rows {
        held.push(Held {
            policy: row.try_get("", "policy")?,
            revision: row.try_get("", "policy_revision")?,
            state: row.try_get("", "state")?,
            blocked: row.try_get("", "blocked")?,
            reasons: row.try_get("", "reasons")?,
        });
    }
    Ok(held)
}

async fn tagged(
    db: &DatabaseConnection,
    repository: &str,
    digest: &str,
) -> Result<Vec<i64>, DbErr> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT DISTINCT component_id FROM asset \
             WHERE repository = $1 AND format = 'docker' AND kind = 'tag' \
               AND component_id IS NOT NULL AND attributes #>> '{docker,digest}' = $2 \
             ORDER BY component_id LIMIT $3",
            [repository.into(), digest.into(), MAX_TAGGED.into()],
        ))
        .await?;
    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        ids.push(row.try_get("", "component_id")?);
    }
    Ok(ids)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Queued {
    Yes,
    NoComponent,
    Failed,
}

fn scan_state(reasons: &Value) -> Option<&str> {
    reasons
        .as_array()?
        .iter()
        .find(|item| text(item, "kind") == Some("unscanned"))
        .and_then(|item| text(item, "scan"))
}

fn rescannable(verdict: &Held) -> bool {
    verdict.state == State::Unscanned.as_str()
        && matches!(scan_state(&verdict.reasons), Some("failed" | "no-data"))
}

fn reported(verdict: &Held) -> Judgement {
    match verdict.state == State::Report.as_str() {
        true => Judgement::Allowed(Outcome::WouldBlock),
        false => Judgement::Allowed(Outcome::Pass),
    }
}

fn decided(policy: &Policy, verdict: &Held) -> Judgement {
    let unscanned = verdict.state == State::Unscanned.as_str()
        && policy.vulnerabilities.unscanned == Unscanned::Block;
    match refusal::blocks(&verdict.state, verdict.blocked) || unscanned {
        false => Judgement::Allowed(Outcome::Pass),
        true => refuse(&policy.name, &verdict.state, &details(&verdict.reasons)),
    }
}

fn complete_pass(verdict: &Held) -> bool {
    verdict.state == State::Pass.as_str()
        && !verdict.blocked
        && !verdict.reasons.as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| text(item, "kind") == Some("unscanned"))
        })
}

fn inherited(policy: &Policy, revision: i64, parents: &[Held]) -> Option<Judgement> {
    let blocking = parents
        .iter()
        .find(|parent| refusal::blocks(&parent.state, parent.blocked));
    match blocking {
        Some(parent) => Some(refuse(
            &policy.name,
            &parent.state,
            &format!("{IN_BLOCKED_INDEX}{}", details(&parent.reasons)),
        )),
        None => parents
            .iter()
            .any(|parent| parent.revision == revision && complete_pass(parent))
            .then_some(Judgement::Allowed(Outcome::Pass)),
    }
}

impl PolicyGate {
    async fn judge_image(
        &self,
        ctx: &Ctx,
        repository: &str,
        digest: &str,
    ) -> Result<Judgement, Unchecked> {
        let db = ctx.db();
        let held = read(db, repository, &subject::image(digest))
            .await?
            .ok_or(Unchecked::Vanished)?;
        let Some(name) = held.policy else {
            return Ok(Judgement::Unguarded);
        };
        let snapshot = self.snapshot_for(db, &name).await?;
        let Some(stored) = snapshot.policies.get(&name) else {
            return Ok(unreadable(&name));
        };
        let Ok(policy) = &stored.policy else {
            return Ok(unreadable(&name));
        };
        let mode = policy.vulnerabilities.mode;
        if mode == Mode::Off {
            return Ok(Judgement::Allowed(Outcome::Pass));
        }
        let (own, stale) = match held.verdict {
            Some(verdict) if verdict.policy == name && verdict.revision == stored.revision => {
                (Some(verdict), None)
            }
            Some(verdict) if verdict.policy == name => (None, Some(verdict)),
            _ => (None, None),
        };
        if let Some(verdict) = &own {
            let judged = match mode {
                Mode::Report => reported(verdict),
                _ => decided(policy, verdict),
            };
            let refused = matches!(judged, Judgement::Refused(..));
            if refused && rescannable(verdict) {
                self.queue_image(db, repository, digest).await;
            }
            return Ok(judged);
        }
        if mode == Mode::Block
            && let Some(verdict) = &stale
            && refusal::blocks(&verdict.state, verdict.blocked)
        {
            return Ok(refuse(
                &policy.name,
                &verdict.state,
                &details(&verdict.reasons),
            ));
        }
        let above = parents(db, repository, digest, &name).await?;
        if mode == Mode::Report {
            let would = above.iter().any(|parent| {
                parent.revision == stored.revision && parent.state == State::Report.as_str()
            });
            return Ok(Judgement::Allowed(match would {
                true => Outcome::WouldBlock,
                false => Outcome::Pass,
            }));
        }
        if let Some(refused) = inherited(policy, stored.revision, &above) {
            return Ok(refused);
        }
        Ok(self.without_verdict(db, repository, digest, policy).await)
    }

    async fn without_verdict(
        &self,
        db: &DatabaseConnection,
        repository: &str,
        digest: &str,
        policy: &Policy,
    ) -> Judgement {
        match policy.vulnerabilities.unscanned {
            Unscanned::Allow => Judgement::Allowed(Outcome::InlineUnscanned),
            Unscanned::Block => {
                let details = match self.queue_image(db, repository, digest).await {
                    Queued::Yes => BEING_SCANNED,
                    Queued::NoComponent => NO_SCAN_DATA,
                    Queued::Failed => NOT_QUEUED,
                };
                refuse(&policy.name, State::Unscanned.as_str(), details)
            }
        }
    }

    async fn queue_image(&self, db: &DatabaseConnection, repository: &str, digest: &str) -> Queued {
        let queued = async {
            let ids = tagged(db, repository, digest).await?;
            match ids.is_empty() {
                true => Ok(Queued::NoComponent),
                false => queue::enqueue_missing_components(db, repository, &ids, SCAN_REASON)
                    .await
                    .map(|_| Queued::Yes),
            }
        };
        let failure = match tokio::time::timeout(MARK_BUDGET, queued).await {
            Ok(Ok(queued)) => return queued,
            Ok(Err(error)) => error.to_string(),
            Err(_) => format!("it was not written within {} ms", MARK_BUDGET.as_millis()),
        };
        self.metrics.count(Outcome::MarkFailed);
        tracing::error!(%repository, %digest, error = %failure, "an image without a verdict was not queued for the workers");
        Queued::Failed
    }

    async fn audit_image(
        &self,
        ctx: &Ctx,
        repository: &str,
        digest: &str,
        reference: Option<&str>,
        refusal: &Refusal,
    ) {
        let subject = subject::image(digest);
        if !self.audited.first(repository, &subject, Instant::now()) {
            return;
        }
        let mut attributes = json!({
            "repository": repository,
            "digest": digest,
            "policy": refusal.policy,
            "state": refusal.state,
        });
        if let Some(reference) = reference {
            attributes["reference"] = json!(reference);
        }
        self.record(ctx, repository, &subject, &subject, attributes)
            .await;
    }

    pub(super) async fn check_image(
        &self,
        ctx: &Ctx,
        repository: &str,
        digest: &str,
        reference: Option<&str>,
    ) -> Result<(), ContentError> {
        if repository == ctx.name() && ctx.repository.policy_name().is_none() {
            return Ok(());
        }
        let started = Instant::now();
        let judged = self.judge_image(ctx, repository, digest).await;
        if let Err(Unchecked::Vanished) = &judged {
            return Err(ContentError::not_found(format!("{repository}@{digest}")));
        }
        self.metrics.observe(started.elapsed());
        match judged {
            Ok(Judgement::Unguarded) => {
                self.metrics.count(Outcome::Unguarded);
                Ok(())
            }
            Ok(Judgement::Allowed(outcome)) => {
                self.metrics.count(outcome);
                Ok(())
            }
            Ok(Judgement::Refused(outcome, refusal)) => {
                self.metrics.count(outcome);
                self.audit_image(ctx, repository, digest, reference, &refusal)
                    .await;
                Err(refusal.error)
            }
            Ok(Judgement::Cooled(error)) => {
                self.metrics.count(Outcome::Blocked);
                Err(error)
            }
            Err(error) => {
                self.metrics.count(Outcome::Error);
                tracing::error!(%repository, %digest, %error, "the pull was refused because its content policy could not be checked");
                Err(ContentError::Unavailable(UNAVAILABLE.to_owned()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(advisory: &str, blocking: bool) -> Value {
        json!({"kind": "vulnerability", "advisory": advisory, "aliases": ["CVE-9"],
            "severity": "high", "malicious": false, "package": "npm/secret-name@1.0.0",
            "fixed": "free text from an advisory", "blocking": blocking})
    }

    #[test]
    fn the_text_names_advisory_ids_and_counts_and_nothing_else_from_the_findings() {
        let reasons = json!([
            finding("GHSA-1", true),
            finding("GHSA-1", true),
            finding("GHSA-2", true),
            finding("GHSA-3", false),
            {"kind": "unscanned", "scan": "incomplete"}
        ]);
        assert_eq!(details(&reasons), "2 blocking advisories: GHSA-1, GHSA-2");
        assert_eq!(
            details(&json!([finding("GHSA-1", true)])),
            "1 blocking advisory: GHSA-1"
        );
        let many: Vec<Value> = (0..12)
            .map(|index| finding(&format!("GHSA-{index}"), true))
            .collect();
        let text = details(&Value::Array(many));
        assert!(
            text.starts_with("12 blocking advisories: GHSA-0, "),
            "{text}"
        );
        assert!(text.ends_with(" and 4 more"), "{text}");
        assert!(!text.contains("secret-name") && !text.contains("free text"));
    }

    #[test]
    fn without_a_blocking_advisory_the_text_names_the_error_or_the_scan_state() {
        assert_eq!(
            details(&json!([{"kind": "policy-error", "message": "the policy \"p\" is gone"}])),
            "the policy \"p\" is gone"
        );
        assert_eq!(
            details(&json!([{"kind": "unscanned", "scan": "incomplete"}])),
            "the image has not been scanned (scan: incomplete)"
        );
        for nothing in [json!([]), json!(null), json!([42]), json!({"kind": "x"})] {
            assert_eq!(details(&nothing), "the image is blocked");
        }
    }

    #[test]
    fn the_refusal_is_capped_at_four_hundred_characters_and_has_no_control_characters() {
        let long = json!([
            finding(&format!("1{}", "A".repeat(300)), true),
            finding(&format!("2{}", "A".repeat(300)), true),
            finding("B\n\u{202E}C", true)
        ]);
        let error = denied("strict", &details(&long));
        let ContentError::Blocked(text) = &error else {
            panic!("a refusal is a blocked error: {error:?}");
        };
        assert_eq!(text.chars().count(), MAX_DENIED);
        assert!(text.starts_with("blocked by policy strict: 3 blocking advisories: "));
        assert!(text.ends_with('\u{2026}'));
        assert!(text.chars().all(|ch| !ch.is_control()), "{text}");
        assert_eq!(error.blocking_policy(), Some("strict"));
        let short = denied("strict", BEING_SCANNED);
        assert_eq!(
            short.to_string(),
            "blocked by policy strict: the image is being scanned, retry"
        );
    }
}
