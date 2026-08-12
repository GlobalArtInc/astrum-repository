#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub name: String,
    pub version: String,
    pub user: String,
    pub channel: String,
}

impl Recipe {
    pub fn base(&self) -> String {
        format!(
            "/{}/{}/{}/{}",
            self.name, self.version, self.user, self.channel
        )
    }

    pub fn display(&self) -> String {
        format!(
            "{}/{}@{}/{}",
            self.name, self.version, self.user, self.channel
        )
    }

    pub fn namespace(&self) -> String {
        if self.user == "_" && self.channel == "_" {
            String::new()
        } else {
            format!("{}/{}", self.user, self.channel)
        }
    }

    pub fn api(&self) -> String {
        format!("/v2/conans{}", self.base())
    }

    pub fn export_path(&self, revision: &str, file: &str) -> String {
        format!("{}/{revision}/export/{file}", self.base())
    }

    pub fn package_path(
        &self,
        revision: &str,
        package: &str,
        package_revision: &str,
        file: &str,
    ) -> String {
        format!(
            "{}/{revision}/package/{package}/{package_revision}/{file}",
            self.base()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Ping,
    Authenticate,
    CheckCredentials,
    Search,
    Revisions(Recipe),
    Latest(Recipe),
    RecipeRevision(Recipe, String),
    RecipeFiles(Recipe, String),
    RecipeFile(Recipe, String, String),
    PackageSearch(Recipe, Option<String>),
    Packages(Recipe, String),
    PackageRevisions(Recipe, String, String),
    PackageLatest(Recipe, String, String),
    PackageRevision(Recipe, String, String, String),
    PackageFiles(Recipe, String, String, String),
    PackageFile(Recipe, String, String, String, String),
    Directory,
    Other,
}

fn name_part(part: &str) -> bool {
    !part.is_empty()
        && part.len() <= 101
        && part != "."
        && part != ".."
        && part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'))
}

fn revision(part: &str) -> bool {
    !part.is_empty() && part.len() <= 64 && part.chars().all(|c| c.is_ascii_alphanumeric())
}

fn file_name(part: &str) -> bool {
    name_part(part) && !part.starts_with('.')
}

pub fn route(path: &str) -> Route {
    if path.ends_with('/') {
        return Route::Directory;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        ["v1" | "v2", "ping"] => return Route::Ping,
        ["v1" | "v2", "users", "authenticate"] => return Route::Authenticate,
        ["v1" | "v2", "users", "check_credentials"] => return Route::CheckCredentials,
        ["v2", "conans", "search"] => return Route::Search,
        _ => {}
    }
    let ["v2", "conans", name, version, user, channel, rest @ ..] = parts.as_slice() else {
        return Route::Other;
    };
    if ![name, version, user, channel]
        .iter()
        .all(|part| name_part(part))
    {
        return Route::Other;
    }
    let recipe = Recipe {
        name: (*name).to_owned(),
        version: (*version).to_owned(),
        user: (*user).to_owned(),
        channel: (*channel).to_owned(),
    };
    let owned = |text: &&str| (*text).to_owned();
    match rest {
        ["revisions"] => Route::Revisions(recipe),
        ["latest"] => Route::Latest(recipe),
        ["search"] => Route::PackageSearch(recipe, None),
        ["revisions", rrev] if revision(rrev) => Route::RecipeRevision(recipe, owned(rrev)),
        ["revisions", rrev, "files"] if revision(rrev) => Route::RecipeFiles(recipe, owned(rrev)),
        ["revisions", rrev, "files", file] if revision(rrev) && file_name(file) => {
            Route::RecipeFile(recipe, owned(rrev), owned(file))
        }
        ["revisions", rrev, "search"] if revision(rrev) => {
            Route::PackageSearch(recipe, Some(owned(rrev)))
        }
        ["revisions", rrev, "packages"] if revision(rrev) => Route::Packages(recipe, owned(rrev)),
        ["revisions", rrev, "packages", package, "revisions"]
            if revision(rrev) && revision(package) =>
        {
            Route::PackageRevisions(recipe, owned(rrev), owned(package))
        }
        ["revisions", rrev, "packages", package, "latest"]
            if revision(rrev) && revision(package) =>
        {
            Route::PackageLatest(recipe, owned(rrev), owned(package))
        }
        ["revisions", rrev, "packages", package, "revisions", prev]
            if revision(rrev) && revision(package) && revision(prev) =>
        {
            Route::PackageRevision(recipe, owned(rrev), owned(package), owned(prev))
        }
        [
            "revisions",
            rrev,
            "packages",
            package,
            "revisions",
            prev,
            "files",
        ] if revision(rrev) && revision(package) && revision(prev) => {
            Route::PackageFiles(recipe, owned(rrev), owned(package), owned(prev))
        }
        [
            "revisions",
            rrev,
            "packages",
            package,
            "revisions",
            prev,
            "files",
            file,
        ] if revision(rrev) && revision(package) && revision(prev) && file_name(file) => {
            Route::PackageFile(
                recipe,
                owned(rrev),
                owned(package),
                owned(prev),
                owned(file),
            )
        }
        _ => Route::Other,
    }
}

pub struct Stored {
    pub recipe: Recipe,
    pub revision: String,
    pub package: Option<(String, String)>,
    pub file: String,
}

pub fn parse_stored(path: &str) -> Option<Stored> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (recipe, revision, package, file) = match parts.as_slice() {
        [name, version, user, channel, rrev, "export", file] => {
            ((name, version, user, channel), rrev, None, file)
        }
        [
            name,
            version,
            user,
            channel,
            rrev,
            "package",
            package,
            prev,
            file,
        ] => (
            (name, version, user, channel),
            rrev,
            Some(((*package).to_owned(), (*prev).to_owned())),
            file,
        ),
        _ => return None,
    };
    Some(Stored {
        recipe: Recipe {
            name: (*recipe.0).to_owned(),
            version: (*recipe.1).to_owned(),
            user: (*recipe.2).to_owned(),
            channel: (*recipe.3).to_owned(),
        },
        revision: (*revision).to_owned(),
        package,
        file: (*file).to_owned(),
    })
}

pub fn glob(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.to_ascii_lowercase().chars().collect();
    let text: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let (mut p, mut t) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == '?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            mark = t;
            p += 1;
        } else if let Some(at) = star {
            p = at + 1;
            mark += 1;
            t = mark;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == '*' {
        p += 1;
    }
    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_paths_route_to_recipes_and_packages() {
        assert_eq!(route("/v1/ping"), Route::Ping);
        assert_eq!(route("/v2/users/authenticate"), Route::Authenticate);
        assert_eq!(route("/v2/conans/search"), Route::Search);
        let recipe = Recipe {
            name: "zlib".to_owned(),
            version: "1.3.1".to_owned(),
            user: "_".to_owned(),
            channel: "_".to_owned(),
        };
        assert_eq!(
            route("/v2/conans/zlib/1.3.1/_/_/latest"),
            Route::Latest(recipe.clone())
        );
        assert_eq!(
            route("/v2/conans/zlib/1.3.1/_/_/revisions/abc/files/conanfile.py"),
            Route::RecipeFile(recipe.clone(), "abc".to_owned(), "conanfile.py".to_owned())
        );
        assert_eq!(
            route(
                "/v2/conans/zlib/1.3.1/_/_/revisions/abc/packages/def/revisions/123/files/conan_package.tgz"
            ),
            Route::PackageFile(
                recipe.clone(),
                "abc".to_owned(),
                "def".to_owned(),
                "123".to_owned(),
                "conan_package.tgz".to_owned()
            )
        );
        assert_eq!(
            route("/v2/conans/zlib/1.3.1/_/_/revisions/a.b/files"),
            Route::Other
        );
        assert_eq!(
            recipe.export_path("abc", "conanfile.py"),
            "/zlib/1.3.1/_/_/abc/export/conanfile.py"
        );
        assert_eq!(recipe.display(), "zlib/1.3.1@_/_");
        assert_eq!(recipe.namespace(), "");
        let stored = parse_stored("/zlib/1.3.1/_/_/abc/package/def/123/conaninfo.txt").unwrap();
        assert_eq!(stored.package, Some(("def".to_owned(), "123".to_owned())));
        assert_eq!(stored.file, "conaninfo.txt");
    }

    #[test]
    fn search_patterns_are_globs() {
        assert!(glob("m4c*", "m4chello/1.0.0@_/_"));
        assert!(glob("zlib/*", "zlib/1.3.1@_/_"));
        assert!(glob("*hello*", "m4chello/1.0.0@_/_"));
        assert!(!glob("zlib", "zlib/1.3.1@_/_"));
        assert!(glob("Z?IB*", "zlib/1"));
    }
}
