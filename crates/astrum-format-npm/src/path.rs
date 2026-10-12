use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name {
    pub scope: Option<String>,
    pub bare: String,
}

impl Name {
    pub fn parse(full: &str) -> Option<Name> {
        match full.split_once('/') {
            Some((scope, bare)) => Name::from_segments(&[scope, bare]),
            None => Name::from_segments(&[full]),
        }
    }

    fn from_segments(segments: &[&str]) -> Option<Name> {
        let (scope, bare) = match segments {
            [scope, bare] if scope.starts_with('@') => {
                let scope = &scope[1..];
                if scope.is_empty() || !valid_part(scope) {
                    return None;
                }
                (Some(scope.to_owned()), *bare)
            }
            [bare] if !bare.starts_with('@') => (None, *bare),
            _ => return None,
        };
        if !valid_part(bare) || bare == "-" || bare.starts_with('.') {
            return None;
        }
        Some(Name {
            scope,
            bare: bare.to_owned(),
        })
    }

    pub fn packument_path(&self) -> String {
        format!("/{self}")
    }

    pub fn tarball_path(&self, file: &str) -> String {
        format!("/{self}/-/{file}")
    }

    pub fn tarball_prefix(&self) -> String {
        format!("/{self}/-/")
    }

    pub fn tarball_file(&self, version: &str) -> String {
        format!("{}-{version}.tgz", self.bare)
    }

    pub fn version_of_file(&self, file: &str) -> Option<String> {
        let version = file
            .strip_prefix(&self.bare)?
            .strip_prefix('-')?
            .strip_suffix(".tgz")?;
        (!version.is_empty()).then(|| version.to_owned())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scope {
            Some(scope) => write!(f, "@{scope}/{}", self.bare),
            None => f.write_str(&self.bare),
        }
    }
}

fn valid_part(part: &str) -> bool {
    !part.is_empty()
        && part.len() <= 214
        && part
            .chars()
            .all(|c| !c.is_control() && !c.is_whitespace() && !matches!(c, '/' | '\\'))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Ping,
    Whoami,
    Search,
    WebLogin,
    User,
    Audit,
    DistTags { name: Name, tag: Option<String> },
    Packument { name: Name },
    Version { name: Name, version: String },
    Tarball { name: Name, file: String },
    Unknown,
}

pub fn route(path: &str) -> Route {
    let segments: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    match segments.as_slice() {
        ["-", "ping"] => Route::Ping,
        ["-", "whoami"] => Route::Whoami,
        ["-", "v1", "search"] => Route::Search,
        ["-", "v1", "login", ..] => Route::WebLogin,
        ["-", "user", ..] => Route::User,
        ["-", "npm", "v1", "security", ..] => Route::Audit,
        ["-", "package", rest @ ..] => dist_tags(rest),
        ["-", ..] | [] => Route::Unknown,
        _ => package(&segments),
    }
}

fn split_name<'a>(segments: &'a [&'a str]) -> Option<(Name, &'a [&'a str])> {
    match segments {
        [scope, bare, rest @ ..] if scope.starts_with('@') => {
            Some((Name::from_segments(&[scope, bare])?, rest))
        }
        [bare, rest @ ..] => Some((Name::from_segments(&[bare])?, rest)),
        [] => None,
    }
}

fn dist_tags(rest: &[&str]) -> Route {
    let Some((name, rest)) = split_name(rest) else {
        return Route::Unknown;
    };
    match rest {
        ["dist-tags"] => Route::DistTags { name, tag: None },
        ["dist-tags", tag] => Route::DistTags {
            name,
            tag: Some((*tag).to_owned()),
        },
        _ => Route::Unknown,
    }
}

fn package(segments: &[&str]) -> Route {
    if let Some(marker) = segments.iter().position(|part| *part == "-") {
        let (before, after) = (&segments[..marker], &segments[marker + 1..]);
        let file = match after {
            [file] | [file, "-rev", _] => *file,
            _ => return Route::Unknown,
        };
        let (name, prefix) = match before {
            [prefix @ .., scope, bare] if scope.starts_with('@') => {
                (Name::from_segments(&[scope, bare]), prefix)
            }
            [prefix @ .., bare] => (Name::from_segments(&[bare]), prefix),
            [] => return Route::Unknown,
        };
        let prefix_allowed =
            prefix.is_empty() || (prefix.len() >= 2 && prefix[prefix.len() - 2] == "repository");
        return match name {
            Some(name) if prefix_allowed => Route::Tarball {
                name,
                file: file.to_owned(),
            },
            _ => Route::Unknown,
        };
    }
    let Some((name, rest)) = split_name(segments) else {
        return Route::Unknown;
    };
    match rest {
        [] | ["-rev", _] => Route::Packument { name },
        [version] => Route::Version {
            name,
            version: (*version).to_owned(),
        },
        _ => Route::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(full: &str) -> Name {
        Name::parse(full).unwrap()
    }

    #[test]
    fn names_carry_their_scope() {
        let scoped = name("@team/tool");
        assert_eq!(scoped.scope.as_deref(), Some("team"));
        assert_eq!(scoped.bare, "tool");
        assert_eq!(scoped.to_string(), "@team/tool");
        assert_eq!(scoped.packument_path(), "/@team/tool");
        assert_eq!(
            scoped.tarball_path("tool-1.0.0.tgz"),
            "/@team/tool/-/tool-1.0.0.tgz"
        );
        assert_eq!(
            scoped.version_of_file("tool-1.0.0-beta.1.tgz").as_deref(),
            Some("1.0.0-beta.1")
        );
        assert_eq!(scoped.version_of_file("other-1.0.0.tgz"), None);
        assert_eq!(name("lodash").to_string(), "lodash");
        for bad in ["@/x", "@team", "", ".hidden", "-", "a b", "@team/"] {
            assert_eq!(Name::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn paths_become_routes() {
        assert_eq!(route("/-/ping"), Route::Ping);
        assert_eq!(route("/-/whoami"), Route::Whoami);
        assert_eq!(route("/-/v1/search"), Route::Search);
        assert_eq!(route("/-/user/org.couchdb.user:bob"), Route::User);
        assert_eq!(route("/-/all"), Route::Unknown);
        assert_eq!(
            route("/-/package/@team/tool/dist-tags/beta"),
            Route::DistTags {
                name: name("@team/tool"),
                tag: Some("beta".to_owned())
            }
        );
        assert_eq!(
            route("/-/package/tool/dist-tags"),
            Route::DistTags {
                name: name("tool"),
                tag: None
            }
        );
        assert_eq!(
            route("/@team/tool"),
            Route::Packument {
                name: name("@team/tool")
            }
        );
        assert_eq!(
            route("/tool/-rev/3"),
            Route::Packument { name: name("tool") }
        );
        assert_eq!(
            route("/@team/tool/1.2.3"),
            Route::Version {
                name: name("@team/tool"),
                version: "1.2.3".to_owned()
            }
        );
        assert_eq!(
            route("/tool/-/tool-1.2.3.tgz"),
            Route::Tarball {
                name: name("tool"),
                file: "tool-1.2.3.tgz".to_owned()
            }
        );
        assert_eq!(
            route("/repository/npm-hosted/@team/tool/-/tool-1.2.3.tgz/-rev/4"),
            Route::Tarball {
                name: name("@team/tool"),
                file: "tool-1.2.3.tgz".to_owned()
            }
        );
        assert_eq!(route("/a/b/-/tool-1.2.3.tgz"), Route::Unknown);
        assert_eq!(route("/tool/1.0.0/extra"), Route::Unknown);
        assert_eq!(route("/@team"), Route::Unknown);
    }
}
