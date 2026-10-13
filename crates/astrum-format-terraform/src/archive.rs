use std::collections::BTreeSet;

const MAX_ENTRIES: usize = 100_000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Layout {
    pub directories: BTreeSet<String>,
    pub root_files: bool,
}

impl Layout {
    pub fn single_top_directory(&self) -> Option<String> {
        if self.root_files {
            return None;
        }
        let top: Vec<&String> = self
            .directories
            .iter()
            .filter(|directory| !directory.contains('/'))
            .collect();
        match top.as_slice() {
            [only] => Some((*only).clone()),
            _ => None,
        }
    }

    fn add(&mut self, entry: &str, directory: bool) {
        let name = trim(&entry.replace('\\', "/"));
        if name.is_empty() || name == "pax_global_header" {
            return;
        }
        let parent = match (directory, name.rfind('/')) {
            (true, _) => name.as_str(),
            (false, Some(slash)) => &name[..slash],
            (false, None) => {
                self.root_files = true;
                return;
            }
        };
        let mut current = parent.to_owned();
        while !current.is_empty() && self.directories.insert(current.clone()) {
            current = match current.rfind('/') {
                Some(slash) => current[..slash].to_owned(),
                None => String::new(),
            };
        }
    }
}

fn trim(path: &str) -> String {
    let mut value = path;
    while let Some(rest) = value.strip_prefix("./") {
        value = rest;
    }
    value.trim_matches('/').to_owned()
}

pub fn layout(data: &[u8], extension: &str) -> Option<Layout> {
    match extension {
        "tar.gz" => tar_layout(data),
        "zip" => zip_layout(data),
        _ => None,
    }
}

fn tar_layout(data: &[u8]) -> Option<Layout> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(data));
    let mut layout = Layout::default();
    for (count, entry) in archive.entries().ok()?.enumerate() {
        if count >= MAX_ENTRIES {
            break;
        }
        let entry = entry.ok()?;
        let directory = entry.header().entry_type().is_dir();
        let path = entry.path().ok()?.to_string_lossy().into_owned();
        layout.add(&path, directory);
    }
    Some(layout)
}

fn u16_at(data: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?) as usize)
}

fn u32_at(data: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?) as usize)
}

fn zip_layout(data: &[u8]) -> Option<Layout> {
    let search = data.len().saturating_sub(65_557);
    let end = (search..data.len().saturating_sub(21))
        .rev()
        .find(|&at| data[at..at + 4] == [0x50, 0x4b, 0x05, 0x06])?;
    let entries = u16_at(data, end + 10)?;
    let mut at = u32_at(data, end + 16)?;
    let mut layout = Layout::default();
    for _ in 0..entries.min(MAX_ENTRIES) {
        if data.get(at..at + 4)? != [0x50, 0x4b, 0x01, 0x02] {
            return None;
        }
        let name_length = u16_at(data, at + 28)?;
        let extra = u16_at(data, at + 30)?;
        let comment = u16_at(data, at + 32)?;
        let name = String::from_utf8_lossy(data.get(at + 46..at + 46 + name_length)?).into_owned();
        layout.add(&name, name.ends_with('/'));
        at += 46 + name_length + extra + comment;
    }
    Some(layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tar_gz(files: &[&str]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for file in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(1);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, file, &b"x"[..]).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn a_single_top_directory_is_found() {
        let layout = layout(&tar_gz(&["vpc/main.tf", "vpc/modules/a/x.tf"]), "tar.gz").unwrap();
        assert_eq!(layout.single_top_directory().as_deref(), Some("vpc"));
        let flat = layout_of(&["main.tf", "modules/a/x.tf"]);
        assert_eq!(flat.single_top_directory(), None);
        let two = layout_of(&["a/main.tf", "b/main.tf"]);
        assert_eq!(two.single_top_directory(), None);
        assert!(super::layout(b"not an archive", "tar.gz").is_none());
    }

    fn layout_of(files: &[&str]) -> Layout {
        layout(&tar_gz(files), "tar.gz").unwrap()
    }
}
