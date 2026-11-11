mod access;
mod usage;

use astrum_db::audit::Event;
use astrum_db::entity::customer;
use astrum_security::catalogue::ANONYMOUS_USER;
use astrum_security::customers::{CustomerError, CustomerFields, CustomerSummary, NewCustomer};
use astrum_security::tokens::MAX_DAYS;
use astrum_security::{NewToken, Scope, TokenError, TokenInfo, UserStatus};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{ApiError, AppState, Caller, Json, Validation};

pub(crate) use access::{audit_released_tokens, resolve_repositories};

const DOMAIN: &str = "security.customer";
const TOKEN_DOMAIN: &str = "security.accesstoken";
const MAX_ID: usize = 64;
const MAX_NAME: usize = 200;
const MAX_REFERENCE: usize = 200;
const MAX_NOTE: usize = 2000;
const READ: &str = "astrum:customers:read";
const CREATE: &str = "astrum:customers:create";
const UPDATE: &str = "astrum:customers:update";
const DELETE: &str = "astrum:customers:delete";

pub const CUSTOMERS: &str = "/service/rest/v1/customers";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(CUSTOMERS, get(list).post(create))
        .route(&format!("{CUSTOMERS}/usage.csv"), get(usage::csv))
        .route(
            &format!("{CUSTOMERS}/{{id}}"),
            get(one).put(update).delete(remove),
        )
        .route(
            &format!("{CUSTOMERS}/{{id}}/tokens"),
            get(tokens).post(issue),
        )
        .route(
            &format!("{CUSTOMERS}/{{id}}/tokens/{{token_id}}"),
            delete(revoke_one),
        )
        .route(&format!("{CUSTOMERS}/{{id}}/revoke"), post(revoke_all))
        .route(&format!("{CUSTOMERS}/{{id}}/usage"), get(usage::daily))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CustomerView {
    id: String,
    name: String,
    reference: String,
    note: String,
    disabled: bool,
    created_at: String,
    created_by: String,
    tokens: i64,
    nearest_expiry: Option<String>,
    downloads30d: i64,
    bytes30d: i64,
    last_access_at: Option<String>,
}

impl From<CustomerSummary> for CustomerView {
    fn from(row: CustomerSummary) -> Self {
        Self {
            id: row.id,
            name: row.name,
            reference: row.reference,
            note: row.note,
            disabled: row.disabled,
            created_at: row.created_at.to_rfc3339(),
            created_by: row.created_by,
            tokens: row.tokens,
            nearest_expiry: row.nearest_expiry.map(|at| at.to_rfc3339()),
            downloads30d: row.downloads,
            bytes30d: row.bytes,
            last_access_at: row.last_access_at.map(|at| at.to_rfc3339()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenView {
    id: Uuid,
    name: String,
    hint: String,
    created_at: String,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    last_used_ip: Option<String>,
    owner: String,
    repositories: Vec<String>,
    created_by: Option<String>,
    expiry_notice: Option<i16>,
}

impl From<TokenInfo> for TokenView {
    fn from(token: TokenInfo) -> Self {
        Self {
            id: token.id,
            name: token.name,
            hint: token.hint,
            created_at: token.created_at.to_rfc3339(),
            expires_at: token.expires_at.map(|at| at.to_rfc3339()),
            last_used_at: token.last_used_at.map(|at| at.to_rfc3339()),
            last_used_ip: token.last_used_ip,
            owner: token.user_id,
            repositories: token.repositories.unwrap_or_default(),
            created_by: token.created_by,
            expiry_notice: token.expiry_notice,
        }
    }
}

#[derive(Deserialize)]
struct NewCustomerBody {
    id: Option<String>,
    name: Option<String>,
    reference: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct CustomerBody {
    name: Option<String>,
    reference: Option<String>,
    note: Option<String>,
    disabled: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueBody {
    name: Option<String>,
    owner: Option<String>,
    repositories: Option<Vec<String>>,
    expires_in_days: Option<i64>,
}

pub(crate) fn slug_problem(id: &str) -> Option<&'static str> {
    let valid = !id.is_empty()
        && id.len() <= MAX_ID
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    (!valid).then_some("must be 1 to 64 characters from a-z, 0-9 and '-'")
}

fn missing_customer() -> ApiError {
    ApiError::not_found("there is no such customer")
}

fn missing_token() -> ApiError {
    ApiError::not_found("there is no such token")
}

fn text(validation: &mut Validation, id: &str, value: Option<String>, max: usize) -> String {
    match value {
        Some(value) if value.len() <= max => value.trim().to_owned(),
        Some(_) => {
            validation.add(id, format!("must be at most {max} bytes"));
            String::new()
        }
        None => {
            validation.add(id, "must be given");
            String::new()
        }
    }
}

fn named(validation: &mut Validation, value: Option<String>) -> String {
    if value
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        validation.add("name", "must not be empty");
        return String::new();
    }
    text(validation, "name", value, MAX_NAME)
}

async fn existing(state: &AppState, id: &str) -> Result<customer::Model, ApiError> {
    state
        .security()
        .customer(id)
        .await?
        .ok_or_else(missing_customer)
}

async fn view(state: &AppState, id: &str) -> Result<CustomerView, ApiError> {
    state
        .security()
        .customer_summaries(Some(id))
        .await?
        .into_iter()
        .next()
        .map(CustomerView::from)
        .ok_or_else(missing_customer)
}

fn plain(model: &customer::Model) -> Value {
    json!({
        "name": model.name,
        "reference": model.reference,
        "note": model.note,
        "disabled": model.disabled,
    })
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<CustomerView>>, ApiError> {
    caller.require(READ)?;
    let rows = state.security().customer_summaries(None).await?;
    Ok(axum::Json(
        rows.into_iter().map(CustomerView::from).collect(),
    ))
}

async fn one(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<axum::Json<CustomerView>, ApiError> {
    caller.require(READ)?;
    Ok(axum::Json(view(&state, &id).await?))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<NewCustomerBody>,
) -> Result<Response, ApiError> {
    caller.require(CREATE)?;
    let mut validation = Validation::new();
    let id = body.id.unwrap_or_default();
    if let Some(problem) = slug_problem(&id) {
        validation.add("id", problem);
    }
    let name = named(&mut validation, body.name);
    let reference = body.reference.unwrap_or_default();
    validation.check(
        reference.len() <= MAX_REFERENCE,
        "reference",
        "must be at most 200 bytes",
    );
    let note = body.note.unwrap_or_default();
    validation.check(note.len() <= MAX_NOTE, "note", "must be at most 2000 bytes");
    validation.finish()?;
    if state.security().model().users.contains_key(&id) {
        return Err(ApiError::invalid(
            "id",
            "a user has this id; choose another one",
        ));
    }
    let created = state
        .security()
        .create_customer(NewCustomer {
            id: id.clone(),
            name,
            reference: reference.trim().to_owned(),
            note: note.trim().to_owned(),
            created_by: caller.initiator(),
        })
        .await;
    let model = match created {
        Ok(model) => model,
        Err(CustomerError::Exists) => {
            return Ok((
                StatusCode::CONFLICT,
                axum::Json(
                    json!({"id": "id", "message": "a customer with this id already exists"}),
                ),
            )
                .into_response());
        }
        Err(CustomerError::Internal(error)) => return Err(error.into()),
        Err(error @ CustomerError::TooManyRows(_)) => {
            return Err(ApiError::internal(anyhow::anyhow!(error)));
        }
    };
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "created", &id).with(plain(&model)),
        )
        .await;
    Ok((StatusCode::CREATED, axum::Json(view(&state, &id).await?)).into_response())
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Json(body): Json<CustomerBody>,
) -> Result<axum::Json<CustomerView>, ApiError> {
    caller.require(UPDATE)?;
    let mut validation = Validation::new();
    let name = named(&mut validation, body.name);
    let reference = text(&mut validation, "reference", body.reference, MAX_REFERENCE);
    let note = text(&mut validation, "note", body.note, MAX_NOTE);
    if body.disabled.is_none() {
        validation.add("disabled", "must be given");
    }
    validation.finish()?;
    let changed = state
        .security()
        .update_customer(
            &id,
            CustomerFields {
                name,
                reference,
                note,
                disabled: body.disabled.unwrap_or_default(),
            },
        )
        .await?;
    let (before, after) = changed.ok_or_else(missing_customer)?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", &id)
                .with(json!({"before": plain(&before), "after": plain(&after)})),
        )
        .await;
    Ok(axum::Json(view(&state, &id).await?))
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(DELETE)?;
    let removed = state
        .security()
        .delete_customer(&id)
        .await?
        .ok_or_else(missing_customer)?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "deleted", &id).with(plain(&removed)),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn tokens(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<axum::Json<Vec<TokenView>>, ApiError> {
    caller.require(READ)?;
    existing(&state, &id).await?;
    let tokens = state.security().customer_tokens(&id).await?;
    Ok(axum::Json(
        tokens.into_iter().map(TokenView::from).collect(),
    ))
}

async fn issue(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    Json(body): Json<IssueBody>,
) -> Result<Response, ApiError> {
    let subject = caller.require(CREATE)?.clone();
    subject
        .require_unnarrowed()
        .map_err(|_| ApiError::forbidden())?;
    if existing(&state, &id).await?.disabled {
        return Err(ApiError::invalid(
            "customer",
            "enable the customer before issuing tokens",
        ));
    }
    let mut validation = Validation::new();
    let name = body.name.unwrap_or_default();
    validation.check(
        !name.trim().is_empty() && name.len() <= MAX_NAME,
        "name",
        "must be 1 to 200 bytes",
    );
    let owner = body.owner.unwrap_or_default();
    validation.check(!owner.trim().is_empty(), "owner", OWNER_PROBLEM);
    let days = body.expires_in_days;
    validation.check(
        days.is_some_and(|days| (1..=MAX_DAYS).contains(&days)),
        "expiresInDays",
        &format!("must be given, from 1 to {MAX_DAYS} days"),
    );
    let listed = body.repositories.unwrap_or_default();
    validation.check(
        !listed.is_empty(),
        "repositories",
        "list at least one repository",
    );
    validation.finish()?;
    check_owner(&state, &owner)?;
    let repositories = resolve_repositories(&state, &listed).await?;
    access::require_readable(&state, &subject, &repositories).await?;
    let names: Vec<String> = repositories
        .iter()
        .map(|repository| repository.name.clone())
        .collect();
    let created = state
        .security()
        .create_token(
            &owner,
            NewToken {
                name,
                scope: Scope::Read,
                days,
                repositories: Some(names),
                customer: Some(id.clone()),
                created_by: Some(caller.initiator()),
            },
        )
        .await;
    let (info, token) = created.map_err(token_problem)?;
    state
        .audit(
            &caller,
            Event::new(TOKEN_DOMAIN, "created", &info.name).with(json!({
                "id": info.id,
                "user": info.user_id,
                "scope": info.scope,
                "customer": id,
                "repositories": info.repositories,
            })),
        )
        .await;
    let shown = json!({
        "id": info.id,
        "name": info.name,
        "hint": info.hint,
        "token": token,
        "expiresAt": info.expires_at.map(|at| at.to_rfc3339()),
        "owner": info.user_id,
        "repositories": info.repositories,
    });
    let mut response = (StatusCode::CREATED, axum::Json(shown)).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

const OWNER_PROBLEM: &str = "the owner must be an existing, enabled local user";

fn check_owner(state: &AppState, owner: &str) -> Result<(), ApiError> {
    let model = state.security().model();
    let usable = owner != ANONYMOUS_USER
        && owner != model.anonymous.user_id
        && model
            .users
            .get(owner)
            .is_some_and(|user| user.status == UserStatus::Active);
    match usable {
        true => Ok(()),
        false => Err(ApiError::invalid("owner", OWNER_PROBLEM)),
    }
}

fn token_problem(error: TokenError) -> ApiError {
    match error {
        TokenError::Name => ApiError::invalid("name", error.to_string()),
        TokenError::Days | TokenError::CustomerExpiry => {
            ApiError::invalid("expiresInDays", error.to_string())
        }
        TokenError::Repositories(message) => ApiError::invalid("repositories", message),
        TokenError::UnknownCustomer => missing_customer(),
        TokenError::TooMany | TokenError::TooManyForCustomer | TokenError::Scope => {
            ApiError::BadRequest(error.to_string())
        }
        TokenError::Internal(error) => error.into(),
    }
}

async fn revoke_one(
    State(state): State<AppState>,
    caller: Caller,
    Path((id, token_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    caller.require(DELETE)?;
    let token_id = Uuid::parse_str(&token_id).map_err(|_| missing_token())?;
    let held = state.security().customer_tokens(&id).await?;
    let token = held
        .into_iter()
        .find(|token| token.id == token_id)
        .ok_or_else(missing_token)?;
    if !state
        .security()
        .revoke_customer_token(&id, token_id)
        .await?
    {
        return Err(missing_token());
    }
    state
        .audit(
            &caller,
            Event::new(TOKEN_DOMAIN, "deleted", &token.name).with(json!({
                "id": token.id,
                "user": token.user_id,
                "customer": id,
                "repositories": token.repositories,
            })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn revoke_all(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<axum::Json<Value>, ApiError> {
    caller.require(DELETE)?;
    existing(&state, &id).await?;
    let count = state.security().revoke_customer_tokens(&id).await?;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "revoked", &id).with(json!({"count": count})),
        )
        .await;
    Ok(axum::Json(json!({"revoked": count})))
}
