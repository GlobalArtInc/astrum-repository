use std::collections::BTreeSet;

use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_security::catalogue::DEFAULT_SOURCE;
use astrum_security::two_factor::passkeys::{RelyingParty, RelyingPartyError};
use astrum_security::two_factor::store::StoreError;
use astrum_security::two_factor::{
    AUDIT_DOMAIN, Methods, POLICY_KEY, RelyingPartyPolicy, RequiredBy, Requirement, lock_policy,
    reset_event,
};
use astrum_security::{SecurityModel, TwoFactorPolicy};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement, TransactionTrait};
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use super::users::user_not_found;
use super::{SECURITY, changed};
use crate::{ApiError, AppState, Caller, Json, Validation};

const USERS_READ: &str = "astrum:users:read";
const USERS_UPDATE: &str = "astrum:users:update";
const ADMINISTRATOR: &str = "astrum:*";
const MAX_GRACE_DAYS: u32 = 90;
const MAX_ROLES: usize = 100;
const MAX_ORIGINS: usize = 20;
const MAX_ORIGIN_BYTES: usize = 255;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{SECURITY}/users/{{user_id}}/two-factor"),
            get(status).delete(reset),
        )
        .route(
            &format!("{SECURITY}/users/{{user_id}}/requirement"),
            put(set_requirement),
        )
        .route(
            &format!("{SECURITY}/two-factor"),
            get(policy).put(set_policy),
        )
}

const POLICY_FIELDS: [&str; 6] = [
    "totp",
    "passkeys",
    "required",
    "graceDays",
    "requiredSince",
    "relyingParty",
];
const REQUIRED_FIELDS: [&str; 3] = ["everyone", "administrators", "roles"];
const RELYING_PARTY_FIELDS: [&str; 2] = ["id", "origins"];
const NO_METHOD: &str = "turn on authenticator apps (totp) before requiring a second factor: passkeys cannot be used for enrollment yet";
const PASSKEYS_WITHOUT_ID: &str =
    "passkeys need a relying party id: set the base URL or relyingParty.id";

#[derive(Debug, Deserialize)]
struct RequirementBody {
    requirement: Option<String>,
}

fn store_error(user_id: &str, error: StoreError) -> ApiError {
    match error {
        StoreError::UnknownUser => user_not_found(user_id),
        StoreError::InvalidRequirement => requirement_invalid(),
        StoreError::NoMethodAvailable => ApiError::invalid("requirement", NO_METHOD),
        other => ApiError::internal(other),
    }
}

fn delegated_refusal(what: &str) -> ApiError {
    ApiError::Forbidden(format!("only an administrator may {what}"))
}

fn requirement_invalid() -> ApiError {
    ApiError::invalid("requirement", "must be one of policy, required, exempt")
}

fn known(model: &SecurityModel, user_id: &str) -> Result<(), ApiError> {
    match model.users.contains_key(user_id) {
        true => Ok(()),
        false => Err(user_not_found(user_id)),
    }
}

async fn status(
    State(state): State<AppState>,
    caller: Caller,
    Path(user_id): Path<String>,
) -> Result<axum::Json<Value>, ApiError> {
    caller.require(USERS_READ)?;
    let security = state.security();
    let model = security.model();
    known(&model, &user_id)?;
    let overview = security
        .summary(&user_id)
        .await
        .map_err(|error| store_error(&user_id, error))?;
    let standing = model.second_factor_standing(&user_id);
    Ok(axum::Json(json!({
        "userId": user_id,
        "enrolled": overview.enrolled(),
        "totp": overview.totp,
        "recoveryCodesLeft": overview.recovery_codes_left,
        "requirement": overview.requirement.as_str(),
        "requirementSource": overview.requirement.as_str(),
        "required": standing.required,
        "enrollBy": standing.enroll_by.filter(|_| !overview.enrolled()),
        "lockedUntil": overview.locked_until,
    })))
}

async fn set_requirement(
    State(state): State<AppState>,
    caller: Caller,
    Path(user_id): Path<String>,
    Json(body): Json<RequirementBody>,
) -> Result<StatusCode, ApiError> {
    let subject = caller.require(USERS_UPDATE)?;
    let requirement = match body.requirement.as_deref() {
        Some("policy") => Requirement::Policy,
        Some("required") => Requirement::Required,
        Some("exempt") => Requirement::Exempt,
        _ => return Err(requirement_invalid()),
    };
    let security = state.security();
    let model = security.model();
    known(&model, &user_id)?;
    if !subject.is_permitted(ADMINISTRATOR) {
        if requirement == Requirement::Exempt {
            return Err(delegated_refusal("exempt a user from the second factor"));
        }
        if model.is_administrator(&user_id) {
            return Err(delegated_refusal(
                "change the second factor requirement of an administrator",
            ));
        }
    }
    let before = security
        .summary(&user_id)
        .await
        .map_err(|error| store_error(&user_id, error))?
        .requirement;
    security
        .set_requirement(&user_id, requirement)
        .await
        .map_err(|error| store_error(&user_id, error))?;
    state
        .audit(
            &caller,
            Event::new(AUDIT_DOMAIN, "requirement.updated", &user_id).with(json!({
                "userId": user_id,
                "before": before.as_str(),
                "after": requirement.as_str(),
            })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn reset(
    State(state): State<AppState>,
    caller: Caller,
    Path(user_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let subject = caller.require(USERS_UPDATE)?;
    let security = state.security();
    let model = security.model();
    known(&model, &user_id)?;
    if subject.user_id == user_id && subject.source == DEFAULT_SOURCE {
        return Err(ApiError::BadRequest(
            "you cannot reset the second factor of the user you are signed in as".to_owned(),
        ));
    }
    if !subject.is_permitted(ADMINISTRATOR) && model.is_administrator(&user_id) {
        return Err(delegated_refusal(
            "reset the second factor of an administrator",
        ));
    }
    security
        .reset(&user_id)
        .await
        .map_err(|error| store_error(&user_id, error))?;
    state.audit(&caller, reset_event(&user_id)).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn policy(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<TwoFactorPolicy>, ApiError> {
    caller.require(ADMINISTRATOR)?;
    Ok(axum::Json(state.security().model().two_factor.clone()))
}

async fn set_policy(
    State(state): State<AppState>,
    caller: Caller,
    Json(body): Json<Value>,
) -> Result<axum::Json<TwoFactorPolicy>, ApiError> {
    caller.require(ADMINISTRATOR)?;
    check_shape(&body)?;
    let mut policy: TwoFactorPolicy =
        serde_json::from_value(body).map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let model = state.security().model();
    let base = state.configured_base_url().await;
    let mut validation = Validation::new();
    validation.check(
        policy.grace_days <= MAX_GRACE_DAYS,
        "graceDays",
        "must be between 0 and 90",
    );
    check_roles(&mut validation, &model, &policy.required.roles);
    check_relying_party(&mut validation, &mut policy.relying_party, base.as_deref());

    let txn = state.db().conn().begin().await?;
    lock_policy(&txn).await?;
    let stored = stored_policy(&txn).await?;
    if requires_anyone(&policy.required) || anyone_required(&txn).await? {
        match policy.methods(base.as_deref()) {
            Methods::Available => {}
            Methods::NoneEnabled => validation.add("totp", NO_METHOD),
            Methods::PasskeysWithoutId => validation.add("passkeys", PASSKEYS_WITHOUT_ID),
        }
    }
    validation.finish()?;
    policy.required_since = match widens(&stored.required, &policy.required) {
        true => Some(Utc::now()),
        false => stored.required_since,
    };
    setting::put(&txn, POLICY_KEY, &policy).await?;
    txn.commit().await?;

    changed(
        &state,
        &caller,
        Event::new(AUDIT_DOMAIN, "policy.updated", "policy")
            .with(json!({"before": stored, "after": policy})),
    )
    .await?;
    Ok(axum::Json(policy))
}

fn check_shape(body: &Value) -> Result<(), ApiError> {
    let Value::Object(fields) = body else {
        return Err(ApiError::BadRequest(
            "the policy must be a JSON object".to_owned(),
        ));
    };
    let mut validation = Validation::new();
    check_fields(&mut validation, "", fields, &POLICY_FIELDS);
    for (name, allowed) in [
        ("required", &REQUIRED_FIELDS[..]),
        ("relyingParty", &RELYING_PARTY_FIELDS[..]),
    ] {
        match fields.get(name) {
            None => {}
            Some(Value::Object(nested)) => {
                check_fields(&mut validation, &format!("{name}."), nested, allowed);
            }
            Some(_) => validation.add(name, "must be an object"),
        }
    }
    validation.finish()
}

fn check_fields(
    validation: &mut Validation,
    prefix: &str,
    fields: &serde_json::Map<String, Value>,
    allowed: &[&str],
) {
    for name in fields
        .keys()
        .filter(|name| !allowed.contains(&name.as_str()))
    {
        validation.add(format!("{prefix}{name}"), "is not a field of the policy");
    }
}

fn requires_anyone(required: &RequiredBy) -> bool {
    required.everyone || required.administrators || !required.roles.is_empty()
}

fn widens(old: &RequiredBy, new: &RequiredBy) -> bool {
    (new.everyone && !old.everyone)
        || (new.administrators && !old.administrators)
        || new.roles.iter().any(|role| !old.roles.contains(role))
}

async fn anyone_required(db: &impl ConnectionTrait) -> Result<bool, ApiError> {
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT EXISTS (SELECT 1 FROM user_second_factor WHERE requirement = 'required') AS found",
        ))
        .await?
        .ok_or_else(|| ApiError::internal(DbErr::RecordNotFound("required users".to_owned())))?;
    Ok(row.try_get("", "found")?)
}

async fn stored_policy(db: &impl ConnectionTrait) -> Result<TwoFactorPolicy, ApiError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT value FROM setting WHERE key = $1 FOR UPDATE",
            [POLICY_KEY.into()],
        ))
        .await?;
    let Some(row) = row else {
        return Ok(TwoFactorPolicy::default());
    };
    let value: Value = row.try_get("", "value")?;
    Ok(serde_json::from_value(value).unwrap_or_else(|error| {
        tracing::error!(%error, "the stored two-factor policy cannot be read and is replaced");
        TwoFactorPolicy::default()
    }))
}

fn check_roles(validation: &mut Validation, model: &SecurityModel, roles: &[String]) {
    if roles.len() > MAX_ROLES {
        validation.add("required.roles", "must name at most 100 roles");
    }
    let mut seen = BTreeSet::new();
    for role in roles {
        if !seen.insert(role) {
            validation.add("required.roles", format!("the role {role} is listed twice"));
        } else if !model.roles.contains_key(role) {
            validation.add("required.roles", format!("the role {role} does not exist"));
        }
    }
}

const FIELD_ID: &str = "relyingParty.id";
const FIELD_ORIGINS: &str = "relyingParty.origins";

fn origin_problem(origin: &str) -> Option<&'static str> {
    if origin.len() > MAX_ORIGIN_BYTES {
        return Some("must be at most 255 bytes");
    }
    match Url::parse(origin) {
        Ok(url) if url.origin().ascii_serialization() == origin => None,
        Ok(_) => Some(
            "must be written exactly as https://host[:port] or http://localhost[:port], in lower case, with no path, query, user info, default port or trailing slash",
        ),
        Err(_) => Some("is not a URL"),
    }
}

fn check_relying_party(
    validation: &mut Validation,
    relying_party: &mut RelyingPartyPolicy,
    base: Option<&str>,
) {
    relying_party.id = relying_party.id.take().filter(|id| !id.is_empty());
    let mut clean = true;
    if let Some(id) = &relying_party.id {
        if *id != id.to_ascii_lowercase() {
            validation.add(FIELD_ID, "must be a lower-case host name");
            clean = false;
        } else if base.is_none() {
            validation.add(
                FIELD_ID,
                "needs a base URL to be checked against; set the base URL first",
            );
            clean = false;
        }
    }
    if relying_party.origins.len() > MAX_ORIGINS {
        validation.add(FIELD_ORIGINS, "must name at most 20 origins");
        clean = false;
    }
    for origin in &relying_party.origins {
        if let Some(problem) = origin_problem(origin) {
            validation.add(FIELD_ORIGINS, format!("{origin}: {problem}"));
            clean = false;
        }
    }
    if !clean || (relying_party.id.is_none() && relying_party.origins.is_empty()) {
        return;
    }
    let Err(error) = RelyingParty::resolve(relying_party, base) else {
        return;
    };
    let field = match (&error, &relying_party.id) {
        (RelyingPartyError::Origin { .. } | RelyingPartyError::Unavailable, _) => FIELD_ORIGINS,
        (_, Some(_)) => FIELD_ID,
        (_, None) => FIELD_ORIGINS,
    };
    validation.add(field, error.to_string());
}
