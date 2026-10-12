mod file;
mod group;
mod s3;

use std::time::Duration;

use astrum_blob::config::{self, Settings, StoreConfig};
use astrum_blob::{QuotaKind, SoftQuota, StoreKind, Totals, quota};
use astrum_db::audit::Event;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{ApiError, AppState, Caller, Validation};

const BLOBSTORES: &str = "/service/rest/v1/blobstores";
const READ: &str = "astrum:blobstores:read";
const CREATE: &str = "astrum:blobstores:create";
const UPDATE: &str = "astrum:blobstores:update";
const DELETE: &str = "astrum:blobstores:delete";
const DOMAIN: &str = "blobstore";
const CHECK_TIMEOUT: Duration = Duration::from_secs(5);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(BLOBSTORES, get(list))
        .route(&format!("{BLOBSTORES}/{{name}}"), delete(remove))
        .route(
            &format!("{BLOBSTORES}/{{name}}/quota-status"),
            get(quota_status),
        )
        .route(&format!("{BLOBSTORES}/file"), post(file::create))
        .route(
            &format!("{BLOBSTORES}/file/{{name}}"),
            get(file::read).put(file::update),
        )
        .route(&format!("{BLOBSTORES}/group"), post(group::create))
        .route(
            &format!("{BLOBSTORES}/group/{{name}}"),
            get(group::read).put(group::update),
        )
        .route(&format!("{BLOBSTORES}/s3"), post(s3::create))
        .route(
            &format!("{BLOBSTORES}/s3/{{name}}"),
            get(s3::read).put(s3::update),
        )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    soft_quota: Option<SoftQuota>,
    name: String,
    #[serde(rename = "type")]
    kind: &'static str,
    unavailable: bool,
    blob_count: u64,
    total_size_in_bytes: u64,
    available_space_in_bytes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuotaResult {
    is_violation: bool,
    message: String,
    blob_store_name: String,
}

pub(crate) struct Measured {
    pub(crate) unavailable: bool,
    pub(crate) totals: Totals,
    pub(crate) available: Option<u64>,
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<Summary>>, ApiError> {
    caller.require(READ)?;
    Ok(axum::Json(
        measure_all(&state)
            .await?
            .into_iter()
            .map(|(config, measured)| Summary {
                soft_quota: config.soft_quota,
                kind: config.kind().as_str(),
                unavailable: measured.unavailable,
                blob_count: measured.totals.blob_count,
                total_size_in_bytes: measured.totals.total_bytes,
                available_space_in_bytes: measured.available,
                name: config.name,
            })
            .collect(),
    ))
}

pub(crate) async fn measure_all(
    state: &AppState,
) -> Result<Vec<(StoreConfig, Measured)>, ApiError> {
    let configs = state.blob_stores().configs().await.map_err(internal)?;
    let mut checks = JoinSet::new();
    for (index, config) in configs.iter().cloned().enumerate() {
        let state = state.clone();
        checks.spawn(async move { (index, measure(&state, &config).await) });
    }
    let mut measured: Vec<Option<Measured>> = configs.iter().map(|_| None).collect();
    while let Some(done) = checks.join_next().await {
        let (index, result) = done.map_err(ApiError::internal)?;
        measured[index] = Some(result);
    }
    Ok(configs
        .into_iter()
        .zip(measured)
        .map(|(config, measured)| (config, measured.expect("every store was measured")))
        .collect())
}

async fn quota_status(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<QuotaResult>, ApiError> {
    caller.require(READ)?;
    let config = find(&state, &name).await?;
    let measured = measure(&state, &config).await;
    let status = quota::status(
        &config.name,
        config.soft_quota.as_ref(),
        measured.totals,
        measured.available,
    );
    Ok(axum::Json(QuotaResult {
        is_violation: status.violation,
        message: status.message,
        blob_store_name: config.name,
    }))
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(DELETE)?;
    let config = find(&state, &name).await?;
    match config::delete(state.db().conn(), &name).await {
        Ok(true) => {}
        Ok(false) => return Err(missing(&name)),
        Err(astrum_blob::Error::InUse(_)) => {
            return Err(ApiError::BadRequest(format!(
                "Blob store {name} is in use and cannot be deleted"
            )));
        }
        Err(error) => return Err(internal(error)),
    }
    state.blob_stores().forget();
    if let Settings::File(file) = &config.settings {
        let root = state.blob_stores().resolve(&file.path);
        if let Err(error) = astrum_blob::file::remove_if_empty(&root).await {
            tracing::warn!(store = %name, %error, "the directory of a deleted blob store was kept");
        }
    }
    state
        .audit(&caller, Event::new(DOMAIN, "deleted", &name))
        .await;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn measure(state: &AppState, config: &StoreConfig) -> Measured {
    let stores = state.blob_stores();
    let store = match stores.get(&config.name).await {
        Ok(store) => store,
        Err(error) => {
            tracing::warn!(store = %config.name, %error, "a blob store cannot be opened");
            let totals = stores
                .metrics(&config.name)
                .totals()
                .await
                .unwrap_or_default();
            return Measured {
                unavailable: true,
                totals,
                available: None,
            };
        }
    };
    let healthy = match tokio::time::timeout(CHECK_TIMEOUT, store.check()).await {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            tracing::warn!(store = %config.name, %error, "a blob store is unavailable");
            false
        }
        Err(_) => {
            tracing::warn!(store = %config.name, "a blob store did not answer in time");
            false
        }
    };
    let totals = store.totals().await.unwrap_or_else(|error| {
        tracing::warn!(store = %config.name, %error, "the totals of a blob store cannot be read");
        Totals::default()
    });
    let available = match store.available_space().await {
        Ok(available) => available,
        Err(error) => {
            tracing::warn!(store = %config.name, %error, "the free space of a blob store is unknown");
            None
        }
    };
    Measured {
        unavailable: !healthy,
        totals,
        available,
    }
}

#[derive(Debug, Default, Deserialize)]
struct QuotaInput {
    #[serde(rename = "type")]
    kind: Option<String>,
    limit: Option<i64>,
}

fn check_quota(validation: &mut Validation, input: Option<QuotaInput>) -> Option<SoftQuota> {
    let input = input?;
    let kind = input.kind?;
    let Some(kind) = QuotaKind::parse(&kind) else {
        validation.add(
            "*",
            "A soft quota needs a type: spaceRemainingQuota or spaceUsedQuota",
        );
        return None;
    };
    let label = match kind {
        QuotaKind::SpaceRemainingQuota => "space remaining",
        QuotaKind::SpaceUsedQuota => "space used",
    };
    match input.limit {
        Some(limit) if limit > 0 => Some(SoftQuota {
            kind,
            limit: limit as u64,
        }),
        _ => {
            validation.add("*", format!("A {label} quota needs a limit above 0"));
            None
        }
    }
}

fn check_name(validation: &mut Validation, name: Option<&str>) -> String {
    let name = name.unwrap_or_default().trim();
    if name.is_empty() {
        validation.add("PARAMETER name", "must not be empty");
    } else if !valid_name(name) {
        validation.add(
            "PARAMETER name",
            "may hold only letters, digits, '-', '_' and '.', and must start with a letter, a digit or '-'",
        );
    }
    name.to_owned()
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric() || first == '-')
        && chars.all(|next| next.is_ascii_alphanumeric() || matches!(next, '-' | '_' | '.'))
}

async fn ensure_unique(state: &AppState, name: &str) -> Result<(), ApiError> {
    match config::name_taken(state.db().conn(), name)
        .await
        .map_err(internal)?
    {
        Some(_) => Err(duplicate()),
        None => Ok(()),
    }
}

async fn insert(state: &AppState, config: &StoreConfig) -> Result<(), ApiError> {
    match config::insert(state.db().conn(), config).await {
        Ok(()) => Ok(()),
        Err(astrum_blob::Error::Duplicate(_)) => Err(duplicate()),
        Err(error) => Err(internal(error)),
    }
}

async fn save(state: &AppState, config: &StoreConfig) -> Result<(), ApiError> {
    if !config::update(state.db().conn(), config)
        .await
        .map_err(internal)?
    {
        return Err(missing(&config.name));
    }
    state.blob_stores().forget();
    Ok(())
}

async fn find(state: &AppState, name: &str) -> Result<StoreConfig, ApiError> {
    state
        .blob_stores()
        .config(name)
        .await
        .map_err(internal)?
        .ok_or_else(|| missing(name))
}

fn refuse_type_change(config: &StoreConfig, wanted: StoreKind) -> Result<(), ApiError> {
    if config.kind() == wanted {
        return Ok(());
    }
    Err(ApiError::BadRequest(format!(
        "Blob store {} is a {} blob store; the type of a blob store cannot change",
        config.name,
        config.kind()
    )))
}

fn duplicate() -> ApiError {
    ApiError::invalid(
        "name",
        "The name is used by another blob store (names are compared ignoring case)",
    )
}

fn missing(name: &str) -> ApiError {
    ApiError::not_found(format!("There is no blob store named '{name}'"))
}

fn internal(error: astrum_blob::Error) -> ApiError {
    ApiError::internal(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_documented_pattern() {
        for name in ["default", "a", "-a", "A.b_c-1", "0"] {
            assert!(valid_name(name), "{name}");
        }
        for name in ["", "_a", ".a", "a b", "a/b", "ä"] {
            assert!(!valid_name(name), "{name}");
        }
    }

    #[test]
    fn quotas_without_a_type_mean_no_quota() {
        let mut validation = Validation::new();
        assert_eq!(check_quota(&mut validation, None), None);
        assert_eq!(
            check_quota(&mut validation, Some(QuotaInput::default())),
            None
        );
        assert_eq!(
            check_quota(
                &mut validation,
                Some(QuotaInput {
                    kind: None,
                    limit: Some(5)
                })
            ),
            None
        );
        assert!(validation.is_empty());
        assert_eq!(
            check_quota(
                &mut validation,
                Some(QuotaInput {
                    kind: Some("spaceUsedQuota".to_owned()),
                    limit: Some(5)
                })
            ),
            Some(SoftQuota {
                kind: QuotaKind::SpaceUsedQuota,
                limit: 5
            })
        );
        assert!(validation.is_empty());
    }
}
