use std::collections::BTreeSet;

use astrum_blob::ContentStream;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness, group, proxy};
use astrum_repo::RepositoryType;
use bytes::Bytes;
use chrono::{DateTime, FixedOffset};
use futures_util::{StreamExt, TryStreamExt};
use serde_json::{Value, json};

use crate::packument;
use crate::path::Name;
use crate::{PACKUMENT, TARBALL, TARBALL_TYPE};

pub const MAX_DOCUMENT: usize = 256 * 1024 * 1024;

pub struct Loaded {
    pub doc: Value,
    pub modified: Option<DateTime<FixedOffset>>,
}

pub struct Stored {
    pub asset: Asset,
    pub doc: Value,
}

pub fn not_found(name: &Name) -> ContentError {
    ContentError::not_found(format!("package {name} not found"))
}

pub async fn read_json(ctx: &Ctx, asset: &Asset) -> Result<Value, ContentError> {
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let content = store.read(asset.blob_id.into()).await?.ok_or_else(|| {
        ContentError::internal(anyhow::anyhow!(
            "the blob of {}{} is missing",
            asset.repository,
            asset.path
        ))
    })?;
    let mut stream = content.stream;
    let mut collected = Vec::with_capacity(asset.size.max(0) as usize);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if collected.len() + chunk.len() > MAX_DOCUMENT {
            return Err(ContentError::internal(anyhow::anyhow!(
                "the package document {}{} is too large",
                asset.repository,
                asset.path
            )));
        }
        collected.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&collected).map_err(|error| {
        ContentError::BadGateway(format!(
            "the package document {} is not valid JSON: {error}",
            asset.path
        ))
    })
}

pub fn stream(data: Vec<u8>) -> ContentStream {
    futures_util::stream::once(async move { Ok(Bytes::from(data)) }).boxed()
}

pub fn packument_put(name: &Name, path: String) -> PutAsset {
    PutAsset::cached(path)
        .kind(PACKUMENT)
        .content_type(crate::reply::JSON)
        .attributes(json!({"npm": {"name": name.to_string()}}))
}

pub fn tarball_put(name: &Name, base: PutAsset, version: Option<String>) -> PutAsset {
    let put = base.kind(TARBALL).content_type(TARBALL_TYPE);
    match version {
        Some(version) => put
            .attributes(json!({"npm": {"name": name.to_string(), "version": version}}))
            .component(component(name, &version)),
        None => put.attributes(json!({"npm": {"name": name.to_string()}})),
    }
}

pub fn component(name: &Name, version: &str) -> ComponentKey {
    let attributes = match &name.scope {
        Some(scope) => json!({"npm": {"scope": scope}}),
        None => json!({"npm": {}}),
    };
    ComponentKey {
        namespace: name.scope.clone().unwrap_or_default(),
        name: name.bare.clone(),
        version: version.to_owned(),
        kind: String::new(),
        attributes,
    }
}

pub async fn hosted(ctx: &Ctx, name: &Name) -> Result<Option<Stored>, ContentError> {
    let Some(asset) = store::find(ctx.db(), ctx.name(), &name.packument_path()).await? else {
        return Ok(None);
    };
    let doc = read_json(ctx, &asset).await?;
    let present: BTreeSet<String> =
        store::under(ctx.db(), ctx.name(), &name.tarball_prefix(), 1_000_000)
            .await?
            .into_iter()
            .filter_map(|asset| asset.path.rsplit('/').next().map(str::to_owned))
            .collect();
    let gone: BTreeSet<String> = packument::versions(&doc)
        .into_iter()
        .flatten()
        .filter(|(version, manifest)| {
            let file =
                packument::tarball_file(manifest).unwrap_or_else(|| name.tarball_file(version));
            !present.contains(&file)
        })
        .map(|(version, _)| version.clone())
        .collect();
    let doc = packument::remove_versions(doc, &gone);
    Ok(Some(Stored { asset, doc }))
}

async fn leaf(ctx: &Ctx, name: &Name) -> Result<Option<Loaded>, ContentError> {
    match ctx.kind() {
        RepositoryType::Hosted => Ok(hosted(ctx, name).await?.and_then(|stored| {
            let empty = packument::versions(&stored.doc).is_none_or(|versions| versions.is_empty());
            (!empty).then_some(Loaded {
                doc: stored.doc,
                modified: Some(stored.asset.blob_updated_at),
            })
        })),
        RepositoryType::Proxy => {
            let path = name.packument_path();
            let found = proxy::fetch(ctx, &path, None, Freshness::Metadata, || {
                packument_put(name, path.clone())
            })
            .await?;
            match found {
                Some(asset) => Ok(Some(Loaded {
                    doc: read_json(ctx, &asset).await?,
                    modified: Some(asset.blob_updated_at),
                })),
                None => Ok(None),
            }
        }
        RepositoryType::Group => Ok(None),
    }
}

pub async fn load(ctx: &Ctx, name: &Name) -> Result<Option<Loaded>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return leaf(ctx, name).await;
    }
    let members = group::members(ctx).await?;
    let loads = members.into_iter().map(|member| {
        let member_ctx = ctx.member(member);
        async move {
            let found = leaf(&member_ctx, name).await;
            (member_ctx, found)
        }
    });
    let mut docs = Vec::new();
    let mut modified: Option<DateTime<FixedOffset>> = None;
    for (member, found) in futures_util::future::join_all(loads).await {
        match found {
            Ok(Some(loaded)) => {
                modified = modified.max(loaded.modified);
                docs.push(loaded.doc);
            }
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; using the others");
            }
        }
    }
    Ok(packument::merge(docs).map(|doc| Loaded { doc, modified }))
}

pub async fn save(ctx: &Ctx, name: &Name, doc: &Value) -> Result<Asset, ContentError> {
    let bytes = serde_json::to_vec(doc).map_err(ContentError::internal)?;
    let put =
        packument_put(name, name.packument_path()).attributes(packument::search_attributes(doc));
    let put = PutAsset {
        verified: false,
        ..put
    };
    store::put(ctx, put, stream(bytes)).await
}
