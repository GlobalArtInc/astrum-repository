use std::cmp::Ordering;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use sha2::{Digest, Sha256};

pub const UNIVERSAL: &str = "universal";
pub const WEB: &str = "web";

pub const PLATFORMS: &[&str] = &[
    "universal",
    "web",
    "win32-x64",
    "win32-ia32",
    "win32-arm64",
    "linux-x64",
    "linux-arm64",
    "linux-armhf",
    "alpine-x64",
    "alpine-arm64",
    "darwin-x64",
    "darwin-arm64",
];

const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'[')
    .add(b']');

const PATH: &AsciiSet = &SEGMENT.remove(b'/');

pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    name.len() <= 255
        && chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn valid_version(version: &str) -> bool {
    version.len() <= 128 && semver::Version::parse(version).is_ok()
}

pub fn is_platform(value: &str) -> bool {
    PLATFORMS.contains(&value)
}

pub fn platform_rank(platform: &str) -> usize {
    PLATFORMS
        .iter()
        .position(|known| *known == platform)
        .unwrap_or(PLATFORMS.len())
}

pub fn compare_versions(a: &str, b: &str) -> Ordering {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(a), Ok(b)) => a.cmp(&b),
        (Ok(_), Err(_)) => Ordering::Greater,
        (Err(_), Ok(_)) => Ordering::Less,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

pub fn has_prerelease_tag(version: &str) -> bool {
    semver::Version::parse(version).is_ok_and(|parsed| !parsed.pre.is_empty())
}

pub fn split_id(id: &str) -> Option<(&str, &str)> {
    let (namespace, name) = id.split_once('.').or_else(|| id.split_once('/'))?;
    (valid_name(namespace) && valid_name(name)).then_some((namespace, name))
}

pub fn id(namespace: &str, name: &str) -> String {
    format!("{namespace}.{name}")
}

pub fn key(namespace: &str, name: &str) -> String {
    id(namespace, name).to_lowercase()
}

fn derived(kind: &str, value: &str) -> String {
    let digest = Sha256::digest(format!("openvsx:{kind}:{}", value.to_lowercase()).as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    uuid::Builder::from_sha1_bytes(bytes)
        .into_uuid()
        .hyphenated()
        .to_string()
}

pub fn extension_uuid(namespace: &str, name: &str) -> String {
    derived("extension", &id(namespace, name))
}

pub fn namespace_uuid(namespace: &str) -> String {
    derived("namespace", namespace)
}

pub fn encode(segment: &str) -> String {
    utf8_percent_encode(segment, SEGMENT).to_string()
}

pub fn encode_path(path: &str) -> String {
    utf8_percent_encode(path, PATH).to_string()
}

pub fn vsix_stem(namespace: &str, name: &str, version: &str, platform: &str) -> String {
    match platform {
        UNIVERSAL => format!("{namespace}.{name}-{version}"),
        _ => format!("{namespace}.{name}-{version}@{platform}"),
    }
}

pub fn vsix_name(namespace: &str, name: &str, version: &str, platform: &str) -> String {
    format!("{}.vsix", vsix_stem(namespace, name, version, platform))
}

pub fn sha256_name(namespace: &str, name: &str, version: &str, platform: &str) -> String {
    format!("{}.sha256", vsix_stem(namespace, name, version, platform))
}

pub fn version_dir(namespace: &str, name: &str, platform: &str, version: &str) -> String {
    match platform {
        UNIVERSAL => format!("/api/{namespace}/{name}/{version}"),
        _ => format!("/api/{namespace}/{name}/{platform}/{version}"),
    }
}

pub fn file_path(namespace: &str, name: &str, platform: &str, version: &str, file: &str) -> String {
    format!(
        "{}/file/{file}",
        version_dir(namespace, name, platform, version)
    )
}

pub fn vsix_path(namespace: &str, name: &str, platform: &str, version: &str) -> String {
    file_path(
        namespace,
        name,
        platform,
        version,
        &vsix_name(namespace, name, version, platform),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRef {
    pub namespace: String,
    pub name: String,
    pub platform: String,
    pub version: String,
    pub file: String,
}

impl FileRef {
    pub fn parse(path: &str) -> Option<FileRef> {
        let parts: Vec<&str> = path.trim_start_matches('/').splitn(7, '/').collect();
        let (namespace, name, platform, version, file) = match parts.as_slice() {
            ["api", namespace, name, version, "file", file] => {
                (*namespace, *name, UNIVERSAL, *version, *file)
            }
            ["api", namespace, name, platform, version, "file", file] if is_platform(platform) => {
                (*namespace, *name, *platform, *version, *file)
            }
            _ => return None,
        };
        (valid_name(namespace)
            && valid_name(name)
            && !version.is_empty()
            && !version.contains('/')
            && !file.is_empty()
            && !file.ends_with('/'))
        .then(|| FileRef {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            platform: platform.to_owned(),
            version: version.to_owned(),
            file: file.to_owned(),
        })
    }

    pub fn path(&self) -> String {
        file_path(
            &self.namespace,
            &self.name,
            &self.platform,
            &self.version,
            &self.file,
        )
    }

    pub fn is_vsix(&self) -> bool {
        self.file.eq_ignore_ascii_case(&vsix_name(
            &self.namespace,
            &self.name,
            &self.version,
            &self.platform,
        ))
    }

    pub fn is_sha256(&self) -> bool {
        self.file.eq_ignore_ascii_case(&sha256_name(
            &self.namespace,
            &self.name,
            &self.version,
            &self.platform,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_versions_and_platforms_are_checked() {
        for good in ["acme", "hello-astrum", "Ms-Python", "a1_b"] {
            assert!(valid_name(good), "{good}");
        }
        for bad in ["", "-x", "a.b", "a/b", "a b"] {
            assert!(!valid_name(bad), "{bad}");
        }
        assert!(valid_version("1.0.0"));
        assert!(valid_version("2.1.0-next.3"));
        assert!(!valid_version("1.0"));
        assert!(is_platform("darwin-arm64"));
        assert!(!is_platform("darwin"));
        assert_eq!(
            compare_versions("1.10.0", "1.9.0"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.0.0-beta", "1.0.0"),
            std::cmp::Ordering::Less
        );
        assert!(has_prerelease_tag("1.0.0-beta"));
        assert!(platform_rank("universal") < platform_rank("linux-x64"));
    }

    #[test]
    fn ids_split_and_derive_stable_uuids() {
        assert_eq!(split_id("acme.hello"), Some(("acme", "hello")));
        assert_eq!(split_id("acme/hello"), Some(("acme", "hello")));
        assert_eq!(split_id("acme"), None);
        assert_eq!(
            extension_uuid("acme", "hello"),
            extension_uuid("ACME", "Hello")
        );
        assert_ne!(extension_uuid("acme", "hello"), namespace_uuid("acme"));
        assert_eq!(extension_uuid("acme", "hello").len(), 36);
    }

    #[test]
    fn file_paths_follow_the_api() {
        assert_eq!(
            vsix_path("acme", "hello", "universal", "1.0.0"),
            "/api/acme/hello/1.0.0/file/acme.hello-1.0.0.vsix"
        );
        assert_eq!(
            vsix_path("acme", "hello", "linux-x64", "1.0.0"),
            "/api/acme/hello/linux-x64/1.0.0/file/acme.hello-1.0.0@linux-x64.vsix"
        );
        let universal = FileRef::parse("/api/acme/hello/1.0.0/file/readme.md").unwrap();
        assert_eq!(universal.platform, "universal");
        assert_eq!(universal.file, "readme.md");
        let explicit =
            FileRef::parse("/api/acme/hello/universal/1.0.0/file/acme.hello-1.0.0.vsix").unwrap();
        assert!(explicit.is_vsix());
        assert_eq!(
            explicit.path(),
            "/api/acme/hello/1.0.0/file/acme.hello-1.0.0.vsix"
        );
        let platform = FileRef::parse(
            "/api/acme/hello/darwin-arm64/1.0.0/file/acme.hello-1.0.0@darwin-arm64.sha256",
        )
        .unwrap();
        assert!(platform.is_sha256());
        assert!(FileRef::parse("/api/acme/hello/1.0.0/file/").is_none());
        assert!(FileRef::parse("/api/acme/hello/mars/1.0.0/file/x").is_none());
        assert_eq!(encode("a b#c"), "a%20b%23c");
        assert_eq!(encode_path("extension/a b.md"), "extension/a%20b.md");
    }
}
