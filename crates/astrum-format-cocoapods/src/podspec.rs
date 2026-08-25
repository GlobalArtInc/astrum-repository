use md5::{Digest, Md5};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde_json::{Map, Value};

pub const DEFAULT_PREFIX_LENGTHS: [usize; 3] = [1, 1, 1];
const MAX_PREFIXES: usize = 8;
const MAX_ARCHIVE_NAME: usize = 200;

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
    .add(b'}');

fn plain_segment(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 255
        && !text.starts_with('.')
        && !text.contains('/')
        && !text.chars().any(char::is_control)
}

pub fn valid_name(name: &str) -> bool {
    plain_segment(name)
}

pub fn valid_version(version: &str) -> bool {
    plain_segment(version) && !version.contains(char::is_whitespace)
}

pub fn valid_archive_name(file: &str) -> bool {
    file.len() <= MAX_ARCHIVE_NAME
        && !file.starts_with('.')
        && !file.is_empty()
        && file
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+' | '~'))
}

pub fn valid_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix.len() <= 32
        && prefix
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

pub fn shard(name: &str, lengths: &[usize]) -> Vec<String> {
    let hashed = hex::encode(Md5::digest(name.as_bytes()));
    let mut rest = hashed.as_str();
    let mut parts = Vec::with_capacity(lengths.len());
    for length in lengths {
        let length = (*length).min(rest.len());
        let (part, tail) = rest.split_at(length);
        if !part.is_empty() {
            parts.push(part.to_owned());
        }
        rest = tail;
    }
    parts
}

pub fn podspec_path(name: &str, version: &str, lengths: &[usize]) -> String {
    let mut path = String::from("/Specs");
    for part in shard(name, lengths) {
        path.push('/');
        path.push_str(&part);
    }
    format!("{path}/{name}/{version}/{name}.podspec.json")
}

pub fn versions_index_path(name: &str, lengths: &[usize]) -> String {
    let parts = shard(name, lengths);
    if parts.is_empty() {
        "/all_pods_versions.txt".to_owned()
    } else {
        format!("/all_pods_versions_{}.txt", parts.join("_"))
    }
}

pub fn archive_path(name: &str, version: &str, file: &str) -> String {
    format!("/pods/{name}/{version}/{file}")
}

pub fn archive_location(name: &str, version: &str, file: &str) -> String {
    format!(
        "/pods/{}/{}/{}",
        utf8_percent_encode(name, SEGMENT),
        utf8_percent_encode(version, SEGMENT),
        utf8_percent_encode(file, SEGMENT)
    )
}

pub fn prefix_lengths(yaml: &[u8]) -> Option<Vec<usize>> {
    let document: Value = serde_norway::from_slice(yaml).ok()?;
    let lengths = document.get("prefix_lengths")?.as_array()?;
    if lengths.len() > MAX_PREFIXES {
        return None;
    }
    lengths
        .iter()
        .map(|length| {
            length
                .as_u64()
                .filter(|length| (1..=32).contains(length))
                .map(|length| length as usize)
        })
        .collect()
}

pub fn archive_name(url: &url::Url) -> String {
    let last = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .unwrap_or_default();
    let decoded = percent_decode_str(last).decode_utf8_lossy();
    let mut name: String = decoded
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+' | '~') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = name.trim_start_matches('.').len();
    name.drain(..name.len() - trimmed);
    if name.len() > MAX_ARCHIVE_NAME {
        name.drain(..name.len() - MAX_ARCHIVE_NAME);
        let trimmed = name.trim_start_matches('.').len();
        name.drain(..name.len() - trimmed);
    }
    if name.is_empty() {
        "source".to_owned()
    } else {
        name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpSource {
    pub url: url::Url,
    pub headers: Vec<(String, String)>,
}

pub fn http_source(spec: &Value) -> Option<HttpSource> {
    let source = spec.get("source")?.as_object()?;
    let url = url::Url::parse(source.get("http")?.as_str()?).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return None;
    }
    let headers = match source.get("headers") {
        Some(Value::Array(headers)) => headers
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|header| header.split_once(':'))
            .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
            .filter(|(name, _)| !name.is_empty())
            .collect(),
        _ => Vec::new(),
    };
    Some(HttpSource { url, headers })
}

pub fn rewritten_source(spec: &Value, location: &str) -> Option<Value> {
    let mut source: Map<String, Value> = spec.get("source")?.as_object()?.clone();
    source.insert("http".to_owned(), Value::String(location.to_owned()));
    source.remove("headers");
    Some(Value::Object(source))
}

pub fn rewrite(text: &str, spec: &Value, location: &str) -> Option<String> {
    let source = rewritten_source(spec, location)?;
    if let Some(spliced) = splice(text, "source", &source)
        && serde_json::from_str::<Value>(&spliced)
            .ok()
            .and_then(|document| document.get("source").cloned())
            .is_some_and(|written| written == source)
    {
        return Some(spliced);
    }
    let mut document = spec.as_object()?.clone();
    document.insert("source".to_owned(), source);
    serde_json::to_string_pretty(&Value::Object(document)).ok()
}

fn splice(text: &str, key: &str, value: &Value) -> Option<String> {
    let (key_start, value_start, value_end) = top_level_member(text.as_bytes(), key)?;
    let line_start = text[..key_start].rfind('\n').map(|at| at + 1);
    let replacement = match line_start {
        Some(line_start) => {
            let indent = &text[line_start..key_start];
            if !indent.chars().all(|c| c == ' ' || c == '\t') {
                return None;
            }
            serde_json::to_string_pretty(value)
                .ok()?
                .replace('\n', &format!("\n{indent}"))
        }
        None => serde_json::to_string(value).ok()?,
    };
    Some(format!(
        "{}{replacement}{}",
        &text[..value_start],
        &text[value_end..]
    ))
}

fn skip_whitespace(bytes: &[u8], mut at: usize) -> usize {
    while at < bytes.len() && matches!(bytes[at], b' ' | b'\t' | b'\n' | b'\r') {
        at += 1;
    }
    at
}

fn skip_string(bytes: &[u8], mut at: usize) -> Option<usize> {
    if bytes.get(at) != Some(&b'"') {
        return None;
    }
    at += 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b'"' => return Some(at + 1),
            _ => at += 1,
        }
    }
    None
}

fn skip_value(bytes: &[u8], at: usize) -> Option<usize> {
    match bytes.get(at)? {
        b'"' => skip_string(bytes, at),
        b'{' | b'[' => {
            let mut depth = 0usize;
            let mut at = at;
            while at < bytes.len() {
                match bytes[at] {
                    b'"' => {
                        at = skip_string(bytes, at)?;
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(at + 1);
                        }
                    }
                    _ => {}
                }
                at += 1;
            }
            None
        }
        _ => {
            let mut at = at;
            while at < bytes.len()
                && !matches!(bytes[at], b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r')
            {
                at += 1;
            }
            Some(at)
        }
    }
}

fn top_level_member(bytes: &[u8], key: &str) -> Option<(usize, usize, usize)> {
    let mut at = skip_whitespace(bytes, 0);
    if bytes.get(at) != Some(&b'{') {
        return None;
    }
    at += 1;
    loop {
        at = skip_whitespace(bytes, at);
        if bytes.get(at) == Some(&b'}') {
            return None;
        }
        let key_start = at;
        let key_end = skip_string(bytes, at)?;
        let name: String = serde_json::from_slice(&bytes[key_start..key_end]).ok()?;
        at = skip_whitespace(bytes, key_end);
        if bytes.get(at) != Some(&b':') {
            return None;
        }
        let value_start = skip_whitespace(bytes, at + 1);
        let value_end = skip_value(bytes, value_start)?;
        if name == key {
            return Some((key_start, value_start, value_end));
        }
        at = skip_whitespace(bytes, value_end);
        match bytes.get(at) {
            Some(b',') => at += 1,
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn shards_follow_the_md5_of_the_name() {
        assert_eq!(
            shard("AFNetworking", &DEFAULT_PREFIX_LENGTHS),
            ["a", "7", "5"]
        );
        assert_eq!(shard("AFNetworking", &[2, 1]), ["a7", "5"]);
        assert!(shard("AFNetworking", &[]).is_empty());
        assert_eq!(
            podspec_path("AFNetworking", "4.0.1", &DEFAULT_PREFIX_LENGTHS),
            "/Specs/a/7/5/AFNetworking/4.0.1/AFNetworking.podspec.json"
        );
        assert_eq!(
            versions_index_path("Alamofire", &DEFAULT_PREFIX_LENGTHS),
            "/all_pods_versions_d_a_2.txt"
        );
        assert_eq!(
            versions_index_path("Alamofire", &[]),
            "/all_pods_versions.txt"
        );
    }

    #[test]
    fn prefix_lengths_come_from_the_version_file() {
        let yaml = b"---\nmin: 1.0.0\nlast: 1.16.1\nprefix_lengths:\n- 1\n- 1\n- 1\n";
        assert_eq!(prefix_lengths(yaml), Some(vec![1, 1, 1]));
        assert_eq!(
            prefix_lengths(b"min: 1.0.0\nprefix_lengths: []\n"),
            Some(vec![])
        );
        assert_eq!(prefix_lengths(b"min: 1.0.0\n"), None);
        assert_eq!(prefix_lengths(b"prefix_lengths: [0]\n"), None);
        assert_eq!(prefix_lengths(b"not: [yaml"), None);
    }

    #[test]
    fn names_and_versions_are_plain_path_segments() {
        for name in [
            "AFNetworking",
            "Firebase",
            "React-Core",
            "lottie-ios",
            "NSData+Base64",
            "a_b.c",
        ] {
            assert!(valid_name(name), "{name}");
        }
        for name in [
            "!ProtoCompiler",
            "ID.me WebVerify",
            "AFDateHelper(icanzilb)",
            "ATMHud@dhoerl",
            "Floater💩",
        ] {
            assert!(valid_name(name), "{name}");
        }
        for name in ["", ".hidden", "a/b", "..", "a\nb"] {
            assert!(!valid_name(name), "{name:?}");
        }
        assert_eq!(
            archive_location("ID.me WebVerify", "1.0", "kit.zip"),
            "/pods/ID.me%20WebVerify/1.0/kit.zip"
        );
        assert_eq!(
            archive_location("!Proto#1", "1.0", "a.zip"),
            "/pods/!Proto%231/1.0/a.zip"
        );
        assert_eq!(
            shard("!ProtoCompiler", &DEFAULT_PREFIX_LENGTHS),
            shard("!ProtoCompiler", &[1, 1, 1])
        );
        for version in ["1.0.0", "2.0.0-beta.1", "1.0RC1", "3.0.0+build"] {
            assert!(valid_version(version), "{version}");
        }
        assert!(!valid_version("1 0") && !valid_version("1/0"));
        assert!(valid_prefix("a") && valid_prefix("0f") && !valid_prefix("G") && !valid_prefix(""));
    }

    #[test]
    fn archives_keep_the_file_name_of_their_address() {
        let name = |url: &str| archive_name(&url::Url::parse(url).unwrap());
        assert_eq!(
            name("https://github.com/acme/kit/archive/refs/tags/1.2.0.tar.gz"),
            "1.2.0.tar.gz"
        );
        assert_eq!(
            name(
                "https://dl.google.com/firebase/ios/analytics/673ce3bb917a7de5/GoogleAppMeasurement-9.6.0.tar.gz"
            ),
            "GoogleAppMeasurement-9.6.0.tar.gz"
        );
        assert_eq!(name("https://example.com/files/a%20b.zip?x=1"), "a_b.zip");
        assert_eq!(name("https://example.com/"), "source");
        assert_eq!(name("https://example.com/..zip"), "zip");
        let long = format!("https://example.com/{}.zip", "x".repeat(300));
        let shortened = name(&long);
        assert_eq!(shortened.len(), MAX_ARCHIVE_NAME);
        assert!(shortened.ends_with(".zip"));
        assert!(valid_archive_name(&shortened));
    }

    #[test]
    fn http_sources_are_read_with_their_headers() {
        let spec = json!({"source": {"http": "https://example.com/kit.zip", "sha256": "ab", "headers": ["Accept: application/octet-stream", "broken"]}});
        let source = http_source(&spec).unwrap();
        assert_eq!(source.url.as_str(), "https://example.com/kit.zip");
        assert_eq!(
            source.headers,
            [("Accept".to_owned(), "application/octet-stream".to_owned())]
        );
        for other in [
            json!({"source": {"git": "https://github.com/acme/kit.git", "tag": "1.0"}}),
            json!({"source": {"http": "file:///tmp/kit.zip"}}),
            json!({"source": {"http": 5}}),
            json!({"name": "x"}),
        ] {
            assert_eq!(http_source(&other), None, "{other}");
        }
    }

    #[test]
    fn rewriting_replaces_only_the_source() {
        let text = "{\n  \"name\": \"Kit\",\n  \"version\": \"1.0\",\n  \"source\": {\n    \"http\": \"https://example.com/kit.zip\",\n    \"sha256\": \"ab\",\n    \"headers\": [\"Accept: */*\"]\n  },\n  \"summary\": \"A kit with \\\"quotes\\\" and {braces}\"\n}\n";
        let spec: Value = serde_json::from_str(text).unwrap();
        let rewritten = rewrite(
            text,
            &spec,
            "http://astrum/repository/pods/pods/Kit/1.0/kit.zip",
        )
        .unwrap();
        assert_eq!(
            rewritten,
            "{\n  \"name\": \"Kit\",\n  \"version\": \"1.0\",\n  \"source\": {\n    \"http\": \"http://astrum/repository/pods/pods/Kit/1.0/kit.zip\",\n    \"sha256\": \"ab\"\n  },\n  \"summary\": \"A kit with \\\"quotes\\\" and {braces}\"\n}\n"
        );
        let compact =
            r#"{"name":"Kit","source":{"http":"https://example.com/kit.zip"},"version":"1.0"}"#;
        let spec: Value = serde_json::from_str(compact).unwrap();
        assert_eq!(
            rewrite(compact, &spec, "http://a/k.zip").unwrap(),
            r#"{"name":"Kit","source":{"http":"http://a/k.zip"},"version":"1.0"}"#
        );
    }

    #[test]
    fn a_duplicated_source_falls_back_to_a_written_document() {
        let text = r#"{"source":{"http":"https://a/one.zip"},"name":"Kit","source":{"http":"https://a/two.zip"}}"#;
        let spec: Value = serde_json::from_str(text).unwrap();
        let rewritten = rewrite(text, &spec, "http://astrum/two.zip").unwrap();
        let document: Value = serde_json::from_str(&rewritten).unwrap();
        assert_eq!(document["source"]["http"], "http://astrum/two.zip");
        assert!(!rewritten.contains("https://a/"));
    }
}
