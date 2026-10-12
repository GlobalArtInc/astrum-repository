use astrum_blob::config::{FileSettings, Settings, StoreConfig};
use astrum_blob::{FileBlobStore, SoftQuota, StoreKind};
use astrum_db::audit::Event;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use super::{
    CREATE, DOMAIN, QuotaInput, READ, UPDATE, check_name, check_quota, ensure_unique, find, insert,
    internal, refuse_type_change, save,
};
use crate::{ApiError, AppState, Caller, Json, Validation};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileRequest {
    name: Option<String>,
    path: Option<String>,
    soft_quota: Option<QuotaInput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileModel {
    soft_quota: Option<SoftQuota>,
    path: String,
}

pub(super) async fn create(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<FileRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(CREATE)?;
    let Json(body) = body?;
    let mut validation = Validation::new();
    let name = check_name(&mut validation, body.name.as_deref());
    let path = check_path(&mut validation, body.path.as_deref());
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    validation.finish()?;
    ensure_unique(&state, &name).await?;
    ensure_own_path(&state, &path, None).await?;
    let config = StoreConfig {
        name,
        settings: Settings::File(FileSettings { path }),
        soft_quota,
    };
    prepare(&state, &config).await?;
    insert(&state, &config).await?;
    state.blob_stores().forget();
    state
        .audit(&caller, Event::new(DOMAIN, "created", &config.name))
        .await;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<FileModel>, ApiError> {
    caller.require(READ)?;
    let config = state
        .blob_stores()
        .config(&name)
        .await
        .map_err(internal)?
        .filter(|config| config.kind() == StoreKind::File)
        .ok_or_else(|| {
            ApiError::not_found(format!("There is no file blob store named '{name}'"))
        })?;
    let path = config
        .file()
        .map(|file| file.path.clone())
        .unwrap_or_default();
    Ok(axum::Json(FileModel {
        soft_quota: config.soft_quota,
        path,
    }))
}

pub(super) async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<Json<FileRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let Json(body) = body?;
    let current = find(&state, &name).await?;
    refuse_type_change(&current, StoreKind::File)?;
    let mut validation = Validation::new();
    let path = check_path(&mut validation, body.path.as_deref());
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    validation.finish()?;

    let stores = state.blob_stores();
    let old_path = current
        .file()
        .map(|file| file.path.clone())
        .unwrap_or_default();
    let config = StoreConfig {
        name: current.name.clone(),
        settings: Settings::File(FileSettings { path: path.clone() }),
        soft_quota,
    };
    if stores.resolve(&old_path) != stores.resolve(&path) {
        let totals = stores
            .metrics(&current.name)
            .totals()
            .await
            .map_err(internal)?;
        if totals.blob_count > 0 {
            return Err(ApiError::invalid(
                "PARAMETER path",
                "The path of a blob store that holds blobs cannot change",
            ));
        }
        ensure_own_path(&state, &path, Some(&current.name)).await?;
        prepare(&state, &config).await?;
    }
    save(&state, &config).await?;
    state
        .audit(&caller, Event::new(DOMAIN, "updated", &config.name))
        .await;
    Ok(StatusCode::NO_CONTENT)
}

fn check_path(validation: &mut Validation, path: Option<&str>) -> String {
    let path = path.unwrap_or_default().trim();
    if path.is_empty() {
        validation.add("PARAMETER path", "must not be empty");
    }
    path.to_owned()
}

async fn ensure_own_path(
    state: &AppState,
    path: &str,
    except: Option<&str>,
) -> Result<(), ApiError> {
    let overlapping = state
        .blob_stores()
        .overlapping(path, except)
        .await
        .map_err(internal)?;
    if overlapping.is_empty() {
        return Ok(());
    }
    Err(ApiError::invalid(
        "*",
        format!(
            "A file blob store needs a path of its own, neither inside nor around another blob store; it overlaps {}",
            overlapping.join(", ")
        ),
    ))
}

async fn prepare(state: &AppState, config: &StoreConfig) -> Result<(), ApiError> {
    let stores = state.blob_stores();
    let path = config
        .file()
        .map(|file| file.path.as_str())
        .unwrap_or_default();
    FileBlobStore::open(
        config.name.as_str(),
        stores.resolve(path),
        stores.metrics(&config.name),
    )
    .await
    .map(|_| ())
    .map_err(|error| ApiError::invalid("PARAMETER path", error.to_string()))
}
