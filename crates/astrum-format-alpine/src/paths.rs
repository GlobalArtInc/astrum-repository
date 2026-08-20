use crate::index;
use crate::signing::KEY_SUFFIX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Directory,
    Index(String),
    Key(String),
    Package,
    File,
}

pub fn route(path: &str) -> Route {
    if path.is_empty() || path.ends_with('/') {
        return Route::Directory;
    }
    let file = file_name(path);
    let directory = directory(path);
    if file == index::FILE {
        Route::Index(directory.to_owned())
    } else if file.ends_with(KEY_SUFFIX) && directory.is_empty() {
        Route::Key(file.to_owned())
    } else if is_package(path) {
        Route::Package
    } else {
        Route::File
    }
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

pub fn is_package(path: &str) -> bool {
    path.ends_with(".apk")
}

pub fn index_path(directory: &str) -> String {
    format!("{directory}/{}", index::FILE)
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

pub fn branch(directory: &str) -> &str {
    self::directory(directory).trim_start_matches('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_indexes_keys_and_packages() {
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/v3.20/main/"), Route::Directory);
        assert_eq!(
            route("/v3.20/main/x86_64/APKINDEX.tar.gz"),
            Route::Index("/v3.20/main/x86_64".to_owned())
        );
        assert_eq!(route("/APKINDEX.tar.gz"), Route::Index(String::new()));
        assert_eq!(
            route("/apks.rsa.pub"),
            Route::Key("apks.rsa.pub".to_owned())
        );
        assert_eq!(route("/keys/apks.rsa.pub"), Route::File);
        assert_eq!(route("/x86_64/hello-1.0-r0.apk"), Route::Package);
        assert_eq!(route("/x86_64/README"), Route::File);
    }

    #[test]
    fn directories_join_without_escaping() {
        assert_eq!(
            join("", "v3.20//main/x86_64/"),
            Some("/v3.20/main/x86_64".to_owned())
        );
        assert_eq!(join("/a", "./b"), Some("/a/b".to_owned()));
        assert_eq!(join("", ""), Some(String::new()));
        assert_eq!(join("", "../etc"), None);
        assert_eq!(last_segment("/v3.20/main/x86_64"), "x86_64");
        assert_eq!(last_segment(""), "");
        assert_eq!(branch("/v3.20/main/x86_64"), "v3.20/main");
        assert_eq!(branch("/x86_64"), "");
        assert_eq!(index_path(""), "/APKINDEX.tar.gz");
    }
}
