use std::sync::Arc;

use astrum_content::store;
use astrum_repo::RepositoryType;
use astrum_security::SecurityModel;
use astrum_tasks::kinds::{ALL, count};
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;

use crate::{Hex, KEY, REGISTRY, files};

pub const ID: &str = "repository.hex.rebuild.metadata";
pub const RESET: &str = "resetProxyMetadata";

#[derive(Default)]
pub struct RebuildMetadata {
    hex: Hex,
}

fn failed(error: impl std::fmt::Display) -> TaskError {
    TaskError::Failed(error.to_string())
}

#[async_trait]
impl TaskKind for RebuildMetadata {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Hex - Rebuild Hex registries"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("Select the Hex repository whose signed registry to rebuild")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted", "proxy", "group"])
                .with_all(),
            Field::new(RESET, FieldType::Checkbox, "Reset proxy and group registries").help(
                "Delete the cached registry files of proxy repositories and the merged registry files of group repositories so that they are fetched or merged again. Hosted registries are always written anew.",
            ),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => "rebuilds the registries of every Hex repository".to_owned(),
            Some(name) => format!("rebuilds the registry of {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repositoryName").unwrap_or(ALL).to_owned();
        let reset = run.flag(RESET);
        let repositories: Vec<_> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| repository.format.id == crate::ID)
            .filter(|repository| wanted == ALL || repository.name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a Hex repository"
            )));
        }
        let security = Arc::new(SecurityModel::default());
        let mut done = 0;
        for repository in repositories {
            run.check()?;
            let name = repository.name.clone();
            let kind = repository.kind;
            let ctx = run
                .content()
                .context(Arc::new(repository), security.clone(), None, "", "");
            match kind {
                RepositoryType::Hosted => {
                    let written = self.hex.rebuild(&ctx).await.map_err(failed)?;
                    run.info(format!(
                        "wrote {} of {name}",
                        count(written as i64, "registry file", "registry files")
                    ));
                    done += 1;
                }
                _ if reset => {
                    let mut cached = files::of_kind(run.db(), &name, REGISTRY)
                        .await
                        .map_err(failed)?;
                    cached.extend(files::of_kind(run.db(), &name, KEY).await.map_err(failed)?);
                    let deleted = cached.len();
                    for found in cached {
                        store::delete_asset(run.content(), found)
                            .await
                            .map_err(failed)?;
                    }
                    run.content().forget_repository(&name);
                    run.info(format!(
                        "deleted {} of {name}",
                        count(deleted as i64, "stored registry file", "stored registry files")
                    ));
                    done += 1;
                }
                _ => run.info(format!(
                    "{name} fetches or merges its registry when it is asked for (check Reset proxy and group registries to start over)"
                )),
            }
        }
        Ok(format!(
            "rebuilt {}",
            count(done as i64, "repository", "repositories")
        ))
    }
}
