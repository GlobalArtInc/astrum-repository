use astrum_db::audit::Event;
use astrum_security::{
    EXTERNAL_CREDENTIALS, NewToken as TokenDraft, Scope, Subject, TokenError, TokenInfo, realm,
};
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::INTERNAL;
use crate::routes::customers::resolve_repositories;
use crate::{ApiError, AppState, Caller, Json};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{INTERNAL}/access-tokens"), get(list).post(create))
        .route(&format!("{INTERNAL}/access-tokens/{{id}}"), delete(remove))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenView {
    id: Uuid,
    user_id: String,
    name: String,
    hint: String,
    scope: Scope,
    created_at: String,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repositories: Option<Vec<String>>,
}

impl From<TokenInfo> for TokenView {
    fn from(token: TokenInfo) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            name: token.name,
            hint: token.hint,
            scope: token.scope,
            created_at: token.created_at.to_rfc3339(),
            expires_at: token.expires_at.map(|at| at.to_rfc3339()),
            last_used_at: token.last_used_at.map(|at| at.to_rfc3339()),
            repositories: token.repositories,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    user_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewToken {
    name: String,
    #[serde(default)]
    scope: Scope,
    #[serde(default, deserialize_with = "given")]
    expires_in_days: Option<Option<i64>>,
    repositories: Option<Vec<String>>,
}

fn given<'de, D: serde::Deserializer<'de>>(value: D) -> Result<Option<Option<i64>>, D::Error> {
    Option::<i64>::deserialize(value).map(Some)
}

fn audited(mut attributes: serde_json::Value, token: &TokenInfo) -> serde_json::Value {
    if let Some(customer) = &token.customer {
        attributes["customer"] = json!(customer);
    }
    if let Some(repositories) = &token.repositories {
        attributes["repositories"] = json!(repositories);
    }
    attributes
}

fn owner(caller: &Caller) -> Result<&Subject, ApiError> {
    let subject = caller.signed_in()?;
    if subject.realm == realm::ACCESS_TOKEN {
        return Err(ApiError::Forbidden(
            "access tokens are managed after signing in with a password".to_owned(),
        ));
    }
    Ok(subject)
}

fn own_id(subject: &Subject) -> Result<&str, ApiError> {
    subject
        .local_user_id()
        .ok_or_else(|| ApiError::Forbidden(EXTERNAL_CREDENTIALS.to_owned()))
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<ListQuery>,
) -> Result<axum::Json<Vec<TokenView>>, ApiError> {
    let subject = owner(&caller)?;
    let user = match query.user_id.as_deref() {
        Some("*") => {
            caller.require("astrum:accesstokens:read")?;
            None
        }
        Some(user) if user != subject.user_id => {
            caller.require("astrum:accesstokens:read")?;
            Some(user.to_owned())
        }
        _ => Some(own_id(subject)?.to_owned()),
    };
    let tokens = state.security().tokens(user.as_deref()).await?;
    Ok(axum::Json(
        tokens.into_iter().map(TokenView::from).collect(),
    ))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<NewToken>,
) -> Result<Response, ApiError> {
    let owner_id = own_id(owner(&caller)?)?;
    let days = body.expires_in_days.ok_or_else(|| {
        ApiError::invalid(
            "expiresInDays",
            "give the number of days or null for no expiry",
        )
    })?;
    let repositories = match body.repositories.as_deref() {
        None => None,
        Some(listed) => {
            if body.scope != Scope::Read {
                return Err(ApiError::invalid("scope", TokenError::Scope.to_string()));
            }
            let found = resolve_repositories(&state, listed).await?;
            Some(
                found
                    .into_iter()
                    .map(|repository| repository.name)
                    .collect(),
            )
        }
    };
    let created = state
        .security()
        .create_token(
            owner_id,
            TokenDraft {
                created_by: Some(owner_id.to_owned()),
                repositories,
                ..TokenDraft::personal(&body.name, body.scope, days)
            },
        )
        .await;
    let (info, token) = match created {
        Ok(created) => created,
        Err(TokenError::Name) => {
            return Err(ApiError::invalid("name", TokenError::Name.to_string()));
        }
        Err(TokenError::Days) => {
            return Err(ApiError::invalid(
                "expiresInDays",
                TokenError::Days.to_string(),
            ));
        }
        Err(TokenError::Repositories(message)) => {
            return Err(ApiError::invalid("repositories", message));
        }
        Err(
            error @ (TokenError::TooMany
            | TokenError::TooManyForCustomer
            | TokenError::Scope
            | TokenError::CustomerExpiry
            | TokenError::UnknownCustomer),
        ) => {
            return Err(ApiError::BadRequest(error.to_string()));
        }
        Err(TokenError::Internal(error)) => return Err(error.into()),
    };
    state
        .audit(
            &caller,
            Event::new("security.accesstoken", "created", &info.name).with(audited(
                json!({"id": info.id, "user": info.user_id, "scope": info.scope}),
                &info,
            )),
        )
        .await;
    let mut view = serde_json::to_value(TokenView::from(info)).map_err(ApiError::internal)?;
    view["token"] = json!(token);
    Ok((StatusCode::CREATED, axum::Json(view)).into_response())
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let subject = owner(&caller)?;
    let missing = || ApiError::not_found("there is no such access token");
    let id = Uuid::parse_str(&id).map_err(|_| missing())?;
    let token = state.security().token(id).await?.ok_or_else(missing)?;
    if token.user_id == subject.user_id {
        own_id(subject)?;
    } else if !subject.is_permitted("astrum:accesstokens:delete") {
        return Err(missing());
    }
    state.security().delete_token(id).await?;
    state
        .audit(
            &caller,
            Event::new("security.accesstoken", "deleted", &token.name).with(audited(
                json!({"id": token.id, "user": token.user_id}),
                &token,
            )),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}
