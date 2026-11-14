use std::collections::{BTreeMap, HashSet};

use astrum_content::inventory::PackageRef;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use super::apk::ApkPackage;
use super::distro::{self, Distro, Family, OsRelease};
use super::dpkg::DpkgPackage;
use super::facts::LayerFacts;
use super::paths::{self, APK_DB, DPKG_STATUS};

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

#[derive(Default)]
struct State<'a> {
    os_release: Option<&'a OsRelease>,
    apk: Option<&'a [ApkPackage]>,
    dpkg: Option<&'a [DpkgPackage]>,
    status_d: BTreeMap<&'a str, &'a [DpkgPackage]>,
}

impl<'a> State<'a> {
    fn remove(&mut self, removed: &str) {
        self.os_release
            .take_if(|os| paths::covers(removed, &os.path));
        if paths::covers(removed, APK_DB) {
            self.apk = None;
        }
        if paths::covers(removed, DPKG_STATUS) {
            self.dpkg = None;
        }
        self.status_d
            .retain(|path, _| !paths::covers(removed, path));
    }

    fn hide_below(&mut self, dir: &str) {
        self.os_release
            .take_if(|os| paths::hides_below(dir, &os.path));
        if paths::hides_below(dir, APK_DB) {
            self.apk = None;
        }
        if paths::hides_below(dir, DPKG_STATUS) {
            self.dpkg = None;
        }
        self.status_d
            .retain(|path, _| !paths::hides_below(dir, path));
    }

    fn apply(&mut self, layer: &'a LayerFacts) {
        for removed in &layer.removed {
            self.remove(removed);
        }
        for dir in &layer.opaque {
            self.hide_below(dir);
        }
        if let Some(os) = &layer.os_release {
            self.os_release = Some(os);
        }
        if let Some(apk) = &layer.apk {
            self.apk = Some(apk);
        }
        if let Some(dpkg) = &layer.dpkg {
            self.dpkg = Some(dpkg);
        }
        for file in &layer.dpkg_status_d {
            self.status_d.insert(&file.path, &file.packages);
        }
    }

    fn has_packages(&self) -> bool {
        self.apk.is_some_and(|packages| !packages.is_empty())
            || self.dpkg.is_some_and(|packages| !packages.is_empty())
            || self.status_d.values().any(|packages| !packages.is_empty())
    }
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
    let source = package
        .source
        .as_deref()
        .filter(|source| *source != package.name);
    PackageRef {
        ecosystem: distro.ecosystem.clone(),
        name: package
            .source
            .clone()
            .unwrap_or_else(|| package.name.clone()),
        version: package.version.clone(),
        purl: purl(
            "deb",
            &distro.id,
            &package.name,
            &package.version,
            package.arch.as_deref(),
            distro,
            source,
        ),
    }
}

fn note(reasons: &mut Vec<String>, reason: String) {
    if reasons.len() < MAX_REASONS && !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn refs(state: &State<'_>, distro: &Distro) -> (Vec<PackageRef>, bool) {
    match distro.family {
        Family::Apk => {
            let skipped = state.dpkg.is_some_and(|packages| !packages.is_empty())
                || state.status_d.values().any(|packages| !packages.is_empty());
            let packages = state
                .apk
                .unwrap_or_default()
                .iter()
                .map(|package| apk_ref(distro, package))
                .collect();
            (packages, skipped)
        }
        Family::Dpkg => {
            let skipped = state.apk.is_some_and(|packages| !packages.is_empty());
            let packages = state
                .dpkg
                .unwrap_or_default()
                .iter()
                .chain(state.status_d.values().flat_map(|packages| packages.iter()))
                .map(|package| deb_ref(distro, package))
                .collect();
            (packages, skipped)
        }
    }
}

pub fn merge(layers: &[LayerFacts], max_packages: usize) -> Merged {
    let mut state = State::default();
    let mut reasons = Vec::new();
    let mut complete = true;
    for layer in layers {
        state.apply(layer);
        if layer.incomplete {
            complete = false;
            for reason in &layer.reasons {
                note(&mut reasons, reason.clone());
            }
        }
    }
    let Some(distro) = state.os_release.and_then(distro::detect) else {
        if state.has_packages() {
            complete = false;
            note(
                &mut reasons,
                "the image has installed packages but its distribution is unknown or unsupported"
                    .to_owned(),
            );
        }
        return Merged {
            packages: Vec::new(),
            complete,
            reasons,
        };
    };
    let (found, skipped) = refs(&state, &distro);
    if skipped {
        complete = false;
        note(
            &mut reasons,
            "packages of another package manager were skipped".to_owned(),
        );
    }
    let mut seen = HashSet::new();
    let mut packages = Vec::new();
    for package in found {
        let identity = (package.name.clone(), package.version.clone());
        if !seen.insert(identity) {
            continue;
        }
        if packages.len() >= max_packages {
            complete = false;
            note(
                &mut reasons,
                format!("the image lists more than {max_packages} distinct packages"),
            );
            break;
        }
        packages.push(package);
    }
    Merged {
        packages,
        complete,
        reasons,
    }
}

#[cfg(test)]
mod tests;
