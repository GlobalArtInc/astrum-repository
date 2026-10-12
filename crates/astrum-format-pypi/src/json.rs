use serde_json::{Map, Value, json};

use crate::html::href;
use crate::index::{File, Hashes, Project};

pub const CONTENT_TYPE: &str = "application/vnd.pypi.simple.v1+json";

fn hashes_json(hashes: &Hashes) -> Value {
    Value::Object(
        hashes
            .iter()
            .map(|(name, value)| (name.clone(), Value::String(value.clone())))
            .collect(),
    )
}

fn file_json(file: &File, prefix: &str) -> Value {
    let mut entry = Map::new();
    entry.insert("filename".to_owned(), json!(file.filename));
    entry.insert("url".to_owned(), json!(href(prefix, &file.url)));
    entry.insert("hashes".to_owned(), hashes_json(&file.hashes));
    if let Some(requires) = &file.requires_python {
        entry.insert("requires-python".to_owned(), json!(requires));
    }
    entry.insert(
        "yanked".to_owned(),
        match &file.yanked {
            None => json!(false),
            Some(reason) if reason.is_empty() => json!(true),
            Some(reason) => json!(reason),
        },
    );
    if let Some(metadata) = &file.core_metadata {
        let value = match metadata.is_empty() {
            true => json!(true),
            false => hashes_json(metadata),
        };
        entry.insert("core-metadata".to_owned(), value.clone());
        entry.insert("dist-info-metadata".to_owned(), value);
    }
    if let Some(size) = file.size {
        entry.insert("size".to_owned(), json!(size));
    }
    if let Some(time) = &file.upload_time {
        entry.insert("upload-time".to_owned(), json!(time));
    }
    Value::Object(entry)
}

pub fn render_project(project: &Project, prefix: &str) -> String {
    let complete = project.files.iter().all(|file| file.size.is_some());
    let mut page = json!({
        "meta": {"api-version": if complete { "1.1" } else { "1.0" }},
        "name": project.name,
        "files": project.files.iter().map(|file| file_json(file, prefix)).collect::<Vec<_>>(),
    });
    if complete {
        page["versions"] = json!(project.all_versions());
    }
    page.to_string()
}

pub fn render_root(names: &[String]) -> String {
    json!({
        "meta": {"api-version": "1.0"},
        "projects": names.iter().map(|name| json!({"name": name})).collect::<Vec<_>>(),
    })
    .to_string()
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|text| !text.trim().is_empty())
}

fn parse_hashes(value: &Value) -> Hashes {
    value
        .as_object()
        .map(|object| {
            object
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .as_str()
                        .map(|digest| (name.to_ascii_lowercase(), digest.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_metadata(value: Option<&Value>) -> Option<Hashes> {
    match value? {
        Value::Bool(true) => Some(Hashes::new()),
        object @ Value::Object(_) => Some(parse_hashes(object)),
        _ => None,
    }
}

#[derive(Debug, thiserror::Error)]
#[error("the upstream index is not a PEP 691 document: {0}")]
pub struct Invalid(String);

pub fn parse_project(name: &str, body: &[u8], base: &url::Url) -> Result<Project, Invalid> {
    let document: Value =
        serde_json::from_slice(body).map_err(|error| Invalid(error.to_string()))?;
    let files = document
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| Invalid("files is missing".to_owned()))?;
    let mut parsed = Vec::with_capacity(files.len());
    for file in files {
        let (Some(filename), Some(url)) = (text(file.get("filename")), text(file.get("url")))
        else {
            continue;
        };
        if !crate::name::is_safe_segment(&filename) {
            continue;
        }
        let Ok(mut url) = base.join(url.trim()) else {
            continue;
        };
        url.set_fragment(None);
        let yanked = match file.get("yanked") {
            Some(Value::Bool(true)) => Some(String::new()),
            Some(Value::String(reason)) => Some(reason.clone()),
            _ => None,
        };
        parsed.push(File {
            filename,
            url: url.to_string(),
            version: None,
            hashes: file.get("hashes").map(parse_hashes).unwrap_or_default(),
            requires_python: text(file.get("requires-python")),
            yanked,
            core_metadata: parse_metadata(
                file.get("core-metadata")
                    .or_else(|| file.get("dist-info-metadata"))
                    .or_else(|| file.get("data-dist-info-metadata")),
            ),
            size: file.get("size").and_then(Value::as_u64),
            upload_time: text(file.get("upload-time")),
        });
    }
    Ok(Project {
        name: text(document.get("name")).unwrap_or_else(|| name.to_owned()),
        files: parsed,
        versions: document
            .get("versions")
            .and_then(Value::as_array)
            .map(|versions| {
                versions
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn parse_root(body: &[u8]) -> Result<Vec<String>, Invalid> {
    let document: Value =
        serde_json::from_slice(body).map_err(|error| Invalid(error.to_string()))?;
    Ok(document
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| Invalid("projects is missing".to_owned()))?
        .iter()
        .filter_map(|project| text(project.get("name")))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const UPSTREAM: &str = r#"{
      "meta": {"_last-serial": 1, "api-version": "1.4"},
      "name": "six",
      "versions": ["1.16.0", "1.17.0"],
      "files": [
        {"filename": "six-1.17.0-py2.py3-none-any.whl", "url": "https://files.example/b7/six-1.17.0-py2.py3-none-any.whl",
         "hashes": {"sha256": "4721"}, "requires-python": ">=2.7", "yanked": false,
         "core-metadata": {"sha256": "5620"}, "data-dist-info-metadata": {"sha256": "5620"}, "size": 11050,
         "upload-time": "2024-12-04T17:35:26.475883Z"},
        {"filename": "six-1.16.0.tar.gz", "url": "../../packages/six-1.16.0.tar.gz", "hashes": {"sha256": "1e61", "md5": "a7c9"},
         "yanked": "use 1.17", "dist-info-metadata": true, "requires-python": ""},
        {"filename": "../escape", "url": "x"},
        {"url": "missing-name"}
      ]
    }"#;

    #[test]
    fn upstream_json_parses() {
        let base = url::Url::parse("https://pypi.example/simple/six/").unwrap();
        let project = parse_project("six", UPSTREAM.as_bytes(), &base).unwrap();
        assert_eq!(project.versions, ["1.16.0", "1.17.0"]);
        assert_eq!(project.files.len(), 2);
        let wheel = &project.files[0];
        assert_eq!(wheel.size, Some(11050));
        assert_eq!(wheel.core_metadata.as_ref().unwrap()["sha256"], "5620");
        assert_eq!(wheel.yanked, None);
        assert_eq!(
            wheel.upload_time.as_deref(),
            Some("2024-12-04T17:35:26.475883Z")
        );
        let sdist = &project.files[1];
        assert_eq!(sdist.url, "https://pypi.example/packages/six-1.16.0.tar.gz");
        assert_eq!(sdist.yanked.as_deref(), Some("use 1.17"));
        assert_eq!(sdist.core_metadata, Some(Hashes::new()));
        assert_eq!(sdist.requires_python, None);
        assert_eq!(sdist.hashes.len(), 2);
        assert!(parse_project("six", b"<html>", &base).is_err());
    }

    #[test]
    fn pages_render_as_pep_691() {
        let base = url::Url::parse("https://pypi.example/simple/six/").unwrap();
        let project = parse_project("six", UPSTREAM.as_bytes(), &base)
            .unwrap()
            .localize();
        let rendered: Value = serde_json::from_str(&render_project(&project, "../../")).unwrap();
        assert_eq!(rendered["meta"]["api-version"], "1.0");
        assert!(rendered.get("versions").is_none());
        let wheel = &rendered["files"][0];
        assert_eq!(
            wheel["url"],
            "../../packages/six/1.17.0/six-1.17.0-py2.py3-none-any.whl"
        );
        assert_eq!(wheel["core-metadata"]["sha256"], "5620");
        assert_eq!(wheel["dist-info-metadata"]["sha256"], "5620");
        assert_eq!(wheel["yanked"], false);
        assert_eq!(rendered["files"][1]["yanked"], "use 1.17");
        assert_eq!(rendered["files"][1]["core-metadata"], true);

        let mut sized = project.clone();
        sized.files.truncate(1);
        let rendered: Value = serde_json::from_str(&render_project(&sized, "")).unwrap();
        assert_eq!(rendered["meta"]["api-version"], "1.1");
        assert_eq!(rendered["versions"], json!(["1.16.0", "1.17.0"]));
    }

    #[test]
    fn roots_round_trip() {
        let names = vec!["a".to_owned(), "b-c".to_owned()];
        assert_eq!(parse_root(render_root(&names).as_bytes()).unwrap(), names);
    }
}
