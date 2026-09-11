use std::sync::{Arc, Mutex};

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::header;
use serde_json::Value;

use crate::hosted::{self, asset_attributes, key};
use crate::listing::{Entry, Listing, archive_path};
use crate::{ACCEPT_V2, ARCHIVE, ARCHIVE_TYPE, METADATA, V2_TYPE};

const MAX_DOCUMENT: u64 = 128 * 1024 * 1024;

pub fn listing_path(name: &str) -> String {
    format!("/api/packages/{name}")
}

pub fn advisories_path(name: &str) -> String {
    format!("/api/packages/{name}/advisories")
}

pub const COMPLETION_PATH: &str = "/api/package-name-completion-data";

async fn metadata(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let owned = path.to_owned();
    proxy::fetch_from(
        ctx,
        path,
        Freshness::Metadata,
        || async {
            let url = proxy::remote_url(&ctx.repository, path, None)?;
            Ok(Some(
                Origin::new(url).header(header::ACCEPT, ACCEPT_V2.to_owned()),
            ))
        },
        || PutAsset::cached(owned).kind(METADATA).content_type(V2_TYPE),
    )
    .await
}

pub async fn document(ctx: &Ctx, path: &str) -> Result<Option<Value>, ContentError> {
    let Some(found) = metadata(ctx, path).await? else {
        return Ok(None);
    };
    let body = hosted::read_blob(ctx, &found.blob_store, found.blob_id, MAX_DOCUMENT).await?;
    match serde_json::from_slice::<Value>(&body) {
        Ok(value) => Ok(Some(value)),
        Err(error) => {
            tracing::info!(repository = ctx.name(), path, %error, "the remote answered a document that is not JSON");
            store::delete(ctx, path).await?;
            Err(ContentError::BadGateway(format!(
                "the remote answer for {path} is not JSON"
            )))
        }
    }
}

pub async fn listing(ctx: &Ctx, name: &str) -> Result<Option<Listing>, ContentError> {
    Ok(document(ctx, &listing_path(name))
        .await?
        .map(|value| Listing::parse(name, &value))
        .filter(|listing| !listing.versions.is_empty()))
}

pub async fn advisories(ctx: &Ctx, name: &str) -> Result<Option<Value>, ContentError> {
    document(ctx, &advisories_path(name)).await
}

pub async fn completion(ctx: &Ctx) -> Result<Option<Value>, ContentError> {
    document(ctx, COMPLETION_PATH).await
}

fn origin_of(
    ctx: &Ctx,
    name: &str,
    version: &str,
    entry: Option<&Entry>,
) -> Result<url::Url, ContentError> {
    let listed = entry
        .and_then(|entry| entry.archive_url.as_deref())
        .and_then(|address| url::Url::parse(address).ok())
        .filter(|address| matches!(address.scheme(), "http" | "https"));
    match listed {
        Some(address) => Ok(address),
        None => proxy::remote_url(&ctx.repository, &archive_path(name, version), None),
    }
}

pub async fn archive(ctx: &Ctx, name: &str, version: &str) -> Result<Option<Asset>, ContentError> {
    let path = archive_path(name, version);
    let found: Arc<Mutex<Option<Entry>>> = Arc::new(Mutex::new(None));
    let remembered = found.clone();
    let fetched = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async move {
            let listed = listing(ctx, name).await?;
            let entry = listed
                .as_ref()
                .and_then(|listing| listing.find(version))
                .cloned();
            let url = origin_of(ctx, name, version, entry.as_ref())?;
            *remembered.lock().expect("the entry lock is never poisoned") = entry;
            Ok(Some(Origin::new(url)))
        },
        || {
            let entry = found
                .lock()
                .expect("the entry lock is never poisoned")
                .clone()
                .unwrap_or_default();
            let pubspec = match entry.pubspec {
                Value::Object(_) => entry.pubspec.clone(),
                _ => serde_json::json!({"name": name, "version": version}),
            };
            let attributes =
                hosted::component_attributes(&pubspec, entry.retracted, &hosted::Flags::default());
            PutAsset::cached(path.clone())
                .kind(ARCHIVE)
                .content_type(ARCHIVE_TYPE)
                .attributes(asset_attributes(name, version, &pubspec))
                .component(key(name, version, attributes))
        },
    )
    .await?;
    let Some(stored) = fetched else {
        return Ok(None);
    };
    let expected = match found
        .lock()
        .expect("the entry lock is never poisoned")
        .clone()
    {
        Some(entry) => entry.sha256,
        None => None,
    };
    if let Some(expected) = expected
        && !stored.sha256.eq_ignore_ascii_case(&expected)
    {
        tracing::warn!(
            repository = ctx.name(),
            path,
            expected,
            found = stored.sha256,
            "a fetched package archive does not match the checksum of its listing"
        );
        store::delete(ctx, &path).await?;
        return Err(ContentError::BadGateway(format!(
            "the archive of {name} {version} from the remote does not match the archive_sha256 of its listing"
        )));
    }
    Ok(Some(stored))
}
