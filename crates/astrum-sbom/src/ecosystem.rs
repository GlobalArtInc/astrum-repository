pub(crate) fn of(purl: &str) -> Option<String> {
    let head = purl.get(..4)?;
    if !head.eq_ignore_ascii_case("pkg:") {
        return None;
    }
    let rest = purl[4..].trim_start_matches('/');
    let rest = rest.split('#').next().unwrap_or_default();
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let kind = path.split('/').next()?.to_ascii_lowercase();
    match kind.as_str() {
        "npm" => Some("npm".to_owned()),
        "maven" => Some("Maven".to_owned()),
        "pypi" => Some("PyPI".to_owned()),
        "golang" => Some("Go".to_owned()),
        "cargo" => Some("crates.io".to_owned()),
        "nuget" => Some("NuGet".to_owned()),
        "gem" => Some("RubyGems".to_owned()),
        "deb" => debian_family(query),
        "apk" => alpine(query),
        _ => None,
    }
}

fn distro(query: &str) -> Option<(String, &str)> {
    let value = query
        .split('&')
        .find_map(|pair| pair.strip_prefix("distro="))?;
    let (id, release) = value.split_once('-')?;
    Some((id.to_ascii_lowercase(), release))
}

fn number(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

fn debian_family(query: &str) -> Option<String> {
    let (id, release) = distro(query)?;
    let mut parts = release.split('.');
    let first = parts.next()?;
    match id.as_str() {
        "debian" if number(first) => Some(format!("Debian:{first}")),
        "ubuntu" => {
            let month = parts.next()?;
            let lts = number(first) && month == "04" && first.parse::<u32>().ok()? % 2 == 0;
            lts.then(|| format!("Ubuntu:{first}.{month}:LTS"))
        }
        _ => None,
    }
}

fn alpine(query: &str) -> Option<String> {
    let (id, release) = distro(query)?;
    let mut parts = release.split('.');
    let (major, minor) = (parts.next()?, parts.next()?);
    (id == "alpine" && number(major) && number(minor)).then(|| format!("Alpine:v{major}.{minor}"))
}
