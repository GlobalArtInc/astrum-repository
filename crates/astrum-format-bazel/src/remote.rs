use astrum_content::proxy;
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};

use crate::blobs;
use crate::documents::{self, Document, MAX_DOCUMENT};
use crate::hosted::{JSON_TYPE, METADATA_KIND, REGISTRY_KIND, component_key};
use crate::names::{self, REGISTRY_PATH, VersionFile};

pub async fn file(ctx: &Ctx, file: &VersionFile) -> Result<Option<Asset>, ContentError> {
    let path = file.path();
    if let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? {
        return Ok(Some(found));
    }
    proxy::fetch(ctx, &path, None, Freshness::Content, || {
        PutAsset::cached(path.clone())
            .kind(file.entry.kind())
            .content_type(file.entry.content_type())
            .component(component_key(file))
    })
    .await
}

async fn document(
    ctx: &Ctx,
    path: &str,
    kind: &'static str,
) -> Result<Option<Asset>, ContentError> {
    proxy::fetch(ctx, path, None, Freshness::Metadata, || {
        PutAsset::cached(path).kind(kind).content_type(JSON_TYPE)
    })
    .await
}

pub async fn registry(ctx: &Ctx) -> Result<Option<Asset>, ContentError> {
    document(ctx, REGISTRY_PATH, REGISTRY_KIND).await
}

pub async fn metadata(ctx: &Ctx, module: &str) -> Result<Option<Asset>, ContentError> {
    document(ctx, &names::metadata_path(module), METADATA_KIND).await
}

pub async fn parsed(ctx: &Ctx, found: &Asset) -> Result<Document, ContentError> {
    let body = blobs::read_all(ctx, found, MAX_DOCUMENT).await?;
    documents::parse(&body, &found.path).map_err(|error| {
        ContentError::BadGateway(format!("the remote answered {}: {error}", found.path))
    })
}
