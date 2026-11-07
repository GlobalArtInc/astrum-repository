#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageRef {
    pub ecosystem: String,
    pub name: String,
    pub version: String,
    pub purl: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LicenseFact {
    pub declared: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    pub key: String,
    pub packages: Vec<PackageRef>,
    pub license: Option<LicenseFact>,
    pub complete: bool,
}
