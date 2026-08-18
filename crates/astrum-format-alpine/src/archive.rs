use std::io::{Read, Write};

use flate2::Compression;
use flate2::write::GzEncoder;

pub const SIGNATURE_PREFIX: &str = ".SIGN.";
const BLOCK: usize = 512;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArchiveError {
    #[error("the file ends inside a gzip stream")]
    Truncated,
    #[error("the file is not a gzip stream")]
    NotGzip,
    #[error("a gzip stream of the file is damaged: {0}")]
    Damaged(String),
    #[error("a gzip stream of the file unpacks to more than {0} bytes")]
    TooLarge(u64),
    #[error("a tar archive of the file is damaged: {0}")]
    Tar(String),
}

#[derive(Debug)]
pub struct Member<'a> {
    pub bytes: &'a [u8],
    pub content: Vec<u8>,
}

pub fn member(data: &[u8], limit: u64) -> Result<Member<'_>, ArchiveError> {
    if data.len() < 2 || data[0] != 0x1f || data[1] != 0x8b {
        return Err(
            if data.len() < 2 && data.first().is_none_or(|byte| *byte == 0x1f) {
                ArchiveError::Truncated
            } else {
                ArchiveError::NotGzip
            },
        );
    }
    let mut decoder = flate2::bufread::GzDecoder::new(data);
    let mut content = Vec::new();
    match (&mut decoder).take(limit + 1).read_to_end(&mut content) {
        Ok(_) if content.len() as u64 > limit => return Err(ArchiveError::TooLarge(limit)),
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(ArchiveError::Truncated);
        }
        Err(error) => return Err(ArchiveError::Damaged(error.to_string())),
    }
    let rest = decoder.into_inner();
    let used = data.len() - rest.len();
    Ok(Member {
        bytes: &data[..used],
        content,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
}

pub fn entries(tar: &[u8], wanted: impl Fn(&str) -> bool) -> Result<Vec<Entry>, ArchiveError> {
    let mut archive = tar::Archive::new(tar);
    let mut found = Vec::new();
    let invalid = |error: std::io::Error| ArchiveError::Tar(error.to_string());
    for entry in archive.entries().map_err(invalid)? {
        let mut entry = entry.map_err(invalid)?;
        let kind = entry.header().entry_type();
        if kind.is_pax_global_extensions() || kind.is_pax_local_extensions() {
            continue;
        }
        let name = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
        let name = name.strip_prefix("./").unwrap_or(&name).to_owned();
        let mut data = Vec::new();
        if wanted(&name) {
            entry.read_to_end(&mut data).map_err(invalid)?;
        }
        found.push(Entry { name, data });
    }
    Ok(found)
}

pub fn is_signature(name: &str) -> bool {
    name.starts_with(SIGNATURE_PREFIX) && !name.contains('/')
}

#[derive(Debug)]
pub struct Signed<'a> {
    pub signatures: Vec<Member<'a>>,
    pub names: Vec<String>,
    pub rest: &'a [u8],
}

pub fn split(file: &[u8], limit: u64) -> Result<Signed<'_>, ArchiveError> {
    let mut signatures = Vec::new();
    let mut names = Vec::new();
    let mut offset = 0;
    while offset < file.len() {
        let found = member(&file[offset..], limit)?;
        let listed = entries(&found.content, |_| false)?;
        if listed.is_empty() || !listed.iter().all(|entry| is_signature(&entry.name)) {
            break;
        }
        names.extend(listed.into_iter().map(|entry| entry.name));
        offset += found.bytes.len();
        signatures.push(found);
    }
    Ok(Signed {
        signatures,
        names,
        rest: &file[offset..],
    })
}

fn header(size: usize, mtime: u64) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(size as u64);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(mtime);
    header.set_entry_type(tar::EntryType::Regular);
    header
}

pub fn tar(files: &[(&str, &[u8])], mtime: u64, cut: bool) -> std::io::Result<Vec<u8>> {
    let mut builder = tar::Builder::new(Vec::new());
    for (name, data) in files {
        let mut header = header(data.len(), mtime);
        builder.append_data(&mut header, name, *data)?;
    }
    let mut bytes = builder.into_inner()?;
    if cut {
        bytes.truncate(bytes.len() - 2 * BLOCK);
    }
    Ok(bytes)
}

pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(data)
        .expect("writing into memory does not fail");
    encoder.finish().expect("writing into memory does not fail")
}

pub fn signature_member(file_name: &str, signature: &[u8], mtime: u64) -> Vec<u8> {
    let tar = tar(&[(file_name, signature)], mtime, true)
        .expect("a signature file name is a valid tar path");
    gzip(&tar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concatenated_gzip_streams_are_read_one_at_a_time() {
        let first = gzip(b"first stream");
        let second = gzip(b"second");
        let file = [first.clone(), second.clone(), b"tail".to_vec()].concat();
        let one = member(&file, 1024).unwrap();
        assert_eq!(one.bytes, first.as_slice());
        assert_eq!(one.content, b"first stream");
        let two = member(&file[first.len()..], 1024).unwrap();
        assert_eq!(two.bytes, second.as_slice());
        assert_eq!(two.content, b"second");
        assert_eq!(
            member(&file[first.len() + second.len()..], 1024).unwrap_err(),
            ArchiveError::NotGzip
        );
        assert_eq!(
            member(&first[..first.len() - 3], 1024).unwrap_err(),
            ArchiveError::Truncated
        );
        assert_eq!(
            member(&gzip(&[0; 4096]), 100).unwrap_err(),
            ArchiveError::TooLarge(100)
        );
    }

    #[test]
    fn cut_archives_have_no_end_marker_and_still_list() {
        let whole = tar(&[("a", b"x")], 7, false).unwrap();
        let cut = tar(&[("a", b"x")], 7, true).unwrap();
        assert_eq!(whole.len(), cut.len() + 1024);
        assert_eq!(cut.len(), 1024);
        let listed = entries(&cut, |_| true).unwrap();
        assert_eq!(
            listed,
            [Entry {
                name: "a".to_owned(),
                data: b"x".to_vec()
            }]
        );
    }

    #[test]
    fn signature_streams_are_split_off() {
        let signature = signature_member(".SIGN.RSA.a.rsa.pub", &[1; 256], 0);
        let other = signature_member(".SIGN.RSA.b.rsa.pub", &[2; 256], 0);
        let index = gzip(&tar(&[("APKINDEX", b"P:x\n")], 0, false).unwrap());
        let file = [signature.clone(), other.clone(), index.clone()].concat();
        let signed = split(&file, 1 << 20).unwrap();
        assert_eq!(signed.signatures.len(), 2);
        assert_eq!(signed.names, [".SIGN.RSA.a.rsa.pub", ".SIGN.RSA.b.rsa.pub"]);
        assert_eq!(signed.rest, index.as_slice());
        let unsigned = split(&index, 1 << 20).unwrap();
        assert!(unsigned.signatures.is_empty());
        assert_eq!(unsigned.rest, index.as_slice());
    }
}
