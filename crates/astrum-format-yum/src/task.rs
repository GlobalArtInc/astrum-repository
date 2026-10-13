use std::sync::Arc;

use astrum_repo::RepositoryType;
use astrum_security::SecurityModel;
use astrum_tasks::kinds::{ALL, count};
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;

use crate::{hosted, signing};

pub const ID: &str = "repository.yum.rebuild.metadata";
pub const SOFT: &str = "yumMetadataCaching";

#[derive(Default)]
pub struct RebuildMetadata {
    keys: signing::Keys,
}

#[async_trait]
impl TaskKind for RebuildMetadata {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Repair - Rebuild Yum repository metadata (repodata)"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("Select the Yum repository to rebuild metadata for")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted"]),
            Field::new(SOFT, FieldType::Checkbox, "Soft repair").help(
                "Only update the metadata with RPM changes since the last metadata generation",
            ),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => "rebuilds the repodata of every hosted Yum repository".to_owned(),
            Some(name) => format!("rebuilds the repodata of {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repositoryName").unwrap_or(ALL).to_owned();
        let force = !run.flag(SOFT);
        let repositories: Vec<_> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| {
                repository.format.id == crate::ID && repository.kind == RepositoryType::Hosted
            })
            .filter(|repository| wanted == ALL || repository.name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a hosted Yum repository"
            )));
        }
        let security = Arc::new(SecurityModel::default());
        let mut rebuilt = 0;
        for repository in repositories {
            run.check()?;
            let name = repository.name.clone();
            let ctx = run
                .content()
                .context(Arc::new(repository), security.clone(), None, "", "");
            let bases = hosted::rebuild_all(&ctx, &self.keys, force).await?;
            run.info(format!(
                "rebuilt {} of {name}",
                count(bases as i64, "repodata directory", "repodata directories")
            ));
            rebuilt += bases;
        }
        Ok(format!(
            "rebuilt {}",
            count(rebuilt as i64, "repodata directory", "repodata directories")
        ))
    }
}
