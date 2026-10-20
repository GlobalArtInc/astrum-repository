use std::sync::Arc;

use astrum_content::store;
use astrum_db::audit::Event;
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use astrum_security::SecurityModel;
use astrum_tasks::kinds::{ALL, count, size};
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::json;

use crate::{NAR, db};

pub const ID: &str = "repository.nix.gc";

#[derive(Default)]
pub struct Gc;

#[async_trait]
impl TaskKind for Gc {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Nix - Delete NAR files no narinfo uses"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("The hosted Nix repository to clean")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted"])
                .with_all(),
            Field::new("deployOffset", FieldType::Number, "Deploy offset in hours")
                .help("NAR files uploaded more recently than this are kept, so that a nix copy in progress can still add their narinfo files")
                .initial("24")
                .min(0)
                .max(8760),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => {
                "deletes the NAR files no narinfo uses in every hosted Nix repository".to_owned()
            }
            Some(name) => format!("deletes the NAR files no narinfo uses in {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repositoryName").unwrap_or(ALL).to_owned();
        let hours = run.number("deployOffset").unwrap_or(24).max(0);
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
                "{wanted} is not a hosted Nix repository"
            )));
        }
        let cutoff = (Utc::now() - chrono::Duration::hours(hours)).fixed_offset();
        let security = Arc::new(SecurityModel::default());
        let (mut deleted, mut freed) = (0i64, 0i64);
        let total = repositories.len() as i64;
        for repository in repositories {
            run.check()?;
            let name = repository.name.clone();
            let ctx = run
                .content()
                .context(Arc::new(repository), security.clone(), None, "", "");
            let nars = asset::Entity::find()
                .filter(asset::Column::Repository.eq(&name))
                .filter(asset::Column::Format.eq(crate::ID))
                .filter(asset::Column::Kind.eq(NAR))
                .filter(asset::Column::BlobUpdatedAt.lt(cutoff))
                .order_by_asc(asset::Column::Id)
                .all(run.db())
                .await?;
            let (mut here, mut bytes) = (0i64, 0i64);
            for nar in nars {
                run.check()?;
                if !db::referrers(&ctx, &nar.path)
                    .await
                    .map_err(|error| TaskError::Failed(error.to_string()))?
                    .is_empty()
                {
                    continue;
                }
                let path = nar.path.clone();
                let length = nar.size;
                store::delete_asset(run.content(), nar).await?;
                here += 1;
                bytes += length;
                run.count("nars", 1);
                run.count("bytes", length);
                run.audit(
                    Event::new("asset", "deleted", &path)
                        .with(json!({"repository": name, "reason": "no narinfo uses it"})),
                )
                .await;
            }
            run.info(format!(
                "{name}: deleted {} ({})",
                count(here, "unused NAR file", "unused NAR files"),
                size(bytes)
            ));
            deleted += here;
            freed += bytes;
        }
        Ok(format!(
            "deleted {} ({}) in {}",
            count(deleted, "unused NAR file", "unused NAR files"),
            size(freed),
            count(total, "repository", "repositories")
        ))
    }
}
