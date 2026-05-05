use std::collections::{HashMap, HashSet};

use astrum_content::store::{self, Asset};
use astrum_db::audit::Event;
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use astrum_tasks::kinds::{count, size};
use astrum_tasks::{Expiry, Field, FieldType, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::json;

use crate::manifest::{self, Manifest};
use crate::storage::{self, BLOB, MANIFEST, TAG};

pub const ID: &str = "repository.docker.gc";
const ALL: &str = "*";

pub struct Gc;

#[async_trait]
impl TaskKind for Gc {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Docker - Delete unused manifests and images"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("repositoryName", FieldType::Repository, "Repository")
                .help("The hosted Docker repository to clean")
                .required()
                .formats(&[crate::ID])
                .types(&["hosted"])
                .with_all(),
            Field::new("deployOffset", FieldType::Number, "Deploy offset in hours")
                .help("Manifests and layers pushed more recently than this are kept")
                .initial("24")
                .min(0)
                .max(8760),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("repositoryName").map(String::as_str) {
            Some(ALL) | None => {
                "deletes untagged manifests and unused layers of every hosted Docker repository"
                    .to_owned()
            }
            Some(name) => format!("deletes untagged manifests and unused layers of {name}"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("repositoryName").unwrap_or(ALL).to_owned();
        let hours = run.number("deployOffset").unwrap_or(24).max(0);
        let repositories: Vec<String> = astrum_repo::store::list(run.db())
            .await?
            .into_iter()
            .filter(|repository| {
                repository.format.id == crate::ID && repository.kind == RepositoryType::Hosted
            })
            .map(|repository| repository.name)
            .filter(|name| wanted == ALL || *name == wanted)
            .collect();
        if wanted != ALL && repositories.is_empty() {
            return Err(TaskError::Failed(format!(
                "{wanted} is not a hosted Docker repository"
            )));
        }
        let cutoff = (Utc::now() - chrono::Duration::hours(hours)).fixed_offset();
        let mut totals = (0i64, 0i64, 0i64);
        for name in &repositories {
            run.check()?;
            let (manifests, blobs, bytes) = collect(run, name, cutoff).await?;
            totals.0 += manifests;
            totals.1 += blobs;
            totals.2 += bytes;
        }
        Ok(format!(
            "deleted {} and {} ({}) in {}",
            count(totals.0, "untagged manifest", "untagged manifests"),
            count(totals.1, "unused layer", "unused layers"),
            size(totals.2),
            count(repositories.len() as i64, "repository", "repositories")
        ))
    }
}

pub struct UploadExpiry;

#[async_trait]
impl Expiry for UploadExpiry {
    fn name(&self) -> &'static str {
        "Docker uploads unfinished for 24 hours"
    }

    async fn expire(&self, content: &astrum_content::Content) -> Result<u64, TaskError> {
        Ok(crate::upload::sweep(content).await?)
    }
}

fn digest_of(asset: &Asset) -> Option<String> {
    asset
        .attributes
        .pointer("/docker/digest")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

async fn read_manifest(run: &Run, asset: &Asset) -> Result<Manifest, String> {
    let blob_store = run
        .content()
        .blob_stores()
        .get(&asset.blob_store)
        .await
        .map_err(|error| error.to_string())?;
    let content = blob_store
        .read(asset.blob_id.into())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("the blob of {} is missing", asset.path))?;
    let mut bytes = Vec::new();
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(|error| error.to_string())? {
        if bytes.len() + chunk.len() > crate::MANIFEST_LIMIT {
            return Err(format!("{} is too large to be a manifest", asset.path));
        }
        bytes.extend_from_slice(&chunk);
    }
    manifest::parse(&bytes, Some(&storage::media_type_of(asset)))
}

async fn manifests(
    run: &Run,
    repository: &str,
    since: Option<DateTime<FixedOffset>>,
) -> Result<Vec<Asset>, TaskError> {
    let mut query = asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Format.eq(crate::ID))
        .filter(asset::Column::Kind.is_in([TAG, MANIFEST]));
    if let Some(since) = since {
        query = query.filter(asset::Column::BlobUpdatedAt.gte(since));
    }
    Ok(query.order_by_asc(asset::Column::Id).all(run.db()).await?)
}

async fn parse_all(
    run: &Run,
    assets: &[Asset],
    parsed: &mut HashMap<String, Manifest>,
) -> Result<(), String> {
    for asset in assets {
        let Some(digest) = digest_of(asset) else {
            return Err(format!("{} names no digest", asset.path));
        };
        if parsed.contains_key(&digest) {
            continue;
        }
        let manifest = read_manifest(run, asset).await?;
        parsed.insert(digest, manifest);
    }
    Ok(())
}

pub fn reachable(tags: &[String], parsed: &HashMap<String, Manifest>) -> HashSet<String> {
    let mut reachable: HashSet<String> = HashSet::new();
    let mut pending: Vec<String> = tags.to_vec();
    loop {
        while let Some(digest) = pending.pop() {
            if !reachable.insert(digest.clone()) {
                continue;
            }
            if let Some(manifest) = parsed.get(&digest) {
                pending.extend(manifest.children.iter().map(ToString::to_string));
            }
        }
        for (digest, manifest) in parsed {
            if !reachable.contains(digest)
                && manifest
                    .subject
                    .as_ref()
                    .is_some_and(|subject| reachable.contains(&subject.to_string()))
            {
                pending.push(digest.clone());
            }
        }
        if pending.is_empty() {
            return reachable;
        }
    }
}

async fn collect(
    run: &Run,
    repository: &str,
    cutoff: DateTime<FixedOffset>,
) -> Result<(i64, i64, i64), TaskError> {
    let started = Utc::now().fixed_offset();
    let assets = manifests(run, repository, None).await?;
    let mut parsed = HashMap::new();
    if let Err(problem) = parse_all(run, &assets, &mut parsed).await {
        run.warn(format!(
            "{repository}: skipped, because a manifest cannot be read: {problem}"
        ));
        return Ok((0, 0, 0));
    }
    let tags: Vec<String> = assets
        .iter()
        .filter(|asset| asset.kind == TAG)
        .filter_map(digest_of)
        .collect();
    let reachable = reachable(&tags, &parsed);
    let mut referenced: HashSet<String> = HashSet::new();
    let mut deleted_manifests = 0i64;
    for asset in assets {
        run.check()?;
        let digest = digest_of(&asset).unwrap_or_default();
        let doomed = asset.kind == MANIFEST
            && !reachable.contains(&digest)
            && asset.blob_updated_at < cutoff;
        if !doomed {
            if let Some(manifest) = parsed.get(&digest) {
                referenced.extend(manifest.blobs.iter().map(ToString::to_string));
            }
            continue;
        }
        let path = asset.path.clone();
        store::delete_asset(run.content(), asset).await?;
        deleted_manifests += 1;
        run.count("manifests", 1);
        run.audit(
            Event::new("asset", "deleted", &path)
                .with(json!({"repository": repository, "reason": "untagged manifest"})),
        )
        .await;
    }
    let fresh = manifests(run, repository, Some(started)).await?;
    let mut fresh_parsed = HashMap::new();
    if let Err(problem) = parse_all(run, &fresh, &mut fresh_parsed).await {
        run.warn(format!(
            "{repository}: layers were kept, because a new manifest cannot be read: {problem}"
        ));
        return Ok((deleted_manifests, 0, 0));
    }
    for manifest in fresh_parsed.values() {
        referenced.extend(manifest.blobs.iter().map(ToString::to_string));
    }
    let blobs = asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Format.eq(crate::ID))
        .filter(asset::Column::Kind.eq(BLOB))
        .filter(asset::Column::BlobUpdatedAt.lt(cutoff))
        .order_by_asc(asset::Column::Id)
        .all(run.db())
        .await?;
    let (mut deleted_blobs, mut freed) = (0i64, 0i64);
    for blob in blobs {
        run.check()?;
        let Some(digest) = digest_of(&blob) else {
            continue;
        };
        if referenced.contains(&digest) {
            continue;
        }
        let path = blob.path.clone();
        let size = blob.size;
        store::delete_asset(run.content(), blob).await?;
        deleted_blobs += 1;
        freed += size;
        run.count("layers", 1);
        run.count("bytes", size);
        run.audit(
            Event::new("asset", "deleted", &path)
                .with(json!({"repository": repository, "reason": "unused layer"})),
        )
        .await;
    }
    run.info(format!(
        "{repository}: deleted {} and {} ({})",
        count(deleted_manifests, "untagged manifest", "untagged manifests"),
        count(deleted_blobs, "unused layer", "unused layers"),
        size(freed)
    ));
    Ok((deleted_manifests, deleted_blobs, freed))
}
