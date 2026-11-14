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

pub struct Raw<'a> {
    name: &'a [u8],
    prefix: &'a [u8],
    flag: u8,
    size: u64,
    magic: &'a [u8],
    version: &'a [u8],
    link: &'a [u8],
    signed: bool,
    extended: u8,
}

impl<'a> Raw<'a> {
    pub fn new(name: &'a str) -> Self {
        Raw {
            name: name.as_bytes(),
            prefix: b"",
            flag: b'0',
            size: 0,
            magic: b"ustar\0",
            version: b"00",
            link: b"",
            signed: false,
            extended: 0,
        }
    }

    pub fn bytes(mut self, name: &'a [u8]) -> Self {
        self.name = name;
        self
    }

    pub fn prefix(mut self, prefix: &'a [u8]) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn flag(mut self, flag: u8) -> Self {
        self.flag = flag;
        self
    }

    pub fn magic(mut self, magic: &'a [u8], version: &'a [u8]) -> Self {
        self.magic = magic;
        self.version = version;
        self
    }

    pub fn link(mut self, link: &'a str) -> Self {
        self.link = link.as_bytes();
        self
    }

    pub fn signed(mut self) -> Self {
        self.signed = true;
        self
    }

    pub fn extended(mut self, flag: u8) -> Self {
        self.extended = flag;
        self
    }

    pub fn build(&self) -> Vec<u8> {
        self.build_sized(self.size)
    }

    fn build_sized(&self, size: u64) -> Vec<u8> {
        let mut block = vec![0u8; 512];
        block[..self.name.len()].copy_from_slice(self.name);
        block[100..107].copy_from_slice(b"0000644");
        block[108..115].copy_from_slice(b"0000000");
        block[116..123].copy_from_slice(b"0000000");
        block[124..135].copy_from_slice(format!("{size:011o}").as_bytes());
        block[136..147].copy_from_slice(b"00000000000");
        block[148..156].copy_from_slice(b"        ");
        block[156] = self.flag;
        block[157..157 + self.link.len()].copy_from_slice(self.link);
        block[257..257 + self.magic.len()].copy_from_slice(self.magic);
        block[263..263 + self.version.len()].copy_from_slice(self.version);
        block[345..345 + self.prefix.len()].copy_from_slice(self.prefix);
        block[482] = self.extended;
        let unsigned: i64 = block.iter().map(|byte| i64::from(*byte)).sum();
        let signed: i64 = block.iter().map(|byte| i64::from(*byte as i8)).sum();
        let sum = if self.signed { signed } else { unsigned };
        block[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        block
    }

    pub fn with(&self, data: &[u8]) -> Vec<u8> {
        let mut block = self.build_sized(data.len() as u64);
        block.extend_from_slice(data);
        pad(&mut block, data.len());
        block
    }
}

pub fn pax_record(key: &str, value: &str) -> Vec<u8> {
    let base = key.len() + value.len() + 3;
    let mut length = base + base.to_string().len();
    while length != base + length.to_string().len() {
        length = base + length.to_string().len();
    }
    format!("{length} {key}={value}\n").into_bytes()
}

pub fn pax(records: &[(&str, &str)]) -> Vec<u8> {
    let data: Vec<u8> = records
        .iter()
        .flat_map(|(key, value)| pax_record(key, value))
        .collect();
    Raw::new("pax").flag(b'x').with(&data)
}

pub struct Generated {
    entries: u64,
    make: Box<dyn Fn(u64) -> Vec<u8>>,
    next: u64,
    chunk: Vec<u8>,
    at: usize,
    ended: bool,
}

impl Generated {
    pub fn new(entries: u64, make: impl Fn(u64) -> Vec<u8> + 'static) -> Self {
        Generated {
            entries,
            make: Box::new(make),
            next: 0,
            chunk: Vec::new(),
            at: 0,
            ended: false,
        }
    }
}

impl Read for Generated {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.at >= self.chunk.len() {
            if self.next < self.entries {
                self.chunk = (self.make)(self.next);
                self.next += 1;
            } else if !self.ended {
                self.chunk = END.to_vec();
                self.ended = true;
            } else {
                return Ok(0);
            }
            self.at = 0;
        }
        let taken = buf.len().min(self.chunk.len() - self.at);
        buf[..taken].copy_from_slice(&self.chunk[self.at..self.at + taken]);
        self.at += taken;
        Ok(taken)
    }
}
