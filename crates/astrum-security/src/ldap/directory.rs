use std::time::Duration;

use ldap3_proto::proto::{LdapFilter, LdapResultCode};
use serde::Serialize;

use super::client::{ClientError, Connection, Entry, Scope, Search, TlsClient};
use super::config::{AuthScheme, GROUP_OBJECT_CLASSES, GroupType, LdapServer};
use super::sasl::{self, DigestMd5};
use super::{dn, filter, passwords};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct LdapServerError(pub String);

impl LdapServerError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LdapUser {
    pub username: String,
    pub dn: String,
    pub real_name: Option<String>,
    pub email: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Authenticated(LdapUser),
    IncorrectPassword,
    UserNotFound,
}

pub struct Directory<'a> {
    pub server: &'a LdapServer,
    pub bind_password: Option<&'a str>,
    pub tls: &'a TlsClient,
}

impl Directory<'_> {
    fn timeout(&self) -> Duration {
        Duration::from_secs(u64::from(self.server.connection_timeout_seconds.max(1)))
    }

    async fn open(&self) -> Result<Connection, LdapServerError> {
        Connection::open(
            self.server.bare_host(),
            self.server.port,
            self.server.transport(),
            self.tls,
            self.timeout(),
        )
        .await
        .map_err(|error| LdapServerError::new(error.to_string()))
    }

    async fn connect(&self) -> Result<Connection, LdapServerError> {
        let mut connection = self.open().await?;
        match self.system_bind(&mut connection).await {
            Ok(()) => Ok(connection),
            Err(error) => {
                connection.close().await;
                Err(error)
            }
        }
    }

    async fn system_bind(&self, connection: &mut Connection) -> Result<(), LdapServerError> {
        let scheme = self.server.auth_scheme;
        if !scheme.requires_credentials() {
            return Ok(());
        }
        let password = self.bind_password.filter(|password| !password.is_empty());
        let Some(password) = password else {
            return Err(LdapServerError::new(format!(
                "No password is set for the system account of {}",
                self.server.address()
            )));
        };
        let username = self.server.auth_username.clone().unwrap_or_default();
        let result = match scheme {
            AuthScheme::None => Ok(()),
            AuthScheme::Simple => connection.simple_bind(&username, password).await,
            AuthScheme::CramMd5 => cram_md5(connection, &username, password).await,
            AuthScheme::DigestMd5 => {
                let client = DigestMd5 {
                    username: username.clone(),
                    password: password.to_owned(),
                    realm: self.server.auth_realm.clone(),
                    digest_uri: format!("ldap/{}", self.server.bare_host()),
                };
                digest_md5(connection, &client).await
            }
        };
        result.map_err(|error| match error.code() {
            Some(LdapResultCode::InvalidCredentials) => LdapServerError::new(
                "The LDAP server rejected the username or password of the system account",
            ),
            _ => LdapServerError::new(format!(
                "Could not authenticate to {}: {error}",
                self.server.address()
            )),
        })
    }

    pub async fn verify_connection(&self) -> Result<(), LdapServerError> {
        let mut connection = self.connect().await?;
        let result = connection
            .search(Search {
                base: &self.server.search_base,
                subtree: Scope::Base,
                filter: filter::present("objectClass"),
                attributes: Vec::new(),
                limit: 1,
            })
            .await;
        connection.close().await;
        match result {
            Ok(_) => Ok(()),
            Err(error) => match error.code() {
                Some(LdapResultCode::NoSuchObject) => Err(LdapServerError::new(format!(
                    "The search base '{}' does not exist",
                    self.server.search_base
                ))),
                Some(LdapResultCode::InsufficentAccessRights) => Ok(()),
                _ => Err(search_failed(&self.server.search_base, &error)),
            },
        }
    }

    pub async fn find_user(&self, user_id: &str) -> Result<Option<LdapUser>, LdapServerError> {
        let mut connection = self.connect().await?;
        let found = self.find_entry(&mut connection, user_id, false).await;
        let user = match found {
            Ok(Some(entry)) => self.to_user(&mut connection, &entry).await.map(Some),
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        };
        connection.close().await;
        user
    }

    pub async fn search_users(
        &self,
        prefix: Option<&str>,
        limit: usize,
    ) -> Result<Vec<LdapUser>, LdapServerError> {
        let id_attribute = self.user_id_attribute()?;
        let id_filter = match prefix.map(str::trim).filter(|prefix| !prefix.is_empty()) {
            Some(prefix) => filter::starts_with(&id_attribute, prefix),
            None => filter::present(&id_attribute),
        };
        let mut connection = self.connect().await?;
        let result = self
            .search_users_with(&mut connection, id_filter, limit)
            .await;
        connection.close().await;
        result
    }

    async fn search_users_with(
        &self,
        connection: &mut Connection,
        id_filter: LdapFilter,
        limit: usize,
    ) -> Result<Vec<LdapUser>, LdapServerError> {
        let id_attribute = self.user_id_attribute()?;
        let base = self.server.user_base();
        let entries = search(
            connection,
            &base,
            scope(self.server.user_subtree),
            self.user_filter(id_filter)?,
            self.user_attributes(false),
            limit,
        )
        .await?;
        let mut users = Vec::new();
        for entry in entries {
            if entry.text(&id_attribute).is_some() {
                users.push(self.to_user(connection, &entry).await?);
            }
        }
        Ok(users)
    }

    pub async fn authenticate(
        &self,
        user_id: &str,
        password: &str,
    ) -> Result<Outcome, LdapServerError> {
        let mut connection = self.connect().await?;
        let result = self
            .authenticate_with(&mut connection, user_id, password)
            .await;
        connection.close().await;
        result
    }

    async fn authenticate_with(
        &self,
        connection: &mut Connection,
        user_id: &str,
        password: &str,
    ) -> Result<Outcome, LdapServerError> {
        let compare = self.server.user_password_attribute.is_some();
        let Some(entry) = self.find_entry(connection, user_id, compare).await? else {
            return Ok(Outcome::UserNotFound);
        };
        if password.is_empty() {
            return Ok(Outcome::IncorrectPassword);
        }
        let valid = match &self.server.user_password_attribute {
            Some(attribute) => entry
                .first_bytes(attribute)
                .is_some_and(|stored| passwords::matches(password, &stored)),
            None => self.binds_as(&entry.dn, password).await?,
        };
        if !valid {
            return Ok(Outcome::IncorrectPassword);
        }
        Ok(Outcome::Authenticated(
            self.to_user(connection, &entry).await?,
        ))
    }

    async fn binds_as(&self, dn: &str, password: &str) -> Result<bool, LdapServerError> {
        if dn.is_empty() {
            return Ok(false);
        }
        let mut connection = self.open().await?;
        let result = connection.simple_bind(dn, password).await;
        connection.close().await;
        match result {
            Ok(()) => Ok(true),
            Err(ClientError::Result { .. }) => Ok(false),
            Err(error @ ClientError::Connection(_)) => Err(LdapServerError::new(format!(
                "Lost the connection to {}: {error}",
                self.server.address()
            ))),
        }
    }

    pub async fn search_groups(
        &self,
        query: Option<&str>,
        limit: usize,
    ) -> Result<Vec<String>, LdapServerError> {
        if !self.server.maps_groups() {
            return Ok(Vec::new());
        }
        let name_filter = query
            .map(str::trim)
            .filter(|query| !query.is_empty())
            .map(|query| filter::contains(&self.group_name_attribute(), query));
        let mut connection = self.connect().await?;
        let result = self.groups(&mut connection, name_filter, limit).await;
        connection.close().await;
        result
    }

    pub async fn group_exists(&self, group: &str) -> Result<bool, LdapServerError> {
        if !self.server.maps_groups() || group.is_empty() {
            return Ok(false);
        }
        let mut connection = self.connect().await?;
        let result = self
            .groups(
                &mut connection,
                Some(filter::equality(&self.group_name_attribute(), group)),
                10,
            )
            .await;
        connection.close().await;
        Ok(result?
            .iter()
            .any(|found| found.eq_ignore_ascii_case(group)))
    }

    fn group_name_attribute(&self) -> String {
        match self.server.group_type {
            Some(GroupType::Static) => self
                .server
                .group_id_attribute
                .clone()
                .unwrap_or_else(|| "cn".to_owned()),
            _ => "cn".to_owned(),
        }
    }

    async fn groups(
        &self,
        connection: &mut Connection,
        name_filter: Option<LdapFilter>,
        limit: usize,
    ) -> Result<Vec<String>, LdapServerError> {
        let mut names: Vec<String> = Vec::new();
        let mut push = |name: String| {
            if !names.iter().any(|known| known.eq_ignore_ascii_case(&name)) {
                names.push(name);
            }
        };
        match self.server.group_type {
            Some(GroupType::Static) => {
                let id_attribute = self.group_name_attribute();
                let class = filter::equality(
                    "objectClass",
                    self.server
                        .group_object_class
                        .as_deref()
                        .unwrap_or_default(),
                );
                let filter = match name_filter {
                    Some(name) => LdapFilter::And(vec![class, name]),
                    None => class,
                };
                for entry in search(
                    connection,
                    &self.server.group_base(),
                    scope(self.server.group_subtree),
                    filter,
                    vec![id_attribute.clone()],
                    limit,
                )
                .await?
                {
                    for name in entry.texts(&id_attribute) {
                        push(name);
                    }
                }
            }
            Some(GroupType::Dynamic) => {
                let classes = LdapFilter::Or(
                    GROUP_OBJECT_CLASSES
                        .iter()
                        .map(|class| filter::equality("objectClass", class))
                        .collect(),
                );
                let filter = match name_filter {
                    Some(name) => LdapFilter::And(vec![classes, name]),
                    None => classes,
                };
                for entry in search(
                    connection,
                    &self.server.search_base,
                    Scope::Subtree,
                    filter,
                    Vec::new(),
                    limit,
                )
                .await?
                {
                    push(group_name(&entry.dn));
                }
            }
            None => {}
        }
        Ok(names)
    }

    fn user_id_attribute(&self) -> Result<String, LdapServerError> {
        self.server
            .user_id_attribute
            .clone()
            .ok_or_else(|| LdapServerError::new("The username attribute is not set"))
    }

    fn user_filter(&self, id_filter: LdapFilter) -> Result<LdapFilter, LdapServerError> {
        let mut parts = Vec::new();
        if let Some(class) = &self.server.user_object_class {
            parts.push(filter::equality("objectClass", class));
        }
        parts.push(id_filter);
        if let Some(extra) = &self.server.user_ldap_filter {
            parts.push(filter::parse(extra).map_err(|_| {
                LdapServerError::new(format!("The user filter '{extra}' is not valid"))
            })?);
        }
        Ok(LdapFilter::And(parts))
    }

    fn user_attributes(&self, with_password: bool) -> Vec<String> {
        let server = self.server;
        let candidates = [
            server.user_id_attribute.clone(),
            server.user_real_name_attribute.clone(),
            server.user_email_address_attribute.clone(),
            match server.group_type {
                Some(GroupType::Dynamic) if server.ldap_groups_as_roles => {
                    server.user_member_of_attribute.clone()
                }
                _ => None,
            },
            with_password
                .then(|| server.user_password_attribute.clone())
                .flatten(),
        ];
        let mut attributes: Vec<String> = Vec::new();
        for attribute in candidates.into_iter().flatten() {
            if !attributes
                .iter()
                .any(|known| known.eq_ignore_ascii_case(&attribute))
            {
                attributes.push(attribute);
            }
        }
        attributes
    }

    async fn find_entry(
        &self,
        connection: &mut Connection,
        user_id: &str,
        with_password: bool,
    ) -> Result<Option<Entry>, LdapServerError> {
        if user_id.trim().is_empty() {
            return Ok(None);
        }
        let id_attribute = self.user_id_attribute()?;
        let entries = search(
            connection,
            &self.server.user_base(),
            scope(self.server.user_subtree),
            self.user_filter(filter::equality(&id_attribute, user_id))?,
            self.user_attributes(with_password),
            2,
        )
        .await?;
        if entries.len() > 1 {
            tracing::warn!(
                server = %self.server.name,
                user = user_id,
                dn = %entries[0].dn,
                "more than one LDAP entry matches the user; the first one is used"
            );
        }
        Ok(entries.into_iter().next())
    }

    async fn to_user(
        &self,
        connection: &mut Connection,
        entry: &Entry,
    ) -> Result<LdapUser, LdapServerError> {
        let id_attribute = self.user_id_attribute()?;
        let username = entry.text(&id_attribute).unwrap_or_default();
        let groups = self.membership(connection, entry, &username).await?;
        Ok(LdapUser {
            username,
            dn: entry.dn.clone(),
            real_name: self
                .server
                .user_real_name_attribute
                .as_deref()
                .and_then(|attribute| entry.text(attribute)),
            email: self
                .server
                .user_email_address_attribute
                .as_deref()
                .and_then(|attribute| entry.text(attribute)),
            groups,
        })
    }

    async fn membership(
        &self,
        connection: &mut Connection,
        user: &Entry,
        username: &str,
    ) -> Result<Vec<String>, LdapServerError> {
        if !self.server.maps_groups() {
            return Ok(Vec::new());
        }
        let mut groups: Vec<String> = Vec::new();
        let mut push = |name: String| {
            if !name.is_empty() && !groups.iter().any(|known| known.eq_ignore_ascii_case(&name)) {
                groups.push(name);
            }
        };
        match self.server.group_type {
            Some(GroupType::Dynamic) => {
                if let Some(attribute) = &self.server.user_member_of_attribute {
                    for value in user.texts(attribute) {
                        push(group_name(&value));
                    }
                }
            }
            Some(GroupType::Static) => {
                let format = self
                    .server
                    .group_member_format
                    .as_deref()
                    .unwrap_or_default();
                let member = format
                    .replace("${username}", username)
                    .replace("${dn}", &user.dn);
                let id_attribute = self.group_name_attribute();
                let filter = LdapFilter::And(vec![
                    filter::equality(
                        "objectClass",
                        self.server
                            .group_object_class
                            .as_deref()
                            .unwrap_or_default(),
                    ),
                    filter::equality(
                        self.server
                            .group_member_attribute
                            .as_deref()
                            .unwrap_or_default(),
                        &member,
                    ),
                ]);
                for entry in search(
                    connection,
                    &self.server.group_base(),
                    scope(self.server.group_subtree),
                    filter,
                    vec![id_attribute.clone()],
                    0,
                )
                .await?
                {
                    if let Some(name) = entry.text(&id_attribute) {
                        push(name);
                    }
                }
            }
            None => {}
        }
        groups.sort_by_key(|name| name.to_lowercase());
        Ok(groups)
    }
}

pub fn group_name(value: &str) -> String {
    dn::first_value(value).unwrap_or_else(|| value.to_owned())
}

fn scope(subtree: bool) -> Scope {
    match subtree {
        true => Scope::Subtree,
        false => Scope::One,
    }
}

async fn search(
    connection: &mut Connection,
    base: &str,
    scope: Scope,
    filter: LdapFilter,
    attributes: Vec<String>,
    limit: usize,
) -> Result<Vec<Entry>, LdapServerError> {
    let result = connection
        .search(Search {
            base,
            subtree: scope,
            filter,
            attributes,
            limit,
        })
        .await;
    match result {
        Ok(entries) => Ok(entries),
        Err(error) if error.code() == Some(&LdapResultCode::NoSuchObject) => {
            tracing::debug!(base, "the LDAP search base does not exist");
            Ok(Vec::new())
        }
        Err(error) => Err(search_failed(base, &error)),
    }
}

fn search_failed(base: &str, error: &ClientError) -> LdapServerError {
    LdapServerError::new(format!("LDAP search under '{base}' failed: {error}"))
}

async fn cram_md5(
    connection: &mut Connection,
    username: &str,
    password: &str,
) -> Result<(), ClientError> {
    let (result, challenge) = connection.sasl_bind("CRAM-MD5", Vec::new()).await?;
    if result.code != LdapResultCode::SaslBindInProgress {
        return Err(super::client::refused(&result));
    }
    let response = sasl::cram_md5(username, password, &challenge.unwrap_or_default());
    let (result, _) = connection.sasl_bind("CRAM-MD5", response).await?;
    match result.code {
        LdapResultCode::Success => Ok(()),
        _ => Err(super::client::refused(&result)),
    }
}

async fn digest_md5(connection: &mut Connection, client: &DigestMd5) -> Result<(), ClientError> {
    let (result, challenge) = connection.sasl_bind("DIGEST-MD5", Vec::new()).await?;
    if result.code != LdapResultCode::SaslBindInProgress {
        return Err(super::client::refused(&result));
    }
    let reply = client
        .respond(&challenge.unwrap_or_default())
        .map_err(|error| ClientError::Connection(error.to_string()))?;
    let (mut result, mut last) = connection.sasl_bind("DIGEST-MD5", reply.response).await?;
    if result.code == LdapResultCode::SaslBindInProgress {
        check_rspauth(last.as_deref(), &reply.expected_rspauth)?;
        let (next, creds) = connection.sasl_bind("DIGEST-MD5", Vec::new()).await?;
        result = next;
        last = creds;
        if result.code == LdapResultCode::Success {
            return Ok(());
        }
    }
    match result.code {
        LdapResultCode::Success => {
            if last.is_some() {
                check_rspauth(last.as_deref(), &reply.expected_rspauth)?;
            }
            Ok(())
        }
        _ => Err(super::client::refused(&result)),
    }
}

fn check_rspauth(received: Option<&[u8]>, expected: &str) -> Result<(), ClientError> {
    match received.and_then(sasl::rspauth) {
        Some(rspauth) if rspauth == expected => Ok(()),
        _ => Err(ClientError::Connection(
            "the server did not prove that it knows the password".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_names_come_from_the_first_rdn() {
        assert_eq!(
            group_name("cn=developers,ou=groups,dc=example,dc=com"),
            "developers"
        );
        assert_eq!(
            group_name("CN=Domain Admins,CN=Users,DC=corp"),
            "Domain Admins"
        );
        assert_eq!(group_name("plain"), "plain");
    }
}
