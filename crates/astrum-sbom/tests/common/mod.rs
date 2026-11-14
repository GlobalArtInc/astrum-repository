#![allow(dead_code)]

use astrum_sbom::{Package, Sbom};
use serde_json::Value;

pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

pub fn fixture_json(name: &str) -> Value {
    serde_json::from_slice(&fixture(name)).unwrap()
}

pub fn find<'a>(sbom: &'a Sbom, name: &str) -> &'a Package {
    sbom.packages
        .iter()
        .find(|package| package.name == name)
        .unwrap_or_else(|| panic!("no package named {name}"))
}

pub fn spdx(packages: &str) -> Vec<u8> {
    format!(r#"{{"spdxVersion":"SPDX-2.3","packages":[{packages}]}}"#).into_bytes()
}

pub fn cyclonedx(components: &str) -> Vec<u8> {
    format!(r#"{{"bomFormat":"CycloneDX","specVersion":"1.5","components":[{components}]}}"#)
        .into_bytes()
}
