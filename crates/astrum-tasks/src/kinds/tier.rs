use std::time::Duration;

use astrum_blob::{Limiter, StoreConfig};
use async_trait::async_trait;

use super::compact::physical;
use super::{ALL, count, expired, size};
use crate::Problems;
use crate::kind::{Field, FieldType, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "blobstore.tier";
const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

mod walk;

use walk::{Plan, Totals};

pub struct Tier;

#[async_trait]
impl TaskKind for Tier {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Admin - Move idle blobs to another store"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("source", FieldType::BlobStore, "Source blob store")
                .help("The blob store whose idle blobs move; a group moves its members")
                .required()
                .with_groups(),
            Field::new("target", FieldType::BlobStore, "Target blob store")
                .help("The blob store the blobs move to; it must not be the source or hold a member of it")
                .required()
                .with_groups(),
            Field::new("repository", FieldType::Repository, "Repository")
                .help("Only the assets of this repository move; * for all of them")
                .required()
                .initial(ALL)
                .with_all(),
            Field::new("idleDays", FieldType::Number, "Idle for days")
                .help("An asset moves when its blob was not written and it was not downloaded for this many days")
                .initial("90")
                .min(0),
            Field::new("minSizeMiB", FieldType::Number, "Minimum size in MiB")
                .help("Smaller assets stay where they are")
                .initial("1")
                .min(0),
            Field::new("maxGiBPerRun", FieldType::Number, "Limit per run in GiB")
                .help("The run stops after moving this many GiB; 0 means no byte limit")
                .initial("50")
                .min(0),
            Field::new("maxAssetsPerRun", FieldType::Number, "Limit per run in assets")
                .help("The run stops after moving this many assets; 0 means no count limit")
                .initial("100000")
                .min(0),
            Field::new("maxMiBPerSecond", FieldType::Number, "Speed limit in MiB per second")
                .help("0 means unlimited")
                .initial("0")
                .min(0),
            Field::new("drainMinutes", FieldType::Number, "Drain in minutes")
                .help("How long a replaced blob stays readable for readers that loaded the old row")
                .initial("60")
                .min(1),
            Field::new("dryRun", FieldType::Checkbox, "Dry run")
                .help("Only count what would move; nothing is written")
                .initial("false"),
        ]
    }

    fn check(&self, properties: &Properties, stores: &[StoreConfig], problems: &mut Problems) {
        if let Some(message) = overlap(properties, stores) {
            problems.add("properties.target", message);
        }
    }

    fn describe(&self, properties: &Properties) -> String {
        match (properties.get("source"), properties.get("target")) {
            (Some(source), Some(target)) => {
                format!("moves idle blobs of the blob store {source} to {target}")
            }
            _ => "moves idle blobs of a blob store to another blob store".to_owned(),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let configs = run.content().blob_stores().configs().await?;
        let mut problems = Problems::default();
        self.check(&run.properties, &configs, &mut problems);
        if let Some(problem) = problems.into_inner().into_iter().next() {
            return Err(TaskError::Failed(problem.message));
        }
        let source = run.property("source").unwrap_or_default();
        let target = run.property("target").unwrap_or_default();
        let stores: Vec<String> = physical(&configs, source)?.into_keys().collect();
        physical(&configs, target)?;
        let plan = plan(run, source, target, stores);
        let freed = match plan.dry_run {
            true => 0,
            false => {
                let (_, bytes) = expired::drain_lapsed_holds(run.content()).await?;
                run.count("freedBytes", bytes as i64);
                bytes
            }
        };
        let mut totals = Totals::default();
        let walked = walk::walk(run, &plan, &mut totals).await;
        record(run, &totals, plan.dry_run);
        walked?;
        Ok(report(&plan, &totals, freed))
    }
}

fn overlap(properties: &Properties, stores: &[StoreConfig]) -> Option<String> {
    let source = properties.get("source")?;
    let target = properties.get("target")?;
    let from = physical(stores, source).ok()?;
    let to = physical(stores, target).ok()?;
    let shared: Vec<&str> = from
        .keys()
        .filter(|store| to.contains_key(*store))
        .map(String::as_str)
        .collect();
    if shared.is_empty() {
        return None;
    }
    Some(match source == target {
        true => format!(
            "the source and the target are the same blob store {source}; a blob store cannot be a source and a target at once"
        ),
        false => format!(
            "the source {source} and the target {target} share {}; a blob store cannot be a source and a target at once",
            shared.join(", ")
        ),
    })
}

fn plan(run: &Run, source: &str, target: &str, stores: Vec<String>) -> Plan {
    let number = |id: &str, default: i64| run.number(id).unwrap_or(default).max(0) as u64;
    Plan {
        source: source.to_owned(),
        target: target.to_owned(),
        stores,
        repository: run
            .property("repository")
            .filter(|name| *name != ALL)
            .map(str::to_owned),
        idle_days: number("idleDays", 90).min(i32::MAX as u64) as i32,
        min_size: number("minSizeMiB", 1)
            .saturating_mul(MIB)
            .min(i64::MAX as u64) as i64,
        bytes_limit: number("maxGiBPerRun", 50).saturating_mul(GIB),
        assets_limit: number("maxAssetsPerRun", 100_000),
        limiter: Limiter::new(number("maxMiBPerSecond", 0).saturating_mul(MIB)),
        drain: Duration::from_secs(number("drainMinutes", 60).saturating_mul(60)),
        dry_run: run.flag("dryRun"),
    }
}

fn record(run: &Run, totals: &Totals, dry_run: bool) {
    run.count("candidates", totals.candidates as i64);
    if !dry_run {
        run.count("moved", totals.moved as i64);
        run.count("bytesMoved", totals.bytes as i64);
    }
    run.count("changed", totals.changed as i64);
    run.count("corrupt", totals.corrupt as i64);
    run.count("failed", totals.failed as i64);
}

fn report(plan: &Plan, totals: &Totals, freed: u64) -> String {
    let moved = format!(
        "{} ({})",
        count(totals.moved as i64, "asset", "assets"),
        size(totals.bytes as i64)
    );
    match plan.dry_run {
        true => format!("would move {moved} from {} to {}", plan.source, plan.target),
        false => format!(
            "moved {moved} from {} to {}; {} changed meanwhile, {} corrupt at source, {} failed; freed {} from earlier runs",
            plan.source,
            plan.target,
            totals.changed,
            totals.corrupt,
            totals.failed,
            size(freed as i64)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astrum_blob::{FileSettings, FillPolicy, GroupSettings, Settings};

    fn file(name: &str) -> StoreConfig {
        StoreConfig {
            name: name.to_owned(),
            settings: Settings::File(FileSettings {
                path: name.to_owned(),
            }),
            soft_quota: None,
        }
    }

    #[test]
    fn overlapping_stores_are_refused_naming_them() {
        let configs = vec![
            file("a"),
            file("b"),
            file("c"),
            StoreConfig {
                name: "g".to_owned(),
                settings: Settings::Group(GroupSettings {
                    members: vec!["a".to_owned(), "b".to_owned()],
                    fill_policy: FillPolicy::WriteToFirst,
                }),
                soft_quota: None,
            },
        ];
        let refused = |source: &str, target: &str| {
            let mut properties = Properties::new();
            properties.insert("source".to_owned(), source.to_owned());
            properties.insert("target".to_owned(), target.to_owned());
            let mut problems = Problems::default();
            Tier.check(&properties, &configs, &mut problems);
            problems.into_inner()
        };
        assert!(refused("a", "c").is_empty());
        assert!(refused("c", "g").is_empty());
        let group = refused("g", "a");
        assert_eq!(group.len(), 1);
        assert_eq!(group[0].id, "properties.target");
        assert_eq!(
            group[0].message,
            "the source g and the target a share a; a blob store cannot be a source and a target at once"
        );
        let same = refused("c", "c");
        assert_eq!(same.len(), 1);
        assert!(same[0].message.contains("same blob store c"), "{same:?}");
    }
}
