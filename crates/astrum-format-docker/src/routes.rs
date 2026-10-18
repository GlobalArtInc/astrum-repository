use std::collections::HashMap;
use std::sync::Arc;

use astrum_api::domains::DomainRoute;
use astrum_api::{AppState, Caller};
use astrum_content::ContentRequest;
use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use serde_json::json;

use crate::error::RegistryError;
use crate::{Docker, ID, Mount, admit, auth, insert, json_response, link, paginate};

pub const FIRST_SUGGESTED_PORT: u16 = 8082;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Connector(pub u16);

pub fn routes() -> Router<AppState> {
    let docker = Arc::new(Docker::default());
    let catalog_docker = docker.clone();
    Router::new()
        .route(
            "/service/rest/internal/ui/docker/suggest-port",
            get(suggest_port),
        )
        .route("/v2", get(auth::ping))
        .route("/v2/", get(auth::ping))
        .route("/v2/token", get(auth::token).post(auth::oauth))
        .route(
            "/v2/_catalog",
            get(
                move |state: State<AppState>, caller: Caller, request: Request| {
                    let docker = catalog_docker.clone();
                    async move { catalog(docker, state, caller, request).await }
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

fn subdomain_of(headers: &HeaderMap, uri: &Uri) -> Option<String> {
    let base = astrum_api::routes::repositories::base_url(headers, uri);
    let host = auth::service(&base);
    let host = match host.strip_prefix('[') {
        Some(_) => return None,
        None => host.rsplit_once(':').map_or(host, |(host, _)| host),
    };
    if host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    let (label, rest) = host.split_once('.')?;
    (!label.is_empty() && !rest.is_empty()).then(|| label.to_ascii_lowercase())
}

pub async fn dedicated(
    state: &AppState,
    connector: Option<Connector>,
    route: Option<&DomainRoute>,
    headers: &HeaderMap,
    uri: &Uri,
) -> Result<Option<astrum_repo::config::Repository>, RegistryError> {
    let db = state.db().conn();
    let internal = |error| RegistryError::from(astrum_content::ContentError::internal(error));
    if let Some(DomainRoute::Root { repository, .. }) = route {
        return match astrum_repo::store::find(db, repository)
            .await
            .map_err(internal)?
        {
            Some(found) if found.format.id == ID => Ok(Some(found)),
            _ => Err(unreachable_name(repository)),
        };
    }
    if route.is_some() {
        return Ok(None);
    }
    if let Some(Connector(port)) = connector {
        return match astrum_repo::store::docker_on_port(db, port)
            .await
            .map_err(internal)?
        {
            Some(repository) => Ok(Some(repository)),
            None => Err(RegistryError::new(
                StatusCode::NOT_FOUND,
                "NAME_UNKNOWN",
                format!("no Docker repository answers on the port {port}"),
            )),
        };
    }
    let Some(subdomain) = subdomain_of(headers, uri) else {
        return Ok(None);
    };
    astrum_repo::store::docker_on_subdomain(db, &subdomain)
        .await
        .map_err(internal)
}

async fn suggest_port(State(state): State<AppState>, caller: Caller, uri: Uri) -> Response {
    if caller.subject().is_none_or(|subject| subject.anonymous) {
        return astrum_api::ApiError::Unauthorized.into_response();
    }
    let repositories = match astrum_repo::store::list(state.db().conn()).await {
        Ok(repositories) => repositories,
        Err(error) => return astrum_api::ApiError::internal(error).into_response(),
    };
    let mut used: std::collections::BTreeSet<i32> = repositories
        .iter()
        .filter_map(|repository| repository.attributes.docker.as_ref())
        .flat_map(|docker| docker.ports().map(|(_, port)| port).collect::<Vec<_>>())
        .collect();
    used.extend(state.application_port().map(i32::from));
    used.extend(
        url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
            .filter(|(key, _)| key == "exclude")
            .filter_map(|(_, value)| value.parse::<i32>().ok()),
    );
    let port =
        (i32::from(FIRST_SUGGESTED_PORT)..=i32::from(u16::MAX)).find(|port| !used.contains(port));
    match port {
        Some(port) => axum::Json(port).into_response(),
        None => astrum_api::ApiError::NotFound("every port is taken".to_owned()).into_response(),
    }
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
    let connector = parts.extensions.get::<Connector>().copied();
    let route = parts.extensions.get::<DomainRoute>().cloned();
    let dedicated = match dedicated(
        &state,
        connector,
        route.as_ref(),
        &parts.headers,
        &parts.uri,
    )
    .await
    {
        Ok(dedicated) => dedicated,
        Err(error) => return auth::with_challenge(error.into_response(), &model, &base, None),
    };
    let (name, remainder, mount) = match &dedicated {
        Some(repository) => (repository.name.as_str(), rest.as_str(), Mount::Root),
        None => {
            let (segment, remainder) = rest.split_once('/').unwrap_or((rest.as_str(), ""));
            let name = match &route {
                Some(route) => match route.repository_for(segment) {
                    Some(name) => name,
                    None => {
                        return auth::with_challenge(
                            unreachable_name(segment).into_response(),
                            &model,
                            &base,
                            None,
                        );
                    }
                },
                None => segment,
            };
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
            let reachable = match &route {
                Some(_) => found
                    .as_deref()
                    .is_some_and(|repository| repository.format.id == ID),
                None => found.as_deref().is_some_and(path_enabled),
            };
            if !reachable {
                return auth::with_challenge(
                    unreachable_name(name).into_response(),
                    &model,
                    &base,
                    None,
                );
            }
            (name, remainder, Mount::Path(segment.to_owned()))
        }
    };
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
    docker.serve(ctx, request, mount).await
}

async fn catalog(
    docker: Arc<Docker>,
    State(state): State<AppState>,
    caller: Caller,
    request: Request,
) -> Response {
    let (parts, _) = request.into_parts();
    let (headers, uri) = (&parts.headers, &parts.uri);
    let base = astrum_api::routes::repositories::base_url(headers, uri);
    let model = state.security().model();
    let connector = parts.extensions.get::<Connector>().copied();
    let route = parts.extensions.get::<DomainRoute>().cloned();
    match dedicated(&state, connector, route.as_ref(), headers, uri).await {
        Ok(Some(repository)) => {
            let context = astrum_api::routes::content::context(
                &state,
                caller,
                &repository.name,
                headers,
                uri,
            )
            .await;
            let request = context.and_then(|(ctx, _)| {
                ContentRequest::new(
                    parts.method.clone(),
                    "/_catalog",
                    uri.query().map(str::to_owned),
                    headers.clone(),
                    axum::body::Body::empty(),
                )
                .map(|request| (ctx, request))
            });
            return match request {
                Ok((ctx, request)) => docker.serve(ctx, request, Mount::Root).await,
                Err(error) => auth::with_challenge(
                    RegistryError::from(error).into_response(),
                    &model,
                    &base,
                    Some("registry:catalog:*"),
                ),
            };
        }
        Ok(None) => {}
        Err(error) => {
            return auth::with_challenge(
                error.into_response(),
                &model,
                &base,
                Some("registry:catalog:*"),
            );
        }
    }
    let params: HashMap<String, String> = uri
        .query()
        .map(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();
    let result = async {
        if caller.subject().is_none() {
            return Err(RegistryError::unauthorized());
        }
        let repositories = astrum_repo::store::list(state.db().conn())
            .await
            .map_err(|error| RegistryError::from(astrum_content::ContentError::internal(error)))?;
        let mut names = Vec::new();
        for repository in repositories {
            let listed = match &route {
                Some(route) => route.admits(&repository.name) && repository.format.id == ID,
                None => path_enabled(&repository),
            };
            if !repository.online || !listed {
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
            let shown = route
                .as_ref()
                .and_then(|route| route.segment_of(&name))
                .unwrap_or(&name)
                .to_owned();
            for image in docker.images_of(&ctx).await? {
                names.push(format!("{shown}/{image}"));
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
