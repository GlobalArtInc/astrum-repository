use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, View};
use astrum_repo::config::WritePolicy;
use axum::http::StatusCode;
use axum::response::Response;
use bytes::Bytes;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::names::{self, listing_path, narinfo_path};
use crate::narinfo::NarInfo;
use crate::{
    File, ID, MAX_DOCUMENT, MAX_NARINFO, NAR, NAR_TYPE, NARINFO, NARINFO_TYPE, Route, db, status,
    store_dir,
};

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

async fn authorize(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let existing = store::find(ctx.db(), ctx.name(), path).await?;
    let view = match existing.is_some() && write_policy(ctx) == WritePolicy::Allow {
        true => View::Edit,
        false => View::Add,
    };
    ctx.require(view, path)?;
    if write_policy(ctx) == WritePolicy::Deny {
        return Err(ContentError::BadRequest(format!(
            "the repository {} is read-only",
            ctx.name()
        )));
    }
    Ok(existing)
}

fn encoding(request: &ContentRequest) -> Option<String> {
    request
        .header("content-encoding")
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty() && value != "identity")
}

fn whole(body: Bytes) -> ContentStream {
    Box::pin(futures_util::stream::once(async move {
        Ok::<_, std::io::Error>(body)
    }))
}

fn nar_of(found: &Asset) -> Option<String> {
    found
        .attributes
        .pointer("/nix/nar")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

pub fn key(info: &NarInfo) -> ComponentKey {
    let (name, version) = names::package(&info.name);
    ComponentKey {
        namespace: info.hash.clone(),
        name: name.to_owned(),
        version: version.to_owned(),
        kind: String::new(),
        attributes: info.component_attributes(),
    }
}

pub async fn narinfo(ctx: &Ctx, hash: &str) -> Result<Option<Asset>, ContentError> {
    let Some(found) = store::find(ctx.db(), ctx.name(), &narinfo_path(hash)).await? else {
        return Ok(None);
    };
    if let Some(nar) = nar_of(&found)
        && store::find(ctx.db(), ctx.name(), &nar).await?.is_none()
    {
        tracing::info!(
            repository = ctx.name(),
            hash,
            nar,
            "a narinfo whose NAR file is gone is not served"
        );
        return Ok(None);
    }
    Ok(Some(found))
}

pub async fn settle(ctx: &Ctx, nar_path: &str) -> Result<(), ContentError> {
    let Some(nar) = store::find(ctx.db(), ctx.name(), nar_path).await? else {
        return Ok(());
    };
    let referrers = db::referrers(ctx, nar_path).await?;
    let owner = match referrers.as_slice() {
        [] => return Ok(()),
        [only] => only.component_id,
        _ => None,
    };
    db::place(ctx, &nar, owner, None).await
}

async fn release(ctx: &Ctx, nar_path: &str) -> Result<(), ContentError> {
    if !db::referrers(ctx, nar_path).await?.is_empty() {
        return settle(ctx, nar_path).await;
    }
    if let Some(nar) = store::find(ctx.db(), ctx.name(), nar_path).await? {
        store::delete_asset(&ctx.content, nar).await?;
    }
    Ok(())
}

pub async fn put(
    ctx: &Ctx,
    request: ContentRequest,
    route: Route,
) -> Result<Response, ContentError> {
    match route {
        Route::CacheInfo => put_cache_info(ctx, request).await,
        Route::NarInfo(hash) => put_narinfo(ctx, request, &hash).await,
        Route::File(File::Nar) => put_nar(ctx, request).await,
        Route::File(file) => put_file(ctx, request, file).await,
        Route::Other => Err(ContentError::BadRequest(format!(
            "{} is not a file of a Nix binary cache; nix copy writes narinfo, nar/, .ls, log/ and realisations/ files",
            request.path
        ))),
    }
}

async fn put_cache_info(ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
    match ctx.grant(View::Add).any() {
        true => {}
        false => ctx.require(View::Add, &request.path)?,
    }
    let body = request.bytes(MAX_DOCUMENT).await?;
    let text = String::from_utf8_lossy(&body);
    let wanted = store_dir(ctx);
    let given = text
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.trim() == "StoreDir")
        .map(|(_, value)| value.trim().to_owned());
    match given {
        Some(given) if given != wanted => Err(ContentError::BadRequest(format!(
            "{} is a binary cache for the store {wanted}, not {given}",
            ctx.name()
        ))),
        _ => Ok(status(StatusCode::NO_CONTENT)),
    }
}

async fn put_narinfo(
    ctx: &Ctx,
    request: ContentRequest,
    hash: &str,
) -> Result<Response, ContentError> {
    let path = narinfo_path(hash);
    let previous = authorize(ctx, &path).await?;
    if let Some(encoding) = encoding(&request) {
        return Err(ContentError::BadRequest(format!(
            "the narinfo arrived with Content-Encoding {encoding}; upload narinfo files uncompressed (leave narinfo-compression unset)"
        )));
    }
    let body = request.bytes(MAX_NARINFO).await?;
    if previous
        .as_ref()
        .is_some_and(|previous| previous.sha256 == hex::encode(Sha256::digest(&body)))
    {
        return Ok(status(StatusCode::NO_CONTENT));
    }
    let info = NarInfo::parse(&body, &store_dir(ctx))
        .map_err(|error| ContentError::BadRequest(format!("the narinfo is not valid: {error}")))?;
    if info.hash != hash {
        return Err(ContentError::BadRequest(format!(
            "{} is not the store path of {hash}.narinfo",
            info.store_path
        )));
    }
    let nar_path = info.nar_path().ok_or_else(|| {
        ContentError::BadRequest(format!(
            "URL {} must name a NAR file of this repository, such as nar/<hash>.nar.xz",
            info.url
        ))
    })?;
    let nar = store::find(ctx.db(), ctx.name(), &nar_path)
        .await?
        .ok_or_else(|| {
            ContentError::BadRequest(format!(
                "{} is not in {}; upload the NAR file before its narinfo",
                info.url,
                ctx.name()
            ))
        })?;
    if let Some(size) = info.file_size
        && size != nar.size as u64
    {
        return Err(ContentError::BadRequest(format!(
            "FileSize is {size}, but {} has {} bytes",
            info.url, nar.size
        )));
    }
    if let Some(file_hash) = info.file_hash.as_ref().filter(|hash| hash.is_sha256())
        && file_hash.hex() != nar.sha256
    {
        return Err(ContentError::BadRequest(format!(
            "FileHash {} is not the SHA-256 of {}",
            file_hash.nix32(),
            info.url
        )));
    }
    let put = PutAsset::hosted(ctx, &path)
        .kind(NARINFO)
        .content_type(NARINFO_TYPE)
        .component(key(&info))
        .attributes(info.asset_attributes());
    let stored = store::put(ctx, put, whole(body)).await?;
    if let Some(old) = previous.as_ref().and_then(nar_of)
        && old != nar_path
    {
        release(ctx, &old).await?;
    }
    settle(ctx, &nar_path).await?;
    if let Some(listing) = store::find(ctx.db(), ctx.name(), &listing_path(hash)).await? {
        db::place(ctx, &listing, stored.component_id, None).await?;
    }
    Ok(status(StatusCode::CREATED))
}

async fn put_nar(ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
    let path = request.path.clone();
    let existing = authorize(ctx, &path).await?;
    if let Some(encoding) = encoding(&request) {
        return Err(ContentError::BadRequest(format!(
            "the NAR file arrived with Content-Encoding {encoding}; its compression belongs in the file name"
        )));
    }
    let written = store::write_blob(ctx, request.into_stream()).await?;
    if unchanged(ctx, existing.as_ref(), &written) {
        return Ok(status(StatusCode::NO_CONTENT));
    }
    let put = PutAsset::hosted(ctx, &path)
        .kind(NAR)
        .content_type(NAR_TYPE);
    store::adopt(ctx, put, &written).await?;
    settle(ctx, &path).await?;
    Ok(status(StatusCode::CREATED))
}

fn unchanged(ctx: &Ctx, existing: Option<&Asset>, written: &Written) -> bool {
    let same = existing.is_some_and(|existing| {
        existing.sha256 == written.digests.sha256 && existing.size == written.size as i64
    });
    if same {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    }
    same
}

async fn put_file(
    ctx: &Ctx,
    request: ContentRequest,
    file: File,
) -> Result<Response, ContentError> {
    let path = request.path.clone();
    let existing = authorize(ctx, &path).await?;
    let encoding = encoding(&request);
    let attributes = match &encoding {
        Some(encoding) => json!({ ID: {"contentEncoding": encoding} }),
        None => json!({}),
    };
    let put = PutAsset::hosted(ctx, &path)
        .kind(file.kind())
        .content_type(file.content_type())
        .attributes(attributes);
    let body = match file.document() && encoding.is_none() {
        true => {
            let body = request.bytes(MAX_DOCUMENT).await?;
            if !serde_json::from_slice::<Value>(&body).is_ok_and(|document| document.is_object()) {
                return Err(ContentError::BadRequest(format!(
                    "{path} must be a JSON object"
                )));
            }
            whole(body)
        }
        false => request.into_stream(),
    };
    let written = store::write_blob(ctx, body).await?;
    if unchanged(ctx, existing.as_ref(), &written) {
        return Ok(status(StatusCode::NO_CONTENT));
    }
    let stored = store::adopt(ctx, put, &written).await?;
    if file == File::Listing {
        attach_listing(ctx, &stored).await?;
    }
    Ok(status(StatusCode::CREATED))
}

pub async fn attach_listing(ctx: &Ctx, listing: &Asset) -> Result<(), ContentError> {
    let Some(hash) = listing
        .path
        .strip_prefix('/')
        .and_then(|name| name.strip_suffix(".ls"))
    else {
        return Ok(());
    };
    let owner = store::find(ctx.db(), ctx.name(), &narinfo_path(hash))
        .await?
        .and_then(|found| found.component_id);
    match owner {
        Some(_) => db::place(ctx, listing, owner, None).await,
        None => Ok(()),
    }
}

pub async fn delete_narinfo(ctx: &Ctx, hash: &str) -> Result<bool, ContentError> {
    let Some(found) = store::find(ctx.db(), ctx.name(), &narinfo_path(hash)).await? else {
        return Ok(false);
    };
    let listing = store::find(ctx.db(), ctx.name(), &listing_path(hash)).await?;
    if let Some(listing) = &listing {
        ctx.require(View::Delete, &listing.path)?;
    }
    let nar = nar_of(&found);
    store::delete_asset(&ctx.content, found).await?;
    if let Some(listing) = listing {
        store::delete_asset(&ctx.content, listing).await?;
    }
    if let Some(nar) = nar {
        release(ctx, &nar).await?;
    }
    Ok(true)
}
