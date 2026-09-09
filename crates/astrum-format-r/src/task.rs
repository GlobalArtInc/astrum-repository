use std::sync::Arc;

use astrum_content::store;
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use astrum_security::SecurityModel;
use astrum_tasks::kinds::{ALL, count};
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{METADATA, R};

pub const ID: &str = "repository.r.rebuild.metadata";
pub const RESET: &str = "resetProxyMetadata";

#[derive(Default)]
pub struct RebuildMetadata {
    r: R,
}

#[async_trait]
impl TaskKind for RebuildMetadata {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Repair - Rebuild R repository metadata (PACKAGES)"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("Select the R repository to rebuild the PACKAGES files of")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted", "proxy"]),
            Field::new(RESET, FieldType::Checkbox, "Reset proxy metadata").help(
                "Delete the cached PACKAGES files so that they are fetched again. Only applies to proxy repositories.",
            ),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => "rebuilds the PACKAGES files of every R repository".to_owned(),
            Some(name) => format!("rebuilds the PACKAGES files of {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repositoryName").unwrap_or(ALL).to_owned();
        let reset = run.flag(RESET);
        let repositories: Vec<_> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| {
                repository.format.id == crate::ID && repository.kind != RepositoryType::Group
            })
            .filter(|repository| wanted == ALL || repository.name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a hosted or proxy R repository"
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
                    let directories = self
                        .r
                        .rebuild(&ctx)
                        .await
                        .map_err(|error| TaskError::Failed(error.to_string()))?;
                    run.info(format!(
                        "rebuilt {} of {name}",
                        count(directories as i64, "package directory", "package directories")
                    ));
                    done += 1;
                }
                _ if reset => {
                    let cached = asset::Entity::find()
                        .filter(asset::Column::Repository.eq(&name))
                        .filter(asset::Column::Kind.eq(METADATA))
                        .all(run.db())
                        .await
                        .map_err(|error| TaskError::Failed(error.to_string()))?;
                    let deleted = cached.len();
                    for found in cached {
                        store::delete_asset(run.content(), found)
                            .await
                            .map_err(|error| TaskError::Failed(error.to_string()))?;
                    }
                    run.content().forget_repository(&name);
                    run.info(format!(
                        "deleted {} of {name}",
                        count(deleted as i64, "cached metadata file", "cached metadata files")
                    ));
                    done += 1;
                }
                _ => run.info(format!(
                    "{name} is a proxy; its PACKAGES files are fetched from the remote (check Reset proxy metadata to fetch them again)"
                )),
            }
        }
        Ok(format!(
            "rebuilt {}",
            count(done as i64, "repository", "repositories")
        ))
    }
}
