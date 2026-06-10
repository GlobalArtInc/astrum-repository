use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::future::Future;

use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, group};
use astrum_repo::config::WritePolicy;
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseTransaction, Statement, TransactionTrait};
use serde_json::{Value, json};

use crate::index::{self, Specs, VersionsLine};
use crate::{BINARY, ID, INDEX, Index, Route, Rubygems, TEXT};

const MERGE_FORMAT: u32 = 1;

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

async fn answers(
    rubygems: &Rubygems,
    ctx: &Ctx,
    route: &Route,
) -> Result<Vec<(Ctx, Index)>, ContentError> {
    let mut found = Vec::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match Box::pin(rubygems.index(&member, route)).await {
            Ok(Some(answer)) => found.push((member, answer)),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no index");
            }
        }
    }
    Ok(found)
}

pub async fn index(
    rubygems: &Rubygems,
    ctx: &Ctx,
    route: &Route,
) -> Result<Option<Index>, ContentError> {
    match route {
        Route::Versions => versions(rubygems, ctx).await,
        Route::Names => names(rubygems, ctx).await,
        Route::Info(_) => info(rubygems, ctx, route).await,
        Route::Specs(kind) => specs(rubygems, ctx, *kind).await,
        _ => Ok(None),
    }
}

async fn names(rubygems: &Rubygems, ctx: &Ctx) -> Result<Option<Index>, ContentError> {
    let found = answers(rubygems, ctx, &Route::Names).await?;
    if found.is_empty() {
        return Ok(None);
    }
    let mut names = BTreeSet::new();
    for (_, answer) in found {
        let text = lossy(&answer.bytes().await?);
        names.extend(
            index::body_lines(&text)
                .into_iter()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned),
        );
    }
    Ok(Some(Index::Built(index::names(names.iter()))))
}

async fn info(
    rubygems: &Rubygems,
    ctx: &Ctx,
    route: &Route,
) -> Result<Option<Index>, ContentError> {
    let found = answers(rubygems, ctx, route).await?;
    if found.is_empty() {
        return Ok(None);
    }
    let mut infos = Vec::with_capacity(found.len());
    for (_, answer) in found {
        infos.push(lossy(&answer.bytes().await?));
    }
    Ok(Some(Index::Built(index::merge_info(&infos))))
}

pub async fn versions(rubygems: &Rubygems, ctx: &Ctx) -> Result<Option<Index>, ContentError> {
    let mut found = answers(rubygems, ctx, &Route::Versions).await?;
    if found.len() <= 1 {
        return Ok(found.pop().map(|(_, answer)| answer));
    }
    let signature = format!(
        "{MERGE_FORMAT}:{}",
        found
            .iter()
            .map(|(member, answer)| format!("{}={}", member.name(), answer.md5()))
            .collect::<Vec<_>>()
            .join(",")
    );
    cached_or_built(ctx, &Route::Versions.path(), &signature, TEXT, || {
        merge_versions(rubygems, ctx, found)
    })
    .await
    .map(Some)
}

async fn merge_versions(
    rubygems: &Rubygems,
    ctx: &Ctx,
    found: Vec<(Ctx, Index)>,
) -> Result<Vec<u8>, ContentError> {
    let mut texts = Vec::with_capacity(found.len());
    for (member, answer) in found {
        texts.push((member, lossy(&answer.bytes().await?)));
    }
    let mut seen = HashSet::new();
    let mut overlapping = BTreeSet::new();
    for (_, text) in &texts {
        let names: HashSet<String> = index::body_lines(text)
            .into_iter()
            .filter_map(VersionsLine::parse)
            .map(|line| line.name)
            .collect();
        for name in names {
            if !seen.insert(name.clone()) {
                overlapping.insert(name);
            }
        }
    }
    let mut out = index::header(Utc::now());
    for (_, text) in &texts {
        for line in index::body_lines(text)
            .into_iter()
            .filter_map(VersionsLine::parse)
            .filter(|line| !overlapping.contains(&line.name))
        {
            out.push_str(&line.line());
            out.push('\n');
        }
    }
    for name in &overlapping {
        let route = Route::Info(name.clone());
        let mut infos = Vec::new();
        for (member, _) in &texts {
            match Box::pin(rubygems.index(member, &route)).await {
                Ok(Some(answer)) => infos.push(lossy(&answer.bytes().await?)),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), gem = name, %error, "a member's gem information is skipped");
                }
            }
        }
        if infos.is_empty() {
            continue;
        }
        let merged = index::merge_info(&infos);
        let versions = index::versions_of(&index::parse_info(&lossy(&merged)));
        if versions.is_empty() {
            continue;
        }
        let line = VersionsLine {
            name: name.clone(),
            versions,
            checksum: index::md5_hex(&merged),
        };
        out.push_str(&line.line());
        out.push('\n');
    }
    tracing::debug!(
        group = ctx.name(),
        members = texts.len(),
        overlapping = overlapping.len(),
        "merged the versions of a group"
    );
    Ok(out.into_bytes())
}

pub fn collect(text: &str) -> BTreeMap<String, Vec<String>> {
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in index::body_lines(text)
        .into_iter()
        .filter_map(VersionsLine::parse)
    {
        for version in line.versions {
            match version.strip_prefix('-') {
                Some(yanked) => {
                    if let Some(list) = by_name.get_mut(&line.name) {
                        list.retain(|known| known != yanked);
                        if list.is_empty() {
                            by_name.remove(&line.name);
                        }
                    }
                }
                None if !version.is_empty() => {
                    by_name.entry(line.name.clone()).or_default().push(version);
                }
                None => {}
            }
        }
    }
    by_name
}

async fn specs(rubygems: &Rubygems, ctx: &Ctx, kind: Specs) -> Result<Option<Index>, ContentError> {
    let Some(versions) = versions(rubygems, ctx).await? else {
        return Ok(None);
    };
    let signature = format!("{MERGE_FORMAT}:{}", versions.md5());
    cached_or_built(ctx, kind.path(), &signature, BINARY, || async move {
        let text = lossy(&versions.bytes().await?);
        tokio::task::spawn_blocking(move || index::legacy_specs(kind, &collect(&text)))
            .await
            .map_err(ContentError::internal)
    })
    .await
    .map(Some)
}

fn signature_of(found: &Asset) -> Option<&str> {
    found
        .attributes
        .get(ID)
        .and_then(|attributes| attributes.get("signature"))
        .and_then(Value::as_str)
}

async fn cached(ctx: &Ctx, path: &str, signature: &str) -> Result<Option<Asset>, ContentError> {
    Ok(store::find(ctx.db(), ctx.name(), path)
        .await?
        .filter(|found| signature_of(found) == Some(signature)))
}

async fn lock(ctx: &Ctx, path: &str) -> Result<DatabaseTransaction, ContentError> {
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("rubygems:{}:{path}", ctx.name()).into()],
    ))
    .await?;
    Ok(txn)
}

async fn cached_or_built<F, Fut>(
    ctx: &Ctx,
    path: &str,
    signature: &str,
    content_type: &str,
    build: F,
) -> Result<Index, ContentError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Vec<u8>, ContentError>>,
{
    let stored = |found: Asset| Index::Stored(Box::new((ctx.clone(), found)));
    if let Some(found) = cached(ctx, path, signature).await? {
        return Ok(stored(found));
    }
    let held = lock(ctx, path).await?;
    if let Some(found) = cached(ctx, path, signature).await? {
        return Ok(stored(found));
    }
    let bytes = build().await?;
    let mut put = PutAsset::hosted(ctx, path)
        .kind(INDEX)
        .content_type(content_type)
        .attributes(json!({ ID: { "signature": signature } }));
    put.check_content_type = false;
    put.write_policy = Some(WritePolicy::Allow);
    let found = store::put(ctx, put, astrum_blob::content_of(bytes)).await?;
    held.commit().await?;
    Ok(stored(found))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collected_versions_drop_yanked_ones() {
        let map = collect(
            "created_at: x\n---\nhola 1.0.0,1.1.0 abc\nhola -1.0.0 def\nold 1.0 x\nold -1.0 y\n",
        );
        assert_eq!(map["hola"], ["1.1.0"]);
        assert!(!map.contains_key("old"));
    }
}
