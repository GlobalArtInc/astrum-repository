mod file;
mod filter;
mod level;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

pub use file::{Chunk, FileWriter, LogFile, LogFiles, MAX_FILE_BYTES, ROTATED_FILES, file_name};
pub use filter::{Directives, ROOT, compose, parse, valid_name};
pub use level::{Level, UnknownLevel};

pub const LOG_DIR: &str = "log";

pub const KNOWN_LOGGERS: &[&str] = &[
    "astrum_api",
    "astrum_blob",
    "astrum_content",
    "astrum_db",
    "astrum_format_docker",
    "astrum_format_helm",
    "astrum_format_maven",
    "astrum_format_npm",
    "astrum_format_nuget",
    "astrum_format_pypi",
    "astrum_format_raw",
    "astrum_repo",
    "astrum_security",
    "astrum_server",
    "astrum_tasks",
    "aws_config",
    "aws_sdk_s3",
    "hyper",
    "reqwest",
    "rustls",
    "sea_orm",
    "sqlx",
    "tower_http",
];

type Apply = dyn Fn(&str) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub struct Logging {
    inner: Arc<Inner>,
}

struct Inner {
    node: String,
    base: Directives,
    files: Option<Arc<LogFiles>>,
    overrides: Mutex<BTreeMap<String, Level>>,
    apply: Box<Apply>,
}

impl std::fmt::Debug for Logging {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logging")
            .field("node", &self.inner.node)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Logger {
    pub name: String,
    pub level: Level,
    #[serde(rename = "override")]
    pub overridden: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "{0:?} is not a logger name; use ROOT or a Rust module path such as astrum_api::routes"
    )]
    Name(String),
    #[error("the logging filter cannot be changed: {0}")]
    Apply(String),
}

impl Logging {
    pub fn new(
        node: impl Into<String>,
        base_filter: &str,
        files: Option<Arc<LogFiles>>,
        apply: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                node: node.into(),
                base: parse(base_filter),
                files,
                overrides: Mutex::new(BTreeMap::new()),
                apply: Box::new(apply),
            }),
        }
    }

    pub fn detached() -> Self {
        Self::new(node_name(), "info", None, |_| Ok(()))
    }

    pub fn node(&self) -> &str {
        &self.inner.node
    }

    pub fn files(&self) -> Option<&LogFiles> {
        self.inner.files.as_deref()
    }

    pub fn filter(&self) -> String {
        compose(&self.inner.base, &self.lock())
    }

    pub fn overrides(&self) -> BTreeMap<String, Level> {
        self.lock().clone()
    }

    pub fn loggers(&self) -> Vec<Logger> {
        let overrides = self.lock().clone();
        let mut names: Vec<String> = KNOWN_LOGGERS
            .iter()
            .map(|name| (*name).to_owned())
            .collect();
        names.push(ROOT.to_owned());
        names.extend(self.inner.base.loggers.iter().map(|(name, _)| name.clone()));
        names.extend(overrides.keys().cloned());
        names.sort();
        names.dedup();
        names
            .into_iter()
            .map(|name| self.describe(&name, &overrides))
            .collect()
    }

    pub fn logger(&self, name: &str) -> Logger {
        let overrides = self.lock().clone();
        self.describe(name, &overrides)
    }

    pub fn set(&self, name: &str, level: Level) -> Result<(), Error> {
        if !valid_name(name) {
            return Err(Error::Name(name.to_owned()));
        }
        let mut overrides = self.lock().clone();
        overrides.insert(name.to_owned(), level);
        self.replace(overrides)
    }

    pub fn reset(&self, name: &str) -> Result<(), Error> {
        let mut overrides = self.lock().clone();
        overrides.remove(name);
        self.replace(overrides)
    }

    pub fn replace(&self, overrides: BTreeMap<String, Level>) -> Result<(), Error> {
        let overrides: BTreeMap<String, Level> = overrides
            .into_iter()
            .filter(|(name, _)| valid_name(name))
            .collect();
        let mut current = self.lock();
        if *current == overrides {
            return Ok(());
        }
        (self.inner.apply)(&compose(&self.inner.base, &overrides)).map_err(Error::Apply)?;
        *current = overrides;
        Ok(())
    }

    fn describe(&self, name: &str, overrides: &BTreeMap<String, Level>) -> Logger {
        if let Some(level) = overrides.get(name) {
            return Logger {
                name: name.to_owned(),
                level: *level,
                overridden: true,
            };
        }
        Logger {
            name: name.to_owned(),
            level: self.inherited(name, overrides),
            overridden: false,
        }
    }

    fn inherited(&self, name: &str, overrides: &BTreeMap<String, Level>) -> Level {
        let configured = self
            .inner
            .base
            .loggers
            .iter()
            .filter(|(target, _)| !overrides.contains_key(target))
            .map(|(target, level)| (target.as_str(), *level))
            .chain(
                overrides
                    .iter()
                    .filter(|(target, _)| target.as_str() != ROOT)
                    .map(|(target, level)| (target.as_str(), *level)),
            );
        let mut best: Option<(usize, Level)> = None;
        for (target, level) in configured {
            if covers(target, name) && best.is_none_or(|(length, _)| target.len() > length) {
                best = Some((target.len(), level));
            }
        }
        match best {
            Some((_, level)) => level,
            None => overrides
                .get(ROOT)
                .copied()
                .or(self.inner.base.root)
                .unwrap_or(Level::Error),
        }
    }

    fn lock(&self) -> MutexGuard<'_, BTreeMap<String, Level>> {
        self.inner
            .overrides
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn covers(target: &str, name: &str) -> bool {
    name == target
        || name
            .strip_prefix(target)
            .is_some_and(|rest| rest.starts_with("::"))
}

pub fn node_name() -> String {
    let configured = std::env::var("ASTRUM_NODE_NAME")
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .filter(|name| !name.trim().is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|name| name.trim().to_owned())
        })
        .filter(|name| !name.is_empty());
    configured.unwrap_or_else(|| "astrum".to_owned())
}

pub fn open_files(data_dir: &Path, node: &str) -> std::io::Result<Arc<LogFiles>> {
    LogFiles::open(&data_dir.join(LOG_DIR), node).map(Arc::new)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StdMutex;

    use super::*;

    fn logging() -> (Logging, Arc<StdMutex<Vec<String>>>) {
        let applied = Arc::new(StdMutex::new(Vec::new()));
        let seen = applied.clone();
        let logging = Logging::new("n1", "info,sqlx=warn", None, move |filter| {
            seen.lock().unwrap().push(filter.to_owned());
            Ok(())
        });
        (logging, applied)
    }

    #[test]
    fn lists_known_loggers_with_their_effective_levels() {
        let (logging, _) = logging();
        let loggers = logging.loggers();
        let find = |name: &str| loggers.iter().find(|logger| logger.name == name).unwrap();
        assert_eq!(find(ROOT).level, Level::Info);
        assert_eq!(find("sqlx").level, Level::Warn);
        assert_eq!(find("astrum_api").level, Level::Info);
        assert!(loggers.iter().all(|logger| !logger.overridden));
    }

    #[test]
    fn overrides_apply_and_reset() {
        let (logging, applied) = logging();
        logging.set("astrum_api::routes", Level::Debug).unwrap();
        logging.set(ROOT, Level::Warn).unwrap();
        assert_eq!(logging.filter(), "warn,sqlx=warn,astrum_api::routes=debug");
        assert_eq!(
            logging.logger("astrum_api::routes::status").level,
            Level::Debug
        );
        assert_eq!(logging.logger("astrum_api").level, Level::Warn);
        assert!(logging.logger("astrum_api::routes").overridden);
        logging.reset("astrum_api::routes").unwrap();
        assert_eq!(logging.logger("astrum_api::routes").level, Level::Warn);
        logging.replace(BTreeMap::new()).unwrap();
        assert_eq!(logging.filter(), "info,sqlx=warn");
        assert_eq!(applied.lock().unwrap().len(), 4);
        logging.replace(BTreeMap::new()).unwrap();
        assert_eq!(applied.lock().unwrap().len(), 4);
    }

    #[test]
    fn refuses_names_that_are_not_module_paths() {
        let (logging, applied) = logging();
        assert!(matches!(
            logging.set("org.example", Level::Debug),
            Err(Error::Name(_))
        ));
        assert!(applied.lock().unwrap().is_empty());
    }

    #[test]
    fn keeps_the_old_filter_when_applying_fails() {
        let logging = Logging::new("n1", "info", None, |_| Err("no".to_owned()));
        assert!(logging.set("sqlx", Level::Debug).is_err());
        assert!(logging.overrides().is_empty());
    }
}
