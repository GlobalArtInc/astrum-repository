use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    List(String),
    Latest(String),
    Info(String, String),
    Mod(String, String),
    Zip(String, String),
}

impl Request {
    pub fn module(&self) -> &str {
        match self {
            Request::List(module)
            | Request::Latest(module)
            | Request::Info(module, _)
            | Request::Mod(module, _)
            | Request::Zip(module, _) => module,
        }
    }

    pub fn version(&self) -> Option<&str> {
        match self {
            Request::Info(_, version) | Request::Mod(_, version) | Request::Zip(_, version) => {
                Some(version)
            }
            _ => None,
        }
    }

    pub fn path(&self) -> String {
        self.remote_path()
    }

    pub fn remote_path(&self) -> String {
        match self {
            Request::List(module) => format!("/{}/@v/list", escape(module)),
            Request::Latest(module) => format!("/{}/@latest", escape(module)),
            Request::Info(module, version) => {
                format!("/{}/@v/{}.info", escape(module), escape(version))
            }
            Request::Mod(module, version) => {
                format!("/{}/@v/{}.mod", escape(module), escape(version))
            }
            Request::Zip(module, version) => {
                format!("/{}/@v/{}.zip", escape(module), escape(version))
            }
        }
    }
}

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_uppercase() {
            out.push('!');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

pub fn unescape(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut bang = false;
    for c in text.chars() {
        if bang {
            if !c.is_ascii_lowercase() {
                return None;
            }
            out.push(c.to_ascii_uppercase());
            bang = false;
        } else if c == '!' {
            bang = true;
        } else if c.is_ascii_uppercase() {
            return None;
        } else {
            out.push(c);
        }
    }
    (!bang).then_some(out)
}

pub fn valid_module(module: &str) -> bool {
    !module.is_empty()
        && module.len() <= 512
        && !module.starts_with('/')
        && !module.ends_with('/')
        && module.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && !segment.starts_with('@')
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~' | '+'))
        })
}

pub fn semver(version: &str) -> Option<semver::Version> {
    let bare = version.strip_prefix('v')?;
    semver::Version::parse(bare).ok()
}

pub fn valid_version(version: &str) -> bool {
    semver(version).is_some()
}

pub fn is_pseudo(version: &str) -> bool {
    let Some(parsed) = semver(version) else {
        return false;
    };
    let pre = parsed.pre.as_str();
    let last = pre.rsplit(['.', '-']).next().unwrap_or("");
    let timestamp = pre
        .split(['.', '-'])
        .any(|part| part.len() == 14 && part.chars().all(|c| c.is_ascii_digit()));
    timestamp && last.len() == 12 && last.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn compare(left: &str, right: &str) -> Ordering {
    match (semver(left), semver(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => left.cmp(right),
    }
}

pub fn latest<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let all: Vec<&str> = versions.into_iter().collect();
    let releases = all
        .iter()
        .copied()
        .filter(|version| semver(version).is_some_and(|parsed| parsed.pre.is_empty()))
        .max_by(|left, right| compare(left, right));
    releases.or_else(|| {
        all.iter()
            .copied()
            .filter(|version| !is_pseudo(version))
            .max_by(|left, right| compare(left, right))
            .or_else(|| {
                all.iter()
                    .copied()
                    .max_by(|left, right| compare(left, right))
            })
    })
}

pub fn parse(path: &str) -> Option<Request> {
    let trimmed = path.strip_prefix('/')?;
    if let Some(module) = trimmed.strip_suffix("/@latest") {
        let module = unescape(module)?;
        return valid_module(&module).then_some(Request::Latest(module));
    }
    let (module, file) = trimmed.split_once("/@v/")?;
    let module = unescape(module)?;
    if !valid_module(&module) || file.contains('/') {
        return None;
    }
    if file == "list" {
        return Some(Request::List(module));
    }
    let (version, extension) = file.rsplit_once('.')?;
    let version = unescape(version)?;
    if !valid_version(&version) {
        return None;
    }
    match extension {
        "info" => Some(Request::Info(module, version)),
        "mod" => Some(Request::Mod(module, version)),
        "zip" => Some(Request::Zip(module, version)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_paths_are_case_encoded() {
        assert_eq!(
            escape("github.com/Azure/go-AMQP"),
            "github.com/!azure/go-!a!m!q!p"
        );
        assert_eq!(
            unescape("github.com/!azure/go-!a!m!q!p").as_deref(),
            Some("github.com/Azure/go-AMQP")
        );
        assert_eq!(unescape("github.com/Azure"), None);
        assert_eq!(unescape("github.com/!"), None);
        assert_eq!(unescape("github.com/!1"), None);
    }

    #[test]
    fn requests_are_parsed_from_paths() {
        assert_eq!(
            parse("/github.com/!azure/sdk/@v/list"),
            Some(Request::List("github.com/Azure/sdk".to_owned()))
        );
        assert_eq!(
            parse("/example.com/m/@latest"),
            Some(Request::Latest("example.com/m".to_owned()))
        );
        assert_eq!(
            parse("/example.com/m/@v/v1.2.3.info"),
            Some(Request::Info(
                "example.com/m".to_owned(),
                "v1.2.3".to_owned()
            ))
        );
        assert_eq!(
            parse("/example.com/m/v2/@v/v2.0.0+incompatible.zip"),
            Some(Request::Zip(
                "example.com/m/v2".to_owned(),
                "v2.0.0+incompatible".to_owned()
            ))
        );
        assert_eq!(
            parse("/example.com/m/@v/v1.0.0-!r!c1.mod"),
            Some(Request::Mod(
                "example.com/m".to_owned(),
                "v1.0.0-RC1".to_owned()
            ))
        );
        for bad in [
            "/example.com/m/@v/1.0.0.zip",
            "/example.com/m/@v/v1.0.0.tar",
            "/example.com/../m/@v/list",
            "/@v/list",
            "/example.com/M/@v/list",
            "/example.com/m/@v/x/v1.0.0.zip",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
        let request = Request::Zip("github.com/Azure/sdk".to_owned(), "v1.0.0-RC1".to_owned());
        assert_eq!(
            request.remote_path(),
            "/github.com/!azure/sdk/@v/v1.0.0-!r!c1.zip"
        );
        assert_eq!(request.path(), request.remote_path());
    }

    #[test]
    fn the_latest_version_prefers_releases() {
        assert_eq!(
            latest(["v1.0.0", "v1.10.0", "v1.2.0", "v2.0.0-beta.1"]),
            Some("v1.10.0")
        );
        assert_eq!(
            latest(["v2.0.0-beta.1", "v0.0.0-20240101120000-abcdefabcdef"]),
            Some("v2.0.0-beta.1")
        );
        assert!(is_pseudo("v0.0.0-20240101120000-abcdefabcdef"));
        assert!(is_pseudo("v1.2.4-0.20240101120000-abcdefabcdef"));
        assert!(!is_pseudo("v1.2.4-rc.1"));
        assert_eq!(latest(Vec::<&str>::new()), None);
    }
}
