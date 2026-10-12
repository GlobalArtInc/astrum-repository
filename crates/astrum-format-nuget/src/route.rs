#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Feed {
    Packages,
    FindPackagesById,
    Search,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    ServiceIndex,
    Versions {
        id: String,
    },
    Content {
        id: String,
        version: String,
        file: String,
    },
    RegistrationIndex {
        id: String,
    },
    RegistrationPage {
        id: String,
        lower: String,
        upper: String,
    },
    RegistrationLeaf {
        id: String,
        version: String,
    },
    CatalogLeaf {
        id: String,
        version: String,
    },
    Query,
    Autocomplete,
    V2Root,
    V2Metadata,
    V2Feed {
        feed: Feed,
        count: bool,
    },
    V2Entry {
        id: String,
        version: String,
    },
    V2Download {
        id: String,
        version: String,
    },
    Publish,
    Package {
        id: String,
        version: String,
    },
    Asset {
        id: String,
        version: String,
        file: String,
    },
    Unknown,
}

fn segments(path: &str) -> Vec<&str> {
    path.trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn feed(name: &str) -> Option<Feed> {
    let bare = name.strip_suffix("()").unwrap_or(name);
    match bare.to_ascii_lowercase().as_str() {
        "packages" => Some(Feed::Packages),
        "findpackagesbyid" => Some(Feed::FindPackagesById),
        "search" => Some(Feed::Search),
        _ => None,
    }
}

fn literal(text: &str) -> Option<String> {
    let inner = text.trim().strip_prefix('\'')?.strip_suffix('\'')?;
    Some(inner.replace("''", "'"))
}

fn entry_key(segment: &str) -> Option<(String, String)> {
    let inner = segment
        .strip_prefix("Packages(")
        .or_else(|| segment.strip_prefix("packages("))?
        .strip_suffix(')')?;
    let mut id = None;
    let mut version = None;
    for pair in inner.split(',') {
        let (key, value) = pair.split_once('=')?;
        match key.trim().to_ascii_lowercase().as_str() {
            "id" => id = literal(value),
            "version" => version = literal(value),
            _ => return None,
        }
    }
    Some((id?, version?))
}

fn json_name(file: &str) -> Option<&str> {
    file.strip_suffix(".json")
}

pub fn route(path: &str) -> Route {
    let parts = segments(path);
    match parts.as_slice() {
        [] => Route::V2Root,
        ["index.json"] => Route::ServiceIndex,
        ["v3", "content", id, "index.json"] => Route::Versions {
            id: (*id).to_owned(),
        },
        ["v3", "content", id, version, file] => Route::Content {
            id: (*id).to_owned(),
            version: (*version).to_owned(),
            file: (*file).to_owned(),
        },
        ["v3", "registration", id, "index.json"] => Route::RegistrationIndex {
            id: (*id).to_owned(),
        },
        ["v3", "registration", id, "page", lower, upper] => match json_name(upper) {
            Some(upper) => Route::RegistrationPage {
                id: (*id).to_owned(),
                lower: (*lower).to_owned(),
                upper: upper.to_owned(),
            },
            None => Route::Unknown,
        },
        ["v3", "registration", id, leaf] => match json_name(leaf) {
            Some(version) => Route::RegistrationLeaf {
                id: (*id).to_owned(),
                version: version.to_owned(),
            },
            None => Route::Unknown,
        },
        ["v3", "catalog", id, leaf] => match json_name(leaf) {
            Some(version) => Route::CatalogLeaf {
                id: (*id).to_owned(),
                version: version.to_owned(),
            },
            None => Route::Unknown,
        },
        ["v3", "query"] => Route::Query,
        ["v3", "autocomplete"] => Route::Autocomplete,
        ["$metadata"] => Route::V2Metadata,
        ["api", "v2", "package"] => Route::Publish,
        ["package", id, version] => Route::V2Download {
            id: (*id).to_owned(),
            version: (*version).to_owned(),
        },
        ["api", "v2", "package", id, version] => Route::Package {
            id: (*id).to_owned(),
            version: (*version).to_owned(),
        },
        [name] => match (feed(name), entry_key(name)) {
            (Some(feed), _) => Route::V2Feed { feed, count: false },
            (None, Some((id, version))) => Route::V2Entry { id, version },
            _ => Route::Unknown,
        },
        [name, "$count"] => match feed(name) {
            Some(feed) => Route::V2Feed { feed, count: true },
            None => Route::Unknown,
        },
        [id, version] => Route::Package {
            id: (*id).to_owned(),
            version: (*version).to_owned(),
        },
        [id, version, file] => Route::Asset {
            id: (*id).to_owned(),
            version: (*version).to_owned(),
            file: (*file).to_owned(),
        },
        _ => Route::Unknown,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Package,
    Symbols,
    Nuspec,
}

pub fn file_kind(id: &str, version: &str, file: &str) -> Option<FileKind> {
    let id = id.to_lowercase();
    let version = version.to_lowercase();
    let file = file.to_lowercase();
    if file == format!("{id}.{version}.nupkg") {
        Some(FileKind::Package)
    } else if file == format!("{id}.{version}.snupkg") {
        Some(FileKind::Symbols)
    } else if file == format!("{id}.nuspec") {
        Some(FileKind::Nuspec)
    } else {
        None
    }
}

pub fn package_path(id: &str, version: &str) -> String {
    let id = id.to_lowercase();
    let version = version.to_lowercase();
    format!("/{id}/{version}/{id}.{version}.nupkg")
}

pub fn symbols_path(id: &str, version: &str) -> String {
    let id = id.to_lowercase();
    let version = version.to_lowercase();
    format!("/{id}/{version}/{id}.{version}.snupkg")
}

pub fn query_pairs(query: Option<&str>) -> Vec<(String, String)> {
    url::form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

pub fn param(pairs: &[(String, String)], name: &str) -> Option<String> {
    pairs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.clone())
}

pub fn odata_string(pairs: &[(String, String)], name: &str) -> Option<String> {
    param(pairs, name).map(|value| literal(&value).unwrap_or(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v3_paths_are_recognised() {
        assert_eq!(route("/index.json"), Route::ServiceIndex);
        assert_eq!(
            route("/v3/content/Astrum.Probe/index.json"),
            Route::Versions {
                id: "Astrum.Probe".to_owned()
            }
        );
        assert_eq!(
            route("/v3/content/a/1.0.0/a.1.0.0.nupkg"),
            Route::Content {
                id: "a".to_owned(),
                version: "1.0.0".to_owned(),
                file: "a.1.0.0.nupkg".to_owned()
            }
        );
        assert_eq!(
            route("/v3/registration/a/index.json"),
            Route::RegistrationIndex { id: "a".to_owned() }
        );
        assert_eq!(
            route("/v3/registration/a/page/1.0.0/2.0.0.json"),
            Route::RegistrationPage {
                id: "a".to_owned(),
                lower: "1.0.0".to_owned(),
                upper: "2.0.0".to_owned()
            }
        );
        assert_eq!(
            route("/v3/registration/a/1.0.0.json"),
            Route::RegistrationLeaf {
                id: "a".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(
            route("/v3/catalog/a/1.0.0.json"),
            Route::CatalogLeaf {
                id: "a".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(route("/v3/query"), Route::Query);
        assert_eq!(route("/v3/autocomplete"), Route::Autocomplete);
        assert_ne!(route("/v3/index.json"), Route::ServiceIndex);
    }

    #[test]
    fn v2_paths_are_recognised() {
        assert_eq!(route("/"), Route::V2Root);
        assert_eq!(route("/$metadata"), Route::V2Metadata);
        assert_eq!(
            route("/Packages()"),
            Route::V2Feed {
                feed: Feed::Packages,
                count: false
            }
        );
        assert_eq!(
            route("/FindPackagesById()/$count"),
            Route::V2Feed {
                feed: Feed::FindPackagesById,
                count: true
            }
        );
        assert_eq!(
            route("/Search"),
            Route::V2Feed {
                feed: Feed::Search,
                count: false
            }
        );
        assert_eq!(
            route("/Packages(Id='It''s',Version='1.0')"),
            Route::V2Entry {
                id: "It's".to_owned(),
                version: "1.0".to_owned()
            }
        );
        assert_eq!(
            route("/package/A/1.0.0"),
            Route::V2Download {
                id: "A".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(route("/api/v2/package"), Route::Publish);
        assert_eq!(
            route("/api/v2/package/A/1.0.0"),
            Route::Package {
                id: "A".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(
            route("/A/1.0.0"),
            Route::Package {
                id: "A".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(
            route("/a/1.0.0/a.1.0.0.nupkg"),
            Route::Asset {
                id: "a".to_owned(),
                version: "1.0.0".to_owned(),
                file: "a.1.0.0.nupkg".to_owned()
            }
        );
        assert_eq!(route("/nothing"), Route::Unknown);
    }

    #[test]
    fn files_and_paths_follow_the_flat_container() {
        assert_eq!(
            file_kind("A.B", "1.0.0-RC", "a.b.1.0.0-rc.nupkg"),
            Some(FileKind::Package)
        );
        assert_eq!(
            file_kind("a", "1.0.0", "a.1.0.0.snupkg"),
            Some(FileKind::Symbols)
        );
        assert_eq!(file_kind("a", "1.0.0", "a.nuspec"), Some(FileKind::Nuspec));
        assert_eq!(file_kind("a", "1.0.0", "b.1.0.0.nupkg"), None);
        assert_eq!(
            package_path("Astrum.Probe", "1.0.0-Beta"),
            "/astrum.probe/1.0.0-beta/astrum.probe.1.0.0-beta.nupkg"
        );
        assert_eq!(symbols_path("A", "1.0.0"), "/a/1.0.0/a.1.0.0.snupkg");
    }

    #[test]
    fn odata_parameters_lose_their_quotes() {
        let pairs = query_pairs(Some(
            "id='Astrum.Probe'&searchTerm=%27a%20b%27&semVerLevel=2.0.0",
        ));
        assert_eq!(odata_string(&pairs, "id").as_deref(), Some("Astrum.Probe"));
        assert_eq!(odata_string(&pairs, "searchterm").as_deref(), Some("a b"));
        assert_eq!(param(&pairs, "semVerLevel").as_deref(), Some("2.0.0"));
        assert_eq!(param(&pairs, "missing"), None);
    }
}
