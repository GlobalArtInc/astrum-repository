#![allow(dead_code)]

use std::collections::HashMap;

use astrum_api::gate::Outcome;
use astrum_scan::verdict::{self, SweepState};
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::inventory_common::*;
use crate::support::{Auth, Reply};

pub const POLICY: &str = "/service/rest/v1/security/policies/strict";
pub const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";

pub async fn set_policy(registry: &Registry, rule: Value) {
    let reply = registry
        .fixture
        .call(
            Method::PUT,
            POLICY,
            Auth::Admin,
            Some(json!({"name": "strict", "vulnerabilities": rule})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

pub async fn unscanned_block(registry: &Registry) {
    set_policy(
        registry,
        json!({"mode": "block", "severity": "high", "unscanned": "block"}),
    )
    .await;
}

pub async fn rejudge(registry: &Registry) {
    let db = registry.fixture.state.db().conn();
    let mut sweeper = SweepState::default();
    verdict::sweep(db, &verdict::Config::default(), &mut sweeper)
        .await
        .unwrap();
    registry.settle().await;
}

pub async fn blocked_image(registry: &Registry) -> String {
    let digest = registry.image(Some("1.0"), b"layer").await;
    registry.document(&digest, None, SPDX, &spdx(&[MUSL])).await;
    registry.settle().await;
    assert_eq!(
        registry.verdicts().await[1],
        strings(&["oci", "block", "true", "1"])
    );
    digest
}

pub async fn pull_from(
    registry: &Registry,
    repository: &str,
    method: Method,
    reference: &str,
) -> Reply {
    let path = format!("/v2/{repository}/app/manifests/{reference}");
    registry
        .send(method, &path, Vec::new(), "application/json")
        .await
}

pub async fn pull(registry: &Registry, reference: &str) -> Reply {
    pull_from(registry, "hosted", Method::GET, reference).await
}

pub fn denied(reply: &Reply, fragment: &str) {
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["errors"][0]["code"], "DENIED", "{body}");
    let message = body["errors"][0]["message"].as_str().unwrap();
    assert!(
        message.starts_with("blocked by policy strict: "),
        "{message}"
    );
    assert!(message.contains(fragment), "{message}");
    assert!(message.chars().count() <= 400, "{message}");
}

pub fn results(registry: &Registry) -> HashMap<&'static str, u64> {
    let snapshot = registry.fixture.state.gate_metrics().snapshot();
    Outcome::ALL
        .iter()
        .zip(snapshot.results)
        .filter(|(_, count)| *count > 0)
        .map(|(outcome, count)| (outcome.label(), count))
        .collect()
}
