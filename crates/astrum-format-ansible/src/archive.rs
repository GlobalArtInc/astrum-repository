use std::collections::BTreeSet;
use std::io::{self, Read};

use serde_json::{Map, Value, json};

use crate::names::{self, Coll};

const MAX_MANIFEST: u64 = 4 * 1024 * 1024;
const MAX_RUNTIME: u64 = 1024 * 1024;
const MAX_INFLATED: u64 = 1024 * 1024 * 1024;
const SKIPPED_PLUGIN_FOLDERS: &[&str] = &["module_utils", "doc_fragments", "__pycache__"];

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("{0}")]
    Invalid(String),
    #[error("the collection archive cannot be read: {0}")]
    Io(#[from] io::Error),
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

/// What the server keeps of a collection archive: its `MANIFEST.json`, the `requires_ansible`
/// of `meta/runtime.yml` and the content Galaxy lists (plugins, roles, playbooks).
#[derive(Clone, Debug, PartialEq)]
pub struct Collection {
    pub coll: Coll,
    pub version: String,
    pub manifest: Map<String, Value>,
    pub requires_ansible: Option<String>,
    pub contents: Vec<Value>,
}

impl Collection {
    pub fn info(&self) -> &Map<String, Value> {
        self.manifest
            .get("collection_info")
            .and_then(Value::as_object)
            .expect("a checked manifest has its collection_info")
    }

    /// The component attributes: the fields of `collection_info` Galaxy shows as metadata.
    pub fn attributes(&self) -> Value {
        let info = self.info();
        let text = |key: &str| info.get(key).filter(|value| value.is_string()).cloned();
        let list = |key: &str| match info.get(key) {
            Some(Value::Array(items)) => Value::Array(
                items
                    .iter()
                    .filter(|item| item.is_string())
                    .cloned()
                    .collect(),
            ),
            Some(Value::String(one)) => json!([one]),
            _ => json!([]),
        };
        json!({
            "authors": list("authors"),
            "description": text("description").unwrap_or(json!("")),
            "license": list("license"),
            "license_file": text("license_file"),
            "tags": list("tags"),
            "dependencies": info.get("dependencies").cloned().unwrap_or_else(|| json!({})),
            "repository": text("repository"),
            "documentation": text("documentation"),
            "homepage": text("homepage"),
            "issues": text("issues"),
            "requires_ansible": self.requires_ansible,
            "contents": self.contents,
        })
    }
}

fn entry_path(path: &str) -> String {
    let mut path = path.trim_start_matches("./").to_owned();
    while path.ends_with('/') {
        path.pop();
    }
    path
}

fn content_of(path: &str, is_dir: bool, found: &mut BTreeSet<(String, String)>) {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.as_slice() {
        ["plugins", folder, file] if !is_dir && !SKIPPED_PLUGIN_FOLDERS.contains(folder) => {
            if let Some(stem) = file
                .strip_suffix(".py")
                .or_else(|| file.strip_suffix(".ps1"))
                && stem != "__init__"
            {
                let kind = match *folder {
                    "modules" => "module",
                    other => other,
                };
                found.insert((kind.to_owned(), stem.to_owned()));
            }
        }
        ["roles", role, ..] if !role.is_empty() && !role.starts_with('.') => {
            found.insert(("role".to_owned(), (*role).to_owned()));
        }
        ["playbooks", file] if !is_dir => {
            if let Some(stem) = file
                .strip_suffix(".yml")
                .or_else(|| file.strip_suffix(".yaml"))
            {
                found.insert(("playbook".to_owned(), stem.to_owned()));
            }
        }
        _ => {}
    }
}

/// Reads a collection from the gzip-compressed tar `ansible-galaxy collection build` writes and
/// checks its `MANIFEST.json`.
pub fn read(reader: impl Read) -> Result<Collection, ArchiveError> {
    let inflated = flate2::read::GzDecoder::new(reader).take(MAX_INFLATED);
    let mut archive = tar::Archive::new(inflated);
    let mut manifest = None;
    let mut runtime = None;
    let mut contents = BTreeSet::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let is_dir = entry.header().entry_type().is_dir();
        let path = match entry.path() {
            Ok(path) => entry_path(&path.to_string_lossy()),
            Err(_) => continue,
        };
        match path.as_str() {
            "MANIFEST.json" if !is_dir => {
                let mut bytes = Vec::new();
                (&mut entry).take(MAX_MANIFEST).read_to_end(&mut bytes)?;
                manifest = Some(bytes);
            }
            "meta/runtime.yml" | "meta/runtime.yaml" if !is_dir => {
                let mut bytes = Vec::new();
                (&mut entry).take(MAX_RUNTIME).read_to_end(&mut bytes)?;
                runtime = Some(bytes);
            }
            other => content_of(other, is_dir, &mut contents),
        }
    }
    let manifest =
        manifest.ok_or_else(|| invalid("the archive has no MANIFEST.json at its top"))?;
    let manifest = match serde_json::from_slice::<Value>(&manifest) {
        Ok(Value::Object(object)) => object,
        Ok(_) => return Err(invalid("MANIFEST.json is not a JSON object")),
        Err(error) => return Err(invalid(format!("MANIFEST.json is not valid JSON: {error}"))),
    };
    let (coll, version) = check(&manifest)?;
    let requires_ansible = runtime.and_then(|bytes| requires_ansible(&bytes));
    Ok(Collection {
        coll,
        version,
        manifest,
        requires_ansible,
        contents: contents
            .into_iter()
            .map(|(kind, name)| json!({"name": name, "content_type": kind, "description": null}))
            .collect(),
    })
}

fn requires_ansible(bytes: &[u8]) -> Option<String> {
    let runtime: serde_norway::Value = serde_norway::from_slice(bytes).ok()?;
    runtime
        .get("requires_ansible")
        .and_then(serde_norway::Value::as_str)
        .map(str::to_owned)
}

fn check(manifest: &Map<String, Value>) -> Result<(Coll, String), ArchiveError> {
    let info = manifest
        .get("collection_info")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("MANIFEST.json has no collection_info"))?;
    let text = |key: &str| {
        info.get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid(format!("MANIFEST.json has no {key} of the collection")))
    };
    let namespace = text("namespace")?;
    let name = text("name")?;
    let version = text("version")?;
    for (what, value) in [("namespace", namespace), ("name", name)] {
        if !names::valid_name(value) {
            return Err(invalid(format!(
                "the {what} {value} is not valid: use lower-case letters, digits and underscores, start with a letter and do not repeat underscores"
            )));
        }
    }
    if !names::valid_version(version) {
        return Err(invalid(format!(
            "the version {version} is not a semantic version"
        )));
    }
    match info.get("dependencies") {
        None | Some(Value::Null) => {}
        Some(Value::Object(dependencies)) => {
            for (dependency, wanted) in dependencies {
                if names::parse_fqcn(dependency).is_none() {
                    return Err(invalid(format!(
                        "the dependency {dependency} is not a <namespace>.<name> collection name"
                    )));
                }
                if !wanted
                    .as_str()
                    .is_some_and(|wanted| !wanted.trim().is_empty())
                {
                    return Err(invalid(format!(
                        "the dependency {dependency} needs a version range as text"
                    )));
                }
            }
        }
        Some(_) => {
            return Err(invalid(
                "the dependencies of MANIFEST.json are not an object",
            ));
        }
    }
    let coll = Coll::new(namespace, name).expect("valid names are valid in paths");
    Ok((coll, version.to_owned()))
}

#[cfg(test)]
pub(crate) mod testing {
    use std::io::Write;

    use serde_json::{Value, json};

    pub fn manifest(namespace: &str, name: &str, version: &str, dependencies: Value) -> Value {
        json!({
            "collection_info": {
                "namespace": namespace,
                "name": name,
                "version": version,
                "authors": ["Jane Doe <jane@example.org>"],
                "readme": "README.md",
                "tags": ["tools", "demo"],
                "description": "A test collection",
                "license": ["MIT"],
                "license_file": null,
                "dependencies": dependencies,
                "repository": "https://example.org/repo",
                "documentation": null,
                "homepage": null,
                "issues": null
            },
            "file_manifest_file": {"name": "FILES.json", "ftype": "file", "chksum_type": "sha256", "chksum_sha256": "0", "format": 1},
            "format": 1
        })
    }

    pub fn tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        for (path, data) in files {
            let mut header = tar::Header::new_gnu();
            if path.ends_with('/') {
                header.set_entry_type(tar::EntryType::Directory);
                header.set_size(0);
                header.set_mode(0o755);
            } else {
                header.set_size(data.len() as u64);
                header.set_mode(0o644);
            }
            header.set_cksum();
            builder.append_data(&mut header, path, *data).unwrap();
        }
        let mut encoder = builder.into_inner().unwrap();
        encoder.flush().unwrap();
        encoder.finish().unwrap()
    }

    pub fn collection(manifest: &Value) -> Vec<u8> {
        let text = manifest.to_string();
        tar_gz(&[
            ("MANIFEST.json", text.as_bytes()),
            ("FILES.json", b"{\"files\": [], \"format\": 1}"),
            ("README.md", b"# Test\n"),
            ("plugins/", b""),
            ("plugins/modules/", b""),
            ("plugins/modules/hello.py", b"print('hello')\n"),
            ("plugins/modules/__init__.py", b""),
            ("plugins/module_utils/common.py", b""),
            ("plugins/lookup/greeting.py", b""),
            ("roles/web/tasks/main.yml", b"---\n"),
            ("meta/runtime.yml", b"---\nrequires_ansible: '>=2.15.0'\n"),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;

    #[test]
    fn a_built_collection_answers_its_manifest_and_contents() {
        let archive = collection(&manifest(
            "acme",
            "tools",
            "1.2.0",
            json!({"community.general": ">=8.0.0"}),
        ));
        let found = read(archive.as_slice()).unwrap();
        assert_eq!(found.coll.to_string(), "acme.tools");
        assert_eq!(found.version, "1.2.0");
        assert_eq!(found.requires_ansible.as_deref(), Some(">=2.15.0"));
        let contents: Vec<(String, String)> = found
            .contents
            .iter()
            .map(|item| {
                (
                    item["content_type"].as_str().unwrap().to_owned(),
                    item["name"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        assert_eq!(
            contents,
            [
                ("lookup".to_owned(), "greeting".to_owned()),
                ("module".to_owned(), "hello".to_owned()),
                ("role".to_owned(), "web".to_owned()),
            ]
        );
        let attributes = found.attributes();
        assert_eq!(attributes["tags"], json!(["tools", "demo"]));
        assert_eq!(attributes["dependencies"]["community.general"], ">=8.0.0");
        assert_eq!(attributes["documentation"], Value::Null);
    }

    #[test]
    fn broken_manifests_are_refused() {
        let cases = [
            (tar_gz(&[("README.md", b"x")]), "no MANIFEST.json"),
            (tar_gz(&[("MANIFEST.json", b"[1]")]), "not a JSON object"),
            (tar_gz(&[("MANIFEST.json", b"{}")]), "no collection_info"),
            (
                collection(&manifest("Acme", "tools", "1.0.0", json!({}))),
                "namespace Acme is not valid",
            ),
            (
                collection(&manifest("acme", "tools", "1.0", json!({}))),
                "not a semantic version",
            ),
            (
                collection(&manifest("acme", "tools", "1.0.0", json!({"general": "*"}))),
                "dependency general",
            ),
            (
                collection(&manifest("acme", "tools", "1.0.0", json!({"a.b": 1}))),
                "version range",
            ),
            (b"not gzip at all".to_vec(), "cannot be read"),
        ];
        for (archive, expected) in cases {
            let error = read(archive.as_slice()).unwrap_err().to_string();
            assert!(error.contains(expected), "{error} lacks {expected}");
        }
    }

    #[test]
    fn a_manifest_below_a_folder_is_not_the_collection_manifest() {
        let manifest = manifest("acme", "tools", "1.0.0", json!({})).to_string();
        let nested = tar_gz(&[("acme-tools/MANIFEST.json", manifest.as_bytes())]);
        assert!(read(nested.as_slice()).is_err());
        let dotted = tar_gz(&[("./MANIFEST.json", manifest.as_bytes())]);
        assert_eq!(read(dotted.as_slice()).unwrap().version, "1.0.0");
    }
}
