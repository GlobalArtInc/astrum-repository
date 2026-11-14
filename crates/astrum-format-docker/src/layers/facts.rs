use std::collections::{HashMap, HashSet};
use std::io::{self, Read};

use serde::{Deserialize, Serialize};

use super::apk::{self, ApkPackage};
use super::distro::{self, OsRelease};
use super::dpkg::{self, Database, DpkgPackage};
use super::entries::{self, Item, Stop};
use super::limits::Limits;
use super::paths::{self, APK_DB, DPKG_STATUS, ETC_OS_RELEASE, STATUS_D_PREFIX, USR_OS_RELEASE};
use super::records::{Budget, PACKAGE_OVERHEAD, Parsed, Truncation};
use super::stream::{self, Cut, Guard, Watch};

const MAX_REASONS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusFile {
    pub path: String,
    pub packages: Vec<DpkgPackage>,
}

impl StatusFile {
    pub fn cost(&self) -> usize {
        self.path.len()
            + PACKAGE_OVERHEAD
            + self.packages.iter().map(DpkgPackage::cost).sum::<usize>()
    }
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
    pub unreadable: Vec<String>,
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
    Deadline,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Source(message) => {
                write!(formatter, "the layer could not be read: {message}")
            }
            Failure::Cancelled => formatter.write_str("reading the layer was cancelled"),
            Failure::Deadline => formatter.write_str("reading the layer passed its deadline"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Wanted {
    OsRelease,
    Apk,
    Dpkg,
    StatusD,
}

impl Wanted {
    fn database(self) -> bool {
        self != Wanted::OsRelease
    }
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
    budget: Budget,
    retained: usize,
    etc: Option<OsRelease>,
    usr: Option<OsRelease>,
    unreadable_seen: HashSet<String>,
    removed_seen: HashSet<String>,
    opaque_seen: HashSet<String>,
    files_seen: HashSet<String>,
    status_index: HashMap<String, usize>,
}

impl<'a> Gather<'a> {
    fn new(limits: &'a Limits) -> Self {
        Gather {
            facts: LayerFacts::default(),
            limits,
            budget: Budget::new(limits.facts_budget),
            retained: 0,
            etc: None,
            usr: None,
            unreadable_seen: HashSet::new(),
            removed_seen: HashSet::new(),
            opaque_seen: HashSet::new(),
            files_seen: HashSet::new(),
            status_index: HashMap::new(),
        }
    }

    fn over_budget(&mut self) {
        self.facts.note(format!(
            "more than {} bytes of package data in one layer",
            self.limits.facts_budget
        ));
    }

    fn exhausted(&self) -> bool {
        self.retained >= self.limits.max_db_packages || self.budget.left() < PACKAGE_OVERHEAD
    }

    fn unreadable(&mut self, path: &str, kind: Wanted) {
        if !kind.database() || self.unreadable_seen.contains(path) {
            return;
        }
        if self.unreadable_seen.len() >= self.limits.max_removed {
            self.facts.note(format!(
                "more than {} unreadable package databases",
                self.limits.max_removed
            ));
            return;
        }
        if !self.budget.take(path.len()) {
            self.over_budget();
            return;
        }
        self.unreadable_seen.insert(path.to_owned());
        self.facts.unreadable.push(path.to_owned());
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

    fn account<T>(&mut self, path: &str, parsed: Parsed<T>) -> Vec<T> {
        if parsed.malformed > 0 {
            self.facts.note(format!(
                "{} unreadable package records in {path}",
                parsed.malformed
            ));
        }
        match parsed.truncated {
            Some(Truncation::Count) => self.facts.note(format!(
                "a layer lists more than {} packages",
                self.limits.max_db_packages
            )),
            Some(Truncation::Budget) => self.over_budget(),
            None => {}
        }
        self.retained += parsed.packages.len();
        parsed.packages
    }

    fn room(&self) -> usize {
        self.limits.max_db_packages.saturating_sub(self.retained)
    }

    fn file(&mut self, path: &str, size: u64, body: &mut dyn Read) -> io::Result<()> {
        let Some(kind) = wanted(path) else {
            return Ok(());
        };
        if kind.database() && self.exhausted() {
            self.facts.note(format!(
                "package data beyond {} packages or {} bytes in one layer was left unread",
                self.limits.max_db_packages, self.limits.facts_budget
            ));
            self.unreadable(path, kind);
            return Ok(());
        }
        if kind == Wanted::StatusD
            && !self.status_index.contains_key(path)
            && self.facts.dpkg_status_d.len() >= self.limits.max_status_files
        {
            self.facts.note(format!(
                "more than {} files in {}",
                self.limits.max_status_files,
                paths::STATUS_D
            ));
            self.unreadable(path, kind);
            return Ok(());
        }
        let Some(bytes) = self.read(path, size, body)? else {
            self.unreadable(path, kind);
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
                let parsed = apk::parse(&bytes, self.room(), &mut self.budget);
                self.facts.apk = Some(self.account(path, parsed));
            }
            Wanted::Dpkg => {
                let parsed = dpkg::parse(&bytes, self.room(), &mut self.budget, Database::Status);
                self.facts.dpkg = Some(self.account(path, parsed));
            }
            Wanted::StatusD => {
                let parsed = dpkg::parse(&bytes, self.room(), &mut self.budget, Database::Part);
                let packages = self.account(path, parsed);
                self.put_status(path, packages);
            }
        }
        Ok(())
    }

    fn put_status(&mut self, path: &str, packages: Vec<DpkgPackage>) {
        if !self.budget.take(path.len() + PACKAGE_OVERHEAD) {
            self.over_budget();
            self.unreadable(path, Wanted::StatusD);
            return;
        }
        let file = StatusFile {
            path: path.to_owned(),
            packages,
        };
        match self.status_index.get(path) {
            Some(position) => self.facts.dpkg_status_d[*position] = file,
            None => {
                self.status_index
                    .insert(path.to_owned(), self.facts.dpkg_status_d.len());
                self.facts.dpkg_status_d.push(file);
            }
        }
    }

    fn special(&mut self, path: &str, what: &str) {
        let Some(kind) = wanted(path) else {
            return;
        };
        if kind.database() || what == "sparse" {
            self.facts
                .note(format!("{path} is a {what} entry, not a regular file"));
        }
        self.unreadable(path, kind);
    }

    fn conflict(&mut self, path: &str) {
        self.facts.note(format!(
            "a whiteout and an entry for {path} are in the same layer"
        ));
        if let Some(kind) = wanted(path) {
            self.unreadable(path, kind);
        }
    }

    fn entry(&mut self, path: &str, regular: bool) {
        if regular && paths::removal_matters(path) {
            if self.removed_seen.contains(path) {
                self.conflict(path);
            }
            if !self.files_seen.contains(path) {
                match self.budget.take(path.len() + PACKAGE_OVERHEAD) {
                    true => {
                        self.files_seen.insert(path.to_owned());
                    }
                    false => self.over_budget(),
                }
            }
        }
        if paths::is_ancestor(path) {
            self.facts
                .note(format!("{path} is not a directory in this layer"));
            self.keep(path.to_owned(), true);
        }
    }

    fn item(&mut self, item: Item<'_>) -> io::Result<()> {
        match item {
            Item::File { path, size, body } => {
                self.entry(&path, true);
                self.file(&path, size, body)?;
            }
            Item::Link { path } => {
                self.entry(&path, false);
                self.special(&path, "link");
            }
            Item::Node { path } => self.entry(&path, false),
            Item::Sparse { path } => {
                self.entry(&path, false);
                self.special(&path, "sparse");
            }
            Item::Whiteout { path } => {
                if paths::removal_matters(&path) {
                    let seen = self.files_seen.contains(&path);
                    self.keep(path.clone(), true);
                    if seen {
                        self.conflict(&path);
                    }
                }
            }
            Item::Opaque { dir } => {
                if paths::opaque_matters(&dir) {
                    self.keep(dir, false);
                }
            }
        }
        Ok(())
    }

    fn keep(&mut self, path: String, removed: bool) {
        let (list, seen) = match removed {
            true => (&mut self.facts.removed, &mut self.removed_seen),
            false => (&mut self.facts.opaque, &mut self.opaque_seen),
        };
        if seen.contains(&path) {
            return;
        }
        if seen.len() >= self.limits.max_removed || !self.budget.take(path.len()) {
            self.facts.note(format!(
                "more than {} whiteouts of package databases",
                self.limits.max_removed
            ));
            return;
        }
        seen.insert(path.clone());
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
        Cut::Frames => format!(
            "the layer has more than {} compressed members",
            limits.max_frames
        ),
        Cut::Window => format!(
            "the layer asks for a decompression window above {} bytes",
            limits.max_window
        ),
        Cut::Stalled => format!(
            "the layer needs more than {} bytes of input for less than {} bytes of output",
            limits.stall_input, limits.stall_output
        ),
        Cut::Deadline | Cut::Cancelled => "reading the layer was cut short".to_owned(),
    }
}

fn stop_reason(stop: Stop, limits: &Limits) -> String {
    match stop {
        Stop::Entries => format!("the layer has more than {} entries", limits.max_entries),
        Stop::Malformed => "the archive has a zero block followed by more data".to_owned(),
        Stop::Metadata => format!(
            "a long name or pax block is larger than {} bytes",
            limits.max_meta
        ),
        Stop::Pax => "a pax header could not be parsed".to_owned(),
    }
}

fn failure(watch: &Watch, message: String) -> Option<Failure> {
    match watch.cut() {
        Some(Cut::Cancelled) => return Some(Failure::Cancelled),
        Some(Cut::Deadline) => return Some(Failure::Deadline),
        _ => {}
    }
    watch.source_failed().then_some(Failure::Source(message))
}

pub fn extract<R: Read>(source: R, limits: &Limits, guard: Guard) -> Result<LayerFacts, Failure> {
    let watch = stream::watch(guard, *limits);
    let reader = match stream::open(source, *limits, watch.clone()) {
        Ok(reader) => reader,
        Err(error) => {
            let message = error.to_string();
            return Err(failure(&watch, message.clone()).unwrap_or(Failure::Source(message)));
        }
    };
    let mut gather = Gather::new(limits);
    let walked = entries::walk(reader, limits, |item| gather.item(item));
    let message = match &walked {
        Err(error) => error.to_string(),
        Ok(_) => "the blob store stopped answering".to_owned(),
    };
    if let Some(failure) = failure(&watch, message) {
        return Err(failure);
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
