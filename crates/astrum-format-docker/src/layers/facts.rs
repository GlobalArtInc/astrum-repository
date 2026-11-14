use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};

use super::apk::{self, ApkPackage};
use super::distro::{self, OsRelease};
use super::dpkg::{self, DpkgPackage};
use super::entries::{self, Item, Stop};
use super::limits::Limits;
use super::paths::{self, APK_DB, DPKG_STATUS, ETC_OS_RELEASE, STATUS_D_PREFIX, USR_OS_RELEASE};
use super::records::Parsed;
use super::stream::{self, Cut};

const MAX_REASONS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusFile {
    pub path: String,
    pub packages: Vec<DpkgPackage>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerFacts {
    pub os_release: Option<OsRelease>,
    pub apk: Option<Vec<ApkPackage>>,
    pub dpkg: Option<Vec<DpkgPackage>>,
    pub dpkg_status_d: Vec<StatusFile>,
    pub removed: Vec<String>,
    pub opaque: Vec<String>,
    pub incomplete: bool,
    pub reasons: Vec<String>,
}

impl LayerFacts {
    pub fn unavailable(reason: impl Into<String>) -> Self {
        let mut facts = LayerFacts::default();
        facts.note(reason.into());
        facts
    }

    pub fn note(&mut self, reason: String) {
        self.incomplete = true;
        if self.reasons.len() < MAX_REASONS && !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
    }
}

#[derive(Debug)]
pub enum Failure {
    Source(String),
    Cancelled,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Source(message) => {
                write!(formatter, "the layer could not be read: {message}")
            }
            Failure::Cancelled => formatter.write_str("reading the layer was cancelled"),
        }
    }
}

enum Wanted {
    OsRelease,
    Apk,
    Dpkg,
    StatusD,
}

fn wanted(path: &str) -> Option<Wanted> {
    match path {
        ETC_OS_RELEASE | USR_OS_RELEASE => Some(Wanted::OsRelease),
        APK_DB => Some(Wanted::Apk),
        DPKG_STATUS => Some(Wanted::Dpkg),
        _ => path
            .strip_prefix(STATUS_D_PREFIX)
            .filter(|name| !name.is_empty() && !name.contains('/') && !name.ends_with(".md5sums"))
            .map(|_| Wanted::StatusD),
    }
}

struct Gather<'a> {
    facts: LayerFacts,
    limits: &'a Limits,
    packages: usize,
    etc: Option<OsRelease>,
    usr: Option<OsRelease>,
}

impl<'a> Gather<'a> {
    fn new(limits: &'a Limits) -> Self {
        Gather {
            facts: LayerFacts::default(),
            limits,
            packages: 0,
            etc: None,
            usr: None,
        }
    }

    fn read(&mut self, path: &str, size: u64, body: &mut dyn Read) -> io::Result<Option<Vec<u8>>> {
        if size > self.limits.max_file {
            self.facts.note(format!(
                "{path} is larger than {} bytes",
                self.limits.max_file
            ));
            return Ok(None);
        }
        let mut bytes = Vec::new();
        body.take(self.limits.max_file).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != size {
            self.facts
                .note(format!("{path} ended before its declared size"));
            return Ok(None);
        }
        Ok(Some(bytes))
    }

    fn room(&self) -> usize {
        self.limits.max_packages.saturating_sub(self.packages)
    }

    fn account<T>(&mut self, path: &str, parsed: Parsed<T>) -> Vec<T> {
        if parsed.malformed > 0 {
            self.facts.note(format!(
                "{} unreadable package records in {path}",
                parsed.malformed
            ));
        }
        if parsed.truncated {
            self.facts.note(format!(
                "{path} lists more than {} packages",
                self.limits.max_packages
            ));
        }
        self.packages += parsed.packages.len();
        parsed.packages
    }

    fn file(&mut self, path: &str, size: u64, body: &mut dyn Read) -> io::Result<()> {
        let Some(kind) = wanted(path) else {
            return Ok(());
        };
        if matches!(kind, Wanted::StatusD)
            && self.facts.dpkg_status_d.len() >= self.limits.max_status_files
            && !self
                .facts
                .dpkg_status_d
                .iter()
                .any(|file| file.path == path)
        {
            self.facts.note(format!(
                "more than {} files in {}",
                self.limits.max_status_files,
                paths::STATUS_D
            ));
            return Ok(());
        }
        let Some(bytes) = self.read(path, size, body)? else {
            return Ok(());
        };
        match kind {
            Wanted::OsRelease => {
                let parsed = distro::parse_os_release(path, &bytes);
                match path == ETC_OS_RELEASE {
                    true => self.etc = parsed,
                    false => self.usr = parsed,
                }
            }
            Wanted::Apk => {
                let parsed = apk::parse(&bytes, self.room());
                self.facts.apk = Some(self.account(path, parsed));
            }
            Wanted::Dpkg => {
                let parsed = dpkg::parse(&bytes, self.room());
                self.facts.dpkg = Some(self.account(path, parsed));
            }
            Wanted::StatusD => {
                let parsed = dpkg::parse(&bytes, self.room());
                let packages = self.account(path, parsed);
                let file = StatusFile {
                    path: path.to_owned(),
                    packages,
                };
                match self
                    .facts
                    .dpkg_status_d
                    .iter_mut()
                    .find(|existing| existing.path == path)
                {
                    Some(existing) => *existing = file,
                    None => self.facts.dpkg_status_d.push(file),
                }
            }
        }
        Ok(())
    }

    fn item(&mut self, item: Item<'_>) -> io::Result<()> {
        match item {
            Item::File { path, size, body } => self.file(&path, size, body)?,
            Item::Whiteout { path } => {
                if paths::removal_matters(&path) {
                    self.keep(path, true);
                }
            }
            Item::Opaque { dir } => {
                if paths::opaque_matters(&dir) {
                    self.keep(dir, false);
                }
            }
            Item::Link => {}
        }
        Ok(())
    }

    fn keep(&mut self, path: String, removed: bool) {
        let list = match removed {
            true => &mut self.facts.removed,
            false => &mut self.facts.opaque,
        };
        if list.contains(&path) {
            return;
        }
        if list.len() >= self.limits.max_removed {
            self.facts.note(format!(
                "more than {} whiteouts of package databases",
                self.limits.max_removed
            ));
            return;
        }
        list.push(path);
    }

    fn finish(mut self) -> LayerFacts {
        self.facts.os_release = self.etc.take().or_else(|| self.usr.take());
        self.facts
    }
}

fn cut_reason(cut: Cut, limits: &Limits) -> String {
    match cut {
        Cut::Output => format!(
            "the layer is larger than {} bytes uncompressed",
            limits.max_output
        ),
        Cut::Ratio => format!("the layer inflates more than {}:1", limits.max_ratio),
        Cut::Frames => format!(
            "the layer has more than {} compressed members",
            limits.max_frames
        ),
        Cut::Window => format!(
            "the layer asks for a decompression window above {} bytes",
            limits.max_window
        ),
        Cut::Cancelled => "reading the layer was cancelled".to_owned(),
    }
}

fn stop_reason(stop: Stop, limits: &Limits) -> String {
    match stop {
        Stop::Entries => format!("the layer has more than {} entries", limits.max_entries),
        Stop::PaxSize => "an entry size is overridden by a pax header".to_owned(),
    }
}

pub fn extract<R: Read>(
    source: R,
    limits: &Limits,
    cancel: Arc<AtomicBool>,
) -> Result<LayerFacts, Failure> {
    let layer = stream::open(source, *limits, cancel)
        .map_err(|error| Failure::Source(error.to_string()))?;
    let stream::Layer { reader, watch } = layer;
    let mut gather = Gather::new(limits);
    let walked = entries::walk(reader, limits, |item| gather.item(item));
    if watch.cut() == Some(Cut::Cancelled) {
        return Err(Failure::Cancelled);
    }
    if watch.source_failed() {
        let message = match walked {
            Err(error) => error.to_string(),
            Ok(_) => "the blob store stopped answering".to_owned(),
        };
        return Err(Failure::Source(message));
    }
    match (watch.cut(), walked) {
        (Some(cut), _) => gather.facts.note(cut_reason(cut, limits)),
        (None, Ok(Some(stop))) => gather.facts.note(stop_reason(stop, limits)),
        (None, Ok(None)) => {}
        (None, Err(error)) => gather
            .facts
            .note(format!("the layer archive is damaged: {error}")),
    }
    Ok(gather.finish())
}

#[cfg(test)]
mod tests;
