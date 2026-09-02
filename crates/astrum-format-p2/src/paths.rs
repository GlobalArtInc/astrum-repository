use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};

pub const REMOTE: &str = ".remote";

const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}')
    .add(b'\'');

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repository {
    Content,
    Artifacts,
    CompositeContent,
    CompositeArtifacts,
}

impl Repository {
    pub fn is_composite(self) -> bool {
        matches!(self, Self::CompositeContent | Self::CompositeArtifacts)
    }

    pub fn is_rewritten(self) -> bool {
        !matches!(self, Self::Content)
    }

    fn stem(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Artifacts => "artifacts",
            Self::CompositeContent => "compositeContent",
            Self::CompositeArtifacts => "compositeArtifacts",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Jar,
    Xml,
    Xz,
}

impl Encoding {
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Jar => "application/java-archive",
            Self::Xml => "application/xml",
            Self::Xz => "application/x-xz",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folder {
    Features,
    Plugins,
    Binary,
}

impl Folder {
    pub fn name(self) -> &'static str {
        match self {
            Self::Features => "features",
            Self::Plugins => "plugins",
            Self::Binary => "binary",
        }
    }

    pub fn classifier(self) -> &'static str {
        match self {
            Self::Features => "org.eclipse.update.feature",
            Self::Plugins => "osgi.bundle",
            Self::Binary => "binary",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        match name {
            "features" => Some(Self::Features),
            "plugins" => Some(Self::Plugins),
            "binary" => Some(Self::Binary),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub repository: Repository,
    pub encoding: Encoding,
    pub location: String,
}

impl Metadata {
    pub fn entry_name(&self) -> String {
        format!("{}.xml", self.repository.stem())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub folder: Folder,
    pub file: String,
    pub location: String,
}

impl Artifact {
    pub fn stem(&self) -> &str {
        let file = self.file.as_str();
        match self.folder {
            Folder::Binary => file,
            _ => file
                .strip_suffix(".jar.pack.gz")
                .or_else(|| file.strip_suffix(".jar"))
                .unwrap_or(file),
        }
    }

    pub fn is_jar(&self) -> bool {
        self.folder != Folder::Binary && self.file.ends_with(".jar")
    }

    pub fn content_type(&self) -> &'static str {
        if self.file.ends_with(".jar") {
            "application/java-archive"
        } else if self.file.ends_with(".gz") {
            "application/gzip"
        } else {
            "application/octet-stream"
        }
    }

    pub fn artifacts_paths(&self) -> [String; 3] {
        [
            format!("{}/artifacts.xml.xz", self.location),
            format!("{}/artifacts.jar", self.location),
            format!("{}/artifacts.xml", self.location),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Directory,
    Index,
    Metadata(Metadata),
    Artifact(Artifact),
    Unknown,
}

fn metadata_file(file: &str) -> Option<(Repository, Encoding)> {
    let repository = [
        Repository::Content,
        Repository::Artifacts,
        Repository::CompositeContent,
        Repository::CompositeArtifacts,
    ]
    .into_iter()
    .find(|repository| file.starts_with(repository.stem()))?;
    let encoding = match &file[repository.stem().len()..] {
        ".jar" => Encoding::Jar,
        ".xml" => Encoding::Xml,
        ".xml.xz" if !repository.is_composite() => Encoding::Xz,
        _ => return None,
    };
    Some((repository, encoding))
}

pub fn route(path: &str) -> Route {
    if path.is_empty() || path.ends_with('/') {
        return Route::Directory;
    }
    let (parent, file) = path.rsplit_once('/').unwrap_or(("", path));
    if file == "p2.index" {
        return Route::Index;
    }
    if let Some((repository, encoding)) = metadata_file(file) {
        return Route::Metadata(Metadata {
            repository,
            encoding,
            location: parent.to_owned(),
        });
    }
    let (location, folder) = parent.rsplit_once('/').unwrap_or(("", parent));
    match Folder::parse(folder) {
        Some(folder) if !file.is_empty() => Route::Artifact(Artifact {
            folder,
            file: file.to_owned(),
            location: location.to_owned(),
        }),
        _ => Route::Unknown,
    }
}

pub fn segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

pub fn remote_target(path: &str) -> Option<url::Url> {
    let parts = segments(path);
    let [REMOTE, scheme, authority, rest @ ..] = parts.as_slice() else {
        return None;
    };
    if !matches!(*scheme, "http" | "https") || authority.is_empty() {
        return None;
    }
    let mut url = url::Url::parse(&format!("{scheme}://{authority}/")).ok()?;
    if url.host_str().is_none_or(str::is_empty)
        || url.path() != "/"
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    url.path_segments_mut().ok()?.pop().extend(rest.iter());
    if path.ends_with('/') {
        url.path_segments_mut().ok()?.push("");
    }
    Some(url)
}

fn decoded_segments(url: &url::Url) -> Vec<String> {
    url.path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty())
        .map(|segment| percent_decode_str(segment).decode_utf8_lossy().into_owned())
        .collect()
}

fn authority(url: &url::Url) -> String {
    let host = url.host_str().unwrap_or_default();
    match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    }
}

pub fn without_target(url: &url::Url) -> url::Url {
    let mut plain = url.clone();
    plain.set_query(None);
    plain.set_fragment(None);
    let trimmed = plain.path().trim_end_matches('/').to_owned();
    plain.set_path(if trimmed.is_empty() { "/" } else { &trimmed });
    plain
}

pub fn location_of(url: &url::Url, base: &url::Url) -> Vec<String> {
    let found = decoded_segments(url);
    let root = decoded_segments(base);
    if url.origin() == base.origin() && found.starts_with(&root) {
        return found[root.len()..].to_vec();
    }
    let mut mapped = vec![REMOTE.to_owned(), url.scheme().to_owned(), authority(url)];
    mapped.extend(found);
    mapped
}

pub fn relative(from: &[&str], to: &[String]) -> String {
    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(a, b)| **a == b.as_str())
        .count();
    let mut parts: Vec<String> = vec!["..".to_owned(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|segment| utf8_percent_encode(segment, SEGMENT).to_string()),
    );
    match parts.is_empty() {
        true => ".".to_owned(),
        false => parts.join("/"),
    }
}

pub fn under(url: &str, location: &str) -> bool {
    let location = location.trim_end_matches('/');
    url == location
        || url
            .strip_prefix(location)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn is_version(text: &str) -> bool {
    let mut parts = text.splitn(4, '.');
    let numeric = |part: Option<&str>| {
        part.is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    };
    if !numeric(parts.next()) {
        return false;
    }
    for _ in 0..2 {
        match parts.next() {
            None => return true,
            part if numeric(part) => {}
            _ => return false,
        }
    }
    match parts.next() {
        None => true,
        Some(qualifier) => {
            !qualifier.is_empty()
                && qualifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        }
    }
}

pub fn split_name(stem: &str) -> Option<(String, String)> {
    stem.match_indices('_').find_map(|(at, _)| {
        let (id, version) = (&stem[..at], &stem[at + 1..]);
        (!id.is_empty() && is_version(version)).then(|| (id.to_owned(), version.to_owned()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(path: &str) -> Option<(Repository, Encoding, String)> {
        match route(path) {
            Route::Metadata(found) => Some((found.repository, found.encoding, found.location)),
            _ => None,
        }
    }

    #[test]
    fn metadata_files_are_known_at_every_location() {
        assert_eq!(route("/p2.index"), Route::Index);
        assert_eq!(route("/child/p2.index"), Route::Index);
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/child/"), Route::Directory);
        assert_eq!(
            metadata("/content.xml.xz"),
            Some((Repository::Content, Encoding::Xz, String::new()))
        );
        assert_eq!(
            metadata("/a/b/artifacts.jar"),
            Some((Repository::Artifacts, Encoding::Jar, "/a/b".to_owned()))
        );
        assert_eq!(
            metadata("/compositeContent.xml"),
            Some((Repository::CompositeContent, Encoding::Xml, String::new()))
        );
        assert_eq!(
            metadata("/x/compositeArtifacts.jar"),
            Some((
                Repository::CompositeArtifacts,
                Encoding::Jar,
                "/x".to_owned()
            ))
        );
        assert_eq!(metadata("/compositeContent.xml.xz"), None);
        assert_eq!(metadata("/contents.jar"), None);
        assert_eq!(route("/site.xml"), Route::Unknown);
        assert_eq!(route("/index.html"), Route::Unknown);
    }

    #[test]
    fn artifacts_live_in_their_folders() {
        let Route::Artifact(plugin) = route("/2024/plugins/org.example.core_1.2.0.v2024.jar")
        else {
            panic!("not an artifact");
        };
        assert_eq!(plugin.folder, Folder::Plugins);
        assert_eq!(plugin.location, "/2024");
        assert_eq!(plugin.stem(), "org.example.core_1.2.0.v2024");
        assert!(plugin.is_jar());
        assert_eq!(
            plugin.artifacts_paths()[1],
            "/2024/artifacts.jar".to_owned()
        );
        let Route::Artifact(packed) = route("/plugins/a_1.0.0.jar.pack.gz") else {
            panic!("not an artifact");
        };
        assert_eq!(packed.stem(), "a_1.0.0");
        assert!(!packed.is_jar());
        assert_eq!(packed.content_type(), "application/gzip");
        let Route::Artifact(binary) = route("/binary/org.example.launcher_root_1.0.0") else {
            panic!("not an artifact");
        };
        assert_eq!(binary.folder, Folder::Binary);
        assert_eq!(binary.stem(), "org.example.launcher_root_1.0.0");
        assert_eq!(route("/features/"), Route::Directory);
        assert_eq!(route("/other/a_1.0.0.jar"), Route::Unknown);
    }

    #[test]
    fn names_split_at_the_first_version() {
        assert_eq!(
            split_name("org.eclipse.swt.gtk.linux.x86_64_3.125.0.v20240227-1638"),
            Some((
                "org.eclipse.swt.gtk.linux.x86_64".to_owned(),
                "3.125.0.v20240227-1638".to_owned()
            ))
        );
        assert_eq!(
            split_name(
                "org.eclipse.equinox.executable_root.gtk.linux.x86_64_3.8.2300.v20240213-1244"
            ),
            Some((
                "org.eclipse.equinox.executable_root.gtk.linux.x86_64".to_owned(),
                "3.8.2300.v20240213-1244".to_owned()
            ))
        );
        assert_eq!(
            split_name("javax.xml_1.3.4.v201005080400"),
            Some(("javax.xml".to_owned(), "1.3.4.v201005080400".to_owned()))
        );
        assert_eq!(
            split_name("a_1.0.0.v_b-2"),
            Some(("a".to_owned(), "1.0.0.v_b-2".to_owned()))
        );
        assert_eq!(split_name("a_1.0.0_b"), None);
        assert_eq!(
            split_name("foo_2_1.0.0"),
            Some(("foo_2".to_owned(), "1.0.0".to_owned()))
        );
        assert_eq!(split_name("plain"), None);
        assert_eq!(split_name("_1.0.0"), None);
        assert_eq!(split_name("a_1.0.0.q.x"), None);
        assert_eq!(split_name("a_x1.0"), None);
    }

    #[test]
    fn remote_locations_map_into_the_proxy_and_back() {
        let base = url::Url::parse("https://download.example.org/releases/2024-03/").unwrap();
        let inside = base.join("202403131000").unwrap();
        assert_eq!(location_of(&inside, &base), vec!["202403131000"]);
        let outside = base.join("../../technology/epp/packages/2024-03/").unwrap();
        assert_eq!(
            location_of(&without_target(&outside), &base),
            vec![
                ".remote",
                "https",
                "download.example.org",
                "technology",
                "epp",
                "packages",
                "2024-03"
            ]
        );
        let other = url::Url::parse("http://mirror.example.net:8080/p2/a%20b").unwrap();
        let mapped = location_of(&other, &base);
        assert_eq!(
            mapped,
            vec![".remote", "http", "mirror.example.net:8080", "p2", "a b"]
        );
        let path = format!("/{}/content.jar", mapped.join("/"));
        assert_eq!(
            remote_target(&path).unwrap().as_str(),
            "http://mirror.example.net:8080/p2/a%20b/content.jar"
        );
        assert_eq!(
            remote_target("/.remote/https/h.example/x/")
                .unwrap()
                .as_str(),
            "https://h.example/x/"
        );
        assert!(remote_target("/.remote/ftp/h.example/x").is_none());
        assert!(remote_target("/.remote/https").is_none());
        assert!(remote_target("/.remote/https/user@h.example/x").is_none());
        assert!(remote_target("/plugins/a.jar").is_none());
        let elsewhere =
            url::Url::parse("https://download.example.org/releases/2024-030/x").unwrap();
        assert_eq!(location_of(&elsewhere, &base)[0], ".remote");
    }

    #[test]
    fn child_locations_are_written_relative_to_the_composite() {
        let to = |parts: &[&str]| {
            parts
                .iter()
                .map(|part| part.to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(relative(&[], &to(&["202403131000"])), "202403131000");
        assert_eq!(
            relative(&["a", "b"], &to(&[".remote", "https", "h", "x y"])),
            "../../.remote/https/h/x%20y"
        );
        assert_eq!(relative(&["a"], &to(&["a", "c"])), "c");
        assert_eq!(relative(&["a"], &to(&["a"])), ".");
    }

    #[test]
    fn locations_contain_their_files() {
        assert!(under("https://h/x/content.jar", "https://h/x"));
        assert!(under("https://h/x/content.jar", "https://h/x/"));
        assert!(under("https://h/x", "https://h/x"));
        assert!(!under("https://h/xy/content.jar", "https://h/x"));
    }
}
