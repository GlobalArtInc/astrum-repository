use std::io::Read;

use serde_json::{Value, json};

use crate::archive::{self, Compression};
use crate::paths;

pub const PKGINFO: &str = ".PKGINFO";
const INFO_LIMIT: u64 = 1 << 20;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("{0}")]
    Archive(String),
    #[error("the package archive has no .PKGINFO")]
    NoPkgInfo,
    #[error("the .PKGINFO of the package has no {0}")]
    Missing(&'static str),
    #[error("the .PKGINFO of the package is not valid: {0}")]
    Invalid(String),
}

fn damaged(error: impl std::fmt::Display) -> PackageError {
    PackageError::Archive(format!("the package archive cannot be read: {error}"))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PkgInfo {
    pub fields: Vec<(String, String)>,
}

impl PkgInfo {
    pub fn parse(text: &str) -> Result<PkgInfo, PackageError> {
        let mut fields = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(PackageError::Invalid(format!(
                    "the line {line:?} is not key = value"
                )));
            };
            let key = key.trim();
            if key.is_empty() || key.chars().any(|c| c.is_whitespace() || c.is_control()) {
                return Err(PackageError::Invalid(format!(
                    "the line {line:?} has no valid key"
                )));
            }
            fields.push((key.to_owned(), value.trim().to_owned()));
        }
        let info = PkgInfo { fields };
        for key in ["pkgname", "pkgver", "arch"] {
            if info.first(key).is_none_or(str::is_empty) {
                return Err(PackageError::Missing(key));
            }
        }
        let named = paths::PackageFile::parse(&info.file_name("")).is_some_and(|file| {
            file.name == info.name() && file.version == info.version() && file.arch == info.arch()
        });
        if !named {
            return Err(PackageError::Invalid(format!(
                "the name {:?}, version {:?} and architecture {:?} do not form a package file name",
                info.name(),
                info.version(),
                info.arch()
            )));
        }
        Ok(info)
    }

    pub fn first(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(field, _)| field == key)
            .map(|(_, value)| value.as_str())
    }

    pub fn all(&self, key: &str) -> Vec<String> {
        self.fields
            .iter()
            .filter(|(field, value)| field == key && !value.is_empty())
            .map(|(_, value)| value.clone())
            .collect()
    }

    pub fn name(&self) -> &str {
        self.first("pkgname").unwrap_or_default()
    }

    pub fn version(&self) -> &str {
        self.first("pkgver").unwrap_or_default()
    }

    pub fn arch(&self) -> &str {
        self.first("arch").unwrap_or_default()
    }

    pub fn file_name(&self, extension: &str) -> String {
        paths::package_file_name(self.name(), self.version(), self.arch(), extension)
    }

    pub fn attributes(&self) -> Value {
        let text = |key: &str| {
            self.first(key)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        let number = |key: &str| {
            self.first(key)
                .and_then(|value| value.trim().parse::<u64>().ok())
        };
        let mut attributes = json!({
            "name": self.name(),
            "base": text("pkgbase"),
            "version": self.version(),
            "description": text("pkgdesc"),
            "url": text("url"),
            "architecture": self.arch(),
            "buildDate": number("builddate"),
            "packager": text("packager"),
            "installedSize": number("size"),
            "license": self.all("license"),
            "groups": self.all("group"),
            "replaces": self.all("replaces"),
            "conflicts": self.all("conflict"),
            "provides": self.all("provides"),
            "depends": self.all("depend"),
            "optDepends": self.all("optdepend"),
            "makeDepends": self.all("makedepend"),
            "checkDepends": self.all("checkdepend"),
        });
        if let Some(map) = attributes.as_object_mut() {
            map.retain(|_, value| match value {
                Value::Null => false,
                Value::Array(items) => !items.is_empty(),
                _ => true,
            });
        }
        attributes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contents {
    pub info: PkgInfo,
    pub compression: Compression,
    pub files: Vec<String>,
}

fn data_file(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.')
}

pub fn read(input: impl Read, with_files: bool) -> Result<Contents, PackageError> {
    let opened = archive::open(input).map_err(damaged)?;
    let compression = opened.compression;
    let incomplete = opened.incomplete;
    let mut tar = tar::Archive::new(opened.reader);
    let mut info = None;
    let mut files = Vec::new();
    for entry in tar.entries().map_err(damaged)? {
        let mut entry = entry.map_err(damaged)?;
        if archive::is_extension(&entry) {
            continue;
        }
        let name = archive::entry_name(&entry);
        if name == PKGINFO {
            let mut text = String::new();
            (&mut entry)
                .take(INFO_LIMIT)
                .read_to_string(&mut text)
                .map_err(|_| PackageError::Invalid("it is not UTF-8 text".to_owned()))?;
            info = Some(PkgInfo::parse(&text)?);
            if !with_files {
                break;
            }
        } else if with_files && data_file(&name) {
            let directory = entry.header().entry_type().is_dir();
            files.push(match directory && !name.ends_with('/') {
                true => format!("{name}/"),
                false => name,
            });
        }
    }
    if with_files && incomplete {
        return Err(damaged("the xz stream ends early"));
    }
    let info = info.ok_or(PackageError::NoPkgInfo)?;
    files.sort();
    files.dedup();
    Ok(Contents {
        info,
        compression,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{self, Spec};

    #[test]
    fn pkginfo_lines_are_read_in_order() {
        let info = PkgInfo::parse(
            "# Generated by makepkg 7.0.0\npkgname = hello\npkgbase = hello\nxdata = pkgtype=pkg\npkgver = 1:1.0-2\npkgdesc = Says hello = world\narch = x86_64\ndepend = glibc\ndepend = bash>=5\nlicense = MIT\nsize = 120\n",
        )
        .unwrap();
        assert_eq!(info.name(), "hello");
        assert_eq!(info.version(), "1:1.0-2");
        assert_eq!(info.first("pkgdesc"), Some("Says hello = world"));
        assert_eq!(info.all("depend"), ["glibc", "bash>=5"]);
        assert_eq!(info.file_name(".zst"), "hello-1:1.0-2-x86_64.pkg.tar.zst");
        let attributes = info.attributes();
        assert_eq!(attributes["installedSize"], 120);
        assert_eq!(attributes["depends"], json!(["glibc", "bash>=5"]));
        assert!(attributes.get("groups").is_none());
        assert!(attributes.get("url").is_none());
        assert!(matches!(
            PkgInfo::parse("pkgver = 1-1\narch = any\n").unwrap_err(),
            PackageError::Missing("pkgname")
        ));
        assert!(matches!(
            PkgInfo::parse("pkgname = a b\npkgver = 1-1\narch = any\n").unwrap_err(),
            PackageError::Invalid(_)
        ));
        assert!(matches!(
            PkgInfo::parse("pkgname = a\npkgver = 1\narch = any\n").unwrap_err(),
            PackageError::Invalid(_)
        ));
        assert!(matches!(
            PkgInfo::parse("pkgname hello\n").unwrap_err(),
            PackageError::Invalid(_)
        ));
    }

    #[test]
    fn packages_give_their_pkginfo_and_data_files() {
        for compression in [
            Compression::Zstd,
            Compression::Xz,
            Compression::Gzip,
            Compression::None,
        ] {
            let built = testing::build(&Spec {
                compression,
                ..Spec::new("hello", "1.0-1", "x86_64")
            });
            let contents = read(built.as_slice(), true).unwrap();
            assert_eq!(contents.compression, compression);
            assert_eq!(contents.info.name(), "hello");
            assert_eq!(
                contents.files,
                [
                    "usr/",
                    "usr/bin/",
                    "usr/bin/hello",
                    "usr/share/",
                    "usr/share/doc/",
                    "usr/share/doc/hello/",
                    "usr/share/doc/hello/README"
                ],
                "{compression:?}"
            );
            let head = read(built.as_slice(), false).unwrap();
            assert!(head.files.is_empty());
        }
    }

    #[test]
    fn broken_packages_are_refused() {
        let built = testing::build(&Spec::new("hello", "1.0-1", "x86_64"));
        assert!(read(&built[..built.len() / 2], true).is_err());
        assert!(matches!(
            read(&b"not a package"[..], false).unwrap_err(),
            PackageError::Archive(_)
        ));
        let empty = testing::tar_zst(&[("usr/bin/x", b"x".as_slice())]);
        assert!(matches!(
            read(empty.as_slice(), false).unwrap_err(),
            PackageError::NoPkgInfo
        ));
    }
}
