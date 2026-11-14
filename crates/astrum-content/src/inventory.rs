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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    #[default]
    Metadata,
    Sbom,
    Index,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Metadata => "metadata",
            Source::Sbom => "sbom",
            Source::Index => "index",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    pub key: String,
    pub packages: Vec<PackageRef>,
    pub license: Option<LicenseFact>,
    pub complete: bool,
    pub source: Source,
    pub digest: Option<String>,
}
