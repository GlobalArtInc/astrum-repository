use astrum_content::replicated;
use astrum_content::transfer::origin_of;
use astrum_content::{ContentError, View};
use astrum_db::entity::component;
use astrum_replication::protocol::{BLOB, BUNDLE, component_key, valid_sha256};
use axum::Router;
use axum::body::Body;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use super::receive::{PULL, Refusal, instance_id, open_as, statement};
use crate::{ApiError, AppState, Caller};

const COMPONENT_BLOB: &str = "SELECT a.blob_store, a.blob_id, c.namespace, c.name, c.version FROM asset a \
     JOIN component c ON c.id = a.component_id \
     WHERE a.repository = $1 AND c.repository = $1 AND a.sha256 = $2 \
     ORDER BY a.id LIMIT 1";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(BUNDLE, get(bundle))
        .route(BLOB, get(blob))
}

#[derive(Deserialize)]
struct Coordinates {
    namespace: Option<String>,
    name: Option<String>,
    version: Option<String>,
}

#[derive(FromQueryResult)]
struct Located {
    blob_store: String,
    blob_id: Uuid,
    namespace: String,
    name: String,
    version: String,
}

async fn bundle(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    query: Result<Query<Coordinates>, QueryRejection>,
) -> Result<Response, Refusal> {
    let who = caller.initiator();
    let ctx = open_as(&state, caller, &repository, View::Read, &[PULL]).await?;
    replicated::require_hosted(&ctx)?;
    let format = replicated::format_of(&ctx)?;
    let Query(wanted) = query.map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
    let name = wanted
        .name
        .ok_or_else(|| ApiError::invalid("name", "must name the component"))?;
    let found = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Namespace.eq(wanted.namespace.unwrap_or_default()))
        .filter(component::Column::Name.eq(name))
        .filter(component::Column::Version.eq(wanted.version.unwrap_or_default()))
        .one(state.db().conn())
        .await?
        .ok_or_else(replicated::component_gone)?;
    let mut bundle = replicated::transfer_of(&format)?
        .bundle(&ctx, &found)
        .await?;
    let origin = match origin_of(&found.attributes) {
        Some(origin) => origin.to_owned(),
        None => instance_id(&state).await?.to_string(),
    };
    bundle.origin = Some(origin);
    let bytes: u64 = bundle.assets.iter().map(|asset| asset.size).sum();
    tracing::info!(
        repository = ctx.name(),
        component = %component_key(&bundle.namespace, &bundle.name, &bundle.version),
        caller = %who,
        assets = bundle.assets.len(),
        bytes,
        "a bundle was served to a replication pull"
    );
    Ok(axum::Json(bundle).into_response())
}

async fn blob(
    State(state): State<AppState>,
    caller: Caller,
    Path((repository, sha256)): Path<(String, String)>,
) -> Result<Response, Refusal> {
    let who = caller.initiator();
    let ctx = open_as(&state, caller, &repository, View::Read, &[PULL]).await?;
    replicated::require_hosted(&ctx)?;
    replicated::format_of(&ctx)?;
    if !valid_sha256(&sha256) {
        return Err(replicated::refuse(
            "bad_digest",
            "the digest in the path is not 64 lowercase hexadecimal characters",
        )
        .into());
    }
    let located = Located::find_by_statement(statement(
        COMPONENT_BLOB,
        vec![ctx.name().into(), sha256.clone().into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| {
        Refusal::Status(
            StatusCode::NOT_FOUND,
            "blob_not_found",
            "no asset of a component of the repository has this digest".to_owned(),
        )
    })?;
    let store = state
        .content()
        .blob_stores()
        .get(&located.blob_store)
        .await
        .map_err(ContentError::from)?;
    let content = store
        .read(located.blob_id.into())
        .await
        .map_err(ContentError::from)?
        .ok_or_else(|| {
            Refusal::Status(
                StatusCode::NOT_FOUND,
                "blob_missing",
                "the blob of the asset is missing from its blob store".to_owned(),
            )
        })?;
    tracing::info!(
        repository = ctx.name(),
        component = %component_key(&located.namespace, &located.name, &located.version),
        caller = %who,
        sha256,
        bytes = content.length,
        "a blob was served to a replication pull"
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, content.length)
        .body(Body::from_stream(content.stream))
        .map_err(|error| ApiError::internal(anyhow::Error::from(error)).into())
}
