use std::path::PathBuf;

use astrum_db::audit::Event;
use astrum_scan::osv::{
    self, DEFAULT_BASE_URL, DEFAULT_ECOSYSTEMS, DEFAULT_STALE_AFTER_DAYS, HttpDump, Outcome, Stale,
    SyncError,
};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;

use super::count;
use crate::kind::{Field, FieldType, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "advisories.sync";
const MAX_ECOSYSTEMS: usize = 64;

pub struct AdvisorySync {
    dir: PathBuf,
}

impl AdvisorySync {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

fn ecosystems(text: &str) -> Result<Vec<String>, TaskError> {
    let mut found: Vec<String> = Vec::new();
    for name in text
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        if !osv::valid_ecosystem(name) {
            return Err(TaskError::Failed(format!(
                "{name:?} is not an ecosystem name"
            )));
        }
        if !found.iter().any(|known| known == name) {
            found.push(name.to_owned());
        }
    }
    if found.is_empty() || found.len() > MAX_ECOSYSTEMS {
        return Err(TaskError::Failed(format!(
            "the task needs between 1 and {MAX_ECOSYSTEMS} ecosystems"
        )));
    }
    Ok(found)
}

pub fn announce_stale(run: &Run, stale: &[Stale], days: i64) {
    for feed in stale {
        match feed.synced_at {
            Some(at) => run.warn(format!(
                "{}: the advisory data is older than {} (last synced {})",
                feed.ecosystem,
                count(days, "day", "days"),
                at.to_rfc3339()
            )),
            None => run.warn(format!(
                "{}: the advisory data was never synced",
                feed.ecosystem
            )),
        }
    }
}

#[async_trait]
impl TaskKind for AdvisorySync {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Security - Synchronize vulnerability data"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("url", FieldType::Text, "Source URL")
                .help("The base URL of OSV dumps; <base>/<ecosystem>/all.zip is downloaded. Point it at an internal mirror to work offline")
                .required()
                .initial(DEFAULT_BASE_URL),
            Field::new("ecosystems", FieldType::Text, "Ecosystems")
                .help("Comma-separated OSV ecosystem names")
                .required()
                .initial(DEFAULT_ECOSYSTEMS),
            Field::new("staleAfterDays", FieldType::Number, "Stale after days")
                .help("Report advisory data that was not refreshed for this many days")
                .required()
                .initial("7")
                .min(1)
                .max(3650),
        ]
    }

    fn singleton(&self) -> bool {
        true
    }

    fn describe(&self, properties: &Properties) -> String {
        let url = properties
            .get("url")
            .map(String::as_str)
            .unwrap_or(DEFAULT_BASE_URL);
        let ecosystems = properties
            .get("ecosystems")
            .map(String::as_str)
            .unwrap_or(DEFAULT_ECOSYSTEMS);
        format!("downloads vulnerability data for {ecosystems} from {url}")
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let url = run.property("url").unwrap_or(DEFAULT_BASE_URL).to_owned();
        let wanted = ecosystems(run.property("ecosystems").unwrap_or(DEFAULT_ECOSYSTEMS))?;
        let days = run
            .number("staleAfterDays")
            .unwrap_or(DEFAULT_STALE_AFTER_DAYS);
        let source = HttpDump::new(run.content().clone(), &url, self.dir.clone())
            .map_err(TaskError::failed)?;
        let cancelled = || run.cancelled();
        let (mut imported, mut unchanged, mut failed) = (0usize, 0usize, Vec::new());
        for ecosystem in &wanted {
            run.check()?;
            match osv::sync(run.db(), &source, ecosystem, &cancelled).await {
                Ok(Outcome::NotModified) => {
                    unchanged += 1;
                    run.info(format!("{ecosystem}: not modified"));
                }
                Ok(Outcome::Imported(stats)) => {
                    imported += 1;
                    run.count("records", stats.records as i64);
                    run.count("changed", stats.changed as i64);
                    run.count("removed", stats.removed as i64);
                    run.count("skipped", stats.skipped as i64);
                    run.info(format!(
                        "{ecosystem}: {} read, {} changed, {} unchanged, {} removed, {} skipped",
                        stats.records, stats.changed, stats.unchanged, stats.removed, stats.skipped
                    ));
                    run.audit(Event::new("advisories", "imported", ecosystem).with(json!({
                        "records": stats.records,
                        "changed": stats.changed,
                        "removed": stats.removed,
                        "skipped": stats.skipped,
                    })))
                    .await;
                }
                Err(SyncError::Import(osv::ImportError::Cancelled)) => {
                    return Err(TaskError::Cancelled);
                }
                Err(SyncError::Busy(_)) => {
                    run.info(format!("{ecosystem}: already running"));
                }
                Err(error) => {
                    run.error(format!("{ecosystem}: {error}"));
                    failed.push(ecosystem.clone());
                }
            }
        }
        let purged = osv::purge::run(run.db()).await?;
        run.info(format!(
            "deleted {}, {} and {} nobody uses",
            count(purged.inventories as i64, "inventory", "inventories"),
            count(purged.packages as i64, "package", "packages"),
            count(purged.layer_facts as i64, "layer record", "layer records")
        ));
        let stale = osv::stale(run.db(), &wanted, days, Utc::now()).await?;
        announce_stale(run, &stale, days);
        if !failed.is_empty() {
            return Err(TaskError::Failed(format!(
                "{} of {} failed: {}",
                count(failed.len() as i64, "ecosystem", "ecosystems"),
                wanted.len(),
                failed.join(", ")
            )));
        }
        Ok(format!(
            "synchronized {}, {} not modified",
            count(imported as i64, "ecosystem", "ecosystems"),
            unchanged
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::ecosystems;

    #[test]
    fn the_ecosystem_list_is_trimmed_deduplicated_and_checked() {
        assert_eq!(ecosystems(" npm, PyPI ,npm,").unwrap(), ["npm", "PyPI"]);
        assert!(ecosystems("").is_err());
        assert!(ecosystems("npm,../x").is_err());
    }
}
