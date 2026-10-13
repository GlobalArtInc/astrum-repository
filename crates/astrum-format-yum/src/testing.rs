use std::collections::BTreeMap;
use std::io::Write;

use crate::rpm::{TYPE_I18NSTRING, TYPE_INT16, TYPE_INT32, TYPE_STRING, TYPE_STRING_ARRAY, tag};

#[derive(Clone, Debug)]
enum Value {
    Strings(Vec<String>),
    Int32(Vec<u32>),
    Int16(Vec<u16>),
}

#[derive(Default)]
struct HeaderWriter {
    entries: BTreeMap<u32, (u32, Value)>,
}

impl HeaderWriter {
    fn string(&mut self, tag: u32, value: &str) {
        self.entries
            .insert(tag, (TYPE_STRING, Value::Strings(vec![value.to_owned()])));
    }

    fn i18n(&mut self, tag: u32, value: &str) {
        self.entries.insert(
            tag,
            (TYPE_I18NSTRING, Value::Strings(vec![value.to_owned()])),
        );
    }

    fn strings(&mut self, tag: u32, values: Vec<String>) {
        self.entries
            .insert(tag, (TYPE_STRING_ARRAY, Value::Strings(values)));
    }

    fn int32(&mut self, tag: u32, values: Vec<u32>) {
        self.entries.insert(tag, (TYPE_INT32, Value::Int32(values)));
    }

    fn int16(&mut self, tag: u32, values: Vec<u16>) {
        self.entries.insert(tag, (TYPE_INT16, Value::Int16(values)));
    }

    fn bytes(&self) -> Vec<u8> {
        let mut index = Vec::new();
        let mut data: Vec<u8> = Vec::new();
        for (tag, (kind, value)) in &self.entries {
            let alignment = match *kind {
                TYPE_INT32 => 4,
                TYPE_INT16 => 2,
                _ => 1,
            };
            while !data.len().is_multiple_of(alignment) {
                data.push(0);
            }
            let offset = data.len() as u32;
            let count = match value {
                Value::Strings(values) => {
                    for value in values {
                        data.extend_from_slice(value.as_bytes());
                        data.push(0);
                    }
                    values.len()
                }
                Value::Int32(values) => {
                    for value in values {
                        data.extend_from_slice(&value.to_be_bytes());
                    }
                    values.len()
                }
                Value::Int16(values) => {
                    for value in values {
                        data.extend_from_slice(&value.to_be_bytes());
                    }
                    values.len()
                }
            };
            for number in [*tag, *kind, offset, count as u32] {
                index.extend_from_slice(&number.to_be_bytes());
            }
        }
        let mut out = vec![0x8E, 0xAD, 0xE8, 0x01, 0, 0, 0, 0];
        out.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(&index);
        out.extend_from_slice(&data);
        out
    }
}

type Dependency = (String, u32, String);

/// Builds small but well-formed RPM packages for tests.
#[derive(Clone, Debug)]
pub struct TestRpm {
    name: String,
    version: String,
    release: String,
    arch: String,
    epoch: Option<u32>,
    source: bool,
    summary: String,
    description: String,
    build_time: u32,
    files: Vec<(String, u16, u32, u32)>,
    dependencies: BTreeMap<u32, Vec<Dependency>>,
    changelogs: Vec<(u32, String, String)>,
}

impl TestRpm {
    pub fn new(name: &str, version: &str, release: &str) -> Self {
        Self {
            name: name.to_owned(),
            version: version.to_owned(),
            release: release.to_owned(),
            arch: "x86_64".to_owned(),
            epoch: None,
            source: false,
            summary: format!("The {name} package"),
            description: format!("The {name} package for tests."),
            build_time: 1_700_000_000,
            files: Vec::new(),
            dependencies: BTreeMap::new(),
            changelogs: Vec::new(),
        }
    }

    pub fn arch(mut self, arch: &str) -> Self {
        self.arch = arch.to_owned();
        self
    }

    pub fn epoch(mut self, epoch: u32) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn source(mut self) -> Self {
        self.source = true;
        self
    }

    pub fn summary(mut self, summary: &str) -> Self {
        self.summary = summary.to_owned();
        self
    }

    pub fn file(mut self, path: &str, size: u32) -> Self {
        self.files.push((path.to_owned(), 0o100644, 0, size));
        self
    }

    pub fn directory(mut self, path: &str) -> Self {
        self.files.push((path.to_owned(), 0o040755, 0, 0));
        self
    }

    pub fn ghost(mut self, path: &str) -> Self {
        self.files.push((path.to_owned(), 0o100644, 1 << 6, 0));
        self
    }

    fn dependency(mut self, name_tag: u32, name: &str, flags: u32, evr: &str) -> Self {
        self.dependencies.entry(name_tag).or_default().push((
            name.to_owned(),
            flags,
            evr.to_owned(),
        ));
        self
    }

    pub fn provides(self, name: &str, flags: u32, evr: &str) -> Self {
        self.dependency(tag::PROVIDENAME, name, flags, evr)
    }

    pub fn requires(self, name: &str, flags: u32, evr: &str) -> Self {
        self.dependency(tag::REQUIRENAME, name, flags, evr)
    }

    pub fn conflicts(self, name: &str, flags: u32, evr: &str) -> Self {
        self.dependency(tag::CONFLICTNAME, name, flags, evr)
    }

    pub fn obsoletes(self, name: &str, flags: u32, evr: &str) -> Self {
        self.dependency(tag::OBSOLETENAME, name, flags, evr)
    }

    pub fn changelog(mut self, time: u32, author: &str, text: &str) -> Self {
        self.changelogs
            .push((time, author.to_owned(), text.to_owned()));
        self
    }

    pub fn file_name(&self) -> String {
        format!(
            "{}-{}-{}.{}.rpm",
            self.name,
            self.version,
            self.release,
            if self.source { "src" } else { &self.arch }
        )
    }

    fn dependency_tags(header: &mut HeaderWriter, entries: &[Dependency], tags: [u32; 3]) {
        if entries.is_empty() {
            return;
        }
        header.strings(
            tags[0],
            entries.iter().map(|entry| entry.0.clone()).collect(),
        );
        header.int32(tags[1], entries.iter().map(|entry| entry.1).collect());
        header.strings(
            tags[2],
            entries.iter().map(|entry| entry.2.clone()).collect(),
        );
    }

    pub fn build(&self) -> Vec<u8> {
        let mut payload = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        payload
            .write_all(b"0707010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b00000000TRAILER!!!\0\0\0\0")
            .expect("writing to memory never fails");
        let payload = payload.finish().expect("writing to memory never fails");
        let mut header = HeaderWriter::default();
        header.string(tag::NAME, &self.name);
        header.string(tag::VERSION, &self.version);
        header.string(tag::RELEASE, &self.release);
        if let Some(epoch) = self.epoch {
            header.int32(tag::EPOCH, vec![epoch]);
        }
        header.i18n(tag::SUMMARY, &self.summary);
        header.i18n(tag::DESCRIPTION, &self.description);
        header.int32(tag::BUILDTIME, vec![self.build_time]);
        header.string(tag::BUILDHOST, "build.example.com");
        header.int32(tag::SIZE, vec![self.files.iter().map(|file| file.3).sum()]);
        header.string(tag::LICENSE, "MIT");
        header.i18n(tag::GROUP, "Unspecified");
        header.string(tag::URL, &format!("https://example.org/{}", self.name));
        header.string(tag::OS, "linux");
        header.string(tag::ARCH, &self.arch);
        match self.source {
            true => header.int32(tag::SOURCEPACKAGE, vec![1]),
            false => header.string(
                tag::SOURCERPM,
                &format!("{}-{}-{}.src.rpm", self.name, self.version, self.release),
            ),
        }
        header.string(tag::PAYLOADFORMAT, "cpio");
        header.string(tag::PAYLOADCOMPRESSOR, "gzip");
        if !self.files.is_empty() {
            let mut dirs: Vec<String> = Vec::new();
            let mut bases = Vec::new();
            let mut indexes = Vec::new();
            for (path, _, _, _) in &self.files {
                let slash = path.rfind('/').map(|at| at + 1).unwrap_or(0);
                let dir = path[..slash].to_owned();
                let index = match dirs.iter().position(|known| *known == dir) {
                    Some(index) => index,
                    None => {
                        dirs.push(dir);
                        dirs.len() - 1
                    }
                };
                indexes.push(index as u32);
                bases.push(path[slash..].to_owned());
            }
            header.int32(
                tag::FILESIZES,
                self.files.iter().map(|file| file.3).collect(),
            );
            header.int16(
                tag::FILEMODES,
                self.files.iter().map(|file| file.1).collect(),
            );
            header.int32(
                tag::FILEFLAGS,
                self.files.iter().map(|file| file.2).collect(),
            );
            header.int32(tag::DIRINDEXES, indexes);
            header.strings(tag::BASENAMES, bases);
            header.strings(tag::DIRNAMES, dirs);
        }
        let evr = format!(
            "{}{}-{}",
            self.epoch
                .map(|epoch| format!("{epoch}:"))
                .unwrap_or_default(),
            self.version,
            self.release
        );
        let mut provides = self
            .dependencies
            .get(&tag::PROVIDENAME)
            .cloned()
            .unwrap_or_default();
        if !self.source {
            provides.push((self.name.clone(), 8, evr.clone()));
            provides.push((format!("{}(x86-64)", self.name), 8, evr));
        }
        Self::dependency_tags(
            &mut header,
            &provides,
            [tag::PROVIDENAME, tag::PROVIDEFLAGS, tag::PROVIDEVERSION],
        );
        let mut requires = self
            .dependencies
            .get(&tag::REQUIRENAME)
            .cloned()
            .unwrap_or_default();
        requires.push((
            "rpmlib(CompressedFileNames)".to_owned(),
            (1 << 24) | 8 | 2,
            "3.0.4-1".to_owned(),
        ));
        Self::dependency_tags(
            &mut header,
            &requires,
            [tag::REQUIRENAME, tag::REQUIREFLAGS, tag::REQUIREVERSION],
        );
        for tags in [
            [tag::CONFLICTNAME, tag::CONFLICTFLAGS, tag::CONFLICTVERSION],
            [tag::OBSOLETENAME, tag::OBSOLETEFLAGS, tag::OBSOLETEVERSION],
        ] {
            if let Some(entries) = self.dependencies.get(&tags[0]) {
                Self::dependency_tags(&mut header, entries, tags);
            }
        }
        if !self.changelogs.is_empty() {
            header.int32(
                tag::CHANGELOGTIME,
                self.changelogs.iter().map(|entry| entry.0).collect(),
            );
            header.strings(
                tag::CHANGELOGNAME,
                self.changelogs
                    .iter()
                    .map(|entry| entry.1.clone())
                    .collect(),
            );
            header.strings(
                tag::CHANGELOGTEXT,
                self.changelogs
                    .iter()
                    .map(|entry| entry.2.clone())
                    .collect(),
            );
        }
        let header = header.bytes();
        let mut signature = HeaderWriter::default();
        signature.int32(tag::SIG_SIZE, vec![(header.len() + payload.len()) as u32]);
        signature.int32(tag::SIG_PAYLOADSIZE, vec![124]);
        let signature = signature.bytes();
        let mut lead = vec![0u8; 96];
        lead[..4].copy_from_slice(&[0xED, 0xAB, 0xEE, 0xDB]);
        lead[4] = 3;
        lead[7] = u8::from(self.source);
        lead[9] = 1;
        let nevr = format!("{}-{}-{}", self.name, self.version, self.release);
        let length = nevr.len().min(65);
        lead[10..10 + length].copy_from_slice(&nevr.as_bytes()[..length]);
        lead[77] = 1;
        lead[79] = 5;
        let mut out = lead;
        out.extend_from_slice(&signature);
        out.resize(out.len() + (8 - signature.len() % 8) % 8, 0);
        out.extend_from_slice(&header);
        out.extend_from_slice(&payload);
        out
    }
}

pub fn signing_key(passphrase: Option<&str>) -> String {
    use pgp::composed::{ArmorOptions, KeyType, SecretKeyParamsBuilder};
    let mut params = SecretKeyParamsBuilder::default();
    params
        .key_type(KeyType::Ed25519Legacy)
        .can_certify(true)
        .can_sign(true)
        .primary_user_id("Astrum Test <test@example.org>".into());
    if let Some(passphrase) = passphrase {
        params.passphrase(Some(passphrase.to_owned()));
    }
    let key = params
        .build()
        .expect("the test key parameters are valid")
        .generate(rand08::thread_rng())
        .expect("a test key can be generated");
    key.to_armored_string(ArmorOptions::default())
        .expect("a test key can be armored")
}
