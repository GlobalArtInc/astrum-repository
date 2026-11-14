use astrum_db::audit::Event;
use astrum_repo::RepositoryType;
use astrum_scan::queue::{self, Selection};
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::Deserialize;
use serde_json::json;

use crate::error::ApiError;
use crate::{AppState, Caller};

pub const SCAN: &str = "/service/rest/v1/security/scan";
pub const DOMAIN: &str = "scan";
pub const REASON: &str = "request";
pub const UPDATE: &str = "astrum:policies:update";

pub fn routes() -> Router<AppState> {
    Router::new().route(SCAN, post(scan))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    repository: String,
    component_id: Option<String>,
}

async fn scan(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<Request>, ApiError>,
) -> Result<Response, ApiError> {
    caller.require(UPDATE)?;
    let crate::Json(request) = body?;
    let repository = astrum_repo::store::find(state.db().conn(), &request.repository)
        .await?
        .filter(|found| found.name == request.repository)
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "the repository {} does not exist",
                request.repository
            ))
        })?;
    if repository.kind == RepositoryType::Group {
        return Err(ApiError::BadRequest(format!(
            "{} is a group repository and holds no components of its own",
            repository.name
        )));
    }
    let Some(policy) = repository.policy_name() else {
        return Err(ApiError::BadRequest(format!(
            "{} has no content policy, so there is nothing to scan for",
            repository.name
        )));
    };
    let queued = match &request.component_id {
        Some(id) => {
            let number = astrum_content::ids::decode(id)
                .filter(|(named, _)| named == &repository.name)
                .map(|(_, number)| number)
                .ok_or_else(|| ApiError::not_found("the component does not exist"))?;
            Some(
                queue::enqueue_component(state.db().conn(), &repository.name, number, REASON)
                    .await?
                    .ok_or_else(|| ApiError::not_found("the component does not exist"))?,
            )
        }
        None => {
            enqueue_detached(&state, &repository.name, REASON);
            None
        }
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "queued", &repository.name).with(json!({
                "policy": policy,
                "component": request.component_id,
                "queued": queued,
                "started": queued.is_none(),
            })),
        )
        .await;
    let body = match queued {
        Some(queued) => json!({"repository": repository.name, "queued": queued}),
        None => json!({"repository": repository.name, "started": true}),
    };
    Ok((StatusCode::ACCEPTED, axum::Json(body)).into_response())
}

pub(crate) fn enqueue_detached(state: &AppState, repository: &str, reason: &'static str) {
    let db = state.db().conn().clone();
    let repository = repository.to_owned();
    tokio::spawn(async move {
        let queued = queue::enqueue_repository(
            &db,
            &repository,
            reason,
            Selection::Every,
            queue::CHUNK,
            &|| false,
        )
        .await;
        match queued {
            Ok(done) => tracing::info!(
                repository,
                reason,
                components = done.components,
                queued = done.queued,
                "the components of a repository were queued for a scan"
            ),
            Err(error) => tracing::error!(
                repository,
                reason,
                %error,
                "the components of a repository were not queued for a scan; run the task scan.repository"
            ),
        }
    });
}
