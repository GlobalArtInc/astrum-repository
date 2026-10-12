pub const OCTET_STREAM: &str = "application/octet-stream";
pub const TEXT: &str = "text/plain";
pub const SAMPLE: usize = 8192;

pub fn from_path(path: &str) -> Option<String> {
    let name = path.rsplit('/').next().unwrap_or(path);
    if name.ends_with(".tgz") || name.ends_with(".tar.gz") {
        return Some("application/x-gzip".to_owned());
    }
    let guessed = mime_guess::from_path(name).first()?;
    Some(guessed.essence_str().to_owned())
}

pub fn detect(sample: &[u8]) -> Option<String> {
    if sample.is_empty() {
        return None;
    }
    if let Some(kind) = infer::get(sample) {
        return Some(kind.mime_type().to_owned());
    }
    let text = !sample.contains(&0) && is_utf8_prefix(sample);
    Some(if text { TEXT } else { OCTET_STREAM }.to_owned())
}

fn is_utf8_prefix(sample: &[u8]) -> bool {
    match std::str::from_utf8(sample) {
        Ok(_) => true,
        Err(error) => error.error_len().is_none() && sample.len() >= SAMPLE - 4,
    }
}

fn family(mime: &str) -> &str {
    match mime {
        "application/zip"
        | "application/java-archive"
        | "application/x-java-archive"
        | "application/vnd.android.package-archive"
        | "application/x-zip-compressed" => "zip",
        "application/gzip"
        | "application/x-gzip"
        | "application/x-compressed"
        | "application/x-tar" => "gzip",
        _ if is_textual(mime) => "text",
        _ => mime,
    }
}

pub fn is_textual(mime: &str) -> bool {
    mime.starts_with("text/")
        || mime.ends_with("+xml")
        || mime.ends_with("+json")
        || matches!(
            mime,
            "application/json"
                | "application/xml"
                | "application/javascript"
                | "application/x-javascript"
                | "application/x-sh"
                | "application/x-yaml"
                | "application/yaml"
                | "application/toml"
                | "application/x-ruby"
                | "application/x-python"
                | "application/sql"
                | "application/pgp-signature"
                | "application/pgp-keys"
        )
}

pub fn compatible(detected: Option<&str>, expected: Option<&str>) -> bool {
    let (Some(detected), Some(expected)) = (detected, expected) else {
        return true;
    };
    if expected == OCTET_STREAM || detected == OCTET_STREAM || detected == expected {
        return true;
    }
    let (detected, expected) = (family(detected), family(expected));
    if detected == "zip" && expected != "text" {
        return true;
    }
    detected == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_come_from_the_name() {
        assert_eq!(from_path("/a/b/hello.txt").as_deref(), Some("text/plain"));
        assert_eq!(from_path("/x.json").as_deref(), Some("application/json"));
        assert_eq!(
            from_path("/lib.jar").as_deref(),
            Some("application/java-archive")
        );
        assert_eq!(from_path("/pkg.tgz").as_deref(), Some("application/x-gzip"));
        assert_eq!(from_path("/README").as_deref(), None);
    }

    #[test]
    fn content_is_sniffed() {
        assert_eq!(detect(b"hello world\n").as_deref(), Some("text/plain"));
        assert_eq!(
            detect(b"PK\x03\x04\x14\x00\x00\x00").as_deref(),
            Some("application/zip")
        );
        assert_eq!(
            detect(b"\x1f\x8b\x08\x00\x00").as_deref(),
            Some("application/gzip")
        );
        assert_eq!(detect(&[0, 1, 2, 3]).as_deref(), Some(OCTET_STREAM));
        assert_eq!(detect(b"").as_deref(), None);
    }

    #[test]
    fn mismatches_are_only_the_clear_ones() {
        assert!(!compatible(
            Some("text/plain"),
            Some("application/java-archive")
        ));
        assert!(compatible(
            Some("application/zip"),
            Some("application/java-archive")
        ));
        assert!(compatible(Some("text/plain"), Some("application/json")));
        assert!(compatible(Some("text/plain"), Some("application/xml")));
        assert!(compatible(
            Some("application/gzip"),
            Some("application/x-gzip")
        ));
        assert!(compatible(None, Some("application/java-archive")));
        assert!(compatible(Some("image/png"), None));
        assert!(compatible(Some("image/png"), Some(OCTET_STREAM)));
        assert!(!compatible(Some("image/png"), Some("text/plain")));
        assert!(!compatible(Some("application/zip"), Some("text/plain")));
    }
}
