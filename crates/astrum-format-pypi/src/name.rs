const ARCHIVES: &[&str] = &[
    ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst", ".tar.z", ".tgz", ".tbz", ".tbz2", ".txz",
    ".tar", ".zip", ".whl", ".egg", ".exe", ".msi", ".rpm", ".dmg", ".deb",
];

pub fn normalize(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut separator = false;
    for c in name.chars() {
        if matches!(c, '-' | '_' | '.') {
            if !separator {
                normalized.push('-');
            }
            separator = true;
            continue;
        }
        separator = false;
        normalized.extend(c.to_lowercase());
    }
    normalized
}

pub fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 200
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

pub fn is_valid_version(version: &str) -> bool {
    is_safe_segment(version)
        && version.len() <= 128
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+' | '!'))
}

pub fn is_safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment.len() <= 255
        && !segment
            .chars()
            .any(|c| c == '/' || c == '\\' || c.is_control())
}

fn split_archive(filename: &str) -> (&str, &str) {
    let lower = filename.to_ascii_lowercase();
    for extension in ARCHIVES {
        if lower.ends_with(extension) && lower.len() > extension.len() {
            let at = filename.len() - extension.len();
            return (&filename[..at], &filename[at..]);
        }
    }
    (filename, "")
}

pub fn is_wheel(filename: &str) -> bool {
    filename.to_ascii_lowercase().ends_with(".whl")
}

pub fn version_of(project: &str, filename: &str) -> Option<String> {
    let (stem, extension) = split_archive(filename);
    let extension = extension.to_ascii_lowercase();
    if extension == ".whl" || extension == ".egg" {
        return stem
            .split('-')
            .nth(1)
            .filter(|version| !version.is_empty())
            .map(str::to_owned);
    }
    let project = normalize(project);
    stem.char_indices()
        .filter(|(_, c)| *c == '-')
        .find(|(at, _)| normalize(&stem[..*at]) == project)
        .map(|(at, _)| &stem[at + 1..])
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
}

pub fn content_type(filename: &str) -> String {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".whl") || lower.ends_with(".egg") || lower.ends_with(".zip") {
        "application/zip".to_owned()
    } else if lower.ends_with(".tar.bz2") {
        "application/x-bzip2".to_owned()
    } else if lower.ends_with(".metadata") {
        "text/plain".to_owned()
    } else {
        astrum_content::mime::from_path(filename)
            .unwrap_or_else(|| astrum_content::mime::OCTET_STREAM.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_normalize_like_pep_503() {
        assert_eq!(normalize("Foo_Bar"), "foo-bar");
        assert_eq!(normalize("Astrum-Probe.Pkg"), "astrum-probe-pkg");
        assert_eq!(normalize("a__-.b"), "a-b");
        assert_eq!(normalize("Django"), "django");
        assert_eq!(normalize("zope.interface"), "zope-interface");
        assert_eq!(normalize("_leading"), "-leading");
        assert_eq!(normalize("trailing-"), "trailing-");
    }

    #[test]
    fn names_follow_the_core_metadata_rules() {
        for good in ["six", "Foo_Bar", "a.b-c", "x", "A1"] {
            assert!(is_valid_name(good), "{good}");
        }
        for bad in ["", "-a", "a-", "a/b", "a b", "ä", "a..b/../"] {
            assert!(!is_valid_name(bad), "{bad}");
        }
        assert!(is_valid_version("1.0.post1+local.2"));
        assert!(is_valid_version("1!2.0"));
        assert!(!is_valid_version("../1"));
        assert!(!is_valid_version("1 0"));
        assert!(!is_valid_version(""));
    }

    #[test]
    fn versions_come_from_file_names() {
        assert_eq!(
            version_of("six", "six-1.17.0-py2.py3-none-any.whl").as_deref(),
            Some("1.17.0")
        );
        assert_eq!(
            version_of("astrum-probe-pkg", "astrum_probe_pkg-0.1.0.tar.gz").as_deref(),
            Some("0.1.0")
        );
        assert_eq!(
            version_of("foo-bar", "Foo.Bar-2.0rc1.zip").as_deref(),
            Some("2.0rc1")
        );
        assert_eq!(
            version_of("python-dateutil", "python-dateutil-2.9.0.post0.tar.gz").as_deref(),
            Some("2.9.0.post0")
        );
        assert_eq!(
            version_of("setuptools", "setuptools-0.6c11-py2.7.egg").as_deref(),
            Some("0.6c11")
        );
        assert_eq!(version_of("foo-bar", "other-9.9.tar.gz"), None);
    }

    #[test]
    fn segments_must_stay_inside_their_directory() {
        assert!(is_safe_segment("six-1.0.tar.gz"));
        for bad in ["", ".", "..", "a/b", "a\\b", "a\nb"] {
            assert!(!is_safe_segment(bad), "{bad:?}");
        }
    }

    #[test]
    fn content_types_follow_the_archive() {
        assert_eq!(content_type("a-1-py3-none-any.whl"), "application/zip");
        assert_eq!(content_type("a-1.tar.gz"), "application/x-gzip");
        assert_eq!(content_type("a-1.zip"), "application/zip");
        assert_eq!(content_type("a-1-py3-none-any.whl.metadata"), "text/plain");
    }
}
