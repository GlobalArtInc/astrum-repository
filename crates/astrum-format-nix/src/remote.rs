use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness, proxy};
use astrum_repo::RepositoryType;
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::names::{listing_path, narinfo_path};
use crate::narinfo::{self, NarInfo};
use crate::{File, ID, MAX_NARINFO, NARINFO, NARINFO_TYPE, db, hosted, store_dir};

fn remote_base(ctx: &Ctx) -> Option<String> {
    ctx.repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .map(|url| url.trim_end_matches('/').to_owned())
}

fn under_remote(ctx: &Ctx, url: &str) -> Option<String> {
    let base = remote_base(ctx)?;
    url.strip_prefix(&format!("{base}/"))
        .filter(|rest| !rest.is_empty())
        .map(str::to_owned)
}

pub async fn narinfo(ctx: &Ctx, hash: &str) -> Result<Option<Asset>, ContentError> {
    let path = narinfo_path(hash);
    let Some(found) = proxy::fetch(ctx, &path, None, Freshness::Metadata, || {
        PutAsset::cached(&path)
            .kind(NARINFO)
            .content_type(NARINFO_TYPE)
    })
    .await?
    else {
        return Ok(None);
    };
    if found.component_id.is_some() && found.attributes.get(ID).is_some() {
        return Ok(Some(found));
    }
    let bytes = db::read(ctx, &found, MAX_NARINFO).await?;
    let parsed = NarInfo::parse(&bytes, &store_dir(ctx)).and_then(|info| match info.hash == hash {
        true => Ok(info),
        false => Err(format!("it describes {}", info.store_path)),
    });
    let mut info = match parsed {
        Ok(info) => info,
        Err(error) => {
            store::delete_asset(&ctx.content, found).await?;
            return Err(ContentError::BadGateway(format!(
                "the remote's {hash}.narinfo is not valid: {error}"
            )));
        }
    };
    let rewritten = match under_remote(ctx, &info.url) {
        Some(relative) => {
            info.url = relative;
            true
        }
        None => false,
    };
    let mut attributes = info.asset_attributes();
    if rewritten {
        attributes[ID]["rewritten"] = json!(true);
    }
    let component = db::upsert_component(ctx, &hosted::key(&info)).await?;
    db::place(ctx, &found, Some(component), Some(attributes.clone())).await?;
    if let Some(nar) = info.nar_path() {
        hosted::settle(ctx, &nar).await?;
    }
    if let Some(listing) = store::find(ctx.db(), ctx.name(), &listing_path(hash)).await? {
        db::place(ctx, &listing, Some(component), None).await?;
    }
    Ok(Some(Asset {
        component_id: Some(component),
        attributes,
        ..found
    }))
}

pub async fn rewritten(ctx: &Ctx, found: &Asset) -> Result<Option<Vec<u8>>, ContentError> {
    let flagged = found
        .attributes
        .pointer("/nix/rewritten")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if ctx.kind() != RepositoryType::Proxy || !flagged {
        return Ok(None);
    }
    let Some(base) = remote_base(ctx) else {
        return Ok(None);
    };
    let bytes = db::read(ctx, found, MAX_NARINFO).await?;
    Ok(Some(narinfo::rewrite_url(&bytes, &base).unwrap_or(bytes)))
}

pub async fn file(ctx: &Ctx, path: &str, file: File) -> Result<Option<Asset>, ContentError> {
    if let Some(found) = store::find(ctx.db(), ctx.name(), path).await?
        && matches!(file, File::Nar | File::Listing | File::Log)
    {
        return Ok(Some(found));
    }
    match file {
        File::Nar => {
            let fetched = proxy::fetch(ctx, path, None, Freshness::Content, || {
                PutAsset::cached(path)
                    .kind(file.kind())
                    .content_type(file.content_type())
            })
            .await?;
            if fetched.is_some() {
                hosted::settle(ctx, path).await?;
            }
            Ok(fetched)
        }
        File::Listing | File::Log => encoded(ctx, path, file).await,
        File::Realisation | File::DebugInfo => {
            proxy::fetch(ctx, path, None, Freshness::Content, || {
                PutAsset::cached(path)
                    .kind(file.kind())
                    .content_type(file.content_type())
            })
            .await
        }
    }
}

async fn encoded(ctx: &Ctx, path: &str, file: File) -> Result<Option<Asset>, ContentError> {
    let url = proxy::remote_url(&ctx.repository, path, None)?;
    let remote = proxy::request(ctx, url, &[]).await?;
    match remote.status {
        StatusCode::OK => {
            let encoding = remote
                .header("content-encoding")
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty() && value != "identity");
            let content_type = remote
                .header("content-type")
                .and_then(|value| value.split(';').next())
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| file.content_type().to_owned());
            let mut put = PutAsset::cached(path)
                .kind(file.kind())
                .content_type(content_type);
            if let Some(encoding) = encoding {
                put = put.attributes(json!({ ID: {"contentEncoding": encoding} }));
            }
            let stored = store::put(ctx, put, remote.body).await?;
            if file == File::Listing {
                hosted::attach_listing(ctx, &stored).await?;
            }
            Ok(Some(stored))
        }
        StatusCode::NOT_FOUND | StatusCode::GONE => Ok(None),
        status => Err(ContentError::BadGateway(format!(
            "the remote answered {status} for {path}"
        ))),
    }
}
