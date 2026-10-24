pub mod client;
pub mod config;
pub mod directory;
pub mod dn;
pub mod filter;
pub mod passwords;
pub mod sasl;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use rand::RngCore;
use rustls::pki_types::CertificateDer;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub use client::TlsClient;
pub use config::{
    AuthScheme, Check, Draft, GroupType, LdapServer, Problem, Protocol, SERVERS_KEY, TEMPLATES,
    Template,
};
pub use directory::{Directory, LdapServerError, LdapUser, Outcome};

use crate::SecretBox;
use crate::tls::TrustStore;

pub const SOURCE: &str = "LDAP";
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
pub const SEARCH_LIMIT: usize = 100;
pub const VERIFY_USER_LIMIT: usize = 20;
pub const GROUP_LIMIT: usize = 1000;
const CACHE_LIMIT: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LdapAuthError {
    NotFound,
    IncorrectPassword,
    Unavailable(LdapServerError),
}

struct CachedUser {
    at: Instant,
    user: Option<LdapUser>,
}

struct CachedLogin {
    at: Instant,
    salt: [u8; 16],
    hash: [u8; 32],
    user: LdapUser,
}

#[derive(Clone, Copy)]
struct Health {
    failures: u32,
    skip_until: Option<Instant>,
}

#[derive(Clone)]
struct Clients {
    system: TlsClient,
    certificates: Vec<CertificateDer<'static>>,
    trusted: TlsClient,
}

impl Clients {
    fn of(&self, server: &LdapServer) -> &TlsClient {
        match server.use_trust_store {
            true => &self.trusted,
            false => &self.system,
        }
    }
}

pub struct Ldap {
    ttl: RwLock<Duration>,
    tls: RwLock<Clients>,
    trust_store: RwLock<Option<Arc<dyn TrustStore>>>,
    users: Mutex<HashMap<String, CachedUser>>,
    logins: Mutex<HashMap<String, CachedLogin>>,
    health: Mutex<HashMap<String, Health>>,
}

impl std::fmt::Debug for Ldap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ldap")
    }
}

impl Default for Ldap {
    fn default() -> Self {
        Self::new()
    }
}

pub fn bind_password(
    server: &LdapServer,
    secrets: Option<&SecretBox>,
) -> Result<Option<String>, LdapServerError> {
    let Some(sealed) = server.auth_password.as_deref() else {
        return Ok(None);
    };
    let secrets = secrets.ok_or_else(|| {
        LdapServerError(
            "the secrets key is not loaded, so the bind password cannot be read".to_owned(),
        )
    })?;
    secrets.open(sealed).map(Some).map_err(|error| {
        LdapServerError(format!(
            "the bind password of {} cannot be read: {error}",
            server.name
        ))
    })
}

fn key(user_id: &str) -> String {
    user_id.to_lowercase()
}

fn password_hash(salt: &[u8], password: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(salt);
    digest.update(password.as_bytes());
    digest.finalize().into()
}

fn bounded<V>(map: &mut HashMap<String, V>, fresh: impl Fn(&V) -> bool) {
    if map.len() >= CACHE_LIMIT {
        map.retain(|_, value| fresh(value));
        if map.len() >= CACHE_LIMIT {
            map.clear();
        }
    }
}

impl Ldap {
    pub fn new() -> Self {
        Self {
            ttl: RwLock::new(DEFAULT_CACHE_TTL),
            tls: RwLock::new(Clients {
                system: TlsClient::system(),
                certificates: Vec::new(),
                trusted: TlsClient::system(),
            }),
            trust_store: RwLock::new(None),
            users: Mutex::new(HashMap::new()),
            logins: Mutex::new(HashMap::new()),
            health: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_cache_ttl(&self, ttl: Duration) {
        *self.ttl.write().expect("the LDAP lock is never poisoned") = ttl;
        self.clear();
    }

    pub fn cache_ttl(&self) -> Duration {
        *self.ttl.read().expect("the LDAP lock is never poisoned")
    }

    pub fn set_tls(&self, tls: TlsClient) {
        let mut clients = self.tls.write().expect("the LDAP lock is never poisoned");
        clients.trusted = tls.trusting(&clients.certificates);
        clients.system = tls;
    }

    pub fn tls(&self) -> TlsClient {
        self.clients().system
    }

    pub fn set_trust_store(&self, store: Arc<dyn TrustStore>) {
        *self
            .trust_store
            .write()
            .expect("the LDAP lock is never poisoned") = Some(store);
    }

    pub async fn refresh_trust_store(&self) -> bool {
        let store = self
            .trust_store
            .read()
            .expect("the LDAP lock is never poisoned")
            .clone();
        let Some(store) = store else {
            return false;
        };
        let certificates = match store.certificates().await {
            Ok(certificates) => certificates,
            Err(error) => {
                tracing::warn!(%error, "the truststore cannot be read; LDAP keeps the certificates it had");
                return false;
            }
        };
        let current = self.clients();
        if current.certificates == certificates {
            return false;
        }
        let trusted = current.system.trusting(&certificates);
        {
            let mut clients = self.tls.write().expect("the LDAP lock is never poisoned");
            clients.certificates = certificates;
            clients.trusted = trusted;
        }
        self.clear();
        true
    }

    pub async fn tls_for(&self, server: &LdapServer) -> TlsClient {
        self.clients_for(std::slice::from_ref(server))
            .await
            .of(server)
            .clone()
    }

    fn clients(&self) -> Clients {
        self.tls
            .read()
            .expect("the LDAP lock is never poisoned")
            .clone()
    }

    async fn clients_for(&self, servers: &[LdapServer]) -> Clients {
        if servers.iter().any(|server| server.use_trust_store) {
            self.refresh_trust_store().await;
        }
        self.clients()
    }

    pub fn clear(&self) {
        self.users
            .lock()
            .expect("the LDAP lock is never poisoned")
            .clear();
        self.logins
            .lock()
            .expect("the LDAP lock is never poisoned")
            .clear();
        self.health
            .lock()
            .expect("the LDAP lock is never poisoned")
            .clear();
    }

    pub fn cached(&self, user_id: &str) -> Option<LdapUser> {
        self.users
            .lock()
            .expect("the LDAP lock is never poisoned")
            .get(&key(user_id))
            .and_then(|cached| cached.user.clone())
    }

    fn fresh(&self, at: Instant) -> bool {
        at.elapsed() < self.cache_ttl()
    }

    pub async fn authenticate(
        &self,
        servers: &[LdapServer],
        secrets: Option<&SecretBox>,
        user_id: &str,
        password: &str,
    ) -> Result<LdapUser, LdapAuthError> {
        if user_id.trim().is_empty() || servers.is_empty() {
            return Err(LdapAuthError::NotFound);
        }
        let key = key(user_id);
        let tls = self.clients_for(servers).await;
        if let Some(user) = self.cached_login(&key, password) {
            return Ok(user);
        }
        let mut failure = None;
        for server in servers {
            if !self.available(server) {
                failure = Some(skipped(server));
                continue;
            }
            let bind = match bind_password(server, secrets) {
                Ok(bind) => bind,
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            };
            let directory = Directory {
                server,
                bind_password: bind.as_deref(),
                tls: tls.of(server),
            };
            match directory.authenticate(user_id, password).await {
                Ok(Outcome::Authenticated(user)) => {
                    self.succeeded(server);
                    self.remember_user(&key, Some(user.clone()));
                    self.remember_login(key, password, user.clone());
                    return Ok(user);
                }
                Ok(Outcome::IncorrectPassword) => {
                    self.succeeded(server);
                    return Err(LdapAuthError::IncorrectPassword);
                }
                Ok(Outcome::UserNotFound) => self.succeeded(server),
                Err(error) => {
                    self.failed(server, &error);
                    failure = Some(error);
                }
            }
        }
        match failure {
            Some(error) => Err(LdapAuthError::Unavailable(error)),
            None => Err(LdapAuthError::NotFound),
        }
    }

    pub async fn user(
        &self,
        servers: &[LdapServer],
        secrets: Option<&SecretBox>,
        user_id: &str,
    ) -> Result<Option<LdapUser>, LdapServerError> {
        if user_id.trim().is_empty() || servers.is_empty() {
            return Ok(None);
        }
        let key = key(user_id);
        let tls = self.clients_for(servers).await;
        let stale = {
            let users = self.users.lock().expect("the LDAP lock is never poisoned");
            match users.get(&key) {
                Some(cached) if self.fresh(cached.at) => return Ok(cached.user.clone()),
                Some(cached) => cached.user.clone(),
                None => None,
            }
        };
        match self.lookup(servers, secrets, user_id, &tls).await {
            Ok(user) => {
                self.remember_user(&key, user.clone());
                Ok(user)
            }
            Err(error) => match stale {
                Some(user) => {
                    tracing::warn!(user = user_id, %error, "LDAP is unavailable; the cached entry of the user is used");
                    Ok(Some(user))
                }
                None => Err(error),
            },
        }
    }

    async fn lookup(
        &self,
        servers: &[LdapServer],
        secrets: Option<&SecretBox>,
        user_id: &str,
        tls: &Clients,
    ) -> Result<Option<LdapUser>, LdapServerError> {
        let mut failure = None;
        for server in servers {
            if !self.available(server) {
                failure = Some(skipped(server));
                continue;
            }
            let bind = match bind_password(server, secrets) {
                Ok(bind) => bind,
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            };
            let directory = Directory {
                server,
                bind_password: bind.as_deref(),
                tls: tls.of(server),
            };
            match directory.find_user(user_id).await {
                Ok(Some(user)) => {
                    self.succeeded(server);
                    return Ok(Some(user));
                }
                Ok(None) => self.succeeded(server),
                Err(error) => {
                    self.failed(server, &error);
                    failure = Some(error);
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(None),
        }
    }

    pub async fn search_users(
        &self,
        servers: &[LdapServer],
        secrets: Option<&SecretBox>,
        prefix: Option<&str>,
        limit: usize,
    ) -> Vec<LdapUser> {
        let tls = self.clients_for(servers).await;
        let mut found: Vec<LdapUser> = Vec::new();
        for server in servers {
            if !self.available(server) {
                continue;
            }
            let Ok(bind) = bind_password(server, secrets) else {
                continue;
            };
            let directory = Directory {
                server,
                bind_password: bind.as_deref(),
                tls: tls.of(server),
            };
            match directory.search_users(prefix, limit).await {
                Ok(users) => {
                    self.succeeded(server);
                    for user in users {
                        if !found
                            .iter()
                            .any(|known| key(&known.username) == key(&user.username))
                        {
                            found.push(user);
                        }
                    }
                }
                Err(error) => self.failed(server, &error),
            }
        }
        found
    }

    pub async fn search_groups(
        &self,
        servers: &[LdapServer],
        secrets: Option<&SecretBox>,
        query: Option<&str>,
        limit: usize,
    ) -> Vec<String> {
        let tls = self.clients_for(servers).await;
        let mut groups: Vec<String> = Vec::new();
        for server in servers.iter().filter(|server| server.maps_groups()) {
            if !self.available(server) {
                continue;
            }
            let Ok(bind) = bind_password(server, secrets) else {
                continue;
            };
            let directory = Directory {
                server,
                bind_password: bind.as_deref(),
                tls: tls.of(server),
            };
            match directory.search_groups(query, limit).await {
                Ok(names) => {
                    self.succeeded(server);
                    for name in names {
                        if !groups.iter().any(|known| known.eq_ignore_ascii_case(&name)) {
                            groups.push(name);
                        }
                    }
                }
                Err(error) => self.failed(server, &error),
            }
        }
        groups.sort_by_key(|name| name.to_lowercase());
        if limit > 0 {
            groups.truncate(limit);
        }
        groups
    }

    pub async fn group_exists(
        &self,
        servers: &[LdapServer],
        secrets: Option<&SecretBox>,
        group: &str,
    ) -> Result<bool, LdapServerError> {
        let tls = self.clients_for(servers).await;
        let mut failure = None;
        for server in servers.iter().filter(|server| server.maps_groups()) {
            if !self.available(server) {
                failure = Some(skipped(server));
                continue;
            }
            let bind = match bind_password(server, secrets) {
                Ok(bind) => bind,
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            };
            let directory = Directory {
                server,
                bind_password: bind.as_deref(),
                tls: tls.of(server),
            };
            match directory.group_exists(group).await {
                Ok(true) => {
                    self.succeeded(server);
                    return Ok(true);
                }
                Ok(false) => self.succeeded(server),
                Err(error) => {
                    self.failed(server, &error);
                    failure = Some(error);
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(false),
        }
    }

    fn cached_login(&self, key: &str, password: &str) -> Option<LdapUser> {
        if password.is_empty() {
            return None;
        }
        let logins = self.logins.lock().expect("the LDAP lock is never poisoned");
        let cached = logins.get(key)?;
        if !self.fresh(cached.at) {
            return None;
        }
        let hash = password_hash(&cached.salt, password);
        bool::from(hash.ct_eq(&cached.hash)).then(|| cached.user.clone())
    }

    fn remember_login(&self, key: String, password: &str, user: LdapUser) {
        if self.cache_ttl().is_zero() {
            return;
        }
        let mut salt = [0u8; 16];
        rand::rng().fill_bytes(&mut salt);
        let hash = password_hash(&salt, password);
        let ttl = self.cache_ttl();
        let mut logins = self.logins.lock().expect("the LDAP lock is never poisoned");
        bounded(&mut logins, |login| login.at.elapsed() < ttl);
        logins.insert(
            key,
            CachedLogin {
                at: Instant::now(),
                salt,
                hash,
                user,
            },
        );
    }

    fn remember_user(&self, key: &str, user: Option<LdapUser>) {
        let ttl = self.cache_ttl();
        let mut users = self.users.lock().expect("the LDAP lock is never poisoned");
        bounded(&mut users, |cached| cached.at.elapsed() < ttl);
        users.insert(
            key.to_owned(),
            CachedUser {
                at: Instant::now(),
                user,
            },
        );
    }

    fn available(&self, server: &LdapServer) -> bool {
        self.health
            .lock()
            .expect("the LDAP lock is never poisoned")
            .get(&server.id)
            .and_then(|health| health.skip_until)
            .is_none_or(|until| until <= Instant::now())
    }

    fn succeeded(&self, server: &LdapServer) {
        self.health
            .lock()
            .expect("the LDAP lock is never poisoned")
            .remove(&server.id);
    }

    fn failed(&self, server: &LdapServer, error: &LdapServerError) {
        tracing::warn!(server = %server.name, %error, "an LDAP server failed");
        let mut health = self.health.lock().expect("the LDAP lock is never poisoned");
        let previous = health.get(&server.id).copied();
        let failures = match previous {
            Some(previous) if previous.skip_until.is_none() => previous.failures + 1,
            _ => 1,
        };
        let skip = server.max_incidents_count > 0
            && server.connection_retry_delay_seconds > 0
            && failures >= server.max_incidents_count;
        let next = match skip {
            true => {
                tracing::warn!(
                    server = %server.name,
                    seconds = server.connection_retry_delay_seconds,
                    failures,
                    "the LDAP server is skipped for a while after repeated failures"
                );
                Health {
                    failures: 0,
                    skip_until: Some(
                        Instant::now()
                            + Duration::from_secs(u64::from(server.connection_retry_delay_seconds)),
                    ),
                }
            }
            false => Health {
                failures,
                skip_until: None,
            },
        };
        health.insert(server.id.clone(), next);
    }
}

fn skipped(server: &LdapServer) -> LdapServerError {
    LdapServerError(format!(
        "LDAP server {} is skipped after repeated failures",
        server.name
    ))
}
