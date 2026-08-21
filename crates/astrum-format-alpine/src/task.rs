use std::sync::Arc;

use astrum_content::store;
use astrum_repo::RepositoryType;
use astrum_security::SecurityModel;
use astrum_tasks::kinds::{ALL, count};
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;

use crate::{Alpine, INDEX, files};

pub const ID: &str = "repository.alpine.rebuild.metadata";
pub const RESET: &str = "resetProxyMetadata";

#[derive(Default)]
pub struct RebuildMetadata {
    alpine: Alpine,
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
        "Alpine - Rebuild Alpine indexes"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("Select the Alpine repository whose APKINDEX files to rebuild")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted", "proxy", "group"])
                .with_all(),
            Field::new(RESET, FieldType::Checkbox, "Reset proxy and group indexes").help(
                "Delete the cached indexes of proxy repositories and the merged indexes of group repositories so that they are fetched or merged again. Hosted indexes are always written anew.",
            ),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => "rebuilds the indexes of every Alpine repository".to_owned(),
            Some(name) => format!("rebuilds the indexes of {name}"),
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
                "{wanted} is not an Alpine repository"
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
                    let rebuilt = self.alpine.rebuild(&ctx).await.map_err(failed)?;
                    run.info(format!(
                        "wrote {} of {name}",
                        count(rebuilt as i64, "index", "indexes")
                    ));
                    done += 1;
                }
                _ if reset => {
                    let cached = files::of_kind(run.db(), &name, INDEX)
                        .await
                        .map_err(failed)?;
                    let deleted = cached.len();
                    for found in cached {
                        store::delete_asset(run.content(), found)
                            .await
                            .map_err(failed)?;
                    }
                    run.content().forget_repository(&name);
                    run.info(format!(
                        "deleted {} of {name}",
                        count(deleted as i64, "stored index", "stored indexes")
                    ));
                    done += 1;
                }
                _ => run.info(format!(
                    "{name} fetches or merges its indexes when they are asked for (check Reset proxy and group indexes to start over)"
                )),
            }
        }
        Ok(format!(
            "rebuilt {}",
            count(done as i64, "repository", "repositories")
        ))
    }
}
