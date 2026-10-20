use std::io::{self, Read};

use serde_json::{Map, Value, json};

use crate::names::{self, Module};

const MAX_METADATA: u64 = 1024 * 1024;
const MAX_INFLATED: u64 = 1024 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("{0}")]
    Invalid(String),
    #[error("the module archive cannot be read: {0}")]
    Io(#[from] io::Error),
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub module: Module,
    pub version: String,
    pub metadata: Map<String, Value>,
}

impl Release {
    pub fn directory(&self) -> String {
        self.module.release_slug(&self.version)
    }

    pub fn attributes(&self) -> Value {
        attributes_of(&Value::Object(self.metadata.clone()))
    }
}

pub fn attributes_of(metadata: &Value) -> Value {
    let text = |key: &str| {
        metadata
            .get(key)
            .filter(|value| value.is_string())
            .cloned()
            .unwrap_or(Value::Null)
    };
    let tags: Vec<Value> = metadata
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| tags.iter().filter(|tag| tag.is_string()).cloned().collect())
        .unwrap_or_default();
    json!({
        "metadata": metadata,
        "summary": text("summary"),
        "author": text("author"),
        "license": text("license"),
        "source": text("source"),
        "tags": tags,
        "pdk": metadata.get("pdk-version").is_some(),
    })
}

fn entry_path(path: &str) -> String {
    let mut path = path.trim_start_matches("./").to_owned();
    while path.ends_with('/') {
        path.pop();
    }
    path
}

pub fn read(reader: impl Read) -> Result<Release, ArchiveError> {
    let inflated = flate2::read::GzDecoder::new(reader).take(MAX_INFLATED);
    let mut archive = tar::Archive::new(inflated);
    let mut found: Option<(String, Vec<u8>)> = None;
    let mut loose = false;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let kind = entry.header().entry_type();
        if kind.is_pax_global_extensions() || kind.is_pax_local_extensions() || kind.is_dir() {
            continue;
        }
        let path = match entry.path() {
            Ok(path) => entry_path(&path.to_string_lossy()),
            Err(_) => continue,
        };
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            ["metadata.json"] => loose = true,
            [top, "metadata.json"] if found.is_none() && !top.starts_with('.') => {
                let mut bytes = Vec::new();
                (&mut entry).take(MAX_METADATA).read_to_end(&mut bytes)?;
                found = Some(((*top).to_owned(), bytes));
            }
            _ => {}
        }
    }
    let Some((top, bytes)) = found else {
        return Err(invalid(match loose {
            true => {
                "metadata.json must be inside the top directory <owner>-<name>-<version>/ of the archive"
            }
            false => "the archive has no <owner>-<name>-<version>/metadata.json",
        }));
    };
    let metadata = match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(object)) => object,
        Ok(_) => return Err(invalid("metadata.json is not a JSON object")),
        Err(error) => return Err(invalid(format!("metadata.json is not valid JSON: {error}"))),
    };
    let (module, version) = check(&metadata)?;
    let release = Release {
        module,
        version,
        metadata,
    };
    if !top.eq_ignore_ascii_case(&release.directory()) {
        return Err(invalid(format!(
            "the top directory {top}/ does not match metadata.json, which names {} {}",
            release.module, release.version
        )));
    }
    Ok(release)
}

fn check(metadata: &Map<String, Value>) -> Result<(Module, String), ArchiveError> {
    let text = |key: &str| {
        metadata
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid(format!("metadata.json has no {key}")))
    };
    let name = text("name")?;
    let version = text("version")?;
    let module = Module::parse(name).ok_or_else(|| {
        invalid(format!(
            "the name {name} in metadata.json is not <owner>-<name>: the owner takes letters and digits, the name lower-case letters, digits and underscores and starts with a letter"
        ))
    })?;
    if !names::valid_version(version) {
        return Err(invalid(format!(
            "the version {version} in metadata.json is not a semantic version"
        )));
    }
    match metadata.get("dependencies") {
        None | Some(Value::Null) => {}
        Some(Value::Array(dependencies)) => {
            for dependency in dependencies {
                let named = dependency.get("name").and_then(Value::as_str);
                if !named.is_some_and(|named| Module::parse(named).is_some()) {
                    return Err(invalid(format!(
                        "the dependency {dependency} of metadata.json does not name an <owner>-<name> module"
                    )));
                }
                if dependency
                    .get("version_requirement")
                    .is_some_and(|wanted| !wanted.is_string() && !wanted.is_null())
                {
                    return Err(invalid(format!(
                        "the version_requirement of the dependency {} is not text",
                        named.unwrap_or_default()
                    )));
                }
            }
        }
        Some(_) => {
            return Err(invalid("the dependencies of metadata.json are not a list"));
        }
    }
    Ok((module, version.to_owned()))
}

#[cfg(test)]
pub(crate) mod testing {
    use std::io::Write;

    use serde_json::{Value, json};

    pub fn metadata(name: &str, version: &str, dependencies: Value) -> Value {
        json!({
            "name": name,
            "version": version,
            "author": "Jane Doe",
            "summary": "A test module",
            "license": "Apache-2.0",
            "source": "https://example.org/repo",
            "project_page": "https://example.org/project",
            "issues_url": "https://example.org/issues",
            "dependencies": dependencies,
            "operatingsystem_support": [{"operatingsystem": "Debian", "operatingsystemrelease": ["12"]}],
            "requirements": [{"name": "puppet", "version_requirement": ">= 7.0.0 < 9.0.0"}],
            "tags": ["tools", "demo"],
            "pdk-version": "3.0.0"
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

    pub fn module(top: &str, metadata: &Value) -> Vec<u8> {
        let text = metadata.to_string();
        tar_gz(&[
            (&format!("{top}/"), b""),
            (&format!("{top}/metadata.json"), text.as_bytes()),
            (&format!("{top}/README.md"), b"# Test\n"),
            (&format!("{top}/manifests/init.pp"), b"class tools {}\n"),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;

    #[test]
    fn a_built_module_answers_its_metadata() {
        let archive = module(
            "acme-tools-1.2.0",
            &metadata(
                "acme-tools",
                "1.2.0",
                json!([{"name": "puppetlabs/stdlib", "version_requirement": ">= 9.0.0 < 10.0.0"}]),
            ),
        );
        let found = read(archive.as_slice()).unwrap();
        assert_eq!(found.module.to_string(), "acme-tools");
        assert_eq!(found.version, "1.2.0");
        let attributes = found.attributes();
        assert_eq!(attributes["tags"], json!(["tools", "demo"]));
        assert_eq!(attributes["license"], "Apache-2.0");
        assert_eq!(attributes["pdk"], true);
        assert_eq!(
            attributes["metadata"]["dependencies"][0]["name"],
            "puppetlabs/stdlib"
        );
    }

    #[test]
    fn owners_in_any_case_and_slashes_in_names_are_read() {
        let archive = module(
            "Acme-tools-1.0.0",
            &metadata("Acme/tools", "1.0.0", json!([])),
        );
        let found = read(archive.as_slice()).unwrap();
        assert_eq!(found.module.owner, "acme");
        assert_eq!(found.metadata["name"], "Acme/tools");
        let dotted = tar_gz(&[(
            "./acme-tools-1.0.0/metadata.json",
            metadata("acme-tools", "1.0.0", json!([]))
                .to_string()
                .as_bytes(),
        )]);
        assert_eq!(read(dotted.as_slice()).unwrap().version, "1.0.0");
    }

    #[test]
    fn broken_archives_are_refused() {
        let good = metadata("acme-tools", "1.0.0", json!([]));
        let cases = [
            (
                tar_gz(&[("README.md", b"x")]),
                "has no <owner>-<name>-<version>/metadata.json",
            ),
            (
                tar_gz(&[("metadata.json", good.to_string().as_bytes())]),
                "must be inside the top directory",
            ),
            (
                tar_gz(&[("acme-tools-1.0.0/metadata.json", b"[1]")]),
                "not a JSON object",
            ),
            (
                tar_gz(&[("acme-tools-1.0.0/metadata.json", b"{")]),
                "not valid JSON",
            ),
            (
                module("acme-tools-1.0.0", &json!({"version": "1.0.0"})),
                "has no name",
            ),
            (
                module("acme-tools-1.0.0", &metadata("tools", "1.0.0", json!([]))),
                "is not <owner>-<name>",
            ),
            (
                module("acme-tools-1.0", &metadata("acme-tools", "1.0", json!([]))),
                "not a semantic version",
            ),
            (
                module("acme-tools-2.0.0", &good),
                "top directory acme-tools-2.0.0/ does not match",
            ),
            (
                module(
                    "acme-tools-1.0.0",
                    &metadata("acme-tools", "1.0.0", json!([{"name": "stdlib"}])),
                ),
                "does not name an <owner>-<name> module",
            ),
            (
                module(
                    "acme-tools-1.0.0",
                    &metadata(
                        "acme-tools",
                        "1.0.0",
                        json!([{"name": "a-b", "version_requirement": 1}]),
                    ),
                ),
                "is not text",
            ),
            (
                module(
                    "acme-tools-1.0.0",
                    &metadata("acme-tools", "1.0.0", json!({"a-b": "1"})),
                ),
                "are not a list",
            ),
            (b"not gzip at all".to_vec(), "cannot be read"),
        ];
        for (archive, expected) in cases {
            let error = read(archive.as_slice()).unwrap_err().to_string();
            assert!(error.contains(expected), "{error} lacks {expected}");
        }
    }
}
