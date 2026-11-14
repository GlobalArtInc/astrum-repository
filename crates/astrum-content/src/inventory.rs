#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageRef {
    pub ecosystem: String,
    pub name: String,
    pub version: String,
    pub purl: String,
}

pub const LICENSE_TEXT_LIMIT: usize = 1024;

pub fn cap_license(text: &str) -> &str {
    let mut end = text.len().min(LICENSE_TEXT_LIMIT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
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
    Layers,
    SbomAndLayers,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Metadata => "metadata",
            Source::Sbom => "sbom",
            Source::Index => "index",
            Source::Layers => "layers",
            Source::SbomAndLayers => "sbom+layers",
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
