use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};

use flate2::Compression;
use flate2::write::GzEncoder;
use quick_xml::events::Event;
use sha2::{Digest, Sha256};

use crate::rpm::{Dependency, Package, RpmFile};
use crate::xml::{self, COMMON_NS, FILELISTS_NS, OTHER_NS, REPO_NS, RPM_NS};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MetadataType {
    Primary,
    Filelists,
    Other,
}

impl MetadataType {
    pub const ALL: [MetadataType; 3] = [Self::Primary, Self::Filelists, Self::Other];

    pub fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Filelists => "filelists",
            Self::Other => "other",
        }
    }

    pub fn root(self) -> &'static str {
        match self {
            Self::Primary => "metadata",
            Self::Filelists => "filelists",
            Self::Other => "otherdata",
        }
    }

    pub fn namespace(self) -> &'static str {
        match self {
            Self::Primary => COMMON_NS,
            Self::Filelists => FILELISTS_NS,
            Self::Other => OTHER_NS,
        }
    }

    fn declarations(self) -> String {
        match self {
            Self::Primary => format!(" xmlns=\"{COMMON_NS}\" xmlns:rpm=\"{RPM_NS}\""),
            other => format!(" xmlns=\"{}\"", other.namespace()),
        }
    }
}

struct Digesting<W> {
    inner: W,
    digest: Sha256,
    count: u64,
}

impl<W> Digesting<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            digest: Sha256::new(),
            count: 0,
        }
    }
}

impl<W: Write> Write for Digesting<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.digest.update(&buf[..written]);
        self.count += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub struct MetadataWriter {
    kind: MetadataType,
    out: Digesting<GzEncoder<Digesting<BufWriter<File>>>>,
}

pub struct Generated {
    pub kind: MetadataType,
    pub file: File,
    pub data: RepomdData,
}

impl Generated {
    pub fn file_name(&self) -> &str {
        crate::paths::file_name(&self.data.location)
    }
}

impl MetadataWriter {
    pub fn new(kind: MetadataType, packages: u64) -> io::Result<Self> {
        let file = tempfile::tempfile()?;
        let compressed = Digesting::new(BufWriter::with_capacity(64 * 1024, file));
        let mut out = Digesting::new(GzEncoder::new(compressed, Compression::default()));
        writeln!(
            out,
            "{}<{}{} packages=\"{packages}\">",
            xml::DECLARATION,
            kind.root(),
            kind.declarations()
        )?;
        Ok(Self { kind, out })
    }

    pub fn raw(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.out.write_all(bytes)
    }

    pub fn package(&mut self, entry: &Entry<'_>) -> io::Result<()> {
        let mut text = match self.kind {
            MetadataType::Primary => primary(entry),
            MetadataType::Filelists => filelists(entry),
            MetadataType::Other => other(entry),
        };
        text.push('\n');
        self.out.write_all(text.as_bytes())
    }

    pub fn finish(mut self, timestamp: i64) -> io::Result<Generated> {
        writeln!(self.out, "</{}>", self.kind.root())?;
        let open_checksum = hex::encode(self.out.digest.finalize_reset());
        let open_size = self.out.count;
        let compressed = self.out.inner.finish()?;
        let checksum = hex::encode(compressed.digest.finalize());
        let size = compressed.count;
        let mut file = compressed
            .inner
            .into_inner()
            .map_err(|error| error.into_error())?;
        file.seek(SeekFrom::Start(0))?;
        let name = self.kind.name();
        Ok(Generated {
            kind: self.kind,
            file,
            data: RepomdData {
                kind: name.to_owned(),
                checksum_type: "sha256".to_owned(),
                checksum: checksum.clone(),
                open_checksum_type: Some("sha256".to_owned()),
                open_checksum: Some(open_checksum),
                location: format!("repodata/{checksum}-{name}.xml.gz"),
                timestamp: Some(timestamp),
                size: Some(size),
                open_size: Some(open_size),
                location_base: None,
            },
        })
    }
}

pub struct Entry<'a> {
    pub rpm: &'a Package,
    pub location: &'a str,
    pub checksum: &'a str,
    pub size: u64,
    pub file_time: i64,
}

fn element(out: &mut String, name: &str, value: Option<&str>) {
    match value.filter(|value| !value.is_empty()) {
        Some(value) => {
            let _ = write!(out, "<{name}>{}</{name}>", xml::text(value));
        }
        None => {
            let _ = write!(out, "<{name}/>");
        }
    }
}

fn version(out: &mut String, rpm: &Package) {
    let _ = write!(
        out,
        "<version epoch=\"{}\" ver=\"{}\" rel=\"{}\"/>",
        xml::attribute(&rpm.epoch),
        xml::attribute(&rpm.version),
        xml::attribute(&rpm.release)
    );
}

fn dependencies(out: &mut String, name: &str, entries: &[Dependency]) {
    if entries.is_empty() {
        return;
    }
    let _ = write!(out, "<rpm:{name}>");
    for entry in entries {
        let _ = write!(out, "<rpm:entry name=\"{}\"", xml::attribute(&entry.name));
        if let Some(flags) = entry.flags {
            let _ = write!(out, " flags=\"{flags}\"");
            for (key, value) in [
                ("epoch", &entry.epoch),
                ("ver", &entry.version),
                ("rel", &entry.release),
            ] {
                if let Some(value) = value {
                    let _ = write!(out, " {key}=\"{}\"", xml::attribute(value));
                }
            }
        }
        if entry.pre {
            out.push_str(" pre=\"1\"");
        }
        out.push_str("/>");
    }
    let _ = write!(out, "</rpm:{name}>");
}

fn file(out: &mut String, file: &RpmFile) {
    match file.kind {
        "" => {
            let _ = write!(out, "<file>{}</file>", xml::text(&file.path));
        }
        kind => {
            let _ = write!(
                out,
                "<file type=\"{kind}\">{}</file>",
                xml::text(&file.path)
            );
        }
    }
}

pub fn primary(entry: &Entry<'_>) -> String {
    let rpm = entry.rpm;
    let mut out = String::with_capacity(2048);
    out.push_str("<package type=\"rpm\">");
    element(&mut out, "name", Some(&rpm.name));
    element(&mut out, "arch", Some(&rpm.arch));
    version(&mut out, rpm);
    let _ = write!(
        out,
        "<checksum type=\"sha256\" pkgid=\"YES\">{}</checksum>",
        xml::text(entry.checksum)
    );
    element(&mut out, "summary", rpm.summary.as_deref());
    element(&mut out, "description", rpm.description.as_deref());
    element(&mut out, "packager", rpm.packager.as_deref());
    element(&mut out, "url", rpm.url.as_deref());
    let _ = write!(
        out,
        "<time file=\"{}\" build=\"{}\"/><size package=\"{}\" installed=\"{}\" archive=\"{}\"/><location href=\"{}\"/>",
        entry.file_time,
        rpm.build_time,
        entry.size,
        rpm.installed_size,
        rpm.archive_size,
        xml::attribute(entry.location)
    );
    out.push_str("<format>");
    element(&mut out, "rpm:license", rpm.license.as_deref());
    element(&mut out, "rpm:vendor", rpm.vendor.as_deref());
    element(&mut out, "rpm:group", rpm.group.as_deref());
    element(&mut out, "rpm:buildhost", rpm.build_host.as_deref());
    element(&mut out, "rpm:sourcerpm", rpm.source_rpm.as_deref());
    let _ = write!(
        out,
        "<rpm:header-range start=\"{}\" end=\"{}\"/>",
        rpm.header_start, rpm.header_end
    );
    dependencies(&mut out, "provides", &rpm.provides);
    dependencies(&mut out, "requires", &rpm.requires);
    dependencies(&mut out, "conflicts", &rpm.conflicts);
    dependencies(&mut out, "obsoletes", &rpm.obsoletes);
    dependencies(&mut out, "suggests", &rpm.suggests);
    dependencies(&mut out, "enhances", &rpm.enhances);
    dependencies(&mut out, "recommends", &rpm.recommends);
    dependencies(&mut out, "supplements", &rpm.supplements);
    for found in rpm.files.iter().filter(|found| found.is_primary()) {
        file(&mut out, found);
    }
    out.push_str("</format></package>");
    out
}

fn start(out: &mut String, entry: &Entry<'_>) {
    let _ = write!(
        out,
        "<package pkgid=\"{}\" name=\"{}\" arch=\"{}\">",
        xml::attribute(entry.checksum),
        xml::attribute(&entry.rpm.name),
        xml::attribute(&entry.rpm.arch)
    );
    version(out, entry.rpm);
}

pub fn filelists(entry: &Entry<'_>) -> String {
    let mut out = String::with_capacity(1024);
    start(&mut out, entry);
    for found in &entry.rpm.files {
        file(&mut out, found);
    }
    out.push_str("</package>");
    out
}

pub fn other(entry: &Entry<'_>) -> String {
    let mut out = String::with_capacity(1024);
    start(&mut out, entry);
    for changelog in &entry.rpm.changelogs {
        let _ = write!(
            out,
            "<changelog author=\"{}\" date=\"{}\">{}</changelog>",
            xml::attribute(&changelog.author),
            changelog.date,
            xml::text(&changelog.text)
        );
    }
    out.push_str("</package>");
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepomdData {
    pub kind: String,
    pub checksum_type: String,
    pub checksum: String,
    pub open_checksum_type: Option<String>,
    pub open_checksum: Option<String>,
    pub location: String,
    pub timestamp: Option<i64>,
    pub size: Option<u64>,
    pub open_size: Option<u64>,
    pub location_base: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Repomd {
    pub revision: Option<String>,
    pub data: Vec<RepomdData>,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct RepomdError(String);

impl Repomd {
    pub fn data(&self, kind: &str) -> Option<&RepomdData> {
        self.data.iter().find(|entry| entry.kind == kind)
    }

    pub fn to_xml(&self) -> Vec<u8> {
        let mut out = String::with_capacity(2048);
        out.push_str(xml::DECLARATION);
        let _ = write!(out, "<repomd xmlns=\"{REPO_NS}\" xmlns:rpm=\"{RPM_NS}\">");
        if let Some(revision) = &self.revision {
            let _ = write!(out, "\n  <revision>{}</revision>", xml::text(revision));
        }
        for entry in &self.data {
            let _ = write!(
                out,
                "\n  <data type=\"{}\">\n    <checksum type=\"{}\">{}</checksum>",
                xml::attribute(&entry.kind),
                xml::attribute(&entry.checksum_type),
                xml::text(&entry.checksum)
            );
            if let Some(open) = &entry.open_checksum {
                let _ = write!(
                    out,
                    "\n    <open-checksum type=\"{}\">{}</open-checksum>",
                    xml::attribute(entry.open_checksum_type.as_deref().unwrap_or("sha256")),
                    xml::text(open)
                );
            }
            out.push_str("\n    <location");
            if let Some(base) = &entry.location_base {
                let _ = write!(out, " xml:base=\"{}\"", xml::attribute(base));
            }
            let _ = write!(out, " href=\"{}\"/>", xml::attribute(&entry.location));
            for (name, value) in [
                ("timestamp", entry.timestamp.map(|value| value.to_string())),
                ("size", entry.size.map(|value| value.to_string())),
                ("open-size", entry.open_size.map(|value| value.to_string())),
            ] {
                if let Some(value) = value {
                    let _ = write!(out, "\n    <{name}>{value}</{name}>");
                }
            }
            out.push_str("\n  </data>");
        }
        out.push_str("\n</repomd>\n");
        out.into_bytes()
    }

    pub fn parse(bytes: &[u8]) -> Result<Repomd, RepomdError> {
        let fail = |message: String| RepomdError(format!("unreadable repomd.xml: {message}"));
        let mut reader = quick_xml::Reader::from_reader(bytes);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut path: Vec<String> = Vec::new();
        let mut repomd = Repomd::default();
        let mut current: Option<RepomdData> = None;
        let mut text = String::new();
        let mut seen_root = false;
        let mut pending_type: Option<String> = None;
        loop {
            let event = reader
                .read_event_into(&mut buffer)
                .map_err(|error| fail(error.to_string()))?;
            match event {
                Event::Start(ref element) | Event::Empty(ref element) => {
                    let empty = matches!(event, Event::Empty(_));
                    let name = element.local_name().as_ref().to_owned();
                    if !seen_root {
                        if name != "repomd" {
                            return Err(RepomdError("not a repomd.xml document".to_owned()));
                        }
                        seen_root = true;
                    }
                    let mut attributes = std::collections::HashMap::new();
                    for attribute in element.attributes().flatten() {
                        let key = attribute.key;
                        let prefix = key.prefix().map(|prefix| prefix.as_ref().to_owned());
                        let local = key.local_name().as_ref().to_owned();
                        let value = attribute
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .map(|value| value.into_owned())
                            .unwrap_or_default();
                        let key = match prefix.as_deref() {
                            Some("xml") => format!("xml:{local}"),
                            Some(_) => continue,
                            None => local,
                        };
                        attributes.insert(key, value);
                    }
                    match (path.len(), name.as_str()) {
                        (1, "data") => {
                            current = Some(RepomdData {
                                kind: attributes.get("type").cloned().unwrap_or_default(),
                                checksum_type: "sha256".to_owned(),
                                checksum: String::new(),
                                open_checksum_type: None,
                                open_checksum: None,
                                location: String::new(),
                                timestamp: None,
                                size: None,
                                open_size: None,
                                location_base: None,
                            });
                        }
                        (2, "location") => {
                            if let Some(entry) = current.as_mut() {
                                entry.location =
                                    attributes.get("href").cloned().unwrap_or_default();
                                entry.location_base = attributes.get("xml:base").cloned();
                            }
                        }
                        (2, "checksum") | (2, "open-checksum") => {
                            pending_type = attributes.get("type").cloned();
                        }
                        _ => {}
                    }
                    text.clear();
                    if !empty {
                        path.push(name);
                    } else if path.len() == 1
                        && name == "data"
                        && let Some(entry) = current.take()
                    {
                        repomd.data.push(entry);
                    }
                }
                Event::Text(value) => {
                    text.push_str(&value.xml10_content());
                }
                Event::GeneralRef(reference) => {
                    let name = reference.as_ref().to_owned();
                    text.push_str(match name.as_str() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        _ => "",
                    });
                }
                Event::CData(value) => {
                    text.push_str(value.as_ref());
                }
                Event::End(_) => {
                    let Some(name) = path.pop() else {
                        break;
                    };
                    let value = text.trim().to_owned();
                    match (path.len(), name.as_str()) {
                        (1, "revision") => repomd.revision = Some(value),
                        (1, "data") => {
                            if let Some(entry) = current.take() {
                                repomd.data.push(entry);
                            }
                        }
                        (2, field) => {
                            if let Some(entry) = current.as_mut() {
                                match field {
                                    "checksum" => {
                                        entry.checksum = value;
                                        if let Some(kind) = pending_type.take() {
                                            entry.checksum_type = kind;
                                        }
                                    }
                                    "open-checksum" => {
                                        entry.open_checksum = Some(value);
                                        entry.open_checksum_type = pending_type.take();
                                    }
                                    "timestamp" => {
                                        entry.timestamp =
                                            value.parse::<f64>().ok().map(|v| v as i64)
                                    }
                                    "size" => entry.size = value.parse().ok(),
                                    "open-size" => entry.open_size = value.parse().ok(),
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                    text.clear();
                    if path.is_empty() {
                        break;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }
        if !seen_root {
            return Err(RepomdError("not a repomd.xml document".to_owned()));
        }
        if repomd.data.iter().any(|entry| {
            entry.kind.is_empty() || entry.location.is_empty() || entry.checksum.is_empty()
        }) {
            return Err(RepomdError(
                "repomd.xml has an incomplete data entry".to_owned(),
            ));
        }
        Ok(repomd)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use crate::testing::TestRpm;

    #[test]
    fn repomd_documents_round_trip() {
        let repomd = Repomd {
            revision: Some("1700000000".to_owned()),
            data: vec![RepomdData {
                kind: "primary".to_owned(),
                checksum_type: "sha256".to_owned(),
                checksum: "abc".to_owned(),
                open_checksum_type: Some("sha256".to_owned()),
                open_checksum: Some("def".to_owned()),
                location: "repodata/abc-primary.xml.gz".to_owned(),
                timestamp: Some(1_700_000_000),
                size: Some(10),
                open_size: Some(20),
                location_base: Some("https://mirror.example.org/".to_owned()),
            }],
        };
        let text = repomd.to_xml();
        assert!(String::from_utf8_lossy(&text).contains("<location xml:base=\"https://mirror.example.org/\" href=\"repodata/abc-primary.xml.gz\"/>"));
        assert_eq!(Repomd::parse(&text).unwrap(), repomd);
        assert!(Repomd::parse(b"<other/>").is_err());
        let foreign = br#"<?xml version="1.0"?><repomd xmlns="http://linux.duke.edu/metadata/repo"><revision>7</revision><data type="primary_db"><checksum type="sha">x</checksum><location href="repodata/x.sqlite.bz2"/><timestamp>1.5</timestamp></data></repomd>"#;
        let parsed = Repomd::parse(foreign).unwrap();
        assert_eq!(parsed.data[0].checksum_type, "sha");
        assert_eq!(parsed.data[0].timestamp, Some(1));
    }

    #[test]
    fn metadata_files_are_compressed_with_their_checksums() {
        let rpm = crate::rpm::parse(
            &TestRpm::new("hello", "1.0", "1")
                .file("/usr/bin/hello", 3)
                .changelog(1, "A <a@example.org>", "- x & y")
                .build(),
        )
        .unwrap();
        let entry = Entry {
            rpm: &rpm,
            location: "Packages/hello-1.0-1.x86_64.rpm",
            checksum: "0123",
            size: 99,
            file_time: 5,
        };
        let mut writer = MetadataWriter::new(MetadataType::Primary, 1).unwrap();
        writer.package(&entry).unwrap();
        let mut generated = writer.finish(42).unwrap();
        let mut compressed = Vec::new();
        generated.file.read_to_end(&mut compressed).unwrap();
        assert_eq!(
            generated.data.checksum,
            hex::encode(Sha256::digest(&compressed))
        );
        assert_eq!(generated.data.size, Some(compressed.len() as u64));
        let mut open = String::new();
        flate2::read::GzDecoder::new(compressed.as_slice())
            .read_to_string(&mut open)
            .unwrap();
        assert_eq!(generated.data.open_size, Some(open.len() as u64));
        assert!(open.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata xmlns=\"http://linux.duke.edu/metadata/common\" xmlns:rpm=\"http://linux.duke.edu/metadata/rpm\" packages=\"1\">\n<package type=\"rpm\"><name>hello</name>"));
        assert!(open.contains("<location href=\"Packages/hello-1.0-1.x86_64.rpm\"/>"));
        assert!(open.contains("<file>/usr/bin/hello</file>"));
        assert_eq!(
            generated.file_name(),
            format!("{}-primary.xml.gz", generated.data.checksum)
        );
        assert!(other(&entry).contains(
            "<changelog author=\"A &lt;a@example.org&gt;\" date=\"1\">- x &amp; y</changelog>"
        ));
    }
}
