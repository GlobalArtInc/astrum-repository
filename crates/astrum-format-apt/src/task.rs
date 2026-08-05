use std::sync::Arc;

use astrum_content::store;
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use astrum_security::SecurityModel;
use astrum_tasks::kinds::{ALL, count};
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{Apt, METADATA};

pub const ID: &str = "repository.apt.rebuild.metadata";
pub const FULL: &str = "rebuildAptMetadataFullRebuild";
pub const RESET: &str = "resetProxyMetadata";

#[derive(Default)]
pub struct RebuildMetadata {
    apt: Apt,
}

#[async_trait]
impl TaskKind for RebuildMetadata {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Apt - Rebuild Apt metadata"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("Select the Apt repository to rebuild metadata")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted", "proxy"]),
            Field::new(FULL, FieldType::Checkbox, "Full rebuild (hosted only)").help(
                "Read every package again and write the metadata anew. Only applies to hosted repositories.",
            ),
            Field::new(RESET, FieldType::Checkbox, "Reset proxy metadata").help(
                "Delete the cached metadata so that it is fetched again. Use this after changing the remote URL. Only applies to proxy repositories.",
            ),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => "rebuilds the metadata of every APT repository".to_owned(),
            Some(name) => format!("rebuilds the metadata of {name}"),
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
                "{wanted} is not an APT repository"
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
                    self.apt
                        .rebuild(&ctx)
                        .await
                        .map_err(|error| TaskError::Failed(error.to_string()))?;
                    run.info(format!("rebuilt the metadata of {name}"));
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
                    "{name} is a proxy; its metadata is fetched from the remote (check Reset proxy metadata to fetch it again)"
                )),
            }
        }
        Ok(format!(
            "rebuilt {}",
            count(done as i64, "repository", "repositories")
        ))
    }
}
