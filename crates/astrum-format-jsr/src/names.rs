use semver::Version;

pub const META_FILE: &str = "meta.json";
pub const VERSION_META_SUFFIX: &str = "_meta.json";

const MIN_SCOPE: usize = 2;
const MAX_SCOPE: usize = 32;
const MAX_PACKAGE: usize = 58;
const MAX_VERSION: usize = 64;
const API_PREFIX: &str = "/api/";

fn valid_word(text: &str) -> bool {
    !text.starts_with('-')
        && text
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub fn valid_scope(scope: &str) -> bool {
    (MIN_SCOPE..=MAX_SCOPE).contains(&scope.len()) && valid_word(scope)
}

pub fn valid_package(name: &str) -> bool {
    (1..=MAX_PACKAGE).contains(&name.len()) && valid_word(name)
}

pub fn parse_version(text: &str) -> Option<Version> {
    if text.is_empty() || text.len() > MAX_VERSION {
        return None;
    }
    Version::parse(text).ok()
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Package {
    pub scope: String,
    pub name: String,
}

impl Package {
    pub fn new(scope: &str, name: &str) -> Option<Self> {
        (valid_scope(scope) && valid_package(name)).then(|| Self {
            scope: scope.to_owned(),
            name: name.to_owned(),
        })
    }

    pub fn namespace(&self) -> String {
        format!("@{}", self.scope)
    }

    pub fn display(&self) -> String {
        format!("@{}/{}", self.scope, self.name)
    }

    pub fn directory(&self) -> String {
        format!("/@{}/{}", self.scope, self.name)
    }

    pub fn listing_path(&self) -> String {
        format!("{}/{META_FILE}", self.directory())
    }

    pub fn version_meta_path(&self, version: &str) -> String {
        format!("{}/{version}{VERSION_META_SUFFIX}", self.directory())
    }

    pub fn file_path(&self, version: &str, file: &str) -> String {
        format!("{}/{version}/{file}", self.directory())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Listing,
    VersionMeta(String),
    File(String, String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub package: Package,
    pub document: Document,
}

impl Item {
    pub fn path(&self) -> String {
        match &self.document {
            Document::Listing => self.package.listing_path(),
            Document::VersionMeta(version) => self.package.version_meta_path(version),
            Document::File(version, file) => self.package.file_path(version, file),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Item(Item),
    Probe(String, String),
    Publish(String, String, String),
    Task,
    Api,
    Other,
}

pub fn route(path: &str) -> Route {
    if let Some(rest) = path.strip_prefix(API_PREFIX) {
        return api_route(rest);
    }
    let Some(rest) = path.strip_prefix("/@") else {
        return Route::Other;
    };
    let mut parts = rest.splitn(3, '/');
    let (Some(scope), Some(name), Some(tail)) = (parts.next(), parts.next(), parts.next()) else {
        return Route::Other;
    };
    let Some(package) = Package::new(scope, name) else {
        return Route::Other;
    };
    let document = if tail == META_FILE {
        Document::Listing
    } else if let Some((version, file)) = tail.split_once('/') {
        if parse_version(version).is_none() || file.is_empty() || file.ends_with('/') {
            return Route::Other;
        }
        Document::File(version.to_owned(), file.to_owned())
    } else if let Some(version) = tail.strip_suffix(VERSION_META_SUFFIX) {
        match parse_version(version) {
            Some(_) => Document::VersionMeta(version.to_owned()),
            None => return Route::Other,
        }
    } else {
        return Route::Other;
    };
    Route::Item(Item { package, document })
}

fn api_route(rest: &str) -> Route {
    let parts: Vec<&str> = rest.trim_end_matches('/').split('/').collect();
    match parts.as_slice() {
        ["scopes", scope, "packages", name] => {
            Route::Probe((*scope).to_owned(), (*name).to_owned())
        }
        ["scopes", scope, "packages", name, "versions", version] => Route::Publish(
            (*scope).to_owned(),
            (*name).to_owned(),
            (*version).to_owned(),
        ),
        ["publish_status", _] | ["publishing_tasks", _] => Route::Task,
        _ => Route::Api,
    }
}

pub fn content_type(file: &str) -> &'static str {
    let name = file.rsplit('/').next().unwrap_or(file).to_ascii_lowercase();
    let extension = name.rsplit_once('.').map(|(_, extension)| extension);
    match extension {
        Some("ts" | "mts") => "application/typescript",
        Some("js" | "mjs") => "application/javascript",
        Some("tsx") => "text/tsx",
        Some("jsx") => "text/jsx",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

pub fn sorted(versions: &mut [String]) {
    versions.sort_by(
        |left, right| match (parse_version(left), parse_version(right)) {
            (Some(left), Some(right)) => left.cmp(&right),
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => left.cmp(right),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_and_packages_follow_the_length_and_character_rules() {
        for good in ["ab", "a-b", "0a", &"a".repeat(32)] {
            assert!(valid_scope(good), "{good}");
        }
        for bad in ["a", "-ab", "Ab", "a_b", "a.b", "", &"a".repeat(33), "ä1"] {
            assert!(!valid_scope(bad), "{bad}");
        }
        for good in ["a", "hello-world", "9", &"a".repeat(58)] {
            assert!(valid_package(good), "{good}");
        }
        for bad in ["", "-a", "A", "a_b", "a/b", &"a".repeat(59)] {
            assert!(!valid_package(bad), "{bad}");
        }
    }

    #[test]
    fn versions_are_strict_semver_with_prerelease_and_build() {
        for good in ["0.1.0", "1.2.3-rc.1", "1.0.0+build.5", "1.0.0-alpha+001"] {
            assert!(parse_version(good).is_some(), "{good}");
        }
        for bad in [
            "1", "1.0", "v1.0.0", "01.0.0", "1.0.0-", "1.0.0 ", "", "1.0.0/x",
        ] {
            assert!(parse_version(bad).is_none(), "{bad}");
        }
        assert!(parse_version(&format!("1.0.0-{}", "a".repeat(80))).is_none());
    }

    #[test]
    fn read_paths_are_told_apart() {
        let package = Package::new("acme", "hello").unwrap();
        let item = |document| {
            Route::Item(Item {
                package: package.clone(),
                document,
            })
        };
        assert_eq!(route("/@acme/hello/meta.json"), item(Document::Listing));
        assert_eq!(
            route("/@acme/hello/0.1.0_meta.json"),
            item(Document::VersionMeta("0.1.0".to_owned()))
        );
        assert_eq!(
            route("/@acme/hello/0.1.0/sub/mod.ts"),
            item(Document::File("0.1.0".to_owned(), "sub/mod.ts".to_owned()))
        );
        assert_eq!(
            route("/@acme/hello/1.0.0/data_meta.json"),
            item(Document::File(
                "1.0.0".to_owned(),
                "data_meta.json".to_owned()
            ))
        );
        assert_eq!(
            route("/@acme/hello/1.0.0/sub/schema_meta.json"),
            item(Document::File(
                "1.0.0".to_owned(),
                "sub/schema_meta.json".to_owned()
            ))
        );
        assert_eq!(
            route("/@acme/hello/1.0.0/_meta.json"),
            item(Document::File("1.0.0".to_owned(), "_meta.json".to_owned()))
        );
        for other in [
            "/",
            "/@acme",
            "/@acme/hello",
            "/@acme/hello/",
            "/@acme/hello/other.json",
            "/@a/hello/meta.json",
            "/@Acme/hello/meta.json",
            "/acme/hello/meta.json",
            "/@acme/hello/1.0_meta.json",
            "/@acme/hello/latest/mod.ts",
            "/@acme/hello/0.1.0/",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }

    #[test]
    fn api_paths_are_told_apart() {
        assert_eq!(
            route("/api/scopes/acme/packages/hello"),
            Route::Probe("acme".to_owned(), "hello".to_owned())
        );
        assert_eq!(
            route("/api/scopes/acme/packages/hello/versions/0.1.0"),
            Route::Publish("acme".to_owned(), "hello".to_owned(), "0.1.0".to_owned())
        );
        assert_eq!(route("/api/publish_status/abc"), Route::Task);
        assert_eq!(route("/api/publishing_tasks/abc"), Route::Task);
        for other in ["/api/authorizations", "/api/scopes/acme", "/api/"] {
            assert_eq!(route(other), Route::Api, "{other}");
        }
    }

    #[test]
    fn files_get_a_type_by_extension_only() {
        assert_eq!(content_type("mod.ts"), "application/typescript");
        assert_eq!(content_type("types/a.d.ts"), "application/typescript");
        assert_eq!(content_type("a.mts"), "application/typescript");
        assert_eq!(content_type("a.js"), "application/javascript");
        assert_eq!(content_type("a.MJS"), "application/javascript");
        assert_eq!(content_type("a.tsx"), "text/tsx");
        assert_eq!(content_type("a.jsx"), "text/jsx");
        assert_eq!(content_type("deno.json"), "application/json");
        assert_eq!(content_type("README.md"), "application/octet-stream");
        assert_eq!(content_type("LICENSE"), "application/octet-stream");
    }

    #[test]
    fn versions_sort_by_precedence() {
        let mut versions = vec![
            "1.10.0".to_owned(),
            "1.2.0".to_owned(),
            "1.2.0-rc.1".to_owned(),
            "0.9.0".to_owned(),
        ];
        sorted(&mut versions);
        assert_eq!(versions, ["0.9.0", "1.2.0-rc.1", "1.2.0", "1.10.0"]);
    }
}
