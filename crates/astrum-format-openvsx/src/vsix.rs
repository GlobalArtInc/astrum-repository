use std::collections::BTreeMap;

use astrum_content::zip::{self, Source, ZipError};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde_json::Value;

use crate::model::Meta;
use crate::names::{self, UNIVERSAL};

pub const MANIFEST: &str = "extension.vsixmanifest";
const PACKAGE_JSON: &str = "extension/package.json";
const MAX_MANIFEST: u64 = 4 * 1024 * 1024;
const MAX_DEPTH: usize = 32;

pub const ASSET_TYPES: &[(&str, &str)] = &[
    ("manifest", "Microsoft.VisualStudio.Code.Manifest"),
    ("readme", "Microsoft.VisualStudio.Services.Content.Details"),
    (
        "changelog",
        "Microsoft.VisualStudio.Services.Content.Changelog",
    ),
    ("license", "Microsoft.VisualStudio.Services.Content.License"),
    ("icon", "Microsoft.VisualStudio.Services.Icons.Default"),
    (
        "vsixmanifest",
        "Microsoft.VisualStudio.Services.VsixManifest",
    ),
    ("download", "Microsoft.VisualStudio.Services.VSIXPackage"),
];

pub fn kind_of(asset_type: &str) -> Option<&'static str> {
    ASSET_TYPES
        .iter()
        .find(|(_, known)| known.eq_ignore_ascii_case(asset_type))
        .map(|(kind, _)| *kind)
}

pub fn asset_type_of(kind: &str) -> Option<&'static str> {
    ASSET_TYPES
        .iter()
        .find(|(known, _)| *known == kind)
        .map(|(_, asset_type)| *asset_type)
}

#[derive(Debug, thiserror::Error)]
pub enum VsixError {
    #[error("the file is not a VS Code extension package: {0}")]
    Invalid(String),
    #[error("the extension package cannot be read: {0}")]
    Zip(String),
    #[error(transparent)]
    Store(#[from] astrum_blob::Error),
}

impl From<ZipError> for VsixError {
    fn from(error: ZipError) -> Self {
        match error {
            ZipError::Store(error) => VsixError::Store(error),
            other => VsixError::Zip(other.to_string()),
        }
    }
}

fn invalid(message: impl Into<String>) -> VsixError {
    VsixError::Invalid(message.into())
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Package {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub meta: Meta,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Default)]
struct Described {
    id: Option<String>,
    version: Option<String>,
    publisher: Option<String>,
    platform: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    tags: Option<String>,
    categories: Option<String>,
    license: Option<String>,
    icon: Option<String>,
    properties: BTreeMap<String, String>,
    assets: Vec<(String, String)>,
}

fn attribute(element: &BytesStart<'_>, wanted: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|found| {
            found
                .key
                .local_name()
                .into_inner()
                .eq_ignore_ascii_case(wanted)
        })
        .and_then(|found| {
            found
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.trim().to_owned())
        })
}

fn entity(name: &str) -> Option<String> {
    let resolved = match name {
        "lt" => '<',
        "gt" => '>',
        "amp" => '&',
        "apos" => '\'',
        "quot" => '"',
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix('x') {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code).filter(|c| *c != '\0')?
        }
    };
    Some(resolved.to_string())
}

impl Described {
    fn element(&mut self, path: &[String], element: &BytesStart<'_>) {
        let names: Vec<&str> = path.iter().map(String::as_str).collect();
        match names.as_slice() {
            [.., "Metadata", "Identity"] => {
                self.id = attribute(element, "Id");
                self.version = attribute(element, "Version");
                self.publisher = attribute(element, "Publisher");
                self.platform = attribute(element, "TargetPlatform").filter(|tp| !tp.is_empty());
            }
            [.., "Properties", "Property"] => {
                if let (Some(id), Some(value)) =
                    (attribute(element, "Id"), attribute(element, "Value"))
                {
                    self.properties.insert(id, value);
                }
            }
            [.., "Assets", "Asset"] => {
                if let (Some(kind), Some(path)) =
                    (attribute(element, "Type"), attribute(element, "Path"))
                {
                    self.assets.push((kind, path));
                }
            }
            _ => {}
        }
    }

    fn text(&mut self, path: &[String], text: String) {
        let names: Vec<&str> = path.iter().map(String::as_str).collect();
        let slot = match names.as_slice() {
            [.., "Metadata", "DisplayName"] => &mut self.display_name,
            [.., "Metadata", "Description"] => &mut self.description,
            [.., "Metadata", "Tags"] => &mut self.tags,
            [.., "Metadata", "Categories"] => &mut self.categories,
            [.., "Metadata", "License"] => &mut self.license,
            [.., "Metadata", "Icon"] => &mut self.icon,
            _ => return,
        };
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            *slot = Some(trimmed.to_owned());
        }
    }

    fn parse(bytes: &[u8]) -> Result<Described, VsixError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| invalid("extension.vsixmanifest is not UTF-8"))?
            .trim_start_matches('\u{feff}');
        let mut reader = Reader::from_str(text);
        let mut described = Described::default();
        let mut path: Vec<String> = Vec::new();
        let mut text = String::new();
        let broken = |error: quick_xml::Error| {
            invalid(format!("extension.vsixmanifest is not valid XML: {error}"))
        };
        loop {
            match reader.read_event().map_err(broken)? {
                Event::Start(start) => {
                    if path.len() >= MAX_DEPTH {
                        return Err(invalid("extension.vsixmanifest nests too deeply"));
                    }
                    path.push(start.local_name().into_inner().to_owned());
                    described.element(&path, &start);
                    text.clear();
                }
                Event::Empty(empty) => {
                    path.push(empty.local_name().into_inner().to_owned());
                    described.element(&path, &empty);
                    path.pop();
                }
                Event::Text(content) => text.push_str(&content.xml10_content()),
                Event::CData(content) => text.push_str(&content.into_inner()),
                Event::GeneralRef(reference) => {
                    let name = reference.into_inner().into_owned();
                    text.push_str(&entity(&name).ok_or_else(|| {
                        invalid(format!(
                            "extension.vsixmanifest uses the unknown entity &{name};"
                        ))
                    })?);
                }
                Event::End(_) => {
                    described.text(&path, std::mem::take(&mut text));
                    path.pop();
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(described)
    }
}

fn text_of(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        Some(Value::Object(object)) => text_of(object.get("url")),
        _ => None,
    }
}

fn texts_of(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| text_of(Some(item)))
            .collect(),
        Some(Value::String(text)) => list(text),
        _ => Vec::new(),
    }
}

fn list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn meta_of(package: &Value, described: &Described) -> Meta {
    let property = |id: &str| {
        described
            .properties
            .get(id)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    };
    let flag = |id: &str| property(id).is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let engines: BTreeMap<String, String> = package
        .get("engines")
        .and_then(Value::as_object)
        .map(|engines| {
            engines
                .iter()
                .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    let tags = match &described.tags {
        Some(tags) => list(tags),
        None => texts_of(package.get("keywords")),
    };
    let localized = match property("Microsoft.VisualStudio.Code.LocalizedLanguages") {
        Some(languages) => list(languages),
        None => package
            .pointer("/contributes/localizations")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("languageId")?.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let extension_kind = match property("Microsoft.VisualStudio.Code.ExtensionKind") {
        Some(kinds) => list(kinds),
        None => texts_of(package.get("extensionKind")),
    };
    Meta {
        display_name: text_of(package.get("displayName")).or(described.display_name.clone()),
        description: text_of(package.get("description")).or(described.description.clone()),
        categories: match &described.categories {
            Some(categories) => list(categories),
            None => texts_of(package.get("categories")),
        },
        web: flag("Microsoft.VisualStudio.Code.WebExtension")
            || tags.iter().any(|tag| tag == "__web_extension"),
        tags,
        license: text_of(package.get("license")),
        homepage: text_of(package.get("homepage")),
        repository: text_of(package.get("repository")).or_else(|| {
            property("Microsoft.VisualStudio.Services.Links.Source").map(str::to_owned)
        }),
        bugs: text_of(package.get("bugs")),
        qna: text_of(package.get("qna")),
        sponsor_link: text_of(package.get("sponsor"))
            .or_else(|| property("Microsoft.VisualStudio.Code.SponsorLink").map(str::to_owned)),
        gallery_color: text_of(package.pointer("/galleryBanner/color")).or_else(|| {
            property("Microsoft.VisualStudio.Services.Branding.Color").map(str::to_owned)
        }),
        gallery_theme: text_of(package.pointer("/galleryBanner/theme")).or_else(|| {
            property("Microsoft.VisualStudio.Services.Branding.Theme").map(str::to_owned)
        }),
        preview: package
            .get("preview")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        pre_release: flag("Microsoft.VisualStudio.Code.PreRelease"),
        engines,
        dependencies: texts_of(package.get("extensionDependencies")),
        bundled_extensions: texts_of(package.get("extensionPack")),
        extension_kind,
        localized_languages: localized,
    }
}

fn same(a: Option<&str>, b: &str) -> bool {
    a.is_none_or(|a| a.eq_ignore_ascii_case(b))
}

pub async fn read(source: &dyn Source) -> Result<Package, VsixError> {
    let entries = zip::entries(source).await?;
    let find = |name: &str| entries.iter().find(|entry| entry.name == name);
    let manifest = find(MANIFEST).ok_or_else(|| invalid("it has no extension.vsixmanifest"))?;
    let described = Described::parse(&zip::read(source, manifest, MAX_MANIFEST).await?)?;
    let declared = |kind: &str| {
        let wanted = asset_type_of(kind)?;
        described
            .assets
            .iter()
            .find(|(asset_type, _)| asset_type.eq_ignore_ascii_case(wanted))
            .map(|(_, path)| path.clone())
    };
    let package_path = declared("manifest").unwrap_or_else(|| PACKAGE_JSON.to_owned());
    let package_entry =
        find(&package_path).ok_or_else(|| invalid(format!("it has no {package_path}")))?;
    let package: Value =
        serde_json::from_slice(&zip::read(source, package_entry, MAX_MANIFEST).await?)
            .map_err(|error| invalid(format!("{package_path} is not JSON: {error}")))?;
    let field = |key: &str| package.get(key).and_then(Value::as_str).map(str::trim);
    let name = field("name")
        .filter(|name| !name.is_empty())
        .ok_or_else(|| invalid("package.json has no name"))?
        .to_owned();
    let namespace = field("publisher")
        .filter(|publisher| !publisher.is_empty())
        .ok_or_else(|| invalid("package.json has no publisher"))?
        .to_owned();
    let version = field("version")
        .filter(|version| !version.is_empty())
        .ok_or_else(|| invalid("package.json has no version"))?
        .to_owned();
    if !names::valid_name(&namespace) {
        return Err(invalid(format!(
            "the publisher {namespace} must start with a letter or digit and hold only letters, digits, - and _"
        )));
    }
    if !names::valid_name(&name) {
        return Err(invalid(format!(
            "the name {name} must start with a letter or digit and hold only letters, digits, - and _"
        )));
    }
    if !names::valid_version(&version) {
        return Err(invalid(format!(
            "the version {version} is not a semantic version"
        )));
    }
    if !same(described.id.as_deref(), &name)
        || !same(described.publisher.as_deref(), &namespace)
        || !same(described.version.as_deref(), &version)
    {
        return Err(invalid(
            "the identity in extension.vsixmanifest does not match package.json",
        ));
    }
    let platform = described
        .platform
        .clone()
        .unwrap_or_else(|| UNIVERSAL.to_owned());
    if !names::is_platform(&platform) {
        return Err(invalid(format!(
            "the target platform {platform} is not known"
        )));
    }
    let mut files = BTreeMap::new();
    files.insert("manifest".to_owned(), package_path.clone());
    files.insert("vsixmanifest".to_owned(), MANIFEST.to_owned());
    for kind in ["readme", "changelog", "license", "icon"] {
        let path = declared(kind).or_else(|| match kind {
            "license" => described.license.clone(),
            "icon" => described.icon.clone(),
            _ => None,
        });
        if let Some(path) = path.filter(|path| find(path).is_some()) {
            files.insert(kind.to_owned(), path);
        }
    }
    Ok(Package {
        meta: meta_of(&package, &described),
        namespace,
        name,
        version,
        platform,
        files,
    })
}

pub mod testing {
    pub struct Spec<'a> {
        pub namespace: &'a str,
        pub name: &'a str,
        pub version: &'a str,
        pub platform: Option<&'a str>,
        pub pre_release: bool,
        pub extra: &'a str,
    }

    impl Default for Spec<'_> {
        fn default() -> Self {
            Spec {
                namespace: "acme",
                name: "hello",
                version: "1.0.0",
                platform: None,
                pre_release: false,
                extra: "",
            }
        }
    }

    pub fn vsix(spec: &Spec<'_>) -> Vec<u8> {
        let Spec {
            namespace,
            name,
            version,
            platform,
            pre_release,
            extra,
        } = spec;
        let target = platform
            .map(|platform| format!(" TargetPlatform=\"{platform}\""))
            .unwrap_or_default();
        let manifest = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<PackageManifest Version=\"2.0.0\" xmlns=\"http://schemas.microsoft.com/developer/vsx-schema/2011\">\n  <Metadata>\n    <Identity Language=\"en-US\" Id=\"{name}\" Version=\"{version}\" Publisher=\"{namespace}\"{target}/>\n    <DisplayName>Hello &amp; welcome</DisplayName>\n    <Description xml:space=\"preserve\">Says hello.</Description>\n    <Tags>hello,greeting</Tags>\n    <Categories>Other,Linters</Categories>\n    <Properties>\n      <Property Id=\"Microsoft.VisualStudio.Code.Engine\" Value=\"^1.80.0\" />\n      <Property Id=\"Microsoft.VisualStudio.Code.ExtensionDependencies\" Value=\"acme.base\" />\n      <Property Id=\"Microsoft.VisualStudio.Code.ExtensionKind\" Value=\"workspace\" />\n      <Property Id=\"Microsoft.VisualStudio.Code.PreRelease\" Value=\"{pre_release}\" />\n    </Properties>\n    <License>extension/LICENSE.txt</License>\n    <Icon>extension/icon.png</Icon>\n  </Metadata>\n  <Installation><InstallationTarget Id=\"Microsoft.VisualStudio.Code\"/></Installation>\n  <Assets>\n    <Asset Type=\"Microsoft.VisualStudio.Code.Manifest\" Path=\"extension/package.json\" Addressable=\"true\" />\n    <Asset Type=\"Microsoft.VisualStudio.Services.Content.Details\" Path=\"extension/README.md\" Addressable=\"true\" />\n    <Asset Type=\"Microsoft.VisualStudio.Services.Content.Changelog\" Path=\"extension/CHANGELOG.md\" Addressable=\"true\" />\n    <Asset Type=\"Microsoft.VisualStudio.Services.Content.License\" Path=\"extension/LICENSE.txt\" Addressable=\"true\" />\n    <Asset Type=\"Microsoft.VisualStudio.Services.Icons.Default\" Path=\"extension/icon.png\" Addressable=\"true\" />\n  </Assets>\n</PackageManifest>\n"
        );
        let package = format!(
            "{{\"name\": \"{name}\", \"publisher\": \"{namespace}\", \"version\": \"{version}\", \"displayName\": \"Hello {name}\", \"description\": \"The {name} extension.\", \"license\": \"MIT\", \"categories\": [\"Other\", \"Linters\"], \"engines\": {{\"vscode\": \"^1.80.0\"}}, \"extensionDependencies\": [\"acme.base\"], \"repository\": {{\"type\": \"git\", \"url\": \"https://example.org/{name}.git\"}}{extra}}}"
        );
        let readme = format!("# {name} {version}\n");
        astrum_content::zip::write(&[
            ("extension.vsixmanifest", manifest.as_bytes()),
            ("[Content_Types].xml", b"<Types/>"),
            ("extension/package.json", package.as_bytes()),
            ("extension/README.md", readme.as_bytes()),
            ("extension/CHANGELOG.md", b"# Changes\n"),
            ("extension/LICENSE.txt", b"MIT\n"),
            ("extension/icon.png", b"\x89PNG\r\n\x1a\nicon"),
            ("extension/out/main.js", b"exports.activate = () => {};\n"),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{Spec, vsix};
    use super::*;
    use astrum_content::zip::Memory;

    #[tokio::test]
    async fn packages_describe_themselves() {
        let package = read(&Memory(vsix(&Spec {
            platform: Some("linux-x64"),
            pre_release: true,
            ..Spec::default()
        })))
        .await
        .unwrap();
        assert_eq!(package.namespace, "acme");
        assert_eq!(package.name, "hello");
        assert_eq!(package.version, "1.0.0");
        assert_eq!(package.platform, "linux-x64");
        assert!(package.meta.pre_release);
        assert_eq!(package.meta.display_name.as_deref(), Some("Hello hello"));
        assert_eq!(package.meta.tags, ["hello", "greeting"]);
        assert_eq!(package.meta.categories, ["Other", "Linters"]);
        assert_eq!(package.meta.engines["vscode"], "^1.80.0");
        assert_eq!(package.meta.dependencies, ["acme.base"]);
        assert_eq!(package.meta.extension_kind, ["workspace"]);
        assert_eq!(
            package.meta.repository.as_deref(),
            Some("https://example.org/hello.git")
        );
        assert_eq!(package.files["readme"], "extension/README.md");
        assert_eq!(package.files["icon"], "extension/icon.png");
        assert_eq!(package.files["manifest"], "extension/package.json");
        let universal = read(&Memory(vsix(&Spec::default()))).await.unwrap();
        assert_eq!(universal.platform, "universal");
        assert!(!universal.meta.pre_release);
    }

    #[tokio::test]
    async fn broken_packages_are_refused() {
        let plain = astrum_content::zip::write(&[("extension/package.json", b"{}")]);
        let error = read(&Memory(plain)).await.unwrap_err().to_string();
        assert!(error.contains("extension.vsixmanifest"), "{error}");
        let bad_version = vsix(&Spec {
            version: "1.0",
            ..Spec::default()
        });
        let error = read(&Memory(bad_version)).await.unwrap_err().to_string();
        assert!(error.contains("semantic version"), "{error}");
        let bad_platform = vsix(&Spec {
            platform: Some("mars-x64"),
            ..Spec::default()
        });
        let error = read(&Memory(bad_platform)).await.unwrap_err().to_string();
        assert!(error.contains("mars-x64"), "{error}");
        assert!(
            read(&Memory(b"not a zip at all, really".to_vec()))
                .await
                .is_err()
        );
    }
}
