use std::cmp::Ordering;

pub const REGISTRY_PATH: &str = "/bazel_registry.json";
pub const MODULE_FILE: &str = "MODULE.bazel";
pub const SOURCE_FILE: &str = "source.json";
pub const METADATA_FILE: &str = "metadata.json";
pub const PATCHES: &str = "patches";
pub const OVERLAY: &str = "overlay";

const MAX_MODULE: usize = 128;
const MAX_VERSION: usize = 128;
const MAX_SEGMENT: usize = 255;
const MAX_DEPTH: usize = 8;

pub fn valid_module(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_MODULE {
        return false;
    }
    let edge = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    bytes[0].is_ascii_lowercase()
        && edge(bytes[bytes.len() - 1])
        && bytes
            .iter()
            .all(|&byte| edge(byte) || matches!(byte, b'.' | b'-' | b'_'))
}

fn identifiers(text: &str, allow_hyphen: bool) -> bool {
    !text.is_empty()
        && text.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || (allow_hyphen && byte == b'-'))
        })
}

pub fn valid_version(version: &str) -> bool {
    if version.is_empty() || version.len() > MAX_VERSION {
        return false;
    }
    let (main, build) = match version.split_once('+') {
        Some((main, build)) => (main, Some(build)),
        None => (version, None),
    };
    let (release, prerelease) = match main.split_once('-') {
        Some((release, prerelease)) => (release, Some(prerelease)),
        None => (main, None),
    };
    identifiers(release, false)
        && prerelease.is_none_or(|text| identifiers(text, true))
        && build.is_none_or(|text| identifiers(text, true))
}

fn numeric(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn compare_identifier(left: &str, right: &str) -> Ordering {
    match (numeric(left), numeric(right)) {
        (true, true) => {
            let (left, right) = (left.trim_start_matches('0'), right.trim_start_matches('0'));
            left.len().cmp(&right.len()).then_with(|| left.cmp(right))
        }
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => left.cmp(right),
    }
}

fn compare_lists(left: &str, right: &str) -> Ordering {
    let mut left = left.split('.');
    let mut right = right.split('.');
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) => match compare_identifier(a, b) {
                Ordering::Equal => {}
                other => return other,
            },
        }
    }
}

fn split(version: &str) -> (&str, Option<&str>) {
    let main = version.split_once('+').map_or(version, |(main, _)| main);
    match main.split_once('-') {
        Some((release, prerelease)) => (release, Some(prerelease)),
        None => (main, None),
    }
}

pub fn compare(left: &str, right: &str) -> Ordering {
    let ((left_release, left_pre), (right_release, right_pre)) = (split(left), split(right));
    compare_lists(left_release, right_release)
        .then_with(|| match (left_pre, right_pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => compare_lists(a, b),
        })
        .then_with(|| left.cmp(right))
}

pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| compare(left, right));
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= MAX_SEGMENT
        && segment != "."
        && segment != ".."
        && segment.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'-' | b'+' | b'=' | b'@' | b',' | b'~')
        })
}

pub fn valid_relative(path: &str) -> bool {
    let mut depth = 0;
    for segment in path.split('/') {
        depth += 1;
        if depth > MAX_DEPTH || !valid_segment(segment) {
            return false;
        }
    }
    depth > 0
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Module,
    Source,
    Patch(String),
    Overlay(String),
}

impl Entry {
    pub fn parse(relative: &str) -> Option<Self> {
        match relative {
            MODULE_FILE => Some(Entry::Module),
            SOURCE_FILE => Some(Entry::Source),
            other => {
                let (directory, rest) = other.split_once('/')?;
                if !valid_relative(rest) {
                    return None;
                }
                match directory {
                    PATCHES => Some(Entry::Patch(rest.to_owned())),
                    OVERLAY => Some(Entry::Overlay(rest.to_owned())),
                    _ => None,
                }
            }
        }
    }

    pub fn relative(&self) -> String {
        match self {
            Entry::Module => MODULE_FILE.to_owned(),
            Entry::Source => SOURCE_FILE.to_owned(),
            Entry::Patch(name) => format!("{PATCHES}/{name}"),
            Entry::Overlay(name) => format!("{OVERLAY}/{name}"),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Entry::Module => "module",
            Entry::Source => "source",
            Entry::Patch(_) => "patch",
            Entry::Overlay(_) => "overlay",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Entry::Source => "application/json",
            Entry::Module | Entry::Patch(_) | Entry::Overlay(_) => "text/plain",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionFile {
    pub module: String,
    pub version: String,
    pub entry: Entry,
}

impl VersionFile {
    pub fn new(module: &str, version: &str, entry: Entry) -> Option<Self> {
        (valid_module(module) && valid_version(version)).then(|| Self {
            module: module.to_owned(),
            version: version.to_owned(),
            entry,
        })
    }

    pub fn path(&self) -> String {
        format!(
            "/modules/{}/{}/{}",
            self.module,
            self.version,
            self.entry.relative()
        )
    }
}

pub fn metadata_path(module: &str) -> String {
    format!("/modules/{module}/{METADATA_FILE}")
}

pub fn module_prefix(module: &str) -> String {
    format!("/modules/{module}/")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Root,
    Registry,
    Metadata(String),
    File(VersionFile),
    Other,
}

pub fn route(path: &str) -> Route {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    if trimmed == &REGISTRY_PATH[1..] {
        return Route::Registry;
    }
    let Some(rest) = trimmed.strip_prefix("modules/") else {
        return Route::Other;
    };
    let mut parts = rest.splitn(3, '/');
    let (Some(module), Some(second)) = (parts.next(), parts.next()) else {
        return Route::Other;
    };
    if !valid_module(module) {
        return Route::Other;
    }
    match parts.next() {
        None if second == METADATA_FILE => Route::Metadata(module.to_owned()),
        Some(relative) => Entry::parse(relative)
            .and_then(|entry| VersionFile::new(module, second, entry))
            .map(Route::File)
            .unwrap_or(Route::Other),
        None => Route::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(module: &str, version: &str, entry: Entry) -> Route {
        Route::File(VersionFile::new(module, version, entry).unwrap())
    }

    #[test]
    fn module_names_follow_the_bazel_rules() {
        for good in [
            "a",
            "rules_cc",
            "bazel_skylib",
            "zlib",
            "abseil-cpp",
            "a.b",
            "x1",
        ] {
            assert!(valid_module(good), "{good}");
        }
        for bad in [
            "", "Rules", "1abc", "-a", "a-", "a_", "a b", "a/b", "..", "a:b",
        ] {
            assert!(!valid_module(bad), "{bad}");
        }
        assert!(valid_module(&"a".repeat(128)));
        assert!(!valid_module(&"a".repeat(129)));
    }

    #[test]
    fn versions_are_relaxed_semver() {
        for good in [
            "1",
            "1.2.3",
            "1.2.3.bcr.1",
            "1.0.0-rc1",
            "1.0.0-rc.1+build.5",
            "1.19.1-20250305-abcdef",
            "20240116.2",
            "1.0.0+meta",
            "2.0.alpha",
        ] {
            assert!(valid_version(good), "{good}");
        }
        for bad in [
            "", ".1", "1.", "1..2", "1-", "1+", "-1", "1.0/2", "1 0", "1.0_1", "v/1",
        ] {
            assert!(!valid_version(bad), "{bad}");
        }
    }

    #[test]
    fn versions_sort_the_way_bazel_compares_them() {
        let mut versions: Vec<String> = [
            "1.10.0",
            "1.2.0",
            "1.2.0-rc1",
            "1.2.0.bcr.1",
            "1.2",
            "2.0.0-alpha",
            "2.0.0",
            "1.2.0-rc.10",
            "1.2.0-rc.2",
            "1.2.0+build",
        ]
        .map(str::to_owned)
        .to_vec();
        sort_versions(&mut versions);
        assert_eq!(
            versions,
            [
                "1.2",
                "1.2.0-rc.2",
                "1.2.0-rc.10",
                "1.2.0-rc1",
                "1.2.0",
                "1.2.0+build",
                "1.2.0.bcr.1",
                "1.10.0",
                "2.0.0-alpha",
                "2.0.0"
            ]
        );
    }

    #[test]
    fn numbers_are_compared_as_numbers_and_letters_after_them() {
        assert_eq!(compare("1.9", "1.10"), Ordering::Less);
        assert_eq!(compare("1.01", "1.1"), Ordering::Less);
        assert_eq!(compare("1.0.0", "1.0.a"), Ordering::Less);
        assert_eq!(compare("1.0.a", "1.0.b"), Ordering::Less);
        assert_eq!(compare("3.0.0", "3.0.0"), Ordering::Equal);
    }

    #[test]
    fn paths_route_to_the_files_of_a_registry() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/bazel_registry.json"), Route::Registry);
        assert_eq!(
            route("/modules/rules_cc/metadata.json"),
            Route::Metadata("rules_cc".to_owned())
        );
        assert_eq!(
            route("/modules/rules_cc/0.0.9/MODULE.bazel"),
            file("rules_cc", "0.0.9", Entry::Module)
        );
        assert_eq!(
            route("/modules/rules_cc/0.0.9/source.json"),
            file("rules_cc", "0.0.9", Entry::Source)
        );
        assert_eq!(
            route("/modules/rules_cc/0.0.9/patches/fix.patch"),
            file("rules_cc", "0.0.9", Entry::Patch("fix.patch".to_owned()))
        );
        assert_eq!(
            route("/modules/rules_cc/0.0.9/overlay/sub/BUILD.bazel"),
            file(
                "rules_cc",
                "0.0.9",
                Entry::Overlay("sub/BUILD.bazel".to_owned())
            )
        );
    }

    #[test]
    fn other_paths_are_not_part_of_a_registry() {
        for other in [
            "/modules",
            "/modules/",
            "/modules/rules_cc",
            "/modules/rules_cc/",
            "/modules/Rules_cc/metadata.json",
            "/modules/rules_cc/other.json",
            "/modules/rules_cc/0.0.9",
            "/modules/rules_cc/0.0.9/",
            "/modules/rules_cc/0.0.9/presubmit.yml",
            "/modules/rules_cc/0.0.9/patches",
            "/modules/rules_cc/0.0.9/patches/",
            "/modules/rules_cc/0.0.9/patches/a b",
            "/modules/rules_cc/0.0.9/patches/../x",
            "/modules/rules_cc/0.0.9/MODULE.bazel/x",
            "/modules/rules_cc/1..2/MODULE.bazel",
            "/modules/rules_cc/metadata.json/x",
            "/v1/cache/x",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }

    #[test]
    fn entries_round_trip_through_their_paths() {
        for relative in [
            "MODULE.bazel",
            "source.json",
            "patches/a.patch",
            "overlay/x/y.bzl",
        ] {
            assert_eq!(Entry::parse(relative).unwrap().relative(), relative);
        }
        assert_eq!(Entry::parse("patches"), None);
        assert_eq!(Entry::parse("overlay/"), None);
        assert_eq!(Entry::parse("patches/a//b"), None);
        let deep = format!("overlay/{}", ["d"; 9].join("/"));
        assert_eq!(Entry::parse(&deep), None);
        let file = VersionFile::new("zlib", "1.3.1", Entry::Patch("p.patch".to_owned())).unwrap();
        assert_eq!(file.path(), "/modules/zlib/1.3.1/patches/p.patch");
        assert_eq!(metadata_path("zlib"), "/modules/zlib/metadata.json");
    }
}
