use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_security::ldap::{
    self, AuthScheme, Check, Directory, Draft, LdapServer, LdapUser, Outcome, Problem, TEMPLATES,
    Template,
};
use astrum_security::model::REALMS_KEY;
use astrum_security::realm;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sea_orm::{ConnectionTrait, DatabaseTransaction, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::SECURITY;
use crate::{ApiError, AppState, Caller, FieldError, Json};

const DOMAIN: &str = "security.ldap";
const LOCK: i64 = i64::from_be_bytes(*b"astrldap");

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{SECURITY}/ldap"), get(list).post(create))
        .route(&format!("{SECURITY}/ldap/templates"), get(templates))
        .route(&format!("{SECURITY}/ldap/change-order"), post(change_order))
        .route(&format!("{SECURITY}/ldap/cache"), delete(clear_cache))
        .route(
            &format!("{SECURITY}/ldap/verify-connection"),
            post(verify_connection),
        )
        .route(
            &format!("{SECURITY}/ldap/verify-user-mapping"),
            post(verify_user_mapping),
        )
        .route(&format!("{SECURITY}/ldap/verify-login"), post(verify_login))
        .route(
            &format!("{SECURITY}/ldap/{{name}}"),
            get(read).put(update).delete(remove),
        )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerXo {
    id: String,
    order: usize,
    name: String,
    protocol: ldap::Protocol,
    use_trust_store: bool,
    use_start_tls: bool,
    host: String,
    port: u16,
    search_base: String,
    auth_scheme: AuthScheme,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_realm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_username: Option<String>,
    connection_timeout_seconds: u32,
    connection_retry_delay_seconds: u32,
    max_incidents_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_base_dn: Option<String>,
    user_subtree: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_object_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ldap_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id_attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_real_name_attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_email_address_attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_password_attribute: Option<String>,
    ldap_groups_as_roles: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_type: Option<ldap::GroupType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_base_dn: Option<String>,
    group_subtree: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_object_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_id_attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_member_attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_member_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_member_of_attribute: Option<String>,
}

impl ServerXo {
    fn of(server: &LdapServer, order: usize) -> Self {
        let server = server.clone();
        Self {
            id: server.id,
            order,
            name: server.name,
            protocol: server.protocol,
            use_trust_store: server.use_trust_store,
            use_start_tls: server.use_start_tls,
            host: server.host,
            port: server.port,
            search_base: server.search_base,
            auth_scheme: server.auth_scheme,
            auth_realm: server.auth_realm,
            auth_username: server.auth_username,
            connection_timeout_seconds: server.connection_timeout_seconds,
            connection_retry_delay_seconds: server.connection_retry_delay_seconds,
            max_incidents_count: server.max_incidents_count,
            user_base_dn: server.user_base_dn,
            user_subtree: server.user_subtree,
            user_object_class: server.user_object_class,
            user_ldap_filter: server.user_ldap_filter,
            user_id_attribute: server.user_id_attribute,
            user_real_name_attribute: server.user_real_name_attribute,
            user_email_address_attribute: server.user_email_address_attribute,
            user_password_attribute: server.user_password_attribute,
            ldap_groups_as_roles: server.ldap_groups_as_roles,
            group_type: server.group_type,
            group_base_dn: server.group_base_dn,
            group_subtree: server.group_subtree,
            group_object_class: server.group_object_class,
            group_id_attribute: server.group_id_attribute,
            group_member_attribute: server.group_member_attribute,
            group_member_format: server.group_member_format,
            user_member_of_attribute: server.user_member_of_attribute,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserXo {
    username: String,
    real_name: Option<String>,
    email: Option<String>,
    membership: Vec<String>,
}

impl From<LdapUser> for UserXo {
    fn from(user: LdapUser) -> Self {
        Self {
            username: user.username,
            real_name: user.real_name,
            email: user.email,
            membership: user.groups,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Existing {
    existing_server_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyLogin {
    base64_username: Option<String>,
    base64_password: Option<String>,
    ldap_server: Option<Draft>,
}

fn not_found(name: &str) -> ApiError {
    ApiError::not_found(format!("LDAP server '{name}' does not exist"))
}

fn invalid(problems: Vec<Problem>) -> ApiError {
    ApiError::Invalid(
        problems
            .into_iter()
            .map(|problem| FieldError {
                id: problem.field.to_owned(),
                message: problem.message,
            })
            .collect(),
    )
}

fn required(draft: Option<Draft>) -> Result<Draft, ApiError> {
    draft.ok_or_else(|| ApiError::BadRequest("The LDAP server is missing".to_owned()))
}

fn find<'a>(servers: &'a [LdapServer], name: &str) -> Option<&'a LdapServer> {
    servers
        .iter()
        .find(|server| server.name == name)
        .or_else(|| {
            servers
                .iter()
                .find(|server| server.name.eq_ignore_ascii_case(name))
        })
}

fn existing<'a>(
    servers: &'a [LdapServer],
    id: Option<&str>,
    name: Option<&str>,
) -> Option<&'a LdapServer> {
    id.filter(|id| !id.is_empty())
        .and_then(|id| servers.iter().find(|server| server.id == id))
        .or_else(|| name.and_then(|name| find(servers, name)))
}

fn password(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.is_empty())
}

fn check_name(
    servers: &[LdapServer],
    server: &LdapServer,
    own_id: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(other) = servers
        .iter()
        .filter(|other| Some(other.id.as_str()) != own_id)
        .find(|other| other.name.eq_ignore_ascii_case(&server.name))
    {
        return Err(ApiError::invalid(
            "name",
            format!("An LDAP server named '{}' already exists", other.name),
        ));
    }
    Ok(())
}

async fn stored(txn: &impl ConnectionTrait) -> Result<Vec<LdapServer>, ApiError> {
    Ok(setting::get(txn, ldap::SERVERS_KEY)
        .await?
        .unwrap_or_default())
}

async fn locked(state: &AppState) -> Result<(DatabaseTransaction, Vec<LdapServer>), ApiError> {
    let txn = state.db().conn().begin().await?;
    txn.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({LOCK})"))
        .await?;
    let servers = stored(&txn).await?;
    Ok((txn, servers))
}

async fn changed(state: &AppState, caller: &Caller, event: Event) -> Result<(), ApiError> {
    state.security().changed().await?;
    state.security().ldap().clear();
    state.audit(caller, event).await;
    Ok(())
}

fn describe(server: &LdapServer) -> serde_json::Value {
    json!({
        "id": server.id,
        "name": server.name,
        "url": server.url(),
        "authScheme": server.auth_scheme.name(),
        "authUsername": server.auth_username,
        "groupType": server.group_type,
    })
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<axum::Json<Vec<ServerXo>>, ApiError> {
    caller.require("astrum:ldap:read")?;
    let model = state.security().model();
    Ok(axum::Json(
        model
            .ldap_servers
            .iter()
            .enumerate()
            .map(|(index, server)| ServerXo::of(server, index + 1))
            .collect(),
    ))
}

async fn templates(caller: Caller) -> Result<axum::Json<&'static [Template]>, ApiError> {
    caller.require("astrum:ldap:read")?;
    Ok(axum::Json(&TEMPLATES))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<axum::Json<ServerXo>, ApiError> {
    caller.require("astrum:ldap:read")?;
    let model = state.security().model();
    let server = find(&model.ldap_servers, &name).ok_or_else(|| not_found(&name))?;
    let order = model
        .ldap_servers
        .iter()
        .position(|candidate| candidate.id == server.id)
        .unwrap_or_default();
    Ok(axum::Json(ServerXo::of(server, order + 1)))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Json(draft): Json<Option<Draft>>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:ldap:create")?;
    let draft = required(draft)?;
    let mut server = ldap::config::check(&draft, Check::Everything).map_err(invalid)?;
    let (txn, mut servers) = locked(&state).await?;
    check_name(&servers, &server, None)?;
    server.id = uuid::Uuid::new_v4().to_string();
    server.auth_password = match server.auth_scheme.requires_credentials() {
        true => match password(draft.auth_password.as_deref()) {
            Some(password) => Some(state.secrets().seal(password)),
            None => return Err(ApiError::invalid("authPassword", "Password is required")),
        },
        false => None,
    };
    servers.push(server.clone());
    setting::put(&txn, ldap::SERVERS_KEY, &servers).await?;
    let mut realms: Vec<String> = setting::get(&txn, REALMS_KEY)
        .await?
        .unwrap_or_else(|| state.security().model().realms.clone());
    let activated = !realms.iter().any(|id| id == realm::LDAP);
    if activated {
        realms.push(realm::LDAP.to_owned());
        setting::put(&txn, REALMS_KEY, &realms).await?;
    }
    txn.commit().await?;
    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "created", &server.name).with(describe(&server)),
    )
    .await?;
    if activated {
        state
            .audit(
                &caller,
                Event::new("security.realm", "updated", "active").with(json!({"active": realms})),
            )
            .await;
    }
    Ok(StatusCode::CREATED)
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(draft): Json<Option<Draft>>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:ldap:update")?;
    let draft = required(draft)?;
    let mut server = ldap::config::check(&draft, Check::Everything).map_err(invalid)?;
    let (txn, mut servers) = locked(&state).await?;
    let current = existing(&servers, draft.id.as_deref(), Some(&name))
        .cloned()
        .ok_or_else(|| not_found(&name))?;
    check_name(&servers, &server, Some(&current.id))?;
    server.id = current.id.clone();
    server.auth_password = match (
        server.auth_scheme.requires_credentials(),
        password(draft.auth_password.as_deref()),
    ) {
        (false, _) => None,
        (true, Some(password)) => Some(state.secrets().seal(password)),
        (true, None) if current.auth_password.is_some() && current.same_bind_target(&server) => {
            current.auth_password.clone()
        }
        (true, None) => return Err(ApiError::invalid("authPassword", "Password is required")),
    };
    if let Some(slot) = servers
        .iter_mut()
        .find(|candidate| candidate.id == current.id)
    {
        *slot = server.clone();
    }
    setting::put(&txn, ldap::SERVERS_KEY, &servers).await?;
    txn.commit().await?;
    let mut attributes = describe(&server);
    if current.name != server.name {
        attributes["previousName"] = json!(current.name);
    }
    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", &server.name).with(attributes),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:ldap:delete")?;
    let (txn, mut servers) = locked(&state).await?;
    let current = find(&servers, &name)
        .cloned()
        .ok_or_else(|| not_found(&name))?;
    servers.retain(|server| server.id != current.id);
    setting::put(&txn, ldap::SERVERS_KEY, &servers).await?;
    txn.commit().await?;
    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "deleted", &current.name).with(describe(&current)),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn change_order(
    State(state): State<AppState>,
    caller: Caller,
    Json(names): Json<Option<Vec<String>>>,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:ldap:update")?;
    let names =
        names.ok_or_else(|| ApiError::BadRequest("The server order is missing".to_owned()))?;
    let (txn, servers) = locked(&state).await?;
    let mut problems = Vec::new();
    let mut ordered: Vec<LdapServer> = Vec::new();
    for name in &names {
        match find(&servers, name) {
            Some(server) if !ordered.iter().any(|known| known.id == server.id) => {
                ordered.push(server.clone());
            }
            Some(_) => {}
            None => problems.push(FieldError {
                id: "order".to_owned(),
                message: format!("LDAP server '{name}' does not exist"),
            }),
        }
    }
    if !problems.is_empty() {
        return Err(ApiError::Invalid(problems));
    }
    for server in &servers {
        if !ordered.iter().any(|known| known.id == server.id) {
            ordered.push(server.clone());
        }
    }
    setting::put(&txn, ldap::SERVERS_KEY, &ordered).await?;
    txn.commit().await?;
    let order: Vec<&str> = ordered.iter().map(|server| server.name.as_str()).collect();
    changed(
        &state,
        &caller,
        Event::new(DOMAIN, "updated", "order").with(json!({"order": order})),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn clear_cache(
    State(state): State<AppState>,
    caller: Caller,
) -> Result<StatusCode, ApiError> {
    caller.require("astrum:ldap:delete")?;
    state.security().ldap().clear();
    Ok(StatusCode::NO_CONTENT)
}

fn may_verify(caller: &Caller) -> Result<(), ApiError> {
    match caller.require("astrum:ldap:create") {
        Ok(_) => Ok(()),
        Err(_) => caller.require("astrum:ldap:update").map(|_| ()),
    }
}

fn bind_password(
    state: &AppState,
    server: &LdapServer,
    draft: &Draft,
    existing_name: Option<&str>,
) -> Result<Option<String>, ApiError> {
    if !server.auth_scheme.requires_credentials() {
        return Ok(None);
    }
    if let Some(password) = password(draft.auth_password.as_deref()) {
        return Ok(Some(password.to_owned()));
    }
    let model = state.security().model();
    let stored = existing(&model.ldap_servers, draft.id.as_deref(), existing_name)
        .filter(|stored| stored.auth_password.is_some() && stored.same_bind_target(server));
    match stored {
        Some(stored) => ldap::bind_password(stored, Some(state.secrets()))
            .map_err(|error| ApiError::BadRequest(error.0)),
        None => Err(ApiError::invalid("authPassword", "Password is required")),
    }
}

async fn verify_connection(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<Existing>,
    Json(draft): Json<Option<Draft>>,
) -> Result<StatusCode, ApiError> {
    may_verify(&caller)?;
    let draft = required(draft)?;
    let server = ldap::config::check(&draft, Check::Connection).map_err(invalid)?;
    let bind = bind_password(
        &state,
        &server,
        &draft,
        query.existing_server_name.as_deref(),
    )?;
    let tls = state.security().ldap().tls_for(&server).await;
    Directory {
        server: &server,
        bind_password: bind.as_deref(),
        tls: &tls,
    }
    .verify_connection()
    .await
    .map_err(|error| ApiError::BadRequest(error.0))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn verify_user_mapping(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<Existing>,
    Json(draft): Json<Option<Draft>>,
) -> Result<axum::Json<Vec<UserXo>>, ApiError> {
    may_verify(&caller)?;
    let draft = required(draft)?;
    let server = ldap::config::check(&draft, Check::Everything).map_err(invalid)?;
    let bind = bind_password(
        &state,
        &server,
        &draft,
        query.existing_server_name.as_deref(),
    )?;
    let tls = state.security().ldap().tls_for(&server).await;
    let users = Directory {
        server: &server,
        bind_password: bind.as_deref(),
        tls: &tls,
    }
    .search_users(None, ldap::VERIFY_USER_LIMIT)
    .await
    .map_err(|error| ApiError::BadRequest(error.0))?;
    Ok(axum::Json(users.into_iter().map(UserXo::from).collect()))
}

fn decode(field: &str, value: Option<&str>) -> Result<String, ApiError> {
    let Some(value) = value else {
        return Ok(String::new());
    };
    STANDARD
        .decode(value.trim())
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or_else(|| ApiError::invalid(field, "Not valid base64"))
}

async fn verify_login(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<Existing>,
    Json(body): Json<Option<VerifyLogin>>,
) -> Result<StatusCode, ApiError> {
    may_verify(&caller)?;
    let body = body.unwrap_or(VerifyLogin {
        base64_username: None,
        base64_password: None,
        ldap_server: None,
    });
    let Some(draft) = body.ldap_server else {
        return Err(ApiError::invalid(
            "ldapServer",
            "The LDAP server is missing",
        ));
    };
    let server = ldap::config::check(&draft, Check::Everything).map_err(invalid)?;
    let username = decode("base64Username", body.base64_username.as_deref())?;
    let password = decode("base64Password", body.base64_password.as_deref())?;
    if username.is_empty() || password.is_empty() {
        return Err(ApiError::BadRequest(
            "Username and password are required".to_owned(),
        ));
    }
    let bind = bind_password(
        &state,
        &server,
        &draft,
        query.existing_server_name.as_deref(),
    )?;
    let tls = state.security().ldap().tls_for(&server).await;
    let outcome = Directory {
        server: &server,
        bind_password: bind.as_deref(),
        tls: &tls,
    }
    .authenticate(&username, &password)
    .await
    .map_err(|error| ApiError::BadRequest(error.0))?;
    match outcome {
        Outcome::Authenticated(_) => Ok(StatusCode::NO_CONTENT),
        Outcome::IncorrectPassword => Err(ApiError::BadRequest(format!(
            "The password for user '{username}' is incorrect"
        ))),
        Outcome::UserNotFound => Err(ApiError::BadRequest(format!(
            "User '{username}' was not found"
        ))),
    }
}
