use std::io::{self, Read};

use serde_json::{Map, Value, json};

use crate::names;

pub const MAX_METADATA: u64 = 4 * 1024 * 1024;
const MAX_INFLATED: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("{0}")]
    Invalid(String),
    #[error("the cookbook archive cannot be read: {0}")]
    Io(#[from] io::Error),
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub json: Map<String, Value>,
}

fn constraints(value: Option<&Value>) -> Value {
    let mut kept = Map::new();
    if let Some(Value::Object(map)) = value {
        for (name, constraint) in map {
            match constraint {
                Value::String(text) => {
                    kept.insert(name.clone(), json!(text));
                }
                Value::Array(items) if !items.is_empty() => {
                    let joined: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
                    kept.insert(name.clone(), json!(joined.join(", ")));
                }
                _ => {
                    kept.insert(name.clone(), json!(">= 0.0.0"));
                }
            }
        }
    }
    Value::Object(kept)
}

impl Metadata {
    pub fn text(&self, key: &str) -> Option<&str> {
        self.json
            .get(key)
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
    }

    pub fn dependencies(&self) -> Value {
        constraints(self.json.get("dependencies"))
    }

    pub fn platforms(&self) -> Value {
        constraints(self.json.get("platforms"))
    }

    pub fn attributes(&self, category: &str) -> Value {
        json!({
            "description": self.text("description"),
            "maintainer": self.text("maintainer"),
            "license": self.text("license"),
            "source_url": self.text("source_url"),
            "issues_url": self.text("issues_url"),
            "category": category,
            "dependencies": self.dependencies(),
            "platforms": self.platforms(),
        })
    }
}

fn entry_path(path: &str) -> Vec<String> {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .map(str::to_owned)
        .collect()
}

pub fn parse_metadata(folder: &str, data: &[u8]) -> Result<Metadata, ArchiveError> {
    let value: Value = serde_json::from_slice(data)
        .map_err(|error| invalid(format!("{folder}/metadata.json is not JSON: {error}")))?;
    let Value::Object(json) = value else {
        return Err(invalid(format!(
            "{folder}/metadata.json does not hold an object"
        )));
    };
    let name = match json.get("name") {
        Some(Value::String(name)) if !name.is_empty() => name.clone(),
        _ => {
            return Err(invalid(format!(
                "{folder}/metadata.json has no \"name\" field"
            )));
        }
    };
    let version = match json.get("version") {
        Some(Value::String(version)) => version.trim().to_owned(),
        _ => {
            return Err(invalid(format!(
                "{folder}/metadata.json has no \"version\" field"
            )));
        }
    };
    if !names::valid_name(&name) {
        return Err(invalid(format!(
            "\"{name}\" is not a valid cookbook name: use letters, digits, hyphens and underscores"
        )));
    }
    if !names::valid_version(&version) {
        return Err(invalid(format!(
            "\"{version}\" is not a valid cookbook version: use two or three numbers such as 1.2.3"
        )));
    }
    if folder != name {
        return Err(invalid(format!(
            "the archive keeps metadata.json in {folder}/, but the cookbook is named {name}; pack it as {name}/metadata.json"
        )));
    }
    Ok(Metadata {
        name,
        version,
        json,
    })
}

pub fn read(reader: impl Read) -> Result<Metadata, ArchiveError> {
    let mut reader = reader;
    let mut magic = [0u8; 2];
    let mut filled = 0;
    while filled < magic.len() {
        match reader.read(&mut magic[filled..])? {
            0 => break,
            read => filled += read,
        }
    }
    if filled < magic.len() || magic != [0x1f, 0x8b] {
        return Err(invalid(
            "the cookbook is not a gzip-compressed tar archive (.tgz or .tar.gz)",
        ));
    }
    let joined = io::Cursor::new(magic).chain(reader);
    let inflated = flate2::read::GzDecoder::new(joined).take(MAX_INFLATED);
    let mut tar = tar::Archive::new(inflated);
    let mut ruby: Option<String> = None;
    let mut at_top = false;
    for entry in tar.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let parts = entry_path(&entry.path()?.to_string_lossy());
        match parts.as_slice() {
            [folder, file] if file == "metadata.json" => {
                if entry.size() > MAX_METADATA {
                    return Err(invalid(format!(
                        "{folder}/metadata.json is larger than {MAX_METADATA} bytes"
                    )));
                }
                let mut data = Vec::new();
                (&mut entry).take(MAX_METADATA).read_to_end(&mut data)?;
                return parse_metadata(folder, &data);
            }
            [folder, file] if file == "metadata.rb" => {
                ruby.get_or_insert_with(|| folder.clone());
            }
            [file] if file == "metadata.json" => at_top = true,
            _ => {}
        }
    }
    match (ruby, at_top) {
        (Some(folder), _) => Err(invalid(format!(
            "the archive has {folder}/metadata.rb but no {folder}/metadata.json; Astrum Repository reads the cookbook from metadata.json, so generate it with knife cookbook metadata from file {folder}/metadata.rb (or knife cookbook metadata {folder} -o <cookbook path>) and pack the archive again"
        ))),
        (None, true) => Err(invalid(
            "the archive keeps metadata.json at its top; pack the cookbook in a folder named after it, as <name>/metadata.json",
        )),
        (None, false) => Err(invalid(
            "the archive holds no <name>/metadata.json; pack the cookbook folder with its metadata.json",
        )),
    }
}

pub fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (name, data) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, *data)
            .expect("a test archive can be written");
    }
    builder
        .into_inner()
        .and_then(|encoder| encoder.finish())
        .expect("a test archive can be finished")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(name: &str, version: &str) -> String {
        json!({
            "name": name,
            "version": version,
            "description": "Installs things",
            "maintainer": "Acme Ops",
            "license": "Apache-2.0",
            "platforms": {"ubuntu": ">= 22.04", "debian": []},
            "dependencies": {"apt": "~> 7.0", "yum": ">= 0.0.0", "old": [">= 1.0", "< 2.0"]},
            "source_url": "https://example.org/acme",
        })
        .to_string()
    }

    #[test]
    fn metadata_json_in_the_cookbook_folder_is_read() {
        let archive = build(&[
            ("acme/recipes/default.rb", b"package 'acme'\n"),
            ("acme/metadata.rb", b"name 'acme'\nversion '1.2.3'\n"),
            ("acme/metadata.json", metadata("acme", "1.2.3").as_bytes()),
        ]);
        let read = read(archive.as_slice()).unwrap();
        assert_eq!(read.name, "acme");
        assert_eq!(read.version, "1.2.3");
        assert_eq!(
            read.dependencies(),
            json!({"apt": "~> 7.0", "yum": ">= 0.0.0", "old": ">= 1.0, < 2.0"})
        );
        assert_eq!(
            read.platforms(),
            json!({"ubuntu": ">= 22.04", "debian": ">= 0.0.0"})
        );
        let attributes = read.attributes("Other");
        assert_eq!(attributes["maintainer"], "Acme Ops");
        assert_eq!(attributes["issues_url"], Value::Null);
        let dotted = build(&[("./acme/metadata.json", metadata("acme", "1.0").as_bytes())]);
        assert_eq!(read_ok(&dotted).version, "1.0");
    }

    fn read_ok(archive: &[u8]) -> Metadata {
        read(archive).unwrap()
    }

    fn refusal(archive: &[u8]) -> String {
        read(archive).unwrap_err().to_string()
    }

    #[test]
    fn archives_without_a_usable_metadata_json_are_refused() {
        let only_ruby = build(&[("acme/metadata.rb", b"name 'acme'\nversion '1.0.0'\n")]);
        assert!(refusal(&only_ruby).contains("knife cookbook metadata from file acme/metadata.rb"));
        let top = build(&[("metadata.json", metadata("acme", "1.0.0").as_bytes())]);
        assert!(refusal(&top).contains("at its top"));
        let empty = build(&[("acme/README.md", b"# acme\n")]);
        assert!(refusal(&empty).contains("no <name>/metadata.json"));
        let renamed = build(&[(
            "chef-acme/metadata.json",
            metadata("acme", "1.0.0").as_bytes(),
        )]);
        assert!(refusal(&renamed).contains("pack it as acme/metadata.json"));
        assert!(refusal(b"PK\x03\x04").contains("gzip"));
        assert!(refusal(b"").contains("gzip"));
        for (text, message) in [
            ("[1]", "does not hold an object"),
            ("{\"version\": \"1.0.0\"}", "no \"name\""),
            ("{\"name\": \"acme\"}", "no \"version\""),
            (
                "{\"name\": \"acme\", \"version\": \"1.0.0-rc1\"}",
                "not a valid cookbook version",
            ),
            (
                "{\"name\": \"a b\", \"version\": \"1.0.0\"}",
                "not a valid cookbook name",
            ),
            ("{", "not JSON"),
        ] {
            let error = parse_metadata("acme", text.as_bytes())
                .unwrap_err()
                .to_string();
            assert!(error.contains(message), "{text}: {error}");
        }
    }
}
