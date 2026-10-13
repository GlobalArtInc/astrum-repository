use std::collections::{BTreeMap, HashSet};

use astrum_db::audit::Event;
use astrum_repo::config::Repository;
use astrum_repo::routing::{self, RoutingRule, RuleRequest};
use astrum_repo::{RepositoryType, WriteError, Writer, store};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{ApiError, FieldError};
use crate::{AppState, Caller};

pub const PUBLIC: &str = "/service/rest/v1/routing-rules";
pub const INTERNAL: &str = "/service/rest/internal/ui/routing-rules";
pub const DOMAIN: &str = "RoutingRule";
pub const ADMIN: &str = "astrum:*";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(PUBLIC, get(list).post(create))
        .route(
            "/service/rest/v1/routing-rules/{name}",
            get(read).put(update).delete(remove),
        )
        .route(INTERNAL, get(list_internal))
        .route("/service/rest/internal/ui/routing-rules/test", post(test))
        .route(
            "/service/rest/internal/ui/routing-rules/preview",
            get(preview),
        )
        .route(
            "/service/rest/internal/ui/routing-rules/{name}",
            get(read_internal),
        )
}

fn missing(name: &str) -> ApiError {
    ApiError::not_found(format!(
        "Did not find a routing rule with the name '{name}'"
    ))
}

fn write_error(error: WriteError, name: &str) -> ApiError {
    match error {
        WriteError::Invalid(problems) => ApiError::Invalid(
            problems
                .into_iter()
                .map(|problem| FieldError {
                    id: problem.id,
                    message: problem.message,
                })
                .collect(),
        ),
        WriteError::Refused(message) => ApiError::BadRequest(message),
        WriteError::NotFound(_) => missing(name),
        WriteError::Store(error) => ApiError::internal(error),
    }
}

fn validated(request: RuleRequest) -> Result<RoutingRule, ApiError> {
    routing::validate(request).map_err(|problems| write_error(WriteError::Invalid(problems), ""))
}

async fn rules(state: &AppState) -> Result<Vec<RoutingRule>, ApiError> {
    routing::list(state.db().conn())
        .await
        .map_err(ApiError::internal)
}

async fn existing(state: &AppState, name: &str) -> Result<RoutingRule, ApiError> {
    routing::find(state.db().conn(), name)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| missing(name))
}

async fn repositories(state: &AppState) -> Result<Vec<Repository>, ApiError> {
    store::list(state.db().conn())
        .await
        .map_err(ApiError::internal)
}

fn users<'a>(repositories: &'a [Repository], rule: &str) -> Vec<&'a str> {
    repositories
        .iter()
        .filter(|repository| {
            repository.kind != RepositoryType::Hosted
                && repository.attributes.routing_rule_name.as_deref() == Some(rule)
        })
        .map(|repository| repository.name.as_str())
        .collect()
}

fn internal_document(rule: &RoutingRule, repositories: &[Repository]) -> Value {
    let names = users(repositories, &rule.name);
    json!({
        "name": rule.name,
        "description": rule.description,
        "mode": rule.mode,
        "matchers": rule.matchers,
        "assignedRepositoryCount": names.len(),
        "assignedRepositoryNames": names,
    })
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<Json<Vec<RoutingRule>>, ApiError> {
    caller.require(ADMIN)?;
    Ok(Json(rules(&state).await?))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Json<RoutingRule>, ApiError> {
    caller.require(ADMIN)?;
    Ok(Json(existing(&state, &name).await?))
}

async fn list_internal(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<Json<Vec<Value>>, ApiError> {
    caller.require(ADMIN)?;
    let repositories = repositories(&state).await?;
    Ok(Json(
        rules(&state)
            .await?
            .iter()
            .map(|rule| internal_document(rule, &repositories))
            .collect(),
    ))
}

async fn read_internal(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(ADMIN)?;
    let rule = existing(&state, &name).await?;
    let repositories = repositories(&state).await?;
    Ok(Json(internal_document(&rule, &repositories)))
}

fn audit_attributes(rule: &RoutingRule) -> Value {
    json!({"mode": rule.mode, "matchers": rule.matchers, "description": rule.description})
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<RuleRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(ADMIN)?;
    let crate::Json(request) = body?;
    let rule = validated(request)?;
    let writer = Writer::begin(state.db().conn())
        .await
        .map_err(ApiError::internal)?;
    let rule = writer
        .create_rule(rule)
        .await
        .map_err(|error| write_error(error, ""))?;
    state.content().forget_routing_rules();
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "created", &rule.name).with(audit_attributes(&rule)),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<crate::Json<RuleRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(ADMIN)?;
    let crate::Json(mut request) = body?;
    if request
        .name
        .as_deref()
        .is_none_or(|given| given.trim().is_empty())
    {
        request.name = Some(name.clone());
    }
    let rule = validated(request)?;
    let writer = Writer::begin(state.db().conn())
        .await
        .map_err(ApiError::internal)?;
    let (rule, renamed) = writer
        .update_rule(&name, rule)
        .await
        .map_err(|error| write_error(error, &name))?;
    state.content().forget_routing_rules();
    if !renamed.is_empty() {
        state.security().reload().await?;
    }
    let mut attributes = audit_attributes(&rule);
    if rule.name != name {
        attributes["previousName"] = json!(name);
        attributes["repositories"] = json!(renamed);
    }
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &rule.name).with(attributes),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Response, ApiError> {
    caller.require(ADMIN)?;
    let writer = Writer::begin(state.db().conn())
        .await
        .map_err(ApiError::internal)?;
    match writer.delete_rule(&name).await {
        Ok(()) => {}
        Err(WriteError::Refused(message)) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(json!({"id": "*", "message": message})),
            )
                .into_response());
        }
        Err(error) => return Err(write_error(error, &name)),
    }
    state.content().forget_routing_rules();
    state
        .audit(&caller, Event::new(DOMAIN, "deleted", &name))
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestRequest {
    mode: Option<String>,
    #[serde(default)]
    matchers: Vec<String>,
    path: Option<String>,
}

async fn test(
    caller: Caller,
    body: Result<crate::Json<TestRequest>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    caller.require(ADMIN)?;
    let crate::Json(request) = body?;
    let mode = request
        .mode
        .as_deref()
        .and_then(routing::parse_mode)
        .ok_or_else(|| ApiError::invalid("PARAMETER mode", "must be ALLOW or BLOCK"))?;
    let path = routing::request_path(request.path.as_deref().unwrap_or("").trim());
    let (allowed, matchers) = routing::evaluate(mode, &request.matchers, &path);
    Ok(Json(json!({"allowed": allowed, "matchers": matchers})))
}

#[derive(Default, Deserialize)]
struct PreviewQuery {
    path: Option<String>,
    filter: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Groups,
    Proxies,
}

async fn preview(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<PreviewQuery>,
) -> Result<Json<Value>, ApiError> {
    caller.require(ADMIN)?;
    let path = query
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| ApiError::invalid("PARAMETER path", "must not be empty"))?;
    let path = routing::request_path(path);
    let filter = match query
        .filter
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        None | Some("") | Some("all") => Filter::All,
        Some("groups") => Filter::Groups,
        Some("proxies") => Filter::Proxies,
        Some(_) => {
            return Err(ApiError::invalid(
                "PARAMETER filter",
                "must be all, groups or proxies",
            ));
        }
    };
    let repositories = repositories(&state).await?;
    let matchers = routing::matchers(state.db().conn())
        .await
        .map_err(ApiError::internal)?;
    let tree = Tree {
        by_name: repositories
            .iter()
            .map(|repository| (repository.name.to_ascii_lowercase(), repository))
            .collect(),
        matchers: &matchers,
        path: &path,
    };
    let mut children = Vec::new();
    if filter != Filter::Proxies {
        for group in repositories
            .iter()
            .filter(|repository| repository.kind == RepositoryType::Group)
        {
            children.push(tree.node(group, &mut HashSet::new(), true));
        }
    }
    if filter != Filter::Groups {
        for proxy in repositories
            .iter()
            .filter(|repository| repository.kind == RepositoryType::Proxy)
        {
            children.push(tree.node(proxy, &mut HashSet::new(), false));
        }
    }
    Ok(Json(json!({
        "repository": null,
        "type": null,
        "format": null,
        "rule": null,
        "allowed": null,
        "expanded": true,
        "expandable": true,
        "children": children,
    })))
}

struct Tree<'a> {
    by_name: BTreeMap<String, &'a Repository>,
    matchers: &'a std::collections::HashMap<String, routing::Matcher>,
    path: &'a str,
}

impl Tree<'_> {
    fn allowed(&self, repository: &Repository) -> bool {
        if repository.kind == RepositoryType::Hosted {
            return true;
        }
        repository
            .attributes
            .routing_rule_name
            .as_deref()
            .and_then(|rule| self.matchers.get(rule))
            .is_none_or(|matcher| matcher.allows(self.path))
    }

    fn node(&self, repository: &Repository, seen: &mut HashSet<String>, top: bool) -> Value {
        let rule = match repository.kind {
            RepositoryType::Hosted => None,
            _ => repository.attributes.routing_rule_name.clone(),
        };
        let children: Option<Vec<Value>> = match repository.kind {
            RepositoryType::Group if seen.insert(repository.name.to_ascii_lowercase()) => Some(
                repository
                    .group()
                    .map(|group| group.members().to_vec())
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|member| self.by_name.get(&member.to_ascii_lowercase()))
                    .map(|member| self.node(member, seen, false))
                    .collect(),
            ),
            _ => None,
        };
        let expandable = children
            .as_ref()
            .is_some_and(|children| !children.is_empty());
        json!({
            "repository": repository.name,
            "type": repository.kind.as_str(),
            "format": repository.format.id,
            "rule": rule,
            "allowed": self.allowed(repository),
            "expanded": top && expandable,
            "expandable": expandable,
            "children": children,
        })
    }
}
