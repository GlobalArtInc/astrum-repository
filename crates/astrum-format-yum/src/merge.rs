use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};

use quick_xml::events::{BytesStart, Event};

use crate::repodata::{Generated, MetadataType, MetadataWriter, Repomd};
use crate::xml::RPM_NS;

pub struct MemberFiles {
    pub name: String,
    pub repomd: Repomd,
    pub files: HashMap<MetadataType, File>,
}

pub struct Merged {
    pub repomd: Repomd,
    pub files: Vec<Generated>,
    pub packages: u64,
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

pub fn decompress(mut raw: File) -> io::Result<File> {
    raw.seek(SeekFrom::Start(0))?;
    let mut input = BufReader::with_capacity(64 * 1024, raw);
    let head = input.fill_buf()?.to_vec();
    let mut out = tempfile::tempfile()?;
    match head.as_slice() {
        [0x1f, 0x8b, ..] => {
            io::copy(&mut flate2::bufread::MultiGzDecoder::new(input), &mut out)?;
        }
        [0xfd, b'7', b'z', b'X', b'Z', 0, ..] => {
            let mut writer = io::BufWriter::new(&mut out);
            lzma_rs::xz_decompress(&mut input, &mut writer).map_err(invalid)?;
            writer.flush()?;
        }
        [b'B', b'Z', b'h', ..] => {
            io::copy(&mut bzip2::bufread::MultiBzDecoder::new(input), &mut out)?;
        }
        [0x28, 0xb5, 0x2f, 0xfd, ..] => {
            let mut decoder = ruzstd::decoding::StreamingDecoder::new(input).map_err(invalid)?;
            io::copy(&mut decoder, &mut out)?;
        }
        [b'<' | 0xef | b' ' | b'\n' | b'\r' | b'\t', ..] => {
            io::copy(&mut input, &mut out)?;
        }
        [] => {}
        _ => {
            return Err(invalid(
                "the repository metadata uses a compression this server cannot read",
            ));
        }
    }
    out.seek(SeekFrom::Start(0))?;
    Ok(out)
}

struct PackageXml {
    pkgid: Option<String>,
    checksum: Option<String>,
    location: Option<String>,
    raw: String,
}

struct Names {
    prefixes: HashMap<String, String>,
    namespace: &'static str,
}

impl Names {
    fn from_root(root: &BytesStart<'_>, kind: MetadataType) -> Self {
        let mut prefixes = HashMap::new();
        for attribute in root.attributes().flatten() {
            let key = attribute.key.as_ref();
            let value = attribute.value.to_string();
            if key == "xmlns" {
                prefixes.insert(String::new(), value);
            } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                prefixes.insert(prefix.to_owned(), value);
            }
        }
        Self {
            prefixes,
            namespace: kind.namespace(),
        }
    }

    fn rename(&self, name: &str) -> String {
        let (prefix, local) = name.split_once(':').unwrap_or(("", name));
        let namespace = self.prefixes.get(prefix).map(String::as_str);
        let mut out = String::with_capacity(name.len() + 4);
        match namespace {
            Some(namespace) if namespace == self.namespace => {}
            Some(RPM_NS) => out.push_str("rpm:"),
            _ if !prefix.is_empty() => {
                out.push_str(prefix);
                out.push(':');
            }
            _ => {}
        }
        out.push_str(local);
        out
    }
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':')
        .map(|(_, local)| local)
        .unwrap_or(name)
}

fn attribute(element: &BytesStart<'_>, wanted: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attribute| attribute.key.as_ref() == wanted)
        .and_then(|attribute| {
            attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.into_owned())
        })
}

fn write_start(out: &mut String, names: &Names, element: &BytesStart<'_>, empty: bool) {
    out.push('<');
    out.push_str(&names.rename(element.name().as_ref()));
    out.push_str(element.attributes_raw());
    out.push_str(if empty { "/>" } else { ">" });
}

fn each_package(
    file: &mut File,
    kind: MetadataType,
    mut visit: impl FnMut(PackageXml) -> io::Result<()>,
) -> io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let mut reader =
        quick_xml::Reader::from_reader(BufReader::with_capacity(64 * 1024, &mut *file));
    let mut buffer = Vec::new();
    let mut names: Option<Names> = None;
    let mut current: Option<PackageXml> = None;
    let mut depth = 0usize;
    let mut capture: Option<&'static str> = None;
    let mut text = String::new();
    loop {
        let event = reader.read_event_into(&mut buffer).map_err(invalid)?;
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                let empty = matches!(event, Event::Empty(_));
                let Some(known) = names.as_ref() else {
                    names = Some(Names::from_root(element, kind));
                    if empty {
                        break;
                    }
                    continue;
                };
                let local = local_name(element.name().as_ref()).to_owned();
                match current.as_mut() {
                    None => {
                        if local == "package" {
                            let mut package = PackageXml {
                                pkgid: attribute(element, "pkgid"),
                                checksum: None,
                                location: None,
                                raw: String::with_capacity(2048),
                            };
                            write_start(&mut package.raw, known, element, empty);
                            if empty {
                                visit(package)?;
                            } else {
                                current = Some(package);
                                depth = 1;
                            }
                        } else if !empty {
                            reader
                                .read_to_end_into(element.name(), &mut Vec::new())
                                .map_err(invalid)?;
                        }
                    }
                    Some(package) => {
                        write_start(&mut package.raw, known, element, empty);
                        if depth == 1 && local == "location" {
                            let href = attribute(element, "href").unwrap_or_default();
                            package.location = Some(match attribute(element, "xml:base") {
                                Some(base) => format!("{base}|{href}"),
                                None => href,
                            });
                        }
                        if !empty {
                            if depth == 1 && local == "checksum" {
                                capture = Some("checksum");
                                text.clear();
                            }
                            depth += 1;
                        }
                    }
                }
            }
            Event::End(ref element) => {
                let Some(package) = current.as_mut() else {
                    continue;
                };
                package.raw.push_str("</");
                if let Some(known) = names.as_ref() {
                    package.raw.push_str(&known.rename(element.name().as_ref()));
                }
                package.raw.push('>');
                depth -= 1;
                if depth == 1 && capture.take().is_some() {
                    package.checksum = Some(text.trim().to_owned());
                }
                if depth == 0
                    && let Some(done) = current.take()
                {
                    visit(done)?;
                }
            }
            Event::Text(ref value) => {
                if let Some(package) = current.as_mut() {
                    package.raw.push_str(value.as_ref());
                    if capture.is_some() {
                        text.push_str(value.as_ref());
                    }
                }
            }
            Event::GeneralRef(ref value) => {
                if let Some(package) = current.as_mut() {
                    package.raw.push('&');
                    package.raw.push_str(value.as_ref());
                    package.raw.push(';');
                }
            }
            Event::CData(ref value) => {
                if let Some(package) = current.as_mut() {
                    package.raw.push_str("<![CDATA[");
                    package.raw.push_str(value.as_ref());
                    package.raw.push_str("]]>");
                    if capture.is_some() {
                        text.push_str(value.as_ref());
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if current.is_some() {
        return Err(invalid("the repository metadata ends inside a package"));
    }
    Ok(())
}

struct Selection {
    packages: Vec<bool>,
    pkgids: HashSet<String>,
}

pub fn merge(mut members: Vec<MemberFiles>, timestamp: i64) -> io::Result<Merged> {
    let mut locations = HashSet::new();
    let mut selections = Vec::with_capacity(members.len());
    let mut total = 0u64;
    for member in members.iter_mut() {
        let Some(primary) = member.files.get_mut(&MetadataType::Primary) else {
            selections.push(None);
            continue;
        };
        let mut selection = Selection {
            packages: Vec::new(),
            pkgids: HashSet::new(),
        };
        each_package(primary, MetadataType::Primary, |package| {
            let keep = match &package.checksum {
                Some(pkgid) if !pkgid.is_empty() => {
                    let location = package.location.clone().unwrap_or_else(|| pkgid.clone());
                    locations.insert(location) && {
                        selection.pkgids.insert(pkgid.clone());
                        true
                    }
                }
                _ => false,
            };
            selection.packages.push(keep);
            Ok(())
        })
        .map_err(|error| {
            invalid(format!(
                "the primary metadata of {} cannot be read: {error}",
                member.name
            ))
        })?;
        total += selection.packages.iter().filter(|kept| **kept).count() as u64;
        selections.push(Some(selection));
    }
    let mut files = Vec::with_capacity(3);
    let mut writer = MetadataWriter::new(MetadataType::Primary, total)?;
    for (member, selection) in members.iter_mut().zip(&selections) {
        let (Some(selection), Some(primary)) =
            (selection, member.files.get_mut(&MetadataType::Primary))
        else {
            continue;
        };
        let mut index = 0usize;
        each_package(primary, MetadataType::Primary, |package| {
            let keep = selection.packages.get(index).copied().unwrap_or(false);
            index += 1;
            if keep {
                writer.raw(package.raw.as_bytes())?;
                writer.raw(b"\n")?;
            }
            Ok(())
        })?;
    }
    files.push(writer.finish(timestamp)?);
    for kind in [MetadataType::Filelists, MetadataType::Other] {
        let mut writer = MetadataWriter::new(kind, total)?;
        for (member, selection) in members.iter_mut().zip(&selections) {
            let (Some(selection), Some(file)) = (selection, member.files.get_mut(&kind)) else {
                continue;
            };
            let mut copied = HashSet::new();
            each_package(file, kind, |package| {
                if let Some(pkgid) = package.pkgid
                    && selection.pkgids.contains(&pkgid)
                    && copied.insert(pkgid)
                {
                    writer.raw(package.raw.as_bytes())?;
                    writer.raw(b"\n")?;
                }
                Ok(())
            })
            .map_err(|error| {
                invalid(format!(
                    "the {} metadata of {} cannot be read: {error}",
                    kind.name(),
                    member.name
                ))
            })?;
        }
        files.push(writer.finish(timestamp)?);
    }
    let repomd = Repomd {
        revision: Some(timestamp.to_string()),
        data: files.iter().map(|file| file.data.clone()).collect(),
    };
    Ok(Merged {
        repomd,
        files,
        packages: total,
    })
}

pub fn read_all(file: &mut File) -> io::Result<Vec<u8>> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn file(text: &str) -> File {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(text.as_bytes()).unwrap();
        file
    }

    fn gz(text: &str) -> File {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(text.as_bytes()).unwrap();
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(&encoder.finish().unwrap()).unwrap();
        file
    }

    fn primary(packages: &[(&str, &str)], prefix: &str) -> String {
        let mut text = format!(
            "<?xml version=\"1.0\"?>\n<metadata xmlns=\"http://linux.duke.edu/metadata/common\" xmlns:{prefix}=\"http://linux.duke.edu/metadata/rpm\" packages=\"{}\">\n",
            packages.len()
        );
        for (pkgid, location) in packages {
            text.push_str(&format!("<package type=\"rpm\"><name>n &amp; m</name><checksum type=\"sha256\" pkgid=\"YES\">{pkgid}</checksum><location href=\"{location}\"/><format><{prefix}:license>MIT</{prefix}:license></format></package>\n"));
        }
        text.push_str("</metadata>\n");
        text
    }

    fn other(packages: &[&str]) -> String {
        let mut text =
            "<otherdata xmlns=\"http://linux.duke.edu/metadata/other\" packages=\"1\">".to_owned();
        for pkgid in packages {
            text.push_str(&format!("<package pkgid=\"{pkgid}\" name=\"n\" arch=\"x86_64\"><version epoch=\"0\" ver=\"1\" rel=\"1\"/></package>"));
        }
        text.push_str("</otherdata>");
        text
    }

    fn open(generated: &mut Generated) -> String {
        let mut text = String::new();
        generated.file.seek(SeekFrom::Start(0)).unwrap();
        flate2::read::GzDecoder::new(&mut generated.file)
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    #[test]
    fn members_merge_with_the_first_location_winning() {
        let first = MemberFiles {
            name: "a".to_owned(),
            repomd: Repomd::default(),
            files: HashMap::from([
                (
                    MetadataType::Primary,
                    decompress(gz(&primary(&[("aaa", "x.rpm"), ("bbb", "y.rpm")], "rpm"))).unwrap(),
                ),
                (
                    MetadataType::Other,
                    decompress(file(&other(&["aaa", "bbb"]))).unwrap(),
                ),
            ]),
        };
        let second = MemberFiles {
            name: "b".to_owned(),
            repomd: Repomd::default(),
            files: HashMap::from([
                (
                    MetadataType::Primary,
                    decompress(file(&primary(&[("ccc", "y.rpm"), ("ddd", "z.rpm")], "r"))).unwrap(),
                ),
                (
                    MetadataType::Other,
                    decompress(file(&other(&["ccc", "ddd", "ddd"]))).unwrap(),
                ),
            ]),
        };
        let mut merged = merge(vec![first, second], 7).unwrap();
        assert_eq!(merged.packages, 3);
        assert_eq!(merged.repomd.revision.as_deref(), Some("7"));
        let primary_text = open(&mut merged.files[0]);
        assert!(primary_text.contains("packages=\"3\""));
        assert!(
            primary_text.contains(">aaa<")
                && primary_text.contains(">bbb<")
                && primary_text.contains(">ddd<")
        );
        assert!(!primary_text.contains(">ccc<"));
        assert!(primary_text.contains("<rpm:license>MIT</rpm:license>"));
        assert!(primary_text.contains("<name>n &amp; m</name>"));
        let other_text = open(&mut merged.files[2]);
        assert_eq!(other_text.matches("pkgid=\"ddd\"").count(), 1);
        assert!(!other_text.contains("pkgid=\"ccc\""));
        let filelists_text = open(&mut merged.files[1]);
        assert!(filelists_text.contains(
            "<filelists xmlns=\"http://linux.duke.edu/metadata/filelists\" packages=\"3\">"
        ));
    }

    #[test]
    fn compressions_are_recognised() {
        let text = "<metadata/>";
        let mut xz = Vec::new();
        lzma_rs::xz_compress(&mut text.as_bytes(), &mut xz).unwrap();
        let mut xz_file = tempfile::tempfile().unwrap();
        xz_file.write_all(&xz).unwrap();
        assert_eq!(
            read_all(&mut decompress(xz_file).unwrap()).unwrap(),
            text.as_bytes()
        );
        let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
        encoder.write_all(text.as_bytes()).unwrap();
        let bz = encoder.finish().unwrap();
        let mut bz_file = tempfile::tempfile().unwrap();
        bz_file.write_all(&bz).unwrap();
        assert_eq!(
            read_all(&mut decompress(bz_file).unwrap()).unwrap(),
            text.as_bytes()
        );
        assert!(decompress(file("PK\u{3}\u{4}")).is_err());
    }
}
