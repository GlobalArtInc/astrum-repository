use std::fmt;

pub const DISCOVERY: &str = "/.well-known/terraform.json";
pub const TOKEN: &str = "/token";
pub const MODULES: &str = "/v1/modules/";
pub const PROVIDERS: &str = "/v1/providers/";
pub const SHA256SUMS: &str = "SHA256SUMS";
pub const KEY_FILE: &str = "signing-key.asc";

const ARCHIVE_EXTENSIONS: &[&str] = &["tar.gz", "tgz", "zip", "tar.bz2", "tbz2", "tar.xz", "txz"];

pub fn valid_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    let edge = |c: &u8| c.is_ascii_alphanumeric();
    !bytes.is_empty()
        && bytes.len() <= 128
        && bytes.first().is_some_and(edge)
        && bytes.last().is_some_and(edge)
        && bytes
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}

fn identifiers(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
}

pub fn valid_version(value: &str) -> bool {
    let (rest, build) = match value.split_once('+') {
        Some((rest, build)) => (rest, Some(build)),
        None => (value, None),
    };
    if build.is_some_and(|build| !identifiers(build)) {
        return false;
    }
    let (core, pre) = match rest.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (rest, None),
    };
    if pre.is_some_and(|pre| !identifiers(pre)) {
        return false;
    }
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|c| c.is_ascii_digit()))
}

pub fn valid_platform_part(value: &str) -> bool {
    (1..=32).contains(&value.len())
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

pub fn valid_hostname(value: &str) -> bool {
    let (host, port) = match value.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (value, None),
    };
    if port.is_some_and(|port| {
        port.is_empty() || port.len() > 5 || !port.bytes().all(|c| c.is_ascii_digit())
    }) {
        return false;
    }
    let bytes = host.as_bytes();
    let edge = |c: &u8| c.is_ascii_alphanumeric();
    !bytes.is_empty()
        && bytes.len() <= 253
        && bytes.first().is_some_and(edge)
        && bytes.last().is_some_and(edge)
        && bytes
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'.' || *c == b'-')
}

pub fn archive_extension(file: &str) -> Option<&'static str> {
    let lower = file.to_ascii_lowercase();
    ARCHIVE_EXTENSIONS
        .iter()
        .find(|extension| {
            lower.len() > extension.len() + 1 && lower.ends_with(&format!(".{extension}"))
        })
        .map(|extension| canonical(extension))
}

pub fn archive_parameter(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    ARCHIVE_EXTENSIONS
        .iter()
        .find(|extension| **extension == lower)
        .map(|extension| canonical(extension))
}

fn canonical(extension: &'static str) -> &'static str {
    match extension {
        "tgz" => "tar.gz",
        "tbz2" => "tar.bz2",
        "txz" => "tar.xz",
        other => other,
    }
}

pub fn sniff(head: &[u8]) -> Option<&'static str> {
    match head {
        [b'P', b'K', 3, 4, ..] => Some("zip"),
        [0x1f, 0x8b, ..] => Some("tar.gz"),
        [b'B', b'Z', b'h', ..] => Some("tar.bz2"),
        [0xfd, b'7', b'z', b'X', b'Z', 0, ..] => Some("tar.xz"),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Module {
    pub namespace: String,
    pub name: String,
    pub system: String,
    pub version: String,
}

impl Module {
    pub fn new(namespace: &str, name: &str, system: &str, version: &str) -> Option<Self> {
        (valid_name(namespace)
            && valid_name(name)
            && valid_name(system)
            && (version.is_empty() || valid_version(version)))
        .then(|| Self {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            system: system.to_owned(),
            version: version.to_owned(),
        })
    }

    pub fn address(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.name, self.system)
    }

    pub fn component_name(&self) -> String {
        format!("{}/{}", self.name, self.system)
    }

    pub fn base(&self) -> String {
        format!("{MODULES}{}", self.address())
    }

    pub fn versions_path(&self) -> String {
        format!("{}/versions", self.base())
    }

    pub fn directory(&self) -> String {
        format!("{}/{}", self.base(), self.version)
    }

    pub fn download_path(&self) -> String {
        format!("{}/download", self.directory())
    }

    pub fn archive_file(&self, extension: &str) -> String {
        format!("{}-{}-{}.{extension}", self.name, self.system, self.version)
    }

    pub fn archive_path(&self, extension: &str) -> String {
        format!("{}/{}", self.directory(), self.archive_file(extension))
    }

    pub fn at(&self, version: &str) -> Self {
        Self {
            version: version.to_owned(),
            ..self.clone()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Platform<'a> {
    pub os: &'a str,
    pub arch: &'a str,
}

impl fmt::Display for Platform<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.os, self.arch)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Provider {
    pub namespace: String,
    pub kind: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileKind {
    Archive { os: String, arch: String },
    Shasums,
    Signature,
    Manifest,
    Key,
}

impl FileKind {
    pub fn asset_kind(&self) -> &'static str {
        match self {
            FileKind::Archive { .. } => "PROVIDER_ARCHIVE",
            FileKind::Shasums => "PROVIDER_SHASUMS",
            FileKind::Signature => "PROVIDER_SIGNATURE",
            FileKind::Manifest => "PROVIDER_MANIFEST",
            FileKind::Key => "PROVIDER_KEY",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            FileKind::Archive { .. } => "application/zip",
            FileKind::Shasums => "text/plain",
            FileKind::Signature => "application/pgp-signature",
            FileKind::Manifest => "application/json",
            FileKind::Key => "application/pgp-keys",
        }
    }
}

impl Provider {
    pub fn new(namespace: &str, kind: &str, version: &str) -> Option<Self> {
        (valid_name(namespace)
            && valid_name(kind)
            && (version.is_empty() || valid_version(version)))
        .then(|| Self {
            namespace: namespace.to_owned(),
            kind: kind.to_owned(),
            version: version.to_owned(),
        })
    }

    pub fn address(&self) -> String {
        format!("{}/{}", self.namespace, self.kind)
    }

    pub fn base(&self) -> String {
        format!("{PROVIDERS}{}", self.address())
    }

    pub fn versions_path(&self) -> String {
        format!("{}/versions", self.base())
    }

    pub fn directory(&self) -> String {
        format!("{}/{}", self.base(), self.version)
    }

    pub fn download_path(&self, os: &str, arch: &str) -> String {
        format!("{}/download/{os}/{arch}", self.directory())
    }

    pub fn prefix(&self) -> String {
        format!("terraform-provider-{}_{}_", self.kind, self.version)
    }

    pub fn archive_file(&self, os: &str, arch: &str) -> String {
        format!("{}{os}_{arch}.zip", self.prefix())
    }

    pub fn shasums_file(&self) -> String {
        format!("{}{SHA256SUMS}", self.prefix())
    }

    pub fn signature_file(&self) -> String {
        format!("{}.sig", self.shasums_file())
    }

    pub fn manifest_file(&self) -> String {
        format!("{}manifest.json", self.prefix())
    }

    pub fn file_path(&self, file: &str) -> String {
        format!("{}/{file}", self.directory())
    }

    pub fn at(&self, version: &str) -> Self {
        Self {
            version: version.to_owned(),
            ..self.clone()
        }
    }

    pub fn platform_of(&self, file: &str) -> Option<(String, String)> {
        let platform = file.strip_prefix(&self.prefix())?.strip_suffix(".zip")?;
        let (os, arch) = platform.split_once('_')?;
        (!arch.contains('_') && valid_platform_part(os) && valid_platform_part(arch))
            .then(|| (os.to_owned(), arch.to_owned()))
    }

    pub fn kind_of(&self, file: &str) -> Option<FileKind> {
        if file == KEY_FILE {
            return Some(FileKind::Key);
        }
        let rest = file.strip_prefix(&self.prefix())?;
        if let Some((os, arch)) = self.platform_of(file) {
            return Some(FileKind::Archive { os, arch });
        }
        if rest == SHA256SUMS {
            return Some(FileKind::Shasums);
        }
        if rest.starts_with("SHA256SUMS.") && rest.ends_with(".sig") && !rest.contains('/') {
            return Some(FileKind::Signature);
        }
        (rest == "manifest.json").then_some(FileKind::Manifest)
    }

    pub fn stored_name(&self, kind: &FileKind, file: &str) -> String {
        match kind {
            FileKind::Shasums => self.shasums_file(),
            FileKind::Signature => self.signature_file(),
            FileKind::Manifest => self.manifest_file(),
            FileKind::Key => KEY_FILE.to_owned(),
            FileKind::Archive { .. } => file.to_owned(),
        }
    }
}

pub fn is_key_upload(file: &str) -> bool {
    let lower = file.to_ascii_lowercase();
    file == KEY_FILE
        || [".asc", ".gpg", ".pub", ".key"]
            .iter()
            .any(|extension| lower.ends_with(extension))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Endpoint {
    Discovery,
    Token,
    ModuleVersions(Module),
    ModuleDownload(Module),
    ModuleArchive(Module, String),
    ProviderVersions(Provider),
    ProviderDownload(Provider, String, String),
    ProviderFile(Provider, String, FileKind),
    MirrorIndex(String, Provider),
    MirrorVersion(String, Provider),
    Other,
}

impl Endpoint {
    pub fn parse(path: &str) -> Endpoint {
        match path {
            DISCOVERY => return Endpoint::Discovery,
            TOKEN => return Endpoint::Token,
            _ => {}
        }
        let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let parsed = match segments.as_slice() {
            ["v1", "modules", ns, name, system, "versions"] => {
                Module::new(ns, name, system, "").map(Endpoint::ModuleVersions)
            }
            ["v1", "modules", ns, name, system, version, "download"] if !version.is_empty() => {
                Module::new(ns, name, system, version).map(Endpoint::ModuleDownload)
            }
            ["v1", "modules", ns, name, system, version, file]
                if !version.is_empty() && archive_extension(file).is_some() =>
            {
                Module::new(ns, name, system, version)
                    .map(|module| Endpoint::ModuleArchive(module, (*file).to_owned()))
            }
            ["v1", "providers", ns, kind, "versions"] => {
                Provider::new(ns, kind, "").map(Endpoint::ProviderVersions)
            }
            ["v1", "providers", ns, kind, version, "download", os, arch]
                if !version.is_empty() && valid_platform_part(os) && valid_platform_part(arch) =>
            {
                Provider::new(ns, kind, version).map(|provider| {
                    Endpoint::ProviderDownload(provider, (*os).to_owned(), (*arch).to_owned())
                })
            }
            ["v1", "providers", ns, kind, version, file] if !version.is_empty() => {
                Provider::new(ns, kind, version).and_then(|provider| {
                    let kind = provider.kind_of(file)?;
                    Some(Endpoint::ProviderFile(provider, (*file).to_owned(), kind))
                })
            }
            _ => None,
        };
        if let Some(endpoint) = parsed {
            return endpoint;
        }
        match segments.as_slice() {
            [host, ns, kind, "index.json"] if valid_hostname(host) => Provider::new(ns, kind, "")
                .map(|provider| Endpoint::MirrorIndex(host.to_ascii_lowercase(), provider)),
            [host, ns, kind, file] if valid_hostname(host) => file
                .strip_suffix(".json")
                .filter(|version| !version.is_empty() && *version != "index")
                .and_then(|version| Provider::new(ns, kind, version))
                .map(|provider| Endpoint::MirrorVersion(host.to_ascii_lowercase(), provider)),
            _ => None,
        }
        .unwrap_or(Endpoint::Other)
    }
}

pub fn relative(from: &str, to: &str) -> String {
    let from: Vec<&str> = from.trim_start_matches('/').split('/').collect();
    let to: Vec<&str> = to.trim_start_matches('/').split('/').collect();
    let mut common = 0;
    while common + 1 < from.len() && common + 1 < to.len() && from[common] == to[common] {
        common += 1;
    }
    let mut relative = "../".repeat(from.len() - 1 - common);
    if relative.is_empty() {
        relative.push_str("./");
    }
    relative.push_str(&to[common..].join("/"));
    relative
}

pub fn resolve(document: &str, relative: &str) -> Option<String> {
    let base = url::Url::parse(&format!("http://repository{document}")).ok()?;
    let path = base.join(relative).ok()?.path().to_owned();
    (!path.contains("..")).then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_versions_and_hosts_are_checked() {
        assert!(valid_name("terraform-aws_modules"));
        assert!(!valid_name("-x") && !valid_name("x-") && !valid_name("a.b") && !valid_name(""));
        assert!(valid_version("1.2.3") && valid_version("1.2.3-rc.1+build.5"));
        assert!(!valid_version("1.2") && !valid_version("v1.2.3") && !valid_version("1.2.3-"));
        assert!(valid_hostname("registry.terraform.io") && valid_hostname("localhost:8443"));
        assert!(!valid_hostname("a_b.example") && !valid_hostname("x:") && !valid_hostname("v1:x"));
    }

    #[test]
    fn paths_route_to_the_protocol_endpoints() {
        let module = Module::new("acme", "vpc", "aws", "1.2.0").unwrap();
        assert_eq!(
            Endpoint::parse("/v1/modules/acme/vpc/aws/versions"),
            Endpoint::ModuleVersions(module.at(""))
        );
        assert_eq!(
            Endpoint::parse("/v1/modules/acme/vpc/aws/1.2.0/download"),
            Endpoint::ModuleDownload(module.clone())
        );
        assert_eq!(
            Endpoint::parse("/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz"),
            Endpoint::ModuleArchive(module, "vpc-aws-1.2.0.tar.gz".to_owned())
        );
        let provider = Provider::new("acme", "demo", "2.0.0").unwrap();
        assert_eq!(
            Endpoint::parse("/v1/providers/acme/demo/2.0.0/download/linux/amd64"),
            Endpoint::ProviderDownload(provider.clone(), "linux".to_owned(), "amd64".to_owned())
        );
        assert_eq!(
            Endpoint::parse(
                "/v1/providers/acme/demo/2.0.0/terraform-provider-demo_2.0.0_SHA256SUMS.72D7468F.sig"
            ),
            Endpoint::ProviderFile(
                provider.clone(),
                "terraform-provider-demo_2.0.0_SHA256SUMS.72D7468F.sig".to_owned(),
                FileKind::Signature
            )
        );
        assert_eq!(
            Endpoint::parse("/registry.terraform.io/acme/demo/index.json"),
            Endpoint::MirrorIndex("registry.terraform.io".to_owned(), provider.at(""))
        );
        assert_eq!(
            Endpoint::parse("/Registry.Terraform.io/acme/demo/2.0.0.json"),
            Endpoint::MirrorVersion("registry.terraform.io".to_owned(), provider)
        );
        for other in [
            "/v1/modules/acme/vpc/aws/1.2.0/readme.md",
            "/v1/providers/acme/demo/2.0.0/other.zip",
            "/v1/providers/acme/demo/versions/x",
            "/x/y",
        ] {
            assert_eq!(Endpoint::parse(other), Endpoint::Other, "{other}");
        }
    }

    #[test]
    fn provider_files_are_known_by_their_names() {
        let provider = Provider::new("acme", "demo", "2.0.0").unwrap();
        assert_eq!(
            provider.kind_of("terraform-provider-demo_2.0.0_linux_amd64.zip"),
            Some(FileKind::Archive {
                os: "linux".to_owned(),
                arch: "amd64".to_owned()
            })
        );
        assert_eq!(
            provider.kind_of("terraform-provider-demo_2.0.0_SHA256SUMS"),
            Some(FileKind::Shasums)
        );
        assert_eq!(
            provider.kind_of("terraform-provider-demo_2.0.0_manifest.json"),
            Some(FileKind::Manifest)
        );
        assert_eq!(provider.kind_of("signing-key.asc"), Some(FileKind::Key));
        assert_eq!(
            provider.kind_of("terraform-provider-demo_2.0.0_linux_amd_64.zip"),
            None
        );
        assert!(is_key_upload("release.GPG") && !is_key_upload("notes.txt"));
    }

    #[test]
    fn archives_are_known_by_extension_and_magic() {
        assert_eq!(archive_extension("vpc.TGZ"), Some("tar.gz"));
        assert_eq!(archive_extension("vpc.zip"), Some("zip"));
        assert_eq!(archive_extension(".zip"), None);
        assert_eq!(archive_parameter("tbz2"), Some("tar.bz2"));
        assert_eq!(sniff(b"PK\x03\x04rest"), Some("zip"));
        assert_eq!(sniff(&[0x1f, 0x8b, 8]), Some("tar.gz"));
        assert_eq!(sniff(b"plain"), None);
    }

    #[test]
    fn relative_urls_lead_from_one_document_to_another() {
        assert_eq!(relative(DISCOVERY, MODULES), "../v1/modules/");
        assert_eq!(
            relative(
                "/v1/modules/a/b/c/1.0.0/download",
                "/v1/modules/a/b/c/1.0.0/b-c-1.0.0.tar.gz"
            ),
            "./b-c-1.0.0.tar.gz"
        );
        assert_eq!(
            relative(
                "/v1/providers/a/t/1.0.0/download/linux/amd64",
                "/v1/providers/a/t/1.0.0/f.zip"
            ),
            "../../f.zip"
        );
        assert_eq!(
            relative(
                "/registry.terraform.io/a/t/1.0.0.json",
                "/v1/providers/a/t/1.0.0/f.zip"
            ),
            "../../../v1/providers/a/t/1.0.0/f.zip"
        );
        assert_eq!(
            resolve(
                "/v1/providers/a/t/1.0.0/download/linux/amd64",
                "../../f.zip"
            )
            .as_deref(),
            Some("/v1/providers/a/t/1.0.0/f.zip")
        );
    }
}
