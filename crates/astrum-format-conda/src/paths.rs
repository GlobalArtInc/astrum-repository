pub const REPODATA: &str = "repodata.json";
pub const CURRENT: &str = "current_repodata.json";
pub const CHANNELDATA: &str = "channeldata.json";
pub const TAR_BZ2: &str = ".tar.bz2";
pub const CONDA: &str = ".conda";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IndexFile {
    Json,
    Bz2,
    Zst,
    Current,
}

impl IndexFile {
    pub const ALL: [IndexFile; 4] = [
        IndexFile::Json,
        IndexFile::Bz2,
        IndexFile::Zst,
        IndexFile::Current,
    ];

    pub fn name(self) -> &'static str {
        match self {
            IndexFile::Json => REPODATA,
            IndexFile::Bz2 => "repodata.json.bz2",
            IndexFile::Zst => "repodata.json.zst",
            IndexFile::Current => CURRENT,
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            IndexFile::Json | IndexFile::Current => "application/json",
            IndexFile::Bz2 => "application/x-bzip2",
            IndexFile::Zst => "application/zstd",
        }
    }

    fn of(name: &str) -> Option<IndexFile> {
        IndexFile::ALL.into_iter().find(|file| file.name() == name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageType {
    TarBz2,
    Conda,
}

impl PackageType {
    pub fn of(file: &str) -> Option<PackageType> {
        if file.ends_with(TAR_BZ2) {
            Some(PackageType::TarBz2)
        } else if file.ends_with(CONDA) {
            Some(PackageType::Conda)
        } else {
            None
        }
    }

    pub fn sniff(prefix: &[u8]) -> Option<PackageType> {
        if prefix.starts_with(b"PK\x03\x04") {
            Some(PackageType::Conda)
        } else if prefix.starts_with(b"BZh") {
            Some(PackageType::TarBz2)
        } else {
            None
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            PackageType::TarBz2 => TAR_BZ2,
            PackageType::Conda => CONDA,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            PackageType::TarBz2 => "tar.bz2",
            PackageType::Conda => "conda",
        }
    }

    /// The key of the repodata object that lists packages of this type.
    pub fn key(self) -> &'static str {
        match self {
            PackageType::TarBz2 => "packages",
            PackageType::Conda => "packages.conda",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            PackageType::TarBz2 => "application/x-bzip2",
            PackageType::Conda => "application/octet-stream",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileName {
    pub name: String,
    pub version: String,
    pub build: String,
    pub kind: PackageType,
}

impl FileName {
    pub fn parse(file: &str) -> Option<FileName> {
        let kind = PackageType::of(file)?;
        let stem = file.strip_suffix(kind.extension())?;
        let mut parts = stem.rsplitn(3, '-');
        let build = parts.next()?;
        let version = parts.next()?;
        let name = parts.next()?;
        (!name.is_empty() && !version.is_empty() && !build.is_empty()).then(|| FileName {
            name: name.to_owned(),
            version: version.to_owned(),
            build: build.to_owned(),
            kind,
        })
    }

    pub fn stem(name: &str, version: &str, build: &str) -> String {
        format!("{name}-{version}-{build}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Directory,
    Index {
        channel: String,
        subdir: String,
        file: IndexFile,
    },
    Channeldata {
        channel: String,
    },
    Package {
        channel: String,
        subdir: String,
        file: FileName,
    },
    Unsupported,
    Other,
}

/// A platform subdirectory of a channel: `noarch` or `<platform>-<architecture>`, such as
/// `linux-64`, `osx-arm64` or `emscripten-wasm32`.
pub fn valid_subdir(subdir: &str) -> bool {
    if subdir == "noarch" {
        return true;
    }
    let Some((platform, architecture)) = subdir.split_once('-') else {
        return false;
    };
    let part = |text: &str| {
        !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    };
    part(platform) && part(architecture)
}

pub fn valid_channel(channel: &str) -> bool {
    channel.is_empty()
        || channel.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        })
}

pub fn join(channel: &str, rest: &str) -> String {
    if channel.is_empty() {
        format!("/{rest}")
    } else {
        format!("/{channel}/{rest}")
    }
}

pub fn package_path(channel: &str, subdir: &str, file: &str) -> String {
    join(channel, &format!("{subdir}/{file}"))
}

pub fn index_path(channel: &str, subdir: &str, file: IndexFile) -> String {
    join(channel, &format!("{subdir}/{}", file.name()))
}

pub fn channeldata_path(channel: &str) -> String {
    join(channel, CHANNELDATA)
}

fn unsupported(file: &str) -> bool {
    file.ends_with(".jlap") || file.starts_with("repodata_shards") || file.ends_with(".msgpack.zst")
}

pub fn route(path: &str) -> Route {
    if path.is_empty() || path.ends_with('/') {
        return Route::Directory;
    }
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let Some((file, parents)) = segments.split_last() else {
        return Route::Other;
    };
    if *file == CHANNELDATA {
        let channel = parents.join("/");
        return if valid_channel(&channel) {
            Route::Channeldata { channel }
        } else {
            Route::Other
        };
    }
    let Some((subdir, channel)) = parents.split_last() else {
        return Route::Other;
    };
    let channel = channel.join("/");
    if !valid_subdir(subdir) || !valid_channel(&channel) {
        return Route::Other;
    }
    if unsupported(file) {
        return Route::Unsupported;
    }
    if let Some(index) = IndexFile::of(file) {
        return Route::Index {
            channel,
            subdir: (*subdir).to_owned(),
            file: index,
        };
    }
    match FileName::parse(file) {
        Some(parsed) => Route::Package {
            channel,
            subdir: (*subdir).to_owned(),
            file: parsed,
        },
        None => Route::Other,
    }
}

pub fn file_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The channel and subdirectory of a package stored at `/<channel>/<subdir>/<file>`.
pub fn place_of(path: &str) -> Option<(String, String)> {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (_, parents) = segments.split_last()?;
    let (subdir, channel) = parents.split_last()?;
    Some((channel.join("/"), (*subdir).to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_indexes_channels_and_packages() {
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/main/"), Route::Directory);
        assert_eq!(
            route("/main/linux-64/repodata.json"),
            Route::Index {
                channel: "main".to_owned(),
                subdir: "linux-64".to_owned(),
                file: IndexFile::Json
            }
        );
        assert_eq!(
            route("/noarch/repodata.json.zst"),
            Route::Index {
                channel: String::new(),
                subdir: "noarch".to_owned(),
                file: IndexFile::Zst
            }
        );
        assert_eq!(
            route("/a/b/osx-arm64/current_repodata.json"),
            Route::Index {
                channel: "a/b".to_owned(),
                subdir: "osx-arm64".to_owned(),
                file: IndexFile::Current
            }
        );
        assert_eq!(
            route("/main/channeldata.json"),
            Route::Channeldata {
                channel: "main".to_owned()
            }
        );
        assert_eq!(
            route("/channeldata.json"),
            Route::Channeldata {
                channel: String::new()
            }
        );
        assert_eq!(
            route("/main/linux-64/numpy-1.26.4-py312h_0.conda"),
            Route::Package {
                channel: "main".to_owned(),
                subdir: "linux-64".to_owned(),
                file: FileName {
                    name: "numpy".to_owned(),
                    version: "1.26.4".to_owned(),
                    build: "py312h_0".to_owned(),
                    kind: PackageType::Conda
                }
            }
        );
        assert_eq!(route("/main/noarch/repodata.jlap"), Route::Unsupported);
        assert_eq!(
            route("/main/noarch/repodata_shards.msgpack.zst"),
            Route::Unsupported
        );
        assert_eq!(route("/repodata.json"), Route::Other);
        assert_eq!(route("/main/Linux64/repodata.json"), Route::Other);
        assert_eq!(route("/main/noarch/readme.txt"), Route::Other);
    }

    #[test]
    fn file_names_split_from_the_right() {
        let parsed = FileName::parse("python-dateutil-2.9.0-pyhd8ed1ab_1.tar.bz2").unwrap();
        assert_eq!(parsed.name, "python-dateutil");
        assert_eq!(parsed.version, "2.9.0");
        assert_eq!(parsed.build, "pyhd8ed1ab_1");
        assert_eq!(parsed.kind, PackageType::TarBz2);
        assert!(FileName::parse("tool-1.0.conda").is_none());
        assert!(FileName::parse("tool-1.0-0.zip").is_none());
        assert_eq!(
            place_of("/a/b/linux-64/x-1-0.conda"),
            Some(("a/b".to_owned(), "linux-64".to_owned()))
        );
        assert_eq!(
            place_of("/noarch/x-1-0.conda"),
            Some((String::new(), "noarch".to_owned()))
        );
    }

    #[test]
    fn subdirs_and_channels_are_checked() {
        for good in [
            "noarch",
            "linux-64",
            "osx-arm64",
            "win-32",
            "emscripten-wasm32",
            "zos-z",
        ] {
            assert!(valid_subdir(good), "{good}");
        }
        for bad in ["main", "Linux-64", "linux-", "-64", "linux 64"] {
            assert!(!valid_subdir(bad), "{bad}");
        }
        assert!(valid_channel(""));
        assert!(valid_channel("team/main"));
        assert!(!valid_channel("team//main"));
        assert!(!valid_channel("../main"));
    }
}
