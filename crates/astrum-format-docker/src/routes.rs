use std::collections::HashMap;
use std::sync::Arc;

use astrum_api::{AppState, Caller};
use astrum_content::ContentRequest;
use axum::Router;
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use serde_json::json;

use crate::error::RegistryError;
use crate::{Docker, ID, Mount, admit, auth, insert, json_response, link, paginate};

pub fn routes() -> Router<AppState> {
    let docker = Arc::new(Docker::default());
    let catalog_docker = docker.clone();
    Router::new()
        .route("/v2", get(auth::ping))
        .route("/v2/", get(auth::ping))
        .route("/v2/token", get(auth::token).post(auth::oauth))
        .route(
            "/v2/_catalog",
            get(
                move |state: State<AppState>,
                      caller: Caller,
                      headers: HeaderMap,
                      uri: Uri,
                      query: Query<HashMap<String, String>>| {
                    let docker = catalog_docker.clone();
                    async move { catalog(docker, state, caller, headers, uri, query).await }
                },
            ),
        )
        .route(
            "/v2/{*rest}",
            any(
                move |state: State<AppState>,
                      caller: Caller,
                      rest: Path<String>,
                      request: Request| {
                    let docker = docker.clone();
                    async move { routed(docker, state, caller, rest, request).await }
                },
            ),
        )
}

fn path_enabled(repository: &astrum_repo::config::Repository) -> bool {
    repository.format.id == ID
        && repository
            .attributes
            .docker
            .as_ref()
            .and_then(|docker| docker.path_enabled)
            .unwrap_or(false)
}

fn unreachable_name(name: &str) -> RegistryError {
    RegistryError::new(
        StatusCode::NOT_FOUND,
        "NAME_UNKNOWN",
        format!("repository {name} is not a docker repository reachable by path"),
    )
}

async fn routed(
    docker: Arc<Docker>,
    State(state): State<AppState>,
    caller: Caller,
    Path(rest): Path<String>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let base = astrum_api::routes::repositories::base_url(&parts.headers, &parts.uri);
    let model = state.security().model();
    let (name, remainder) = rest.split_once('/').unwrap_or((rest.as_str(), ""));
    let found = match state.content().repository(name).await {
        Ok(found) => found,
        Err(error) => {
            return auth::with_challenge(
                RegistryError::from(error).into_response(),
                &model,
                &base,
                None,
            );
        }
    };
    if !found.as_deref().is_some_and(path_enabled) {
        return auth::with_challenge(unreachable_name(name).into_response(), &model, &base, None);
    }
    let context =
        astrum_api::routes::content::context(&state, caller, name, &parts.headers, &parts.uri)
            .await;
    let ctx = match context {
        Ok((ctx, _)) => ctx,
        Err(error) => {
            return auth::with_challenge(
                RegistryError::from(error).into_response(),
                &model,
                &base,
                None,
            );
        }
    };
    let rule_path = format!("/v2/{remainder}");
    match state
        .content()
        .routing_allows(&ctx.repository, &rule_path)
        .await
    {
        Ok(true) => {}
        Ok(false) => return astrum_api::routes::content::blocked(),
        Err(error) => return RegistryError::from(error).into_response(),
    }
    let ctx = ctx.with_request_path(&rule_path);
    let request = match ContentRequest::new(
        parts.method,
        &format!("/{remainder}"),
        parts.uri.query().map(str::to_owned),
        parts.headers,
        body,
    ) {
        Ok(request) => request,
        Err(error) => {
            return auth::with_challenge(
                RegistryError::from(error).into_response(),
                &model,
                &base,
                None,
            );
        }
    };
    docker
        .serve(ctx, request, Mount::Path(name.to_owned()))
        .await
}

async fn catalog(
    docker: Arc<Docker>,
    State(state): State<AppState>,
    caller: Caller,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let base = astrum_api::routes::repositories::base_url(&headers, &uri);
    let model = state.security().model();
    let result = async {
        if caller.subject().is_none() {
            return Err(RegistryError::unauthorized());
        }
        let repositories = astrum_repo::store::list(state.db().conn())
            .await
            .map_err(|error| RegistryError::from(astrum_content::ContentError::internal(error)))?;
        let mut names = Vec::new();
        for repository in repositories {
            if !repository.online || !path_enabled(&repository) {
                continue;
            }
            let name = repository.name.clone();
            let ctx = admit(state.content().context(
                Arc::new(repository),
                model.clone(),
                caller.0.clone(),
                base.clone(),
                String::new(),
            ));
            if ctx.subject.is_none() {
                continue;
            }
            for image in docker.images_of(&ctx).await? {
                names.push(format!("{name}/{image}"));
            }
        }
        let (repositories, next) = paginate(
            names,
            params.get("n").map(String::as_str),
            params.get("last").map(String::as_str),
        )?;
        let mut response = json_response(
            StatusCode::OK,
            "application/json",
            &json!({ "repositories": repositories }),
        );
        if let Some(link) = link("/v2/_catalog", next) {
            insert(response.headers_mut(), header::LINK.as_str(), &link);
        }
        Ok(response)
    }
    .await;
    let response = result.unwrap_or_else(IntoResponse::into_response);
    auth::with_challenge(response, &model, &base, Some("registry:catalog:*"))
}
