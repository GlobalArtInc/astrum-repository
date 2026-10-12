use std::collections::BTreeMap;

use astrum_db::audit::Event;
use astrum_db::entity::{content_selector, repository, security_privilege};
use astrum_security::permission::Permission;
use astrum_security::{Action, FieldKind, Privilege, PrivilegeKind};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Statement,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{SECURITY, changed, conflict, now, unique_violation, validate};
use crate::{ApiError, AppState, Caller, Json, Validation};

const DOMAIN: &str = "security.privilege";
const SCRIPT: &str = "script";
const ANY: &str = "*";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{SECURITY}/privileges"), get(list))
        .route(
            &format!("{SECURITY}/privileges/{{name}}"),
            get(read).delete(remove).post(create),
        )
        .route(
            &format!("{SECURITY}/privileges/{{kind}}/{{name}}"),
            put(update),
        )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrivilegeXo {
    #[serde(rename = "type")]
    kind: &'static str,
    name: String,
    description: String,
    read_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    actions: Option<Vec<Action>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_selector: Option<String>,
}

impl From<&Privilege> for PrivilegeXo {
    fn from(privilege: &Privilege) -> Self {
        let mut xo = PrivilegeXo {
            kind: privilege.kind.type_id(),
            name: privilege.id.clone(),
            description: privilege.description.clone(),
            read_only: privilege.read_only,
            actions: None,
            domain: None,
            pattern: None,
            format: None,
            repository: None,
            content_selector: None,
        };
        match &privilege.kind {
            PrivilegeKind::Application { domain, actions } => {
                xo.domain = Some(domain.clone());
                xo.actions = Some(actions.clone());
            }
            PrivilegeKind::Wildcard { pattern } => xo.pattern = Some(pattern.clone()),
            PrivilegeKind::RepositoryView {
                format,
                repository,
                actions,
            }
            | PrivilegeKind::RepositoryAdmin {
                format,
                repository,
                actions,
            } => {
                xo.format = Some(format.clone());
                xo.repository = Some(repository.clone());
                xo.actions = Some(actions.clone());
            }
            PrivilegeKind::RepositoryContentSelector {
                content_selector,
                format,
                repository,
                actions,
            } => {
                xo.content_selector = Some(content_selector.clone());
                xo.format = Some(format.clone());
                xo.repository = Some(repository.clone());
                xo.actions = Some(actions.clone());
            }
        }
        xo
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrivilegeRequest {
    name: Option<String>,
    description: Option<String>,
    actions: Option<Vec<String>>,
    domain: Option<String>,
    pattern: Option<String>,
    format: Option<String>,
    repository: Option<String>,
    content_selector: Option<String>,
}

struct Checked {
    name: String,
    description: String,
    kind: PrivilegeKind,
}

fn not_found(name: &str) -> ApiError {
    ApiError::not_found(format!("Privilege '{name}' not found."))
}

fn script_refused() -> ApiError {
    ApiError::BadRequest(
        "script privileges are not supported: Astrum Repository has no script API".to_owned(),
    )
}

fn built_in(name: &str) -> ApiError {
    ApiError::BadRequest(format!(
        "the privilege {name} is built in and cannot be changed or deleted"
    ))
}

fn actions(validation: &mut Validation, values: Option<Vec<String>>) -> Vec<Action> {
    let values = values.unwrap_or_default();
    if values.is_empty() {
        validation.add("actions", "must not be empty");
    }
    let mut actions = Vec::new();
    for value in values {
        match Action::parse(&value) {
            Some(action) if !actions.contains(&action) => actions.push(action),
            Some(_) => {}
            None => validation.add(
                "actions",
                format!(
                    "{value} is not an action; use {}",
                    Action::ALL_ACTIONS.map(Action::as_str).join(", ")
                ),
            ),
        }
    }
    actions
}

impl PrivilegeRequest {
    fn value(&self, id: &str) -> Option<&str> {
        match id {
            "domain" => self.domain.as_deref(),
            "pattern" => self.pattern.as_deref(),
            "format" => self.format.as_deref(),
            "repository" => self.repository.as_deref(),
            "contentSelector" => self.content_selector.as_deref(),
            _ => None,
        }
    }
}

fn check_body(kind: &str, body: PrivilegeRequest) -> Result<Checked, ApiError> {
    let fields = PrivilegeKind::fields(kind);
    if fields.is_empty() {
        return Err(ApiError::not_found(format!("no privilege type {kind}")));
    }
    let mut validation = Validation::new();
    let name = body.name.clone().unwrap_or_default();
    if name.is_empty() {
        validation.add("name", "must not be empty");
    } else if !validate::name(&name) {
        validation.add("name", validate::NAME_RULE);
    }
    let mut values: BTreeMap<&str, String> = BTreeMap::new();
    let mut granted = Vec::new();
    for field in fields {
        if field.kind == FieldKind::Actions {
            granted = actions(&mut validation, body.actions.clone());
            continue;
        }
        let value = body.value(field.id).unwrap_or_default().trim().to_owned();
        if field.required && value.is_empty() {
            validation.add(field.id, "must not be empty");
        }
        if field.kind == FieldKind::Permission
            && !value.is_empty()
            && let Err(error) = Permission::parse(&value)
        {
            validation.add(field.id, error.to_string());
        }
        values.insert(field.id, value);
    }
    let mut value = |id: &str| values.remove(id).unwrap_or_default();
    let kind = match kind {
        "application" => PrivilegeKind::Application {
            domain: value("domain"),
            actions: granted,
        },
        "wildcard" => PrivilegeKind::Wildcard {
            pattern: value("pattern"),
        },
        "repository-view" => PrivilegeKind::RepositoryView {
            format: value("format"),
            repository: value("repository"),
            actions: granted,
        },
        "repository-admin" => PrivilegeKind::RepositoryAdmin {
            format: value("format"),
            repository: value("repository"),
            actions: granted,
        },
        _ => PrivilegeKind::RepositoryContentSelector {
            content_selector: value("contentSelector"),
            format: value("format"),
            repository: value("repository"),
            actions: granted,
        },
    };
    validation.finish()?;
    Ok(Checked {
        name,
        description: body.description.unwrap_or_default(),
        kind,
    })
}

async fn check_targets(state: &AppState, kind: &mut PrivilegeKind) -> Result<(), ApiError> {
    let type_id = kind.type_id();
    let allowed = PrivilegeKind::allowed_actions(type_id);
    let refused: Vec<&str> = kind
        .actions()
        .iter()
        .filter(|action| !allowed.contains(action))
        .map(|action| action.as_str())
        .collect();
    if !refused.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "a privilege of type {type_id} cannot use the action(s) {}",
            refused.join(", ")
        )));
    }
    let (format, repository) = match kind {
        PrivilegeKind::RepositoryView {
            format, repository, ..
        }
        | PrivilegeKind::RepositoryAdmin {
            format, repository, ..
        }
        | PrivilegeKind::RepositoryContentSelector {
            format, repository, ..
        } => (format, repository),
        _ => return Ok(()),
    };
    if format != ANY && astrum_repo::format(format).is_none() {
        return Err(ApiError::BadRequest(format!(
            "the format {format} does not exist"
        )));
    }
    if repository != ANY {
        let found = repository::Entity::find()
            .filter(
                Expr::expr(Func::lower(Expr::col(repository::Column::Name)))
                    .eq(repository.to_lowercase()),
            )
            .one(state.db().conn())
            .await?
            .ok_or_else(|| {
                ApiError::BadRequest(format!("the repository {repository} does not exist"))
            })?;
        if format != ANY && *format != found.format {
            return Err(ApiError::BadRequest(format!(
                "the repository {} has the format {}, not {format}",
                found.name, found.format
            )));
        }
        *repository = found.name;
    }
    if let PrivilegeKind::RepositoryContentSelector {
        content_selector, ..
    } = kind
        && content_selector::Entity::find_by_id(content_selector.clone())
            .one(state.db().conn())
            .await?
            .is_none()
    {
        return Err(ApiError::BadRequest(format!(
            "the content selector {content_selector} does not exist"
        )));
    }
    Ok(())
}

fn check_grant(caller: &Caller, name: &str, kind: &PrivilegeKind) -> Result<(), ApiError> {
    let permitted = caller.subject().is_some_and(|subject| {
        kind.effective_permission()
            .is_some_and(|permission| subject.permits(&permission))
    });
    match permitted {
        true => Ok(()),
        false => Err(ApiError::Forbidden(format!(
            "you cannot create or change the privilege {name}: it gives permissions you do not have"
        ))),
    }
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<PrivilegeXo>>, ApiError> {
    caller.require("astrum:privileges:read")?;
    let model = state.security().model();
    Ok(axum::Json(
        model.privileges.values().map(PrivilegeXo::from).collect(),
    ))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<PrivilegeXo>, ApiError> {
    caller.require("astrum:privileges:read")?;
    let model = state.security().model();
    model
        .privileges
        .get(&name)
        .map(|privilege| axum::Json(PrivilegeXo::from(privilege)))
        .ok_or_else(|| not_found(&name))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Path(kind): Path<String>,
    Json(body): Json<PrivilegeRequest>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:privileges:create")?;
    if kind == SCRIPT {
        return Err(script_refused());
    }
    if !PrivilegeKind::TYPES.contains(&kind.as_str()) {
        return Err(ApiError::not_found(format!("no privilege type {kind}")));
    }
    let mut privilege = check_body(&kind, body)?;
    if state
        .security()
        .model()
        .privileges
        .contains_key(&privilege.name)
    {
        return Err(already_exists());
    }
    check_targets(&state, &mut privilege.kind).await?;
    check_grant(&caller, &privilege.name, &privilege.kind)?;

    let inserted = security_privilege::Entity::insert(security_privilege::ActiveModel {
        id: ActiveValue::Set(privilege.name.clone()),
        kind: ActiveValue::Set(privilege.kind.type_id().to_owned()),
        description: ActiveValue::Set(privilege.description.clone()),
        properties: ActiveValue::Set(privilege.kind.properties()),
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
        Event::new(DOMAIN, "created", &privilege.name).with(audit(&privilege.kind)),
    )
    .await?;
    Ok(StatusCode::CREATED)
}

fn already_exists() -> ApiError {
    ApiError::invalid("name", "a privilege with this name already exists")
}

fn audit(kind: &PrivilegeKind) -> serde_json::Value {
    let mut attributes = kind.properties();
    attributes["type"] = json!(kind.type_id());
    attributes
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path((kind, name)): Path<(String, String)>,
    Json(body): Json<PrivilegeRequest>,
) -> Result<Response, ApiError> {
    caller.require("astrum:privileges:update")?;
    if kind == SCRIPT {
        return Err(script_refused());
    }
    if !PrivilegeKind::TYPES.contains(&kind.as_str()) {
        return Err(ApiError::not_found(format!("no privilege type {kind}")));
    }
    let mut privilege = check_body(&kind, body)?;
    let model = state.security().model();
    let Some(current) = model.privileges.get(&name) else {
        return Err(not_found(&name));
    };
    if current.read_only {
        return Err(built_in(&name));
    }
    if privilege.name != name {
        return Ok(conflict(format!(
            "the privilege name {} in the body does not match the name {name} in the path",
            privilege.name
        )));
    }
    if current.kind.type_id() != kind {
        return Ok(conflict(format!(
            "the privilege {name} is of type {} and cannot be changed through the {kind} endpoint",
            current.kind.type_id()
        )));
    }
    check_targets(&state, &mut privilege.kind).await?;
    check_grant(&caller, &name, &privilege.kind)?;

    security_privilege::Entity::update(security_privilege::ActiveModel {
        id: ActiveValue::Unchanged(name.clone()),
        description: ActiveValue::Set(privilege.description.clone()),
        properties: ActiveValue::Set(privilege.kind.properties()),
        updated_at: ActiveValue::Set(now()),
        ..Default::default()
    })
    .exec(state.db().conn())
    .await?;

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", &name).with(audit(&privilege.kind)),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:privileges:delete")?;
    let model = state.security().model();
    let Some(current) = model.privileges.get(&name) else {
        return Err(not_found(&name));
    };
    if current.read_only {
        return Err(built_in(&name));
    }

    let txn = state.db().conn().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE security_role SET privileges = privileges - $1::text, updated_at = now() WHERE privileges @> jsonb_build_array($1::text)",
        [name.clone().into()],
    ))
    .await?;
    security_privilege::Entity::delete_by_id(name.clone())
        .exec(&txn)
        .await?;
    txn.commit().await?;

    changed(&state, &caller, Event::new(DOMAIN, "deleted", &name)).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn used_by(
    db: &impl ConnectionTrait,
    selector: &str,
) -> Result<Vec<String>, ApiError> {
    let rows = security_privilege::Entity::find()
        .filter(security_privilege::Column::Kind.eq("repository-content-selector"))
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|row| row.properties["contentSelector"].as_str() == Some(selector))
        .map(|row| row.id)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_described_field_reaches_the_request() {
        let request = PrivilegeRequest {
            domain: Some("d".to_owned()),
            pattern: Some("p".to_owned()),
            format: Some("f".to_owned()),
            repository: Some("r".to_owned()),
            content_selector: Some("c".to_owned()),
            ..Default::default()
        };
        for type_id in PrivilegeKind::TYPES {
            for field in PrivilegeKind::fields(type_id) {
                if field.kind != FieldKind::Actions {
                    assert!(request.value(field.id).is_some(), "{type_id}: {}", field.id);
                }
            }
        }
    }

    #[test]
    fn required_fields_come_from_the_description() {
        let Err(ApiError::Invalid(errors)) = check_body(
            "repository-content-selector",
            PrivilegeRequest {
                name: Some("p".to_owned()),
                ..Default::default()
            },
        ) else {
            panic!("an empty privilege was accepted");
        };
        let ids: Vec<&str> = errors.iter().map(|error| error.id.as_str()).collect();
        assert_eq!(ids, ["contentSelector", "format", "repository", "actions"]);
    }
}
