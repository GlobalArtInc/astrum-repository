use astrum_content::ContentError;
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use astrum_tasks::kinds::count;
use astrum_tasks::{Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Statement, TransactionTrait,
};

use crate::gc;
use crate::manifest;
use crate::reference::Digest;
use crate::storage::{self, MANIFEST};

pub const LIMIT: usize = 1000;
pub const ID: &str = "docker.children";
const ALL: &str = "*";
const BATCH: u64 = 200;

pub async fn replace(
    db: &DatabaseConnection,
    repository: &str,
    parent: &Digest,
    children: &[Digest],
) -> Result<(), ContentError> {
    let parent = parent.to_string();
    let txn = db.begin().await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM docker_manifest_child WHERE repository = $1 AND parent = $2",
        vec![repository.into(), parent.clone().into()],
    ))
    .await?;
    if !children.is_empty() {
        let listed: Vec<String> = children.iter().map(ToString::to_string).collect();
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO docker_manifest_child (repository, parent, child) \
             SELECT $1, $2, child FROM unnest($3::text[]) AS child ON CONFLICT DO NOTHING",
            vec![repository.into(), parent.into(), listed.into()],
        ))
        .await?;
    }
    txn.commit().await?;
    Ok(())
}

pub async fn forget(
    db: &DatabaseConnection,
    repository: &str,
    parent: &str,
) -> Result<(), ContentError> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM docker_manifest_child WHERE repository = $1 AND parent = $2 \
         AND NOT EXISTS (SELECT 1 FROM asset WHERE repository = $1 AND format = 'docker' \
         AND kind IN ('manifest', 'tag') AND attributes #>> '{docker,digest}' = $2)",
        vec![repository.into(), parent.into()],
    ))
    .await?;
    Ok(())
}

pub struct Children;

#[async_trait]
impl TaskKind for Children {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Docker - Index manifest children"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repository", FieldType::Repository, "Repository")
                .help("The Docker repository whose index manifests are recorded")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted", "proxy"])
                .with_all(),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repository").map(String::as_str) {
            Some(ALL) | None => {
                "records the children of the index manifests of every Docker repository".to_owned()
            }
            Some(name) => format!("records the children of the index manifests of {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repository").unwrap_or(ALL).to_owned();
        let repositories: Vec<String> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| {
                repository.format.id == crate::ID && repository.kind != RepositoryType::Group
            })
            .map(|repository| repository.name)
            .filter(|name| wanted == ALL || *name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a hosted or proxy Docker repository"
            )));
        }
        let mut recorded = 0i64;
        for name in &repositories {
            run.check()?;
            recorded += record(run, name).await?;
        }
        Ok(format!(
            "recorded {} in {}",
            count(recorded, "child", "children"),
            count(repositories.len() as i64, "repository", "repositories")
        ))
    }
}

async fn record(run: &Run, repository: &str) -> Result<i64, TaskError> {
    let mut recorded = 0i64;
    let mut after = 0i64;
    loop {
        run.check()?;
        let batch = asset::Entity::find()
            .filter(asset::Column::Repository.eq(repository))
            .filter(asset::Column::Format.eq(crate::ID))
            .filter(asset::Column::Kind.eq(MANIFEST))
            .filter(asset::Column::Id.gt(after))
            .order_by_asc(asset::Column::Id)
            .limit(BATCH)
            .all(run.db())
            .await?;
        let Some(last) = batch.last().map(|asset| asset.id) else {
            return Ok(recorded);
        };
        let full = batch.len() as u64 == BATCH;
        for asset in batch
            .into_iter()
            .filter(|asset| manifest::is_index(&storage::media_type_of(asset)))
        {
            let Some(parent) = storage::digest_of(&asset) else {
                run.warn(format!(
                    "{repository}: {} is skipped, because it names no digest",
                    asset.path
                ));
                continue;
            };
            let parsed = match gc::read_manifest(run, &asset).await {
                Ok(parsed) => parsed,
                Err(problem) => {
                    run.warn(format!(
                        "{repository}: {} is skipped, because it cannot be read: {problem}",
                        asset.path
                    ));
                    continue;
                }
            };
            if parsed.children.len() > LIMIT {
                run.warn(format!(
                    "{repository}: {} is skipped, because it lists more than {LIMIT} children",
                    asset.path
                ));
                continue;
            }
            replace(run.db(), repository, &parent, &parsed.children).await?;
            run.count("indexes", 1);
            run.count("children", parsed.children.len() as i64);
            recorded += parsed.children.len() as i64;
        }
        after = last;
        if !full {
            return Ok(recorded);
        }
    }
}
