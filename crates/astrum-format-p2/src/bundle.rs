use std::collections::HashMap;

use astrum_content::zip::{self, Entry, Source};

use crate::xml;

const MAX_ENTRY: u64 = 4 * 1024 * 1024;
const DEFAULT_LOCALIZATION: &str = "OSGI-INF/l10n/bundle";

pub fn text(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => error
            .into_bytes()
            .iter()
            .map(|byte| *byte as char)
            .collect(),
    }
}

/// The headers of the main section of a JAR manifest.
pub fn manifest(text: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix(' ') {
            if let Some((_, value)) = current.as_mut() {
                value.push_str(rest);
            }
            continue;
        }
        if let Some((name, value)) = current.take() {
            headers.insert(name, value);
        }
        if let Some((name, value)) = line.split_once(':') {
            current = Some((name.trim().to_owned(), value.trim_start().to_owned()));
        }
    }
    if let Some((name, value)) = current {
        headers.insert(name, value);
    }
    headers
}

fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('f') => out.push('\u{c}'),
            Some('u') => {
                let code: String = chars.by_ref().take(4).collect();
                if let Some(decoded) = u32::from_str_radix(&code, 16).ok().and_then(char::from_u32)
                {
                    out.push(decoded);
                }
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

fn split_entry(line: &str) -> (String, String) {
    let mut escaped = false;
    let mut end = line.len();
    for (at, c) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '=' | ':' | ' ' | '\t' | '\u{c}' => {
                end = at;
                break;
            }
            _ => {}
        }
    }
    let key = unescape(&line[..end]);
    let mut rest = line[end..].trim_start_matches([' ', '\t', '\u{c}']);
    if let Some(stripped) = rest.strip_prefix(['=', ':']) {
        rest = stripped.trim_start_matches([' ', '\t', '\u{c}']);
    }
    (key, unescape(rest))
}

/// A Java properties file.
pub fn properties(text: &str) -> HashMap<String, String> {
    let mut found = HashMap::new();
    let mut logical = String::new();
    for raw in text.lines() {
        let line = match logical.is_empty() {
            true => raw.trim_start(),
            false => raw.trim_start_matches([' ', '\t', '\u{c}']),
        };
        if logical.is_empty() && (line.is_empty() || line.starts_with('#') || line.starts_with('!'))
        {
            continue;
        }
        let trailing = line.len() - line.trim_end_matches('\\').len();
        if trailing % 2 == 1 {
            logical.push_str(&line[..line.len() - 1]);
            continue;
        }
        logical.push_str(line);
        let (key, value) = split_entry(&logical);
        found.insert(key, value);
        logical.clear();
    }
    if !logical.is_empty() {
        let (key, value) = split_entry(&logical);
        found.insert(key, value);
    }
    found
}

async fn entry_text(source: &dyn Source, entries: &[Entry], name: &str) -> Option<String> {
    let entry = entries.iter().find(|entry| entry.name == name)?;
    zip::read(source, entry, MAX_ENTRY).await.ok().map(text)
}

async fn localized(
    source: &dyn Source,
    entries: &[Entry],
    value: &str,
    base: &str,
) -> Option<String> {
    let Some(key) = value.strip_prefix('%') else {
        return Some(value.to_owned());
    };
    let file = entry_text(source, entries, &format!("{base}.properties")).await?;
    properties(&file).remove(key.trim())
}

/// The human name of a plugin (`Bundle-Name`) or feature (`label`), localized with the default
/// properties file of the JAR.
pub async fn name(source: &dyn Source, feature: bool) -> Option<String> {
    let entries = zip::entries(source).await.ok()?;
    let found = match feature {
        true => {
            let descriptor = entry_text(source, &entries, "feature.xml").await?;
            let label = xml::feature_label(&descriptor)?;
            localized(source, &entries, &label, "feature").await?
        }
        false => {
            let manifest = manifest(&entry_text(source, &entries, "META-INF/MANIFEST.MF").await?);
            let value = manifest.get("Bundle-Name")?;
            let base = manifest
                .get("Bundle-Localization")
                .map(|base| base.trim().to_owned())
                .unwrap_or_else(|| DEFAULT_LOCALIZATION.to_owned());
            localized(source, &entries, value, &base).await?
        }
    };
    let trimmed = found.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

pub fn xz_decompress(bytes: &[u8], limit: u64) -> Result<Vec<u8>, String> {
    struct Limited {
        out: Vec<u8>,
        limit: usize,
    }
    impl std::io::Write for Limited {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.out.len() + buf.len() > self.limit {
                return Err(std::io::Error::other("the file is too large"));
            }
            self.out.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut out = Limited {
        out: Vec::new(),
        limit: limit as usize,
    };
    lzma_rs::xz_decompress(&mut std::io::BufReader::new(bytes), &mut out)
        .map_err(|error| format!("the xz file cannot be read: {error}"))?;
    Ok(out.out)
}

pub fn xz_compress(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    lzma_rs::xz_compress(&mut std::io::BufReader::new(bytes), &mut out)
        .expect("writing into memory does not fail");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifests_join_continued_lines() {
        let headers = manifest(
            "Manifest-Version: 1.0\r\nBundle-SymbolicName: org.example.core;singleton:=t\r\n rue\r\nBundle-Name: %pluginName\r\n\r\nName: x\r\nBundle-Name: other\r\n",
        );
        assert_eq!(
            headers["Bundle-SymbolicName"],
            "org.example.core;singleton:=true"
        );
        assert_eq!(headers["Bundle-Name"], "%pluginName");
    }

    #[test]
    fn properties_files_follow_the_java_rules() {
        let found = properties(
            "# comment\n! other\npluginName = Example Core\nproviderName:GlobalArt\nlong = first \\\n    second\nescaped\\ key=caf\\u00e9\\tbar\nempty\n",
        );
        assert_eq!(found["pluginName"], "Example Core");
        assert_eq!(found["providerName"], "GlobalArt");
        assert_eq!(found["long"], "first second");
        assert_eq!(found["escaped key"], "café\tbar");
        assert_eq!(found["empty"], "");
    }

    #[tokio::test]
    async fn plugin_and_feature_names_are_localized() {
        let plugin = zip::write(&[
            (
                "META-INF/MANIFEST.MF",
                b"Manifest-Version: 1.0\nBundle-Name: %name\nBundle-Localization: plugin\n",
            ),
            ("plugin.properties", b"name=Example Core\n"),
        ]);
        assert_eq!(
            name(&zip::Memory(plugin), false).await,
            Some("Example Core".to_owned())
        );
        let default = zip::write(&[
            ("META-INF/MANIFEST.MF", b"Bundle-Name: %bundleName\n"),
            (
                "OSGI-INF/l10n/bundle.properties",
                b"bundleName=Example UI\n",
            ),
        ]);
        assert_eq!(
            name(&zip::Memory(default), false).await,
            Some("Example UI".to_owned())
        );
        let plain = zip::write(&[("META-INF/MANIFEST.MF", b"Bundle-Name: Plain\n")]);
        assert_eq!(
            name(&zip::Memory(plain), false).await,
            Some("Plain".to_owned())
        );
        let feature = zip::write(&[
            (
                "feature.xml",
                b"<feature id='f' label='%featureName' version='1.0.0'/>",
            ),
            ("feature.properties", b"featureName=Example Feature\n"),
        ]);
        assert_eq!(
            name(&zip::Memory(feature), true).await,
            Some("Example Feature".to_owned())
        );
        let missing = zip::write(&[("META-INF/MANIFEST.MF", b"Bundle-Name: %gone\n")]);
        assert_eq!(name(&zip::Memory(missing), false).await, None);
        assert_eq!(name(&zip::Memory(b"not a jar".to_vec()), false).await, None);
    }

    #[test]
    fn xz_round_trips_and_stops_at_its_limit() {
        let text = "<repository/>\n".repeat(1000);
        let packed = xz_compress(text.as_bytes());
        assert_eq!(xz_decompress(&packed, 1 << 20).unwrap(), text.as_bytes());
        assert!(xz_decompress(&packed, 100).is_err());
        assert!(xz_decompress(b"plain", 100).is_err());
    }
}
