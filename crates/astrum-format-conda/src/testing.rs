use std::io::Write;

use astrum_content::zip;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub fn tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
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

pub fn bzip2(data: &[u8]) -> Vec<u8> {
    let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
    encoder
        .write_all(data)
        .expect("compressing in memory does not fail");
    encoder
        .finish()
        .expect("compressing in memory does not fail")
}

pub fn zstd(data: &[u8]) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(data, ruzstd::encoding::CompressionLevel::Fastest)
}

/// An `info/index.json` of a test package.
pub fn index(name: &str, version: &str, build: &str, subdir: &str) -> Value {
    let mut index = json!({
        "name": name,
        "version": version,
        "build": build,
        "build_number": build.rsplit('_').next().and_then(|n| n.parse::<i64>().ok()).unwrap_or(0),
        "depends": [],
        "license": "MIT",
        "subdir": subdir,
        "timestamp": 1_760_000_000_000_i64,
    });
    if subdir == "noarch" {
        index["noarch"] = json!("generic");
    }
    index
}

type Files = Vec<(String, Vec<u8>)>;

fn parts(index: &Value, about: Option<&Value>) -> (Files, Files) {
    let name = index["name"].as_str().unwrap_or("package");
    let payload_path = format!("share/{name}/README.txt");
    let payload = format!(
        "{name} {} of the Astrum tests\n",
        index["version"].as_str().unwrap_or("")
    )
    .into_bytes();
    let paths = json!({
        "paths": [{
            "_path": payload_path,
            "path_type": "hardlink",
            "sha256": hex::encode(Sha256::digest(&payload)),
            "size_in_bytes": payload.len(),
        }],
        "paths_version": 1,
    });
    let mut info = vec![
        ("info/index.json".to_owned(), index.to_string().into_bytes()),
        ("info/paths.json".to_owned(), paths.to_string().into_bytes()),
        (
            "info/files".to_owned(),
            format!("{payload_path}\n").into_bytes(),
        ),
    ];
    if let Some(about) = about {
        info.push(("info/about.json".to_owned(), about.to_string().into_bytes()));
    }
    (info, vec![(payload_path, payload)])
}

fn tar_of(entries: &Files) -> Vec<u8> {
    let borrowed: Vec<(&str, &[u8])> = entries
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    tar(&borrowed)
}

/// A `.tar.bz2` package with its `info/` folder first, as conda-build writes it.
pub fn tar_bz2(index: &Value, about: Option<&Value>) -> Vec<u8> {
    let (mut entries, payload) = parts(index, about);
    entries.extend(payload);
    bzip2(&tar_of(&entries))
}

/// A `.conda` package: a zip with `metadata.json`, `info-<stem>.tar.zst` and `pkg-<stem>.tar.zst`.
pub fn conda(index: &Value, about: Option<&Value>) -> Vec<u8> {
    let (info, payload) = parts(index, about);
    let stem = format!(
        "{}-{}-{}",
        index["name"].as_str().unwrap_or(""),
        index["version"].as_str().unwrap_or(""),
        index["build"].as_str().unwrap_or("")
    );
    let info_name = format!("info-{stem}.tar.zst");
    let pkg_name = format!("pkg-{stem}.tar.zst");
    let info = zstd(&tar_of(&info));
    let pkg = zstd(&tar_of(&payload));
    zip::write(&[
        (
            "metadata.json",
            br#"{"conda_pkg_format_version": 2}"#.as_slice(),
        ),
        (pkg_name.as_str(), pkg.as_slice()),
        (info_name.as_str(), info.as_slice()),
    ])
}

pub fn unbzip2(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut bzip2::read::MultiBzDecoder::new(data), &mut out)
        .expect("the data is bzip2");
    out
}

pub fn unzstd(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut decoder =
        ruzstd::decoding::StreamingDecoder::new(data).expect("the data is a zstd frame");
    std::io::Read::read_to_end(&mut decoder, &mut out).expect("the data is zstd");
    out
}
