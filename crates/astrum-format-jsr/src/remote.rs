use astrum_content::proxy::{self, Origin};
use astrum_content::store::PutAsset;
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::header;

use crate::documents::{MAX_LISTING, Versions};
use crate::hosted::{FILE_KIND, META_KIND, component_key};
use crate::names::{self, Document, Item};
use crate::reply::JSON_TYPE;
use crate::{blobs, documents};

pub const LISTING_KIND: &str = "listing";
pub const ACCEPT_JSON: &str = "application/json";
pub const ACCEPT_FILES: &str = "*/*";

pub fn plan(document: &Document) -> (Freshness, &'static str) {
    match document {
        Document::Listing => (Freshness::Metadata, ACCEPT_JSON),
        Document::VersionMeta(_) => (Freshness::Content, ACCEPT_JSON),
        Document::File(..) => (Freshness::Content, ACCEPT_FILES),
    }
}

pub async fn fetch(ctx: &Ctx, item: &Item) -> Result<Option<Asset>, ContentError> {
    let path = item.path();
    let (freshness, accept) = plan(&item.document);
    proxy::fetch_from(
        ctx,
        &path,
        freshness,
        || async {
            proxy::remote_url(&ctx.repository, &path, None)
                .map(|url| Some(Origin::new(url).header(header::ACCEPT, accept)))
        },
        || {
            let put = PutAsset::cached(path.clone());
            match &item.document {
                Document::Listing => put.kind(LISTING_KIND).content_type(JSON_TYPE),
                Document::VersionMeta(version) => put
                    .kind(META_KIND)
                    .content_type(JSON_TYPE)
                    .component(component_key(&item.package, version)),
                Document::File(version, file) => put
                    .kind(FILE_KIND)
                    .content_type(names::content_type(file))
                    .component(component_key(&item.package, version)),
            }
        },
    )
    .await
}

pub async fn listing(ctx: &Ctx, item: &Item) -> Result<Option<Versions>, ContentError> {
    let Some(found) = fetch(ctx, item).await? else {
        return Ok(None);
    };
    let body = blobs::read_all(ctx, &found, MAX_LISTING).await?;
    documents::parse_listing(&body).map(Some).map_err(|error| {
        ContentError::BadGateway(format!("the remote answered {}: {error}", found.path))
    })
}
