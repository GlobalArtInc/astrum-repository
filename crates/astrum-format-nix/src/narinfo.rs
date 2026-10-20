use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::hash::Hash;
use crate::names;

pub const UNKNOWN_DERIVER: &str = "unknown-deriver";
const DEFAULT_COMPRESSION: &str = "bzip2";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NarInfo {
    pub store_path: String,
    pub hash: String,
    pub name: String,
    pub url: String,
    pub compression: String,
    pub file_hash: Option<Hash>,
    pub file_size: Option<u64>,
    pub nar_hash: Hash,
    pub nar_size: u64,
    pub references: Vec<String>,
    pub deriver: Option<String>,
    pub system: Option<String>,
    pub signatures: Vec<String>,
    pub ca: Option<String>,
}

fn single<'a>(fields: &[(&'a str, &'a str)], key: &str) -> Result<Option<&'a str>, String> {
    let mut found = fields.iter().filter(|(name, _)| *name == key);
    let first = found.next().map(|(_, value)| *value);
    match found.next() {
        Some(_) => Err(format!("{key} is given more than once")),
        None => Ok(first),
    }
}

fn required<'a>(fields: &[(&'a str, &'a str)], key: &str) -> Result<&'a str, String> {
    single(fields, key)?
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{key} is missing"))
}

fn size(key: &str, value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("{key} must be a number of bytes, not {value}"))
}

fn signature(value: &str) -> Result<String, String> {
    let invalid = || format!("Sig must be <key name>:<base64 signature>, not {value}");
    let (key, encoded) = value.split_once(':').ok_or_else(invalid)?;
    if key.is_empty() || key.chars().any(char::is_whitespace) {
        return Err(invalid());
    }
    match STANDARD.decode(encoded) {
        Ok(bytes) if !bytes.is_empty() => Ok(value.to_owned()),
        _ => Err(invalid()),
    }
}

fn content_address(value: &str) -> Result<String, String> {
    let rest = value
        .strip_prefix("fixed:")
        .or_else(|| value.strip_prefix("text:"))
        .ok_or_else(|| format!("CA must start with fixed: or text:, not {value}"))?;
    let digest = rest
        .strip_prefix("r:")
        .or_else(|| rest.strip_prefix("git:"))
        .unwrap_or(rest);
    Hash::parse(digest)
        .map(|_| value.to_owned())
        .map_err(|error| format!("CA holds no valid hash: {error}"))
}

impl NarInfo {
    pub fn parse(bytes: &[u8], store_dir: &str) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "the narinfo is not UTF-8 text")?;
        let mut fields = Vec::new();
        for (number, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| format!("line {} is not Key: value", number + 1))?;
            fields.push((key.trim(), value.trim()));
        }
        let store_path = required(&fields, "StorePath")?;
        let base = store_path
            .strip_prefix(store_dir)
            .and_then(|rest| rest.strip_prefix('/'))
            .ok_or_else(|| format!("StorePath {store_path} is not in the store {store_dir}"))?;
        let (hash, name) = names::split_base(base)
            .ok_or_else(|| format!("StorePath {store_path} is not a valid store path"))?;
        let url = required(&fields, "URL")?;
        if url.chars().any(char::is_whitespace) {
            return Err(format!("URL {url} holds white space"));
        }
        let compression = single(&fields, "Compression")?
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_COMPRESSION);
        if !compression
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err(format!("Compression {compression} is not a method name"));
        }
        let file_hash = single(&fields, "FileHash")?
            .map(|value| Hash::parse(value).map_err(|error| format!("FileHash: {error}")))
            .transpose()?;
        let file_size = single(&fields, "FileSize")?
            .map(|value| size("FileSize", value))
            .transpose()?;
        let nar_hash = Hash::parse(required(&fields, "NarHash")?)
            .map_err(|error| format!("NarHash: {error}"))?;
        let nar_size = size("NarSize", required(&fields, "NarSize")?)?;
        let references: Vec<String> = single(&fields, "References")?
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        if let Some(bad) = references.iter().find(|base| !names::valid_base(base)) {
            return Err(format!("the reference {bad} is not a store path base name"));
        }
        let deriver = match single(&fields, "Deriver")? {
            None | Some("") | Some(UNKNOWN_DERIVER) => None,
            Some(base) if names::valid_base(base) => Some(base.to_owned()),
            Some(other) => return Err(format!("Deriver {other} is not a store path base name")),
        };
        let signatures = fields
            .iter()
            .filter(|(key, _)| *key == "Sig")
            .map(|(_, value)| signature(value))
            .collect::<Result<Vec<_>, _>>()?;
        let ca = single(&fields, "CA")?
            .filter(|value| !value.is_empty())
            .map(content_address)
            .transpose()?;
        Ok(Self {
            store_path: store_path.to_owned(),
            hash: hash.to_owned(),
            name: name.to_owned(),
            url: url.to_owned(),
            compression: compression.to_owned(),
            file_hash,
            file_size,
            nar_hash,
            nar_size,
            references,
            deriver,
            system: single(&fields, "System")?
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            signatures,
            ca,
        })
    }

    pub fn relative_url(&self) -> Option<String> {
        relative(&self.url)
    }

    pub fn nar_path(&self) -> Option<String> {
        let relative = self.relative_url()?;
        let file = relative.strip_prefix("nar/")?;
        names::valid_nar_file(file).then(|| format!("/{relative}"))
    }

    pub fn key_names(&self) -> Vec<String> {
        self.signatures
            .iter()
            .filter_map(|signature| signature.split_once(':'))
            .map(|(key, _)| key.to_owned())
            .collect()
    }

    pub fn component_attributes(&self) -> Value {
        let mut attributes = json!({
            "storePath": self.store_path,
            "narHash": self.nar_hash.nix32(),
            "narSize": self.nar_size,
            "references": self.references,
            "signatures": self.key_names(),
        });
        if let Some(deriver) = &self.deriver {
            attributes["deriver"] = json!(deriver);
        }
        if let Some(ca) = &self.ca {
            attributes["ca"] = json!(ca);
        }
        if let Some(system) = &self.system {
            attributes["system"] = json!(system);
        }
        json!({ crate::ID: attributes })
    }

    pub fn asset_attributes(&self) -> Value {
        let mut attributes = json!({
            "storePath": self.store_path,
            "url": self.url,
            "compression": self.compression,
            "narHash": self.nar_hash.nix32(),
            "narSize": self.nar_size,
        });
        if let Some(path) = self.nar_path() {
            attributes["nar"] = json!(path);
        }
        if let Some(hash) = &self.file_hash {
            attributes["fileHash"] = json!(hash.nix32());
        }
        if let Some(size) = self.file_size {
            attributes["fileSize"] = json!(size);
        }
        json!({ crate::ID: attributes })
    }
}

pub fn relative(url: &str) -> Option<String> {
    if url.contains("://") || url.starts_with('/') {
        return None;
    }
    let path = url.split(['?', '#']).next().unwrap_or_default();
    (!path.is_empty()
        && !path
            .split('/')
            .any(|segment| segment == ".." || segment == "."))
    .then(|| path.to_owned())
}

pub fn rewrite_url(bytes: &[u8], from: &str) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(bytes).ok()?;
    let base = format!("{}/", from.trim_end_matches('/'));
    let mut changed = false;
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let rewritten = line.split_once(':').and_then(|(key, value)| {
            (key.trim() == "URL")
                .then(|| value.trim().strip_prefix(&base))
                .flatten()
                .filter(|rest| !rest.is_empty())
                .map(|rest| {
                    let ending = if line.ends_with('\n') { "\n" } else { "" };
                    format!("URL: {rest}{ending}")
                })
        });
        match rewritten {
            Some(line) => {
                changed = true;
                out.push_str(&line);
            }
            None => out.push_str(line),
        }
    }
    changed.then(|| out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUBLIC: &str = "StorePath: /nix/store/mw9ls4zxx884y7yq0a04ildfdjfy1vn2-nix-2.34.9
URL: nar/0xfhhhb46a15v23pvkw97a5h578d1qkl34w351p5xxh52f1jbhyl.nar.zst
Compression: zstd
FileHash: sha256:1y4rml10kv2flxavf5lszm3wn907cp6gzcpp48szvpf0sf9vr772
FileSize: 955
NarHash: sha256:0xfhhhb46a15v23pvkw97a5h578d1qkl34w351p5xxh52f1jbhyl
NarSize: 8848
References: bxzi311gg1nqjygyn05wx6x7cbs7i52c-nix-nswrapper-2.34.9 ssswpqihnyi8231b9r515k2hmpnk0h1n-nix-2.34.9
Deriver: lj5yrw204ccjd7x7km8pjfz14q7mg8zc-nix-2.34.9.drv
Sig: cache.nixos.org-1:DKyhQMDhBiUD9GLjd9FzgMR+Z8w9HATdH0fQf5CVsJ1qEhSUNyu6zYUjFJWL/0mgmo7Jnf2c3WCViOsrFlSICA==
";

    #[test]
    fn a_narinfo_of_a_public_cache_is_read() {
        let info = NarInfo::parse(PUBLIC.as_bytes(), "/nix/store").unwrap();
        assert_eq!(info.hash, "mw9ls4zxx884y7yq0a04ildfdjfy1vn2");
        assert_eq!(info.name, "nix-2.34.9");
        assert_eq!(info.compression, "zstd");
        assert_eq!(info.file_size, Some(955));
        assert_eq!(
            info.file_hash.as_ref().unwrap().hex(),
            "e29cbc93d3c0ddfd3522f7b2ffcc650724cb47fd9a16b755a74eec0902ad99f8"
        );
        assert_eq!(info.nar_size, 8848);
        assert_eq!(info.references.len(), 2);
        assert_eq!(
            info.deriver.as_deref(),
            Some("lj5yrw204ccjd7x7km8pjfz14q7mg8zc-nix-2.34.9.drv")
        );
        assert_eq!(info.key_names(), ["cache.nixos.org-1"]);
        assert_eq!(
            info.nar_path().as_deref(),
            Some("/nar/0xfhhhb46a15v23pvkw97a5h578d1qkl34w351p5xxh52f1jbhyl.nar.zst")
        );
        assert_eq!(
            info.asset_attributes()["nix"]["fileHash"],
            "sha256:1y4rml10kv2flxavf5lszm3wn907cp6gzcpp48szvpf0sf9vr772"
        );
    }

    #[test]
    fn defaults_and_mistakes() {
        let minimal = "StorePath: /nix/store/mw9ls4zxx884y7yq0a04ildfdjfy1vn2-x\nURL: nar/abc.nar.bz2?sha256=1\nNarHash: sha256-FePFYIlMuycIXPZbWi7LGEiMmZSX9FMbaQenWBzm1Sc=\nNarSize: 8\nDeriver: unknown-deriver\nCA: fixed:r:sha256:1lr187v6dck1rjh2j6svpikcfz53wyl3qrlcbb405zlh13x0khhh\n";
        let info = NarInfo::parse(minimal.as_bytes(), "/nix/store").unwrap();
        assert_eq!(info.compression, "bzip2");
        assert_eq!(info.deriver, None);
        assert!(info.references.is_empty());
        assert_eq!(info.nar_path().as_deref(), Some("/nar/abc.nar.bz2"));
        assert!(NarInfo::parse(minimal.as_bytes(), "/opt/store").is_err());
        let broken = [
            minimal.replace(
                "StorePath: /nix/store/mw9ls4zxx884y7yq0a04ildfdjfy1vn2-x",
                "",
            ),
            minimal.replace("NarSize: 8", "NarSize: eight"),
            minimal.replace("NarSize: 8", "NarSize: 8\nNarSize: 9"),
            minimal.replace("Deriver: unknown-deriver", "Deriver: nope"),
            minimal.replace("CA: fixed:r:", "CA: flat:"),
            format!("{minimal}References: not-a-path\n"),
            format!("{minimal}Sig: key\n"),
            format!("{minimal}garbage\n"),
            minimal.replace(
                "mw9ls4zxx884y7yq0a04ildfdjfy1vn2-x",
                "mw9ls4zxx884y7yq0a04ildfdjfy1vn2-.x",
            ),
        ];
        for text in broken {
            assert!(
                NarInfo::parse(text.as_bytes(), "/nix/store").is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn only_relative_urls_name_a_nar_of_the_repository() {
        assert_eq!(relative("nar/x.nar"), Some("nar/x.nar".to_owned()));
        assert_eq!(relative("https://other.example/nar/x.nar"), None);
        assert_eq!(relative("/nar/x.nar"), None);
        assert_eq!(relative("../nar/x.nar"), None);
    }

    #[test]
    fn urls_under_the_remote_become_relative() {
        let text = "StorePath: /nix/store/x\nURL: https://cache.example/sub/nar/a.nar.xz\nCompression: xz\n";
        let rewritten = rewrite_url(text.as_bytes(), "https://cache.example/sub/").unwrap();
        assert_eq!(
            String::from_utf8(rewritten).unwrap(),
            "StorePath: /nix/store/x\nURL: nar/a.nar.xz\nCompression: xz\n"
        );
        assert_eq!(rewrite_url(text.as_bytes(), "https://other.example"), None);
        assert_eq!(
            rewrite_url(b"URL: nar/a.nar.xz\n", "https://cache.example"),
            None
        );
    }
}
