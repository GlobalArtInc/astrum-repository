use astrum_db::audit::Event;
use astrum_db::entity::security_role;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::{Role, SecurityModel, Subject};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use sea_orm::{ActiveValue, ConnectionTrait, DbBackend, EntityTrait, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::users::check_grants;
use super::{SECURITY, changed, conflict, dedup, now, unique_violation, validate};
use crate::{ApiError, AppState, Caller, FieldError, Json, Validation};

const DOMAIN: &str = "security.role";
const EXTERNAL_SOURCE: &str = "LDAP";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{SECURITY}/roles"), get(list).post(create))
        .route(&format!("{SECURITY}/roles/assignable"), get(assignable))
        .route(
            &format!("{SECURITY}/roles/{{id}}"),
            get(read).put(update).delete(remove),
        )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoleXo {
    id: String,
    source: String,
    name: String,
    description: String,
    read_only: bool,
    privileges: Vec<String>,
    roles: Vec<String>,
}

impl From<&Role> for RoleXo {
    fn from(role: &Role) -> Self {
        Self {
            id: role.id.clone(),
            source: role.source.clone(),
            name: role.name.clone(),
            description: role.description.clone(),
            read_only: role.read_only,
            privileges: role.privileges.clone(),
            roles: role.roles.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SourceQuery {
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoleRequest {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    privileges: Option<Vec<String>>,
    roles: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Local,
    External,
}

fn source(query: &SourceQuery) -> Result<Source, ApiError> {
    match query.source.as_deref().map(str::trim) {
        None | Some("") | Some(DEFAULT_SOURCE) => Ok(Source::Local),
        Some(EXTERNAL_SOURCE) => Ok(Source::External),
        Some(other) => Err(ApiError::BadRequest(format!(
            "the role source {other} does not exist; use {DEFAULT_SOURCE} or {EXTERNAL_SOURCE}"
        ))),
    }
}

fn role_not_found(id: &str) -> ApiError {
    ApiError::not_found(format!("Role '{id}' not found."))
}

fn built_in(id: &str) -> ApiError {
    ApiError::BadRequest(format!(
        "the role {id} is built in and cannot be changed or deleted"
    ))
}

struct Checked {
    id: String,
    name: String,
    description: Option<String>,
    privileges: Vec<String>,
    roles: Vec<String>,
}

fn check_body(body: RoleRequest) -> Result<Checked, ApiError> {
    let mut validation = Validation::new();
    let id = body.id.unwrap_or_default();
    if let Some(problem) = validate::identifier_problem(&id) {
        validation.add("id", problem);
    }
    validation.require("name", body.name.as_deref());
    validation.finish()?;
    Ok(Checked {
        id,
        name: body.name.unwrap_or_default(),
        description: body.description,
        privileges: dedup(body.privileges.unwrap_or_default()),
        roles: dedup(body.roles.unwrap_or_default()),
    })
}

fn check_references(model: &SecurityModel, role: &Checked) -> Result<(), ApiError> {
    let mut errors = Vec::new();
    for nested in &role.roles {
        if !model.roles.contains_key(nested) {
            errors.push(FieldError {
                id: "roles".to_owned(),
                message: format!("the role {nested} does not exist"),
            });
        }
    }
    for privilege in &role.privileges {
        if !model.privileges.contains_key(privilege) {
            errors.push(FieldError {
                id: "privileges".to_owned(),
                message: format!("the privilege {privilege} does not exist"),
            });
        }
    }
    if !errors.is_empty() {
        return Err(ApiError::Invalid(errors));
    }
    if role
        .roles
        .iter()
        .any(|nested| model.contains_role(nested, &role.id))
    {
        return Err(ApiError::invalid(
            "roles",
            format!(
                "the role {} cannot contain itself, directly or through other roles",
                role.id
            ),
        ));
    }
    Ok(())
}

fn check_privilege_grants<'a>(
    model: &SecurityModel,
    subject: &Subject,
    privileges: impl IntoIterator<Item = &'a String>,
) -> Result<(), ApiError> {
    for privilege in privileges {
        if !model.can_grant_privilege(subject, privilege) {
            return Err(ApiError::Forbidden(format!(
                "you cannot grant the privilege {privilege}: it gives permissions you do not have"
            )));
        }
    }
    Ok(())
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<SourceQuery>,
) -> Result<axum::Json<Vec<RoleXo>>, ApiError> {
    caller.require("astrum:roles:read")?;
    if source(&query)? == Source::External {
        return Ok(axum::Json(Vec::new()));
    }
    let model = state.security().model();
    Ok(axum::Json(model.roles.values().map(RoleXo::from).collect()))
}

async fn assignable(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<SourceQuery>,
) -> Result<axum::Json<Vec<RoleXo>>, ApiError> {
    let subject = caller.require("astrum:roles:read")?;
    if source(&query)? == Source::External {
        return Ok(axum::Json(Vec::new()));
    }
    let model = state.security().model();
    Ok(axum::Json(
        model
            .roles
            .values()
            .filter(|role| model.can_grant_role(subject, &role.id))
            .map(RoleXo::from)
            .collect(),
    ))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Query(query): Query<SourceQuery>,
) -> Result<axum::Json<RoleXo>, ApiError> {
    caller.require("astrum:roles:read")?;
    if source(&query)? == Source::External {
        return Err(role_not_found(&id));
    }
    let model = state.security().model();
    model
        .roles
        .get(&id)
        .map(|role| axum::Json(RoleXo::from(role)))
        .ok_or_else(|| role_not_found(&id))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<SourceQuery>,
    Json(body): Json<RoleRequest>,
) -> Result<axum::Json<RoleXo>, ApiError> {
    let subject = caller.require("astrum:roles:create")?.clone();
    if source(&query)? == Source::External {
        return Err(ApiError::BadRequest(format!(
            "roles of the source {EXTERNAL_SOURCE} come from the directory and cannot be created here; create a {DEFAULT_SOURCE} role with the group name as its id"
        )));
    }
    let role = check_body(body)?;
    let model = state.security().model();
    if model.roles.contains_key(&role.id) {
        return Err(already_exists());
    }
    check_references(&model, &role)?;
    check_privilege_grants(&model, &subject, &role.privileges)?;
    check_grants(&model, &subject, &role.roles)?;

    let description = role.description.clone().unwrap_or_else(|| role.id.clone());
    let inserted = security_role::Entity::insert(security_role::ActiveModel {
        id: ActiveValue::Set(role.id.clone()),
        source: ActiveValue::Set(DEFAULT_SOURCE.to_owned()),
        name: ActiveValue::Set(role.name.clone()),
        description: ActiveValue::Set(description),
        privileges: ActiveValue::Set(json!(role.privileges)),
        roles: ActiveValue::Set(json!(role.roles)),
        read_only: ActiveValue::Set(false),
        created_at: ActiveValue::Set(now()),
        updated_at: ActiveValue::Set(now()),
    })
    .exec(state.db().conn())
    .await;
    match inserted {
        Err(error) if unique_violation(&error) => return Err(already_exists()),
        other => {
            other?;
        }
    }

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "created", &role.id)
            .with(json!({"privileges": role.privileges, "roles": role.roles})),
    )
    .await?;
    let model = state.security().model();
    model
        .roles
        .get(&role.id)
        .map(|role| axum::Json(RoleXo::from(role)))
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("the new role is missing")))
}

fn already_exists() -> ApiError {
    ApiError::invalid("id", "a role with this id already exists")
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Query(query): Query<SourceQuery>,
    Json(body): Json<RoleRequest>,
) -> Result<Response, ApiError> {
    let subject = caller.require("astrum:roles:update")?.clone();
    let external = source(&query)? == Source::External;
    let role = check_body(body)?;
    let model = state.security().model();
    let Some(current) = model.roles.get(&id).filter(|_| !external) else {
        return Err(role_not_found(&id));
    };
    if current.read_only {
        return Err(built_in(&id));
    }
    if role.id != id {
        return Ok(conflict(format!(
            "the role id {} in the body does not match the id {id} in the path",
            role.id
        )));
    }
    check_references(&model, &role)?;
    check_privilege_grants(
        &model,
        &subject,
        role.privileges
            .iter()
            .filter(|privilege| !current.privileges.contains(privilege)),
    )?;
    check_grants(
        &model,
        &subject,
        role.roles
            .iter()
            .filter(|nested| !current.roles.contains(nested)),
    )?;

    let description = role.description.clone().unwrap_or_else(|| role.id.clone());
    security_role::Entity::update(security_role::ActiveModel {
        id: ActiveValue::Unchanged(id.clone()),
        name: ActiveValue::Set(role.name.clone()),
        description: ActiveValue::Set(description),
        privileges: ActiveValue::Set(json!(role.privileges)),
        roles: ActiveValue::Set(json!(role.roles)),
        updated_at: ActiveValue::Set(now()),
        ..Default::default()
    })
    .exec(state.db().conn())
    .await?;

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", &id)
            .with(json!({"privileges": role.privileges, "roles": role.roles})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Query(query): Query<SourceQuery>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:roles:delete")?;
    let external = source(&query)? == Source::External;
    let model = state.security().model();
    let Some(current) = model.roles.get(&id).filter(|_| !external) else {
        return Err(role_not_found(&id));
    };
    if current.read_only {
        return Err(built_in(&id));
    }

    let txn = state.db().conn().begin().await?;
    for sql in [
        "UPDATE security_role SET roles = roles - $1::text, updated_at = now() WHERE roles @> jsonb_build_array($1::text)",
        "UPDATE security_user_role SET roles = roles - $1::text, updated_at = now() WHERE roles @> jsonb_build_array($1::text)",
    ] {
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [id.clone().into()],
        ))
        .await?;
    }
    security_role::Entity::delete_by_id(id.clone())
        .exec(&txn)
        .await?;
    txn.commit().await?;

    changed(&state, &caller, Event::new(DOMAIN, "deleted", &id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
