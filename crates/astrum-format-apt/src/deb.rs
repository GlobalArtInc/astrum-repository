use std::io::{self, Read};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DebError {
    #[error("the file is not a Debian package: {0}")]
    NotDeb(String),
    #[error("the control file of the package lacks the {0} field")]
    Missing(&'static str),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Control {
    pub fields: Vec<(String, String)>,
}

impl Control {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn package(&self) -> &str {
        self.get("Package").unwrap_or_default()
    }

    pub fn version(&self) -> &str {
        self.get("Version").unwrap_or_default()
    }

    pub fn architecture(&self) -> &str {
        self.get("Architecture").unwrap_or_default()
    }

    pub fn parse(text: &str) -> Control {
        let mut fields: Vec<(String, String)> = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                if !fields.is_empty() {
                    break;
                }
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some((_, value)) = fields.last_mut() {
                    value.push('\n');
                    value.push_str(line);
                }
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                fields.push((key.trim().to_owned(), value.trim().to_owned()));
            }
        }
        Control { fields }
    }

    pub fn render(&self) -> String {
        let mut text = String::new();
        for (key, value) in &self.fields {
            text.push_str(key);
            text.push(':');
            if !value.is_empty() && !value.starts_with('\n') {
                text.push(' ');
            }
            text.push_str(value);
            text.push('\n');
        }
        text
    }

    pub fn checked(self) -> Result<Control, DebError> {
        if self.package().is_empty() {
            return Err(DebError::Missing("Package"));
        }
        if self.version().is_empty() {
            return Err(DebError::Missing("Version"));
        }
        if self.architecture().is_empty() {
            return Err(DebError::Missing("Architecture"));
        }
        let package = self.package();
        if !package
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
        {
            return Err(DebError::NotDeb(format!(
                "`{package}` is not a valid package name"
            )));
        }
        if self
            .version()
            .chars()
            .any(|c| c.is_whitespace() || c == '/' || c == '_')
            || self
                .architecture()
                .chars()
                .any(|c| !(c.is_ascii_alphanumeric() || c == '-'))
        {
            return Err(DebError::NotDeb(
                "the version or architecture holds characters a package file name cannot"
                    .to_owned(),
            ));
        }
        Ok(self)
    }
}

struct Member<'a> {
    name: String,
    data: &'a [u8],
}

fn members(data: &[u8]) -> Result<Vec<Member<'_>>, DebError> {
    let Some(mut rest) = data.strip_prefix(b"!<arch>\n") else {
        return Err(DebError::NotDeb(
            "it does not start with an ar header".to_owned(),
        ));
    };
    let mut found = Vec::new();
    while rest.len() >= 60 {
        let header = &rest[..60];
        if &header[58..60] != b"`\n" {
            return Err(DebError::NotDeb(
                "a member header of the archive is broken".to_owned(),
            ));
        }
        let name = String::from_utf8_lossy(&header[..16])
            .trim_end()
            .trim_end_matches('/')
            .to_owned();
        let size: usize = String::from_utf8_lossy(&header[48..58])
            .trim()
            .parse()
            .map_err(|_| DebError::NotDeb("a member size of the archive is broken".to_owned()))?;
        let body = &rest[60..];
        if body.len() < size {
            found.push(Member { name, data: body });
            break;
        }
        found.push(Member {
            name,
            data: &body[..size],
        });
        let next = size + size % 2;
        rest = body.get(next..).unwrap_or_default();
    }
    Ok(found)
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn decompress(name: &str, data: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut input = data;
    if name.ends_with(".gz") {
        flate2::read::GzDecoder::new(input)
            .take(64 * 1024 * 1024)
            .read_to_end(&mut out)?;
    } else if name.ends_with(".xz") {
        lzma_rs::xz_decompress(&mut input, &mut out).map_err(invalid)?;
    } else if name.ends_with(".zst") {
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(input).map_err(invalid)?;
        decoder.read_to_end(&mut out)?;
    } else if name.ends_with(".bz2") {
        bzip2::read::MultiBzDecoder::new(input).read_to_end(&mut out)?;
    } else {
        out.extend_from_slice(data);
    }
    Ok(out)
}

pub fn read_control(data: &[u8]) -> Result<Control, DebError> {
    let members = members(data)?;
    let member = members
        .iter()
        .find(|member| member.name.starts_with("control.tar"))
        .ok_or_else(|| DebError::NotDeb("it has no control archive".to_owned()))?;
    let tarball = decompress(&member.name, member.data).map_err(|error| {
        DebError::NotDeb(format!("the control archive cannot be read: {error}"))
    })?;
    let mut archive = tar::Archive::new(tarball.as_slice());
    let entries = archive
        .entries()
        .map_err(|error| DebError::NotDeb(error.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|error| DebError::NotDeb(error.to_string()))?;
        let path = entry
            .path()
            .map_err(|error| DebError::NotDeb(error.to_string()))?
            .to_string_lossy()
            .trim_start_matches("./")
            .to_owned();
        if path == "control" {
            let mut text = String::new();
            entry
                .take(1024 * 1024)
                .read_to_string(&mut text)
                .map_err(|error| DebError::NotDeb(error.to_string()))?;
            return Control::parse(&text).checked();
        }
    }
    Err(DebError::NotDeb(
        "its control archive has no control file".to_owned(),
    ))
}

pub fn upstream_version(version: &str) -> &str {
    match version.split_once(':') {
        Some((epoch, rest)) if !epoch.is_empty() && epoch.chars().all(|c| c.is_ascii_digit()) => {
            rest
        }
        _ => version,
    }
}

pub fn pool_path(control: &Control) -> String {
    let package = control.package();
    let first = package.chars().next().unwrap_or('_');
    format!(
        "/pool/{first}/{package}/{package}_{}_{}.deb",
        upstream_version(control.version()),
        control.architecture()
    )
}

pub fn from_file_name(file: &str) -> Option<(String, String, String)> {
    let decoded = percent_encoding::percent_decode_str(file)
        .decode_utf8()
        .map(|text| text.into_owned())
        .unwrap_or_else(|_| file.to_owned());
    let stem = decoded
        .strip_suffix(".deb")
        .or_else(|| decoded.strip_suffix(".udeb"))?;
    let mut parts = stem.splitn(3, '_');
    let (package, version, architecture) = (parts.next()?, parts.next()?, parts.next()?);
    (!package.is_empty() && !version.is_empty() && !architecture.is_empty()).then(|| {
        (
            package.to_owned(),
            version.to_owned(),
            architecture.to_owned(),
        )
    })
}

pub fn build(control: &str, payload: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write;
    fn tarball(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, data) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(0);
            header.set_cksum();
            builder
                .append_data(&mut header, name, *data)
                .expect("appending to a tarball in memory does not fail");
        }
        builder
            .into_inner()
            .expect("finishing a tarball in memory does not fail")
    }
    fn gzip(data: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder
            .write_all(data)
            .expect("compressing in memory does not fail");
        encoder
            .finish()
            .expect("compressing in memory does not fail")
    }
    fn member(out: &mut Vec<u8>, name: &str, data: &[u8]) {
        out.extend_from_slice(
            format!(
                "{name:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
                0,
                0,
                0,
                100644,
                data.len()
            )
            .as_bytes(),
        );
        out.extend_from_slice(data);
        if data.len() % 2 == 1 {
            out.push(b'\n');
        }
    }
    let control_tar = gzip(&tarball(&[("./control", control.as_bytes())]));
    let data_tar = gzip(&tarball(payload));
    let mut out = b"!<arch>\n".to_vec();
    member(&mut out, "debian-binary", b"2.0\n");
    member(&mut out, "control.tar.gz", &control_tar);
    member(&mut out, "data.tar.gz", &data_tar);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTROL: &str = "Package: hello-astrum\nVersion: 1:1.0.0-1\nArchitecture: amd64\nMaintainer: Tests <t@example.org>\nDescription: A test\n It has two lines.\n";

    #[test]
    fn control_files_are_read_from_built_packages() {
        let deb = build(CONTROL, &[("./usr/share/doc/hello/README", b"hi\n")]);
        let control = read_control(&deb).unwrap();
        assert_eq!(control.package(), "hello-astrum");
        assert_eq!(control.version(), "1:1.0.0-1");
        assert_eq!(
            control.get("description"),
            Some("A test\n It has two lines.")
        );
        assert_eq!(control.render(), CONTROL);
        assert_eq!(
            pool_path(&control),
            "/pool/h/hello-astrum/hello-astrum_1.0.0-1_amd64.deb"
        );
        assert!(read_control(b"PK\x03\x04").is_err());
        let broken = build("Package: x\nVersion: 1\n", &[]);
        assert_eq!(
            read_control(&broken),
            Err(DebError::Missing("Architecture"))
        );
    }

    #[test]
    fn file_names_give_package_version_and_architecture() {
        assert_eq!(
            from_file_name("bash_5.2.15-2%2bb2_amd64.deb"),
            Some((
                "bash".to_owned(),
                "5.2.15-2+b2".to_owned(),
                "amd64".to_owned()
            ))
        );
        assert_eq!(
            from_file_name("tzdata_2024a-0%3a1_all.deb").map(|parts| parts.1),
            Some("2024a-0:1".to_owned())
        );
        assert_eq!(from_file_name("Release"), None);
    }
}
