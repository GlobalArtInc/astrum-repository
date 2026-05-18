use std::io::Read;

use astrum_content::{ContentError, Ctx, UploadField, UploadForm, UploadSpec};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::hosted::{self, MAX_PUBLISH};
use crate::path::Name;
use crate::{ID, Npm};

const MAX_MANIFEST: u64 = 1024 * 1024;
const MAX_INFLATED: u64 = 1024 * 1024 * 1024;

pub fn spec() -> UploadSpec {
    UploadSpec {
        format: ID,
        multiple_upload: false,
        component_fields: vec![],
        asset_fields: vec![UploadField::file("asset")],
    }
}

fn bad(message: impl Into<String>) -> ContentError {
    ContentError::BadRequest(message.into())
}

pub fn manifest(tarball: &[u8]) -> Result<Value, String> {
    if !tarball.starts_with(&[0x1f, 0x8b]) {
        return Err("the file is not a gzip-compressed tarball (.tgz)".to_owned());
    }
    let inflated = flate2::read::GzDecoder::new(tarball).take(MAX_INFLATED);
    let mut archive = tar::Archive::new(inflated);
    let entries = archive
        .entries()
        .map_err(|error| format!("the tarball cannot be read: {error}"))?;
    for entry in entries {
        let mut entry = entry.map_err(|error| format!("the tarball cannot be read: {error}"))?;
        let path = entry
            .path()
            .map_err(|error| format!("the tarball cannot be read: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let path = path.trim_start_matches("./");
        let top_level = matches!(
            path.split('/').collect::<Vec<_>>().as_slice(),
            [_, "package.json"]
        );
        if !top_level || !entry.header().entry_type().is_file() {
            continue;
        }
        let mut data = Vec::new();
        (&mut entry)
            .take(MAX_MANIFEST)
            .read_to_end(&mut data)
            .map_err(|error| format!("package.json cannot be read: {error}"))?;
        let manifest: Value = serde_json::from_slice(&data)
            .map_err(|error| format!("package.json is not valid JSON: {error}"))?;
        if !manifest.is_object() {
            return Err("package.json is not an object".to_owned());
        }
        return Ok(manifest);
    }
    Err("the tarball holds no package.json".to_owned())
}

pub fn document(manifest: &Value, tarball: &[u8]) -> Result<(Name, Value), String> {
    let full = manifest
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "package.json names no package".to_owned())?;
    let name = Name::parse(full).ok_or_else(|| format!("{full} is not a valid package name"))?;
    let version = manifest
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| "package.json names no version".to_owned())?
        .to_owned();
    let mut published = manifest.clone();
    published["_id"] = Value::String(format!("{name}@{version}"));
    published["dist"] = json!({});
    let mut document = json!({
        "_id": name.to_string(),
        "name": name.to_string(),
        "dist-tags": {"latest": version},
        "versions": {version.clone(): published},
        "_attachments": {
            name.tarball_file(&version): {
                "content_type": "application/octet-stream",
                "data": STANDARD.encode(tarball),
                "length": tarball.len(),
            }
        }
    });
    for key in ["description", "readme", "license", "keywords", "homepage"] {
        if let Some(value) = manifest.get(key) {
            document[key] = value.clone();
        }
    }
    Ok((name, document))
}

pub async fn upload(npm: &Npm, ctx: &Ctx, form: UploadForm) -> Result<(), ContentError> {
    let file = form
        .files
        .get("npm.asset")
        .ok_or_else(|| bad("npm.asset is required"))?;
    if file.size as usize > MAX_PUBLISH {
        return Err(bad("npm.asset is larger than a package may be"));
    }
    let tarball = tokio::fs::read(&file.path)
        .await
        .map_err(ContentError::internal)?;
    let (tarball, found) = tokio::task::spawn_blocking(move || {
        let found = manifest(&tarball);
        (tarball, found)
    })
    .await
    .map_err(ContentError::internal)?;
    let found = found.map_err(|message| bad(format!("npm.asset: {message}")))?;
    let (name, incoming) =
        document(&found, &tarball).map_err(|message| bad(format!("npm.asset: {message}")))?;
    drop(tarball);
    hosted::publish_document(npm, ctx, &name, &incoming).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tgz(files: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for (name, data) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *data).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn the_manifest_comes_from_the_top_level_package_json() {
        let tarball = tgz(&[
            (
                "package/lib/package.json",
                br#"{"name":"inner","version":"0.0.1"}"#,
            ),
            (
                "package/package.json",
                br#"{"name":"@acme/widgets","version":"1.2.3","description":"Widgets"}"#,
            ),
        ]);
        let found = manifest(&tarball).unwrap();
        assert_eq!(found["name"], "@acme/widgets");
        let (name, document) = document(&found, &tarball).unwrap();
        assert_eq!(name.to_string(), "@acme/widgets");
        assert_eq!(document["dist-tags"]["latest"], "1.2.3");
        assert_eq!(document["description"], "Widgets");
        assert_eq!(document["versions"]["1.2.3"]["_id"], "@acme/widgets@1.2.3");
        assert_eq!(
            document["_attachments"]["widgets-1.2.3.tgz"]["length"],
            tarball.len()
        );
    }

    #[test]
    fn broken_tarballs_are_refused() {
        assert!(manifest(b"plain text").unwrap_err().contains("gzip"));
        let empty = tgz(&[("package/index.js", b"module.exports = 1;")]);
        assert!(manifest(&empty).unwrap_err().contains("no package.json"));
        let unnamed = tgz(&[("package/package.json", br#"{"version":"1.0.0"}"#)]);
        let found = manifest(&unnamed).unwrap();
        assert!(
            document(&found, &unnamed)
                .unwrap_err()
                .contains("names no package")
        );
        let bad_name = tgz(&[(
            "package/package.json",
            br#"{"name":"../x","version":"1.0.0"}"#,
        )]);
        let found = manifest(&bad_name).unwrap();
        assert!(
            document(&found, &bad_name)
                .unwrap_err()
                .contains("not a valid")
        );
    }
}
