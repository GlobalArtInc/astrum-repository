use std::fmt;

use astrum_blob::Digests;
use sha2::{Digest as _, Sha256, Sha512};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    pub fn name(self) -> &'static str {
        match self {
            Algorithm::Sha256 => "sha256",
            Algorithm::Sha512 => "sha512",
        }
    }

    fn hex_length(self) -> usize {
        match self {
            Algorithm::Sha256 => 64,
            Algorithm::Sha512 => 128,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Digest {
    algorithm: Algorithm,
    hex: String,
}

impl Digest {
    pub fn parse(value: &str) -> Option<Self> {
        let (algorithm, hex) = value.split_once(':')?;
        let algorithm = match algorithm {
            "sha256" => Algorithm::Sha256,
            "sha512" => Algorithm::Sha512,
            _ => return None,
        };
        let valid = hex.len() == algorithm.hex_length()
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        valid.then(|| Digest {
            algorithm,
            hex: hex.to_owned(),
        })
    }

    pub fn of(algorithm: Algorithm, bytes: &[u8]) -> Self {
        let hex = match algorithm {
            Algorithm::Sha256 => hex::encode(Sha256::digest(bytes)),
            Algorithm::Sha512 => hex::encode(Sha512::digest(bytes)),
        };
        Digest { algorithm, hex }
    }

    pub fn sha256(bytes: &[u8]) -> Self {
        Self::of(Algorithm::Sha256, bytes)
    }

    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub fn hex(&self) -> &str {
        &self.hex
    }

    pub fn matches_bytes(&self, bytes: &[u8]) -> bool {
        Self::of(self.algorithm, bytes) == *self
    }

    pub fn matches(&self, digests: &Digests) -> bool {
        let computed = match self.algorithm {
            Algorithm::Sha256 => &digests.sha256,
            Algorithm::Sha512 => &digests.sha512,
        };
        *computed == self.hex
    }

    pub fn referrers_tag(&self) -> String {
        format!("{}-{}", self.algorithm.name(), &self.hex[..64])
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.name(), self.hex)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    Tag(String),
    Digest(Digest),
}

impl Reference {
    pub fn parse(value: &str) -> Option<Self> {
        if value.contains(':') {
            return Digest::parse(value).map(Reference::Digest);
        }
        valid_tag(value).then(|| Reference::Tag(value.to_owned()))
    }

    pub fn as_str(&self) -> String {
        match self {
            Reference::Tag(tag) => tag.clone(),
            Reference::Digest(digest) => digest.to_string(),
        }
    }
}

pub fn valid_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    let allowed = |byte: &u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-');
    match bytes.split_first() {
        Some((first, rest)) => {
            (first.is_ascii_alphanumeric() || *first == b'_')
                && rest.len() <= 127
                && rest.iter().all(allowed)
        }
        None => false,
    }
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 255 && name.split('/').all(valid_component)
}

fn valid_component(component: &str) -> bool {
    let bytes = component.as_bytes();
    let alphanumeric = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    match (bytes.first(), bytes.last()) {
        (Some(first), Some(last)) if alphanumeric(*first) && alphanumeric(*last) => {}
        _ => return false,
    }
    let mut index = 0;
    while index < bytes.len() {
        if alphanumeric(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && !alphanumeric(bytes[index]) {
            index += 1;
        }
        let separator = &component[start..index];
        let allowed = separator == "."
            || separator == "_"
            || separator == "__"
            || separator.bytes().all(|byte| byte == b'-');
        if !allowed {
            return false;
        }
    }
    true
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Base,
    Catalog,
    Tags(String),
    Manifest(String, String),
    Blob(String, String),
    Uploads(String),
    Upload(String, String),
    Referrers(String, String),
}

pub fn route(path: &str) -> Option<Route> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Some(Route::Base);
    }
    if trimmed == "_catalog" {
        return Some(Route::Catalog);
    }
    let segments: Vec<&str> = trimmed.split('/').collect();
    let count = segments.len();
    let name = |end: usize| segments[..end].join("/");
    let last = segments[count - 1];
    if count >= 3 && segments[count - 2] == "tags" && last == "list" {
        return Some(Route::Tags(name(count - 2)));
    }
    if count >= 3 && segments[count - 2] == "blobs" && last == "uploads" {
        return Some(Route::Uploads(name(count - 2)));
    }
    if count >= 4 && segments[count - 3] == "blobs" && segments[count - 2] == "uploads" {
        return Some(Route::Upload(name(count - 3), last.to_owned()));
    }
    if count >= 2 {
        let reference = last.to_owned();
        match segments[count - 2] {
            "blobs" if count >= 3 => return Some(Route::Blob(name(count - 2), reference)),
            "manifests" if count >= 3 => {
                return Some(Route::Manifest(name(count - 2), reference));
            }
            "referrers" if count >= 3 => {
                return Some(Route::Referrers(name(count - 2), reference));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA256_EMPTY: &str =
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn digests_parse_and_verify() {
        let digest = Digest::parse(SHA256_EMPTY).unwrap();
        assert_eq!(digest.algorithm(), Algorithm::Sha256);
        assert!(digest.matches_bytes(b""));
        assert!(!digest.matches_bytes(b"x"));
        assert_eq!(digest.to_string(), SHA256_EMPTY);
        assert_eq!(Digest::sha256(b""), digest);
        let long = Digest::of(Algorithm::Sha512, b"");
        assert_eq!(long.hex().len(), 128);
        assert_eq!(Digest::parse(&long.to_string()), Some(long.clone()));
        assert!(long.matches_bytes(b""));
        assert_eq!(
            long.referrers_tag(),
            format!("sha512-{}", &long.hex()[..64])
        );
        assert_eq!(digest.referrers_tag(), SHA256_EMPTY.replace(':', "-"));
    }

    #[test]
    fn odd_digests_are_refused() {
        for bad in [
            "sha256:baddigeststring",
            "sha256:E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            "md5:d41d8cd98f00b204e9800998ecf8427e",
            "sha256",
            "",
            "sha512:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ] {
            assert_eq!(Digest::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn digests_compare_with_store_checksums() {
        let digests = Digests {
            sha1: String::new(),
            sha256: SHA256_EMPTY[7..].to_owned(),
            sha512: Digest::of(Algorithm::Sha512, b"").hex().to_owned(),
            md5: String::new(),
        };
        assert!(Digest::parse(SHA256_EMPTY).unwrap().matches(&digests));
        assert!(Digest::of(Algorithm::Sha512, b"").matches(&digests));
        assert!(!Digest::sha256(b"x").matches(&digests));
    }

    #[test]
    fn names_follow_the_distribution_grammar() {
        for good in [
            "app", "team/app", "a/b/c/d", "my-app", "my--app", "my_app", "my__app", "my.app",
            "0x/1",
        ] {
            assert!(valid_name(good), "{good}");
        }
        for bad in [
            "",
            "App",
            "team//app",
            "/app",
            "app/",
            "-app",
            "app-",
            "my___app",
            "my.-app",
            "a b",
            "my..app",
        ] {
            assert!(!valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn tags_follow_the_distribution_grammar() {
        assert!(valid_tag("latest"));
        assert!(valid_tag("_x.1-A"));
        assert!(valid_tag(&"a".repeat(128)));
        assert!(!valid_tag(&"a".repeat(129)));
        assert!(!valid_tag(".x"));
        assert!(!valid_tag("-x"));
        assert!(!valid_tag("a:b"));
        assert!(!valid_tag(""));
        assert_eq!(
            Reference::parse("1.0"),
            Some(Reference::Tag("1.0".to_owned()))
        );
        assert!(matches!(
            Reference::parse(SHA256_EMPTY),
            Some(Reference::Digest(_))
        ));
        assert_eq!(Reference::parse("sha256:short"), None);
    }

    #[test]
    fn routes_are_read_from_the_end() {
        assert_eq!(route(""), Some(Route::Base));
        assert_eq!(route("/"), Some(Route::Base));
        assert_eq!(route("/_catalog"), Some(Route::Catalog));
        assert_eq!(
            route("/team/app/tags/list"),
            Some(Route::Tags("team/app".to_owned()))
        );
        assert_eq!(
            route("/team/app/manifests/1.0"),
            Some(Route::Manifest("team/app".to_owned(), "1.0".to_owned()))
        );
        assert_eq!(
            route("/app/blobs/uploads/"),
            Some(Route::Uploads("app".to_owned()))
        );
        assert_eq!(
            route("/app/blobs/uploads/abc"),
            Some(Route::Upload("app".to_owned(), "abc".to_owned()))
        );
        assert_eq!(
            route("/x/blobs/blobs/uploads/abc"),
            Some(Route::Upload("x/blobs".to_owned(), "abc".to_owned()))
        );
        assert_eq!(
            route("/x/blobs/uploads/blobs/sha256:1"),
            Some(Route::Blob(
                "x/blobs/uploads".to_owned(),
                "sha256:1".to_owned()
            ))
        );
        assert_eq!(
            route("/a/manifests/tags/list"),
            Some(Route::Tags("a/manifests".to_owned()))
        );
        assert_eq!(
            route("/a/referrers/sha256:1"),
            Some(Route::Referrers("a".to_owned(), "sha256:1".to_owned()))
        );
        assert_eq!(route("/a/b"), None);
        assert_eq!(route("/manifests/1.0"), None);
    }
}
