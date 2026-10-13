pub const REPODATA: &str = "repodata";
pub const REPOMD: &str = "repomd.xml";
pub const SIGNATURE: &str = "repomd.xml.asc";
pub const PUBLIC_KEY: &str = "repomd.xml.key";

pub fn segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

pub fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub fn directory(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) | None => "",
        Some(at) => &path[..at],
    }
}

pub fn is_rpm(path: &str) -> bool {
    path.ends_with(".rpm")
}

pub fn in_repodata(path: &str) -> bool {
    segments(path).contains(&REPODATA)
}

pub fn base(package: &str, depth: usize) -> Option<String> {
    let segments = segments(package);
    if segments.len().saturating_sub(1) < depth {
        return None;
    }
    Some(match depth {
        0 => String::new(),
        _ => format!("/{}", segments[..depth].join("/")),
    })
}

pub fn base_of_metadata(path: &str) -> Option<String> {
    let segments = segments(path);
    if segments.len() < 2 || segments[segments.len() - 2] != REPODATA {
        return None;
    }
    let base = &segments[..segments.len() - 2];
    Some(match base.is_empty() {
        true => String::new(),
        false => format!("/{}", base.join("/")),
    })
}

pub fn depth_of(base: &str) -> usize {
    segments(base).len()
}

pub fn repodata_directory(base: &str) -> String {
    format!("{base}/{REPODATA}")
}

pub fn repomd_path(base: &str) -> String {
    format!("{base}/{REPODATA}/{REPOMD}")
}

pub fn signature_path(base: &str) -> String {
    format!("{base}/{REPODATA}/{SIGNATURE}")
}

pub fn location(base: &str, package: &str) -> String {
    package[base.len() + 1..].to_owned()
}

pub fn join(base: &str, relative: &str) -> Option<String> {
    let mut parts: Vec<&str> = segments(base);
    for segment in relative.split('/').filter(|segment| !segment.is_empty()) {
        match segment {
            "." => {}
            ".." => return None,
            segment if segment.chars().any(char::is_control) => return None,
            segment => parts.push(segment),
        }
    }
    Some(format!("/{}", parts.join("/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bases_follow_the_repodata_depth() {
        assert_eq!(base("/a.rpm", 0), Some(String::new()));
        assert_eq!(base("/el9/x86_64/a.rpm", 2), Some("/el9/x86_64".to_owned()));
        assert_eq!(
            base("/el9/x86_64/Packages/a.rpm", 2),
            Some("/el9/x86_64".to_owned())
        );
        assert_eq!(base("/el9/a.rpm", 2), None);
        assert_eq!(
            base_of_metadata("/repodata/repomd.xml"),
            Some(String::new())
        );
        assert_eq!(
            base_of_metadata("/el9/repodata/x-primary.xml.gz"),
            Some("/el9".to_owned())
        );
        assert_eq!(base_of_metadata("/el9/repomd.xml"), None);
        assert_eq!(repomd_path(""), "/repodata/repomd.xml");
        assert_eq!(location("/el9", "/el9/Packages/a.rpm"), "Packages/a.rpm");
        assert_eq!(location("", "/a.rpm"), "a.rpm");
        assert_eq!(depth_of("/el9/x86_64"), 2);
        assert_eq!(directory("/repodata/repomd.xml"), "/repodata");
        assert_eq!(directory("/a.rpm"), "");
        assert!(in_repodata("/el9/repodata/x"));
        assert!(!in_repodata("/el9/repodatax/x"));
        assert_eq!(
            join("/el9", "repodata/x.xml.gz"),
            Some("/el9/repodata/x.xml.gz".to_owned())
        );
        assert_eq!(join("", "../x"), None);
    }
}
