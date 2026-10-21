use std::io::Write;
use std::sync::OnceLock;

use flate2::Compression;
use flate2::write::GzEncoder;
use prost::Message;
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::pkcs8::{DecodePublicKey, EncodePrivateKey, LineEnding};
use rsa::signature::Verifier;
use sha2::{Digest, Sha256, Sha512};

use crate::keys::RegistryKey;
use crate::proto::{self, Names, Package, Versions};
use crate::registry::Record;
use crate::terms::{self, Term};

#[derive(Clone, Debug)]
pub struct Spec {
    pub name: String,
    pub version: String,
    pub app: Option<String>,
    pub description: String,
    pub requirements: Vec<(String, String, bool, Option<String>)>,
    pub files: Vec<(String, Vec<u8>)>,
    pub format_version: String,
    pub checksum: Option<String>,
    pub metadata: Option<String>,
}

pub struct Built {
    pub bytes: Vec<u8>,
    pub metadata: String,
    pub inner_checksum: [u8; 32],
    pub outer_checksum: [u8; 32],
}

fn bin(text: &str) -> Term {
    Term::Binary(text.as_bytes().to_vec())
}

fn pair(key: &str, value: Term) -> Term {
    Term::Tuple(vec![bin(key), value])
}

fn tar(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, data) in files {
        let mut header = tar::Header::new_ustar();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(1_700_000_000);
        header.set_cksum();
        builder
            .append_data(&mut header, path, *data)
            .expect("a test tarball can be written");
    }
    builder.into_inner().expect("a test tarball can be written")
}

impl Spec {
    pub fn new(name: &str, version: &str) -> Spec {
        Spec {
            name: name.to_owned(),
            version: version.to_owned(),
            app: None,
            description: format!("The {name} package — for the tests"),
            requirements: Vec::new(),
            files: vec![
                (
                    format!("lib/{name}.ex"),
                    format!("defmodule Hello do\n  def version, do: \"{version}\"\nend\n")
                        .into_bytes(),
                ),
                (
                    "mix.exs".to_owned(),
                    b"defmodule Hello.MixProject do\nend\n".to_vec(),
                ),
            ],
            format_version: crate::tarball::FORMAT_VERSION.to_owned(),
            checksum: None,
            metadata: None,
        }
    }

    pub fn requirement(mut self, name: &str, requirement: &str, optional: bool) -> Spec {
        self.requirements.push((
            name.to_owned(),
            requirement.to_owned(),
            optional,
            Some("hexpm".to_owned()),
        ));
        self
    }

    pub fn optional(self, name: &str, requirement: &str) -> Spec {
        self.requirement(name, requirement, true)
    }

    pub fn local(mut self, name: &str, requirement: &str, repository: &str) -> Spec {
        self.requirements.push((
            name.to_owned(),
            requirement.to_owned(),
            false,
            Some(repository.to_owned()),
        ));
        self
    }

    pub fn metadata_text(&self) -> String {
        if let Some(metadata) = &self.metadata {
            return metadata.clone();
        }
        let app = self.app.clone().unwrap_or_else(|| self.name.clone());
        let requirements = self
            .requirements
            .iter()
            .map(|(name, requirement, optional, repository)| {
                let mut fields = vec![
                    pair("app", bin(name)),
                    pair("name", bin(name)),
                    pair("optional", Term::Atom(optional.to_string())),
                ];
                if let Some(repository) = repository {
                    fields.push(pair("repository", bin(repository)));
                }
                fields.push(pair("requirement", bin(requirement)));
                Term::List(fields)
            })
            .collect();
        let files = self.files.iter().map(|(path, _)| bin(path)).collect();
        let entries = [
            pair("app", bin(&app)),
            pair("build_tools", Term::List(vec![bin("mix")])),
            pair("description", bin(&self.description)),
            pair("elixir", bin("~> 1.15")),
            pair("files", Term::List(files)),
            pair("licenses", Term::List(vec![bin("Apache-2.0")])),
            pair(
                "links",
                Term::List(vec![Term::Tuple(vec![
                    bin("GitHub"),
                    bin(&format!("https://example.org/{}", self.name)),
                ])]),
            ),
            pair("name", bin(&self.name)),
            pair("requirements", Term::List(requirements)),
            pair("version", bin(&self.version)),
        ];
        entries
            .iter()
            .map(|entry| format!("{}.\n", terms::print(entry)))
            .collect()
    }

    pub fn build(&self) -> Built {
        let metadata = self.metadata_text();
        let files: Vec<(&str, &[u8])> = self
            .files
            .iter()
            .map(|(path, data)| (path.as_str(), data.as_slice()))
            .collect();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&tar(&files))
            .expect("writing to memory does not fail");
        let contents = encoder.finish().expect("writing to memory does not fail");
        let mut digest = Sha256::new();
        digest.update(self.format_version.as_bytes());
        digest.update(metadata.as_bytes());
        digest.update(&contents);
        let inner_checksum: [u8; 32] = digest.finalize().into();
        let checksum = self
            .checksum
            .clone()
            .unwrap_or_else(|| hex::encode_upper(inner_checksum));
        let bytes = tar(&[
            ("VERSION", self.format_version.as_bytes()),
            ("CHECKSUM", checksum.as_bytes()),
            ("metadata.config", metadata.as_bytes()),
            ("contents.tar.gz", &contents),
        ]);
        let outer_checksum = Sha256::digest(&bytes).into();
        Built {
            bytes,
            metadata,
            inner_checksum,
            outer_checksum,
        }
    }
}

impl Built {
    pub fn record(&self, updated_at: i64) -> Record {
        let read = crate::tarball::read(&self.bytes).expect("a built tarball can be read");
        Record {
            name: read.metadata.name,
            version: read.metadata.version,
            inner_checksum: read.inner_checksum.to_vec(),
            outer_checksum: read.outer_checksum.to_vec(),
            requirements: read.metadata.requirements,
            retired: None,
            updated_at,
        }
    }
}

pub fn signed(key: &RegistryKey, message: &impl Message) -> Vec<u8> {
    key.sign(message.encode_to_vec())
}

pub fn rsa_key() -> RsaPrivateKey {
    static KEY: OnceLock<RsaPrivateKey> = OnceLock::new();
    KEY.get_or_init(|| {
        RsaPrivateKey::new(&mut rand_core06::OsRng, 2048).expect("a test key can be generated")
    })
    .clone()
}

pub fn private_key_pem() -> String {
    rsa_key()
        .to_pkcs8_pem(LineEnding::LF)
        .expect("a test key can be written")
        .to_string()
}

pub fn verified(resource: &[u8], public_key_pem: &str) -> Option<Vec<u8>> {
    let signed = proto::open(resource).ok()?;
    let public = rsa::RsaPublicKey::from_public_key_pem(public_key_pem).ok()?;
    let signature = Signature::try_from(signed.signature.as_deref()?).ok()?;
    VerifyingKey::<Sha512>::new(public)
        .verify(&signed.payload, &signature)
        .ok()?;
    Some(signed.payload)
}

pub fn names(resource: &[u8], public_key_pem: &str) -> Option<Names> {
    Names::decode(verified(resource, public_key_pem)?.as_slice()).ok()
}

pub fn versions(resource: &[u8], public_key_pem: &str) -> Option<Versions> {
    Versions::decode(verified(resource, public_key_pem)?.as_slice()).ok()
}

pub fn package(resource: &[u8], public_key_pem: &str) -> Option<Package> {
    Package::decode(verified(resource, public_key_pem)?.as_slice()).ok()
}
