use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use md5::{Digest, Md5};

use crate::gem::{self, Dependency, Spec, is_ruby, version_and_platform};
use crate::marshal::Writer;

pub const SEPARATOR: &str = "---";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfoEntry {
    pub version: String,
    pub platform: String,
    pub dependencies: Vec<Dependency>,
    pub checksum: Option<String>,
    pub ruby: Vec<String>,
    pub rubygems: Vec<String>,
}

pub fn order(left: &InfoEntry, right: &InfoEntry) -> Ordering {
    gem::compare(&left.version, &right.version).then_with(|| {
        let key = |entry: &InfoEntry| match is_ruby(&entry.platform) {
            true => String::new(),
            false => entry.platform.clone(),
        };
        key(left).cmp(&key(right))
    })
}

fn sorted(requirements: &[String]) -> Vec<String> {
    let mut sorted = requirements.to_vec();
    sorted.sort();
    sorted
}

impl InfoEntry {
    pub fn of(spec: &Spec, checksum: &str) -> Self {
        Self {
            version: spec.version.clone(),
            platform: spec.platform.clone(),
            dependencies: spec.runtime_dependencies(),
            checksum: Some(checksum.to_owned()).filter(|checksum| !checksum.is_empty()),
            ruby: spec.required_ruby_version.clone(),
            rubygems: spec.required_rubygems_version.clone(),
        }
    }

    pub fn version_and_platform(&self) -> String {
        version_and_platform(&self.version, &self.platform)
    }

    pub fn line(&self) -> String {
        let mut dependencies = self.dependencies.clone();
        dependencies.sort_by(|a, b| a.name.cmp(&b.name));
        let dependencies: Vec<String> = dependencies
            .iter()
            .map(|dependency| format!("{}:{}", dependency.name, sorted(&dependency.requirements).join("&")))
            .collect();
        let mut requirements = Vec::new();
        if let Some(checksum) = &self.checksum {
            requirements.push(format!("checksum:{checksum}"));
        }
        if !gem::is_any(&self.ruby) {
            requirements.push(format!("ruby:{}", sorted(&self.ruby).join("&")));
        }
        if !gem::is_any(&self.rubygems) {
            requirements.push(format!("rubygems:{}", sorted(&self.rubygems).join("&")));
        }
        format!(
            "{} {}|{}",
            self.version_and_platform(),
            dependencies.join(","),
            requirements.join(",")
        )
    }

    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();
        let (head, rest) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        let (version, platform) = head.split_once('-').unwrap_or((head, gem::RUBY));
        let (dependency_part, requirement_part) = rest.split_once('|').unwrap_or((rest, ""));
        let dependencies = dependency_part
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| {
                let (name, requirements) = part.split_once(':').unwrap_or((part, ""));
                let requirements = match requirements.is_empty() {
                    true => Vec::new(),
                    false => requirements.split('&').map(str::to_owned).collect(),
                };
                Dependency::new(name.trim(), "runtime", requirements)
            })
            .collect();
        let mut entry = InfoEntry {
            version: version.to_owned(),
            platform: gem::normalize_platform(Some(platform)),
            dependencies,
            checksum: None,
            ruby: Vec::new(),
            rubygems: Vec::new(),
        };
        for requirement in requirement_part.split(',') {
            let Some((key, value)) = requirement.split_once(':') else {
                continue;
            };
            match key.trim() {
                "checksum" => entry.checksum = Some(value.trim().to_owned()),
                "ruby" => entry.ruby = value.split('&').map(str::to_owned).collect(),
                "rubygems" => entry.rubygems = value.split('&').map(str::to_owned).collect(),
                _ => {}
            }
        }
        entry
    }
}

pub fn info(entries: &[InfoEntry]) -> Vec<u8> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(order);
    let mut out = format!("{SEPARATOR}\n");
    for entry in &sorted {
        out.push_str(&entry.line());
        out.push('\n');
    }
    out.into_bytes()
}

pub fn body_lines(text: &str) -> Vec<&str> {
    let lines: Vec<&str> = text.lines().collect();
    match lines.iter().position(|line| line.trim() == SEPARATOR) {
        Some(at) => lines[at + 1..].to_vec(),
        None => lines,
    }
}

pub fn parse_info(text: &str) -> Vec<InfoEntry> {
    body_lines(text)
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(InfoEntry::parse)
        .collect()
}

pub fn merge_info(infos: &[String]) -> Vec<u8> {
    if infos.len() == 1 {
        return infos[0].clone().into_bytes();
    }
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();
    for text in infos {
        for entry in parse_info(text) {
            if seen.insert(entry.version_and_platform()) {
                entries.push(entry);
            }
        }
    }
    info(&entries)
}

pub fn md5_hex(data: &[u8]) -> String {
    hex::encode(Md5::digest(data))
}

pub struct VersionsLine {
    pub name: String,
    pub versions: Vec<String>,
    pub checksum: String,
}

impl VersionsLine {
    pub fn parse(line: &str) -> Option<Self> {
        let mut parts = line.trim().splitn(3, ' ');
        let name = parts.next().filter(|name| !name.is_empty())?;
        let versions = parts.next()?;
        Some(Self {
            name: name.to_owned(),
            versions: versions.split(',').map(str::to_owned).collect(),
            checksum: parts.next().unwrap_or_default().trim().to_owned(),
        })
    }
}

pub fn versions(created_at: chrono::DateTime<chrono::Utc>, lines: &[VersionsLine]) -> Vec<u8> {
    let mut out = format!(
        "created_at: {}\n{SEPARATOR}\n",
        created_at.format("%Y-%m-%dT%H:%M:%SZ")
    );
    for line in lines {
        out.push_str(&format!("{} {} {}\n", line.name, line.versions.join(","), line.checksum));
    }
    out.into_bytes()
}

pub fn names<'a>(names: impl IntoIterator<Item = &'a String>) -> Vec<u8> {
    let sorted: BTreeSet<&String> = names.into_iter().collect();
    let mut out = format!("{SEPARATOR}\n");
    for name in sorted {
        out.push_str(name);
        out.push('\n');
    }
    out.into_bytes()
}

pub fn versions_of(entries: &[InfoEntry]) -> Vec<String> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(order);
    sorted.iter().map(InfoEntry::version_and_platform).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Specs {
    All,
    Latest,
    Prerelease,
}

impl Specs {
    pub fn from_path(path: &str) -> Option<Specs> {
        match path {
            "/specs.4.8.gz" => Some(Specs::All),
            "/latest_specs.4.8.gz" => Some(Specs::Latest),
            "/prerelease_specs.4.8.gz" => Some(Specs::Prerelease),
            _ => None,
        }
    }
}

fn split_version(version_and_platform: &str) -> (&str, &str) {
    version_and_platform
        .split_once('-')
        .unwrap_or((version_and_platform, gem::RUBY))
}

fn select(kind: Specs, versions: &[String]) -> Vec<String> {
    let mut selected = Vec::new();
    let mut latest: Vec<(String, String)> = Vec::new();
    for version_and_platform in versions.iter().filter(|value| !value.is_empty()) {
        let (version, platform) = split_version(version_and_platform);
        if !gem::valid_version(version) {
            continue;
        }
        let prerelease = gem::is_prerelease(version);
        match kind {
            Specs::All if !prerelease => selected.push(version_and_platform.clone()),
            Specs::Prerelease if prerelease => selected.push(version_and_platform.clone()),
            Specs::Latest if !prerelease => match latest.iter_mut().find(|(known, _)| known == platform) {
                Some((_, current)) => {
                    if gem::compare(version, split_version(current).0).is_gt() {
                        *current = version_and_platform.clone();
                    }
                }
                None => latest.push((platform.to_owned(), version_and_platform.clone())),
            },
            _ => {}
        }
    }
    if kind == Specs::Latest {
        selected.extend(latest.into_iter().map(|(_, value)| value));
    }
    selected
}

pub fn legacy_specs(kind: Specs, versions: &BTreeMap<String, Vec<String>>) -> Vec<u8> {
    let selected: Vec<(&String, Vec<String>)> = versions
        .iter()
        .map(|(name, list)| (name, select(kind, list)))
        .collect();
    let count = selected.iter().map(|(_, list)| list.len()).sum();
    let mut writer = Writer::new();
    writer.array(count);
    for (name, list) in selected {
        for version_and_platform in list {
            let (version, platform) = split_version(&version_and_platform);
            writer.array(3).string(name).gem_version(version).string(platform);
        }
    }
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(&writer.out)
        .expect("writing to memory never fails");
    encoder.finish().expect("writing to memory never fails")
}

pub struct Platform {
    pub cpu: Option<String>,
    pub os: String,
    pub version: Option<String>,
}

pub fn parse_platform(platform: &str) -> Platform {
    let mut arch: Vec<String> = platform.split('-').map(str::to_owned).collect();
    let trailing_number = |text: &str| {
        regex::Regex::new(r"\d+(\.\d+)?$")
            .map(|pattern| pattern.is_match(text))
            .unwrap_or(false)
    };
    if arch.len() > 2 && !trailing_number(arch.last().map(String::as_str).unwrap_or_default()) {
        let extra = arch.pop().unwrap_or_default();
        if let Some(last) = arch.last_mut() {
            *last = format!("{last}-{extra}");
        }
    }
    let original = arch.remove(0);
    let x86 = regex::Regex::new(r"i\d86").map(|pattern| pattern.is_match(&original)).unwrap_or(false);
    let mut cpu = Some(if x86 { "x86".to_owned() } else { original.clone() });
    let number = regex::Regex::new(r"^\d+(\.\d+)?$").expect("the pattern is valid");
    if arch.len() == 2 && number.is_match(&arch[1]) {
        return Platform {
            cpu,
            os: arch[0].clone(),
            version: Some(arch[1].clone()),
        };
    }
    let os = match arch.first() {
        Some(os) => os.clone(),
        None => {
            cpu = None;
            original
        }
    };
    let rules: [(&str, &str); 16] = [
        (r"aix(\d+)?", "aix"),
        (r"cygwin", "cygwin"),
        (r"darwin(\d+)?", "darwin"),
        (r"^macruby$", "macruby"),
        (r"freebsd(\d+)?", "freebsd"),
        (r"^(?:java|jruby)$", "java"),
        (r"^java([\d.]*)", "java"),
        (r"^dalvik(\d+)?$", "dalvik"),
        (r"^dotnet([\d.]*)", "dotnet"),
        (r"linux-?(\w+)?", "linux"),
        (r"mingw32", "mingw32"),
        (r"mingw-?(\w+)?", "mingw"),
        (r"netbsdelf", "netbsdelf"),
        (r"openbsd(\d+\.\d+)?", "openbsd"),
        (r"solaris(\d+\.\d+)?", "solaris"),
        (r"wasi", "wasi"),
    ];
    for (pattern, name) in rules {
        let Ok(pattern) = regex::Regex::new(pattern) else {
            continue;
        };
        if let Some(found) = pattern.captures(&os) {
            let version = found
                .get(1)
                .map(|value| value.as_str().to_owned())
                .filter(|value| !value.is_empty());
            return Platform {
                cpu,
                os: name.to_owned(),
                version,
            };
        }
    }
    if let Some(found) = regex::Regex::new(r"(mswin\d+)(_(\d+))?")
        .ok()
        .and_then(|pattern| pattern.captures(&os))
    {
        let name = found[1].to_owned();
        return Platform {
            cpu: match (&cpu, name.ends_with("32")) {
                (None, true) => Some("x86".to_owned()),
                _ => cpu,
            },
            os: name,
            version: found.get(3).map(|value| value.as_str().to_owned()),
        };
    }
    Platform {
        cpu,
        os: "unknown".to_owned(),
        version: None,
    }
}

pub fn quick_spec(spec: &Spec) -> Vec<u8> {
    let mut fields = Writer::new();
    fields.array(19);
    fields.string(spec.rubygems_version.as_deref().unwrap_or("3.0.0"));
    fields.integer(spec.specification_version);
    fields.string(&spec.name);
    fields.gem_version(&spec.version);
    match &spec.date {
        Some(date) => fields.string(date),
        None => fields.nil(),
    };
    fields.string(spec.summary.as_deref().unwrap_or(""));
    fields.gem_requirement(&spec.required_ruby_version);
    fields.gem_requirement(&spec.required_rubygems_version);
    fields.string(&spec.platform);
    fields.array(spec.dependencies.len());
    for dependency in &spec.dependencies {
        fields.object("Gem::Dependency", 4);
        fields.symbol("@name").string(&dependency.name);
        fields.symbol("@requirement").gem_requirement(&dependency.requirements);
        fields
            .symbol("@type")
            .symbol(if dependency.runtime() { "runtime" } else { "development" });
        fields.symbol("@prerelease").bool(false);
    }
    fields.string("");
    fields.strings(&spec.email);
    fields.strings(&spec.authors);
    fields.string(spec.description.as_deref().unwrap_or(""));
    fields.string(spec.homepage.as_deref().unwrap_or(""));
    fields.bool(true);
    if is_ruby(&spec.platform) {
        fields.string(gem::RUBY);
    } else {
        let platform = parse_platform(&spec.platform);
        fields.object("Gem::Platform", 3);
        match &platform.cpu {
            Some(cpu) => fields.symbol("@cpu").string(cpu),
            None => fields.symbol("@cpu").nil(),
        };
        fields.symbol("@os").string(&platform.os);
        match &platform.version {
            Some(version) => fields.symbol("@version").string(version),
            None => fields.symbol("@version").nil(),
        };
    }
    fields.strings(&spec.licenses);
    fields.hash(spec.metadata.len());
    for (key, value) in &spec.metadata {
        fields.string(key).string(value);
    }
    let mut outer = Writer::new();
    outer.user_defined("Gem::Specification", &fields.out);
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    encoder
        .write_all(&outer.out)
        .expect("writing to memory never fails");
    encoder.finish().expect("writing to memory never fails")
}

pub fn dependencies(entries: &[(String, Vec<InfoEntry>)]) -> Vec<u8> {
    let count = entries.iter().map(|(_, list)| list.len()).sum();
    let mut writer = Writer::new();
    writer.array(count);
    for (name, list) in entries {
        for entry in list {
            writer.hash(4);
            writer.symbol("name").string(name);
            writer.symbol("number").string(&entry.version);
            writer.symbol("platform").string(&entry.platform);
            writer.symbol("dependencies").array(entry.dependencies.len());
            for dependency in &entry.dependencies {
                writer
                    .array(2)
                    .string(&dependency.name)
                    .string(&dependency.requirements.join(", "));
            }
        }
    }
    writer.out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_lines_round_trip() {
        let line = "1.2.0 rake:< 14&>= 12|checksum:abc,ruby:>= 2.7";
        let entry = InfoEntry::parse(line);
        assert_eq!(entry.dependencies[0].name, "rake");
        assert_eq!(entry.checksum.as_deref(), Some("abc"));
        assert_eq!(entry.line(), line);
        let plain = InfoEntry::parse("0.1.0-java |checksum:x");
        assert_eq!(plain.platform, "java");
        assert_eq!(plain.line(), "0.1.0-java |checksum:x");
        let merged = String::from_utf8(merge_info(&[
            "---\n1.0.0 |checksum:a\n".to_owned(),
            "---\n1.0.0 |checksum:b\n0.9.0 |checksum:c\n".to_owned(),
        ]))
        .unwrap();
        assert_eq!(merged, "---\n0.9.0 |checksum:c\n1.0.0 |checksum:a\n");
    }

    #[test]
    fn legacy_specs_pick_releases_prereleases_and_the_latest() {
        let versions = vec!["1.0.0".to_owned(), "1.1.0".to_owned(), "2.0.0.rc1".to_owned(), "1.0.0-java".to_owned()];
        assert_eq!(select(Specs::All, &versions), ["1.0.0", "1.1.0", "1.0.0-java"]);
        assert_eq!(select(Specs::Prerelease, &versions), ["2.0.0.rc1"]);
        assert_eq!(select(Specs::Latest, &versions), ["1.1.0", "1.0.0-java"]);
        let platform = parse_platform("x86_64-linux");
        assert_eq!((platform.cpu.as_deref(), platform.os.as_str()), (Some("x86_64"), "linux"));
        let platform = parse_platform("java");
        assert_eq!((platform.cpu.as_deref(), platform.os.as_str()), (None, "java"));
    }
}
