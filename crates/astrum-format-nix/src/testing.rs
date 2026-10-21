use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

use crate::hash::nix32;

pub enum Node<'a> {
    File {
        contents: &'a [u8],
        executable: bool,
    },
    Symlink(&'a str),
    Directory(Vec<(&'a str, Node<'a>)>),
}

fn string(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
    out.resize(out.len().next_multiple_of(8), 0);
}

fn object(out: &mut Vec<u8>, node: &Node<'_>) {
    string(out, b"(");
    string(out, b"type");
    match node {
        Node::File {
            contents,
            executable,
        } => {
            string(out, b"regular");
            if *executable {
                string(out, b"executable");
                string(out, b"");
            }
            string(out, b"contents");
            string(out, contents);
        }
        Node::Symlink(target) => {
            string(out, b"symlink");
            string(out, b"target");
            string(out, target.as_bytes());
        }
        Node::Directory(entries) => {
            string(out, b"directory");
            let mut sorted: Vec<&(&str, Node<'_>)> = entries.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            for (name, child) in sorted {
                string(out, b"entry");
                string(out, b"(");
                string(out, b"name");
                string(out, name.as_bytes());
                string(out, b"node");
                object(out, child);
                string(out, b")");
            }
        }
    }
    string(out, b")");
}

pub fn nar(node: &Node<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    string(&mut out, b"nix-archive-1");
    object(&mut out, node);
    out
}

pub fn xz(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    lzma_rs::xz_compress(&mut std::io::BufReader::new(bytes), &mut out)
        .expect("compressing to memory works");
    out
}

pub fn unxz(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    lzma_rs::xz_decompress(&mut std::io::BufReader::new(bytes), &mut out)
        .expect("the test data is xz");
    out
}

pub fn sha256_nix32(bytes: &[u8]) -> String {
    nix32(&Sha256::digest(bytes))
}

pub fn store_hash(seed: &str) -> String {
    nix32(&Sha256::digest(seed.as_bytes())[..20])
}

pub fn signature(key: &str, seed: &str) -> String {
    let mut bytes = Sha256::digest(seed.as_bytes()).to_vec();
    bytes.extend_from_slice(&Sha256::digest(key.as_bytes()));
    format!("{key}:{}", STANDARD.encode(bytes))
}

pub struct StorePath {
    pub hash: String,
    pub name: String,
    pub nar: Vec<u8>,
    pub file: Vec<u8>,
    pub url: String,
    pub compression: &'static str,
    pub references: Vec<String>,
    pub deriver: Option<String>,
    pub signatures: Vec<String>,
}

impl StorePath {
    pub fn new(name: &str, root: &Node<'_>) -> Self {
        let nar = nar(root);
        let file = xz(&nar);
        let hash = store_hash(&format!("{name}:{}", sha256_nix32(&nar)));
        let url = format!("nar/{}.nar.xz", sha256_nix32(&file));
        Self {
            signatures: vec![signature("astrum-test-1", &hash)],
            hash,
            name: name.to_owned(),
            nar,
            file,
            url,
            compression: "xz",
            references: Vec::new(),
            deriver: None,
        }
    }

    pub fn uncompressed(mut self) -> Self {
        self.file = self.nar.clone();
        self.url = format!("nar/{}.nar", sha256_nix32(&self.file));
        self.compression = "none";
        self
    }

    pub fn base(&self) -> String {
        format!("{}-{}", self.hash, self.name)
    }

    pub fn store_path(&self, store_dir: &str) -> String {
        format!("{store_dir}/{}", self.base())
    }

    pub fn nar_path(&self) -> String {
        format!("/{}", self.url)
    }

    pub fn narinfo(&self, store_dir: &str) -> String {
        let mut text = format!(
            "StorePath: {}\nURL: {}\nCompression: {}\nFileHash: sha256:{}\nFileSize: {}\nNarHash: sha256:{}\nNarSize: {}\nReferences: {}\n",
            self.store_path(store_dir),
            self.url,
            self.compression,
            sha256_nix32(&self.file),
            self.file.len(),
            sha256_nix32(&self.nar),
            self.nar.len(),
            self.references.join(" "),
        );
        if let Some(deriver) = &self.deriver {
            text.push_str(&format!("Deriver: {deriver}\n"));
        }
        for signature in &self.signatures {
            text.push_str(&format!("Sig: {signature}\n"));
        }
        text
    }

    pub fn listing(&self) -> String {
        "{\"version\":1,\"root\":{\"type\":\"directory\",\"entries\":{}}}".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::narinfo::NarInfo;

    #[test]
    fn a_nar_follows_the_archive_grammar() {
        let archive = nar(&Node::File {
            contents: b"hi",
            executable: true,
        });
        let words: Vec<&[u8]> = vec![
            b"nix-archive-1",
            b"(",
            b"type",
            b"regular",
            b"executable",
            b"",
            b"contents",
            b"hi",
            b")",
        ];
        let mut expected = Vec::new();
        for word in words {
            string(&mut expected, word);
        }
        assert_eq!(archive, expected);
        assert_eq!(archive.len() % 8, 0);
        let tree = nar(&Node::Directory(vec![
            ("z", Node::Symlink("a")),
            (
                "a",
                Node::File {
                    contents: b"",
                    executable: false,
                },
            ),
        ]));
        let text = String::from_utf8_lossy(&tree);
        assert!(text.find("symlink").unwrap() > text.find("regular").unwrap());
    }

    #[test]
    fn a_built_store_path_has_a_narinfo_that_parses() {
        let mut path = StorePath::new(
            "hello-2.12.1",
            &Node::Directory(vec![(
                "bin",
                Node::Directory(vec![(
                    "hello",
                    Node::File {
                        contents: b"#!/bin/sh\necho hello\n",
                        executable: true,
                    },
                )]),
            )]),
        );
        path.deriver = Some(format!("{}-hello-2.12.1.drv", store_hash("drv")));
        let info = NarInfo::parse(path.narinfo("/nix/store").as_bytes(), "/nix/store").unwrap();
        assert_eq!(info.hash, path.hash);
        assert_eq!(info.file_size, Some(path.file.len() as u64));
        assert_eq!(info.nar_size, path.nar.len() as u64);
        assert_eq!(info.nar_path(), Some(path.nar_path()));
        assert_eq!(info.key_names(), ["astrum-test-1"]);
        let mut back = Vec::new();
        lzma_rs::xz_decompress(&mut path.file.as_slice(), &mut back).unwrap();
        assert_eq!(back, path.nar);
    }
}
