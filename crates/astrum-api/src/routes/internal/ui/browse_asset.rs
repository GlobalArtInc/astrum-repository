use std::sync::Arc;

use astrum_content::{BrowseLayout, View, ids, store};
use astrum_db::entity::asset;
use astrum_repo::{Repository, RepositoryType};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
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
    value["treePath"] = json!(placed_at(db, &layout, &asset).await?);
    value["blobRef"] = json!(format!(
        "{}@{}@{}",
        asset.blob_store,
        asset.blob_id,
        asset.created_at.format("%Y-%m-%dT%H:%M")
    ));
    Ok(axum::Json(value))
}
