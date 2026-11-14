use astrum_content::inventory::PackageRef;
use percent_encoding::percent_decode_str;

const MAX_PURL_BYTES: usize = 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Mapped {
    Package(PackageRef),
    Skipped,
    Unlisted,
}

struct Purl {
    kind: String,
    namespace: String,
    name: String,
    version: Option<String>,
    qualifiers: Vec<(String, String)>,
}

impl Purl {
    fn qualifier(&self, key: &str) -> Option<&str> {
        self.qualifiers
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }
}

fn decode(text: &str) -> Option<String> {
    percent_decode_str(text)
        .decode_utf8()
        .ok()
        .map(|decoded| decoded.into_owned())
}

fn parse(text: &str) -> Option<Purl> {
    if text.len() > MAX_PURL_BYTES || !text.get(..4)?.eq_ignore_ascii_case("pkg:") {
        return None;
    }
    let rest = text[4..].trim_start_matches('/');
    let rest = rest.split('#').next().unwrap_or_default();
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (path, version) = match path.rsplit_once('@') {
        Some((before, version)) if !before.is_empty() && !before.ends_with('/') => (
            before,
            Some(decode(version)?).filter(|version| !version.is_empty()),
        ),
        _ => (path, None),
    };
    let mut segments = path.trim_end_matches('/').split('/');
    let kind = segments.next()?.to_ascii_lowercase();
    let mut parts = Vec::new();
    for segment in segments {
        parts.push(decode(segment)?);
    }
    let name = parts.pop().filter(|name| !name.is_empty())?;
    if kind.is_empty() || parts.iter().any(String::is_empty) {
        return None;
    }
    let mut qualifiers = Vec::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=')?;
        qualifiers.push((key.to_ascii_lowercase(), decode(value)?));
    }
    Some(Purl {
        kind,
        namespace: parts.join("/"),
        name,
        version,
        qualifiers,
    })
}

pub(crate) fn numeric(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn distro(purl: &Purl) -> Option<(String, String)> {
    let value = purl.qualifier("distro")?;
    match value.split_once('-') {
        Some((id, release)) if !id.is_empty() => {
            Some((id.to_ascii_lowercase(), release.to_owned()))
        }
        _ if !purl.namespace.is_empty() => {
            Some((purl.namespace.to_ascii_lowercase(), value.to_owned()))
        }
        _ => None,
    }
}

fn debian_family(id: &str, release: &str) -> Option<String> {
    let mut parts = release.split('.');
    let first = parts.next()?;
    match id {
        "debian" if numeric(first) => Some(format!("Debian:{first}")),
        "ubuntu" => {
            let month = parts.next()?;
            let lts = numeric(first) && month == "04" && first.parse::<u32>().ok()? % 2 == 0;
            lts.then(|| format!("Ubuntu:{first}.{month}:LTS"))
        }
        _ => None,
    }
}

fn alpine(id: &str, release: &str) -> Option<String> {
    let mut parts = release.split('.');
    let (major, minor) = (parts.next()?, parts.next()?);
    (id == "alpine" && numeric(major) && numeric(minor)).then(|| format!("Alpine:v{major}.{minor}"))
}

pub(crate) fn os_ecosystem(id: &str, release: &str) -> Option<String> {
    debian_family(id, release).or_else(|| alpine(id, release))
}

fn joined(purl: &Purl) -> String {
    match purl.namespace.is_empty() {
        true => purl.name.clone(),
        false => format!("{}/{}", purl.namespace, purl.name),
    }
}

fn source_of(purl: &Purl, version: String) -> (String, String) {
    let Some(upstream) = purl
        .qualifier("upstream")
        .filter(|upstream| !upstream.is_empty())
    else {
        return (purl.name.clone(), version);
    };
    match upstream.split_once('@') {
        Some((name, source)) if !name.is_empty() && !source.is_empty() => {
            (name.to_owned(), source.to_owned())
        }
        Some(_) => (purl.name.clone(), version),
        None => (upstream.to_owned(), version),
    }
}

fn with_epoch(purl: &Purl, version: String) -> String {
    match purl.qualifier("epoch") {
        Some(epoch) if numeric(epoch) && epoch != "0" && !version.contains(':') => {
            format!("{epoch}:{version}")
        }
        _ => version,
    }
}

pub fn map(purl: Option<&str>, fallback_version: &str) -> Mapped {
    let Some(text) = purl.filter(|text| !text.is_empty()) else {
        return Mapped::Unlisted;
    };
    let Some(parsed) = parse(text) else {
        return Mapped::Skipped;
    };
    let version = match parsed.version.clone() {
        Some(version) => version,
        None => fallback_version.to_owned(),
    };
    if version.is_empty() {
        return Mapped::Skipped;
    }
    let needs_namespace = matches!(parsed.kind.as_str(), "maven" | "composer");
    if needs_namespace && parsed.namespace.is_empty() {
        return Mapped::Skipped;
    }
    let (ecosystem, name, version) = match parsed.kind.as_str() {
        "npm" => ("npm".to_owned(), joined(&parsed), version),
        "maven" => (
            "Maven".to_owned(),
            format!("{}:{}", parsed.namespace, parsed.name),
            version,
        ),
        "pypi" => ("PyPI".to_owned(), parsed.name.clone(), version),
        "golang" => ("Go".to_owned(), joined(&parsed), version),
        "cargo" => ("crates.io".to_owned(), parsed.name.clone(), version),
        "nuget" => ("NuGet".to_owned(), parsed.name.clone(), version),
        "gem" => ("RubyGems".to_owned(), parsed.name.clone(), version),
        "composer" => ("Packagist".to_owned(), joined(&parsed), version),
        "pub" => ("Pub".to_owned(), parsed.name.clone(), version),
        "hex" => ("Hex".to_owned(), joined(&parsed), version),
        "deb" => {
            let Some(ecosystem) =
                distro(&parsed).and_then(|(id, release)| debian_family(&id, &release))
            else {
                return Mapped::Skipped;
            };
            let (name, version) = source_of(&parsed, with_epoch(&parsed, version));
            (ecosystem, name, version)
        }
        "apk" => {
            let Some(ecosystem) = distro(&parsed).and_then(|(id, release)| alpine(&id, &release))
            else {
                return Mapped::Skipped;
            };
            let (name, version) = source_of(&parsed, version);
            (ecosystem, name, version)
        }
        _ => return Mapped::Skipped,
    };
    let name = astrum_scan::osv::normalise_name(&ecosystem, &name);
    let unsafe_text = |text: &str| text.chars().any(char::is_control);
    if name.is_empty() || unsafe_text(&name) || unsafe_text(&version) || unsafe_text(text) {
        return Mapped::Skipped;
    }
    Mapped::Package(PackageRef {
        ecosystem,
        name,
        version,
        purl: text.to_owned(),
    })
}

#[cfg(test)]
mod tests;
