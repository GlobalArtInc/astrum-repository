use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use astrum_blob::{BlobId, BlobStore, Settings, StoreConfig};
use async_trait::async_trait;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use uuid::Uuid;

use super::{ALL, count, size};
use crate::kind::{Field, FieldType, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "blobstore.compact";
pub const SAFETY: Duration = Duration::from_secs(600);
const PAGE: usize = 1000;

pub struct Compact;

#[async_trait]
impl TaskKind for Compact {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Admin - Compact blob store"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("blobstoreName", FieldType::BlobStore, "Blob store")
                .help("The blob store to compact; a group compacts its members")
                .required()
                .with_all()
                .with_groups(),
            Field::new("blobsOlderThan", FieldType::Number, "Grace period in days")
                .help("Blobs written more recently than this are kept even when nothing refers to them")
                .initial("1")
                .min(0)
                .max(3650),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("blobstoreName").map(String::as_str) {
            Some(ALL) | None => {
                "deletes the blobs no asset refers to in every blob store".to_owned()
            }
            Some(name) => format!("deletes the blobs no asset refers to in the blob store {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("blobstoreName").unwrap_or(ALL).to_owned();
        let configs = run.content().blob_stores().configs().await?;
        let stores = physical(&configs, &wanted)?;
        let days = run.number("blobsOlderThan").unwrap_or(1).max(0) as u64;
        let grace = Duration::from_secs(days * 86_400).max(SAFETY);
        let mut deleted = 0i64;
        let mut freed = 0i64;
        for (store, owners) in stores {
            run.check()?;
            let (count, bytes) = compact(run, &store, &owners, grace).await?;
            deleted += count;
            freed += bytes;
        }
        Ok(match deleted {
            0 => "every blob is in use; nothing was deleted".to_owned(),
            _ => format!(
                "deleted {} and freed {}",
                count(deleted, "unused blob", "unused blobs"),
                size(freed)
            ),
        })
    }
}

pub fn physical(
    configs: &[StoreConfig],
    wanted: &str,
) -> Result<BTreeMap<String, Vec<String>>, TaskError> {
    let mut groups_of: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for config in configs {
        if let Settings::Group(group) = &config.settings {
            for member in &group.members {
                groups_of
                    .entry(member.clone())
                    .or_default()
                    .push(config.name.clone());
            }
        }
    }
    let chosen: Vec<&StoreConfig> = match wanted {
        ALL => configs.iter().collect(),
        name => vec![
            configs
                .iter()
                .find(|config| config.name == name)
                .ok_or_else(|| {
                    TaskError::Failed(format!("the blob store {name} does not exist"))
                })?,
        ],
    };
    let mut physical = BTreeMap::new();
    let mut pending: Vec<&StoreConfig> = chosen;
    let mut seen = BTreeSet::new();
    while let Some(config) = pending.pop() {
        if !seen.insert(config.name.clone()) {
            continue;
        }
        match &config.settings {
            Settings::Group(group) => {
                for member in &group.members {
                    if let Some(found) = configs.iter().find(|config| &config.name == member) {
                        pending.push(found);
                    }
                }
            }
            _ => {
                let mut owners = vec![config.name.clone()];
                let mut parents = groups_of.get(&config.name).cloned().unwrap_or_default();
                while let Some(parent) = parents.pop() {
                    if owners.contains(&parent) {
                        continue;
                    }
                    parents.extend(groups_of.get(&parent).cloned().unwrap_or_default());
                    owners.push(parent);
                }
                physical.insert(config.name.clone(), owners);
            }
        }
    }
    Ok(physical)
}

#[derive(FromQueryResult)]
struct Referenced {
    blob_id: Uuid,
}

async fn compact(
    run: &Run,
    name: &str,
    owners: &[String],
    grace: Duration,
) -> Result<(i64, i64), TaskError> {
    let store: Arc<dyn BlobStore> = run.content().blob_stores().get(name).await?;
    let uploading: HashSet<Uuid> = Referenced::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT (chunk ->> 'blob')::uuid AS blob_id FROM docker_upload, \
         jsonb_array_elements(chunks) AS chunk WHERE blob_store = ANY($1)",
        [owners.to_vec().into()],
    ))
    .all(run.db())
    .await?
    .into_iter()
    .map(|found| found.blob_id)
    .collect();
    let cutoff = SystemTime::now() - grace;
    let mut after: Option<BlobId> = None;
    let (mut examined, mut deleted, mut freed) = (0i64, 0i64, 0i64);
    loop {
        run.check()?;
        let page = store.list(after, PAGE).await?;
        let Some(last) = page.last() else {
            break;
        };
        after = Some(last.id);
        examined += page.len() as i64;
        let old: Vec<_> = page.iter().filter(|blob| blob.modified < cutoff).collect();
        if old.is_empty() {
            continue;
        }
        let ids: Vec<Uuid> = old.iter().map(|blob| blob.id.uuid()).collect();
        let referenced: HashSet<Uuid> =
            Referenced::find_by_statement(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT blob_id FROM asset WHERE blob_store = ANY($1) AND blob_id = ANY($2)",
                [owners.to_vec().into(), ids.into()],
            ))
            .all(run.db())
            .await?
            .into_iter()
            .map(|found| found.blob_id)
            .collect();
        for blob in old {
            let id = blob.id.uuid();
            if referenced.contains(&id) || uploading.contains(&id) {
                continue;
            }
            run.check()?;
            if store.delete(blob.id).await? {
                deleted += 1;
                freed += blob.size as i64;
                run.count("blobs", 1);
                run.count("bytes", blob.size as i64);
            }
        }
    }
    run.info(format!(
        "{name}: examined {examined} blobs, deleted {deleted} that nothing refers to ({})",
        size(freed)
    ));
    Ok((deleted, freed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use astrum_blob::{FileSettings, FillPolicy, GroupSettings};

    fn file(name: &str) -> StoreConfig {
        StoreConfig {
            name: name.to_owned(),
            settings: Settings::File(FileSettings {
                path: name.to_owned(),
            }),
            soft_quota: None,
        }
    }

    #[test]
    fn groups_compact_their_members_with_the_group_as_owner() {
        let configs = vec![
            file("a"),
            file("b"),
            file("c"),
            StoreConfig {
                name: "g".to_owned(),
                settings: Settings::Group(GroupSettings {
                    members: vec!["a".to_owned(), "b".to_owned()],
                    fill_policy: FillPolicy::WriteToFirst,
                }),
                soft_quota: None,
            },
        ];
        let all = physical(&configs, ALL).unwrap();
        assert_eq!(all.keys().cloned().collect::<Vec<_>>(), ["a", "b", "c"]);
        assert_eq!(all["a"], ["a", "g"]);
        assert_eq!(all["c"], ["c"]);
        let group = physical(&configs, "g").unwrap();
        assert_eq!(group.keys().cloned().collect::<Vec<_>>(), ["a", "b"]);
        assert!(physical(&configs, "missing").is_err());
    }
}
