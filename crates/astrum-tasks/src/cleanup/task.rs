use std::collections::BTreeMap;

use astrum_db::audit::Event;
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use serde_json::json;

use super::query::Matcher;
use crate::kind::{Properties, TaskError, TaskKind};
use crate::kinds::{count, size};
use crate::run::Run;

pub const ID: &str = "repository.cleanup";
const BATCH: i64 = 200;

pub struct Cleanup;

#[async_trait]
impl TaskKind for Cleanup {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Admin - Cleanup repositories using their associated policies"
    }

    fn singleton(&self) -> bool {
        true
    }

    fn describe(&self, _properties: &Properties) -> String {
        "deletes the components the cleanup policies of each repository match".to_owned()
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let policies: BTreeMap<String, super::Policy> = super::list(run.db())
            .await?
            .into_iter()
            .map(|policy| (policy.name.clone(), policy))
            .collect();
        let repositories = astrum_repo::store::list(run.db()).await?;
        let mut cleaned = 0u64;
        let mut examined = 0u64;
        for repository in repositories {
            run.check()?;
            let names = repository
                .attributes
                .cleanup
                .as_ref()
                .and_then(|cleanup| cleanup.policy_names.clone())
                .unwrap_or_default();
            if names.is_empty() || repository.kind == RepositoryType::Group {
                continue;
            }
            examined += 1;
            let mut deleted_here = 0i64;
            for name in names {
                let Some(policy) = policies.get(&name) else {
                    run.warn(format!(
                        "{}: the cleanup policy {name} does not exist any more",
                        repository.name
                    ));
                    continue;
                };
                if !policy.fits(repository.format.id) {
                    run.warn(format!(
                        "{}: the cleanup policy {name} is for {} repositories",
                        repository.name, policy.format
                    ));
                    continue;
                }
                if policy.criteria.is_empty() {
                    run.info(format!(
                        "{}: the cleanup policy {name} has no criteria and deletes nothing",
                        repository.name
                    ));
                    continue;
                }
                let matcher = Matcher::new(
                    &repository.name,
                    repository.format.id,
                    policy.criteria.clone(),
                );
                let (components, assets, bytes) = apply(run, &matcher, &name).await?;
                deleted_here += components;
                run.info(format!(
                    "{}: the cleanup policy {name} deleted {} with {} ({})",
                    repository.name,
                    count(components, "component", "components"),
                    count(assets, "asset", "assets"),
                    size(bytes)
                ));
            }
            if deleted_here > 0 {
                cleaned += 1;
            }
        }
        let counts = run.counts();
        let components = counts.get("components").copied().unwrap_or(0);
        let assets = counts.get("assets").copied().unwrap_or(0);
        let bytes = counts.get("bytes").copied().unwrap_or(0);
        Ok(match (examined, components) {
            (0, _) => "no repository has a cleanup policy".to_owned(),
            (_, 0) => format!(
                "nothing matched the cleanup policies of {}",
                count(examined as i64, "repository", "repositories")
            ),
            _ => format!(
                "deleted {} with {} ({}) from {cleaned} of {}",
                count(components, "component", "components"),
                count(assets, "asset", "assets"),
                size(bytes),
                count(examined as i64, "repository", "repositories")
            ),
        })
    }
}

async fn apply(run: &Run, matcher: &Matcher, policy: &str) -> Result<(i64, i64, i64), TaskError> {
    let mut after = 0;
    let mut totals = (0, 0, 0);
    loop {
        run.check()?;
        let found = matcher.batch(run.db(), after, BATCH).await?;
        for component in &found {
            run.check()?;
            after = component.id;
            let deleted =
                astrum_content::store::delete_component(run.content(), component.id).await?;
            if !deleted {
                continue;
            }
            totals.0 += 1;
            totals.1 += component.asset_count;
            totals.2 += component.total_size;
            run.count("components", 1);
            run.count("assets", component.asset_count);
            run.count("bytes", component.total_size);
            run.audit(
                Event::new(
                    "component",
                    "deleted",
                    format!(
                        "{}:{}:{}",
                        component.namespace, component.name, component.version
                    ),
                )
                .with(json!({
                    "repository": matcher.repository,
                    "policy": policy,
                    "assets": component.asset_count,
                    "size": component.total_size,
                })),
            )
            .await;
        }
        if (found.len() as i64) < BATCH {
            return Ok(totals);
        }
    }
}
