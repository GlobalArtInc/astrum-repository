use astrum_db::audit::Event;
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::json;

use super::{ALL, count, size};
use crate::kind::{Field, FieldType, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "repository.purge-unused";
const BATCH: u64 = 500;

pub struct PurgeUnused;

#[async_trait]
impl TaskKind for PurgeUnused {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Repository - Delete unused components"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("The proxy repository to purge cached content from")
                .required()
                .types(&["proxy"])
                .with_all(),
            Field::new("lastUsed", FieldType::Number, "Last used in days")
                .help("Delete cached assets that nobody downloaded for this many days")
                .required()
                .initial("30")
                .min(1)
                .max(36500),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        let days = properties.get("lastUsed").cloned().unwrap_or_default();
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => format!(
                "deletes cached content of every proxy repository that was not used for {days} days"
            ),
            Some(name) => {
                format!("deletes cached content of {name} that was not used for {days} days")
            }
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repositoryName").unwrap_or(ALL).to_owned();
        let days = run
            .number("lastUsed")
            .filter(|days| *days >= 1)
            .ok_or_else(|| TaskError::Failed("the number of days is missing".to_owned()))?;
        let repositories: Vec<_> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| repository.kind == RepositoryType::Proxy)
            .filter(|repository| wanted == ALL || repository.name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a proxy repository"
            )));
        }
        let cutoff = (Utc::now() - chrono::Duration::days(days)).fixed_offset();
        let mut total = (0i64, 0i64);
        for repository in &repositories {
            let mut after = 0i64;
            let mut here = (0i64, 0i64);
            loop {
                run.check()?;
                let found = asset::Entity::find()
                    .filter(asset::Column::Repository.eq(repository.name.as_str()))
                    .filter(asset::Column::Id.gt(after))
                    .filter(Expr::cust_with_values(
                        "coalesce(last_downloaded_at, created_at) < $1",
                        [cutoff],
                    ))
                    .order_by_asc(asset::Column::Id)
                    .limit(BATCH)
                    .all(run.db())
                    .await?;
                let fetched = found.len() as u64;
                for unused in found {
                    run.check()?;
                    after = unused.id;
                    let path = unused.path.clone();
                    let bytes = unused.size;
                    astrum_content::store::delete_asset(run.content(), unused).await?;
                    here.0 += 1;
                    here.1 += bytes;
                    run.count("assets", 1);
                    run.count("bytes", bytes);
                    run.audit(
                        Event::new("asset", "deleted", &path)
                            .with(json!({"repository": repository.name, "unusedDays": days})),
                    )
                    .await;
                }
                if fetched < BATCH {
                    break;
                }
            }
            run.info(format!(
                "{}: deleted {} ({}) not used for {}",
                repository.name,
                count(here.0, "asset", "assets"),
                size(here.1),
                count(days, "day", "days")
            ));
            total.0 += here.0;
            total.1 += here.1;
        }
        Ok(format!(
            "deleted {} ({}) from {}",
            count(total.0, "unused asset", "unused assets"),
            size(total.1),
            count(
                repositories.len() as i64,
                "proxy repository",
                "proxy repositories"
            )
        ))
    }
}
