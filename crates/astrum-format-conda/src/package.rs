use std::io::{self, Read};

use astrum_content::zip;
use serde_json::{Map, Value};

use crate::paths::PackageType;

const MAX_INFO_FILE: u64 = 16 * 1024 * 1024;
const MAX_INFO_ARCHIVE: u64 = 256 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("{0}")]
    Invalid(String),
    #[error("the archive cannot be read: {0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Zip(#[from] zip::ZipError),
}

fn invalid(message: impl Into<String>) -> PackageError {
    PackageError::Invalid(message.into())
}

/// The metadata conda reads from the `info/` folder of a package.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Info {
    pub index: Map<String, Value>,
    pub about: Option<Map<String, Value>>,
    pub run_exports: Option<Value>,
}

impl Info {
    pub fn text(&self, key: &str) -> &str {
        self.index.get(key).and_then(Value::as_str).unwrap_or("")
    }

    pub fn name(&self) -> &str {
        self.text("name")
    }

    pub fn version(&self) -> &str {
        self.text("version")
    }

    pub fn build(&self) -> &str {
        self.text("build")
    }

    pub fn build_number(&self) -> i64 {
        self.index
            .get("build_number")
            .and_then(Value::as_i64)
            .unwrap_or(0)
    }

    /// The subdirectory the package was built for: its `subdir`, else `noarch` for noarch
    /// packages, else the one its `platform` and `arch` name.
    pub fn subdir(&self) -> Option<String> {
        if let Some(subdir) = self.index.get("subdir").and_then(Value::as_str)
            && !subdir.is_empty()
        {
            return Some(subdir.to_owned());
        }
        if self
            .index
            .get("noarch")
            .is_some_and(|noarch| !noarch.is_null() && noarch != &Value::Bool(false))
        {
            return Some("noarch".to_owned());
        }
        let platform = self.text("platform");
        let arch = self.text("arch");
        let found = match (platform, arch) {
            ("linux", "x86_64") => "linux-64",
            ("linux", "x86") => "linux-32",
            ("linux", "aarch64") => "linux-aarch64",
            ("linux", "ppc64le") => "linux-ppc64le",
            ("linux", "s390x") => "linux-s390x",
            ("linux", "armv6l") => "linux-armv6l",
            ("linux", "armv7l") => "linux-armv7l",
            ("osx", "x86_64") => "osx-64",
            ("osx", "arm64") => "osx-arm64",
            ("win", "x86_64") => "win-64",
            ("win", "x86") => "win-32",
            ("win", "arm64") => "win-arm64",
            _ => return None,
        };
        Some(found.to_owned())
    }

    fn check(self) -> Result<Info, PackageError> {
        for key in ["name", "version", "build"] {
            match self.index.get(key) {
                Some(Value::String(value)) if !value.is_empty() => {}
                _ => {
                    return Err(invalid(format!(
                        "info/index.json has no {key} of the package"
                    )));
                }
            }
        }
        if self.name().contains(['/', ' ']) || self.version().contains(['-', '/', ' ']) {
            return Err(invalid(
                "info/index.json names a package or version conda cannot use",
            ));
        }
        if self.build().contains(['-', '/', ' ']) {
            return Err(invalid("info/index.json names a build conda cannot use"));
        }
        Ok(self)
    }
}

fn object(name: &str, bytes: &[u8]) -> Result<Map<String, Value>, PackageError> {
    match serde_json::from_slice::<Value>(bytes) {
        Ok(Value::Object(object)) => Ok(object),
        Ok(_) => Err(invalid(format!("{name} is not a JSON object"))),
        Err(error) => Err(invalid(format!("{name} is not valid JSON: {error}"))),
    }
}

/// Reads `info/index.json`, `info/about.json` and `info/run_exports.json` from a tar stream,
/// stopping after the `info/` entries once the index is found. A stream cut short after the
/// index still answers what was read.
pub fn read_tar(reader: impl Read) -> Result<Info, PackageError> {
    let mut archive = tar::Archive::new(reader);
    let mut index = None;
    let mut about = None;
    let mut run_exports = None;
    let entries = archive.entries()?;
    for entry in entries {
        let mut entry = match entry {
            Ok(entry) => entry,
            Err(error) if index.is_some() => {
                tracing::debug!(%error, "a conda package is read only up to its index");
                break;
            }
            Err(error) => return Err(error.into()),
        };
        let path = match entry.path() {
            Ok(path) => path.to_string_lossy().trim_start_matches("./").to_owned(),
            Err(_) => continue,
        };
        if !path.starts_with("info/") {
            if index.is_some() {
                break;
            }
            continue;
        }
        let wanted = matches!(
            path.as_str(),
            "info/index.json" | "info/about.json" | "info/run_exports.json"
        );
        if !wanted {
            continue;
        }
        let mut bytes = Vec::new();
        let read = (&mut entry).take(MAX_INFO_FILE).read_to_end(&mut bytes);
        if let Err(error) = read {
            if index.is_some() {
                break;
            }
            return Err(error.into());
        }
        match path.as_str() {
            "info/index.json" => index = Some(object(&path, &bytes)?),
            "info/about.json" => about = object(&path, &bytes).ok(),
            _ => run_exports = serde_json::from_slice::<Value>(&bytes).ok(),
        }
    }
    let index = index.ok_or_else(|| invalid("the package has no info/index.json"))?;
    Info {
        index,
        about,
        run_exports,
    }
    .check()
}

pub fn read_tar_bz2(reader: impl Read) -> Result<Info, PackageError> {
    read_tar(bzip2::read::MultiBzDecoder::new(reader))
}

fn decompress_info(name: &str, data: &[u8]) -> Result<Info, PackageError> {
    if name.ends_with(".tar.zst") {
        let decoder = ruzstd::decoding::StreamingDecoder::new(data)
            .map_err(|error| invalid(format!("{name} cannot be decompressed: {error}")))?;
        read_tar(decoder.take(MAX_INFO_ARCHIVE))
    } else if name.ends_with(".tar.bz2") {
        read_tar(bzip2::read::MultiBzDecoder::new(data).take(MAX_INFO_ARCHIVE))
    } else {
        read_tar(data)
    }
}

/// Reads the metadata of a `.conda` package: a zip holding `info-<stem>.tar.zst` next to the
/// package payload.
pub async fn read_conda(source: &dyn zip::Source) -> Result<Info, PackageError> {
    let entries = zip::entries(source).await?;
    let entry = entries
        .iter()
        .find(|entry| {
            entry.name.starts_with("info-")
                && [".tar.zst", ".tar.bz2", ".tar"]
                    .iter()
                    .any(|suffix| entry.name.ends_with(suffix))
        })
        .ok_or_else(|| invalid("the .conda archive has no info-*.tar.zst"))?;
    let data = zip::read(source, entry, MAX_INFO_ARCHIVE).await?;
    let name = entry.name.clone();
    tokio::task::spawn_blocking(move || decompress_info(&name, &data))
        .await
        .map_err(|error| PackageError::Io(io::Error::other(error)))?
}

pub fn kind_of(file: Option<&str>, prefix: &[u8]) -> Option<PackageType> {
    file.and_then(PackageType::of)
        .or_else(|| PackageType::sniff(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use serde_json::json;

    fn index() -> Value {
        json!({"name": "hello", "version": "1.0", "build": "py_0", "build_number": 0, "depends": ["python >=3"], "subdir": "noarch", "noarch": "python"})
    }

    #[test]
    fn tar_bz2_packages_answer_their_index() {
        let package = testing::tar_bz2(&index(), Some(&json!({"summary": "Say hello"})));
        let info = read_tar_bz2(package.as_slice()).unwrap();
        assert_eq!(info.name(), "hello");
        assert_eq!(info.subdir().as_deref(), Some("noarch"));
        assert_eq!(info.about.unwrap()["summary"], "Say hello");
        let empty = testing::tar(&[("bin/hello", b"#!/bin/sh\n".as_slice())]);
        let refused = read_tar(empty.as_slice()).unwrap_err();
        assert!(
            refused.to_string().contains("no info/index.json"),
            "{refused}"
        );
    }

    #[test]
    fn the_index_is_found_after_the_payload_and_a_cut_stream_still_answers() {
        let index = index().to_string();
        let late = testing::tar(&[
            ("share/hello/README.txt", b"hello".as_slice()),
            ("info/index.json", index.as_bytes()),
        ]);
        assert_eq!(read_tar(late.as_slice()).unwrap().name(), "hello");
        let mut early = testing::tar(&[
            ("info/index.json", index.as_bytes()),
            ("share/hello/big.bin", vec![7u8; 64 * 1024].as_slice()),
        ]);
        early.truncate(2048);
        assert_eq!(read_tar(early.as_slice()).unwrap().build(), "py_0");
    }

    #[tokio::test]
    async fn conda_packages_answer_their_index() {
        let package = testing::conda(&index(), None);
        let info = read_conda(&zip::Memory(package)).await.unwrap();
        assert_eq!(info.version(), "1.0");
        assert_eq!(info.build(), "py_0");
        assert!(info.about.is_none());
        let broken = zip::write(&[("metadata.json", b"{}".as_slice())]);
        assert!(read_conda(&zip::Memory(broken)).await.is_err());
    }

    #[test]
    fn subdirs_come_from_old_platform_fields() {
        let info = Info {
            index: json!({"name": "a", "version": "1", "build": "0", "platform": "linux", "arch": "x86_64"})
                .as_object()
                .unwrap()
                .clone(),
            ..Info::default()
        };
        assert_eq!(info.subdir().as_deref(), Some("linux-64"));
        let bad = Info {
            index: json!({"name": "a", "version": "1-2", "build": "0"})
                .as_object()
                .unwrap()
                .clone(),
            ..Info::default()
        };
        assert!(bad.check().is_err());
        assert_eq!(kind_of(None, b"PK\x03\x04rest"), Some(PackageType::Conda));
        assert_eq!(
            kind_of(Some("x-1-0.tar.bz2"), b""),
            Some(PackageType::TarBz2)
        );
    }
}
