use base64::Engine;
use sha1::{Digest, Sha1};

use crate::archive::{self, ArchiveError};
use crate::index::IndexEntry;

pub const PKGINFO: &str = ".PKGINFO";
pub const NOARCH: &str = "noarch";
const CONTROL_LIMIT: u64 = 64 << 20;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PackageError {
    #[error(transparent)]
    Archive(#[from] ArchiveError),
    #[error("the control segment of the package has no .PKGINFO")]
    NoPkgInfo,
    #[error("the .PKGINFO of the package has no {0}")]
    Missing(&'static str),
    #[error("the .PKGINFO of the package is not valid: {0}")]
    Invalid(String),
    #[error("APK v3 packages are not supported; build the package in the v2 format of abuild")]
    Version3,
}

impl PackageError {
    pub fn needs_more(&self) -> bool {
        matches!(self, PackageError::Archive(ArchiveError::Truncated))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PkgInfo {
    pub fields: Vec<(String, String)>,
}

impl PkgInfo {
    pub fn parse(text: &str) -> Result<PkgInfo, PackageError> {
        let mut fields = Vec::new();
        for line in text.lines() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.starts_with('#') {
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
        for (key, what) in [("pkgname", "pkgname"), ("pkgver", "pkgver")] {
            if info.first(key).is_none_or(str::is_empty) {
                return Err(PackageError::Missing(what));
            }
        }
        let name = info.name();
        if name.contains('/') || name.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(PackageError::Invalid(format!(
                "the package name {name:?} is not valid"
            )));
        }
        let version = info.version();
        if version.contains('/') || version.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(PackageError::Invalid(format!(
                "the package version {version:?} is not valid"
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

    pub fn all(&self, key: &str) -> Vec<&str> {
        self.fields
            .iter()
            .filter(|(field, _)| field == key)
            .map(|(_, value)| value.as_str())
            .collect()
    }

    pub fn name(&self) -> &str {
        self.first("pkgname").unwrap_or_default()
    }

    pub fn version(&self) -> &str {
        self.first("pkgver").unwrap_or_default()
    }

    pub fn arch(&self) -> &str {
        self.first("arch")
            .filter(|arch| !arch.is_empty())
            .unwrap_or(NOARCH)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub checksum: String,
    pub info: PkgInfo,
    pub signatures: Vec<String>,
}

impl Package {
    pub fn file_name(&self) -> String {
        file_name(self.info.name(), self.info.version())
    }

    pub fn entry(&self, size: u64) -> IndexEntry {
        IndexEntry::from_package(self, size)
    }
}

pub fn file_name(name: &str, version: &str) -> String {
    format!("{name}-{version}.apk")
}

pub fn checksum(control: &[u8]) -> String {
    format!(
        "Q1{}",
        base64::engine::general_purpose::STANDARD.encode(Sha1::digest(control))
    )
}

pub fn read(data: &[u8]) -> Result<Package, PackageError> {
    if data.starts_with(b"ADB") {
        return Err(PackageError::Version3);
    }
    let mut offset = 0;
    let mut signatures = Vec::new();
    loop {
        if offset >= data.len() {
            return Err(PackageError::Archive(ArchiveError::Truncated));
        }
        let member = archive::member(&data[offset..], CONTROL_LIMIT)?;
        let entries = archive::entries(&member.content, |name| name == PKGINFO)?;
        if !entries.is_empty()
            && entries
                .iter()
                .all(|entry| archive::is_signature(&entry.name))
        {
            signatures.extend(entries.into_iter().map(|entry| entry.name));
            offset += member.bytes.len();
            continue;
        }
        let info = entries
            .iter()
            .find(|entry| entry.name == PKGINFO)
            .ok_or(PackageError::NoPkgInfo)?;
        let text = String::from_utf8(info.data.clone())
            .map_err(|_| PackageError::Invalid("it is not UTF-8 text".to_owned()))?;
        return Ok(Package {
            checksum: checksum(member.bytes),
            info: PkgInfo::parse(&text)?,
            signatures,
        });
    }
}

pub fn parse_file_name(file: &str) -> Option<(String, String)> {
    let stem = file.strip_suffix(".apk")?;
    let mut parts = stem.rsplitn(3, '-');
    let release = parts.next()?;
    let version = parts.next()?;
    let name = parts.next()?;
    let numbered = release
        .strip_prefix('r')
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()));
    if name.is_empty() || version.is_empty() || !numbered {
        return None;
    }
    if !version.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    Some((name.to_owned(), format!("{version}-{release}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn pkginfo_lines_are_read_in_order() {
        let info = PkgInfo::parse(
            "# Generated by abuild\npkgname = hello\npkgver = 1.0-r0\narch = x86_64\ndepend = so:libc.musl-x86_64.so.1\ndepend = busybox\nprovides =\n",
        )
        .unwrap();
        assert_eq!(info.name(), "hello");
        assert_eq!(info.version(), "1.0-r0");
        assert_eq!(info.arch(), "x86_64");
        assert_eq!(info.all("depend"), ["so:libc.musl-x86_64.so.1", "busybox"]);
        assert_eq!(info.first("provides"), Some(""));
        assert_eq!(
            PkgInfo::parse("pkgver = 1\n").unwrap_err(),
            PackageError::Missing("pkgname")
        );
        assert!(matches!(
            PkgInfo::parse("pkgname = a b\npkgver = 1\n").unwrap_err(),
            PackageError::Invalid(_)
        ));
        assert!(matches!(
            PkgInfo::parse("pkgname hello\n").unwrap_err(),
            PackageError::Invalid(_)
        ));
        assert_eq!(
            PkgInfo::parse("pkgname = a\npkgver = 1\n").unwrap().arch(),
            NOARCH
        );
    }

    #[test]
    fn the_checksum_covers_the_compressed_control_stream() {
        let built = testing::build(&testing::Spec::new("hello", "1.0-r0", "x86_64"));
        let package = read(&built.bytes).unwrap();
        assert_eq!(package.checksum, checksum(&built.control));
        assert!(package.checksum.starts_with("Q1"));
        assert_eq!(package.checksum.len(), 30);
        assert_eq!(package.info.name(), "hello");
        assert_eq!(package.signatures, [".SIGN.RSA.packager.rsa.pub"]);
        let unsigned = testing::build(&testing::Spec {
            signed: false,
            ..testing::Spec::new("hello", "1.0-r0", "x86_64")
        });
        let package = read(&unsigned.bytes).unwrap();
        assert_eq!(package.checksum, checksum(&unsigned.control));
        assert!(package.signatures.is_empty());
    }

    #[test]
    fn broken_packages_are_refused() {
        let built = testing::build(&testing::Spec::new("hello", "1.0-r0", "x86_64"));
        let cut = built.bytes.len() - built.data.len() - 10;
        assert!(read(&built.bytes[..cut]).unwrap_err().needs_more());
        assert_eq!(read(b"ADBd\0\0\0\0").unwrap_err(), PackageError::Version3);
        assert_eq!(
            read(b"not a package").unwrap_err(),
            PackageError::Archive(ArchiveError::NotGzip)
        );
        let empty = archive::gzip(&archive::tar(&[("README", b"x")], 0, false).unwrap());
        assert_eq!(read(&empty).unwrap_err(), PackageError::NoPkgInfo);
    }

    #[test]
    fn file_names_give_name_and_version() {
        assert_eq!(
            parse_file_name("py3-foo-bar-1.2.3_rc1-r10.apk"),
            Some(("py3-foo-bar".to_owned(), "1.2.3_rc1-r10".to_owned()))
        );
        assert_eq!(
            parse_file_name("musl-1.2.5-r3.apk"),
            Some(("musl".to_owned(), "1.2.5-r3".to_owned()))
        );
        assert_eq!(parse_file_name("APKINDEX.tar.gz"), None);
        assert_eq!(parse_file_name("hello-world.apk"), None);
        assert_eq!(parse_file_name("hello-x-r1.apk"), None);
        assert_eq!(file_name("hello", "1.0-r0"), "hello-1.0-r0.apk");
    }
}
