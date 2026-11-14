mod inventory_common;
mod support;

use inventory_common::*;
use serde_json::json;

fn scanned(digest: &str, repository: &str, source: &str, packages: &str) -> Vec<String> {
    vec![
        "scanned".to_owned(),
        digest.to_owned(),
        format!("oci:{repository}:{digest}"),
        source.to_owned(),
        packages.to_owned(),
    ]
}

#[tokio::test]
async fn an_image_with_a_referrer_sbom_gets_an_inventory_findings_and_the_same_verdict_by_tag_and_digest()
 {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    registry
        .document(
            &digest,
            None,
            SPDX,
            &spdx(&[MUSL, LODASH, "pkg:npm/fine@1.0.0"]),
        )
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("1.0").await,
        scanned(&digest, "hosted", "sbom", "3")
    );
    assert_eq!(
        registry.verdicts().await,
        vec![
            strings(&["c", "block", "true", "2"]),
            strings(&["oci", "block", "true", "2"])
        ]
    );
    assert!(
        registry
            .rows("SELECT subject AS n FROM policy_verdict WHERE subject LIKE 'oci:%' AND component_id IS NOT NULL")
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn an_image_with_only_a_legacy_sbom_tag_is_inventoried_from_it_with_the_source_package() {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    let bom = json!({"bomFormat": "CycloneDX", "specVersion": "1.5", "components": [
        {"name": "libc6", "version": "2.36-9", "purl": "pkg:deb/debian/libc6@2.36-9?distro=debian-12&upstream=glibc"}]})
    .to_string();
    let tag = format!("{}.sbom", digest.replace(':', "-"));
    registry
        .document(&digest, Some(&tag), CYCLONEDX, &bom)
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("1.0").await,
        scanned(&digest, "hosted", "sbom", "1")
    );
    assert_eq!(
        registry.verdicts().await[0],
        strings(&["c", "block", "true", "1"])
    );
}

#[tokio::test]
async fn a_buildkit_attestation_manifest_inside_an_index_is_the_sbom_of_its_platform_image() {
    let registry = Registry::new().await;
    let platform = registry.image(None, b"layer").await;
    let config = registry.push_blob(b"{}").await;
    let layer = registry.push_blob(statement(&[MUSL]).as_bytes()).await;
    let body = manifest(&config, (&layer, IN_TOTO), None, None);
    let attested = registry
        .push_manifest(&sha256(&serde_json::to_vec(&body).unwrap()), &body)
        .await;
    let listing = index(&[
        (&platform, None),
        (&attested, Some(("attestation-manifest", &platform))),
    ]);
    let top = registry.push_manifest("multi", &listing).await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("multi").await,
        scanned(&top, "hosted", "index", "1")
    );
    assert_eq!(
        registry.verdicts().await.last().unwrap(),
        &strings(&["oci", "block", "true", "1"])
    );
}

#[tokio::test]
async fn an_index_is_judged_on_the_union_of_its_children_and_a_missing_child_makes_it_incomplete() {
    let registry = Registry::new().await;
    let first = registry.image(None, b"one").await;
    let second = registry.image(None, b"two").await;
    registry
        .document(&first, None, SPDX, &spdx(&[MUSL, "pkg:npm/shared@1.0.0"]))
        .await;
    registry
        .document(
            &second,
            None,
            SPDX,
            &spdx(&[LODASH, "pkg:npm/shared@1.0.0"]),
        )
        .await;
    let complete = registry
        .push_manifest("multi", &index(&[(&first, None), (&second, None)]))
        .await;
    let absent = sha256(b"never pushed");
    let partial = registry
        .push_manifest("partial", &index(&[(&first, None), (&absent, None)]))
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("multi").await,
        scanned(&complete, "hosted", "index", "3")
    );
    let incomplete = registry.scan_of("partial").await;
    assert_eq!(incomplete[0], "incomplete");
    assert_eq!(incomplete[1], partial);
    assert_eq!(incomplete[4], "2");
    let union = registry
        .rows(&format!(
            "SELECT high::text AS high FROM policy_verdict WHERE subject = 'oci:{complete}'"
        ))
        .await;
    assert_eq!(union, vec![strings(&["2"])]);
}

#[tokio::test]
async fn an_sbom_attached_to_the_index_itself_is_merged_with_those_of_its_children() {
    let registry = Registry::new().await;
    let child = registry.image(None, b"one").await;
    registry.document(&child, None, SPDX, &spdx(&[MUSL])).await;
    let top = registry
        .push_manifest("multi", &index(&[(&child, None)]))
        .await;
    registry.document(&top, None, SPDX, &spdx(&[LODASH])).await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("multi").await,
        scanned(&top, "hosted", "index", "2")
    );
}

#[tokio::test]
async fn an_image_without_any_sbom_is_unsupported_and_has_no_inventory() {
    let registry = Registry::new().await;
    registry.image(Some("1.0"), b"layer").await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("1.0").await,
        vec![
            "unsupported".to_owned(),
            "-".to_owned(),
            "-".to_owned(),
            "-".to_owned(),
            "0".to_owned()
        ]
    );
}

#[tokio::test]
async fn junk_referrers_cannot_push_the_real_sbom_out_of_the_slots_and_the_overflow_is_flagged() {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    registry.document(&digest, None, SPDX, &spdx(&[MUSL])).await;
    for number in 0..10 {
        let config = registry.push_blob(b"{}").await;
        let layer = registry
            .push_blob(format!("junk {number}").as_bytes())
            .await;
        let mut body = manifest(
            &config,
            (&layer, "application/vnd.syft+json"),
            Some("application/vnd.syft+json"),
            Some(&digest),
        );
        body["annotations"] = json!({"n": number.to_string()});
        registry
            .push_manifest(&sha256(&serde_json::to_vec(&body).unwrap()), &body)
            .await;
    }
    registry.settle().await;
    let scan = registry.scan_of("1.0").await;
    assert_eq!(scan[0], "incomplete");
    assert_eq!(scan[4], "1");
    assert_eq!(
        registry.verdicts().await[0],
        strings(&["c", "block", "true", "1"])
    );
}

#[tokio::test]
async fn an_empty_sbom_is_a_complete_empty_inventory_and_one_with_nothing_mappable_is_incomplete() {
    let registry = Registry::new().await;
    let empty = registry.image(Some("empty"), b"one").await;
    registry.document(&empty, None, SPDX, &spdx(&[])).await;
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(b"two").await;
    let other = registry
        .push_manifest(
            "unmappable",
            &manifest(&config, (&layer, LAYER), None, None),
        )
        .await;
    registry
        .document(&other, None, SPDX, &spdx(&["pkg:rpm/fedora/bash@5.2"]))
        .await;
    registry.settle().await;
    assert_eq!(
        registry.scan_of("empty").await,
        scanned(&empty, "hosted", "sbom", "0")
    );
    assert_eq!(registry.scan_of("unmappable").await[0], "incomplete");
}

#[tokio::test]
async fn an_sbom_pushed_after_the_image_queues_every_tag_of_it_and_the_next_scan_finds_the_packages()
 {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    let config = registry.push_blob(b"{\"os\":\"linux\"}").await;
    let layer = registry.push_blob(b"layer").await;
    registry
        .push_manifest("stable", &manifest(&config, (&layer, LAYER), None, None))
        .await;
    registry.settle().await;
    assert_eq!(registry.scan_of("stable").await[0], "unsupported");
    assert!(
        registry
            .rows("SELECT 1 AS n FROM scan_queue")
            .await
            .is_empty()
    );
    registry.document(&digest, None, SPDX, &spdx(&[MUSL])).await;
    let queued = registry
        .rows("SELECT c.version, q.reason FROM scan_queue q JOIN component c ON c.id = q.component_id ORDER BY c.version")
        .await;
    assert_eq!(
        queued,
        vec![strings(&["1.0", "sbom"]), strings(&["stable", "sbom"])]
    );
    registry.settle().await;
    assert_eq!(registry.scan_of("1.0").await[0], "scanned");
    assert_eq!(registry.scan_of("stable").await[0], "scanned");
    assert_eq!(registry.verdicts().await.len(), 3);
}
