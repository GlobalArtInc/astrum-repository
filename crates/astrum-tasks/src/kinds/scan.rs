use astrum_db::audit::Event;
use astrum_repo::RepositoryType;
use astrum_scan::queue::{self, QueueError, Selection};
use async_trait::async_trait;
use serde_json::json;

use super::{ALL, count};
use crate::kind::{Field, FieldType, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "scan.repository";
pub const REASON: &str = "rescan";
const REPOSITORY: &str = "repositoryName";
const REEXTRACT: &str = "reextract";

pub struct ScanRepository;

#[async_trait]
impl TaskKind for ScanRepository {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Security - Scan components"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new(REPOSITORY, FieldType::Repository, "Repository")
                .help("The repository whose components are queued for a scan; it needs a content policy")
                .required()
                .types(&["hosted", "proxy"])
                .with_all(),
            Field::new(REEXTRACT, FieldType::Checkbox, "Extract again").help(
                "Queue every component. Unchecked, only components never scanned, scanned with an older extractor or whose last scan failed are queued",
            ),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get(REPOSITORY).map(String::as_str) {
            Some(ALL) | None => {
                "queues the components of every repository with a policy for a scan".to_owned()
            }
            Some(name) => format!("queues the components of {name} for a scan"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property(REPOSITORY).unwrap_or(ALL).to_owned();
        let selection = match run.flag(REEXTRACT) {
            true => Selection::Every,
            false => Selection::Unscanned,
        };
        let repositories: Vec<_> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| repository.kind != RepositoryType::Group)
            .filter(|repository| wanted == ALL || repository.name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a hosted or proxy repository"
            )));
        }
        let cancelled = || run.cancelled();
        let (mut seen, mut queued) = (0i64, 0i64);
        for repository in &repositories {
            run.check()?;
            let Some(policy) = repository.policy_name() else {
                if wanted != ALL {
                    return Err(TaskError::Failed(format!(
                        "{wanted} has no content policy, so there is nothing to scan for"
                    )));
                }
                continue;
            };
            let done = queue::enqueue_repository(
                run.db(),
                &repository.name,
                REASON,
                selection,
                queue::CHUNK,
                &cancelled,
            )
            .await
            .map_err(|error| match error {
                QueueError::Cancelled => TaskError::Cancelled,
                QueueError::Database(error) => TaskError::from(error),
            })?;
            seen += done.components as i64;
            queued += done.queued as i64;
            run.count("components", done.components as i64);
            run.count("queued", done.queued as i64);
            run.info(format!(
                "{}: {} checked for policy {policy}, {} queued",
                repository.name,
                count(done.components as i64, "component", "components"),
                done.queued
            ));
            run.audit(Event::new("scan", "queued", &repository.name).with(json!({
                "policy": policy,
                "components": done.components,
                "queued": done.queued,
            })))
            .await;
        }
        Ok(format!(
            "queued {} of {}",
            queued,
            count(seen, "component", "components")
        ))
    }
}
