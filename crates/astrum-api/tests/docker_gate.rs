mod docker_gate_common;
mod inventory_common;
mod support;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use astrum_api::gate::PolicyGate;
use astrum_content::{Content, Ctx, Gate};
use astrum_db::entity::audit_event;
use axum::http::{Method, StatusCode};
use docker_gate_common::*;
use inventory_common::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use support::Auth;

#[tokio::test]
async fn a_blocked_image_is_refused_by_tag_digest_and_head_and_a_conditional_request_cannot_bypass()
{
    let registry = Registry::new().await;
    let digest = blocked_image(&registry).await;
    let clean = registry.image(Some("clean"), b"other").await;
    registry
        .document(&clean, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    registry.settle().await;

    denied(&pull(&registry, "1.0").await, "GHSA-A");
    denied(&pull(&registry, &digest).await, "GHSA-A");
    for reference in ["1.0", digest.as_str()] {
        let head = pull_from(&registry, "hosted", Method::HEAD, reference).await;
        assert_eq!(head.status, StatusCode::FORBIDDEN);
    }
    let etag = format!("\"{digest}\"");
    let path = "/v2/hosted/app/manifests/1.0";
    let conditional = registry
        .send_with(
            Method::GET,
            path,
            Vec::new(),
            "application/json",
            &[("if-none-match", &etag)],
        )
        .await;
    denied(&conditional, "GHSA-A");

    let served = pull(&registry, "clean").await;
    assert_eq!(served.status, StatusCode::OK, "{}", served.text());
    let tag_etag = format!("\"{clean}\"");
    let unchanged = registry
        .send_with(
            Method::GET,
            "/v2/hosted/app/manifests/clean",
            Vec::new(),
            "application/json",
            &[("if-none-match", &tag_etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);
    assert_eq!(results(&registry)["blocked"], 5);
}

#[tokio::test]
async fn a_refusal_is_audited_once_per_digest_without_its_reasons() {
    let registry = Registry::new().await;
    let digest = blocked_image(&registry).await;
    for reference in ["1.0", digest.as_str(), "1.0"] {
        assert_eq!(
            pull(&registry, reference).await.status,
            StatusCode::FORBIDDEN
        );
    }
    let rows = audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("contentPolicy"))
        .filter(audit_event::Column::Kind.eq("blocked"))
        .all(registry.fixture.state.db().conn())
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].context, format!("oci:{digest}"));
    assert_eq!(
        rows[0].attributes,
        json!({"repository": "hosted", "digest": digest, "reference": "1.0",
               "policy": "strict", "state": "block"})
    );
    assert!(
        rows[0].initiator.starts_with("admin"),
        "{}",
        rows[0].initiator
    );
}

#[tokio::test]
async fn report_mode_serves_and_counts_what_it_would_block_and_off_and_no_policy_serve() {
    let registry = Registry::new().await;
    let digest = blocked_image(&registry).await;
    let free = registry.repository_without_policy("free").await;
    free.image(Some("1.0"), b"layer").await;

    set_policy(&registry, json!({"mode": "report", "severity": "high"})).await;
    rejudge(&registry).await;
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);
    assert_eq!(pull(&registry, &digest).await.status, StatusCode::OK);
    assert_eq!(results(&registry), HashMap::from([("would_block", 2)]));

    set_policy(&registry, json!({"mode": "off"})).await;
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);
    let unguarded = pull_from(&free, "free", Method::GET, "1.0").await;
    assert_eq!(unguarded.status, StatusCode::OK, "{}", unguarded.text());
    assert_eq!(
        results(&registry),
        HashMap::from([("would_block", 2), ("pass", 1)])
    );
}

#[tokio::test]
async fn an_exception_by_advisory_or_by_name_unblocks_after_the_verdict_is_judged_again() {
    let registry = Registry::new().await;
    blocked_image(&registry).await;
    denied(&pull(&registry, "1.0").await, "GHSA-A");

    let by_alias = registry
        .fixture
        .call(
            Method::POST,
            EXCEPTIONS,
            Auth::Admin,
            Some(json!({"rule": "vulnerability", "advisory": "CVE-A", "reason": "accepted"})),
        )
        .await;
    assert_eq!(by_alias.status, StatusCode::CREATED, "{}", by_alias.text());
    rejudge(&registry).await;
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);

    let id = by_alias.json()["id"].as_str().unwrap().to_owned();
    let removed = registry
        .fixture
        .call(
            Method::DELETE,
            &format!("{EXCEPTIONS}/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert!(removed.status.is_success(), "{}", removed.text());
    rejudge(&registry).await;
    denied(&pull(&registry, "1.0").await, "GHSA-A");

    let by_name = registry
        .fixture
        .call(
            Method::POST,
            EXCEPTIONS,
            Auth::Admin,
            Some(json!({"rule": "vulnerability", "format": "docker", "name": "app", "reason": "accepted"})),
        )
        .await;
    assert_eq!(by_name.status, StatusCode::CREATED, "{}", by_name.text());
    rejudge(&registry).await;
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);
}

#[tokio::test]
async fn a_verdict_of_an_older_policy_revision_keeps_blocking_and_a_passing_one_counts_as_missing()
{
    let registry = Registry::new().await;
    blocked_image(&registry).await;
    let clean = registry.image(Some("clean"), b"other").await;
    registry
        .document(&clean, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    registry.settle().await;

    set_policy(&registry, json!({"mode": "block", "severity": "medium"})).await;
    denied(&pull(&registry, "1.0").await, "GHSA-A");
    assert_eq!(pull(&registry, "clean").await.status, StatusCode::OK);
    assert_eq!(
        results(&registry),
        HashMap::from([("blocked", 1), ("inline_scan_unscanned", 1)])
    );

    unscanned_block(&registry).await;
    denied(
        &pull(&registry, "clean").await,
        "the image is being scanned, retry",
    );
    rejudge(&registry).await;
    assert_eq!(pull(&registry, "clean").await.status, StatusCode::OK);
    denied(&pull(&registry, "1.0").await, "GHSA-A");
}

#[tokio::test]
async fn without_a_verdict_allow_serves_and_block_refuses_queues_the_tag_and_an_untagged_digest_has_no_scan_data()
 {
    let registry = Registry::new().await;
    let tagged = registry.image(Some("1.0"), b"layer").await;
    let untagged = registry.image(None, b"other").await;
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);
    assert_eq!(pull(&registry, &untagged).await.status, StatusCode::OK);

    unscanned_block(&registry).await;
    registry.execute("DELETE FROM scan_queue").await;
    denied(
        &pull(&registry, "1.0").await,
        "the image is being scanned, retry",
    );
    denied(
        &pull(&registry, &tagged).await,
        "the image is being scanned, retry",
    );
    assert_eq!(
        registry
            .rows("SELECT c.version, q.reason FROM scan_queue q JOIN component c ON c.id = q.component_id")
            .await,
        vec![strings(&["1.0", "image pull"])]
    );

    registry.execute("DELETE FROM scan_queue").await;
    denied(
        &pull(&registry, &untagged).await,
        "no scan data for this image digest",
    );
    assert!(
        registry
            .rows("SELECT 1 AS n FROM scan_queue")
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn refused_pulls_do_not_reset_the_schedule_of_a_scan_that_is_already_queued() {
    let registry = Registry::new().await;
    registry.image(Some("1.0"), b"layer").await;
    unscanned_block(&registry).await;
    registry.execute("DELETE FROM scan_queue").await;
    let schedule = "SELECT component_id::text AS component, reason, attempts::text AS attempts, \
                    queued_at::text AS queued, next_attempt_at::text AS due FROM scan_queue";

    denied(&pull(&registry, "1.0").await, "being scanned");
    assert_eq!(registry.rows(schedule).await.len(), 1);
    registry
        .execute("UPDATE scan_queue SET attempts = 2, next_attempt_at = now() + interval '1 hour'")
        .await;
    let before = registry.rows(schedule).await;

    for _ in 0..4 {
        denied(&pull(&registry, "1.0").await, "being scanned");
    }
    assert_eq!(registry.rows(schedule).await, before);
    assert_eq!(before.len(), 1);
}

#[tokio::test]
async fn a_refusal_says_so_when_the_scan_could_not_be_queued() {
    let registry = Registry::new().await;
    registry.image(Some("1.0"), b"layer").await;
    unscanned_block(&registry).await;
    registry
        .execute("ALTER TABLE scan_queue RENAME TO scan_queue_away")
        .await;
    let reply = pull(&registry, "1.0").await;
    registry
        .execute("ALTER TABLE scan_queue_away RENAME TO scan_queue")
        .await;

    denied(&reply, "could not be queued");
    assert!(!reply.text().contains("being scanned"), "{}", reply.text());
    assert_eq!(results(&registry)["mark_failed"], 1);
}

#[tokio::test]
async fn a_verdict_of_a_failed_or_empty_scan_queues_it_again_and_an_incomplete_one_does_not() {
    let registry = Registry::new().await;
    let mut digests = Vec::new();
    for tag in ["failed", "no-data", "incomplete"] {
        digests.push(registry.image(Some(tag), tag.as_bytes()).await);
    }
    unscanned_block(&registry).await;
    for (tag, digest) in ["failed", "no-data", "incomplete"].iter().zip(&digests) {
        let reasons = json!([{"kind": "unscanned", "scan": tag}]);
        seed_verdict(&registry, "hosted", digest, "unscanned", true, reasons).await;
    }
    let queued =
        "SELECT c.version AS n FROM scan_queue q JOIN component c ON c.id = q.component_id";
    registry.execute("DELETE FROM scan_queue").await;

    for tag in ["failed", "no-data"] {
        denied(&pull(&registry, tag).await, &format!("scan: {tag}"));
        assert_eq!(registry.rows(queued).await, vec![strings(&[tag])]);
        registry.execute("DELETE FROM scan_queue").await;
    }
    denied(&pull(&registry, "incomplete").await, "scan: incomplete");
    assert!(registry.rows(queued).await.is_empty());
}

#[tokio::test]
async fn an_error_reading_the_verdict_answers_503_and_is_not_a_pass() {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    registry
        .document(&digest, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    registry.settle().await;
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);

    registry
        .execute("ALTER TABLE policy_verdict RENAME TO policy_verdict_away")
        .await;
    let reply = pull(&registry, "1.0").await;
    registry
        .execute("ALTER TABLE policy_verdict_away RENAME TO policy_verdict")
        .await;

    assert_eq!(
        reply.status,
        StatusCode::SERVICE_UNAVAILABLE,
        "{}",
        reply.text()
    );
    assert_eq!(reply.json()["errors"][0]["code"], "UNAVAILABLE");
    assert!(!reply.text().contains("policy_verdict"), "{}", reply.text());
    assert_eq!(pull(&registry, "1.0").await.status, StatusCode::OK);
    assert_eq!(
        results(&registry),
        HashMap::from([("pass", 2), ("error", 1)])
    );
}

fn counted(registry: &Registry) -> (Content, Arc<std::sync::atomic::AtomicUsize>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let fixture = &registry.fixture;
    let mut conn = fixture.state.db().conn().clone();
    let counter = Arc::new(AtomicUsize::new(0));
    let seen = counter.clone();
    conn.set_metric_callback(move |_| {
        seen.fetch_add(1, Ordering::Relaxed);
    });
    let secrets = astrum_security::SecretBox::from_key([7; 32]);
    let blobs = Arc::new(astrum_blob::BlobStores::new(
        conn.clone(),
        fixture.data.path(),
        secrets.clone(),
    ));
    (Content::new(conn, blobs, secrets), counter)
}

async fn context(content: &Content, repository: &str) -> Ctx {
    content.context(
        content.repository(repository).await.unwrap().unwrap(),
        Arc::new(astrum_security::SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    )
}

fn fresh_gate() -> PolicyGate {
    PolicyGate::new(
        Arc::new(astrum_policy::cache::Cache::new(Duration::from_secs(3600))),
        Arc::default(),
    )
}

#[tokio::test]
async fn the_hot_path_of_a_pass_is_one_statement_and_a_repository_without_a_policy_costs_none() {
    use std::sync::atomic::Ordering;
    let registry = Registry::new().await;
    let blocked = blocked_image(&registry).await;
    let clean = registry.image(Some("clean"), b"other").await;
    registry
        .document(&clean, None, SPDX, &spdx(&["pkg:npm/fine@1.0.0"]))
        .await;
    registry.settle().await;
    let free = registry.repository_without_policy("free").await;
    free.image(Some("1.0"), b"layer").await;
    let (content, counter) = counted(&registry);
    let hosted = context(&content, "hosted").await;
    let unguarded = context(&content, "free").await;
    let gate = fresh_gate();

    assert!(
        gate.check_digest(&hosted, "hosted", &blocked, None)
            .await
            .is_err()
    );
    counter.store(0, Ordering::Relaxed);
    assert!(
        gate.check_digest(&hosted, "hosted", &blocked, None)
            .await
            .is_err()
    );
    assert_eq!(counter.load(Ordering::Relaxed), 1, "a refusal");
    counter.store(0, Ordering::Relaxed);
    gate.check_digest(&hosted, "hosted", &clean, None)
        .await
        .unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 1, "a pass");
    counter.store(0, Ordering::Relaxed);
    gate.check_digest(&unguarded, "free", &blocked, None)
        .await
        .unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 0, "no policy");

    set_policy(&registry, json!({"mode": "off"})).await;
    let gate = fresh_gate();
    gate.check_digest(&hosted, "hosted", &blocked, None)
        .await
        .unwrap();
    counter.store(0, Ordering::Relaxed);
    gate.check_digest(&hosted, "hosted", &blocked, None)
        .await
        .unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 1, "mode off");
}
