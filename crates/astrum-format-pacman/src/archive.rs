use std::io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write};

use flate2::Compression as Level;
use flate2::write::GzEncoder;
use ruzstd::decoding::errors::{FrameDecoderError, ReadFrameHeaderError};
use ruzstd::decoding::{BlockDecodingStrategy, FrameDecoder};

const HEAD: u64 = 512;
const TAR_MAGIC: std::ops::Range<usize> = 257..262;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compression {
    None,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
}

impl Compression {
    pub fn detect(head: &[u8]) -> Option<Compression> {
        if head.starts_with(&[0x1f, 0x8b]) {
            Some(Compression::Gzip)
        } else if head.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
            Some(Compression::Zstd)
        } else if head.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00]) {
            Some(Compression::Xz)
        } else if head.starts_with(b"BZh") {
            Some(Compression::Bzip2)
        } else if head.get(TAR_MAGIC).is_some_and(|magic| magic == b"ustar") {
            Some(Compression::None)
        } else {
            None
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Compression::None => "application/x-tar",
            Compression::Gzip => "application/gzip",
            Compression::Bzip2 => "application/x-bzip2",
            Compression::Xz => "application/x-xz",
            Compression::Zstd => "application/zstd",
        }
    }
}

pub struct Zstd<R: BufRead> {
    source: R,
    decoder: FrameDecoder,
    active: bool,
}

impl<R: BufRead> Zstd<R> {
    pub fn new(source: R) -> Zstd<R> {
        Zstd {
            source,
            decoder: FrameDecoder::new(),
            active: false,
        }
    }
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

impl<R: BufRead> Read for Zstd<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        loop {
            if !self.active {
                if self.source.fill_buf()?.is_empty() {
                    return Ok(0);
                }
                match self.decoder.reset(&mut self.source) {
                    Ok(()) => self.active = true,
                    Err(FrameDecoderError::ReadFrameHeaderError(
                        ReadFrameHeaderError::SkipFrame { length, .. },
                    )) => {
                        let skipped =
                            io::copy(&mut (&mut self.source).take(length as u64), &mut io::sink())?;
                        if skipped < length as u64 {
                            return Err(io::ErrorKind::UnexpectedEof.into());
                        }
                        continue;
                    }
                    Err(error) => return Err(invalid(error)),
                }
            }
            if self.decoder.can_collect() > 0 {
                return self.decoder.read(buf);
            }
            if self.decoder.is_finished() {
                self.active = false;
                continue;
            }
            self.decoder
                .decode_blocks(&mut self.source, BlockDecodingStrategy::UptoBlocks(1))
                .map_err(invalid)?;
        }
    }
}

pub struct Opened<'a> {
    pub compression: Compression,
    pub reader: Box<dyn Read + 'a>,
    pub incomplete: bool,
}

pub fn open<'a>(mut input: impl Read + 'a) -> io::Result<Opened<'a>> {
    let mut head = Vec::with_capacity(HEAD as usize);
    (&mut input).take(HEAD).read_to_end(&mut head)?;
    let compression = Compression::detect(&head).ok_or_else(|| {
        invalid("the file is not a tar archive compressed with gzip, bzip2, xz or zstd")
    })?;
    let joined = Cursor::new(head).chain(input);
    let (reader, incomplete): (Box<dyn Read + 'a>, bool) = match compression {
        Compression::None => (Box::new(joined), false),
        Compression::Gzip => (Box::new(flate2::read::MultiGzDecoder::new(joined)), false),
        Compression::Bzip2 => (Box::new(bzip2::read::MultiBzDecoder::new(joined)), false),
        Compression::Zstd => (Box::new(Zstd::new(BufReader::new(joined))), false),
        Compression::Xz => {
            let mut spooled = tempfile::tempfile()?;
            let result = {
                let mut writer = io::BufWriter::new(&mut spooled);
                let result = lzma_rs::xz_decompress(&mut BufReader::new(joined), &mut writer);
                writer.flush()?;
                result
            };
            spooled.seek(SeekFrom::Start(0))?;
            (Box::new(BufReader::new(spooled)), result.is_err())
        }
    };
    Ok(Opened {
        compression,
        reader,
        incomplete,
    })
}

fn header(size: usize, mode: u32, mtime: u64, kind: tar::EntryType) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(size as u64);
    header.set_mode(mode);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(mtime);
    header.set_entry_type(kind);
    header
}

pub struct TarWriter {
    builder: tar::Builder<GzEncoder<Vec<u8>>>,
    mtime: u64,
}

impl TarWriter {
    pub fn gzip(mtime: u64) -> TarWriter {
        TarWriter {
            builder: tar::Builder::new(GzEncoder::new(Vec::new(), Level::best())),
            mtime,
        }
    }

    pub fn directory(&mut self, name: &str) -> io::Result<()> {
        let mut header = header(0, 0o755, self.mtime, tar::EntryType::Directory);
        self.builder
            .append_data(&mut header, format!("{name}/"), io::empty())
    }

    pub fn file(&mut self, name: &str, data: &[u8]) -> io::Result<()> {
        let mut header = header(data.len(), 0o644, self.mtime, tar::EntryType::Regular);
        self.builder.append_data(&mut header, name, data)
    }

    pub fn finish(self) -> io::Result<Vec<u8>> {
        self.builder.into_inner()?.finish()
    }
}

pub fn entry_name(entry: &tar::Entry<'_, impl Read>) -> String {
    let name = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
    name.strip_prefix("./").unwrap_or(&name).to_owned()
}

pub fn is_extension(entry: &tar::Entry<'_, impl Read>) -> bool {
    let kind = entry.header().entry_type();
    kind.is_pax_global_extensions() || kind.is_pax_local_extensions()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::compress as compressed;

    fn tar(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, data) in files {
            let mut header = header(data.len(), 0o644, 0, tar::EntryType::Regular);
            builder.append_data(&mut header, name, *data).unwrap();
        }
        builder.into_inner().unwrap()
    }

    #[test]
    fn every_compression_opens_to_the_tar_inside() {
        let data = tar(&[("a", b"first"), ("b", &[7; 5000])]);
        for compression in [
            Compression::None,
            Compression::Gzip,
            Compression::Bzip2,
            Compression::Xz,
            Compression::Zstd,
        ] {
            let file = compressed(&data, compression);
            let mut opened = open(file.as_slice()).unwrap();
            assert_eq!(opened.compression, compression);
            assert!(!opened.incomplete);
            let mut out = Vec::new();
            opened.reader.read_to_end(&mut out).unwrap();
            assert_eq!(out, data, "{compression:?}");
        }
        assert!(open(&b"not an archive"[..]).is_err());
    }

    #[test]
    fn zstd_streams_with_several_frames_and_skippable_frames_are_read_whole() {
        let one = compressed(b"first frame ", Compression::Zstd);
        let two = compressed(b"second frame", Compression::Zstd);
        let mut skippable = vec![0x50, 0x2a, 0x4d, 0x18, 3, 0, 0, 0];
        skippable.extend_from_slice(b"abc");
        let file = [one, skippable, two].concat();
        let mut out = Vec::new();
        Zstd::new(file.as_slice()).read_to_end(&mut out).unwrap();
        assert_eq!(out, b"first frame second frame");
    }

    #[test]
    fn written_archives_hold_directories_and_files() {
        let mut writer = TarWriter::gzip(1_700_000_000);
        writer.directory("hello-1.0-1").unwrap();
        writer.file("hello-1.0-1/desc", b"%NAME%\nhello\n").unwrap();
        let file = writer.finish().unwrap();
        let mut opened = open(file.as_slice()).unwrap();
        assert_eq!(opened.compression, Compression::Gzip);
        let mut archive = tar::Archive::new(&mut opened.reader);
        let names: Vec<(String, bool)> = archive
            .entries()
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry_name(&entry), entry.header().entry_type().is_dir())
            })
            .collect();
        assert_eq!(
            names,
            [
                ("hello-1.0-1/".to_owned(), true),
                ("hello-1.0-1/desc".to_owned(), false)
            ]
        );
    }
}
