use crate::dcf;
use crate::index::{File, Kind};

pub const EXTENSIONS: [&str; 5] = [".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".zip"];

pub const PLACES: &str = "R packages go to src/contrib/, src/contrib/Archive/<package>/, bin/windows/contrib/<R version>/, bin/macosx/contrib/<R version>/ or bin/macosx/<platform>/contrib/<R version>/ as <package>_<version> with the extension of their kind";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFile {
    pub directory: String,
    pub name: String,
    pub version: String,
    pub extension: &'static str,
}

impl PackageFile {
    pub fn file_name(&self) -> String {
        format!("{}_{}{}", self.name, self.version, self.extension)
    }

    pub fn path(&self) -> String {
        format!("{}/{}", self.directory, self.file_name())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Index { directory: String, file: File },
    Package(PackageFile),
    Directory,
    Other,
}

pub fn package_file(path: &str) -> Option<PackageFile> {
    let (directory, file) = path.rsplit_once('/')?;
    let extension = EXTENSIONS
        .iter()
        .find(|extension| file.ends_with(**extension))?;
    let stem = &file[..file.len() - extension.len()];
    let (name, version) = stem.split_once('_')?;
    (dcf::valid_package(name) && dcf::valid_version(version)).then(|| PackageFile {
        directory: directory.to_owned(),
        name: name.to_owned(),
        version: version.to_owned(),
        extension,
    })
}

pub fn route(path: &str) -> Route {
    if path.ends_with('/') {
        return Route::Directory;
    }
    if let Some((directory, name)) = path.rsplit_once('/')
        && let Some(file) = File::parse(name)
    {
        return Route::Index {
            directory: directory.to_owned(),
            file,
        };
    }
    match package_file(path) {
        Some(file) => Route::Package(file),
        None => Route::Other,
    }
}

fn r_version(text: &str) -> bool {
    match text.split_once('.') {
        Some((major, minor)) => {
            !major.is_empty()
                && !minor.is_empty()
                && major.bytes().all(|byte| byte.is_ascii_digit())
                && minor.bytes().all(|byte| byte.is_ascii_digit())
        }
        None => false,
    }
}

fn platform(text: &str) -> bool {
    !text.is_empty()
        && text != "contrib"
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub fn standard_kind(directory: &str) -> Option<Kind> {
    let parts: Vec<&str> = directory.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["src", "contrib"] => Some(Kind::Source),
        ["bin", "windows", "contrib", version] if r_version(version) => Some(Kind::Windows),
        ["bin", "macosx", "contrib", version] if r_version(version) => Some(Kind::Mac),
        ["bin", "macosx", name, "contrib", version] if platform(name) && r_version(version) => {
            Some(Kind::Mac)
        }
        _ => None,
    }
}

pub fn archive_package(directory: &str) -> Option<&str> {
    directory
        .strip_prefix("/src/contrib/Archive/")
        .filter(|package| dcf::valid_package(package))
}

pub fn hosted_kind(directory: &str) -> Option<Kind> {
    if archive_package(directory).is_some() {
        return Some(Kind::Source);
    }
    standard_kind(directory)
}

pub fn content_type(extension: &str) -> &'static str {
    match extension {
        ".zip" => "application/zip",
        ".tar.bz2" => "application/x-bzip2",
        ".tar.xz" => "application/x-xz",
        _ => "application/x-gzip",
    }
}

pub fn is_archive(path: &str) -> bool {
    EXTENSIONS.iter().any(|extension| path.ends_with(extension))
}

pub fn windows_directory(built: &str) -> Option<String> {
    let version = built.strip_prefix("R ")?.split(';').next()?.trim();
    let mut parts = version.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    let directory = format!("/bin/windows/contrib/{major}.{minor}");
    standard_kind(&directory).map(|_| directory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_indexes_packages_and_directories() {
        assert_eq!(
            route("/src/contrib/PACKAGES.rds"),
            Route::Index {
                directory: "/src/contrib".to_owned(),
                file: File::Rds
            }
        );
        assert_eq!(
            route("/bin/windows/contrib/4.4/PACKAGES.gz"),
            Route::Index {
                directory: "/bin/windows/contrib/4.4".to_owned(),
                file: File::Gz
            }
        );
        assert_eq!(
            route("/src/contrib/data.table_1.15.4.tar.gz"),
            Route::Package(PackageFile {
                directory: "/src/contrib".to_owned(),
                name: "data.table".to_owned(),
                version: "1.15.4".to_owned(),
                extension: ".tar.gz",
            })
        );
        assert_eq!(route("/src/contrib/"), Route::Directory);
        for other in [
            "/src/contrib/README",
            "/src/contrib/a_1.tar.gz",
            "/src/contrib/x_1.0.tar",
            "/src/contrib/Meta/archive.rds",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        let file = package_file("/bin/macosx/big-sur-arm64/contrib/4.4/R6_2.5.1.tgz").unwrap();
        assert_eq!(file.file_name(), "R6_2.5.1.tgz");
        assert_eq!(
            file.path(),
            "/bin/macosx/big-sur-arm64/contrib/4.4/R6_2.5.1.tgz"
        );
    }

    #[test]
    fn hosted_packages_live_in_the_trees_r_reads() {
        assert_eq!(hosted_kind("/src/contrib"), Some(Kind::Source));
        assert_eq!(hosted_kind("/src/contrib/Archive/R6"), Some(Kind::Source));
        assert_eq!(hosted_kind("/bin/windows/contrib/4.4"), Some(Kind::Windows));
        assert_eq!(hosted_kind("/bin/macosx/contrib/4.1"), Some(Kind::Mac));
        assert_eq!(
            hosted_kind("/bin/macosx/big-sur-x86_64/contrib/4.4"),
            Some(Kind::Mac)
        );
        for other in [
            "",
            "/src",
            "/src/contrib/Archive",
            "/src/contrib/Archive/R6/old",
            "/bin/windows/contrib/4",
            "/bin/windows/contrib/latest",
            "/bin/linux/contrib/4.4",
            "/other/src/contrib",
        ] {
            assert_eq!(hosted_kind(other), None, "{other}");
        }
        assert_eq!(standard_kind("/src/contrib/Archive/R6"), None);
        assert_eq!(archive_package("/src/contrib/Archive/R6"), Some("R6"));
    }

    #[test]
    fn windows_binaries_find_their_tree_from_the_built_field() {
        assert_eq!(
            windows_directory("R 4.4.1; ; 2024-08-01 10:00:00 UTC; windows").as_deref(),
            Some("/bin/windows/contrib/4.4")
        );
        assert_eq!(windows_directory("4.4.1; windows"), None);
    }
}
