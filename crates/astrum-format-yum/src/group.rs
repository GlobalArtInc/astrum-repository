use std::collections::{HashMap, HashSet};

use astrum_content::store;
use astrum_content::{Asset, ContentError, Ctx, Format, group};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::merge::{self, MemberFiles};
use crate::repodata::{MetadataType, Repomd};
use crate::{GZIP_TYPE, REPODATA, REPOMD, XML_TYPE, Yum, files, paths, signing};

pub enum GroupRepomd {
    Passed(Box<Ctx>, Asset),
    Merged(Asset),
}

impl GroupRepomd {
    pub fn asset(&self) -> &Asset {
        match self {
            GroupRepomd::Passed(_, found) | GroupRepomd::Merged(found) => found,
        }
    }
}

fn merge_key(signer: &str, repomds: &[(Ctx, Vec<u8>)]) -> String {
    let mut digest = Sha256::new();
    digest.update(signer.as_bytes());
    for (member, bytes) in repomds {
        digest.update(
            format!("\n{}={}", member.name(), hex::encode(Sha256::digest(bytes))).as_bytes(),
        );
    }
    hex::encode(digest.finalize())
}

fn stored_key(found: &Asset) -> Option<&str> {
    found
        .attributes
        .get("yum")
        .and_then(|yum| yum.get("mergeKey"))
        .and_then(Value::as_str)
}

pub async fn repomd(yum: &Yum, ctx: &Ctx, base: &str) -> Result<Option<GroupRepomd>, ContentError> {
    let path = paths::repomd_path(base);
    let mut answers = Vec::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match yum.get(&member, &path).await {
            Ok(Some(found)) => answers.push((member, found)),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no repomd.xml");
            }
        }
    }
    if answers.is_empty() {
        return Ok(None);
    }
    if answers.len() == 1 {
        let (member, found) = answers.remove(0);
        return Ok(Some(GroupRepomd::Passed(Box::new(member), found)));
    }
    let mut repomds = Vec::with_capacity(answers.len());
    for (member, found) in answers {
        match files::bytes(&member, &found).await {
            Ok(bytes) => repomds.push((member, bytes)),
            Err(error) => {
                tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's repomd.xml cannot be read");
            }
        }
    }
    if repomds.is_empty() {
        return Ok(None);
    }
    let key = yum.keys.get(ctx)?;
    let wanted = merge_key(signing::signer(key.as_deref()), &repomds);
    let cached =
        |found: Option<Asset>| found.filter(|found| stored_key(found) == Some(wanted.as_str()));
    if let Some(found) = cached(store::find(ctx.db(), ctx.name(), &path).await?) {
        return Ok(Some(GroupRepomd::Merged(found)));
    }
    let lock = files::lock(ctx, base).await?;
    if let Some(found) = cached(store::find(ctx.db(), ctx.name(), &path).await?) {
        return Ok(Some(GroupRepomd::Merged(found)));
    }
    let merged = merge(yum, ctx, base, &wanted, repomds).await?;
    lock.commit().await?;
    Ok(Some(GroupRepomd::Merged(merged)))
}

async fn merge(
    yum: &Yum,
    ctx: &Ctx,
    base: &str,
    wanted: &str,
    repomds: Vec<(Ctx, Vec<u8>)>,
) -> Result<Asset, ContentError> {
    let mut members = Vec::with_capacity(repomds.len());
    for (member, bytes) in repomds {
        let repomd = match Repomd::parse(&bytes) {
            Ok(repomd) => repomd,
            Err(error) => {
                tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's repomd.xml is skipped");
                continue;
            }
        };
        let mut found_files = HashMap::new();
        for kind in MetadataType::ALL {
            let Some(data) = repomd.data(kind.name()) else {
                continue;
            };
            let path = paths::join(base, &data.location).ok_or_else(|| {
                ContentError::BadGateway(format!(
                    "{} lists metadata outside the repository: {}",
                    member.name(),
                    data.location
                ))
            })?;
            let found = yum.get(&member, &path).await?.ok_or_else(|| {
                ContentError::BadGateway(format!("{} has no {path}", member.name()))
            })?;
            let raw = files::download(&member, &found).await?;
            let file = tokio::task::spawn_blocking(move || merge::decompress(raw))
                .await
                .map_err(ContentError::internal)?
                .map_err(|error| {
                    ContentError::BadGateway(format!("{path} of {}: {error}", member.name()))
                })?;
            found_files.insert(kind, file);
        }
        members.push(MemberFiles {
            name: member.name().to_owned(),
            repomd,
            files: found_files,
        });
    }
    let names: Vec<String> = members.iter().map(|member| member.name.clone()).collect();
    let timestamp = chrono::Utc::now().timestamp();
    let merged = tokio::task::spawn_blocking(move || merge::merge(members, timestamp))
        .await
        .map_err(ContentError::internal)?
        .map_err(|error| ContentError::BadGateway(error.to_string()))?;
    let directory = paths::repodata_directory(base);
    let mut stored_files = Vec::with_capacity(merged.files.len());
    for file in merged.files {
        let name = file.file_name().to_owned();
        files::store_generated(
            ctx,
            &format!("{directory}/{name}"),
            REPODATA,
            GZIP_TYPE,
            file.file,
            json!({}),
        )
        .await?;
        stored_files.push(name);
    }
    let path = paths::repomd_path(base);
    let mut keep: HashSet<String> = stored_files.iter().cloned().collect();
    if let Some(previous) = store::find(ctx.db(), ctx.name(), &path).await?
        && let Some(list) = previous
            .attributes
            .get("yum")
            .and_then(|yum| yum.get("files"))
            .and_then(Value::as_array)
    {
        keep.extend(
            list.iter()
                .filter_map(|file| file.as_str().map(str::to_owned)),
        );
    }
    let attributes = json!({"yum": {
        "mergeKey": wanted,
        "members": names,
        "files": stored_files,
        "packageCount": merged.packages,
    }});
    let stored = store::put(
        ctx,
        files::metadata_put(ctx, &path, REPOMD, XML_TYPE, attributes),
        astrum_blob::content_of(merged.repomd.to_xml()),
    )
    .await?;
    for found in files::of_kind_under(ctx.db(), ctx.name(), REPODATA, &directory).await? {
        if paths::directory(&found.path) == directory
            && !keep.contains(paths::file_name(&found.path))
        {
            store::delete_asset(&ctx.content, found).await?;
        }
    }
    Ok(stored)
}

pub async fn metadata(
    yum: &Yum,
    ctx: &Ctx,
    path: &str,
) -> Result<Option<(Ctx, Asset)>, ContentError> {
    if let Some(found) = store::find(ctx.db(), ctx.name(), path).await? {
        return Ok(Some((ctx.clone(), found)));
    }
    group::first(ctx, |member| async move { yum.get(&member, path).await }).await
}
