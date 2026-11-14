mod inventory_common;
mod support;

use inventory_common::*;
use serde_json::json;

#[tokio::test]
async fn the_same_digest_in_two_repositories_keeps_two_inventories_and_one_never_changes_the_other()
{
    let first = Registry::new().await;
    let second = first.repository("other").await;
    let digest = first.image(Some("1.0"), b"layer").await;
    assert_eq!(second.image(Some("1.0"), b"layer").await, digest);
    first.document(&digest, None, SPDX, &spdx(&[MUSL])).await;
    first.settle().await;
    let before = first.verdicts().await;
    assert_eq!(before[1], strings(&["oci", "block", "true", "1"]));

    second.document(&digest, None, SPDX, &spdx(&[])).await;
    second.settle().await;

    assert_eq!(first.verdicts().await, before);
    assert_eq!(
        second.verdicts().await,
        vec![
            strings(&["c", "pass", "false", "0"]),
            strings(&["oci", "pass", "false", "0"])
        ]
    );
    let keys = first
        .rows("SELECT key AS n FROM sca_inventory ORDER BY key")
        .await;
    assert_eq!(
        keys,
        vec![
            vec![format!("oci:hosted:{digest}")],
            vec![format!("oci:other:{digest}")]
        ]
    );
    assert_eq!(
        first.scan_of("1.0").await[4],
        "1",
        "the packages of the first repository are untouched"
    );
}

#[tokio::test]
async fn a_package_with_a_mixed_case_name_matches_the_advisory_imported_for_its_lowercase_name() {
    let registry = Registry::new().await;
    let digest = registry.image(Some("1.0"), b"layer").await;
    let bom = json!({"bomFormat": "CycloneDX", "specVersion": "1.5", "components": [
        {"name": "Newtonsoft.Json", "version": "13.0.1", "purl": "pkg:nuget/Newtonsoft.Json@13.0.1"}]})
    .to_string();
    registry.document(&digest, None, CYCLONEDX, &bom).await;
    registry.settle().await;
    assert_eq!(
        registry.verdicts().await[1],
        strings(&["oci", "block", "true", "1"])
    );
}
