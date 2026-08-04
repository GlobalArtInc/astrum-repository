use std::collections::BTreeMap;
use std::io::Write;

use chrono::{DateTime, Utc};
use md5::Md5;
use sha2::{Digest, Sha256};

use crate::deb::Control;

pub struct Package {
    pub control: Control,
    pub filename: String,
    pub size: i64,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
}

impl Package {
    pub fn stanza(&self) -> String {
        let mut text = self.control.render();
        text.push_str(&format!(
            "Filename: {}\nSize: {}\nMD5Sum: {}\nSHA1: {}\nSHA256: {}\n",
            self.filename.trim_start_matches('/'),
            self.size,
            self.md5,
            self.sha1,
            self.sha256
        ));
        text
    }
}

pub struct File {
    pub path: String,
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

pub fn packages_text(packages: &[&Package]) -> String {
    let mut text = String::new();
    for (at, package) in packages.iter().enumerate() {
        if at > 0 {
            text.push('\n');
        }
        text.push_str(&package.stanza());
    }
    text
}

pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), flate2::Compression::best());
    encoder
        .write_all(data)
        .expect("compressing in memory does not fail");
    encoder
        .finish()
        .expect("compressing in memory does not fail")
}

pub fn bzip2(data: &[u8]) -> Vec<u8> {
    let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::best());
    encoder
        .write_all(data)
        .expect("compressing in memory does not fail");
    encoder
        .finish()
        .expect("compressing in memory does not fail")
}

pub fn architecture_files(distribution: &str, packages: &[Package]) -> (Vec<String>, Vec<File>) {
    let mut by_architecture: BTreeMap<&str, Vec<&Package>> = BTreeMap::new();
    for package in packages {
        by_architecture
            .entry(package.control.architecture())
            .or_default()
            .push(package);
    }
    let mut files = Vec::new();
    let mut architectures = Vec::new();
    for (architecture, mut list) in by_architecture {
        list.sort_by(|left, right| {
            (left.control.package(), left.control.version())
                .cmp(&(right.control.package(), right.control.version()))
        });
        architectures.push(architecture.to_owned());
        let text = packages_text(&list).into_bytes();
        let base = format!("/dists/{distribution}/main/binary-{architecture}/Packages");
        files.push(File {
            path: format!("{base}.gz"),
            content_type: "application/x-gzip",
            bytes: gzip(&text),
        });
        files.push(File {
            path: format!("{base}.bz2"),
            content_type: "application/x-bzip2",
            bytes: bzip2(&text),
        });
        files.push(File {
            path: base,
            content_type: "text/plain",
            bytes: text,
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    (architectures, files)
}

pub fn release(
    distribution: &str,
    architectures: &[String],
    files: &[File],
    date: DateTime<Utc>,
) -> String {
    let prefix = format!("/dists/{distribution}/");
    let mut sha256 = String::new();
    let mut md5 = String::new();
    for file in files {
        let relative = file.path.strip_prefix(&prefix).unwrap_or(&file.path);
        sha256.push_str(&format!(
            " {} {} {relative}\n",
            hex::encode(Sha256::digest(&file.bytes)),
            file.bytes.len()
        ));
        md5.push_str(&format!(
            " {} {} {relative}\n",
            hex::encode(Md5::digest(&file.bytes)),
            file.bytes.len()
        ));
    }
    let text = format!(
        "Suite: {distribution}\nCodename: {distribution}\nComponents: main\nDate: {}\nArchitectures: {}\nSHA256:\n{sha256}MD5Sum:\n{md5}",
        date.format("%a, %d %b %Y %H:%M:%S GMT"),
        architectures.join(" "),
    );
    let mut trimmed: String = text
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    trimmed.push('\n');
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, version: &str, architecture: &str) -> Package {
        Package {
            control: Control::parse(&format!(
                "Package: {name}\nVersion: {version}\nArchitecture: {architecture}\nDescription: test\n"
            )),
            filename: format!(
                "/pool/{}/{name}/{name}_{version}_{architecture}.deb",
                &name[..1]
            ),
            size: 10,
            md5: "m".to_owned(),
            sha1: "s".to_owned(),
            sha256: "h".to_owned(),
        }
    }

    #[test]
    fn packages_and_release_files_list_every_architecture() {
        let packages = vec![
            package("zeta", "1.0", "amd64"),
            package("alpha", "2.0", "amd64"),
            package("doc", "1.0", "all"),
        ];
        let (architectures, files) = architecture_files("bookworm", &packages);
        assert_eq!(architectures, ["all", "amd64"]);
        let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "/dists/bookworm/main/binary-all/Packages",
                "/dists/bookworm/main/binary-all/Packages.bz2",
                "/dists/bookworm/main/binary-all/Packages.gz",
                "/dists/bookworm/main/binary-amd64/Packages",
                "/dists/bookworm/main/binary-amd64/Packages.bz2",
                "/dists/bookworm/main/binary-amd64/Packages.gz",
            ]
        );
        let amd64 = String::from_utf8(files[3].bytes.clone()).unwrap();
        assert!(amd64.starts_with("Package: alpha\n"));
        assert!(amd64.contains("\n\nPackage: zeta\n"));
        assert!(amd64.contains(
            "Filename: pool/a/alpha/alpha_2.0_amd64.deb\nSize: 10\nMD5Sum: m\nSHA1: s\nSHA256: h\n"
        ));
        assert_eq!(gzip(b"same"), gzip(b"same"));
        let date = DateTime::parse_from_rfc3339("2026-10-18T03:01:11Z")
            .unwrap()
            .with_timezone(&Utc);
        let text = release("bookworm", &architectures, &files, date);
        assert!(text.starts_with("Suite: bookworm\nCodename: bookworm\nComponents: main\nDate: Sun, 18 Oct 2026 03:01:11 GMT\nArchitectures: all amd64\nSHA256:\n"));
        assert!(text.contains(&format!(
            " {} {} main/binary-amd64/Packages\n",
            hex::encode(Sha256::digest(&files[3].bytes)),
            files[3].bytes.len()
        )));
        assert!(text.contains("MD5Sum:\n"));
        assert!(text.lines().all(|line| line == line.trim_end()));
    }
}
