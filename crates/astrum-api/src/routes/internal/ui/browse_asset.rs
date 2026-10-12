use std::sync::Arc;

use astrum_content::{View, ids, store};
use astrum_repo::{Repository, RepositoryType};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use serde::Deserialize;
use serde_json::{Value, json};

use super::UI;
use super::access::{context, reach_of, subject};
use super::browse::content_error;
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
    if path.ends_with('/') || !reach.sees(repository.format.id, &path) {
        return Err(asset_missing(&path, &repository.name));
    }
    let readers: Vec<Arc<Repository>> = match repository.kind {
        RepositoryType::Group => astrum_content::group::members(&ctx)
            .await
            .map_err(content_error)?,
        _ => vec![repository.clone()],
    };
    let db = state.db().conn();
    let mut found = None;
    for reader in readers {
        if let Some(asset) = store::find(db, &reader.name, &path)
            .await
            .map_err(content_error)?
        {
            found = Some((reader, asset));
            break;
        }
    }
    let (owner, asset) = found.ok_or_else(|| asset_missing(&path, &repository.name))?;
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
    Ok(axum::Json(value))
}
