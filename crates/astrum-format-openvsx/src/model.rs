use std::cmp::Ordering;
use std::collections::BTreeMap;

use astrum_content::{Asset, Ctx};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::names::{self, UNIVERSAL, WEB};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bugs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qna: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_theme: Option<String>,
    pub preview: bool,
    pub pre_release: bool,
    pub web: bool,
    pub engines: BTreeMap<String, String>,
    pub dependencies: Vec<String>,
    pub bundled_extensions: Vec<String>,
    pub extension_kind: Vec<String>,
    pub localized_languages: Vec<String>,
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn texts(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|item| text(Some(item))).collect())
        .unwrap_or_default()
}

fn references(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    Some(names::id(
                        item.get("namespace")?.as_str()?,
                        item.get("extension")?.as_str()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

impl Meta {
    pub fn from_document(document: &Value) -> Meta {
        Meta {
            display_name: text(document.get("displayName")),
            description: text(document.get("description")),
            categories: texts(document.get("categories")),
            tags: texts(document.get("tags")),
            license: text(document.get("license")),
            homepage: text(document.get("homepage")),
            repository: text(document.get("repository")),
            bugs: text(document.get("bugs")),
            qna: text(document.get("qna")),
            sponsor_link: text(document.get("sponsorLink")),
            gallery_color: text(document.get("galleryColor")),
            gallery_theme: text(document.get("galleryTheme")),
            preview: document
                .get("preview")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            pre_release: document
                .get("preRelease")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            web: texts(document.get("tags"))
                .iter()
                .any(|tag| tag == "__web_extension"),
            engines: document
                .get("engines")
                .and_then(Value::as_object)
                .map(|engines| {
                    engines
                        .iter()
                        .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())))
                        .collect()
                })
                .unwrap_or_default(),
            dependencies: references(document.get("dependencies")),
            bundled_extensions: references(document.get("bundledExtensions")),
            extension_kind: texts(document.get("extensionKind")),
            localized_languages: texts(document.get("localizedLanguages")),
        }
    }

    pub fn is_pre_release(&self, version: &str) -> bool {
        self.pre_release || names::has_prerelease_tag(version)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Stored {
    pub target_platform: String,
    pub files: BTreeMap<String, String>,
    pub meta: Meta,
}

#[derive(Clone, Debug)]
pub struct Release {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub meta: Meta,
    pub files: BTreeMap<String, String>,
    pub published: DateTime<Utc>,
    pub asset: Asset,
}

impl Release {
    pub fn key(&self) -> String {
        names::key(&self.namespace, &self.name)
    }

    pub fn vsix_name(&self) -> String {
        names::vsix_name(&self.namespace, &self.name, &self.version, &self.platform)
    }

    pub fn sha256_name(&self) -> String {
        names::sha256_name(&self.namespace, &self.name, &self.version, &self.platform)
    }

    pub fn file_name(&self, kind: &str) -> Option<String> {
        match kind {
            "download" => Some(self.vsix_name()),
            "sha256" => Some(self.sha256_name()),
            _ => self
                .files
                .get(kind)
                .map(|path| path.rsplit('/').next().unwrap_or(path).to_owned()),
        }
    }

    pub fn entry_for(&self, file: &str) -> Option<&str> {
        self.files
            .values()
            .find(|path| path.rsplit('/').next().unwrap_or(path) == file)
            .map(String::as_str)
    }

    pub fn is_pre_release(&self) -> bool {
        self.meta.is_pre_release(&self.version)
    }
}

pub fn time(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Micros, true)
}

pub fn order(a: &Release, b: &Release) -> Ordering {
    names::compare_versions(&b.version, &a.version)
        .then_with(|| names::platform_rank(&a.platform).cmp(&names::platform_rank(&b.platform)))
}

pub struct Ext<'a> {
    pub namespace: &'a str,
    pub name: &'a str,
    pub releases: Vec<&'a Release>,
}

pub fn extensions(releases: &[Release]) -> Vec<Ext<'_>> {
    let mut grouped: BTreeMap<String, Vec<&Release>> = BTreeMap::new();
    for release in releases {
        grouped.entry(release.key()).or_default().push(release);
    }
    grouped
        .into_values()
        .map(|mut releases| {
            releases.sort_by(|a, b| order(a, b));
            Ext {
                namespace: &releases[0].namespace,
                name: &releases[0].name,
                releases,
            }
        })
        .collect()
}

fn fits(release: &Release, platform: Option<&str>) -> bool {
    match platform {
        None => true,
        Some(platform) => release.platform == platform || release.platform == UNIVERSAL,
    }
}

fn pick<'a>(candidates: Vec<&'a Release>, platform: Option<&str>) -> Option<&'a Release> {
    candidates.into_iter().min_by_key(|release| match platform {
        Some(platform) if release.platform == platform => 0,
        _ => match release.platform.as_str() {
            UNIVERSAL => 1,
            WEB => 2,
            other => 3 + names::platform_rank(other),
        },
    })
}

impl<'a> Ext<'a> {
    pub fn key(&self) -> String {
        names::key(self.namespace, self.name)
    }

    pub fn versions(&self) -> Vec<&'a str> {
        let mut versions: Vec<&str> = Vec::new();
        for release in &self.releases {
            if !versions.contains(&release.version.as_str()) {
                versions.push(&release.version);
            }
        }
        versions
    }

    pub fn of_version(&self, version: &str) -> Vec<&'a Release> {
        self.releases
            .iter()
            .copied()
            .filter(|release| release.version == version)
            .collect()
    }

    pub fn version(&self, version: &str, platform: Option<&str>) -> Option<&'a Release> {
        let candidates: Vec<&Release> = self
            .of_version(version)
            .into_iter()
            .filter(|release| fits(release, platform))
            .collect();
        pick(candidates, platform)
    }

    fn newest(
        &self,
        platform: Option<&str>,
        keep: impl Fn(&Release) -> bool,
    ) -> Option<&'a Release> {
        let version = self
            .releases
            .iter()
            .find(|release| fits(release, platform) && keep(release))?
            .version
            .clone();
        self.version(&version, platform)
    }

    pub fn latest(&self, platform: Option<&str>) -> Option<&'a Release> {
        self.newest(platform, |release| !release.is_pre_release())
            .or_else(|| self.newest(platform, |_| true))
    }

    pub fn pre_release(&self, platform: Option<&str>) -> Option<&'a Release> {
        self.newest(platform, Release::is_pre_release)
    }

    pub fn aliased(&self, alias: &str, platform: Option<&str>) -> Option<&'a Release> {
        match alias {
            "latest" => self.latest(platform),
            "pre-release" => self.pre_release(platform),
            version => self.version(version, platform),
        }
    }

    pub fn downloads(&self) -> i64 {
        self.releases
            .iter()
            .map(|release| release.asset.download_count)
            .sum()
    }

    pub fn first_published(&self) -> DateTime<Utc> {
        self.releases
            .iter()
            .map(|release| release.published)
            .min()
            .unwrap_or_else(Utc::now)
    }

    pub fn last_published(&self) -> DateTime<Utc> {
        self.releases
            .iter()
            .map(|release| release.published)
            .max()
            .unwrap_or_else(Utc::now)
    }

    pub fn display_name(&self) -> String {
        self.releases
            .first()
            .and_then(|release| release.meta.display_name.clone())
            .unwrap_or_else(|| self.name.to_owned())
    }
}

#[derive(Clone, Debug)]
pub struct Urls {
    pub base: String,
}

impl Urls {
    pub fn of(ctx: &Ctx) -> Urls {
        Urls {
            base: ctx.repository_url(),
        }
    }

    pub fn namespace(&self, namespace: &str) -> String {
        format!("{}/api/{namespace}", self.base)
    }

    pub fn extension(&self, namespace: &str, name: &str) -> String {
        format!("{}/api/{namespace}/{name}", self.base)
    }

    pub fn under(&self, namespace: &str, name: &str, rest: &str) -> String {
        format!("{}/{rest}", self.extension(namespace, name))
    }

    pub fn version(&self, release: &Release) -> String {
        format!(
            "{}/{}/{}",
            self.extension(&release.namespace, &release.name),
            release.platform,
            release.version
        )
    }

    pub fn file(&self, release: &Release, file: &str) -> String {
        format!(
            "{}{}",
            self.base,
            names::file_path(
                &release.namespace,
                &release.name,
                &release.platform,
                &release.version,
                &names::encode(file)
            )
        )
    }

    pub fn kind(&self, release: &Release, kind: &str) -> Option<String> {
        release
            .file_name(kind)
            .map(|file| self.file(release, &file))
    }

    pub fn asset(&self, namespace: &str, name: &str, version: &str) -> String {
        format!("{}/vscode/asset/{namespace}/{name}/{version}", self.base)
    }

    pub fn unpkg(&self, namespace: &str, name: &str, version: &str, path: &str) -> String {
        format!(
            "{}/vscode/unpkg/{namespace}/{name}/{version}/{}",
            self.base,
            names::encode_path(path)
        )
    }
}

fn files(urls: &Urls, release: &Release, kinds: &[&str]) -> Value {
    let mut files = Map::new();
    for kind in kinds {
        if let Some(url) = urls.kind(release, kind) {
            files.insert((*kind).to_owned(), json!(url));
        }
    }
    Value::Object(files)
}

const ALL_FILES: &[&str] = &[
    "download",
    "manifest",
    "readme",
    "changelog",
    "license",
    "icon",
    "vsixmanifest",
    "sha256",
];

fn reference(urls: &Urls, id: &str) -> Option<Value> {
    let (namespace, extension) = names::split_id(id)?;
    Some(json!({
        "namespace": namespace,
        "extension": extension,
        "url": urls.extension(namespace, extension),
    }))
}

fn insert_text(object: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        object.insert(key.to_owned(), json!(value));
    }
}

pub fn extension_document(urls: &Urls, ext: &Ext<'_>, release: &Release) -> Value {
    let meta = &release.meta;
    let mut all_versions = Map::new();
    if ext.latest(None).is_some() {
        all_versions.insert(
            "latest".to_owned(),
            json!(urls.under(ext.namespace, ext.name, "latest")),
        );
    }
    if ext.pre_release(None).is_some() {
        all_versions.insert(
            "pre-release".to_owned(),
            json!(urls.under(ext.namespace, ext.name, "pre-release")),
        );
    }
    for version in ext.versions() {
        all_versions.insert(
            version.to_owned(),
            json!(urls.under(ext.namespace, ext.name, version)),
        );
    }
    let mut aliases = Vec::new();
    if ext
        .latest(None)
        .is_some_and(|latest| latest.version == release.version)
    {
        aliases.push("latest");
    }
    if ext
        .pre_release(None)
        .is_some_and(|pre| pre.version == release.version)
    {
        aliases.push("pre-release");
    }
    let mut downloads = Map::new();
    for sibling in ext.of_version(&release.version) {
        if let Some(url) = urls.kind(sibling, "download") {
            downloads.insert(sibling.platform.clone(), json!(url));
        }
    }
    let mut object = Map::new();
    object.insert(
        "namespaceUrl".into(),
        json!(urls.namespace(&release.namespace)),
    );
    object.insert(
        "reviewsUrl".into(),
        json!(urls.under(&release.namespace, &release.name, "reviews")),
    );
    object.insert("files".into(), files(urls, release, ALL_FILES));
    object.insert("name".into(), json!(release.name));
    object.insert("namespace".into(), json!(release.namespace));
    object.insert("targetPlatform".into(), json!(release.platform));
    object.insert("version".into(), json!(release.version));
    object.insert("preRelease".into(), json!(release.is_pre_release()));
    object.insert(
        "publishedBy".into(),
        json!({"loginName": release.namespace}),
    );
    object.insert("publishedWithTrustedPublishing".into(), json!(false));
    object.insert("verified".into(), json!(true));
    object.insert("unrelatedPublisher".into(), json!(false));
    object.insert("allVersions".into(), Value::Object(all_versions));
    object.insert(
        "allVersionsUrl".into(),
        json!(urls.under(&release.namespace, &release.name, "versions")),
    );
    object.insert("downloadCount".into(), json!(ext.downloads()));
    object.insert("reviewCount".into(), json!(0));
    object.insert("versionAlias".into(), json!(aliases));
    object.insert("timestamp".into(), json!(time(release.published)));
    object.insert("preview".into(), json!(meta.preview));
    insert_text(&mut object, "displayName", &meta.display_name);
    object.insert("namespaceDisplayName".into(), json!(release.namespace));
    insert_text(&mut object, "description", &meta.description);
    object.insert("engines".into(), json!(meta.engines));
    object.insert("categories".into(), json!(meta.categories));
    object.insert("extensionKind".into(), json!(meta.extension_kind));
    object.insert("tags".into(), json!(meta.tags));
    insert_text(&mut object, "license", &meta.license);
    insert_text(&mut object, "homepage", &meta.homepage);
    insert_text(&mut object, "repository", &meta.repository);
    insert_text(&mut object, "sponsorLink", &meta.sponsor_link);
    insert_text(&mut object, "bugs", &meta.bugs);
    insert_text(&mut object, "qna", &meta.qna);
    insert_text(&mut object, "galleryColor", &meta.gallery_color);
    insert_text(&mut object, "galleryTheme", &meta.gallery_theme);
    object.insert("localizedLanguages".into(), json!(meta.localized_languages));
    object.insert("badges".into(), json!([]));
    object.insert(
        "dependencies".into(),
        Value::Array(
            meta.dependencies
                .iter()
                .filter_map(|id| reference(urls, id))
                .collect(),
        ),
    );
    object.insert(
        "bundledExtensions".into(),
        Value::Array(
            meta.bundled_extensions
                .iter()
                .filter_map(|id| reference(urls, id))
                .collect(),
        ),
    );
    object.insert("downloads".into(), Value::Object(downloads));
    object.insert("url".into(), json!(urls.version(release)));
    object.insert("deprecated".into(), json!(false));
    object.insert("downloadable".into(), json!(true));
    Value::Object(object)
}

pub fn version_reference(urls: &Urls, release: &Release) -> Value {
    json!({
        "url": urls.version(release),
        "files": files(urls, release, &["download", "manifest", "readme", "changelog", "license", "icon", "sha256"]),
        "version": release.version,
        "targetPlatform": release.platform,
        "engines": release.meta.engines,
    })
}

pub fn search_entry(urls: &Urls, ext: &Ext<'_>, release: &Release, all_versions: bool) -> Value {
    let mut object = Map::new();
    object.insert(
        "url".into(),
        json!(urls.extension(&release.namespace, &release.name)),
    );
    object.insert(
        "files".into(),
        files(
            urls,
            release,
            &[
                "download",
                "manifest",
                "readme",
                "changelog",
                "license",
                "icon",
                "sha256",
            ],
        ),
    );
    object.insert("name".into(), json!(release.name));
    object.insert("namespace".into(), json!(release.namespace));
    object.insert("version".into(), json!(release.version));
    object.insert("timestamp".into(), json!(time(release.published)));
    object.insert("verified".into(), json!(true));
    object.insert("downloadCount".into(), json!(ext.downloads()));
    object.insert("reviewCount".into(), json!(0));
    insert_text(&mut object, "displayName", &release.meta.display_name);
    insert_text(&mut object, "description", &release.meta.description);
    object.insert("deprecated".into(), json!(false));
    if all_versions {
        object.insert(
            "allVersions".into(),
            Value::Array(
                ext.releases
                    .iter()
                    .map(|release| version_reference(urls, release))
                    .collect(),
            ),
        );
    }
    object.insert(
        "allVersionsUrl".into(),
        json!(urls.under(&release.namespace, &release.name, "version-references")),
    );
    Value::Object(object)
}

pub fn rewrite(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::String(text) => {
            if let Some(rest) = text.strip_prefix(from)
                && (rest.starts_with("/api/") || rest.starts_with("/vscode/"))
            {
                *text = format!("{to}{rest}");
            }
        }
        Value::Array(items) => {
            for item in items {
                rewrite(item, from, to);
            }
        }
        Value::Object(object) => {
            for item in object.values_mut() {
                rewrite(item, from, to);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, platform: &str, pre_release: bool) -> Release {
        Release {
            namespace: "acme".to_owned(),
            name: "hello".to_owned(),
            version: version.to_owned(),
            platform: platform.to_owned(),
            meta: Meta {
                pre_release,
                engines: BTreeMap::from([("vscode".to_owned(), "^1.80.0".to_owned())]),
                dependencies: vec!["acme.base".to_owned()],
                ..Meta::default()
            },
            files: BTreeMap::from([
                ("manifest".to_owned(), "extension/package.json".to_owned()),
                ("readme".to_owned(), "extension/README.md".to_owned()),
            ]),
            published: Utc::now(),
            asset: asset(),
        }
    }

    fn asset() -> Asset {
        let now = Utc::now().fixed_offset();
        Asset {
            id: 1,
            repository: "vsx".to_owned(),
            format: "openvsx".to_owned(),
            path: "/x".to_owned(),
            kind: "vsix".to_owned(),
            component_id: None,
            blob_store: "default".to_owned(),
            blob_id: uuid::Uuid::nil(),
            size: 1,
            content_type: "application/vsix".to_owned(),
            sha1: String::new(),
            sha256: String::new(),
            sha512: String::new(),
            md5: String::new(),
            attributes: json!({}),
            uploader: String::new(),
            uploader_ip: String::new(),
            created_at: now,
            blob_updated_at: now,
            last_downloaded_at: None,
            download_count: 2,
            last_verified_at: None,
            remote_etag: None,
            remote_last_modified: None,
        }
    }

    #[test]
    fn the_latest_release_prefers_releases_and_the_universal_build() {
        let releases = vec![
            release("1.0.0", "universal", false),
            release("1.1.0", "linux-x64", false),
            release("1.1.0", "universal", false),
            release("1.2.0", "darwin-arm64", true),
        ];
        let all = extensions(&releases);
        assert_eq!(all.len(), 1);
        let ext = &all[0];
        assert_eq!(ext.versions(), ["1.2.0", "1.1.0", "1.0.0"]);
        let latest = ext.latest(None).unwrap();
        assert_eq!(
            (latest.version.as_str(), latest.platform.as_str()),
            ("1.1.0", "universal")
        );
        let linux = ext.latest(Some("linux-x64")).unwrap();
        assert_eq!(linux.platform, "linux-x64");
        let mac = ext.latest(Some("darwin-arm64")).unwrap();
        assert_eq!(
            (mac.version.as_str(), mac.platform.as_str()),
            ("1.1.0", "universal")
        );
        let pre = ext.pre_release(Some("darwin-arm64")).unwrap();
        assert_eq!(pre.version, "1.2.0");
        assert!(ext.aliased("1.2.0", Some("linux-x64")).is_none());
        let urls = Urls {
            base: "https://repo.example/repository/vsx".to_owned(),
        };
        let document = extension_document(&urls, ext, latest);
        assert_eq!(document["versionAlias"], json!(["latest"]));
        assert_eq!(
            document["files"]["download"],
            "https://repo.example/repository/vsx/api/acme/hello/1.1.0/file/acme.hello-1.1.0.vsix"
        );
        assert_eq!(
            document["files"]["readme"],
            "https://repo.example/repository/vsx/api/acme/hello/1.1.0/file/README.md"
        );
        assert_eq!(
            document["downloads"]["linux-x64"],
            "https://repo.example/repository/vsx/api/acme/hello/linux-x64/1.1.0/file/acme.hello-1.1.0@linux-x64.vsix"
        );
        assert_eq!(document["dependencies"][0]["extension"], "base");
        assert_eq!(
            document["allVersions"]["pre-release"],
            "https://repo.example/repository/vsx/api/acme/hello/pre-release"
        );
    }

    #[test]
    fn remote_addresses_are_rewritten_to_the_repository() {
        let mut document = json!({
            "files": {"download": "https://open-vsx.example/api/a/b/1.0.0/file/a.b-1.0.0.vsix"},
            "page": "https://open-vsx.example/extension/a/b",
            "homepage": "https://github.com/a/b",
            "assetUri": "https://open-vsx.example/vscode/asset/a/b/1.0.0",
        });
        rewrite(
            &mut document,
            "https://open-vsx.example",
            "http://astrum/repository/vsx",
        );
        assert_eq!(
            document["files"]["download"],
            "http://astrum/repository/vsx/api/a/b/1.0.0/file/a.b-1.0.0.vsix"
        );
        assert_eq!(document["page"], "https://open-vsx.example/extension/a/b");
        assert_eq!(document["homepage"], "https://github.com/a/b");
        assert_eq!(
            document["assetUri"],
            "http://astrum/repository/vsx/vscode/asset/a/b/1.0.0"
        );
    }
}
