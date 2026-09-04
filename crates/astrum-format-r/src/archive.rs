use std::io::{self, BufReader, Read, Write};
use std::sync::Arc;

use astrum_blob::{BlobId, BlobStore, ContentStream};
use astrum_content::{ContentError, zip};
use bytes::Bytes;
use futures_util::StreamExt;

use crate::dcf::{self, Record};

const MAX_DESCRIPTION: u64 = 1024 * 1024;
const MAX_XZ_PREFIX: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Packing {
    Gzip,
    Bzip2,
    Xz,
    Zip,
}

impl Packing {
    pub fn sniff(head: &[u8]) -> Option<Packing> {
        if head.starts_with(&[0x1f, 0x8b]) {
            Some(Packing::Gzip)
        } else if head.starts_with(b"BZh") {
            Some(Packing::Bzip2)
        } else if head.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00]) {
            Some(Packing::Xz)
        } else if head.starts_with(b"PK\x03\x04") {
            Some(Packing::Zip)
        } else {
            None
        }
    }

    pub fn of_extension(extension: &str) -> Option<Packing> {
        match extension {
            ".tar.gz" | ".tgz" => Some(Packing::Gzip),
            ".tar.bz2" => Some(Packing::Bzip2),
            ".tar.xz" => Some(Packing::Xz),
            ".zip" => Some(Packing::Zip),
            _ => None,
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Packing::Gzip => "gzip",
            Packing::Bzip2 => "bzip2",
            Packing::Xz => "xz",
            Packing::Zip => "zip",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    pub folder: String,
    pub record: Record,
}

fn bad(message: impl Into<String>) -> ContentError {
    ContentError::BadRequest(message.into())
}

fn is_description(path: &str) -> Option<String> {
    let path = path.trim_start_matches("./");
    let (folder, file) = path.split_once('/')?;
    (file == "DESCRIPTION" && !folder.is_empty()).then(|| folder.to_owned())
}

fn record_of(folder: String, bytes: &[u8]) -> Result<Description, ContentError> {
    let record = dcf::parse(&dcf::decode(bytes))
        .into_iter()
        .next()
        .ok_or_else(|| bad("the DESCRIPTION file of the package is empty"))?;
    Ok(Description { folder, record })
}

pub fn from_tar(reader: impl Read) -> Result<Description, ContentError> {
    let mut archive = tar::Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|error| bad(format!("the package archive cannot be read: {error}")))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| bad(format!("the package archive cannot be read: {error}")))?;
        let path = entry
            .path()
            .map_err(|error| bad(format!("the package archive cannot be read: {error}")))?
            .to_string_lossy()
            .into_owned();
        if let Some(folder) = is_description(&path) {
            let mut bytes = Vec::new();
            entry
                .take(MAX_DESCRIPTION)
                .read_to_end(&mut bytes)
                .map_err(|error| bad(format!("the DESCRIPTION file cannot be read: {error}")))?;
            return record_of(folder, &bytes);
        }
    }
    Err(bad("the package archive has no <package>/DESCRIPTION file"))
}

struct Limited(Vec<u8>);

impl Write for Limited {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.0.len() + data.len() > MAX_XZ_PREFIX {
            return Err(io::Error::other("enough"));
        }
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn from_packed(reader: impl Read, packing: Packing) -> Result<Description, ContentError> {
    match packing {
        Packing::Gzip => from_tar(flate2::read::MultiGzDecoder::new(reader)),
        Packing::Bzip2 => from_tar(bzip2::read::MultiBzDecoder::new(reader)),
        Packing::Xz => {
            let mut out = Limited(Vec::new());
            let mut input = BufReader::new(reader);
            let result = lzma_rs::xz_decompress(&mut input, &mut out);
            if out.0.is_empty()
                && let Err(error) = result
            {
                return Err(bad(format!("the package archive cannot be read: {error}")));
            }
            from_tar(out.0.as_slice())
        }
        Packing::Zip => Err(bad("a zip archive is read from its directory")),
    }
}

struct Channel {
    receiver: tokio::sync::mpsc::Receiver<io::Result<Bytes>>,
    current: Bytes,
}

impl Read for Channel {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        while self.current.is_empty() {
            match self.receiver.blocking_recv() {
                Some(Ok(chunk)) => self.current = chunk,
                Some(Err(error)) => return Err(error),
                None => return Ok(0),
            }
        }
        let length = buffer.len().min(self.current.len());
        buffer[..length].copy_from_slice(&self.current[..length]);
        self.current = self.current.slice(length..);
        Ok(length)
    }
}

async fn from_stream(stream: ContentStream, packing: Packing) -> Result<Description, ContentError> {
    let (sender, receiver) = tokio::sync::mpsc::channel(4);
    let feeder = tokio::spawn(async move {
        let mut stream = stream;
        while let Some(chunk) = stream.next().await {
            if sender.send(chunk).await.is_err() {
                break;
            }
        }
    });
    let reader = Channel {
        receiver,
        current: Bytes::new(),
    };
    let result = tokio::task::spawn_blocking(move || from_packed(reader, packing))
        .await
        .map_err(ContentError::internal);
    feeder.abort();
    result?
}

pub async fn sniff_blob(
    store: &Arc<dyn BlobStore>,
    id: BlobId,
    size: u64,
) -> Result<Option<Packing>, ContentError> {
    let Some(content) = store.read_range(id, 0, size.min(8)).await? else {
        return Err(ContentError::internal(io::Error::other(
            "a blob that was just written is missing",
        )));
    };
    let mut head = Vec::new();
    let mut stream = content.stream;
    while let Some(chunk) = stream.next().await {
        head.extend_from_slice(&chunk.map_err(ContentError::internal)?);
    }
    Ok(Packing::sniff(&head))
}

pub async fn from_blob(
    store: Arc<dyn BlobStore>,
    id: BlobId,
    size: u64,
    packing: Packing,
) -> Result<Description, ContentError> {
    if packing == Packing::Zip {
        let source = zip::Blob { store, id, size };
        let entries = zip::entries(&source)
            .await
            .map_err(|error| bad(error.to_string()))?;
        let found = entries
            .iter()
            .filter(|entry| !entry.is_directory())
            .find_map(|entry| is_description(&entry.name).map(|folder| (folder, entry)))
            .ok_or_else(|| bad("the package archive has no <package>/DESCRIPTION file"))?;
        let bytes = zip::read(&source, found.1, MAX_DESCRIPTION)
            .await
            .map_err(|error| bad(error.to_string()))?;
        return record_of(found.0, &bytes);
    }
    let Some(content) = store.read(id).await? else {
        return Err(ContentError::internal(io::Error::other(
            "a blob that was just written is missing",
        )));
    };
    from_stream(content.stream, packing).await
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn tarball(folder: &str, description: &str, packing: Packing) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, body) in [
            (format!("{folder}/NAMESPACE"), "export(hello)\n".to_owned()),
            (format!("{folder}/DESCRIPTION"), description.to_owned()),
            (
                format!("{folder}/R/hello.R"),
                "hello <- function() 1\n".to_owned(),
            ),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, name, body.as_bytes())
                .unwrap();
        }
        let tar = builder.into_inner().unwrap();
        match packing {
            Packing::Gzip => {
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
                encoder.write_all(&tar).unwrap();
                encoder.finish().unwrap()
            }
            Packing::Bzip2 => {
                let mut encoder =
                    bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
                encoder.write_all(&tar).unwrap();
                encoder.finish().unwrap()
            }
            Packing::Xz => {
                let mut out = Vec::new();
                lzma_rs::xz_compress(&mut tar.as_slice(), &mut out).unwrap();
                out
            }
            Packing::Zip => zip::write(&[
                (&format!("{folder}/DESCRIPTION"), description.as_bytes()),
                (&format!("{folder}/NAMESPACE"), b"export(hello)\n"),
            ]),
        }
    }

    #[test]
    fn descriptions_are_found_in_every_packing() {
        let text = "Package: hello1\nVersion: 1.0\nImports:\n    utils\n";
        for packing in [Packing::Gzip, Packing::Bzip2, Packing::Xz] {
            let packed = tarball("hello1", text, packing);
            assert_eq!(Packing::sniff(&packed), Some(packing));
            let found = from_packed(packed.as_slice(), packing).unwrap();
            assert_eq!(found.folder, "hello1");
            assert_eq!(found.record.get("Imports"), Some("utils"));
        }
        assert_eq!(
            Packing::sniff(&tarball("x", text, Packing::Zip)),
            Some(Packing::Zip)
        );
        assert_eq!(Packing::sniff(b"hello"), None);
    }

    #[test]
    fn archives_without_a_description_are_refused() {
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(2);
        header.set_cksum();
        builder
            .append_data(&mut header, "hello1/inst/DESCRIPTION", &b"x\n"[..])
            .unwrap();
        let tar = builder.into_inner().unwrap();
        assert!(from_tar(tar.as_slice()).is_err());
        assert!(from_packed(&b"not gzip"[..], Packing::Gzip).is_err());
    }

    #[tokio::test]
    async fn streams_are_read_on_a_blocking_thread() {
        let packed = tarball("hello1", "Package: hello1\nVersion: 2.0\n", Packing::Gzip);
        let chunks: Vec<io::Result<Bytes>> = packed
            .chunks(7)
            .map(|chunk| Ok(Bytes::copy_from_slice(chunk)))
            .collect();
        let stream: ContentStream = futures_util::stream::iter(chunks).boxed();
        let found = from_stream(stream, Packing::Gzip).await.unwrap();
        assert_eq!(found.record.get("Version"), Some("2.0"));
    }
}
