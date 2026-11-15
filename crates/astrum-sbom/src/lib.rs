mod cyclonedx;
mod ecosystem;
mod envelope;
mod error;
mod guard;
mod sink;
mod spdx;
mod statement;
mod text;

use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

use crate::sink::Sink;
use crate::statement::Predicate;
use crate::text::{Text, Whole};

pub use crate::error::Error;

pub const MAX_INPUT_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_PACKAGES: usize = 200_000;
pub const MAX_DEPTH: usize = 64;
pub const MAX_STRING_BYTES: usize = 1024;
pub const MAX_LICENSES: usize = 32;

const IN_TOTO_STATEMENT: &str = "https://in-toto.io/Statement/";
const SPDX_PREDICATE: &str = "https://spdx.dev/Document";
const CYCLONEDX_PREDICATE: &str = "https://cyclonedx.org/bom";
const CYCLONEDX_VERSIONS: [&str; 6] = ["1.2", "1.3", "1.4", "1.5", "1.6", "1.7"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Spdx,
    CycloneDx,
}

impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Format::Spdx => "spdx",
            Format::CycloneDx => "cyclonedx",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub purl: Option<String>,
    pub licenses: Vec<String>,
    pub ecosystem: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sbom {
    pub format: Format,
    pub packages: Vec<Package>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Probe {
    #[serde(rename = "spdxVersion")]
    spdx_version: Text,
    #[serde(rename = "bomFormat")]
    bom_format: Text,
    #[serde(rename = "_type")]
    statement_type: Text,
    #[serde(rename = "predicateType")]
    predicate_type: Text,
    #[serde(rename = "payloadType")]
    payload_type: Text,
    payload: Whole,
}

enum Shape {
    Spdx,
    CycloneDx,
    Statement,
    Envelope,
}

impl Probe {
    fn shape(&self) -> Result<Shape, Error> {
        if self.spdx_version.0.is_some() {
            Ok(Shape::Spdx)
        } else if self.bom_format.0.is_some() {
            Ok(Shape::CycloneDx)
        } else if self.statement_type.0.is_some() {
            Ok(Shape::Statement)
        } else if self.payload_type.0.is_some() || self.payload.0.is_some() {
            Ok(Shape::Envelope)
        } else {
            Err(Error::unsupported("the document has an unknown shape"))
        }
    }
}

fn names(declared: &str, base: &str) -> bool {
    declared == base
        || declared
            .strip_prefix(base)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn read<'de, S: DeserializeSeed<'de>>(
    bytes: &'de [u8],
    seed: S,
) -> Result<S::Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = seed.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

fn extract<'de, S: DeserializeSeed<'de>>(
    bytes: &'de [u8],
    seed: S,
    wrapped: bool,
) -> Result<Option<S::Value>, serde_json::Error> {
    match wrapped {
        true => read(bytes, Predicate(seed)),
        false => read(bytes, seed).map(Some),
    }
}

fn failure(error: serde_json::Error, overflow: bool) -> Error {
    use serde_json::error::Category;
    if overflow {
        return Error::TooManyPackages;
    }
    match error.classify() {
        Category::Eof => Error::malformed("the JSON is truncated"),
        Category::Syntax => Error::malformed("the JSON is invalid"),
        Category::Data => Error::malformed("the JSON has an unexpected structure"),
        Category::Io => Error::malformed("the document cannot be read"),
    }
}

fn settle<T>(outcome: Result<Option<T>, serde_json::Error>, sink: &Sink) -> Result<T, Error> {
    match outcome {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(Error::malformed("the statement has no predicate")),
        Err(error) => Err(failure(error, sink.overflow)),
    }
}

fn probe(bytes: &[u8]) -> Result<Probe, Error> {
    guard::object(bytes)?;
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let probe = Probe::deserialize(&mut deserializer).map_err(|error| failure(error, false))?;
    deserializer.end().map_err(|error| failure(error, false))?;
    Ok(probe)
}

fn spdx_document(bytes: &[u8], wrapped: bool) -> Result<Sbom, Error> {
    let mut sink = Sink::default();
    let outcome = extract(bytes, spdx::Document(&mut sink), wrapped);
    match settle(outcome, &sink)? {
        None => return Err(Error::unsupported("the document is not an SPDX document")),
        Some(version) if !version.starts_with("SPDX-2.") => {
            return Err(Error::unsupported("the SPDX version"));
        }
        Some(_) => {}
    }
    Ok(Sbom {
        format: Format::Spdx,
        packages: sink.packages,
    })
}

fn cyclonedx_document(bytes: &[u8], wrapped: bool) -> Result<Sbom, Error> {
    let mut sink = Sink::default();
    let outcome = extract(bytes, cyclonedx::Document(&mut sink), wrapped);
    let header = settle(outcome, &sink)?;
    match (header.format.as_deref(), header.version.as_deref()) {
        (None, _) => return Err(Error::unsupported("the document is not a CycloneDX BOM")),
        (Some(format), _) if format != "CycloneDX" => {
            return Err(Error::unsupported("the BOM format"));
        }
        (_, None) => return Err(Error::malformed("the CycloneDX version is missing")),
        (_, Some(version)) if !CYCLONEDX_VERSIONS.contains(&version) => {
            return Err(Error::unsupported("the CycloneDX version"));
        }
        _ => {}
    }
    Ok(Sbom {
        format: Format::CycloneDx,
        packages: sink.packages,
    })
}

fn statement(bytes: &[u8], probe: &Probe) -> Result<Sbom, Error> {
    let declared = probe.statement_type.0.as_deref().unwrap_or_default();
    if !declared.starts_with(IN_TOTO_STATEMENT) {
        return Err(Error::unsupported("the statement type"));
    }
    let predicate = probe.predicate_type.0.as_deref().unwrap_or_default();
    if names(predicate, SPDX_PREDICATE) {
        spdx_document(bytes, true)
    } else if names(predicate, CYCLONEDX_PREDICATE) {
        cyclonedx_document(bytes, true)
    } else {
        Err(Error::unsupported("the predicate type"))
    }
}

pub fn parse(bytes: &[u8]) -> Result<Sbom, Error> {
    guard::input(bytes)?;
    let bytes = guard::without_bom(bytes);
    let outer = probe(bytes)?;
    match outer.shape()? {
        Shape::Spdx => spdx_document(bytes, false),
        Shape::CycloneDx => cyclonedx_document(bytes, false),
        Shape::Statement => statement(bytes, &outer),
        Shape::Envelope => {
            let decoded = envelope::open(&outer.payload_type, &outer.payload)?;
            guard::depth(&decoded)?;
            let inner = guard::without_bom(&decoded);
            let content = probe(inner)?;
            match content.shape()? {
                Shape::Statement => statement(inner, &content),
                Shape::Envelope => Err(Error::unsupported("an envelope inside an envelope")),
                Shape::Spdx | Shape::CycloneDx => {
                    Err(Error::unsupported("the envelope does not hold a statement"))
                }
            }
        }
    }
}
