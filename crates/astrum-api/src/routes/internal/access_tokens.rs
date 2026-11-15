use astrum_db::audit::Event;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::{
    EXTERNAL_CREDENTIALS, EXTERNAL_TOKENS_OFF, NewToken as TokenDraft, Scope, Subject, TokenError,
    TokenInfo, ldap, oidc, realm,
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
    source: String,
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
            source: token.source,
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

struct OwnerKey {
    source: String,
    user_id: String,
    external_subject: Option<String>,
}

fn own_key(state: &AppState, subject: &Subject) -> Result<OwnerKey, ApiError> {
    if let Some(user_id) = subject.local_user_id() {
        return Ok(OwnerKey {
            source: DEFAULT_SOURCE.to_owned(),
            user_id: user_id.to_owned(),
            external_subject: None,
        });
    }
    let security = state.security();
    if !security.external_tokens_enabled() {
        return Err(ApiError::Forbidden(EXTERNAL_TOKENS_OFF.to_owned()));
    }
    let external = || ApiError::Forbidden(EXTERNAL_CREDENTIALS.to_owned());
    match subject.source.as_str() {
        oidc::SOURCE => {
            if security.model().local_id_taken(&subject.user_id) {
                return Err(external());
            }
            let bound = security
                .oidc_subject_of(&subject.user_id)
                .ok_or_else(external)?;
            Ok(OwnerKey {
                source: oidc::SOURCE.to_owned(),
                user_id: subject.user_id.clone(),
                external_subject: Some(bound),
            })
        }
        ldap::SOURCE => Ok(OwnerKey {
            source: ldap::SOURCE.to_owned(),
            user_id: subject.user_id.clone(),
            external_subject: None,
        }),
        _ => Err(external()),
    }
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<ListQuery>,
) -> Result<axum::Json<Vec<TokenView>>, ApiError> {
    let subject = owner(&caller)?;
    let security = state.security();
    let tokens = match query.user_id.as_deref() {
        Some("*") => {
            caller.require("astrum:accesstokens:read")?;
            security.tokens(None).await?
        }
        Some(user) if user != subject.user_id => {
            caller.require("astrum:accesstokens:read")?;
            security.tokens(Some(user)).await?
        }
        _ => {
            let key = own_key(&state, subject)?;
            security.tokens_of(&key.source, &key.user_id).await?
        }
    };
    Ok(axum::Json(
        tokens.into_iter().map(TokenView::from).collect(),
    ))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<NewToken>,
) -> Result<Response, ApiError> {
    let key = own_key(&state, owner(&caller)?)?;
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
            &key.user_id,
            TokenDraft {
                source: key.source,
                external_subject: key.external_subject,
                created_by: Some(key.user_id.clone()),
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
            | TokenError::UnknownCustomer
            | TokenError::UnknownSource
            | TokenError::MissingSubject
            | TokenError::ExternalCustomer
            | TokenError::ExternalOff),
        ) => {
            return Err(ApiError::BadRequest(error.to_string()));
        }
        Err(TokenError::Internal(error)) => return Err(error.into()),
    };
    state
        .audit(
            &caller,
            Event::new("security.accesstoken", "created", &info.name).with(audited(
                json!({"id": info.id, "user": info.user_id, "source": info.source, "scope": info.scope}),
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
    let security = state.security();
    let found = security.token(id).await?.ok_or_else(missing)?;
    let own = match found.user_id == subject.user_id {
        true => {
            let key = own_key(&state, subject)?;
            security.token_of(&key.source, &key.user_id, id).await?
        }
        false => None,
    };
    let token = match own {
        Some(token) => token,
        None if subject.is_permitted("astrum:accesstokens:delete") => found,
        None => return Err(missing()),
    };
    security.delete_token(id).await?;
    state
        .audit(
            &caller,
            Event::new("security.accesstoken", "deleted", &token.name).with(audited(
                json!({"id": token.id, "user": token.user_id, "source": token.source}),
                &token,
            )),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}
