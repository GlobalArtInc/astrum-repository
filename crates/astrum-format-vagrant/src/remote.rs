use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::header;
use futures_util::TryStreamExt;
use serde_json::{Value, json};

use crate::catalog::{Catalog, Provider};
use crate::hosted::{self, ARCHIVE_TYPES, asset_attributes, key};
use crate::names::{BoxFile, BoxName};
use crate::{BOX, JSON_TYPE, METADATA};

const MAX_DOCUMENT: u64 = 20 * 1024 * 1024;

async fn read_document(ctx: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > MAX_DOCUMENT {
        return Err(ContentError::BadGateway(
            "the remote answered box metadata larger than 20 MiB".to_owned(),
        ));
    }
    let blob_store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = blob_store.read(found.blob_id.into()).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob of the repository is missing",
        )));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn catalog(ctx: &Ctx, boxed: &BoxName) -> Result<Option<Catalog>, ContentError> {
    let path = boxed.metadata_path();
    let address = proxy::remote_url(&ctx.repository, &boxed.path(), None)?;
    let origin = address.clone();
    let fetched = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Metadata,
        || async move {
            Ok(Some(
                Origin::new(origin).header(header::ACCEPT, JSON_TYPE.to_owned()),
            ))
        },
        || {
            PutAsset::cached(path.clone())
                .kind(METADATA)
                .content_type(JSON_TYPE)
        },
    )
    .await?;
    let Some(found) = fetched else {
        return Ok(None);
    };
    let body = read_document(ctx, &found).await?;
    match serde_json::from_slice::<Value>(&body) {
        Ok(document) => {
            let parsed = Catalog::parse(boxed.clone(), &document, Some(&address));
            Ok((!parsed.is_empty()).then_some(parsed))
        }
        Err(error) => {
            tracing::info!(repository = ctx.name(), path, %error, "the remote answered box metadata that is not JSON");
            store::delete(ctx, &path).await?;
            Err(ContentError::BadGateway(format!(
                "the remote answer for {} is not JSON",
                boxed.tag()
            )))
        }
    }
}

fn verified(stored: &Asset, listed: &Provider) -> Option<bool> {
    let expected = listed.checksum.as_deref()?;
    let found = match listed.checksum_type.as_deref()? {
        "sha256" => &stored.sha256,
        "sha1" => &stored.sha1,
        "sha512" => &stored.sha512,
        "md5" => &stored.md5,
        _ => return None,
    };
    Some(found.eq_ignore_ascii_case(expected))
}

pub async fn box_file(ctx: &Ctx, file: &BoxFile) -> Result<Option<Asset>, ContentError> {
    let path = file.path();
    let listed = catalog(ctx, &file.boxed)
        .await?
        .and_then(|found| found.find(file).cloned());
    let existing = store::find(ctx.db(), ctx.name(), &path).await?;
    let Some(listed) = listed else {
        return Ok(existing);
    };
    if let Some(existing) = &existing
        && verified(existing, &listed) == Some(true)
    {
        return Ok(Some(existing.clone()));
    }
    let Some(address) = listed
        .url
        .as_deref()
        .and_then(|url| url::Url::parse(url).ok())
    else {
        return Ok(existing);
    };
    let fetched = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async move { Ok(Some(Origin::new(address))) },
        || {
            PutAsset::cached(path.clone())
                .kind(BOX)
                .attributes(asset_attributes(file))
                .component(key(file, json!({})))
        },
    )
    .await?;
    let Some(stored) = fetched else {
        return Ok(None);
    };
    if verified(&stored, &listed) == Some(false) {
        tracing::warn!(
            repository = ctx.name(),
            path,
            expected = listed.checksum,
            "a fetched box does not match the checksum of its metadata"
        );
        store::delete(ctx, &path).await?;
        return Err(ContentError::BadGateway(format!(
            "the {} box of {} {} for {} from the remote does not match the {} checksum of its metadata",
            file.provider,
            file.boxed.tag(),
            file.version,
            file.architecture,
            listed.checksum_type.as_deref().unwrap_or_default()
        )));
    }
    if !ARCHIVE_TYPES.contains(&stored.content_type.as_str())
        && let Some(detected) =
            hosted::sniff(ctx, &stored.blob_store, stored.blob_id.into()).await?
    {
        hosted::set_content_type(ctx, stored.id, &detected).await?;
        return store::find(ctx.db(), ctx.name(), &path).await;
    }
    Ok(Some(stored))
}
