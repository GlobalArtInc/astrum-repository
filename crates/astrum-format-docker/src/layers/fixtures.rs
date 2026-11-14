use std::cell::Cell;
use std::io::{self, Read, Write};
use std::rc::Rc;

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

pub fn block(name: &str, size: u64, kind: EntryType, link: Option<&str>) -> Vec<u8> {
    header(name, size, kind, link).as_bytes().to_vec()
}

pub fn with_data(mut block: Vec<u8>, data: &[u8]) -> Vec<u8> {
    block.extend_from_slice(data);
    pad(&mut block, data.len());
    block
}

pub fn claim(name: &str, claimed: u64, body: &[u8]) -> Vec<u8> {
    with_data(block(name, claimed, EntryType::Regular, None), body)
}

pub const END: [u8; 1024] = [0; 1024];

pub struct Repeat {
    pattern: Vec<u8>,
    at: usize,
}

impl Repeat {
    pub fn new(pattern: &[u8]) -> Self {
        Repeat {
            pattern: pattern.to_vec(),
            at: 0,
        }
    }
}

impl Read for Repeat {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for byte in buf.iter_mut() {
            *byte = self.pattern[self.at];
            self.at = (self.at + 1) % self.pattern.len();
        }
        Ok(buf.len())
    }
}

pub struct Counting<R> {
    inner: R,
    read: Rc<Cell<u64>>,
}

impl<R> Counting<R> {
    pub fn new(inner: R) -> (Self, Rc<Cell<u64>>) {
        let read = Rc::new(Cell::new(0));
        (
            Counting {
                inner,
                read: read.clone(),
            },
            read,
        )
    }
}

impl<R: Read> Read for Counting<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.read.set(self.read.get() + read as u64);
        Ok(read)
    }
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
