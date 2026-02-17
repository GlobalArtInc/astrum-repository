use std::collections::BTreeSet;

use astrum_blob::config::{GroupSettings, Settings, StoreConfig};
use astrum_blob::{FillPolicy, SoftQuota, StoreKind};
use astrum_db::audit::Event;
use astrum_db::entity::repository;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    CREATE, DOMAIN, QuotaInput, READ, UPDATE, check_name, check_quota, ensure_unique, find, insert,
    internal, refuse_type_change, save,
};
use crate::{ApiError, AppState, Caller, Json, Validation};

const MEMBERS: &str = "members";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GroupRequest {
    name: Option<String>,
    soft_quota: Option<QuotaInput>,
    members: Option<Vec<String>>,
    fill_policy: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GroupModel {
    name: String,
    soft_quota: Option<SoftQuota>,
    members: Vec<String>,
    fill_policy: FillPolicy,
}

fn check_policy(validation: &mut Validation, value: Option<&str>) -> FillPolicy {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("writeToFirst") => FillPolicy::WriteToFirst,
        Some("roundRobin") => FillPolicy::RoundRobin,
        Some(other) => {
            validation.add(
                "fillPolicy",
                format!("{other} is not a fill policy; use writeToFirst or roundRobin"),
            );
            FillPolicy::WriteToFirst
        }
    }
}

fn check_member_list(validation: &mut Validation, members: Option<Vec<String>>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut kept = Vec::new();
    for member in members.unwrap_or_default() {
        let member = member.trim().to_owned();
        if member.is_empty() {
            continue;
        }
        match seen.insert(member.to_lowercase()) {
            true => kept.push(member),
            false => validation.add(MEMBERS, format!("{member} is listed more than once")),
        }
    }
    if kept.is_empty() {
        validation.add(MEMBERS, "a group blob store needs at least one member");
    }
    kept
}

async fn check_members(
    state: &AppState,
    group: &str,
    members: &[String],
    current: &[String],
) -> Result<(), ApiError> {
    let configs = state.blob_stores().configs().await.map_err(internal)?;
    let mut validation = Validation::new();
    for member in members {
        if member.eq_ignore_ascii_case(group) {
            validation.add(
                MEMBERS,
                format!("the group {group} cannot be its own member"),
            );
            continue;
        }
        let Some(config) = configs.iter().find(|config| config.name == *member) else {
            validation.add(MEMBERS, format!("there is no blob store named {member}"));
            continue;
        };
        if config.kind() == StoreKind::Group {
            validation.add(
                MEMBERS,
                format!(
                    "{member} is a group blob store; a group cannot be a member of another group"
                ),
            );
            continue;
        }
        if let Some(other) = configs.iter().find(|other| {
            other.name != group
                && other
                    .group()
                    .is_some_and(|settings| settings.members.contains(member))
        }) {
            validation.add(
                MEMBERS,
                format!("{member} is already a member of the group {}", other.name),
            );
            continue;
        }
        let users: Vec<String> = repository::Entity::find()
            .select_only()
            .column(repository::Column::Name)
            .filter(repository::Column::BlobStore.eq(member.as_str()))
            .order_by_asc(repository::Column::Name)
            .into_tuple()
            .all(state.db().conn())
            .await?;
        if !users.is_empty() {
            validation.add(
                MEMBERS,
                format!(
                    "{member} stores the repositories {}; a blob store serves repositories either directly or through one group",
                    users.join(", ")
                ),
            );
        }
    }
    for removed in current.iter().filter(|name| !members.contains(name)) {
        let totals = state
            .blob_stores()
            .metrics(removed)
            .totals()
            .await
            .map_err(internal)?;
        if totals.blob_count > 0 {
            validation.add(
                MEMBERS,
                format!(
                    "{removed} holds {} blobs of the group; removing it would lose them",
                    totals.blob_count
                ),
            );
        }
    }
    validation.finish()
}

pub(super) async fn create(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<GroupRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(CREATE)?;
    let Json(body) = body?;
    let mut validation = Validation::new();
    let name = check_name(&mut validation, body.name.as_deref());
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    let fill_policy = check_policy(&mut validation, body.fill_policy.as_deref());
    let members = check_member_list(&mut validation, body.members);
    validation.finish()?;
    ensure_unique(&state, &name).await?;
    check_members(&state, &name, &members, &[]).await?;
    let config = StoreConfig {
        name,
        settings: Settings::Group(GroupSettings {
            members: members.clone(),
            fill_policy,
        }),
        soft_quota,
    };
    insert(&state, &config).await?;
    state.blob_stores().forget();
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "created", &config.name)
                .with(json!({"type": "Group", "members": members, "fillPolicy": fill_policy})),
        )
        .await;
    Ok(StatusCode::CREATED)
}

pub(super) async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<GroupModel>, ApiError> {
    caller.require(READ)?;
    let config = state
        .blob_stores()
        .config(&name)
        .await
        .map_err(internal)?
        .filter(|config| config.kind() == StoreKind::Group)
        .ok_or_else(|| {
            ApiError::not_found(format!("There is no group blob store named '{name}'"))
        })?;
    let group = config.group().cloned().unwrap_or(GroupSettings {
        members: Vec::new(),
        fill_policy: FillPolicy::WriteToFirst,
    });
    Ok(axum::Json(GroupModel {
        name: config.name,
        soft_quota: config.soft_quota,
        members: group.members,
        fill_policy: group.fill_policy,
    }))
}

pub(super) async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<Json<GroupRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let Json(body) = body?;
    let current = find(&state, &name).await?;
    refuse_type_change(&current, StoreKind::Group)?;
    let mut validation = Validation::new();
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    let fill_policy = check_policy(&mut validation, body.fill_policy.as_deref());
    let members = check_member_list(&mut validation, body.members);
    validation.finish()?;
    let before = current
        .group()
        .map(|group| group.members.clone())
        .unwrap_or_default();
    check_members(&state, &current.name, &members, &before).await?;
    let config = StoreConfig {
        name: current.name.clone(),
        settings: Settings::Group(GroupSettings {
            members: members.clone(),
            fill_policy,
        }),
        soft_quota,
    };
    save(&state, &config).await?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &config.name)
                .with(json!({"type": "Group", "members": members, "fillPolicy": fill_policy})),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}
