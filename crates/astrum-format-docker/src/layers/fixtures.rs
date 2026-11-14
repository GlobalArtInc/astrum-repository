use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use tar::{EntryType, Header};

pub const ALPINE_OS: &str = "NAME=\"Alpine Linux\"\nID=alpine\nVERSION_ID=3.19.1\n";
pub const DEBIAN_OS: &str = "PRETTY_NAME=\"Debian GNU/Linux 12\"\nID=debian\nVERSION_ID=\"12\"\n";
pub const APK_DB: &str = "C:Q1abc\nP:musl\nV:1.2.4-r2\nA:x86_64\no:musl\n\n\
P:busybox-binsh\nV:1.36.1-r15\nA:x86_64\no:busybox\n\n";
pub const DPKG_DB: &str = "Package: libc6\nStatus: install ok installed\nVersion: 2.36-9\n\
Source: glibc\nArchitecture: amd64\nDescription: C library\n continued\n\n\
Package: bash\nStatus: install ok installed\nVersion: 5.2.15-2\nArchitecture: amd64\n\n";

pub enum Entry<'a> {
    File(&'a str, &'a [u8]),
    Symlink(&'a str, &'a str),
    Hardlink(&'a str, &'a str),
    Dir(&'a str),
}

fn header(name: &str, size: u64, kind: EntryType, link: Option<&str>) -> Header {
    let mut header = Header::new_gnu();
    header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
    header.set_size(size);
    header.set_entry_type(kind);
    header.set_mode(0o644);
    if let Some(link) = link {
        header.set_link_name(link).unwrap();
    }
    header.set_cksum();
    header
}

fn pad(out: &mut Vec<u8>, length: usize) {
    out.resize(out.len() + (512 - length % 512) % 512, 0);
}

pub fn claim(name: &str, claimed: u64, body: &[u8]) -> Vec<u8> {
    let mut out = header(name, claimed, EntryType::Regular, None)
        .as_bytes()
        .to_vec();
    out.extend_from_slice(body);
    pad(&mut out, body.len());
    out
}

pub fn tar(entries: &[Entry<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in entries {
        match entry {
            Entry::File(name, data) => {
                out.extend_from_slice(
                    header(name, data.len() as u64, EntryType::Regular, None).as_bytes(),
                );
                out.extend_from_slice(data);
                pad(&mut out, data.len());
            }
            Entry::Symlink(name, target) => {
                out.extend_from_slice(header(name, 0, EntryType::Symlink, Some(target)).as_bytes())
            }
            Entry::Hardlink(name, target) => {
                out.extend_from_slice(header(name, 0, EntryType::Link, Some(target)).as_bytes())
            }
            Entry::Dir(name) => {
                out.extend_from_slice(header(name, 0, EntryType::Directory, None).as_bytes())
            }
        }
    }
    out.resize(out.len() + 1024, 0);
    out
}

pub fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

pub fn zstd(bytes: &[u8]) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(bytes, ruzstd::encoding::CompressionLevel::Fastest)
}

pub fn alpine_layer() -> Vec<u8> {
    tar(&[
        Entry::Dir("etc/"),
        Entry::File("etc/os-release", ALPINE_OS.as_bytes()),
        Entry::File("lib/apk/db/installed", APK_DB.as_bytes()),
    ])
}
