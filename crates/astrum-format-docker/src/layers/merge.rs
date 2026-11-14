use std::collections::{BTreeMap, HashSet};

use astrum_content::inventory::PackageRef;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use super::apk::ApkPackage;
use super::distro::{self, Distro, Family, OsRelease};
use super::dpkg::DpkgPackage;
use super::facts::{LayerFacts, StatusFile};
use super::limits::Limits;
use super::paths::{self, APK_DB, DPKG_STATUS, STATUS_D_PREFIX};
use super::records::PACKAGE_OVERHEAD;

const MAX_REASONS: usize = 16;
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub struct Merged {
    pub packages: Vec<PackageRef>,
    pub complete: bool,
    pub reasons: Vec<String>,
}

enum Db<T> {
    Absent,
    Present(Vec<T>),
    Unreadable,
}

impl<T> Db<T> {
    fn listed(&self) -> &[T] {
        match self {
            Db::Present(packages) => packages,
            Db::Absent | Db::Unreadable => &[],
        }
    }
}

pub struct Merge {
    limits: Limits,
    os_release: Option<OsRelease>,
    apk: Db<ApkPackage>,
    dpkg: Db<DpkgPackage>,
    status_d: BTreeMap<String, Vec<DpkgPackage>>,
    status_bytes: usize,
    complete: bool,
    reasons: Vec<String>,
}

impl Merge {
    pub fn new(limits: &Limits) -> Self {
        Merge {
            limits: *limits,
            os_release: None,
            apk: Db::Absent,
            dpkg: Db::Absent,
            status_d: BTreeMap::new(),
            status_bytes: 0,
            complete: true,
            reasons: Vec::new(),
        }
    }

    fn note(&mut self, reason: String) {
        self.complete = false;
        if self.reasons.len() < MAX_REASONS && !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
    }

    fn drop_below(&mut self, dir: &str) {
        let keys: Vec<String> = match dir.is_empty() {
            true => self.status_d.keys().cloned().collect(),
            false => self
                .status_d
                .range(format!("{dir}/")..format!("{dir}0"))
                .map(|(path, _)| path.clone())
                .collect(),
        };
        for path in keys {
            self.forget_status(&path);
        }
    }

    fn forget_status(&mut self, path: &str) {
        if let Some(packages) = self.status_d.remove(path) {
            let cost = path.len()
                + PACKAGE_OVERHEAD
                + packages.iter().map(DpkgPackage::cost).sum::<usize>();
            self.status_bytes = self.status_bytes.saturating_sub(cost);
        }
    }

    fn remove(&mut self, removed: &str) {
        self.os_release
            .take_if(|os| paths::covers(removed, &os.path));
        if paths::covers(removed, APK_DB) {
            self.apk = Db::Absent;
        }
        if paths::covers(removed, DPKG_STATUS) {
            self.dpkg = Db::Absent;
        }
        self.forget_status(removed);
        self.drop_below(removed);
    }

    fn hide_below(&mut self, dir: &str) {
        self.os_release
            .take_if(|os| paths::hides_below(dir, &os.path));
        if paths::hides_below(dir, APK_DB) {
            self.apk = Db::Absent;
        }
        if paths::hides_below(dir, DPKG_STATUS) {
            self.dpkg = Db::Absent;
        }
        self.drop_below(dir);
    }

    fn unreadable(&mut self, path: &str) {
        self.note(format!(
            "{path} is present in a layer but could not be read"
        ));
        match path {
            APK_DB => self.apk = Db::Unreadable,
            DPKG_STATUS => self.dpkg = Db::Unreadable,
            other if other.starts_with(STATUS_D_PREFIX) => self.forget_status(other),
            _ => {}
        }
    }

    fn put_status(&mut self, file: StatusFile) {
        self.forget_status(&file.path);
        let cost = file.cost();
        if self.status_bytes + cost > self.limits.merged_budget {
            self.note(format!(
                "more than {} bytes of dpkg status files",
                self.limits.merged_budget
            ));
            return;
        }
        self.status_bytes += cost;
        self.status_d.insert(file.path, file.packages);
    }

    pub fn apply(&mut self, layer: LayerFacts) {
        if layer.incomplete {
            self.complete = false;
            for reason in layer.reasons {
                self.note(reason);
            }
        }
        for removed in &layer.removed {
            self.remove(removed);
        }
        for dir in &layer.opaque {
            self.hide_below(dir);
        }
        for path in &layer.unreadable {
            self.unreadable(path);
        }
        if let Some(os) = layer.os_release {
            self.os_release = Some(os);
        }
        if let Some(apk) = layer.apk {
            self.apk = Db::Present(apk);
        }
        if let Some(dpkg) = layer.dpkg {
            self.dpkg = Db::Present(dpkg);
        }
        for file in layer.dpkg_status_d {
            self.put_status(file);
        }
    }

    fn has_packages(&self) -> bool {
        !self.apk.listed().is_empty()
            || !self.dpkg.listed().is_empty()
            || self.status_d.values().any(|packages| !packages.is_empty())
    }

    pub fn finish(mut self) -> Merged {
        let Some(distro) = self.os_release.as_ref().and_then(distro::detect) else {
            if self.has_packages() {
                self.note(
                    "the image has installed packages but its distribution is unknown or unsupported"
                        .to_owned(),
                );
            }
            return self.merged(Vec::new());
        };
        let skipped = self.skipped(&distro);
        let mut seen = HashSet::new();
        let mut packages = Vec::new();
        let mut bytes = 0usize;
        let mut capped = false;
        for package in self.refs(&distro) {
            let identity = (package.name.clone(), package.version.clone());
            if seen.contains(&identity) {
                continue;
            }
            let cost = PACKAGE_OVERHEAD
                + package.ecosystem.len()
                + package.name.len()
                + package.version.len()
                + package.purl.len();
            if packages.len() >= self.limits.max_packages
                || bytes + cost > self.limits.merged_budget
            {
                capped = true;
                break;
            }
            seen.insert(identity);
            bytes += cost;
            packages.push(package);
        }
        if skipped {
            self.note("packages of another package manager were skipped".to_owned());
        }
        if capped {
            self.note(format!(
                "the image lists more than {} distinct packages or {} bytes of them",
                self.limits.max_packages, self.limits.merged_budget
            ));
        }
        self.merged(packages)
    }

    fn skipped(&self, distro: &Distro) -> bool {
        match distro.family {
            Family::Apk => {
                !self.dpkg.listed().is_empty()
                    || self.status_d.values().any(|packages| !packages.is_empty())
            }
            Family::Dpkg => !self.apk.listed().is_empty(),
        }
    }

    fn refs<'a>(&'a self, distro: &'a Distro) -> Box<dyn Iterator<Item = PackageRef> + 'a> {
        match distro.family {
            Family::Apk => Box::new(
                self.apk
                    .listed()
                    .iter()
                    .map(move |package| apk_ref(distro, package)),
            ),
            Family::Dpkg => Box::new(
                self.dpkg
                    .listed()
                    .iter()
                    .chain(self.status_d.values().flat_map(|packages| packages.iter()))
                    .map(move |package| deb_ref(distro, package)),
            ),
        }
    }

    fn merged(self, packages: Vec<PackageRef>) -> Merged {
        Merged {
            packages,
            complete: self.complete,
            reasons: self.reasons,
        }
    }
}

#[cfg(test)]
pub fn merge(layers: impl IntoIterator<Item = LayerFacts>, limits: &Limits) -> Merged {
    let mut merge = Merge::new(limits);
    for layer in layers {
        merge.apply(layer);
    }
    merge.finish()
}

fn encoded(text: &str) -> String {
    utf8_percent_encode(text, COMPONENT).to_string()
}

fn purl(
    kind: &str,
    namespace: &str,
    name: &str,
    version: &str,
    arch: Option<&str>,
    distro: &Distro,
    upstream: Option<&str>,
) -> String {
    let mut text = format!(
        "pkg:{kind}/{}/{}@{}",
        encoded(namespace),
        encoded(name),
        encoded(version)
    );
    let mut separator = '?';
    let mut qualifier = |key: &str, value: &str| {
        text.push(separator);
        text.push_str(key);
        text.push('=');
        text.push_str(&encoded(value));
        separator = '&';
    };
    if let Some(arch) = arch {
        qualifier("arch", arch);
    }
    qualifier("distro", &format!("{}-{}", distro.id, distro.release));
    if let Some(upstream) = upstream {
        qualifier("upstream", upstream);
    }
    text
}

fn apk_ref(distro: &Distro, package: &ApkPackage) -> PackageRef {
    let origin = package
        .origin
        .as_deref()
        .filter(|origin| *origin != package.name);
    PackageRef {
        ecosystem: distro.ecosystem.clone(),
        name: package
            .origin
            .clone()
            .unwrap_or_else(|| package.name.clone()),
        version: package.version.clone(),
        purl: purl(
            "apk",
            &distro.id,
            &package.name,
            &package.version,
            package.arch.as_deref(),
            distro,
            origin,
        ),
    }
}

fn deb_ref(distro: &Distro, package: &DpkgPackage) -> PackageRef {
    let upstream = match (&package.source, &package.source_version) {
        (Some(source), Some(version)) => Some(format!("{source}@{version}")),
        (Some(source), None) if *source != package.name => Some(source.clone()),
        _ => None,
    };
    PackageRef {
        ecosystem: distro.ecosystem.clone(),
        name: package
            .source
            .clone()
            .unwrap_or_else(|| package.name.clone()),
        version: package
            .source_version
            .clone()
            .unwrap_or_else(|| package.version.clone()),
        purl: purl(
            "deb",
            &distro.id,
            &package.name,
            &package.version,
            package.arch.as_deref(),
            distro,
            upstream.as_deref(),
        ),
    }
}

#[cfg(test)]
mod tests;
