use astrum_db::audit::Event;
use astrum_db::revision;
use astrum_policy::Error as PolicyError;
use astrum_policy::exception::{self, Exception, Rule};
use astrum_policy::key::{self, TrustedKey};
use astrum_policy::policy::{
    self, CooldownRule, LicenseList, LicenseRule, Malicious, Mode, Policy, Severity, SignatureRule,
    UnknownLicense, Unscanned, VulnerabilityRule,
};
use astrum_repo::Writer;
use astrum_signature::PublicKey;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, TransactionTrait};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::{ApiError, Validation};
use crate::routes::tasks::status;
use crate::{AppState, Caller};

pub const POLICIES: &str = "/service/rest/v1/security/policies";
pub const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";
pub const KEYS: &str = "/service/rest/v1/security/keys";
pub const POLICY_DOMAIN: &str = "contentPolicy";
pub const EXCEPTION_DOMAIN: &str = "policyException";
pub const KEY_DOMAIN: &str = "trustedKey";
pub const READ: &str = "astrum:policies:read";
pub const CREATE: &str = "astrum:policies:create";
pub const UPDATE: &str = "astrum:policies:update";
pub const DELETE: &str = "astrum:policies:delete";
pub const EXCEPTION_READ: &str = "astrum:policyexceptions:read";
pub const EXCEPTION_CREATE: &str = "astrum:policyexceptions:create";
pub const EXCEPTION_DELETE: &str = "astrum:policyexceptions:delete";
pub const MAX_NAME: usize = 64;
pub const MAX_NOTES: usize = 1000;
pub const MAX_REASON: usize = 1000;
pub const MAX_TARGET: usize = 256;
pub const MAX_KEYS_PER_POLICY: usize = 64;
pub const MAX_AUDITED_IDS: usize = 100;

const NAME_RULE: &str =
    "may hold only letters, digits, '-', '_' and '.', and must not start with '_' or '.'";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(POLICIES, get(list_policies).post(create_policy))
        .route(
            "/service/rest/v1/security/policies/{name}",
            get(read_policy).put(update_policy).delete(remove_policy),
        )
        .route(EXCEPTIONS, get(list_exceptions).post(create_exception))
        .route(
            "/service/rest/v1/security/exceptions/{id}",
            delete(remove_exception),
        )
        .route(KEYS, get(list_keys).post(create_key))
        .route("/service/rest/v1/security/keys/{name}", delete(remove_key))
}

type Reply = Result<Response, ApiError>;

fn failed(error: PolicyError) -> Reply {
    match error {
        PolicyError::Duplicate(_) => Ok(status(StatusCode::CONFLICT, error.to_string())),
        PolicyError::NotFound(_) => Err(ApiError::not_found(error.to_string())),
        PolicyError::Invalid(message) => Err(ApiError::BadRequest(message)),
        other => Err(ApiError::internal(other)),
    }
}

fn internal(error: impl Into<anyhow::Error>) -> ApiError {
    ApiError::internal(error)
}

fn plain(text: &str, newlines: bool) -> bool {
    !text
        .chars()
        .any(|character| character.is_control() && !(newlines && character == '\n'))
}

fn check_name(name: &str, validation: &mut Validation) {
    if name.is_empty() {
        validation.add("name", "must not be empty");
    } else if !astrum_repo::config::valid_name(name) {
        validation.add("name", NAME_RULE);
    } else if name.len() > MAX_NAME {
        validation.add(
            "name",
            format!("must be at most {MAX_NAME} characters long"),
        );
    }
}

fn check_text(id: &str, text: &str, limit: usize, newlines: bool, validation: &mut Validation) {
    if text.chars().count() > limit {
        validation.add(id, format!("must be at most {limit} characters long"));
    }
    if !plain(text, newlines) {
        validation.add(id, "must not hold control characters");
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PolicyRequest {
    name: Option<String>,
    notes: Option<String>,
    vulnerabilities: Option<VulnerabilityBody>,
    licenses: Option<LicenseBody>,
    signatures: Option<SignatureBody>,
    cooldown: Option<CooldownRule>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VulnerabilityBody {
    mode: Option<Mode>,
    severity: Option<Severity>,
    #[serde(default)]
    malicious: Malicious,
    #[serde(default)]
    unscanned: Unscanned,
    notify: Option<Severity>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LicenseBody {
    mode: Option<Mode>,
    list: Option<LicenseList>,
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default)]
    unknown: UnknownLicense,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SignatureBody {
    mode: Option<Mode>,
    #[serde(default)]
    keys: Vec<String>,
}

fn required_mode(block: &str, mode: Option<Mode>, validation: &mut Validation) -> Mode {
    mode.unwrap_or_else(|| {
        validation.add(
            format!("{block}.mode"),
            format!("is required when {block} is given"),
        );
        Mode::Off
    })
}

impl VulnerabilityBody {
    fn into_rule(self, validation: &mut Validation) -> VulnerabilityRule {
        VulnerabilityRule {
            mode: required_mode("vulnerabilities", self.mode, validation),
            severity: self.severity,
            malicious: self.malicious,
            unscanned: self.unscanned,
            notify: self.notify,
        }
    }
}

impl LicenseBody {
    fn into_rule(self, validation: &mut Validation) -> LicenseRule {
        LicenseRule {
            mode: required_mode("licenses", self.mode, validation),
            list: self.list.unwrap_or_default(),
            ids: self.ids,
            unknown: self.unknown,
        }
    }
}

impl SignatureBody {
    fn into_rule(self, validation: &mut Validation) -> SignatureRule {
        SignatureRule {
            mode: required_mode("signatures", self.mode, validation),
            keys: self.keys,
        }
    }
}

struct Blocks {
    name: Option<String>,
    notes: Option<String>,
    vulnerabilities: Option<VulnerabilityRule>,
    licenses: Option<LicenseRule>,
    signatures: Option<SignatureRule>,
    cooldown: Option<CooldownRule>,
}

impl PolicyRequest {
    fn checked(self, validation: &mut Validation) -> Blocks {
        let blocks = Blocks {
            name: self.name,
            notes: self.notes,
            vulnerabilities: self.vulnerabilities.map(|body| body.into_rule(validation)),
            licenses: self.licenses.map(|body| body.into_rule(validation)),
            signatures: self.signatures.map(|body| body.into_rule(validation)),
            cooldown: self.cooldown,
        };
        if let Some(notes) = &blocks.notes {
            check_text("notes", notes, MAX_NOTES, true, validation);
        }
        if let Some(licenses) = &blocks.licenses {
            check_licenses(licenses, validation);
        }
        if let Some(signatures) = &blocks.signatures {
            check_signatures(signatures, validation);
        }
        blocks
    }
}

fn check_licenses(rule: &LicenseRule, validation: &mut Validation) {
    if let Err(message) = rule.check() {
        validation.add("licenses.ids", message);
    }
}

fn check_signatures(rule: &SignatureRule, validation: &mut Validation) {
    if rule.mode == Mode::Block && rule.keys.is_empty() {
        validation.add(
            "signatures.keys",
            "must name at least one key when the mode is block",
        );
    }
}

async fn check_keys(
    db: &impl ConnectionTrait,
    policy: &Policy,
    validation: &mut Validation,
) -> Result<(), ApiError> {
    let names = &policy.signatures.keys;
    if names.len() > MAX_KEYS_PER_POLICY {
        validation.add(
            "signatures.keys",
            format!("must name at most {MAX_KEYS_PER_POLICY} keys"),
        );
        return Ok(());
    }
    let mut seen = std::collections::BTreeSet::new();
    for name in names {
        if !seen.insert(name.as_str()) {
            validation.add("signatures.keys", format!("{name} is named twice"));
            continue;
        }
        let known = key::find(db, name).await.map_err(internal)?;
        if known.is_none() {
            validation.add(
                "signatures.keys",
                format!("there is no trusted key named {name}"),
            );
        }
    }
    Ok(())
}

fn document(policy: &Policy) -> Value {
    serde_json::to_value(policy).expect("a policy serializes to JSON")
}

fn audited(policy: &Policy) -> Value {
    json!({
        "notes": policy.notes,
        "vulnerabilities": policy.vulnerabilities,
        "licenses": policy.licenses,
        "signatures": policy.signatures,
        "cooldown": policy.cooldown,
    })
}

async fn stored(state: &AppState, name: &str) -> Result<Policy, ApiError> {
    policy::find(state.db().conn(), name)
        .await
        .map_err(internal)?
        .ok_or_else(|| missing_policy(name))
}

fn missing_policy(name: &str) -> ApiError {
    ApiError::not_found(format!("there is no content policy named {name}"))
}

async fn list_policies(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let policies = policy::list(state.db().conn()).await.map_err(internal)?;
    Ok(Json(Value::Array(policies.iter().map(document).collect())))
}

async fn read_policy(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    Ok(Json(document(&stored(&state, &name).await?)))
}

async fn create_policy(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<PolicyRequest>, ApiError>,
) -> Reply {
    caller.require(CREATE)?;
    let crate::Json(request) = body?;
    let mut validation = Validation::new();
    let blocks = request.checked(&mut validation);
    let policy = Policy {
        name: blocks.name.unwrap_or_default(),
        notes: blocks.notes.unwrap_or_default(),
        vulnerabilities: blocks.vulnerabilities.unwrap_or_default(),
        licenses: blocks.licenses.unwrap_or_default(),
        signatures: blocks.signatures.unwrap_or_default(),
        cooldown: blocks.cooldown.unwrap_or_default(),
        revision: 1,
    };
    check_name(&policy.name, &mut validation);
    validation.finish()?;
    let writer = Writer::begin(state.db().conn()).await.map_err(internal)?;
    let mut validation = Validation::new();
    check_keys(writer.transaction(), &policy, &mut validation).await?;
    validation.finish()?;
    if let Err(error) = policy::create(writer.transaction(), &policy).await {
        return failed(error);
    }
    revision::bump(writer.transaction(), revision::POLICIES)
        .await
        .map_err(internal)?;
    writer.commit().await.map_err(internal)?;
    state.policy_cache().invalidate();
    state
        .audit(
            &caller,
            Event::new(POLICY_DOMAIN, "created", &policy.name).with(audited(&policy)),
        )
        .await;
    let saved = stored(&state, &policy.name).await?;
    Ok((StatusCode::CREATED, Json(document(&saved))).into_response())
}

async fn update_policy(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<crate::Json<PolicyRequest>, ApiError>,
) -> Reply {
    caller.require(UPDATE)?;
    let crate::Json(request) = body?;
    let mut validation = Validation::new();
    let blocks = request.checked(&mut validation);
    if blocks.name.as_deref().is_some_and(|given| given != name) {
        validation.add(
            "name",
            "cannot be changed; content policies are not renamed",
        );
    }
    validation.finish()?;
    let writer = Writer::begin(state.db().conn()).await.map_err(internal)?;
    let current = match policy::find(writer.transaction(), &name)
        .await
        .map_err(internal)?
    {
        Some(current) => current,
        None => return Err(missing_policy(&name)),
    };
    let policy = Policy {
        name: current.name.clone(),
        notes: blocks.notes.unwrap_or_else(|| current.notes.clone()),
        vulnerabilities: blocks
            .vulnerabilities
            .unwrap_or_else(|| current.vulnerabilities.clone()),
        licenses: blocks.licenses.unwrap_or_else(|| current.licenses.clone()),
        signatures: blocks
            .signatures
            .unwrap_or_else(|| current.signatures.clone()),
        cooldown: blocks.cooldown.unwrap_or_else(|| current.cooldown.clone()),
        revision: current.revision,
    };
    if policy == current {
        return Ok(Json(document(&current)).into_response());
    }
    let mut validation = Validation::new();
    check_keys(writer.transaction(), &policy, &mut validation).await?;
    validation.finish()?;
    if let Err(error) = policy::update(writer.transaction(), &policy).await {
        return failed(error);
    }
    revision::bump(writer.transaction(), revision::POLICIES)
        .await
        .map_err(internal)?;
    writer.commit().await.map_err(internal)?;
    state.policy_cache().invalidate();
    state
        .audit(
            &caller,
            Event::new(POLICY_DOMAIN, "updated", &policy.name).with(audited(&policy)),
        )
        .await;
    let saved = stored(&state, &policy.name).await?;
    Ok(Json(document(&saved)).into_response())
}

async fn remove_policy(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Reply {
    caller.require(DELETE)?;
    let writer = Writer::begin(state.db().conn()).await.map_err(internal)?;
    let users = writer.content_policy_users(&name);
    if !users.is_empty() {
        return Ok(status(
            StatusCode::CONFLICT,
            format!(
                "the content policy {name} is used by repositories: {}",
                users.join(", ")
            ),
        ));
    }
    let removed: Vec<String> = exception::list(writer.transaction())
        .await
        .map_err(internal)?
        .iter()
        .filter(|held| held.policy.as_deref() == Some(name.as_str()))
        .map(|held| held.id.to_string())
        .collect();
    match policy::delete(writer.transaction(), &name)
        .await
        .map_err(internal)?
    {
        true => {}
        false => return Err(missing_policy(&name)),
    }
    revision::bump(writer.transaction(), revision::POLICIES)
        .await
        .map_err(internal)?;
    writer.commit().await.map_err(internal)?;
    state.policy_cache().invalidate();
    state
        .audit(
            &caller,
            Event::new(POLICY_DOMAIN, "deleted", &name).with(json!({
                "exceptions": removed.len(),
                "exceptionIds": removed.iter().take(MAX_AUDITED_IDS).collect::<Vec<_>>(),
            })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExceptionRequest {
    policy: Option<String>,
    rule: Rule,
    advisory: Option<String>,
    license: Option<String>,
    format: Option<String>,
    namespace: Option<String>,
    name: Option<String>,
    version: Option<String>,
    digest: Option<String>,
    reason: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

fn target(id: &str, value: &Option<String>, validation: &mut Validation) -> Option<String> {
    let text = value.as_deref()?.trim();
    if text.is_empty() {
        validation.add(id, "must not be empty when given");
        return None;
    }
    check_text(id, text, MAX_TARGET, false, validation);
    Some(text.to_owned())
}

fn valid_digest(digest: &str) -> bool {
    let hexadecimal = |hash: &str, length: usize| {
        hash.len() == length
            && hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    match digest.split_once(':') {
        Some(("sha256", hash)) => hexadecimal(hash, 64),
        Some(("sha512", hash)) => hexadecimal(hash, 128),
        _ => false,
    }
}

async fn checked_exception(
    state: &AppState,
    caller: &Caller,
    request: ExceptionRequest,
) -> Result<Exception, ApiError> {
    let mut validation = Validation::new();
    let advisory = target("advisory", &request.advisory, &mut validation);
    let license = target("license", &request.license, &mut validation);
    let format = target("format", &request.format, &mut validation);
    let namespace = target("namespace", &request.namespace, &mut validation);
    let name = target("name", &request.name, &mut validation);
    let version = target("version", &request.version, &mut validation);
    let digest = target("digest", &request.digest, &mut validation);
    if let Some(format) = &format
        && !astrum_repo::FORMATS
            .iter()
            .any(|known| known.id == format.as_str())
    {
        validation.add(
            "format",
            format!("{format} is not a format this server knows"),
        );
    }
    if let Some(digest) = &digest
        && !valid_digest(digest)
    {
        validation.add(
            "digest",
            "must be sha256: or sha512: followed by lowercase hexadecimal digits",
        );
    }
    let rule = request.rule.as_str();
    if request.rule == Rule::License && advisory.is_some() {
        validation.add("advisory", "a license exception cannot target an advisory");
    }
    if request.rule != Rule::License && license.is_some() {
        validation.add(
            "license",
            format!("a {rule} exception cannot target a license"),
        );
    }
    if request.rule == Rule::Signature && request.policy.is_none() {
        validation.add("policy", "a signature exception needs a policy");
    }
    if request.rule == Rule::Signature && advisory.is_some() {
        validation.add(
            "advisory",
            "a signature exception cannot target an advisory",
        );
    }
    let given = [
        &request.advisory,
        &request.license,
        &request.name,
        &request.digest,
    ]
    .iter()
    .any(|value| value.is_some());
    if !given {
        validation.add(
            "target",
            "give at least one of advisory, license, name or digest",
        );
    }
    let reason = request.reason.as_deref().unwrap_or_default().trim();
    if reason.is_empty() {
        validation.add("reason", "must not be empty");
    } else {
        check_text("reason", reason, MAX_REASON, true, &mut validation);
    }
    let now = Utc::now();
    if request.expires_at.is_some_and(|expires| expires <= now) {
        validation.add("expiresAt", "must be in the future");
    }
    let policy = match request.policy.as_deref() {
        None => None,
        Some("") => {
            validation.add("policy", "must not be empty when given");
            None
        }
        Some(given) => {
            let known = policy::find(state.db().conn(), given)
                .await
                .map_err(internal)?;
            if known.is_none() {
                validation.add(
                    "policy",
                    format!("there is no content policy named {given}"),
                );
            }
            Some(given.to_owned())
        }
    };
    validation.finish()?;
    Ok(Exception {
        id: Uuid::new_v4(),
        policy,
        rule: request.rule,
        advisory,
        license,
        format,
        namespace,
        name,
        version,
        digest,
        reason: reason.to_owned(),
        expires_at: request.expires_at,
        created_by: caller.initiator(),
        created_at: now,
    })
}

fn exception_document(exception: &Exception) -> Value {
    serde_json::to_value(exception).expect("an exception serializes to JSON")
}

async fn list_exceptions(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<Json<Value>, ApiError> {
    caller.require(EXCEPTION_READ)?;
    let held = exception::list(state.db().conn()).await.map_err(internal)?;
    Ok(Json(Value::Array(
        held.iter().map(exception_document).collect(),
    )))
}

async fn create_exception(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<ExceptionRequest>, ApiError>,
) -> Reply {
    caller.require(EXCEPTION_CREATE)?;
    let crate::Json(request) = body?;
    let exception = checked_exception(&state, &caller, request).await?;
    let txn = state.db().conn().begin().await?;
    if let Err(error) = exception::create(&txn, &exception).await {
        return match error {
            PolicyError::NotFound(_) => Err(ApiError::invalid(
                "policy",
                "the content policy was deleted meanwhile",
            )),
            other => failed(other),
        };
    }
    revision::bump(&txn, revision::POLICIES)
        .await
        .map_err(internal)?;
    txn.commit().await?;
    state.policy_cache().invalidate();
    state
        .audit(
            &caller,
            Event::new(EXCEPTION_DOMAIN, "created", exception.id.to_string())
                .with(exception_document(&exception)),
        )
        .await;
    Ok((StatusCode::CREATED, Json(exception_document(&exception))).into_response())
}

async fn remove_exception(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Reply {
    caller.require(EXCEPTION_DELETE)?;
    let missing = || ApiError::not_found(format!("there is no policy exception with the id {id}"));
    let parsed = Uuid::parse_str(&id).map_err(|_| missing())?;
    let txn = state.db().conn().begin().await?;
    let held = exception::find(&txn, parsed)
        .await
        .map_err(internal)?
        .ok_or_else(missing)?;
    if !exception::delete(&txn, parsed).await.map_err(internal)? {
        return Err(missing());
    }
    revision::bump(&txn, revision::POLICIES)
        .await
        .map_err(internal)?;
    txn.commit().await?;
    state.policy_cache().invalidate();
    state
        .audit(
            &caller,
            Event::new(EXCEPTION_DOMAIN, "deleted", parsed.to_string())
                .with(exception_document(&held)),
        )
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct KeyRequest {
    name: Option<String>,
    public_key: Option<String>,
    notes: Option<String>,
}

fn key_document(key: &TrustedKey) -> Value {
    serde_json::to_value(key).expect("a trusted key serializes to JSON")
}

async fn list_keys(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let keys = key::list(state.db().conn()).await.map_err(internal)?;
    Ok(Json(Value::Array(keys.iter().map(key_document).collect())))
}

async fn create_key(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<KeyRequest>, ApiError>,
) -> Reply {
    caller.require(CREATE)?;
    let crate::Json(request) = body?;
    let name = request.name.unwrap_or_default();
    let notes = request.notes.unwrap_or_default();
    let mut validation = Validation::new();
    check_name(&name, &mut validation);
    check_text("notes", &notes, MAX_NOTES, true, &mut validation);
    let parsed = match request.public_key.as_deref() {
        None => {
            validation.add("publicKey", "must not be empty");
            None
        }
        Some(pem) => match key::parse_public_key(pem) {
            Ok(mut parsed) => match PublicKey::from_pem(&parsed.public_key) {
                Ok(verifying) => {
                    parsed.fingerprint = verifying.fingerprint().to_owned();
                    Some(parsed)
                }
                Err(error) => {
                    validation.add("publicKey", error.to_string());
                    None
                }
            },
            Err(PolicyError::Invalid(message)) => {
                validation.add("publicKey", message);
                None
            }
            Err(other) => return failed(other),
        },
    };
    validation.finish()?;
    let parsed = parsed.expect("a key is parsed when the validation passed");
    let created = TrustedKey {
        name,
        algorithm: parsed.algorithm.to_owned(),
        public_key: parsed.public_key,
        fingerprint: parsed.fingerprint,
        notes,
        created_at: Utc::now(),
    };
    let txn = state.db().conn().begin().await?;
    if let Err(error) = key::create(&txn, &created).await {
        return failed(error);
    }
    revision::bump(&txn, revision::POLICIES)
        .await
        .map_err(internal)?;
    txn.commit().await?;
    state
        .audit(
            &caller,
            Event::new(KEY_DOMAIN, "created", &created.name).with(json!({
                "algorithm": created.algorithm,
                "fingerprint": created.fingerprint,
            })),
        )
        .await;
    Ok((StatusCode::CREATED, Json(key_document(&created))).into_response())
}

async fn remove_key(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Reply {
    caller.require(DELETE)?;
    let writer = Writer::begin(state.db().conn()).await.map_err(internal)?;
    let held = key::find(writer.transaction(), &name)
        .await
        .map_err(internal)?
        .ok_or_else(|| ApiError::not_found(format!("there is no trusted key named {name}")))?;
    let users: Vec<String> = policy::list(writer.transaction())
        .await
        .map_err(internal)?
        .into_iter()
        .filter(|policy| policy.signatures.keys.contains(&name))
        .map(|policy| policy.name)
        .collect();
    if !users.is_empty() {
        return Ok(status(
            StatusCode::CONFLICT,
            format!(
                "the trusted key {name} is used by content policies: {}",
                users.join(", ")
            ),
        ));
    }
    if !key::delete(writer.transaction(), &name)
        .await
        .map_err(internal)?
    {
        return Err(ApiError::not_found(format!(
            "there is no trusted key named {name}"
        )));
    }
    revision::bump(writer.transaction(), revision::POLICIES)
        .await
        .map_err(internal)?;
    writer.commit().await.map_err(internal)?;
    state
        .audit(
            &caller,
            Event::new(KEY_DOMAIN, "deleted", &name).with(json!({
                "algorithm": held.algorithm,
                "fingerprint": held.fingerprint,
            })),
        )
        .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
