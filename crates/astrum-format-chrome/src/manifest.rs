use astrum_content::zip::Source;
use serde_json::Value;

use crate::crx::{self, CrxError, valid_version};

pub const MANIFEST: &str = "manifest.json";
const MAX_MANIFEST: u64 = 1024 * 1024;
const MAX_MESSAGES: u64 = 4 * 1024 * 1024;
const MAX_TEXT: usize = 4096;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub manifest_version: u64,
    pub description: String,
    pub update_url: String,
}

fn invalid(message: impl Into<String>) -> CrxError {
    CrxError::Invalid(message.into())
}

pub fn relaxed(text: &str) -> String {
    let chars: Vec<char> = text.trim_start_matches('\u{feff}').chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut at = 0;
    let mut quoted = false;
    while at < chars.len() {
        let c = chars[at];
        if quoted {
            out.push(c);
            if c == '\\' {
                if let Some(next) = chars.get(at + 1) {
                    out.push(*next);
                }
                at += 2;
                continue;
            }
            quoted = c != '"';
            at += 1;
            continue;
        }
        match (c, chars.get(at + 1)) {
            ('"', _) => {
                quoted = true;
                out.push(c);
                at += 1;
            }
            ('/', Some('/')) => {
                while at < chars.len() && chars[at] != '\n' {
                    at += 1;
                }
            }
            ('/', Some('*')) => {
                at += 2;
                while at < chars.len() && !(chars[at] == '*' && chars.get(at + 1) == Some(&'/')) {
                    at += 1;
                }
                at += 2;
                out.push(' ');
            }
            (',', _) => {
                let next = chars[at + 1..]
                    .iter()
                    .find(|candidate| !candidate.is_whitespace());
                if !matches!(next, Some('}') | Some(']')) {
                    out.push(c);
                }
                at += 1;
            }
            _ => {
                out.push(c);
                at += 1;
            }
        }
    }
    out
}

pub fn parse_json(bytes: &[u8], file: &str) -> Result<Value, CrxError> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid(format!("{file} is not UTF-8")))?;
    let text = text.trim_start_matches('\u{feff}');
    serde_json::from_str(text)
        .or_else(|_| serde_json::from_str(&relaxed(text)))
        .map_err(|error| invalid(format!("{file} is not valid JSON: {error}")))
}

fn text(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(|found| found.trim().chars().take(MAX_TEXT).collect())
        .unwrap_or_default()
}

fn message_key(text: &str) -> Option<&str> {
    let key = text.strip_prefix("__MSG_")?.strip_suffix("__")?;
    (!key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@'))
    .then_some(key)
}

fn localized(messages: Option<&Value>, text: String) -> String {
    let (Some(Value::Object(messages)), Some(key)) = (messages, message_key(&text)) else {
        return text;
    };
    messages
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .and_then(|(_, entry)| entry.get("message"))
        .and_then(Value::as_str)
        .map(|message| message.trim().chars().take(MAX_TEXT).collect())
        .unwrap_or(text)
}

fn safe_locale(locale: &str) -> bool {
    !locale.is_empty()
        && locale.len() <= 16
        && locale
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn parse(manifest: &Value, messages: Option<&Value>) -> Result<Manifest, CrxError> {
    if !manifest.is_object() {
        return Err(invalid("manifest.json is not a JSON object"));
    }
    let version = match manifest.get("version") {
        Some(Value::String(version)) => version.trim().to_owned(),
        Some(_) => return Err(invalid("the version of manifest.json is not a string")),
        None => return Err(invalid("manifest.json has no version")),
    };
    if !valid_version(&version) {
        return Err(invalid(format!(
            "the version {version:?} of manifest.json is not an extension version: one to four numbers from 0 to 65535, separated by dots"
        )));
    }
    let manifest_version = manifest
        .get("manifest_version")
        .and_then(Value::as_u64)
        .filter(|number| (1..=1000).contains(number))
        .ok_or_else(|| invalid("manifest.json has no manifest_version"))?;
    let name = localized(messages, text(manifest, "name"));
    if name.is_empty() {
        return Err(invalid("manifest.json has no name"));
    }
    Ok(Manifest {
        name,
        version,
        manifest_version,
        description: localized(messages, text(manifest, "description")),
        update_url: text(manifest, "update_url"),
    })
}

pub async fn read(archive: &dyn Source) -> Result<Manifest, CrxError> {
    let entries = crx::entries(archive).await?;
    let found = entries
        .iter()
        .find(|entry| entry.name == MANIFEST)
        .ok_or_else(|| invalid("the extension has no manifest.json at the root of its archive"))?;
    let bytes = crx::entry(archive, found, MAX_MANIFEST).await?;
    let manifest = parse_json(&bytes, MANIFEST)?;
    let locale = text(&manifest, "default_locale");
    let mut messages = None;
    if safe_locale(&locale) {
        let wanted = format!("_locales/{locale}/messages.json");
        if let Some(entry) = entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&wanted))
        {
            let bytes = crx::entry(archive, entry, MAX_MESSAGES).await?;
            messages = parse_json(&bytes, &wanted).ok();
        }
    }
    parse(&manifest, messages.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn comments_and_trailing_commas_are_tolerated() {
        let text = "{\n  // the name\n  \"name\": \"a // b\", /* block */\n  \"list\": [1, 2,],\n  \"quote\": \"x\\\"y\",\n}";
        let value = parse_json(text.as_bytes(), MANIFEST).unwrap();
        assert_eq!(value["name"], "a // b");
        assert_eq!(value["list"], json!([1, 2]));
        assert_eq!(value["quote"], "x\"y");
        assert!(parse_json(b"{\"name\": ", MANIFEST).is_err());
    }

    #[test]
    fn messages_name_localized_extensions() {
        let manifest = json!({
            "manifest_version": 3,
            "name": "__MSG_appName__",
            "description": "__MSG_missing__",
            "version": "2.0.1",
            "update_url": "https://example.test/updates.xml",
            "default_locale": "en"
        });
        let messages = json!({"appname": {"message": "Localized Name"}});
        let parsed = parse(&manifest, Some(&messages)).unwrap();
        assert_eq!(parsed.name, "Localized Name");
        assert_eq!(parsed.description, "__MSG_missing__");
        assert_eq!(parsed.manifest_version, 3);
        assert_eq!(parsed.update_url, "https://example.test/updates.xml");
        assert_eq!(parse(&manifest, None).unwrap().name, "__MSG_appName__");
    }

    #[test]
    fn incomplete_manifests_are_refused() {
        let refused = |value: Value| parse(&value, None).unwrap_err().to_string();
        assert!(refused(json!({"manifest_version": 3, "name": "a"})).contains("no version"));
        assert!(
            refused(json!({"manifest_version": 3, "name": "a", "version": "1.0beta"}))
                .contains("not an extension version")
        );
        assert!(refused(json!({"name": "a", "version": "1"})).contains("manifest_version"));
        assert!(refused(json!({"manifest_version": 2, "version": "1"})).contains("no name"));
        assert!(refused(json!([1])).contains("not a JSON object"));
    }
}
