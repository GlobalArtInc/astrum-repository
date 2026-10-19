use std::sync::Arc;

use astrum_blob::BlobId;
use astrum_content::{BrowseLayout, View, ids, store};
use astrum_db::entity::asset;
use astrum_repo::{Repository, RepositoryType};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use futures_util::StreamExt;
use sea_orm::{ConnectionTrait, EntityTrait};
use serde::Deserialize;
use serde_json::{Value, json};

use super::UI;
use super::access::{Sql, context, reach_of, subject};
use super::browse::{content_error, layout_of};
use crate::routes::components::{asset_json, encode_path};
use crate::routes::repositories::BaseUrl;
use crate::{ApiError, AppState, Caller};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        &format!("{UI}/browse/{{repository}}/asset"),
        get(asset_details),
    )
}

#[derive(Deserialize)]
struct AssetQuery {
    path: Option<String>,
}

const MANIFEST_LIMIT: usize = 4 * 1024 * 1024;
const IMAGE_MANIFESTS: [&str; 2] = [
    "application/vnd.docker.distribution.manifest.v2+json",
    "application/vnd.oci.image.manifest.v1+json",
];
const INDEXES: [&str; 2] = [
    "application/vnd.docker.distribution.manifest.list.v2+json",
    "application/vnd.oci.image.index.v1+json",
];

async fn manifest_of(state: &AppState, asset: &store::Asset) -> Option<Value> {
    let store = state.blob_stores().get(&asset.blob_store).await.ok()?;
    let content = store.read(BlobId::from(asset.blob_id)).await.ok()??;
    if content.size > MANIFEST_LIMIT as u64 {
        return None;
    }
    let mut bytes = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.ok()?);
    }
    serde_json::from_slice(&bytes).ok()
}

fn media_type(asset: &store::Asset, manifest: &Value) -> String {
    asset.attributes["docker"]["mediaType"]
        .as_str()
        .or_else(|| manifest["mediaType"].as_str())
        .unwrap_or_default()
        .to_owned()
}

pub(crate) fn image_size(manifest: &Value) -> Option<(u64, usize)> {
    let layers = manifest["layers"].as_array()?;
    let config = manifest["config"]["size"].as_u64().unwrap_or(0);
    let total = layers
        .iter()
        .filter_map(|layer| layer["size"].as_u64())
        .fold(config, u64::saturating_add);
    Some((total, layers.len()))
}

fn platform_of(descriptor: &Value) -> Option<String> {
    let platform = &descriptor["platform"];
    let os = platform["os"].as_str()?;
    let architecture = platform["architecture"].as_str()?;
    let attestation = descriptor["annotations"]["vnd.docker.reference.type"].as_str()
        == Some("attestation-manifest");
    if attestation || os == "unknown" {
        return None;
    }
    Some(match platform["variant"].as_str() {
        Some(variant) => format!("{os}/{architecture}/{variant}"),
        None => format!("{os}/{architecture}"),
    })
}

async fn image_of(state: &AppState, asset: &store::Asset) -> Option<Value> {
    if asset.format != "docker" {
        return None;
    }
    let manifest = manifest_of(state, asset).await?;
    let kind = media_type(asset, &manifest);
    if IMAGE_MANIFESTS.contains(&kind.as_str()) {
        let (size, layers) = image_size(&manifest)?;
        return Some(json!({ "size": size, "layers": layers }));
    }
    if !INDEXES.contains(&kind.as_str()) {
        return None;
    }
    let name = asset.attributes["docker"]["name"].as_str()?;
    let mut platforms = Vec::new();
    for descriptor in manifest["manifests"].as_array()? {
        let (Some(platform), Some(digest)) =
            (platform_of(descriptor), descriptor["digest"].as_str())
        else {
            continue;
        };
        let child = store::find(
            state.db().conn(),
            &asset.repository,
            &format!("/v2/{name}/manifests/{digest}"),
        )
        .await
        .ok()
        .flatten();
        let size = match child {
            Some(child) => manifest_of(state, &child)
                .await
                .and_then(|manifest| image_size(&manifest)),
            None => None,
        };
        platforms.push(json!({
            "platform": platform,
            "digest": digest,
            "size": size.map(|(size, _)| size),
            "layers": size.map(|(_, layers)| layers),
        }));
    }
    Some(json!({ "platforms": platforms }))
}

fn asset_missing(path: &str, repository: &str) -> ApiError {
    ApiError::not_found(format!("{path} is not in the repository {repository}"))
}

async fn placed(
    db: &sea_orm::DatabaseConnection,
    layout: &BrowseLayout,
    repository: &str,
    path: &str,
) -> Result<Option<store::Asset>, ApiError> {
    let Some(expression) = layout.path else {
        return Ok(None);
    };
    let mut sql = Sql::default();
    let name = sql.bind(repository.to_owned());
    let wanted = sql.bind(path.to_owned());
    let query = format!(
        "SELECT a.* FROM asset a LEFT JOIN component c ON c.id = a.component_id WHERE a.repository = {name} AND ({expression}) = {wanted} LIMIT 1"
    );
    Ok(asset::Entity::find()
        .from_raw_sql(sql.statement(&query))
        .one(db)
        .await?)
}

async fn placed_at(
    db: &sea_orm::DatabaseConnection,
    layout: &BrowseLayout,
    asset: &store::Asset,
) -> Result<Option<String>, ApiError> {
    let Some(expression) = layout.path else {
        return Ok(Some(asset.path.clone()));
    };
    let mut sql = Sql::default();
    let id = sql.bind(asset.id);
    let query = format!(
        "SELECT ({expression}) AS tree_path FROM asset a LEFT JOIN component c ON c.id = a.component_id WHERE a.id = {id}"
    );
    let row = db.query_one(sql.statement(&query)).await?;
    Ok(match row {
        Some(row) => row.try_get::<Option<String>>("", "tree_path")?,
        None => None,
    })
}

async fn asset_details(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
    Path(name): Path<String>,
    Query(query): Query<AssetQuery>,
) -> Result<axum::Json<Value>, ApiError> {
    let subject = subject(&caller)?;
    let repository = state
        .content()
        .repository(&name)
        .await
        .map_err(content_error)?
        .ok_or_else(|| ApiError::not_found(format!("Repository not found: {name}")))?;
    let ctx = context(&state, &subject, repository.clone());
    let reach = reach_of(&ctx);
    if !reach.visible() {
        return Err(ApiError::not_found(format!("Repository not found: {name}")));
    }
    let path = astrum_content::normalize(query.path.as_deref().unwrap_or_default())
        .map_err(content_error)?;
    if path.ends_with('/') {
        return Err(asset_missing(&path, &repository.name));
    }
    let layout = layout_of(&state, &repository);
    let readers: Vec<Arc<Repository>> = match repository.kind {
        RepositoryType::Group => astrum_content::group::members(&ctx)
            .await
            .map_err(content_error)?,
        _ => vec![repository.clone()],
    };
    let db = state.db().conn();
    let mut found = None;
    for reader in readers {
        let stored = match store::find(db, &reader.name, &path)
            .await
            .map_err(content_error)?
        {
            Some(asset) => Some(asset),
            None => placed(db, &layout, &reader.name, &path).await?,
        };
        if let Some(asset) = stored {
            found = Some((reader, asset));
            break;
        }
    }
    let (owner, asset) = found
        .filter(|(_, asset)| reach.sees(repository.format.id, &asset.path))
        .ok_or_else(|| asset_missing(&path, &repository.name))?;
    let component = match asset.component_id {
        Some(id) => store::component_by_id(db, id)
            .await
            .map_err(content_error)?,
        None => None,
    };
    let owner_ctx = context(&state, &subject, owner);
    let can_delete = owner_ctx.allowed(View::Delete, &asset.path)
        && (owner_ctx.allowed(View::Browse, &asset.path)
            || owner_ctx.allowed(View::Read, &asset.path));
    let mut value = asset_json(&base.0, &asset);
    value["downloadUrl"] = json!(format!(
        "{}/repository/{}{}",
        base.0,
        repository.name,
        encode_path(&asset.path)
    ));
    value["browsedRepository"] = json!(repository.name);
    value["component"] = component.map_or(Value::Null, |component| {
        json!({
            "id": ids::encode(&component.repository, component.id),
            "repository": component.repository,
            "group": component.namespace,
            "name": component.name,
            "version": component.version,
        })
    });
    value["canDelete"] = json!(can_delete);
    if let Some(image) = image_of(&state, &asset).await {
        value["image"] = image;
    }
    value["treePath"] = json!(placed_at(db, &layout, &asset).await?);
    value["blobRef"] = json!(format!(
        "{}@{}@{}",
        asset.blob_store,
        asset.blob_id,
        asset.created_at.format("%Y-%m-%dT%H:%M")
    ));
    Ok(axum::Json(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_image_weighs_its_config_and_layers() {
        let manifest = json!({
            "config": {"size": 1500},
            "layers": [{"size": 30_000_000}, {"size": 2_000_000}],
        });
        assert_eq!(image_size(&manifest), Some((32_001_500, 2)));
        assert_eq!(image_size(&json!({"manifests": []})), None);
    }

    #[test]
    fn attestations_are_not_platforms() {
        let amd = json!({"platform": {"os": "linux", "architecture": "amd64"}});
        let arm = json!({"platform": {"os": "linux", "architecture": "arm64", "variant": "v8"}});
        let attestation = json!({
            "platform": {"os": "unknown", "architecture": "unknown"},
            "annotations": {"vnd.docker.reference.type": "attestation-manifest"},
        });
        assert_eq!(platform_of(&amd).as_deref(), Some("linux/amd64"));
        assert_eq!(platform_of(&arm).as_deref(), Some("linux/arm64/v8"));
        assert_eq!(platform_of(&attestation), None);
    }
}
