pub const ANY: &str = "any";
pub const SIGNATURE_SUFFIX: &str = ".sig";
pub const KEY_SUFFIX: &str = ".key";
const PACKAGE_MARK: &str = ".pkg.tar";
const COMPRESSIONS: &[&str] = &[
    "", ".gz", ".bz2", ".xz", ".zst", ".lz4", ".lzo", ".lrz", ".Z", ".lz",
];
const STORED_SUFFIX: &str = ".tar.gz";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variant {
    Db,
    Files,
}

impl Variant {
    pub fn extension(self) -> &'static str {
        match self {
            Variant::Db => ".db",
            Variant::Files => ".files",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Database {
    pub directory: String,
    pub name: String,
    pub variant: Variant,
    pub suffix: String,
}

impl Database {
    pub fn generated(&self) -> bool {
        self.suffix.is_empty() || self.suffix == STORED_SUFFIX
    }

    pub fn file_name(&self) -> String {
        format!("{}{}{}", self.name, self.variant.extension(), self.suffix)
    }

    pub fn path(&self) -> String {
        format!("{}/{}", self.directory, self.file_name())
    }

    pub fn signature_path(&self) -> String {
        format!("{}{SIGNATURE_SUFFIX}", self.path())
    }

    pub fn stored_path(&self) -> String {
        stored_path(&self.directory, &self.name, self.variant)
    }
}

pub fn stored_path(directory: &str, name: &str, variant: Variant) -> String {
    format!("{directory}/{name}{}{STORED_SUFFIX}", variant.extension())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFile {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub extension: String,
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with(['-', '.'])
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-'))
}

fn valid_part(part: &str) -> bool {
    !part.is_empty()
        && part
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '/' | '-' | '\\'))
}

pub fn valid_arch(arch: &str) -> bool {
    !arch.is_empty() && arch.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl PackageFile {
    pub fn parse(file: &str) -> Option<PackageFile> {
        let at = file.rfind(PACKAGE_MARK)?;
        let extension = &file[at + PACKAGE_MARK.len()..];
        if !COMPRESSIONS.contains(&extension) {
            return None;
        }
        let mut parts = file[..at].rsplitn(4, '-');
        let arch = parts.next()?;
        let release = parts.next()?;
        let version = parts.next()?;
        let name = parts.next()?;
        if !valid_name(name) || !valid_part(version) || !valid_part(release) || !valid_arch(arch) {
            return None;
        }
        Some(PackageFile {
            name: name.to_owned(),
            version: format!("{version}-{release}"),
            arch: arch.to_owned(),
            extension: extension.to_owned(),
        })
    }

    pub fn file_name(&self) -> String {
        format!(
            "{}-{}-{}{PACKAGE_MARK}{}",
            self.name, self.version, self.arch, self.extension
        )
    }
}

pub fn package_file_name(name: &str, version: &str, arch: &str, extension: &str) -> String {
    format!("{name}-{version}-{arch}{PACKAGE_MARK}{extension}")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Directory,
    Database(Database),
    DatabaseSignature(Database),
    Package(PackageFile),
    Signature(PackageFile),
    Key(String),
    File,
}

fn database(directory: &str, file: &str) -> Option<Database> {
    let found = [Variant::Db, Variant::Files]
        .into_iter()
        .find_map(|variant| {
            let extension = variant.extension();
            if let Some(name) = file.strip_suffix(extension) {
                return Some((name, variant, ""));
            }
            let marker = format!("{extension}.tar");
            let at = file.rfind(&marker)?;
            let rest = &file[at + marker.len()..];
            COMPRESSIONS
                .contains(&rest)
                .then(|| (&file[..at], variant, &file[at + extension.len()..]))
        });
    let (name, variant, suffix) = found?;
    if name.is_empty() || name.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    Some(Database {
        directory: directory.to_owned(),
        name: name.to_owned(),
        variant,
        suffix: suffix.to_owned(),
    })
}

pub fn route(path: &str) -> Route {
    if path.is_empty() || path.ends_with('/') {
        return Route::Directory;
    }
    let file = file_name(path);
    let directory = directory(path);
    let (base, signed) = match file.strip_suffix(SIGNATURE_SUFFIX) {
        Some(base) => (base, true),
        None => (file, false),
    };
    if let Some(package) = PackageFile::parse(base) {
        return match signed {
            true => Route::Signature(package),
            false => Route::Package(package),
        };
    }
    if let Some(database) = database(directory, base) {
        return match signed {
            true => Route::DatabaseSignature(database),
            false => Route::Database(database),
        };
    }
    if !signed
        && directory.is_empty()
        && file.len() > KEY_SUFFIX.len()
        && file.ends_with(KEY_SUFFIX)
    {
        return Route::Key(file.to_owned());
    }
    Route::File
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

pub fn last_segment(directory: &str) -> &str {
    directory.rsplit('/').next().unwrap_or_default()
}

pub fn any_directory(directory: &str) -> String {
    format!("{}/{ANY}", self::directory(directory))
}

pub fn is_any(directory: &str) -> bool {
    last_segment(directory) == ANY
}

pub fn join(base: &str, relative: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    for segment in relative.split('/').filter(|segment| !segment.is_empty()) {
        match segment {
            "." => {}
            ".." => return None,
            segment if segment.chars().any(char::is_control) => return None,
            segment => parts.push(segment),
        }
    }
    Some(match parts.is_empty() {
        true => String::new(),
        false => format!("/{}", parts.join("/")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, version: &str, arch: &str, extension: &str) -> PackageFile {
        PackageFile {
            name: name.to_owned(),
            version: version.to_owned(),
            arch: arch.to_owned(),
            extension: extension.to_owned(),
        }
    }

    #[test]
    fn package_file_names_give_name_version_and_architecture() {
        assert_eq!(
            PackageFile::parse("python-pip-24.2-1-any.pkg.tar.zst"),
            Some(package("python-pip", "24.2-1", "any", ".zst"))
        );
        assert_eq!(
            PackageFile::parse("ffmpeg-2:7.1-4-x86_64.pkg.tar.xz"),
            Some(package("ffmpeg", "2:7.1-4", "x86_64", ".xz"))
        );
        assert_eq!(
            PackageFile::parse("lib32-glibc-2.40+r16-1.1-x86_64.pkg.tar"),
            Some(package("lib32-glibc", "2.40+r16-1.1", "x86_64", ""))
        );
        assert_eq!(PackageFile::parse("hello-1.0-x86_64.pkg.tar.zst"), None);
        assert_eq!(PackageFile::parse("hello-1.0-1-x86_64.pkg.tar.rar"), None);
        assert_eq!(PackageFile::parse("-x-1.0-1-any.pkg.tar.zst"), None);
        assert_eq!(PackageFile::parse("core.db.tar.gz"), None);
        assert_eq!(
            package("hello", "1:1.0-2", "aarch64", ".gz").file_name(),
            "hello-1:1.0-2-aarch64.pkg.tar.gz"
        );
    }

    #[test]
    fn paths_route_to_databases_packages_and_signatures() {
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/x86_64/"), Route::Directory);
        let db = |directory: &str, name: &str, variant, suffix: &str| Database {
            directory: directory.to_owned(),
            name: name.to_owned(),
            variant,
            suffix: suffix.to_owned(),
        };
        assert_eq!(
            route("/core/os/x86_64/core.db"),
            Route::Database(db("/core/os/x86_64", "core", Variant::Db, ""))
        );
        assert_eq!(
            route("/x86_64/my.repo.files.tar.gz"),
            Route::Database(db("/x86_64", "my.repo", Variant::Files, ".tar.gz"))
        );
        assert_eq!(
            route("/x86_64/extra.db.tar.zst.sig"),
            Route::DatabaseSignature(db("/x86_64", "extra", Variant::Db, ".tar.zst"))
        );
        assert_eq!(
            route("/x86_64/extra.files.sig"),
            Route::DatabaseSignature(db("/x86_64", "extra", Variant::Files, ""))
        );
        assert_eq!(
            route("/x86_64/hello-1.0-1-x86_64.pkg.tar.zst"),
            Route::Package(package("hello", "1.0-1", "x86_64", ".zst"))
        );
        assert_eq!(
            route("/x86_64/x.db-1.0-1-any.pkg.tar.zst.sig"),
            Route::Signature(package("x.db", "1.0-1", "any", ".zst"))
        );
        assert_eq!(route("/pkgs.key"), Route::Key("pkgs.key".to_owned()));
        assert_eq!(route("/x86_64/pkgs.key"), Route::File);
        assert_eq!(route("/.key"), Route::File);
        assert_eq!(route("/lastsync"), Route::File);
        assert_eq!(route("/x86_64/.db"), Route::File);
        assert_eq!(
            db("/x86_64", "core", Variant::Db, "").stored_path(),
            "/x86_64/core.db.tar.gz"
        );
        assert!(db("/x86_64", "core", Variant::Db, ".tar.gz").generated());
        assert!(!db("/x86_64", "core", Variant::Db, ".tar.zst").generated());
        assert_eq!(
            db("/x86_64", "core", Variant::Files, ".tar.zst").signature_path(),
            "/x86_64/core.files.tar.zst.sig"
        );
    }

    #[test]
    fn directories_join_and_find_their_any_sibling() {
        assert_eq!(join("", "custom//os/"), Some("/custom/os".to_owned()));
        assert_eq!(join("/a", "./b"), Some("/a/b".to_owned()));
        assert_eq!(join("", ""), Some(String::new()));
        assert_eq!(join("", "../etc"), None);
        assert_eq!(any_directory("/custom/os/x86_64"), "/custom/os/any");
        assert_eq!(any_directory("/x86_64"), "/any");
        assert!(is_any("/custom/any"));
        assert!(!is_any("/x86_64"));
        assert_eq!(last_segment("/core/os/x86_64"), "x86_64");
        assert_eq!(directory("/x86_64/a.db"), "/x86_64");
        assert_eq!(directory("/a.db"), "");
        assert!(valid_arch("x86_64"));
        assert!(!valid_arch("x86-64"));
    }
}
