pub const REPO_PATH: &str = "/repo";
pub const INDEX_PATH: &str = "/index.tar.gz";
pub const REPO_TEXT: &str = "opam-version: \"2.0\"\narchive-mirrors: \"cache\"\n";

pub const MAX_NAME: usize = 64;
pub const MAX_VERSION: usize = 64;
pub const MAX_FILE_SEGMENTS: usize = 8;
pub const MAX_SEGMENT: usize = 255;

const SHA256_HEX: usize = 64;
const SHA512_HEX: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }

    fn length(self) -> usize {
        match self {
            Self::Sha256 => SHA256_HEX,
            Self::Sha512 => SHA512_HEX,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheKey {
    pub algorithm: Algorithm,
    pub hex: String,
}

impl CacheKey {
    pub fn path(&self) -> String {
        format!(
            "/cache/{}/{}/{}",
            self.algorithm.as_str(),
            &self.hex[..2],
            self.hex
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Opam,
    File(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFile {
    pub name: String,
    pub version: String,
    pub entry: Entry,
}

impl PackageFile {
    pub fn path(&self) -> String {
        let directory = format!("/packages/{0}/{0}.{1}", self.name, self.version);
        match &self.entry {
            Entry::Opam => format!("{directory}/opam"),
            Entry::File(path) => format!("{directory}/files/{path}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Repo,
    Index,
    Package(PackageFile),
    Cache(CacheKey),
    Refused(String),
    Other,
}

fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    name.len() <= MAX_NAME
        && bytes.next().is_some_and(|b| b.is_ascii_alphanumeric())
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-'))
}

fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= MAX_VERSION
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'.' | b'~' | b'-'))
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= MAX_SEGMENT
        && segment != "."
        && segment != ".."
        && segment
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'+' | b'-'))
}

fn refused(message: impl Into<String>) -> Route {
    Route::Refused(message.into())
}

fn package(name: &str, directory: &str, rest: &[&str]) -> Route {
    if !valid_name(name) {
        return refused(format!(
            "a package name starts with a letter or digit, holds letters, digits, _, + and - and is at most {MAX_NAME} bytes long: {name}"
        ));
    }
    let Some((directory_name, version)) = directory.split_once('.') else {
        return refused(format!(
            "the directory of a package is named <name>.<version>: {directory}"
        ));
    };
    if directory_name != name {
        return refused(format!(
            "the directory {directory} must be named {name}.<version> for the package {name}"
        ));
    }
    if !valid_version(version) {
        return refused(format!(
            "a version holds letters, digits, _, +, ., ~ and - and is at most {MAX_VERSION} bytes long: {version}"
        ));
    }
    let entry = match rest {
        ["opam"] => Entry::Opam,
        ["files", segments @ ..]
            if !segments.is_empty()
                && segments.len() <= MAX_FILE_SEGMENTS
                && segments.iter().all(|segment| valid_segment(segment)) =>
        {
            Entry::File(segments.join("/"))
        }
        _ => {
            return refused(format!(
                "a package directory holds opam and files/<path> of at most {MAX_FILE_SEGMENTS} segments of letters, digits, ., _, + and -"
            ));
        }
    };
    Route::Package(PackageFile {
        name: name.to_owned(),
        version: version.to_owned(),
        entry,
    })
}

fn cache(algorithm: &str, prefix: &str, hex: &str) -> Route {
    let Some(algorithm) = Algorithm::parse(algorithm) else {
        return refused("the cache holds sha256 and sha512 digests only");
    };
    let lowercase_hex = hex
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if hex.len() != algorithm.length() || !lowercase_hex {
        return refused(format!(
            "a {} digest is {} lowercase hexadecimal characters",
            algorithm.as_str(),
            algorithm.length()
        ));
    }
    if prefix != &hex[..2] {
        return refused("the directory of a cache file is the first two characters of its digest");
    }
    Route::Cache(CacheKey {
        algorithm,
        hex: hex.to_owned(),
    })
}

pub fn parse(path: &str) -> Route {
    let Some(trimmed) = path.strip_prefix('/') else {
        return Route::Other;
    };
    let segments: Vec<&str> = trimmed.split('/').collect();
    match segments.as_slice() {
        ["repo"] => Route::Repo,
        ["index.tar.gz"] => Route::Index,
        ["packages", name, directory, rest @ ..] if !rest.is_empty() => {
            package(name, directory, rest)
        }
        ["packages", ..] => refused("a package file is packages/<name>/<name>.<version>/opam"),
        ["cache", algorithm, prefix, hex] if hex.len() >= 2 => cache(algorithm, prefix, hex),
        ["cache", ..] => refused("a cache file is cache/<sha256|sha512>/<xx>/<hex digest>"),
        _ => Route::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused_path(path: &str) -> bool {
        matches!(parse(path), Route::Refused(_))
    }

    #[test]
    fn packages_are_named_by_their_directory() {
        let route = parse("/packages/hello/hello.1.0~rc+1/opam");
        let Route::Package(file) = route else {
            panic!("not a package: {route:?}")
        };
        assert_eq!(
            (file.name.as_str(), file.version.as_str()),
            ("hello", "1.0~rc+1")
        );
        assert_eq!(file.path(), "/packages/hello/hello.1.0~rc+1/opam");
        assert!(refused_path("/packages/hello/other.1.0/opam"));
        assert!(refused_path("/packages/hello/hello/opam"));
        assert!(refused_path("/packages/hello/hello./opam"));
        assert!(refused_path("/packages/-x/-x.1/opam"));
        assert!(refused_path("/packages/a.b/a.b.1/opam"));
        assert!(refused_path(&format!(
            "/packages/{0}/{0}.1/opam",
            "a".repeat(MAX_NAME + 1)
        )));
        assert!(refused_path(&format!(
            "/packages/a/a.{}/opam",
            "1".repeat(MAX_VERSION + 1)
        )));
        assert!(refused_path("/packages/a/a.1"));
        assert!(refused_path("/packages/a/a.1/notes"));
    }

    #[test]
    fn files_are_bounded_by_segments_and_characters() {
        assert!(matches!(
            parse("/packages/a/a.1/files/patches/fix.patch"),
            Route::Package(PackageFile {
                entry: Entry::File(_),
                ..
            })
        ));
        let deep = ["d"; MAX_FILE_SEGMENTS].join("/");
        assert!(matches!(
            parse(&format!("/packages/a/a.1/files/{deep}")),
            Route::Package(_)
        ));
        assert!(refused_path(&format!("/packages/a/a.1/files/{deep}/x")));
        assert!(refused_path("/packages/a/a.1/files"));
        assert!(refused_path("/packages/a/a.1/files/a b"));
        assert!(refused_path("/packages/a/a.1/files/..%2f"));
        assert!(refused_path("/packages/a/a.1/files/../x"));
        assert!(refused_path("/packages/a/a.1/files/a//b"));
    }

    #[test]
    fn cache_files_sit_under_the_start_of_their_digest() {
        let hex = "ab".repeat(32);
        let route = parse(&format!("/cache/sha256/ab/{hex}"));
        let Route::Cache(key) = route else {
            panic!("not a cache file: {route:?}")
        };
        assert_eq!(key.path(), format!("/cache/sha256/ab/{hex}"));
        assert!(refused_path(&format!("/cache/sha256/cd/{hex}")));
        assert!(refused_path(&format!(
            "/cache/sha256/ab/{}",
            "ab".repeat(33)
        )));
        assert!(refused_path(&format!(
            "/cache/sha256/AB/{}",
            "AB".repeat(32)
        )));
        assert!(refused_path(&format!("/cache/md5/ab/{hex}")));
        assert!(matches!(
            parse(&format!("/cache/sha512/ab/{}", "ab".repeat(64))),
            Route::Cache(_)
        ));
    }

    #[test]
    fn the_generated_files_and_strangers_are_told_apart() {
        assert_eq!(parse("/repo"), Route::Repo);
        assert_eq!(parse("/index.tar.gz"), Route::Index);
        assert_eq!(parse("/"), Route::Other);
        assert_eq!(parse("/version"), Route::Other);
        assert_eq!(parse("/repo/"), Route::Other);
    }
}
