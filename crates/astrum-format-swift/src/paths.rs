use crate::archive;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub scope: String,
    pub name: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Login,
    Availability,
    Identifiers,
    Releases(Package),
    Release(Package),
    Archive(Package),
    Manifest(Package, String),
    Other,
}

pub fn valid_scope(scope: &str) -> bool {
    let bytes = scope.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 39
        && bytes[0].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'-')
        && !scope.contains("--")
        && !scope.ends_with('-')
}

pub fn valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 100
        && bytes[0].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == b'-' || *c == b'_')
        && bytes
            .windows(2)
            .all(|pair| !(matches!(pair[0], b'-' | b'_') && matches!(pair[1], b'-' | b'_')))
        && !name.ends_with(['-', '_'])
}

pub fn valid_version(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

impl Package {
    fn new(scope: &str, name: &str, version: Option<&str>) -> Option<Self> {
        (valid_scope(scope) && valid_name(name) && version.is_none_or(valid_version)).then(|| {
            Self {
                scope: scope.to_owned(),
                name: name.to_owned(),
                version: version.map(str::to_owned),
            }
        })
    }

    pub fn identifier(&self) -> String {
        format!("{}.{}", self.scope, self.name)
    }

    pub fn at(&self, version: &str) -> Self {
        Self {
            version: Some(version.to_owned()),
            ..self.clone()
        }
    }

    pub fn package_path(&self) -> String {
        format!("/{}/{}", self.scope, self.name)
    }

    pub fn release_path(&self) -> String {
        format!(
            "{}/{}",
            self.package_path(),
            self.version.clone().unwrap_or_default()
        )
    }

    pub fn archive_path(&self) -> String {
        format!("{}.zip", self.release_path())
    }

    pub fn manifest_path(&self, file: &str) -> String {
        format!("{}/{file}", self.release_path())
    }
}

pub fn route(path: &str) -> Route {
    match path {
        "/login" | "/" => return Route::Login,
        "/availability" => return Route::Availability,
        "/identifiers" => return Route::Identifiers,
        _ => {}
    }
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let found = match segments.as_slice() {
        [scope, name] => {
            let name = name.strip_suffix(".json").unwrap_or(name);
            (!name.contains('.'))
                .then(|| Package::new(scope, name, None).map(Route::Releases))
                .flatten()
        }
        [scope, name, version] => {
            if let Some(version) = version.strip_suffix(".zip") {
                Package::new(scope, name, Some(version)).map(Route::Archive)
            } else {
                let version = version.strip_suffix(".json").unwrap_or(version);
                Package::new(scope, name, Some(version)).map(Route::Release)
            }
        }
        [scope, name, version, file] if archive::is_manifest(file) => {
            Package::new(scope, name, Some(version))
                .map(|package| Route::Manifest(package, (*file).to_owned()))
        }
        _ => None,
    };
    found.unwrap_or(Route::Other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_paths_are_routed() {
        let package = Package::new("mona", "LinkedList", None).unwrap();
        assert_eq!(route("/mona/LinkedList"), Route::Releases(package.clone()));
        assert_eq!(
            route("/mona/LinkedList.json"),
            Route::Releases(package.clone())
        );
        assert_eq!(
            route("/mona/LinkedList/1.1.1"),
            Route::Release(package.at("1.1.1"))
        );
        assert_eq!(
            route("/mona/LinkedList/1.1.1.json"),
            Route::Release(package.at("1.1.1"))
        );
        assert_eq!(
            route("/mona/LinkedList/1.1.1.zip"),
            Route::Archive(package.at("1.1.1"))
        );
        assert_eq!(
            route("/mona/LinkedList/1.1.1/Package@swift-5.9.swift"),
            Route::Manifest(package.at("1.1.1"), "Package@swift-5.9.swift".to_owned())
        );
        assert_eq!(route("/identifiers"), Route::Identifiers);
        assert_eq!(route("/login"), Route::Login);
        assert_eq!(route("/-mona/LinkedList"), Route::Other);
        assert_eq!(route("/mona/LinkedList/1.1"), Route::Other);
        assert_eq!(route("/mona/LinkedList/1.1.1/README.md"), Route::Other);
        assert!(valid_name("Linked_List") && !valid_name("Linked__List") && !valid_name("_x"));
    }
}
