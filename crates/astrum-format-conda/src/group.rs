use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Format, group};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use futures_util::TryStreamExt;
use sea_orm::{ConnectionTrait, Statement, TransactionTrait};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::paths::{self, IndexFile};
use crate::repodata::{self, Repodata};
use crate::{Conda, ID, METADATA};

const MAX_MERGED: u64 = 2 * 1024 * 1024 * 1024;

/// What a group answers for one of its index files.
pub enum Answer {
    Member(Box<Ctx>, Asset),
    Merged(Asset),
    Generated(Vec<u8>, &'static str),
}

pub async fn read(ctx: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > MAX_MERGED {
        return Err(ContentError::BadGateway(format!(
            "{} of {} is too large to merge",
            found.path, found.repository
        )));
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(found.size.max(0) as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn answers(
    conda: &Conda,
    ctx: &Ctx,
    path: &str,
) -> Result<(Vec<(Ctx, Asset)>, bool), ContentError> {
    let mut found = Vec::new();
    let mut hosted = false;
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        hosted |= member.kind() == RepositoryType::Hosted;
        match conda.get(&member, path).await {
            Ok(Some(asset)) => found.push((member, asset)),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no {path}");
            }
        }
    }
    Ok((found, hosted))
}

fn merge_key(kind: &str, answers: &[(Ctx, Asset)]) -> String {
    let mut digest = Sha256::new();
    digest.update(kind.as_bytes());
    for (member, found) in answers {
        digest.update(format!("\n{}={}", member.name(), found.sha256).as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_key(found: &Asset) -> Option<&str> {
    found.attributes.get(ID)?.get("mergeKey")?.as_str()
}

async fn cached<F, Fut>(
    ctx: &Ctx,
    path: &str,
    key: &str,
    content_type: &str,
    build: F,
) -> Result<Asset, ContentError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<u8>, ContentError>>,
{
    let current = |found: Option<Asset>| found.filter(|found| stored_key(found) == Some(key));
    if let Some(found) = current(store::find(ctx.db(), ctx.name(), path).await?) {
        return Ok(found);
    }
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("conda:{}:{path}", ctx.name()).into()],
    ))
    .await?;
    if let Some(found) = current(store::find(ctx.db(), ctx.name(), path).await?) {
        txn.commit().await?;
        return Ok(found);
    }
    let built = build().await;
    let stored = match built {
        Ok(bytes) => {
            let mut put = PutAsset::hosted(ctx, path)
                .kind(METADATA)
                .content_type(content_type)
                .attributes(json!({ ID: {"mergeKey": key} }));
            put.check_content_type = false;
            put.write_policy = Some(WritePolicy::Allow);
            store::put(ctx, put, astrum_blob::content_of(bytes)).await
        }
        Err(error) => Err(error),
    };
    txn.commit().await?;
    stored
}

async fn merged_json(
    ctx: &Ctx,
    channel: &str,
    subdir: &str,
    answers: &[(Ctx, Asset)],
) -> Result<Asset, ContentError> {
    let key = merge_key("repodata", answers);
    let path = paths::index_path(channel, subdir, IndexFile::Json);
    cached(ctx, &path, &key, IndexFile::Json.content_type(), || async {
        let mut documents = Vec::with_capacity(answers.len());
        for (member, found) in answers {
            documents.push((member.name().to_owned(), read(member, found).await?));
        }
        let subdir = subdir.to_owned();
        let group = ctx.name().to_owned();
        tokio::task::spawn_blocking(move || {
            let mut merged = Repodata::empty(&subdir);
            for (member, bytes) in &documents {
                match Repodata::parse(&subdir, bytes) {
                    Ok(document) => merged.absorb(document),
                    Err(error) => {
                        tracing::warn!(group, member, %error, "the repodata of a group member is skipped");
                    }
                }
            }
            merged.to_bytes()
        })
        .await
        .map_err(ContentError::internal)
    })
    .await
}

async fn derived(
    ctx: &Ctx,
    channel: &str,
    subdir: &str,
    file: IndexFile,
    key: &str,
    source: Asset,
) -> Result<Asset, ContentError> {
    let path = paths::index_path(channel, subdir, file);
    cached(ctx, &path, key, file.content_type(), || async {
        let bytes = read(ctx, &source).await?;
        let subdir = subdir.to_owned();
        tokio::task::spawn_blocking(move || match file {
            IndexFile::Bz2 => Ok(repodata::bz2(&bytes)),
            IndexFile::Zst => Ok(repodata::zst(&bytes)),
            IndexFile::Current => Repodata::parse(&subdir, &bytes)
                .map(|document| document.current().to_bytes())
                .map_err(|error| ContentError::internal(std::io::Error::other(error))),
            IndexFile::Json => Ok(bytes),
        })
        .await
        .map_err(ContentError::internal)?
    })
    .await
}

fn empty_index(subdir: &str, file: IndexFile) -> Vec<u8> {
    let json = Repodata::empty(subdir).to_bytes();
    match file {
        IndexFile::Json | IndexFile::Current => json,
        IndexFile::Bz2 => repodata::bz2(&json),
        IndexFile::Zst => repodata::zst(&json),
    }
}

/// A group's index file of one subdirectory: passed through when one member answers, merged from
/// the members' `repodata.json` (first member wins per file name) when several do.
pub async fn index(
    conda: &Conda,
    ctx: &Ctx,
    channel: &str,
    subdir: &str,
    file: IndexFile,
) -> Result<Option<Answer>, ContentError> {
    let json_path = paths::index_path(channel, subdir, IndexFile::Json);
    let (mut found, hosted) = answers(conda, ctx, &json_path).await?;
    match found.len() {
        0 if hosted => Ok(Some(Answer::Generated(
            empty_index(subdir, file),
            file.content_type(),
        ))),
        0 => Ok(None),
        1 => {
            let (member, json) = found.remove(0);
            if file == IndexFile::Json {
                return Ok(Some(Answer::Member(Box::new(member), json)));
            }
            let path = paths::index_path(channel, subdir, file);
            Ok(conda
                .get(&member, &path)
                .await?
                .map(|asset| Answer::Member(Box::new(member), asset)))
        }
        _ => {
            let merged = merged_json(ctx, channel, subdir, &found).await?;
            if file == IndexFile::Json {
                return Ok(Some(Answer::Merged(merged)));
            }
            let key = merge_key("repodata", &found);
            Ok(Some(Answer::Merged(
                derived(ctx, channel, subdir, file, &key, merged).await?,
            )))
        }
    }
}

/// A group's `channeldata.json`, merged like its indexes.
pub async fn channeldata(
    conda: &Conda,
    ctx: &Ctx,
    channel: &str,
) -> Result<Option<Answer>, ContentError> {
    let path = paths::channeldata_path(channel);
    let (mut found, hosted) = answers(conda, ctx, &path).await?;
    match found.len() {
        0 if hosted => Ok(Some(Answer::Generated(
            repodata::empty_channeldata(),
            "application/json",
        ))),
        0 => Ok(None),
        1 => {
            let (member, asset) = found.remove(0);
            Ok(Some(Answer::Member(Box::new(member), asset)))
        }
        _ => {
            let key = merge_key("channeldata", &found);
            let merged = cached(ctx, &path, &key, "application/json", || async {
                let mut documents = Vec::with_capacity(found.len());
                for (member, asset) in &found {
                    let bytes = read(member, asset).await?;
                    match serde_json::from_slice::<Value>(&bytes) {
                        Ok(document) => documents.push(document),
                        Err(error) => {
                            tracing::warn!(group = ctx.name(), member = member.name(), %error, "the channeldata of a group member is skipped");
                        }
                    }
                }
                tokio::task::spawn_blocking(move || repodata::merge_channeldata(&documents))
                    .await
                    .map_err(ContentError::internal)
            })
            .await?;
            Ok(Some(Answer::Merged(merged)))
        }
    }
}
