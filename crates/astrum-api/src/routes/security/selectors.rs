use astrum_db::audit::Event;
use astrum_db::entity::content_selector;
use astrum_security::Selector;
use astrum_security::selector::CSEL;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use sea_orm::{ActiveValue, EntityTrait, QueryOrder};
use serde::{Deserialize, Serialize};

use super::privileges::used_by;
use super::{SECURITY, changed, now, unique_violation, validate};
use crate::{ApiError, AppState, Caller, Json, Validation};

const DOMAIN: &str = "security.contentselector";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{SECURITY}/content-selectors"),
            get(list).post(create),
        )
        .route(
            &format!("{SECURITY}/content-selectors/{{name}}"),
            get(read).put(update).delete(remove),
        )
}

#[derive(Debug, Serialize)]
struct SelectorXo {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    description: Option<String>,
    expression: String,
}

impl From<content_selector::Model> for SelectorXo {
    fn from(row: content_selector::Model) -> Self {
        Self {
            name: row.name,
            kind: row.kind,
            description: Some(row.description).filter(|description| !description.is_empty()),
            expression: row.expression,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateSelector {
    name: Option<String>,
    description: Option<String>,
    expression: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateSelector {
    description: Option<String>,
    expression: Option<String>,
}

fn not_found(name: &str) -> ApiError {
    ApiError::not_found(format!("Content selector '{name}' not found."))
}

fn check_expression(validation: &mut Validation, expression: Option<&str>) {
    match expression.filter(|expression| !expression.trim().is_empty()) {
        None => validation.add("expression", "must not be empty"),
        Some(expression) => {
            if let Err(error) = Selector::parse(expression) {
                validation.add("expression", error.to_string());
            }
        }
    }
}

async fn find(state: &AppState, name: &str) -> Result<content_selector::Model, ApiError> {
    content_selector::Entity::find_by_id(name.to_owned())
        .one(state.db().conn())
        .await?
        .ok_or_else(|| not_found(name))
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<SelectorXo>>, ApiError> {
    caller.require("astrum:selectors:read")?;
    let rows = content_selector::Entity::find()
        .order_by_asc(content_selector::Column::Name)
        .all(state.db().conn())
        .await?;
    Ok(axum::Json(rows.into_iter().map(SelectorXo::from).collect()))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<SelectorXo>, ApiError> {
    caller.require("astrum:selectors:read")?;
    Ok(axum::Json(find(&state, &name).await?.into()))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<CreateSelector>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:selectors:create")?;
    let mut validation = Validation::new();
    let name = body.name.unwrap_or_default();
    if name.is_empty() {
        validation.add("name", "must not be empty");
    } else if !validate::name(&name) {
        validation.add("name", validate::NAME_RULE);
    }
    check_expression(&mut validation, body.expression.as_deref());
    validation.finish()?;
    let expression = body.expression.unwrap_or_default();

    let inserted = content_selector::Entity::insert(content_selector::ActiveModel {
        name: ActiveValue::Set(name.clone()),
        kind: ActiveValue::Set(CSEL.to_owned()),
        description: ActiveValue::Set(body.description.unwrap_or_default()),
        expression: ActiveValue::Set(expression.clone()),
        created_at: ActiveValue::Set(now()),
        updated_at: ActiveValue::Set(now()),
    })
    .exec(state.db().conn())
    .await;
    match inserted {
        Err(error) if unique_violation(&error) => {
            return Err(ApiError::invalid(
                "name",
                "a content selector with this name already exists",
            ));
        }
        other => {
            other?;
        }
    }

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "created", &name).with(serde_json::json!({"expression": expression})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    Json(body): Json<UpdateSelector>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:selectors:update")?;
    find(&state, &name).await?;
    let mut validation = Validation::new();
    check_expression(&mut validation, body.expression.as_deref());
    validation.finish()?;
    let expression = body.expression.unwrap_or_default();

    content_selector::Entity::update(content_selector::ActiveModel {
        name: ActiveValue::Unchanged(name.clone()),
        kind: ActiveValue::Set(CSEL.to_owned()),
        description: ActiveValue::Set(body.description.unwrap_or_default()),
        expression: ActiveValue::Set(expression.clone()),
        updated_at: ActiveValue::Set(now()),
        ..Default::default()
    })
    .exec(state.db().conn())
    .await?;

    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", &name).with(serde_json::json!({"expression": expression})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:selectors:delete")?;
    find(&state, &name).await?;
    let users = used_by(state.db().conn(), &name).await?;
    if !users.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "the content selector {name} is used by the privilege(s) {} and cannot be deleted",
            users.join(", ")
        )));
    }
    content_selector::Entity::delete_by_id(name.clone())
        .exec(state.db().conn())
        .await?;

    changed(&state, &caller, Event::new(DOMAIN, "deleted", &name)).await?;
    Ok(StatusCode::NO_CONTENT)
}
