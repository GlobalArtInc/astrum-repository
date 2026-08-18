use astrum_content::store;
use astrum_content::{Asset, ContentError, Ctx, Format, group};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{Alpine, ID, INDEX, INDEX_TYPE, files, hosted, index, paths, signing};

pub enum GroupIndex {
    Passed(Box<Ctx>, Asset),
    Merged(Asset),
}

impl GroupIndex {
    pub fn asset(&self) -> &Asset {
        match self {
            GroupIndex::Passed(_, found) | GroupIndex::Merged(found) => found,
        }
    }
}

fn merge_key(signer: &str, answers: &[(Ctx, Asset)]) -> String {
    let mut digest = Sha256::new();
    digest.update(signer.as_bytes());
    for (member, found) in answers {
        digest.update(format!("\n{}={}", member.name(), found.sha256).as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_key(found: &Asset) -> Option<&str> {
    files::attribute(found, "mergeKey").and_then(Value::as_str)
}

pub async fn index(
    alpine: &Alpine,
    ctx: &Ctx,
    directory: &str,
) -> Result<Option<GroupIndex>, ContentError> {
    let path = paths::index_path(directory);
    let mut answers = Vec::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match alpine.get(&member, &path).await {
            Ok(Some(found)) => answers.push((member, found)),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no index here");
            }
        }
    }
    if answers.is_empty() {
        return Ok(None);
    }
    let key = alpine.keys.get(ctx)?;
    if key.is_none() && answers.len() == 1 {
        let (member, found) = answers.remove(0);
        return Ok(Some(GroupIndex::Passed(Box::new(member), found)));
    }
    let signer = signing::signer(key.as_deref());
    let wanted = merge_key(&signer, &answers);
    let cached =
        |found: Option<Asset>| found.filter(|found| stored_key(found) == Some(wanted.as_str()));
    if let Some(found) = cached(store::find(ctx.db(), ctx.name(), &path).await?) {
        return Ok(Some(GroupIndex::Merged(found)));
    }
    let lock = files::lock(ctx, directory).await?;
    if let Some(found) = cached(store::find(&lock, ctx.name(), &path).await?) {
        lock.commit().await?;
        return Ok(Some(GroupIndex::Merged(found)));
    }
    let mut texts = Vec::with_capacity(answers.len());
    let mut members = Vec::with_capacity(answers.len());
    for (member, found) in &answers {
        let bytes = match files::bytes(member, found).await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's index cannot be read");
                continue;
            }
        };
        let read = tokio::task::spawn_blocking(move || index::read(&bytes))
            .await
            .map_err(ContentError::internal)?;
        match read {
            Ok(read) => {
                texts.push(read.index);
                members.push(member.name().to_owned());
            }
            Err(error) => {
                tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's index is skipped");
            }
        }
    }
    let (text, count) = index::merge(texts.iter().map(String::as_str));
    let mtime = chrono::Utc::now().timestamp().max(0) as u64;
    let unsigned = index::archive(&hosted::description(ctx.name(), directory), &text, mtime);
    let file = hosted::signed(key.as_deref(), unsigned, mtime);
    let attributes = json!({ ID: {
        "mergeKey": wanted,
        "members": members,
        "packageCount": count,
        "signer": signer,
    }});
    let stored = store::put(
        ctx,
        files::metadata_put(ctx, &path, INDEX, INDEX_TYPE, attributes),
        astrum_blob::content_of(file),
    )
    .await?;
    lock.commit().await?;
    Ok(Some(GroupIndex::Merged(stored)))
}

pub async fn file(
    alpine: &Alpine,
    ctx: &Ctx,
    path: &str,
) -> Result<Option<(Ctx, Asset)>, ContentError> {
    group::first(ctx, |member| async move { alpine.get(&member, path).await }).await
}
