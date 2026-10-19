use std::collections::BTreeSet;

use astrum_blob::config::Settings;
use astrum_blob::{FillPolicy, SoftQuota, quota};
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use serde::Serialize;

use super::UI;
use super::access::survey;
use crate::routes::blobstores::measure_all;
use crate::{ApiError, AppState, Caller};

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/blobstores"), get(list))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuotaState {
    violation: bool,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StoreXo {
    name: String,
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    members: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fill_policy: Option<FillPolicy>,
    unavailable: bool,
    blob_count: u64,
    total_size: u64,
    available_space: Option<u64>,
    soft_quota: Option<SoftQuota>,
    quota_status: Option<QuotaState>,
    repositories: Vec<String>,
    repository_count: usize,
    groups: Vec<String>,
    deletable: bool,
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<StoreXo>>, ApiError> {
    caller.require("astrum:blobstores:read")?;
    let survey = survey(&state, &caller).await?;
    let visible: BTreeSet<&str> = survey.names();
    let measured = measure_all(&state).await?;
    let groups: Vec<(String, Vec<String>)> = measured
        .iter()
        .filter_map(|(config, _)| {
            config
                .group()
                .map(|group| (config.name.clone(), group.members.clone()))
        })
        .collect();
    let items = measured
        .into_iter()
        .map(|(config, measured)| {
            let users: Vec<&str> = survey
                .all
                .iter()
                .filter(|repository| repository.blob_store() == Some(config.name.as_str()))
                .map(|repository| repository.name.as_str())
                .collect();
            let containing: Vec<String> = groups
                .iter()
                .filter(|(_, members)| members.contains(&config.name))
                .map(|(group, _)| group.clone())
                .collect();
            let quota_status = config.soft_quota.as_ref().map(|soft_quota| {
                let status = quota::status(
                    &config.name,
                    Some(soft_quota),
                    measured.totals,
                    measured.available,
                );
                QuotaState {
                    violation: status.violation,
                    message: status.message,
                }
            });
            let mut item = StoreXo {
                kind: config.kind().as_str(),
                path: None,
                bucket: None,
                region: None,
                prefix: None,
                url: None,
                members: None,
                fill_policy: None,
                unavailable: measured.unavailable,
                blob_count: measured.totals.blob_count,
                total_size: measured.totals.total_bytes,
                available_space: measured.available,
                soft_quota: config.soft_quota,
                quota_status,
                repositories: users
                    .iter()
                    .filter(|name| visible.contains(*name))
                    .map(|name| (*name).to_owned())
                    .collect(),
                repository_count: users.len(),
                deletable: users.is_empty() && containing.is_empty(),
                groups: containing,
                name: config.name.clone(),
            };
            match &config.settings {
                Settings::File(file) => item.path = Some(file.path.clone()),
                Settings::S3(s3) => {
                    item.bucket = Some(s3.bucket.name.clone());
                    item.region = Some(s3.bucket.region.clone());
                    item.prefix = Some(s3.bucket.prefix.clone());
                }
                Settings::Remote(remote) => item.url = Some(remote.url.clone()),
                Settings::Group(group) => {
                    item.members = Some(group.members.clone());
                    item.fill_policy = Some(group.fill_policy);
                }
            }
            item
        })
        .collect();
    Ok(axum::Json(items))
}
