use astrum_db::audit::Event;
use astrum_replication::protocol::Capabilities;
use astrum_replication::{Client, Error, Remote};
use astrum_security::ssrf::Guard;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use sea_orm::FromQueryResult;
use serde_json::json;
use uuid::Uuid;

use super::config::{DOMAIN, Found, UPDATE, message, missing, parse_id, ssrf_settings, statement};
use crate::{ApiError, AppState, Caller};

const MAX_ECHO: usize = 200;
const CHECK: &str = "/service/rest/v1/replication/remotes/{id}/check";

pub fn routes() -> Router<AppState> {
    Router::new().route(CHECK, post(check_remote))
}

#[derive(FromQueryResult)]
struct Stored {
    name: String,
    url: String,
    token: String,
    use_trust_store: bool,
    allow_private_network: bool,
    max_concurrency: i32,
    max_bytes_per_second: Option<i64>,
}

fn internal(what: &str) -> ApiError {
    ApiError::internal(anyhow::anyhow!("{what}"))
}

async fn probe(state: &AppState, stored: &Stored) -> Result<Result<Capabilities, Error>, ApiError> {
    let Ok(token) = state.secrets().open(&stored.token) else {
        return Ok(Err(Error::Refused(
            "the stored token cannot be opened with the key of this server; enter the token again"
                .to_owned(),
        )));
    };
    let snapshot = state
        .content()
        .outbound()
        .snapshot()
        .await
        .map_err(|error| ApiError::internal(anyhow::Error::from(error)))?;
    let ssrf = ssrf_settings(state).await?;
    let guard = Guard::new(&snapshot.guard_settings(&ssrf));
    let remote = Remote {
        url: stored.url.clone(),
        use_trust_store: stored.use_trust_store,
        allow_private_network: stored.allow_private_network,
        max_concurrency: usize::try_from(stored.max_concurrency)
            .map_err(|_| internal("a stored remote has a negative concurrency"))?,
        max_bytes_per_second: stored
            .max_bytes_per_second
            .map(u64::try_from)
            .transpose()
            .map_err(|_| internal("a stored remote has a negative rate limit"))?,
    };
    let client = match Client::new(&remote, &token, &snapshot, state.secrets(), &guard) {
        Ok(client) => client,
        Err(error) => return Ok(Err(error)),
    };
    Ok(client.check().await)
}

fn echoed(error: &Error) -> String {
    error.to_string().chars().take(MAX_ECHO).collect()
}

async fn check_remote(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    caller.require(UPDATE)?;
    let id = parse_id(&id, "replication remote")?;
    let stored = Stored::find_by_statement(statement(
        "SELECT name, url, token, use_trust_store, allow_private_network, max_concurrency, \
         max_bytes_per_second FROM replication_remote WHERE id = $1",
        vec![id.into()],
    ))
    .one(state.db().conn())
    .await?
    .ok_or_else(|| missing("replication remote", id))?;
    let checked = probe(&state, &stored).await?;
    let recorded = match &checked {
        Ok(capabilities) => {
            let instance = Uuid::parse_str(&capabilities.instance_id).map_err(|_| {
                internal("a checked remote reported an instance id that is no UUID")
            })?;
            Found::find_by_statement(statement(
                "UPDATE replication_remote SET remote_instance_id = $2, remote_version = $3, \
                 checked_at = now() WHERE id = $1 AND url = $4 RETURNING id",
                vec![
                    id.into(),
                    instance.into(),
                    capabilities.version.clone().into(),
                    stored.url.clone().into(),
                ],
            ))
            .one(state.db().conn())
            .await?
        }
        Err(_) => {
            Found::find_by_statement(statement(
                "UPDATE replication_remote SET checked_at = now() \
                 WHERE id = $1 AND url = $2 RETURNING id",
                vec![id.into(), stored.url.clone().into()],
            ))
            .one(state.db().conn())
            .await?
        }
    };
    if recorded.is_none() {
        return Ok(message(
            StatusCode::CONFLICT,
            "The remote was changed or removed while it was being checked; check it again",
        ));
    }
    let (answer, kind) = match checked {
        Ok(capabilities) => (
            json!({
                "ok": true,
                "protocol": capabilities.protocol,
                "instanceId": capabilities.instance_id,
                "version": capabilities.version,
                "formats": capabilities.formats,
            }),
            None,
        ),
        Err(error) => (
            json!({"ok": false, "error": {"kind": error.kind(), "message": echoed(&error)}}),
            Some(error.kind()),
        ),
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "remote.checked", &stored.name).with(json!({
                "id": id,
                "name": stored.name,
                "ok": kind.is_none(),
                "kind": kind,
            })),
        )
        .await;
    Ok((StatusCode::OK, axum::Json(answer)).into_response())
}
