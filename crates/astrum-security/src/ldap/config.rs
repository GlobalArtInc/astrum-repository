use serde::{Deserialize, Serialize};

use super::client::Transport;
use super::{dn, filter};

pub const SERVERS_KEY: &str = "security.ldap.servers";
pub const DEFAULT_CONNECTION_TIMEOUT: u32 = 30;
pub const DEFAULT_RETRY_DELAY: u32 = 300;
pub const DEFAULT_MAX_INCIDENTS: u32 = 3;
pub const MAX_CONNECTION_TIMEOUT: i64 = 3600;
pub const GROUP_OBJECT_CLASSES: [&str; 5] = [
    "group",
    "groupOfNames",
    "groupOfUniqueNames",
    "groupOfURLs",
    "posixGroup",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    Ldap,
    Ldaps,
}

impl Protocol {
    pub fn parse(value: &str) -> Option<Self> {
        match normalized(value).as_str() {
            "LDAP" => Some(Protocol::Ldap),
            "LDAPS" => Some(Protocol::Ldaps),
            _ => None,
        }
    }

    pub fn default_port(self) -> u16 {
        match self {
            Protocol::Ldap => 389,
            Protocol::Ldaps => 636,
        }
    }

    pub fn scheme(self) -> &'static str {
        match self {
            Protocol::Ldap => "ldap",
            Protocol::Ldaps => "ldaps",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthScheme {
    None,
    Simple,
    DigestMd5,
    CramMd5,
}

impl AuthScheme {
    pub fn parse(value: &str) -> Option<Self> {
        match normalized(value).as_str() {
            "NONE" => Some(AuthScheme::None),
            "SIMPLE" => Some(AuthScheme::Simple),
            "DIGEST_MD5" => Some(AuthScheme::DigestMd5),
            "CRAM_MD5" => Some(AuthScheme::CramMd5),
            _ => None,
        }
    }

    pub fn requires_credentials(self) -> bool {
        self != AuthScheme::None
    }

    pub fn uses_realm(self) -> bool {
        matches!(self, AuthScheme::DigestMd5 | AuthScheme::CramMd5)
    }

    pub fn name(self) -> &'static str {
        match self {
            AuthScheme::None => "NONE",
            AuthScheme::Simple => "SIMPLE",
            AuthScheme::DigestMd5 => "DIGEST_MD5",
            AuthScheme::CramMd5 => "CRAM_MD5",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GroupType {
    Static,
    Dynamic,
}

impl GroupType {
    pub fn parse(value: &str) -> Option<Self> {
        match normalized(value).as_str() {
            "STATIC" => Some(GroupType::Static),
            "DYNAMIC" => Some(GroupType::Dynamic),
            _ => None,
        }
    }

    pub fn ui_value(self) -> &'static str {
        match self {
            GroupType::Static => "static",
            GroupType::Dynamic => "dynamic",
        }
    }
}

fn normalized(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_uppercase()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdapServer {
    pub id: String,
    pub name: String,
    pub protocol: Protocol,
    #[serde(default)]
    pub use_start_tls: bool,
    #[serde(default)]
    pub use_trust_store: bool,
    pub host: String,
    pub port: u16,
    pub search_base: String,
    pub auth_scheme: AuthScheme,
    #[serde(default)]
    pub auth_realm: Option<String>,
    #[serde(default)]
    pub auth_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_password: Option<String>,
    pub connection_timeout_seconds: u32,
    pub connection_retry_delay_seconds: u32,
    pub max_incidents_count: u32,
    #[serde(default)]
    pub user_base_dn: Option<String>,
    #[serde(default)]
    pub user_subtree: bool,
    #[serde(default)]
    pub user_object_class: Option<String>,
    #[serde(default)]
    pub user_ldap_filter: Option<String>,
    #[serde(default)]
    pub user_id_attribute: Option<String>,
    #[serde(default)]
    pub user_real_name_attribute: Option<String>,
    #[serde(default)]
    pub user_email_address_attribute: Option<String>,
    #[serde(default)]
    pub user_password_attribute: Option<String>,
    #[serde(default)]
    pub ldap_groups_as_roles: bool,
    #[serde(default)]
    pub group_type: Option<GroupType>,
    #[serde(default)]
    pub group_base_dn: Option<String>,
    #[serde(default)]
    pub group_subtree: bool,
    #[serde(default)]
    pub group_object_class: Option<String>,
    #[serde(default)]
    pub group_id_attribute: Option<String>,
    #[serde(default)]
    pub group_member_attribute: Option<String>,
    #[serde(default)]
    pub group_member_format: Option<String>,
    #[serde(default)]
    pub user_member_of_attribute: Option<String>,
}

impl LdapServer {
    pub fn url(&self) -> String {
        format!(
            "{}://{}:{}/{}",
            self.protocol.scheme(),
            self.host,
            self.port,
            self.search_base
        )
    }

    pub fn address(&self) -> String {
        format!("{}://{}:{}", self.protocol.scheme(), self.host, self.port)
    }

    pub fn transport(&self) -> Transport {
        match (self.protocol, self.use_start_tls) {
            (Protocol::Ldaps, _) => Transport::Ldaps,
            (Protocol::Ldap, true) => Transport::StartTls,
            (Protocol::Ldap, false) => Transport::Plain,
        }
    }

    pub fn bare_host(&self) -> &str {
        self.host
            .strip_prefix('[')
            .and_then(|host| host.strip_suffix(']'))
            .unwrap_or(&self.host)
    }

    pub fn maps_groups(&self) -> bool {
        self.ldap_groups_as_roles && self.group_type.is_some()
    }

    pub fn user_base(&self) -> String {
        relative_to(self.user_base_dn.as_deref(), &self.search_base)
    }

    pub fn group_base(&self) -> String {
        relative_to(self.group_base_dn.as_deref(), &self.search_base)
    }

    pub fn same_bind_target(&self, draft: &LdapServer) -> bool {
        self.protocol == draft.protocol
            && self.use_start_tls == draft.use_start_tls
            && self.host.eq_ignore_ascii_case(&draft.host)
            && self.port == draft.port
            && self.auth_scheme == draft.auth_scheme
            && self.auth_username == draft.auth_username
            && self.auth_realm == draft.auth_realm
    }
}

fn relative_to(relative: Option<&str>, base: &str) -> String {
    match relative.filter(|relative| !relative.is_empty()) {
        Some(relative) => format!("{relative},{base}"),
        None => base.to_owned(),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: Option<String>,
    pub order: Option<i64>,
    pub name: Option<String>,
    pub protocol: Option<String>,
    pub use_start_tls: Option<bool>,
    pub use_trust_store: Option<bool>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub search_base: Option<String>,
    pub auth_scheme: Option<String>,
    pub auth_realm: Option<String>,
    pub auth_username: Option<String>,
    pub auth_password: Option<String>,
    pub connection_timeout_seconds: Option<i64>,
    pub connection_retry_delay_seconds: Option<i64>,
    pub max_incidents_count: Option<i64>,
    pub user_base_dn: Option<String>,
    pub user_subtree: Option<bool>,
    pub user_object_class: Option<String>,
    pub user_ldap_filter: Option<String>,
    pub user_id_attribute: Option<String>,
    pub user_real_name_attribute: Option<String>,
    pub user_email_address_attribute: Option<String>,
    pub user_password_attribute: Option<String>,
    pub ldap_groups_as_roles: Option<bool>,
    pub group_type: Option<String>,
    pub group_base_dn: Option<String>,
    pub group_subtree: Option<bool>,
    pub group_object_class: Option<String>,
    pub group_id_attribute: Option<String>,
    pub group_member_attribute: Option<String>,
    pub group_member_format: Option<String>,
    pub user_member_of_attribute: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub field: &'static str,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Connection,
    Everything,
}

struct Problems(Vec<Problem>);

impl Problems {
    fn add(&mut self, field: &'static str, message: impl Into<String>) {
        self.0.push(Problem {
            field,
            message: message.into(),
        });
    }

    fn required(&mut self, field: &'static str, label: &str, value: &Option<String>) -> bool {
        match value {
            Some(_) => true,
            None => {
                self.add(field, format!("{label} is required"));
                false
            }
        }
    }

    fn dn(&mut self, field: &'static str, label: &str, value: &str) {
        if !dn::is_valid(value) {
            self.add(field, format!("{label} is not a valid DN"));
        }
    }
}

fn text(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub fn is_host(host: &str) -> bool {
    let bare = host
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(host);
    if bare.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    let host = host.strip_suffix('.').unwrap_or(host);
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

pub fn check(draft: &Draft, scope: Check) -> Result<LdapServer, Vec<Problem>> {
    let mut problems = Problems(Vec::new());
    let protocol = draft.protocol.as_deref().and_then(Protocol::parse);
    if protocol.is_none() {
        problems.add("protocol", "Protocol must be ldap or ldaps");
    }
    let protocol = protocol.unwrap_or(Protocol::Ldap);
    let host = text(&draft.host);
    if problems.required("host", "Host", &host) && !is_host(host.as_deref().unwrap_or_default()) {
        problems.add("host", "Host must be a host name or an IP address");
    }
    let port = draft.port.unwrap_or(i64::from(protocol.default_port()));
    if !(1..=65535).contains(&port) {
        problems.add("port", "Port must be between 1 and 65535");
    }
    let search_base = text(&draft.search_base);
    if problems.required("searchBase", "Search base", &search_base) {
        problems.dn(
            "searchBase",
            "Search base",
            search_base.as_deref().unwrap_or_default(),
        );
    }
    let auth_scheme = draft.auth_scheme.as_deref().and_then(AuthScheme::parse);
    let mut auth_username = text(&draft.auth_username);
    let mut auth_realm = text(&draft.auth_realm);
    match auth_scheme {
        None => problems.add(
            "authScheme",
            "Authentication method must be none, simple, DIGEST-MD5 or CRAM-MD5",
        ),
        Some(scheme) => {
            if scheme.requires_credentials() {
                problems.required("authUsername", "Username or DN", &auth_username);
            } else {
                auth_username = None;
            }
            if !scheme.uses_realm() {
                auth_realm = None;
            }
        }
    }
    let timeout = draft
        .connection_timeout_seconds
        .unwrap_or(i64::from(DEFAULT_CONNECTION_TIMEOUT));
    if !(1..=MAX_CONNECTION_TIMEOUT).contains(&timeout) {
        problems.add(
            "connectionTimeoutSeconds",
            format!("Connection timeout must be between 1 and {MAX_CONNECTION_TIMEOUT} seconds"),
        );
    }
    let retry = draft
        .connection_retry_delay_seconds
        .unwrap_or(i64::from(DEFAULT_RETRY_DELAY));
    if !(0..=i64::from(u32::MAX)).contains(&retry) {
        problems.add(
            "connectionRetryDelaySeconds",
            "Retry delay must not be negative",
        );
    }
    let incidents = draft
        .max_incidents_count
        .unwrap_or(i64::from(DEFAULT_MAX_INCIDENTS));
    if !(0..=i64::from(u32::MAX)).contains(&incidents) {
        problems.add(
            "maxIncidentsCount",
            "Max incidents count must not be negative",
        );
    }

    let group_type = draft.group_type.as_deref().and_then(GroupType::parse);
    let groups_as_roles = draft.ldap_groups_as_roles.unwrap_or(
        draft
            .group_type
            .as_deref()
            .is_some_and(|kind| !kind.trim().is_empty()),
    );
    let mut server = LdapServer {
        id: text(&draft.id).unwrap_or_default(),
        name: text(&draft.name).unwrap_or_default(),
        protocol,
        use_start_tls: protocol == Protocol::Ldap && draft.use_start_tls.unwrap_or(false),
        use_trust_store: draft.use_trust_store.unwrap_or(false)
            && (protocol == Protocol::Ldaps || draft.use_start_tls.unwrap_or(false)),
        host: host.unwrap_or_default(),
        port: u16::try_from(port).unwrap_or_default(),
        search_base: search_base.unwrap_or_default(),
        auth_scheme: auth_scheme.unwrap_or(AuthScheme::None),
        auth_realm,
        auth_username,
        auth_password: None,
        connection_timeout_seconds: u32::try_from(timeout).unwrap_or(DEFAULT_CONNECTION_TIMEOUT),
        connection_retry_delay_seconds: u32::try_from(retry).unwrap_or(DEFAULT_RETRY_DELAY),
        max_incidents_count: u32::try_from(incidents).unwrap_or(DEFAULT_MAX_INCIDENTS),
        user_base_dn: text(&draft.user_base_dn),
        user_subtree: draft.user_subtree.unwrap_or(false),
        user_object_class: text(&draft.user_object_class),
        user_ldap_filter: text(&draft.user_ldap_filter),
        user_id_attribute: text(&draft.user_id_attribute),
        user_real_name_attribute: text(&draft.user_real_name_attribute),
        user_email_address_attribute: text(&draft.user_email_address_attribute),
        user_password_attribute: text(&draft.user_password_attribute),
        ldap_groups_as_roles: groups_as_roles,
        group_type: groups_as_roles.then_some(group_type).flatten(),
        group_base_dn: text(&draft.group_base_dn),
        group_subtree: draft.group_subtree.unwrap_or(false),
        group_object_class: text(&draft.group_object_class),
        group_id_attribute: text(&draft.group_id_attribute),
        group_member_attribute: text(&draft.group_member_attribute),
        group_member_format: text(&draft.group_member_format),
        user_member_of_attribute: text(&draft.user_member_of_attribute),
    };
    if scope == Check::Everything {
        problems.required("name", "Name", &text(&draft.name));
        check_mapping(&mut server, &mut problems);
    }
    match problems.0.is_empty() {
        true => Ok(server),
        false => Err(problems.0),
    }
}

fn check_mapping(server: &mut LdapServer, problems: &mut Problems) {
    if let Some(user_base) = server.user_base_dn.clone() {
        problems.dn("userBaseDn", "User relative DN", &user_base);
    }
    problems.required(
        "userObjectClass",
        "User object class",
        &server.user_object_class,
    );
    if let Some(user_filter) = &server.user_ldap_filter
        && filter::parse(user_filter).is_err()
    {
        problems.add("userLdapFilter", "User filter is not a valid LDAP filter");
    }
    problems.required(
        "userIdAttribute",
        "Username attribute",
        &server.user_id_attribute,
    );
    problems.required(
        "userRealNameAttribute",
        "Real name attribute",
        &server.user_real_name_attribute,
    );
    problems.required(
        "userEmailAddressAttribute",
        "Email attribute",
        &server.user_email_address_attribute,
    );
    if !server.ldap_groups_as_roles {
        return;
    }
    match server.group_type {
        None => problems.add("groupType", "Group type must be static or dynamic"),
        Some(GroupType::Static) => {
            if let Some(group_base) = server.group_base_dn.clone() {
                problems.dn("groupBaseDn", "Group relative DN", &group_base);
            }
            problems.required(
                "groupObjectClass",
                "Group object class",
                &server.group_object_class,
            );
            problems.required(
                "groupIdAttribute",
                "Group ID attribute",
                &server.group_id_attribute,
            );
            problems.required(
                "groupMemberAttribute",
                "Group member attribute",
                &server.group_member_attribute,
            );
            problems.required(
                "groupMemberFormat",
                "Group member format",
                &server.group_member_format,
            );
        }
        Some(GroupType::Dynamic) => {
            problems.required(
                "userMemberOfAttribute",
                "Group member of attribute",
                &server.user_member_of_attribute,
            );
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub name: &'static str,
    pub user_base_dn: Option<&'static str>,
    pub user_subtree: bool,
    pub user_object_class: &'static str,
    pub user_ldap_filter: Option<&'static str>,
    pub user_id_attribute: &'static str,
    pub user_real_name_attribute: &'static str,
    pub user_email_address_attribute: &'static str,
    pub user_password_attribute: Option<&'static str>,
    pub ldap_groups_as_roles: bool,
    pub group_type: &'static str,
    pub group_base_dn: Option<&'static str>,
    pub group_subtree: bool,
    pub group_object_class: Option<&'static str>,
    pub group_id_attribute: Option<&'static str>,
    pub group_member_attribute: Option<&'static str>,
    pub group_member_format: Option<&'static str>,
    pub user_member_of_attribute: Option<&'static str>,
}

const fn dynamic(
    name: &'static str,
    user_base_dn: Option<&'static str>,
    user_object_class: &'static str,
    user_id_attribute: &'static str,
) -> Template {
    Template {
        name,
        user_base_dn,
        user_subtree: false,
        user_object_class,
        user_ldap_filter: None,
        user_id_attribute,
        user_real_name_attribute: "cn",
        user_email_address_attribute: "mail",
        user_password_attribute: None,
        ldap_groups_as_roles: true,
        group_type: "dynamic",
        group_base_dn: None,
        group_subtree: false,
        group_object_class: None,
        group_id_attribute: None,
        group_member_attribute: None,
        group_member_format: None,
        user_member_of_attribute: Some("memberOf"),
    }
}

pub const TEMPLATES: [Template; 4] = [
    dynamic(
        "Active Directory",
        Some("cn=users"),
        "user",
        "sAMAccountName",
    ),
    Template {
        name: "Posix with Static Groups",
        user_base_dn: Some("ou=people"),
        user_subtree: false,
        user_object_class: "posixAccount",
        user_ldap_filter: None,
        user_id_attribute: "uid",
        user_real_name_attribute: "cn",
        user_email_address_attribute: "mail",
        user_password_attribute: None,
        ldap_groups_as_roles: true,
        group_type: "static",
        group_base_dn: Some("ou=groups"),
        group_subtree: false,
        group_object_class: Some("posixGroup"),
        group_id_attribute: Some("cn"),
        group_member_attribute: Some("memberUid"),
        group_member_format: Some("${username}"),
        user_member_of_attribute: None,
    },
    dynamic(
        "Posix with Dynamic Groups",
        Some("ou=people"),
        "posixAccount",
        "uid",
    ),
    dynamic("Generic Ldap Server", None, "inetOrgPerson", "uid"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> Draft {
        Draft {
            name: Some(" corp ".to_owned()),
            protocol: Some("ldap".to_owned()),
            host: Some("ldap.example.com".to_owned()),
            search_base: Some("dc=example,dc=com".to_owned()),
            auth_scheme: Some("simple".to_owned()),
            auth_username: Some("cn=svc,dc=example,dc=com".to_owned()),
            auth_realm: Some("ignored".to_owned()),
            user_object_class: Some("inetOrgPerson".to_owned()),
            user_id_attribute: Some("uid".to_owned()),
            user_real_name_attribute: Some("cn".to_owned()),
            user_email_address_attribute: Some("mail".to_owned()),
            group_type: Some("dynamic".to_owned()),
            user_member_of_attribute: Some("memberOf".to_owned()),
            ..Draft::default()
        }
    }

    #[test]
    fn a_complete_draft_becomes_a_server_with_defaults() {
        let server = check(&draft(), Check::Everything).unwrap();
        assert_eq!(server.name, "corp");
        assert_eq!(server.port, 389);
        assert_eq!(server.auth_scheme, AuthScheme::Simple);
        assert_eq!(server.auth_realm, None);
        assert_eq!(server.connection_timeout_seconds, 30);
        assert_eq!(server.connection_retry_delay_seconds, 300);
        assert_eq!(server.max_incidents_count, 3);
        assert!(server.ldap_groups_as_roles);
        assert_eq!(server.group_type, Some(GroupType::Dynamic));
        assert_eq!(
            server.url(),
            "ldap://ldap.example.com:389/dc=example,dc=com"
        );
        assert_eq!(server.transport(), Transport::Plain);
    }

    #[test]
    fn enums_parse_in_any_spelling() {
        assert_eq!(AuthScheme::parse("DIGEST-MD5"), Some(AuthScheme::DigestMd5));
        assert_eq!(AuthScheme::parse("cram_md5"), Some(AuthScheme::CramMd5));
        assert_eq!(Protocol::parse(" LDAPS "), Some(Protocol::Ldaps));
        assert_eq!(GroupType::parse("Static"), Some(GroupType::Static));
        assert_eq!(GroupType::parse("other"), None);
        assert_eq!(
            serde_json::to_value(AuthScheme::DigestMd5).unwrap(),
            "DIGEST_MD5"
        );
    }

    #[test]
    fn problems_name_their_fields() {
        let broken = Draft {
            protocol: Some("http".to_owned()),
            host: Some("bad host!".to_owned()),
            port: Some(70000),
            search_base: Some("not a dn".to_owned()),
            auth_scheme: Some("kerberos".to_owned()),
            connection_timeout_seconds: Some(0),
            user_ldap_filter: Some("(uid=".to_owned()),
            ldap_groups_as_roles: Some(true),
            group_type: Some("static".to_owned()),
            ..Draft::default()
        };
        let fields: Vec<&str> = check(&broken, Check::Everything)
            .unwrap_err()
            .iter()
            .map(|problem| problem.field)
            .collect();
        for field in [
            "protocol",
            "host",
            "port",
            "searchBase",
            "authScheme",
            "connectionTimeoutSeconds",
            "name",
            "userObjectClass",
            "userLdapFilter",
            "userIdAttribute",
            "groupObjectClass",
            "groupMemberFormat",
        ] {
            assert!(fields.contains(&field), "{field} in {fields:?}");
        }
        let connection_only = check(
            &Draft {
                name: None,
                user_object_class: None,
                ..draft()
            },
            Check::Connection,
        );
        assert!(connection_only.is_ok());
    }

    #[test]
    fn settings_that_do_not_apply_are_dropped() {
        let server = check(
            &Draft {
                auth_scheme: Some("none".to_owned()),
                use_trust_store: Some(true),
                use_start_tls: Some(false),
                ldap_groups_as_roles: Some(false),
                ..draft()
            },
            Check::Everything,
        )
        .unwrap();
        assert_eq!(server.auth_username, None);
        assert!(!server.use_trust_store);
        assert_eq!(server.group_type, None);
        let secure = check(
            &Draft {
                protocol: Some("LDAPS".to_owned()),
                use_start_tls: Some(true),
                use_trust_store: Some(true),
                ..draft()
            },
            Check::Everything,
        )
        .unwrap();
        assert_eq!(secure.port, 636);
        assert!(!secure.use_start_tls);
        assert!(secure.use_trust_store);
        assert_eq!(secure.transport(), Transport::Ldaps);
        let start_tls = check(
            &Draft {
                use_start_tls: Some(true),
                ..draft()
            },
            Check::Everything,
        )
        .unwrap();
        assert_eq!(start_tls.transport(), Transport::StartTls);
    }

    #[test]
    fn the_stored_password_follows_only_the_same_account_on_the_same_server() {
        let server = check(&draft(), Check::Everything).unwrap();
        let mut other = server.clone();
        assert!(server.same_bind_target(&other));
        other.host = "LDAP.example.com".to_owned();
        assert!(server.same_bind_target(&other));
        other.port = 1389;
        assert!(!server.same_bind_target(&other));
        let mut other = server.clone();
        other.auth_username = Some("cn=other".to_owned());
        assert!(!server.same_bind_target(&other));
        let mut other = server.clone();
        other.use_start_tls = true;
        assert!(!server.same_bind_target(&other));
    }

    #[test]
    fn hosts_are_names_or_addresses() {
        for host in [
            "ldap",
            "ldap.example.com",
            "10.0.0.1",
            "::1",
            "[::1]",
            "a-b.c.",
        ] {
            assert!(is_host(host), "{host}");
        }
        for host in ["", "bad host", "-a.com", "a..b", "ldap://x"] {
            assert!(!is_host(host), "{host}");
        }
    }

    #[test]
    fn templates_match_the_directory_layouts() {
        assert_eq!(TEMPLATES.len(), 4);
        let json = serde_json::to_value(&TEMPLATES[1]).unwrap();
        assert_eq!(json["groupType"], "static");
        assert_eq!(json["groupMemberFormat"], "${username}");
        assert_eq!(json["userLdapFilter"], serde_json::Value::Null);
    }
}
