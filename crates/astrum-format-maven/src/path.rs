use serde_json::{Map, Value, json};

use crate::version::{self, SNAPSHOT};

pub const METADATA: &str = "maven-metadata.xml";
pub const ARCHETYPE_CATALOG: &str = "/archetype-catalog.xml";
const SIGNATURE: &str = ".asc";
const COMPOUND_EXTENSIONS: [&str; 4] = ["tar.gz", "tar.bz2", "tar.xz", "tar.zst"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hash {
    Sha1,
    Md5,
    Sha256,
    Sha512,
}

impl Hash {
    pub const ALL: [Hash; 4] = [Hash::Sha1, Hash::Md5, Hash::Sha256, Hash::Sha512];

    pub fn suffix(self) -> &'static str {
        match self {
            Hash::Sha1 => ".sha1",
            Hash::Md5 => ".md5",
            Hash::Sha256 => ".sha256",
            Hash::Sha512 => ".sha512",
        }
    }

    pub fn hex_length(self) -> usize {
        match self {
            Hash::Sha1 => 40,
            Hash::Md5 => 32,
            Hash::Sha256 => 64,
            Hash::Sha512 => 128,
        }
    }

    fn of(name: &str) -> Option<Hash> {
        Hash::ALL
            .into_iter()
            .find(|hash| name.len() > hash.suffix().len() && name.ends_with(hash.suffix()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coordinates {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub base_version: String,
    pub classifier: Option<String>,
    pub extension: String,
}

impl Coordinates {
    pub fn is_snapshot(&self) -> bool {
        version::is_snapshot(&self.version)
    }

    pub fn path(&self) -> String {
        let classifier = self
            .classifier
            .as_deref()
            .map(|classifier| format!("-{classifier}"))
            .unwrap_or_default();
        format!(
            "/{}/{}/{}/{}-{}{classifier}.{}",
            self.group_id.replace('.', "/"),
            self.artifact_id,
            self.base_version,
            self.artifact_id,
            self.version,
            self.extension
        )
    }

    pub fn attributes(&self) -> Value {
        let mut maven = Map::new();
        maven.insert("groupId".to_owned(), json!(self.group_id));
        maven.insert("artifactId".to_owned(), json!(self.artifact_id));
        maven.insert("version".to_owned(), json!(self.version));
        maven.insert("baseVersion".to_owned(), json!(self.base_version));
        if let Some(classifier) = &self.classifier {
            maven.insert("classifier".to_owned(), json!(classifier));
        }
        maven.insert("extension".to_owned(), json!(self.extension));
        json!({ crate::ID: maven })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Artifact(Coordinates),
    Signature(Coordinates),
    Metadata,
    ArchetypeCatalog,
    Index,
    Other,
}

impl Kind {
    pub fn coordinates(&self) -> Option<&Coordinates> {
        match self {
            Kind::Artifact(coordinates) | Kind::Signature(coordinates) => Some(coordinates),
            _ => None,
        }
    }

    pub fn asset_kind(&self) -> &'static str {
        match self {
            Kind::Artifact(_) => "ARTIFACT",
            Kind::Signature(_) => "ARTIFACT_SUBORDINATE",
            Kind::Metadata | Kind::ArchetypeCatalog => "REPOSITORY_METADATA",
            Kind::Index => "REPOSITORY_INDEX",
            Kind::Other => "OTHER",
        }
    }

    pub fn follows_layout(&self) -> bool {
        matches!(
            self,
            Kind::Artifact(_) | Kind::Signature(_) | Kind::Metadata
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MavenPath {
    pub path: String,
    pub hash: Option<Hash>,
    pub base: String,
    pub kind: Kind,
}

impl MavenPath {
    pub fn directory(&self) -> &str {
        match self.base.rfind('/') {
            Some(index) => &self.base[..index],
            None => "",
        }
    }

    pub fn in_snapshot_directory(&self) -> bool {
        self.directory()
            .rsplit('/')
            .next()
            .is_some_and(|name| name.ends_with(SNAPSHOT))
    }
}

pub fn parse(path: &str) -> MavenPath {
    let name = path.rsplit('/').next().unwrap_or_default();
    let hash = Hash::of(name);
    let base = match hash {
        Some(hash) => &path[..path.len() - hash.suffix().len()],
        None => path,
    };
    MavenPath {
        path: path.to_owned(),
        hash,
        base: base.to_owned(),
        kind: classify(base),
    }
}

fn classify(path: &str) -> Kind {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let name = segments.last().copied().unwrap_or_default();
    if path == ARCHETYPE_CATALOG {
        return Kind::ArchetypeCatalog;
    }
    if matches!(segments.first(), Some(&".index" | &".meta")) && segments.len() > 1 {
        return Kind::Index;
    }
    if name == METADATA {
        return Kind::Metadata;
    }
    if Hash::of(name).is_some() {
        return Kind::Other;
    }
    if let Some(signed) = path.strip_suffix(SIGNATURE)
        && let Some(mut coordinates) = artifact(signed)
    {
        coordinates.extension.push_str(SIGNATURE);
        return Kind::Signature(coordinates);
    }
    match artifact(path) {
        Some(coordinates) => Kind::Artifact(coordinates),
        None => Kind::Other,
    }
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty() && !segment.starts_with('.') && !segment.chars().any(char::is_whitespace)
}

fn artifact(path: &str) -> Option<Coordinates> {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if segments.len() < 4 || !segments.iter().all(|segment| valid_segment(segment)) {
        return None;
    }
    let count = segments.len();
    let name = segments[count - 1];
    let base_version = segments[count - 2];
    let artifact_id = segments[count - 3];
    let group_id = segments[..count - 3].join(".");
    let rest = name.strip_prefix(artifact_id)?.strip_prefix('-')?;
    let (file_version, tail) = split_version(rest, base_version)?;
    let (classifier, extension) = split_tail(tail)?;
    Some(Coordinates {
        group_id,
        artifact_id: artifact_id.to_owned(),
        version: file_version.to_owned(),
        base_version: base_version.to_owned(),
        classifier,
        extension,
    })
}

fn split_version<'a>(rest: &'a str, base_version: &str) -> Option<(&'a str, &'a str)> {
    let separated = |tail: &str| tail.starts_with('.') || tail.starts_with('-');
    if let Some(tail) = rest.strip_prefix(base_version)
        && separated(tail)
    {
        return Some((&rest[..base_version.len()], tail));
    }
    let prefix = base_version.strip_suffix(SNAPSHOT)?;
    let stamped = rest.strip_prefix(prefix)?;
    let timestamp = stamped.get(..15)?;
    if !version::is_timestamp(timestamp) {
        return None;
    }
    let build = stamped[15..].strip_prefix('-')?;
    let digits = build.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let end = prefix.len() + 15 + 1 + digits;
    let tail = &rest[end..];
    separated(tail).then(|| (&rest[..end], tail))
}

fn extension_of(text: &str) -> &str {
    for compound in COMPOUND_EXTENSIONS {
        if text == compound
            || text
                .strip_suffix(compound)
                .is_some_and(|before| before.ends_with('.'))
        {
            return &text[text.len() - compound.len()..];
        }
    }
    match text.rfind('.') {
        Some(index) => &text[index + 1..],
        None => text,
    }
}

fn split_tail(tail: &str) -> Option<(Option<String>, String)> {
    if let Some(extensions) = tail.strip_prefix('.') {
        let extension = extension_of(extensions);
        return (!extension.is_empty()).then(|| (None, extension.to_owned()));
    }
    let named = tail.strip_prefix('-')?;
    if !named.contains('.') {
        return None;
    }
    let extension = extension_of(named);
    let classifier = named[..named.len() - extension.len()].strip_suffix('.')?;
    if classifier.is_empty() || extension.is_empty() {
        return None;
    }
    Some((Some(classifier.to_owned()), extension.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coordinates(path: &str) -> Coordinates {
        match parse(path).kind {
            Kind::Artifact(coordinates) | Kind::Signature(coordinates) => coordinates,
            other => panic!("{path} is {other:?}"),
        }
    }

    #[test]
    fn release_artifacts_are_parsed() {
        let jar = coordinates("/org/example/lib/1.0/lib-1.0.jar");
        assert_eq!(
            jar,
            Coordinates {
                group_id: "org.example".to_owned(),
                artifact_id: "lib".to_owned(),
                version: "1.0".to_owned(),
                base_version: "1.0".to_owned(),
                classifier: None,
                extension: "jar".to_owned(),
            }
        );
        assert_eq!(jar.path(), "/org/example/lib/1.0/lib-1.0.jar");
        let sources = coordinates("/org/example/lib/1.0/lib-1.0-sources.jar");
        assert_eq!(sources.classifier.as_deref(), Some("sources"));
        assert_eq!(sources.path(), "/org/example/lib/1.0/lib-1.0-sources.jar");
        let tarball = coordinates("/org/x/lib/1.0/lib-1.0-bin.tar.gz");
        assert_eq!(
            (tarball.classifier.as_deref(), tarball.extension.as_str()),
            (Some("bin"), "tar.gz")
        );
        assert_eq!(
            coordinates("/org/x/lib/1.0/lib-1.0.tar.gz").extension,
            "tar.gz"
        );
        assert_eq!(coordinates("/org/x/lib/1.0/lib-1.0.a.b.c").extension, "c");
        let dashed = coordinates("/org/x/lib/1.0/lib-1.0-x-y.zip");
        assert_eq!(dashed.classifier.as_deref(), Some("x-y"));
        let dotted = coordinates("/org/x/lib/1.0/lib-1.0-jdk1.8.jar");
        assert_eq!(
            (dotted.classifier.as_deref(), dotted.extension.as_str()),
            (Some("jdk1.8"), "jar")
        );
        assert_eq!(
            coordinates("/a/b/c/d/lib/2.0-RC1/lib-2.0-RC1.pom").group_id,
            "a.b.c.d"
        );
    }

    #[test]
    fn snapshots_keep_both_versions() {
        let stamped = coordinates("/org/x/lib/1.0-SNAPSHOT/lib-1.0-20261012.101010-3-tests.jar");
        assert_eq!(stamped.version, "1.0-20261012.101010-3");
        assert_eq!(stamped.base_version, "1.0-SNAPSHOT");
        assert_eq!(stamped.classifier.as_deref(), Some("tests"));
        assert!(stamped.is_snapshot());
        assert_eq!(
            stamped.path(),
            "/org/x/lib/1.0-SNAPSHOT/lib-1.0-20261012.101010-3-tests.jar"
        );
        let plain = coordinates("/org/x/lib/1.0-SNAPSHOT/lib-1.0-SNAPSHOT.pom");
        assert_eq!(plain.version, "1.0-SNAPSHOT");
        assert_eq!(plain.classifier, None);
        let sources = coordinates("/org/x/lib/1.0-SNAPSHOT/lib-1.0-SNAPSHOT-sources.jar");
        assert_eq!(sources.classifier.as_deref(), Some("sources"));
        assert_eq!(
            parse("/org/x/lib/1.0-SNAPSHOT/lib-1.1-20261012.101010-1.jar").kind,
            Kind::Other
        );
    }

    #[test]
    fn checksums_and_signatures_point_at_their_base() {
        let checksum = parse("/org/x/lib/1.0/lib-1.0.jar.sha1");
        assert_eq!(checksum.hash, Some(Hash::Sha1));
        assert_eq!(checksum.base, "/org/x/lib/1.0/lib-1.0.jar");
        assert_eq!(checksum.kind.coordinates().unwrap().extension, "jar");
        let signature = parse("/org/x/lib/1.0/lib-1.0.pom.asc");
        assert_eq!(signature.hash, None);
        assert!(matches!(&signature.kind, Kind::Signature(found) if found.extension == "pom.asc"));
        let signed_checksum = parse("/org/x/lib/1.0/lib-1.0-bin.tar.gz.asc.sha512");
        assert_eq!(signed_checksum.hash, Some(Hash::Sha512));
        assert!(
            matches!(&signed_checksum.kind, Kind::Signature(found) if found.extension == "tar.gz.asc")
        );
        let metadata = parse("/org/x/lib/maven-metadata.xml.md5");
        assert_eq!(
            (metadata.hash, metadata.kind),
            (Some(Hash::Md5), Kind::Metadata)
        );
        assert_eq!(
            parse("/org/x/lib/1.0/lib-1.0.jar.sha1.md5").kind,
            Kind::Other
        );
    }

    #[test]
    fn other_paths_are_classified() {
        assert_eq!(parse("/archetype-catalog.xml").kind, Kind::ArchetypeCatalog);
        assert_eq!(parse("/.index/repository-index.gz").kind, Kind::Index);
        assert_eq!(parse("/.meta/prefixes.txt").kind, Kind::Index);
        for other in [
            "/foo.txt",
            "/lib/1.0/lib-1.0.jar",
            "/org/x/lib/1.0/lib-1.0",
            "/org/x/lib/1.0/lib-1.0-sources",
            "/org/x/lib/1.0/other-1.0.jar",
            "/org/x/lib/1.0/lib-2.0.jar",
            "/org/x/lib/1.0/lib-1.0.",
            "/org/x/lib/1.0/lib-1.0-.jar",
            "/org/x/lib/1.0/lib-1.0x.jar",
            "/org/.x/lib/1.0/lib-1.0.jar",
            "/org/x/lib/1.0-SNAPSHOT/lib-1.0-20261012.101010-.jar",
        ] {
            assert_eq!(parse(other).kind, Kind::Other, "{other}");
        }
    }

    #[test]
    fn metadata_directories_tell_snapshots() {
        assert!(parse("/org/x/lib/1.0-SNAPSHOT/maven-metadata.xml").in_snapshot_directory());
        assert!(!parse("/org/x/lib/maven-metadata.xml").in_snapshot_directory());
        assert_eq!(
            parse("/org/x/lib/maven-metadata.xml").directory(),
            "/org/x/lib"
        );
    }

    #[test]
    fn attributes_name_the_coordinates() {
        let attributes = coordinates("/org/x/lib/1.0/lib-1.0-sources.jar").attributes();
        assert_eq!(
            attributes,
            json!({"maven2": {"groupId": "org.x", "artifactId": "lib", "version": "1.0",
                "baseVersion": "1.0", "classifier": "sources", "extension": "jar"}})
        );
    }
}
