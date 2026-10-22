use astrum_content::zip::{self, Memory, Source, ZipError};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde::{Deserialize, Serialize};

use crate::markup::{attribute, entity};

pub const DESCRIPTOR: &str = "META-INF/plugin.xml";
const MAX_DESCRIPTOR: u64 = 4 * 1024 * 1024;
const MAX_JAR: u64 = 256 * 1024 * 1024;
const MAX_JARS: usize = 256;
const MAX_UNPACKED: u64 = 1024 * 1024 * 1024;
const MAX_DEPTH: usize = 32;
const MAX_ID: usize = 255;
const MAX_VERSION: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Store(#[from] astrum_blob::Error),
}

fn invalid(message: impl Into<String>) -> PluginError {
    PluginError::Invalid(message.into())
}

fn archive_error(error: ZipError) -> PluginError {
    match error {
        ZipError::Store(error) => PluginError::Store(error),
        other => invalid(format!(
            "the file is not a plugin archive (a zip or jar): {other}"
        )),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Packaging {
    Jar,
    Zip,
}

impl Packaging {
    pub fn extension(self) -> &'static str {
        match self {
            Packaging::Jar => "jar",
            Packaging::Zip => "zip",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Vendor {
    pub name: String,
    pub email: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub vendor: Vendor,
    pub since_build: Option<String>,
    pub until_build: Option<String>,
    pub description: String,
    pub change_notes: String,
    pub depends: Vec<String>,
    pub category: String,
}

pub fn valid_segment(text: &str, limit: usize) -> bool {
    !text.is_empty()
        && text.len() <= limit
        && text == text.trim()
        && text != "."
        && text != ".."
        && !text
            .chars()
            .any(|c| c.is_control() || matches!(c, '/' | '\\' | '?' | '#' | '%' | '"'))
}

pub fn valid_id(id: &str) -> bool {
    valid_segment(id, MAX_ID)
}

pub fn valid_version(version: &str) -> bool {
    valid_segment(version, MAX_VERSION)
}

pub fn artifact_path(id: &str, version: &str, extension: &str) -> String {
    format!("/{id}/{version}/{id}-{version}.{extension}")
}

impl Plugin {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("plugin.xml has neither an <id> nor a <name>".to_owned());
        }
        if !valid_id(&self.id) {
            return Err(format!(
                "the plugin id {:?} of plugin.xml cannot be used in a repository path",
                self.id
            ));
        }
        if self.version.is_empty() {
            return Err("plugin.xml has no <version>".to_owned());
        }
        if !valid_version(&self.version) {
            return Err(format!(
                "the plugin version {:?} of plugin.xml cannot be used in a repository path",
                self.version
            ));
        }
        Ok(())
    }

    pub fn path(&self, packaging: Packaging) -> String {
        artifact_path(&self.id, &self.version, packaging.extension())
    }
}

#[derive(Debug)]
pub struct Found {
    pub plugin: Plugin,
    pub packaging: Packaging,
}

fn element(plugin: &mut Plugin, path: &[String], start: &BytesStart<'_>, optional: &mut bool) {
    let names: Vec<&str> = path.iter().map(String::as_str).collect();
    match names.as_slice() {
        ["idea-plugin", "idea-version"] => {
            plugin.since_build = attribute(start, "since-build").filter(|value| !value.is_empty());
            plugin.until_build = attribute(start, "until-build").filter(|value| !value.is_empty());
        }
        ["idea-plugin", "vendor"] => {
            plugin.vendor.email = attribute(start, "email").unwrap_or_default();
            plugin.vendor.url = attribute(start, "url").unwrap_or_default();
        }
        ["idea-plugin", "depends"] => {
            *optional = attribute(start, "optional").is_some_and(|value| value == "true");
        }
        ["idea-plugin", "dependencies", "plugin"] => {
            if attribute(start, "optional").is_none_or(|value| value != "true")
                && let Some(id) = attribute(start, "id").filter(|id| !id.is_empty())
            {
                plugin.depends.push(id);
            }
        }
        ["idea-plugin", "dependencies", "module"] => {
            if let Some(name) = attribute(start, "name").filter(|name| !name.is_empty()) {
                plugin.depends.push(name);
            }
        }
        _ => {}
    }
}

fn finished(plugin: &mut Plugin, path: &[String], text: &str, optional: bool) {
    let names: Vec<&str> = path.iter().map(String::as_str).collect();
    let text = text.trim();
    match names.as_slice() {
        ["idea-plugin", "id"] => plugin.id = text.to_owned(),
        ["idea-plugin", "name"] => plugin.name = text.to_owned(),
        ["idea-plugin", "version"] => plugin.version = text.to_owned(),
        ["idea-plugin", "vendor"] => plugin.vendor.name = text.to_owned(),
        ["idea-plugin", "description"] => plugin.description = text.to_owned(),
        ["idea-plugin", "change-notes"] => plugin.change_notes = text.to_owned(),
        ["idea-plugin", "category"] => plugin.category = text.to_owned(),
        ["idea-plugin", "depends"] if !optional && !text.is_empty() => {
            plugin.depends.push(text.to_owned());
        }
        _ => {}
    }
}

pub fn parse(bytes: &[u8]) -> Result<Plugin, PluginError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid("META-INF/plugin.xml is not UTF-8"))?
        .trim_start_matches('\u{feff}');
    let mut reader = Reader::from_str(text);
    let mut plugin = Plugin::default();
    let mut path: Vec<String> = Vec::new();
    let mut buffer = String::new();
    let mut optional = false;
    let mut rooted = false;
    let broken =
        |error: quick_xml::Error| invalid(format!("META-INF/plugin.xml is not valid XML: {error}"));
    loop {
        match reader.read_event().map_err(broken)? {
            Event::Start(start) => {
                if path.len() >= MAX_DEPTH {
                    return Err(invalid("META-INF/plugin.xml nests too deeply"));
                }
                path.push(start.local_name().into_inner().to_owned());
                if path.len() == 1 {
                    rooted = path[0] == "idea-plugin";
                }
                if path.len() == 2 {
                    buffer.clear();
                }
                element(&mut plugin, &path, &start, &mut optional);
            }
            Event::Empty(empty) => {
                path.push(empty.local_name().into_inner().to_owned());
                if path.len() == 1 {
                    rooted = path[0] == "idea-plugin";
                }
                element(&mut plugin, &path, &empty, &mut optional);
                path.pop();
            }
            Event::Text(content) => buffer.push_str(&content.xml10_content()),
            Event::CData(content) => buffer.push_str(&content.into_inner()),
            Event::GeneralRef(reference) => {
                let name = reference.into_inner().into_owned();
                buffer.push_str(&entity(&name).ok_or_else(|| {
                    invalid(format!(
                        "META-INF/plugin.xml uses the unknown entity &{name};"
                    ))
                })?);
            }
            Event::End(_) => {
                if path.len() <= 2 {
                    finished(&mut plugin, &path, &std::mem::take(&mut buffer), optional);
                }
                path.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !rooted {
        return Err(invalid("META-INF/plugin.xml has no <idea-plugin> element"));
    }
    if plugin.id.is_empty() {
        plugin.id = plugin.name.clone();
    }
    plugin.depends.dedup();
    Ok(plugin)
}

fn is_library(name: &str) -> bool {
    name.ends_with(".jar") && name.rsplit('/').nth(1) == Some("lib")
}

pub async fn inspect(source: &dyn Source) -> Result<Found, PluginError> {
    let entries = zip::entries(source).await.map_err(archive_error)?;
    if let Some(entry) = entries.iter().find(|entry| entry.name == DESCRIPTOR) {
        let bytes = zip::read(source, entry, MAX_DESCRIPTOR)
            .await
            .map_err(archive_error)?;
        return Ok(Found {
            plugin: parse(&bytes)?,
            packaging: Packaging::Jar,
        });
    }
    let mut libraries: Vec<&zip::Entry> = entries
        .iter()
        .filter(|entry| !entry.is_directory() && is_library(&entry.name))
        .collect();
    libraries.sort_by_key(|entry| entry.size);
    let mut unpacked = 0u64;
    for entry in libraries.into_iter().take(MAX_JARS) {
        if entry.size > MAX_JAR {
            continue;
        }
        unpacked = unpacked.saturating_add(entry.size);
        if unpacked > MAX_UNPACKED {
            return Err(invalid(
                "the archive holds too much to look for META-INF/plugin.xml in its jars",
            ));
        }
        let bytes = zip::read(source, entry, MAX_JAR)
            .await
            .map_err(archive_error)?;
        let inner = Memory(bytes);
        let Ok(inner_entries) = zip::entries(&inner).await else {
            continue;
        };
        if let Some(descriptor) = inner_entries.iter().find(|inner| inner.name == DESCRIPTOR) {
            let xml = zip::read(&inner, descriptor, MAX_DESCRIPTOR)
                .await
                .map_err(archive_error)?;
            return Ok(Found {
                plugin: parse(&xml)?,
                packaging: Packaging::Zip,
            });
        }
    }
    Err(invalid(
        "the archive has no META-INF/plugin.xml, neither at its root nor in a jar under lib/",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const XCODE: &str = r#"<idea-plugin>
  <name>Xcode Color Scheme</name>
  <id>com.intellij.plugins.xcode.colorscheme</id>
  <version>243.21565.122</version>
  <idea-version since-build="243.21565" until-build="243.*" />
  <vendor email="support@example.com" url="https://example.com/">Example</vendor>
  <category>UI</category>
  <depends>com.intellij.modules.lang</depends>
  <depends optional="true" config-file="x.xml">other.plugin</depends>
  <description><![CDATA[Xcode editor <b>color</b> scheme.]]></description>
  <change-notes>Fixed &amp; improved</change-notes>
  <dependencies>
    <plugin id="org.example.base"/>
    <plugin id="org.example.soft" optional="true"/>
    <module name="intellij.platform.x"/>
  </dependencies>
  <extensions defaultExtensionNs="com.intellij"><x id="not-the-plugin"/></extensions>
</idea-plugin>"#;

    #[test]
    fn a_descriptor_is_read() {
        let plugin = parse(XCODE.as_bytes()).unwrap();
        assert_eq!(plugin.id, "com.intellij.plugins.xcode.colorscheme");
        assert_eq!(plugin.name, "Xcode Color Scheme");
        assert_eq!(plugin.version, "243.21565.122");
        assert_eq!(plugin.since_build.as_deref(), Some("243.21565"));
        assert_eq!(plugin.until_build.as_deref(), Some("243.*"));
        assert_eq!(plugin.vendor.name, "Example");
        assert_eq!(plugin.vendor.email, "support@example.com");
        assert_eq!(plugin.vendor.url, "https://example.com/");
        assert_eq!(plugin.category, "UI");
        assert_eq!(plugin.description, "Xcode editor <b>color</b> scheme.");
        assert_eq!(plugin.change_notes, "Fixed & improved");
        assert_eq!(
            plugin.depends,
            [
                "com.intellij.modules.lang",
                "org.example.base",
                "intellij.platform.x"
            ]
        );
        assert!(plugin.validate().is_ok());
    }

    #[test]
    fn the_name_stands_in_for_a_missing_id() {
        let plugin =
            parse(b"<idea-plugin><name>Lombok Plugin</name><version>1.2</version></idea-plugin>")
                .unwrap();
        assert_eq!(plugin.id, "Lombok Plugin");
        assert!(plugin.since_build.is_none());
        assert!(plugin.validate().is_ok());
    }

    #[test]
    fn broken_descriptors_are_refused() {
        assert!(parse(b"<plugin><id>a</id></plugin>").is_err());
        assert!(parse(b"<idea-plugin><id>a</idea-plugin>").is_err());
        assert!(parse(&[0xff, 0xfe, 0x00]).is_err());
        let no_version = parse(b"<idea-plugin><id>a</id></idea-plugin>").unwrap();
        assert!(no_version.validate().is_err());
        let slash = parse(b"<idea-plugin><id>a/b</id><version>1</version></idea-plugin>").unwrap();
        assert!(slash.validate().is_err());
        let dots = parse(b"<idea-plugin><id>..</id><version>1</version></idea-plugin>").unwrap();
        assert!(dots.validate().is_err());
        let percent =
            parse(b"<idea-plugin><id>a</id><version>1%2f</version></idea-plugin>").unwrap();
        assert!(percent.validate().is_err());
    }

    #[test]
    fn paths_are_built_from_id_and_version() {
        assert_eq!(artifact_path("a.b", "1.0", "zip"), "/a.b/1.0/a.b-1.0.zip");
        assert!(valid_id("PlantUML integration"));
        assert!(!valid_id(" lead"));
        assert!(!valid_id(""));
    }

    #[tokio::test]
    async fn a_bare_jar_and_a_bundle_are_both_found() {
        let jar = zip::write(&[(DESCRIPTOR, XCODE.as_bytes()), ("colors/x.xml", b"<x/>")]);
        let found = inspect(&Memory(jar.clone())).await.unwrap();
        assert_eq!(found.packaging, Packaging::Jar);
        assert_eq!(found.plugin.id, "com.intellij.plugins.xcode.colorscheme");

        let library = zip::write(&[("org/example/Thing.class", b"\xca\xfe")]);
        let bundle = zip::write(&[
            ("xcode/lib/zzz-library.jar", library.as_slice()),
            ("xcode/lib/xcode.jar", jar.as_slice()),
            ("xcode/readme.txt", b"hello"),
        ]);
        let found = inspect(&Memory(bundle)).await.unwrap();
        assert_eq!(found.packaging, Packaging::Zip);
        assert_eq!(found.plugin.version, "243.21565.122");
    }

    #[tokio::test]
    async fn archives_without_a_descriptor_are_refused() {
        let plain = zip::write(&[("readme.txt", b"hello")]);
        assert!(matches!(
            inspect(&Memory(plain)).await,
            Err(PluginError::Invalid(_))
        ));
        let library = zip::write(&[("a/lib/x.jar", zip::write(&[("x", b"y")]).as_slice())]);
        assert!(inspect(&Memory(library)).await.is_err());
        assert!(matches!(
            inspect(&Memory(b"not a zip at all, just text".to_vec())).await,
            Err(PluginError::Invalid(_))
        ));
    }
}
