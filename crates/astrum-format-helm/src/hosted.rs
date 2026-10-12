use std::collections::HashMap;

use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use chrono::Utc;
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;

use crate::chart::{self, Chart};
use crate::index::{self, Listing};
use crate::provenance;
use crate::{CHART, CHART_TYPE, ID, PROVENANCE, PROVENANCE_SUFFIX, PROVENANCE_TYPE};

const MAX_PREFIX: u64 = 64 * 1024 * 1024;

pub fn key(chart: &Chart) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: chart.name.clone(),
        version: chart.version.clone(),
        kind: String::new(),
        attributes: chart.attributes(),
    }
}

pub async fn listing(ctx: &Ctx) -> Result<Listing, ContentError> {
    let components: HashMap<i64, Chart> = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .all(ctx.db())
        .await?
        .into_iter()
        .filter_map(|found| Some((found.id, Chart::from_attributes(&found.attributes)?)))
        .collect();
    let charts = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(CHART))
        .all(ctx.db())
        .await?;
    let mut listing = Listing::default();
    for found in charts {
        let Some(mut chart) = found
            .component_id
            .and_then(|id| components.get(&id))
            .cloned()
        else {
            continue;
        };
        let file = found.path.trim_start_matches('/').to_owned();
        if chart.file_name() != file {
            continue;
        }
        let updated = found.blob_updated_at.with_timezone(&Utc);
        listing.generated = listing.generated.max(Some(updated));
        chart.created = Some(index::timestamp(found.created_at.with_timezone(&Utc)));
        chart.digest = Some(found.sha256.clone());
        chart.urls = vec![file];
        listing
            .entries
            .entry(chart.name.clone())
            .or_default()
            .push(chart);
    }
    index::sort(&mut listing.entries);
    Ok(listing)
}

pub async fn authorize(ctx: &Ctx, path: &str) -> Result<(), ContentError> {
    let redeploys = ctx
        .repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
        == WritePolicy::Allow;
    let exists = store::find(ctx.db(), ctx.name(), path).await?.is_some();
    ctx.require(
        if exists && redeploys {
            View::Edit
        } else {
            View::Add
        },
        path,
    )
}

async fn prefix(ctx: &Ctx, written: &Written) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let length = written.size.min(MAX_PREFIX);
    let Some(content) = store.read_range(written.id, 0, length).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob that was just written is missing",
        )));
    };
    let mut body = Vec::with_capacity(length as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn read_chart(ctx: &Ctx, written: &Written) -> Result<Chart, ContentError> {
    let data = prefix(ctx, written).await?;
    tokio::task::spawn_blocking(move || chart::read(&data))
        .await
        .map_err(ContentError::internal)?
        .map_err(|error| {
            ContentError::BadRequest(format!("the file is not a Helm chart archive: {error}"))
        })
}

fn refuse(ctx: &Ctx, written: &Written, error: ContentError) -> ContentError {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    error
}

pub async fn adopt_chart(
    ctx: &Ctx,
    written: Written,
    expected: Option<&str>,
) -> Result<(Asset, Chart), ContentError> {
    let chart = match read_chart(ctx, &written).await {
        Ok(chart) => chart,
        Err(error) => return Err(refuse(ctx, &written, error)),
    };
    let file = chart.file_name();
    if let Some(expected) = expected
        && expected != file
    {
        let message = format!(
            "the archive holds the chart {} {}, which belongs at {file}, not at {expected}",
            chart.name, chart.version
        );
        return Err(refuse(ctx, &written, ContentError::BadRequest(message)));
    }
    let path = format!("/{file}");
    if expected.is_none()
        && let Err(error) = authorize(ctx, &path).await
    {
        return Err(refuse(ctx, &written, error));
    }
    let put = PutAsset::hosted(ctx, path)
        .component(key(&chart))
        .kind(CHART)
        .content_type(CHART_TYPE)
        .attributes(chart.attributes());
    let stored = store::adopt(ctx, put, &written).await?;
    Ok((stored, chart))
}

pub async fn put_chart(
    ctx: &Ctx,
    file: &str,
    body: ContentStream,
) -> Result<(Asset, Chart), ContentError> {
    authorize(ctx, &format!("/{file}")).await?;
    let written = store::write_blob(ctx, body).await?;
    adopt_chart(ctx, written, Some(file)).await
}

pub fn may_add(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Add).any() || ctx.grant(View::Edit).any() {
        true => Ok(()),
        false => ctx.require(View::Add, "/"),
    }
}

pub async fn upload_chart(ctx: &Ctx, body: ContentStream) -> Result<(Asset, Chart), ContentError> {
    may_add(ctx)?;
    let written = store::write_blob(ctx, body).await?;
    adopt_chart(ctx, written, None).await
}

pub async fn put_provenance(
    ctx: &Ctx,
    expected: Option<&str>,
    text: &[u8],
) -> Result<Asset, ContentError> {
    let text = std::str::from_utf8(text).map_err(|_| {
        ContentError::BadRequest("the provenance file is not UTF-8 text".to_owned())
    })?;
    let chart =
        provenance::chart(text).map_err(|error| ContentError::BadRequest(error.to_string()))?;
    let file = format!("{}{PROVENANCE_SUFFIX}", chart.file_name());
    if let Some(expected) = expected
        && expected != file
    {
        return Err(ContentError::BadRequest(format!(
            "the provenance file signs the chart {} {}, so it belongs at {file}, not at {expected}",
            chart.name, chart.version
        )));
    }
    let path = format!("/{file}");
    authorize(ctx, &path).await?;
    let chart_path = format!("/{}", chart.file_name());
    let attributes = match store::find(ctx.db(), ctx.name(), &chart_path).await? {
        Some(found) => match found.component_id {
            Some(id) => store::component_by_id(ctx.db(), id)
                .await?
                .map(|known| known.attributes),
            None => None,
        },
        None => None,
    }
    .unwrap_or_else(|| chart.attributes());
    let mut put = PutAsset::hosted(ctx, path)
        .component(ComponentKey {
            attributes,
            ..key(&chart)
        })
        .kind(PROVENANCE)
        .content_type(PROVENANCE_TYPE)
        .attributes(json!({}));
    put.check_content_type = false;
    store::put(ctx, put, astrum_blob::content_of(text.as_bytes().to_vec())).await
}
