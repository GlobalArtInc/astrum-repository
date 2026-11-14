use astrum_sbom::{Format, Package};

use super::*;

fn listed(name: &str, purl: Option<&str>) -> Package {
    Package {
        name: name.to_owned(),
        version: "1".to_owned(),
        purl: purl.map(str::to_owned),
        licenses: Vec::new(),
        ecosystem: None,
    }
}

fn sbom(packages: Vec<Package>) -> Sbom {
    Sbom {
        format: Format::Spdx,
        packages,
    }
}

#[test]
fn a_package_seen_in_several_documents_is_kept_once_and_unlisted_ones_do_not_count() {
    let mut collector = Collector::new();
    collector.absorb(sbom(vec![
        listed("a", Some("pkg:npm/lodash@4.17.20")),
        listed("root", None),
    ]));
    collector.absorb(sbom(vec![
        listed("b", Some("pkg:npm/lodash@4.17.20")),
        listed("c", Some("pkg:npm/lodash@4.17.21")),
    ]));
    assert_eq!(collector.packages.len(), 2);
    assert!(collector.complete);
}

#[test]
fn a_package_that_cannot_be_mapped_makes_the_inventory_incomplete() {
    let mut collector = Collector::new();
    collector.absorb(sbom(vec![
        listed("bash", Some("pkg:rpm/fedora/bash@5.2")),
        listed("lodash", Some("pkg:npm/lodash@4.17.20")),
    ]));
    assert_eq!(collector.packages.len(), 1);
    assert!(!collector.complete);
}

#[test]
fn an_inventory_keeps_at_most_the_package_limit_and_says_it_is_incomplete() {
    let mut collector = Collector::new();
    let packages = (0..=MAX_PACKAGES)
        .map(|index| listed("p", Some(&format!("pkg:cargo/crate@1.{index}"))))
        .collect();
    collector.absorb(sbom(packages));
    assert_eq!(collector.packages.len(), MAX_PACKAGES);
    assert!(!collector.complete);
}
