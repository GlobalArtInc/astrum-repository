use std::collections::BTreeMap;
use std::fmt::Write;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::index::{File, Hashes, Project};

const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'+')
    .remove(b'!');

pub const REPOSITORY_VERSION: &str = "1.0";

pub fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

pub fn href(prefix: &str, path: &str) -> String {
    let encoded: Vec<String> = path
        .trim_start_matches('/')
        .split('/')
        .map(|segment| utf8_percent_encode(segment, SEGMENT).to_string())
        .collect();
    format!("{prefix}{}", encoded.join("/"))
}

fn hashes_value(hashes: &Hashes) -> String {
    match hashes.get("sha256").map(|v| ("sha256", v)).or_else(|| {
        hashes
            .iter()
            .next()
            .map(|(name, value)| (name.as_str(), value))
    }) {
        Some((name, value)) => format!("{name}={value}"),
        None => "true".to_owned(),
    }
}

fn head(title: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html>\n  <head>\n    <meta name=\"pypi:repository-version\" content=\"{REPOSITORY_VERSION}\">\n    <title>{}</title>\n  </head>\n  <body>\n",
        escape(title)
    )
}

pub fn render_project(project: &Project, prefix: &str) -> String {
    let title = format!("Links for {}", project.name);
    let mut page = head(&title);
    let _ = writeln!(page, "    <h1>{}</h1>", escape(&title));
    for file in &project.files {
        let mut link = href(prefix, &file.url);
        if let Some((algorithm, value)) = file.fragment() {
            let _ = write!(link, "#{algorithm}={value}");
        }
        let _ = write!(page, "    <a href=\"{}\"", escape(&link));
        if let Some(requires) = &file.requires_python {
            let _ = write!(page, " data-requires-python=\"{}\"", escape(requires));
        }
        if let Some(reason) = &file.yanked {
            let _ = write!(page, " data-yanked=\"{}\"", escape(reason));
        }
        if let Some(metadata) = &file.core_metadata {
            let value = escape(&hashes_value(metadata));
            let _ = write!(
                page,
                " data-dist-info-metadata=\"{value}\" data-core-metadata=\"{value}\""
            );
        }
        let _ = writeln!(page, ">{}</a><br/>", escape(&file.filename));
    }
    page.push_str("  </body>\n</html>\n");
    page
}

pub fn render_root(names: &[String], prefix: &str) -> String {
    let mut page = head("Simple index");
    for project in names {
        let link = format!("{}/", href(prefix, project));
        let _ = writeln!(
            page,
            "    <a href=\"{}\">{}</a><br/>",
            escape(&link),
            escape(project)
        );
    }
    page.push_str("  </body>\n</html>\n");
    page
}

#[derive(Debug, Default, PartialEq)]
pub struct Anchor {
    pub attributes: BTreeMap<String, String>,
    pub text: String,
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        _ => {
            let number = entity.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

pub fn unescape(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        decoded.push_str(&rest[..at]);
        let tail = &rest[at + 1..];
        match tail
            .find(';')
            .filter(|end| *end <= 10)
            .and_then(|end| decode_entity(&tail[..end]).map(|c| (c, end)))
        {
            Some((c, end)) => {
                decoded.push(c);
                rest = &tail[end + 1..];
            }
            None => {
                decoded.push('&');
                rest = tail;
            }
        }
    }
    decoded.push_str(rest);
    decoded
}

fn parse_attributes(tag: &str) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::new();
    let chars: Vec<char> = tag.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '/') {
            i += 1;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '=' && chars[i] != '/' {
            i += 1;
        }
        let name: String = chars[start..i].iter().collect::<String>().to_lowercase();
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        let mut value = String::new();
        if i < chars.len() && chars[i] == '=' {
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                let begin = i;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                value = chars[begin..i].iter().collect();
                i += 1;
            } else {
                let begin = i;
                while i < chars.len() && !chars[i].is_whitespace() {
                    i += 1;
                }
                value = chars[begin..i].iter().collect();
            }
        }
        if !name.is_empty() {
            attributes.entry(name).or_insert_with(|| unescape(&value));
        }
    }
    attributes
}

fn strip_tags(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut inside = false;
    for c in text.chars() {
        match c {
            '<' => inside = true,
            '>' if inside => inside = false,
            other if !inside => plain.push(other),
            _ => {}
        }
    }
    plain
}

pub fn anchors(page: &str) -> Vec<Anchor> {
    let lower = page.to_ascii_lowercase();
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = lower[from..].find("<a") {
        let start = from + offset;
        let after = start + 2;
        if !lower[after..].starts_with(|c: char| c.is_whitespace() || c == '>') {
            from = after;
            continue;
        }
        let Some(close) = lower[after..].find('>') else {
            break;
        };
        let tag = &page[after..after + close];
        let body_start = after + close + 1;
        let body_end = lower[body_start..]
            .find("</a")
            .map(|end| body_start + end)
            .unwrap_or(page.len());
        found.push(Anchor {
            attributes: parse_attributes(tag),
            text: unescape(strip_tags(&page[body_start..body_end]).trim()),
        });
        from = body_end.max(body_start);
    }
    found
}

fn parse_hashes(value: &str) -> Option<Hashes> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("false") {
        return None;
    }
    if value.eq_ignore_ascii_case("true") {
        return Some(Hashes::new());
    }
    let (algorithm, digest) = value.split_once('=')?;
    Some(
        [(
            algorithm.trim().to_ascii_lowercase(),
            digest.trim().to_owned(),
        )]
        .into(),
    )
}

pub fn parse_project(name: &str, page: &str, base: &url::Url) -> Project {
    let mut files = Vec::new();
    for anchor in anchors(page) {
        let Some(href) = anchor.attributes.get("href") else {
            continue;
        };
        let Ok(mut url) = base.join(href.trim()) else {
            continue;
        };
        let mut hashes = Hashes::new();
        if let Some((algorithm, digest)) = url.fragment().and_then(|f| f.split_once('=')) {
            hashes.insert(algorithm.to_ascii_lowercase(), digest.to_owned());
        }
        url.set_fragment(None);
        let filename = match anchor.text.trim() {
            "" => url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .map(|last| {
                    percent_encoding::percent_decode_str(last)
                        .decode_utf8_lossy()
                        .into_owned()
                })
                .unwrap_or_default(),
            text => text.to_owned(),
        };
        if !crate::name::is_safe_segment(&filename) {
            continue;
        }
        let core_metadata = anchor
            .attributes
            .get("data-core-metadata")
            .or_else(|| anchor.attributes.get("data-dist-info-metadata"))
            .and_then(|value| parse_hashes(value));
        files.push(File {
            filename,
            url: url.to_string(),
            version: None,
            hashes,
            requires_python: anchor
                .attributes
                .get("data-requires-python")
                .filter(|value| !value.trim().is_empty())
                .cloned(),
            yanked: anchor.attributes.get("data-yanked").cloned(),
            core_metadata,
            size: None,
            upload_time: None,
        });
    }
    Project {
        name: name.to_owned(),
        files,
        versions: Vec::new(),
    }
}

pub fn parse_root(page: &str) -> Vec<String> {
    anchors(page)
        .into_iter()
        .map(|anchor| anchor.text.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const UPSTREAM: &str = r#"<!DOCTYPE html>
<html>
  <head><meta name="pypi:repository-version" content="1.1"><title>Links for six</title></head>
  <body>
    <h1>Links for six</h1>
    <a href="https://files.pythonhosted.org/packages/b7/ce/149a/six-1.17.0-py2.py3-none-any.whl#sha256=4721f391" data-requires-python="!=3.0.*,!=3.1.*,!=3.2.*,&gt;=2.7" data-dist-info-metadata="sha256=5620" data-core-metadata="sha256=5620">six-1.17.0-py2.py3-none-any.whl</a><br />
    <A HREF='../../packages/six/1.0.0/six-1.0.0.tar.gz#md5=abc' data-yanked="">six-1.0.0.tar.gz</A>
    <a href="six-0.9.tar.gz" data-yanked="broken &amp; old" data-core-metadata=true>six-0.9.tar.gz</a>
    <a name="anchor-without-href">ignored</a>
    <abbr>not an anchor</abbr>
    <a href="../evil/..">..</a>
  </body>
</html>"#;

    #[test]
    fn upstream_pages_parse_into_files() {
        let base = url::Url::parse("https://pypi.example/simple/six/").unwrap();
        let project = parse_project("six", UPSTREAM, &base);
        assert_eq!(project.files.len(), 3);
        let wheel = &project.files[0];
        assert_eq!(wheel.filename, "six-1.17.0-py2.py3-none-any.whl");
        assert_eq!(
            wheel.url,
            "https://files.pythonhosted.org/packages/b7/ce/149a/six-1.17.0-py2.py3-none-any.whl"
        );
        assert_eq!(wheel.hashes["sha256"], "4721f391");
        assert_eq!(
            wheel.requires_python.as_deref(),
            Some("!=3.0.*,!=3.1.*,!=3.2.*,>=2.7")
        );
        assert_eq!(wheel.core_metadata.as_ref().unwrap()["sha256"], "5620");
        assert_eq!(wheel.yanked, None);
        let sdist = &project.files[1];
        assert_eq!(
            sdist.url,
            "https://pypi.example/packages/six/1.0.0/six-1.0.0.tar.gz"
        );
        assert_eq!(sdist.hashes["md5"], "abc");
        assert_eq!(sdist.yanked.as_deref(), Some(""));
        let old = &project.files[2];
        assert_eq!(old.url, "https://pypi.example/simple/six/six-0.9.tar.gz");
        assert_eq!(old.yanked.as_deref(), Some("broken & old"));
        assert_eq!(old.core_metadata, Some(Hashes::new()));
    }

    #[test]
    fn pages_render_escaped_and_relative() {
        let project = Project {
            name: "six".to_owned(),
            files: vec![File {
                filename: "six-1.0+local.tar.gz".to_owned(),
                url: "/packages/six/1.0+local/six-1.0+local.tar.gz".to_owned(),
                hashes: [("sha256".to_owned(), "ab".to_owned())].into(),
                requires_python: Some(">=3.8".to_owned()),
                yanked: Some("bad \"release\"".to_owned()),
                core_metadata: Some([("sha256".to_owned(), "cd".to_owned())].into()),
                ..Default::default()
            }],
            versions: vec![],
        };
        let page = render_project(&project, "../../");
        assert!(page.contains(
            "<a href=\"../../packages/six/1.0+local/six-1.0+local.tar.gz#sha256=ab\" data-requires-python=\"&gt;=3.8\" data-yanked=\"bad &quot;release&quot;\" data-dist-info-metadata=\"sha256=cd\" data-core-metadata=\"sha256=cd\">six-1.0+local.tar.gz</a>"
        ), "{page}");
        assert!(page.contains("<meta name=\"pypi:repository-version\" content=\"1.0\">"));
        let base = url::Url::parse("http://h/repository/r/simple/six/").unwrap();
        let parsed = parse_project("six", &page, &base);
        assert_eq!(parsed.files[0].requires_python.as_deref(), Some(">=3.8"));
        assert_eq!(parsed.files[0].yanked.as_deref(), Some("bad \"release\""));
        assert_eq!(
            parsed.files[0].url,
            "http://h/repository/r/packages/six/1.0+local/six-1.0+local.tar.gz"
        );
    }

    #[test]
    fn roots_render_and_parse() {
        let names = vec!["a-b".to_owned(), "six".to_owned()];
        let page = render_root(&names, "");
        assert!(page.contains("<a href=\"a-b/\">a-b</a><br/>"));
        assert_eq!(parse_root(&page), names);
        assert_eq!(
            parse_root("<a href=\"/simple/0/\">0</a>\n<a href=\"/simple/Zope/\">Zope</a>"),
            ["0", "Zope"]
        );
        assert!(render_root(&names, "simple/").contains("href=\"simple/six/\""));
    }

    #[test]
    fn entities_decode() {
        assert_eq!(
            unescape("&lt;&#62;&#x3D;&amp;amp;&bogus;&"),
            "<>=&amp;&bogus;&"
        );
    }
}
