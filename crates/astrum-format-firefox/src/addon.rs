use astrum_content::zip::{self, Source, ZipError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MANIFEST: &str = "manifest.json";
const MAX_MANIFEST: u64 = 4 * 1024 * 1024;
const MAX_MESSAGES: u64 = 4 * 1024 * 1024;
const MAX_SIGNATURE: u64 = 1024 * 1024;
const MAX_ID: usize = 255;
const MAX_VERSION: usize = 128;
const MAX_NAME: usize = 512;
const SIGNATURES: &[&str] = &["META-INF/mozilla.rsa", "META-INF/cose.sig"];
const PKCS7: &str = "META-INF/mozilla.rsa";
const COMMON_NAME: [u8; 5] = [0x06, 0x03, 0x55, 0x04, 0x03];

#[derive(Debug, thiserror::Error)]
pub enum AddonError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Store(#[from] astrum_blob::Error),
}

fn invalid(message: impl Into<String>) -> AddonError {
    AddonError::Invalid(message.into())
}

fn archive_error(error: ZipError) -> AddonError {
    match error {
        ZipError::Store(error) => AddonError::Store(error),
        other => invalid(format!(
            "the file is not a Firefox add-on (an .xpi zip archive): {other}"
        )),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Addon {
    pub id: String,
    pub name: String,
    pub version: String,
    pub signed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_version: Option<String>,
}

fn uuid_of(id: &str) -> Option<&str> {
    id.strip_prefix('{')?.strip_suffix('}')
}

pub fn valid_id(id: &str) -> bool {
    if id.is_empty() || id.len() > MAX_ID {
        return false;
    }
    if let Some(uuid) = uuid_of(id) {
        let lengths: Vec<usize> = uuid.split('-').map(str::len).collect();
        return lengths == [8, 4, 4, 4, 12]
            && uuid.chars().all(|c| c == '-' || c.is_ascii_hexdigit());
    }
    let Some((local, domain)) = id.split_once('@') else {
        return false;
    };
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_');
    !domain.is_empty() && local.chars().all(allowed) && domain.chars().all(allowed)
}

pub fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= MAX_VERSION
        && version.starts_with(|c: char| c.is_ascii_digit())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
}

pub fn valid_key(key: &str) -> bool {
    valid_id(key)
        || (!key.is_empty()
            && key.len() <= MAX_ID
            && !matches!(key, "." | ".." | "-")
            && key
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '~')))
}

fn file_safe(text: &str) -> String {
    text.chars()
        .map(
            |c| match c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                true => c,
                false => '_',
            },
        )
        .collect::<String>()
        .trim_start_matches('.')
        .to_owned()
}

pub fn file_name(id: &str, version: &str) -> String {
    let stem = match uuid_of(id) {
        Some(uuid) => uuid,
        None => id
            .split('@')
            .next()
            .filter(|local| !local.is_empty())
            .unwrap_or(id),
    };
    let stem = match file_safe(stem) {
        safe if safe.is_empty() => "addon".to_owned(),
        safe => safe,
    };
    format!("{stem}-{}.xpi", file_safe(version))
}

pub fn addon_path(id: &str, version: &str) -> String {
    format!("/{id}/{version}/{}", file_name(id, version))
}

impl Addon {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err(
                "manifest.json names no add-on id; set browser_specific_settings.gecko.id"
                    .to_owned(),
            );
        }
        if !valid_id(&self.id) {
            return Err(format!(
                "the add-on id {:?} is neither like name@example.org nor a UUID in braces",
                self.id
            ));
        }
        if self.version.is_empty() {
            return Err("manifest.json has no version".to_owned());
        }
        if !valid_version(&self.version) {
            return Err(format!(
                "the version {:?} of manifest.json is not an add-on version",
                self.version
            ));
        }
        Ok(())
    }

    pub fn path(&self) -> String {
        addon_path(&self.id, &self.version)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Manifest {
    pub id: Option<String>,
    pub name: String,
    pub version: String,
    pub default_locale: Option<String>,
    pub min_version: Option<String>,
    pub max_version: Option<String>,
}

fn without_comments(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn json_of(bytes: &[u8], what: &str) -> Result<Value, AddonError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid(format!("{what} is not UTF-8")))?
        .trim_start_matches('\u{feff}');
    serde_json::from_str(&without_comments(text))
        .map_err(|error| invalid(format!("{what} is not valid JSON: {error}")))
}

fn text(node: Option<&Value>) -> Option<String> {
    node.and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub fn manifest(value: &Value) -> Result<Manifest, AddonError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("manifest.json is not a JSON object"))?;
    if !object.contains_key("manifest_version") {
        return Err(invalid(
            "manifest.json has no manifest_version, so it is not a WebExtension",
        ));
    }
    let gecko = ["browser_specific_settings", "applications"]
        .iter()
        .find_map(|key| object.get(*key)?.get("gecko")?.as_object());
    let field = |name: &str| text(gecko.and_then(|gecko| gecko.get(name)));
    Ok(Manifest {
        id: field("id"),
        name: text(object.get("name")).unwrap_or_default(),
        version: text(object.get("version")).unwrap_or_default(),
        default_locale: text(object.get("default_locale")),
        min_version: field("strict_min_version"),
        max_version: field("strict_max_version"),
    })
}

pub fn message_key(name: &str) -> Option<&str> {
    name.strip_prefix("__MSG_")?
        .strip_suffix("__")
        .filter(|key| !key.is_empty())
}

fn message(messages: &Value, key: &str) -> Option<String> {
    let (_, entry) = messages
        .as_object()?
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))?;
    text(entry.get("message"))
}

fn safe_locale(locale: &str) -> bool {
    !locale.is_empty()
        && locale.len() <= 32
        && locale
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

fn common_names(der: &[u8]) -> Vec<String> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = der.get(at..).and_then(|rest| {
        rest.windows(COMMON_NAME.len())
            .position(|w| w == COMMON_NAME)
    }) {
        let start = at + offset + COMMON_NAME.len();
        at = start;
        let (Some(&tag), Some(&first)) = (der.get(start), der.get(start + 1)) else {
            break;
        };
        if !matches!(tag, 0x0c | 0x13 | 0x16) {
            continue;
        }
        let (length, skip) = match first {
            length if length < 0x80 => (length as usize, 2),
            0x81 => match der.get(start + 2) {
                Some(&length) => (length as usize, 3),
                None => break,
            },
            _ => continue,
        };
        if let Some(value) = der.get(start + skip..start + skip + length)
            && let Ok(name) = std::str::from_utf8(value)
        {
            found.push(name.to_owned());
        }
    }
    found
}

pub fn signature_id(der: &[u8]) -> Option<String> {
    common_names(der).into_iter().find(|name| valid_id(name))
}

fn shortened(name: String) -> String {
    match name.char_indices().nth(MAX_NAME) {
        Some((at, _)) => name[..at].to_owned(),
        None => name,
    }
}

pub async fn inspect(source: &dyn Source) -> Result<Addon, AddonError> {
    let entries = zip::entries(source).await.map_err(archive_error)?;
    let find = |name: &str| entries.iter().find(|entry| entry.name == name);
    let entry = find(MANIFEST).ok_or_else(|| {
        invalid("the file is not a Firefox add-on: it has no manifest.json at the root")
    })?;
    let bytes = zip::read(source, entry, MAX_MANIFEST)
        .await
        .map_err(archive_error)?;
    let parsed = manifest(&json_of(&bytes, "manifest.json")?)?;
    let signed = entries.iter().any(|entry| {
        SIGNATURES
            .iter()
            .any(|signature| entry.name.eq_ignore_ascii_case(signature))
    });
    let mut name = parsed.name.clone();
    if let Some(key) = message_key(&parsed.name)
        && let Some(locale) = parsed.default_locale.as_deref().filter(|l| safe_locale(l))
        && let Some(entry) = find(&format!("_locales/{locale}/messages.json"))
    {
        let bytes = zip::read(source, entry, MAX_MESSAGES)
            .await
            .map_err(archive_error)?;
        if let Some(found) = json_of(&bytes, "messages.json")
            .ok()
            .and_then(|messages| message(&messages, key))
        {
            name = found;
        }
    }
    let mut id = parsed.id.clone();
    if id.is_none()
        && let Some(entry) = entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(PKCS7))
    {
        let der = zip::read(source, entry, MAX_SIGNATURE)
            .await
            .map_err(archive_error)?;
        id = signature_id(&der);
    }
    Ok(Addon {
        id: id.unwrap_or_default(),
        name: shortened(name),
        version: parsed.version,
        signed,
        min_version: parsed.min_version,
        max_version: parsed.max_version,
    })
}

#[cfg(test)]
mod tests {
    use astrum_content::zip::Memory;
    use serde_json::json;

    use super::*;

    #[test]
    fn ids_are_email_like_or_braced_uuids() {
        for good in [
            "name@example.org",
            "@example",
            "uBlock0@raymondhill.net",
            "{ec8030f7-c20a-464f-9b0e-13a3a9e97384}",
            "{EC8030F7-C20A-464F-9B0E-13A3A9E97384}",
        ] {
            assert!(valid_id(good), "{good}");
        }
        for bad in [
            "",
            "name",
            "name@",
            "a@b@c",
            "a/b@example.org",
            "name@exa mple.org",
            "{ec8030f7-c20a-464f-9b0e}",
            "{ec8030f7c20a464f9b0e13a3a9e97384}",
            "{zc8030f7-c20a-464f-9b0e-13a3a9e97384}",
        ] {
            assert!(!valid_id(bad), "{bad}");
        }
        assert!(valid_key("ublock-origin"));
        assert!(valid_key("12345"));
        assert!(valid_key("name@example.org"));
        assert!(!valid_key("-"));
        assert!(!valid_key(".."));
        assert!(!valid_key("a b"));
    }

    #[test]
    fn versions_start_with_a_digit() {
        for good in ["1", "1.2.3", "1.0b2", "2.0pre", "1.0+", "10.0.0.1"] {
            assert!(valid_version(good), "{good}");
        }
        for bad in ["", "latest", "v1.0", "1.0/2", "1 0", "1.0%2f", ".1"] {
            assert!(!valid_version(bad), "{bad}");
        }
    }

    #[test]
    fn files_are_named_after_the_id() {
        assert_eq!(
            addon_path("name@example.org", "1.2.3"),
            "/name@example.org/1.2.3/name-1.2.3.xpi"
        );
        assert_eq!(
            addon_path("{ec8030f7-c20a-464f-9b0e-13a3a9e97384}", "1.0"),
            "/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/1.0/ec8030f7-c20a-464f-9b0e-13a3a9e97384-1.0.xpi"
        );
        assert_eq!(file_name("@example.org", "1.0+"), "_example.org-1.0_.xpi");
        assert_eq!(file_name(".hidden@x", "1"), "hidden-1.xpi");
    }

    #[test]
    fn manifests_name_the_gecko_id_and_versions() {
        let parsed = manifest(&json!({
            "manifest_version": 2,
            "name": "  Example  ",
            "version": "1.2.3",
            "applications": {"gecko": {"id": "old@example.org"}},
            "browser_specific_settings": {"gecko": {"id": "new@example.org", "strict_min_version": "115.0", "strict_max_version": "130.*"}}
        }))
        .unwrap();
        assert_eq!(parsed.id.as_deref(), Some("new@example.org"));
        assert_eq!(parsed.name, "Example");
        assert_eq!(parsed.min_version.as_deref(), Some("115.0"));
        assert_eq!(parsed.max_version.as_deref(), Some("130.*"));
        let legacy = manifest(&json!({
            "manifest_version": 2,
            "name": "x",
            "version": "1",
            "browser_specific_settings": {"edge": {}},
            "applications": {"gecko": {"id": "old@example.org"}}
        }))
        .unwrap();
        assert_eq!(legacy.id.as_deref(), Some("old@example.org"));
        assert!(legacy.min_version.is_none());
        assert!(manifest(&json!({"name": "x", "version": "1"})).is_err());
        assert!(manifest(&json!([1])).is_err());
    }

    #[test]
    fn comments_and_byte_order_marks_are_tolerated() {
        let text = "\u{feff}{\n  // the name\n  \"manifest_version\": 3,\n  \"name\": \"x // y\",\n  \"version\": \"1\"\n}";
        let parsed = manifest(&json_of(text.as_bytes(), "manifest.json").unwrap()).unwrap();
        assert_eq!(parsed.name, "x // y");
    }

    #[test]
    fn common_names_of_a_certificate_give_the_id() {
        let mut der = vec![0x30, 0x10];
        for name in [
            "root-ca-production-amo",
            "signingca1.addons.mozilla.org",
            "a@b.org",
        ] {
            der.extend_from_slice(&COMMON_NAME);
            der.push(0x0c);
            der.push(name.len() as u8);
            der.extend_from_slice(name.as_bytes());
        }
        assert_eq!(signature_id(&der).as_deref(), Some("a@b.org"));
        assert_eq!(signature_id(&der[..der.len() - 3]), None);
        assert_eq!(signature_id(b"nothing here"), None);
    }

    #[tokio::test]
    async fn an_xpi_is_read_with_its_localized_name_and_signature() {
        let manifest = json!({
            "manifest_version": 2,
            "name": "__MSG_extensionName__",
            "version": "2.0",
            "default_locale": "en_US",
            "browser_specific_settings": {"gecko": {"id": "{ec8030f7-c20a-464f-9b0e-13a3a9e97384}", "strict_min_version": "109.0"}}
        })
        .to_string();
        let messages = json!({"ExtensionName": {"message": "Localized Name"}}).to_string();
        let xpi = zip::write(&[
            ("manifest.json", manifest.as_bytes()),
            ("_locales/en_US/messages.json", messages.as_bytes()),
            ("META-INF/cose.sig", b"sig"),
        ]);
        let addon = inspect(&Memory(xpi)).await.unwrap();
        assert_eq!(addon.id, "{ec8030f7-c20a-464f-9b0e-13a3a9e97384}");
        assert_eq!(addon.name, "Localized Name");
        assert_eq!(addon.version, "2.0");
        assert!(addon.signed);
        assert_eq!(addon.min_version.as_deref(), Some("109.0"));
        assert!(addon.max_version.is_none());
        assert!(addon.validate().is_ok());
    }

    #[tokio::test]
    async fn a_signed_xpi_without_an_id_takes_it_from_the_signature() {
        let manifest =
            json!({"manifest_version": 2, "name": "__MSG_missing__", "version": "1.0"}).to_string();
        let mut der = vec![0x30, 0x00];
        der.extend_from_slice(&COMMON_NAME);
        der.extend_from_slice(&[0x0c, 7]);
        der.extend_from_slice(b"x@y.org");
        let xpi = zip::write(&[
            ("manifest.json", manifest.as_bytes()),
            ("META-INF/mozilla.rsa", der.as_slice()),
            ("META-INF/mozilla.sf", b"sf"),
            ("META-INF/manifest.mf", b"mf"),
        ]);
        let addon = inspect(&Memory(xpi)).await.unwrap();
        assert_eq!(addon.id, "x@y.org");
        assert_eq!(addon.name, "__MSG_missing__");
        assert!(addon.signed);
    }

    #[tokio::test]
    async fn files_that_are_no_add_ons_are_refused() {
        assert!(matches!(
            inspect(&Memory(b"not a zip at all, sorry".to_vec())).await,
            Err(AddonError::Invalid(_))
        ));
        let plain = zip::write(&[("readme.txt", b"hello")]);
        assert!(matches!(
            inspect(&Memory(plain)).await,
            Err(AddonError::Invalid(_))
        ));
        let broken = zip::write(&[("manifest.json", b"{ nope")]);
        assert!(matches!(
            inspect(&Memory(broken)).await,
            Err(AddonError::Invalid(_))
        ));
        let unsigned = zip::write(&[(
            "manifest.json",
            br#"{"manifest_version": 3, "name": "x", "version": "1"}"#.as_slice(),
        )]);
        let addon = inspect(&Memory(unsigned)).await.unwrap();
        assert!(!addon.signed);
        assert!(addon.validate().unwrap_err().contains("gecko.id"));
    }
}
