use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoFile {
    pub path: String,
    pub blob_id: Option<String>,
    pub size: i64,
    pub lfs_sha256: Option<String>,
    pub lfs_size: i64,
    pub pointer_size: i64,
}

impl RepoFile {
    pub fn etag(&self) -> Option<&str> {
        self.lfs_sha256.as_deref().or(self.blob_id.as_deref())
    }

    pub fn content_size(&self) -> i64 {
        match self.lfs_sha256 {
            Some(_) if self.lfs_size >= 0 => self.lfs_size,
            _ => self.size,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Revision {
    pub raw: Vec<u8>,
    pub sha: Option<String>,
    pub lists_files: bool,
    files: BTreeMap<String, RepoFile>,
    directories: BTreeSet<String>,
}

fn number(value: &Value) -> i64 {
    value.as_i64().unwrap_or(-1)
}

impl Revision {
    pub fn parse(raw: Vec<u8>) -> Result<Self, String> {
        let root: Value = serde_json::from_slice(&raw)
            .map_err(|error| format!("the revision information is not JSON: {error}"))?;
        if !root.is_object() {
            return Err("the revision information is not a JSON object".to_owned());
        }
        let sha = root["sha"].as_str().map(str::to_owned);
        let siblings = root["siblings"].as_array();
        let mut files = BTreeMap::new();
        for sibling in siblings.into_iter().flatten() {
            let Some(path) = sibling["rfilename"]
                .as_str()
                .filter(|path| !path.is_empty())
            else {
                continue;
            };
            let lfs = &sibling["lfs"];
            let lfs_sha256 = lfs["sha256"].as_str().map(str::to_owned);
            files.insert(
                path.to_owned(),
                RepoFile {
                    path: path.to_owned(),
                    blob_id: sibling["blobId"].as_str().map(str::to_owned),
                    size: number(&sibling["size"]),
                    lfs_size: if lfs_sha256.is_some() {
                        number(&lfs["size"])
                    } else {
                        -1
                    },
                    pointer_size: if lfs_sha256.is_some() {
                        number(&lfs["pointerSize"])
                    } else {
                        -1
                    },
                    lfs_sha256,
                },
            );
        }
        let mut directories = BTreeSet::new();
        for path in files.keys() {
            let mut current = path.as_str();
            while let Some((parent, _)) = current.rsplit_once('/') {
                directories.insert(parent.to_owned());
                current = parent;
            }
        }
        Ok(Self {
            raw,
            sha,
            lists_files: siblings.is_some(),
            files,
            directories,
        })
    }

    pub fn file(&self, path: &str) -> Option<&RepoFile> {
        self.files.get(path)
    }

    fn child(prefix: &str, candidate: &str) -> bool {
        candidate.starts_with(prefix) && !candidate[prefix.len()..].contains('/')
    }

    fn file_entry(file: &RepoFile) -> Value {
        let mut entry = json!({
            "type": "file",
            "oid": file.blob_id.clone().unwrap_or_default(),
            "size": file.content_size().max(0),
        });
        if let Some(sha256) = &file.lfs_sha256 {
            entry["lfs"] = json!({
                "oid": sha256,
                "size": file.lfs_size.max(0),
                "pointerSize": file.pointer_size.max(0),
            });
        }
        entry["path"] = Value::String(file.path.clone());
        entry
    }

    fn directory_entry(directory: &str) -> Value {
        json!({"type": "directory", "oid": "", "size": 0, "path": directory})
    }

    pub fn tree(&self, directory: &str, recursive: bool) -> Option<Value> {
        let directory = directory.trim_matches('/');
        if !directory.is_empty() && !self.directories.contains(directory) {
            return None;
        }
        let prefix = match directory.is_empty() {
            true => String::new(),
            false => format!("{directory}/"),
        };
        let wanted = |candidate: &str| {
            candidate.starts_with(&prefix) && (recursive || Self::child(&prefix, candidate))
        };
        let mut entries: Vec<Value> = self
            .directories
            .iter()
            .filter(|candidate| wanted(candidate))
            .map(|candidate| Self::directory_entry(candidate))
            .collect();
        entries.extend(
            self.files
                .values()
                .filter(|file| wanted(&file.path))
                .map(Self::file_entry),
        );
        Some(Value::Array(entries))
    }

    pub fn paths_info(&self, paths: &[String]) -> Value {
        let mut entries = Vec::new();
        for path in paths {
            let candidate = path.trim_matches('/');
            if let Some(file) = self.files.get(candidate) {
                entries.push(Self::file_entry(file));
            } else if !candidate.is_empty() && self.directories.contains(candidate) {
                entries.push(Self::directory_entry(candidate));
            }
        }
        Value::Array(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INFO: &str = r#"{"id":"acme/bert","sha":"f171d7baecaf37b5da5a3616d8833b9969753535","siblings":[
        {"rfilename":"config.json","blobId":"22ffb3454131a71d4144340befb799c66ad0c670","size":10},
        {"rfilename":"onnx/model.onnx","blobId":"aa","size":134,"lfs":{"sha256":"5e2b","size":2048,"pointerSize":134}}]}"#;

    #[test]
    fn revisions_list_files_and_directories() {
        let revision = Revision::parse(INFO.as_bytes().to_vec()).unwrap();
        assert_eq!(
            revision.sha.as_deref(),
            Some("f171d7baecaf37b5da5a3616d8833b9969753535")
        );
        let model = revision.file("onnx/model.onnx").unwrap();
        assert_eq!(model.etag(), Some("5e2b"));
        assert_eq!(model.content_size(), 2048);
        let root = revision.tree("", false).unwrap();
        assert_eq!(root.as_array().unwrap().len(), 2);
        assert_eq!(root[0]["type"], "directory");
        assert_eq!(
            revision.tree("", true).unwrap().as_array().unwrap().len(),
            3
        );
        assert!(revision.tree("missing", false).is_none());
        let info =
            revision.paths_info(&["config.json".to_owned(), "onnx".to_owned(), "x".to_owned()]);
        assert_eq!(info.as_array().unwrap().len(), 2);
        assert_eq!(info[0]["oid"], "22ffb3454131a71d4144340befb799c66ad0c670");
        assert!(Revision::parse(b"[]".to_vec()).is_err());
    }
}
