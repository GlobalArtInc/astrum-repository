use astrum_content::store;
use astrum_content::{Asset, ContentError, Ctx, Format, group};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::paths::Database;
use crate::{DATABASE, DATABASE_TYPE, ID, Pacman, db, files};

pub enum GroupDatabase {
    Passed(Box<Ctx>, Asset),
    Merged(Asset),
}

impl GroupDatabase {
    pub fn asset(&self) -> &Asset {
        match self {
            GroupDatabase::Passed(_, found) | GroupDatabase::Merged(found) => found,
        }
    }
}

fn merge_key(answers: &[(Ctx, Asset)]) -> String {
    let mut digest = Sha256::new();
    for (member, found) in answers {
        digest.update(format!("{}={}\n", member.name(), found.sha256).as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_key(found: &Asset) -> Option<&str> {
    files::attribute(found, "mergeKey").and_then(Value::as_str)
}

pub async fn database(
    pacman: &Pacman,
    ctx: &Ctx,
    wanted: &Database,
) -> Result<Option<GroupDatabase>, ContentError> {
    let path = wanted.path();
    let mut answers = Vec::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match pacman.get(&member, &path).await {
            Ok(Some(found)) => answers.push((member, found)),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no database here");
            }
        }
    }
    if answers.is_empty() {
        return Ok(None);
    }
    if answers.len() == 1 || !wanted.generated() {
        let (member, found) = answers.remove(0);
        return Ok(Some(GroupDatabase::Passed(Box::new(member), found)));
    }
    let key = merge_key(&answers);
    let stored_path = wanted.stored_path();
    let cached =
        |found: Option<Asset>| found.filter(|found| stored_key(found) == Some(key.as_str()));
    if let Some(found) = cached(store::find(ctx.db(), ctx.name(), &stored_path).await?) {
        return Ok(Some(GroupDatabase::Merged(found)));
    }
    let lock = files::lock(ctx, &stored_path).await?;
    if let Some(found) = cached(store::find(&lock, ctx.name(), &stored_path).await?) {
        lock.commit().await?;
        return Ok(Some(GroupDatabase::Merged(found)));
    }
    let mut databases = Vec::with_capacity(answers.len());
    let mut members = Vec::with_capacity(answers.len());
    for (member, found) in &answers {
        let bytes = match files::bytes(member, found, files::MAX_DATABASE).await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's database cannot be read");
                continue;
            }
        };
        match tokio::task::spawn_blocking(move || db::read(&bytes))
            .await
            .map_err(ContentError::internal)?
        {
            Ok(records) => {
                databases.push(records);
                members.push(member.name().to_owned());
            }
            Err(error) => {
                tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's database is skipped");
            }
        }
    }
    let variant = wanted.variant;
    let mtime = chrono::Utc::now().timestamp().max(0) as u64;
    let (file, count) = tokio::task::spawn_blocking(move || {
        let merged = db::merge(databases);
        db::archive(&merged, variant, mtime).map(|file| (file, merged.len()))
    })
    .await
    .map_err(ContentError::internal)?
    .map_err(ContentError::internal)?;
    let attributes = json!({ ID: {
        "mergeKey": key,
        "members": members,
        "packageCount": count,
    }});
    let stored = store::put(
        ctx,
        files::metadata_put(ctx, &stored_path, DATABASE, DATABASE_TYPE, attributes),
        astrum_blob::content_of(file),
    )
    .await?;
    lock.commit().await?;
    Ok(Some(GroupDatabase::Merged(stored)))
}

pub async fn file(
    pacman: &Pacman,
    ctx: &Ctx,
    path: &str,
) -> Result<Option<(Ctx, Asset)>, ContentError> {
    group::first(ctx, |member| async move { pacman.get(&member, path).await }).await
}
