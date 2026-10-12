use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::packument::{self, valid_version};
use crate::path::Name;

#[derive(Debug)]
pub struct Tarball {
    pub version: String,
    pub manifest: Value,
    pub file: String,
    pub data: Vec<u8>,
}

pub fn tarballs(name: &Name, incoming: &Value) -> Result<Vec<Tarball>, String> {
    let attachments = incoming
        .get("_attachments")
        .and_then(Value::as_object)
        .filter(|attachments| !attachments.is_empty())
        .ok_or_else(|| "the document carries no tarball".to_owned())?;
    let versions = packument::versions(incoming)
        .filter(|versions| !versions.is_empty())
        .ok_or_else(|| "the document names no version".to_owned())?;
    let mut found: Vec<Tarball> = Vec::new();
    for (key, attachment) in attachments {
        let version = owner(name, versions, key)?;
        if found.iter().any(|tarball| tarball.version == version) {
            return Err(format!("version {version} has more than one tarball"));
        }
        let manifest = versions[&version].clone();
        found.push(check(name, &version, manifest, attachment)?);
    }
    Ok(found)
}

fn owner(name: &Name, versions: &Map<String, Value>, key: &str) -> Result<String, String> {
    let file = key.rsplit('/').next().unwrap_or(key);
    let matching: Vec<&String> = versions
        .iter()
        .filter(|(version, manifest)| {
            packument::tarball_file(manifest).as_deref() == Some(file)
                || name.tarball_file(version) == file
        })
        .map(|(version, _)| version)
        .collect();
    match (matching.as_slice(), versions.len()) {
        ([version], _) => Ok((*version).clone()),
        ([], 1) => Ok(versions.keys().next().cloned().unwrap_or_default()),
        _ => Err(format!("the tarball {key} belongs to no version")),
    }
}

fn check(
    name: &Name,
    version: &str,
    mut manifest: Value,
    attachment: &Value,
) -> Result<Tarball, String> {
    if !valid_version(version) {
        return Err(format!("{version} is not a valid semantic version"));
    }
    let full = name.to_string();
    if manifest.get("name").and_then(Value::as_str) != Some(full.as_str()) {
        return Err(format!("the manifest of {version} does not name {full}"));
    }
    if manifest.get("version").and_then(Value::as_str) != Some(version) {
        return Err(format!("the manifest of {version} names another version"));
    }
    let encoded = attachment
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("the tarball of {version} has no data"))?;
    let data = STANDARD
        .decode(encoded.trim())
        .map_err(|_| format!("the tarball of {version} is not valid base64"))?;
    if let Some(length) = attachment.get("length").and_then(Value::as_u64)
        && length != data.len() as u64
    {
        return Err(format!(
            "the tarball of {version} has {} bytes, not {length}",
            data.len()
        ));
    }
    if !data.starts_with(&[0x1f, 0x8b]) {
        return Err(format!("the tarball of {version} is not gzip data"));
    }
    let dist = manifest
        .get("dist")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let (shasum, integrity) = verify(&dist, &data)?;
    let mut dist = dist.as_object().cloned().unwrap_or_default();
    dist.entry("shasum").or_insert(Value::String(shasum));
    dist.entry("integrity").or_insert(Value::String(integrity));
    manifest["dist"] = Value::Object(dist);
    Ok(Tarball {
        version: version.to_owned(),
        manifest,
        file: name.tarball_file(version),
        data,
    })
}

pub fn verify(dist: &Value, data: &[u8]) -> Result<(String, String), String> {
    let sha1 = Sha1::digest(data);
    let sha512 = Sha512::digest(data);
    let shasum = hex::encode(sha1);
    if let Some(given) = dist.get("shasum").and_then(Value::as_str)
        && !given.eq_ignore_ascii_case(&shasum)
    {
        return Err("the shasum does not match the tarball".to_owned());
    }
    if let Some(given) = dist.get("integrity").and_then(Value::as_str) {
        for entry in given.split_whitespace() {
            let entry = entry.split('?').next().unwrap_or(entry);
            let Some((algorithm, digest)) = entry.split_once('-') else {
                continue;
            };
            let computed = match algorithm {
                "sha1" => STANDARD.encode(sha1),
                "sha256" => STANDARD.encode(Sha256::digest(data)),
                "sha384" => STANDARD.encode(Sha384::digest(data)),
                "sha512" => STANDARD.encode(sha512),
                _ => continue,
            };
            if computed != digest {
                return Err(format!(
                    "the {algorithm} integrity does not match the tarball"
                ));
            }
        }
    }
    Ok((shasum, format!("sha512-{}", STANDARD.encode(sha512))))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const GZIP: &[u8] = b"\x1f\x8b\x08\x00fake tarball";

    fn document(name: &str, version: &str, data: &[u8], dist: Value) -> Value {
        json!({
            "name": name,
            "versions": {version: {"name": name, "version": version, "dist": dist}},
            "_attachments": {
                format!("{name}-{version}.tgz"): {
                    "content_type": "application/octet-stream",
                    "data": STANDARD.encode(data),
                    "length": data.len()
                }
            }
        })
    }

    #[test]
    fn tarballs_are_paired_with_their_versions_and_verified() {
        let (shasum, integrity) = verify(&json!({}), GZIP).unwrap();
        let name = Name::parse("@team/tool").unwrap();
        let doc = document(
            "@team/tool",
            "1.0.0",
            GZIP,
            json!({"shasum": shasum, "integrity": integrity, "tarball": "http://x/@team/tool/-/@team/tool-1.0.0.tgz"}),
        );
        let found = tarballs(&name, &doc).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].version, "1.0.0");
        assert_eq!(found[0].file, "tool-1.0.0.tgz");
        assert_eq!(found[0].data, GZIP);
    }

    #[test]
    fn missing_digests_are_filled_in() {
        let name = Name::parse("tool").unwrap();
        let found = tarballs(&name, &document("tool", "1.0.0", GZIP, json!({}))).unwrap();
        let dist = &found[0].manifest["dist"];
        assert_eq!(dist["shasum"].as_str().unwrap().len(), 40);
        assert!(dist["integrity"].as_str().unwrap().starts_with("sha512-"));
    }

    #[test]
    fn wrong_digests_and_names_are_refused() {
        let name = Name::parse("tool").unwrap();
        let bad_sha = document("tool", "1.0.0", GZIP, json!({"shasum": "0".repeat(40)}));
        assert!(tarballs(&name, &bad_sha).unwrap_err().contains("shasum"));
        let bad_integrity = document(
            "tool",
            "1.0.0",
            GZIP,
            json!({"integrity": format!("sha512-{}", STANDARD.encode([0u8; 64]))}),
        );
        assert!(
            tarballs(&name, &bad_integrity)
                .unwrap_err()
                .contains("integrity")
        );
        let other = document("other", "1.0.0", GZIP, json!({}));
        assert!(tarballs(&name, &other).is_err());
        let not_semver = document("tool", "1.0", GZIP, json!({}));
        assert!(tarballs(&name, &not_semver).is_err());
        let not_gzip = document("tool", "1.0.0", b"plain text", json!({}));
        assert!(tarballs(&name, &not_gzip).unwrap_err().contains("gzip"));
        let mut wrong_length = document("tool", "1.0.0", GZIP, json!({}));
        wrong_length["_attachments"]["tool-1.0.0.tgz"]["length"] = json!(3);
        assert!(tarballs(&name, &wrong_length).is_err());
        assert!(tarballs(&name, &json!({"versions": {}})).is_err());
    }
}
