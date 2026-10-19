pub mod api_keys;
pub mod bootstrap;
pub mod catalogue;
pub mod format_tokens;
pub mod ldap;
pub mod model;
pub mod oauth;
pub mod oidc;
pub mod password;
pub mod permission;
pub mod privilege;
pub mod realm;
pub mod secrets;
pub mod selector;
pub mod sessions;
pub mod ssrf;
pub mod subject;
pub mod tokens;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use astrum_db::entity::security_user;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use sha2::{Digest, Sha256};

pub use bootstrap::{Bootstrapped, bootstrap};
pub use model::{AnonymousSettings, ContentSelector, Role, SecurityModel, User, UserStatus};
pub use permission::Permission;
pub use privilege::{Action, Field, FieldKind, Privilege, PrivilegeKind, SelectorGrant};
pub use secrets::SecretBox;
pub use selector::{Selector, SelectorError};
pub use sessions::{ActiveSession, NewSession};
pub use ssrf::SsrfProtection;
pub use subject::{Scope, Subject};
pub use tokens::{TokenError, TokenInfo};

const VERIFIED_FOR: Duration = Duration::from_secs(300);
const VERIFIED_LIMIT: usize = 10_000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] astrum_db::Error),
    #[error("the database refused a query: {0}")]
    Query(#[from] DbErr),
    #[error(transparent)]
    Secret(#[from] secrets::SecretError),
    #[error("cannot write a file: {0}")]
    Io(#[from] std::io::Error),
    #[error("a background job ended early: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Profile {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("the user name or password is wrong")]
    BadCredentials,
    #[error(transparent)]
    Internal(#[from] Error),
}

pub struct Security {
    db: DatabaseConnection,
    model: RwLock<Arc<SecurityModel>>,
    verified: Mutex<HashMap<[u8; 32], Instant>>,
    failures: Mutex<HashMap<String, (u32, Instant)>>,
    secrets: Option<SecretBox>,
    ldap: ldap::Ldap,
    oidc: oidc::OidcProvider,
}

impl std::fmt::Debug for Security {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Security")
    }
}

impl Security {
    pub async fn load(db: DatabaseConnection) -> Result<Self, Error> {
        let model = SecurityModel::load(&db).await?;
        Ok(Self {
            db,
            model: RwLock::new(Arc::new(model)),
            verified: Mutex::new(HashMap::new()),
            failures: Mutex::new(HashMap::new()),
            secrets: None,
            ldap: ldap::Ldap::new(),
            oidc: oidc::OidcProvider::default(),
        })
    }

    pub fn with_secrets(mut self, secrets: SecretBox) -> Self {
        self.secrets = Some(secrets);
        self
    }

    pub fn secrets(&self) -> Option<&SecretBox> {
        self.secrets.as_ref()
    }

    pub fn ldap(&self) -> &ldap::Ldap {
        &self.ldap
    }

    pub fn oidc(&self) -> &oidc::OidcProvider {
        &self.oidc
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn model(&self) -> Arc<SecurityModel> {
        self.model
            .read()
            .expect("the security model lock is never poisoned")
            .clone()
    }

    pub async fn reload(&self) -> Result<(), Error> {
        let model = SecurityModel::load(&self.db).await?;
        let previous = std::mem::replace(
            &mut *self
                .model
                .write()
                .expect("the security model lock is never poisoned"),
            Arc::new(model),
        );
        if previous.ldap_servers != self.model().ldap_servers {
            self.ldap.clear();
        }
        if previous.oidc != self.model().oidc {
            self.oidc.reset();
        }
        self.verified
            .lock()
            .expect("the verification cache lock is never poisoned")
            .clear();
        Ok(())
    }

    pub fn anonymous(&self) -> Option<Subject> {
        self.model().anonymous_subject()
    }

    pub async fn authenticate(&self, user_id: &str, password: &str) -> Result<Subject, AuthError> {
        let model = self.model();
        for realm in &model.realms {
            let found = match realm.as_str() {
                realm::LOCAL => self.authenticate_local(&model, user_id, password).await?,
                realm::LDAP => self.authenticate_ldap(&model, user_id, password).await,
                _ => None,
            };
            if let Some(subject) = found {
                return Ok(subject);
            }
        }
        Err(AuthError::BadCredentials)
    }

    async fn authenticate_local(
        &self,
        model: &SecurityModel,
        user_id: &str,
        password: &str,
    ) -> Result<Option<Subject>, AuthError> {
        let Some(user) = model.users.get(user_id) else {
            return Ok(None);
        };
        let Some(stored) = user.password_hash.clone() else {
            return Ok(None);
        };
        if !user.status.can_sign_in() {
            return Ok(None);
        }
        let key = cache_key(user_id, password, &stored);
        if !self.recently_verified(&key) {
            let candidate = password.to_owned();
            let hash = stored.clone();
            let verified = tokio::task::spawn_blocking(move || password::verify(&hash, &candidate))
                .await
                .map_err(Error::from)?;
            if !verified.matched() {
                return Ok(None);
            }
            if verified == password::Verified::MatchButOutdated {
                self.rehash(user_id, password).await?;
            }
            self.remember(key);
        }
        Ok(model.local_subject(user_id, realm::LOCAL))
    }

    async fn authenticate_ldap(
        &self,
        model: &SecurityModel,
        user_id: &str,
        password: &str,
    ) -> Option<Subject> {
        if model.ldap_servers.is_empty() || password.is_empty() {
            return None;
        }
        match self
            .ldap
            .authenticate(&model.ldap_servers, self.secrets(), user_id, password)
            .await
        {
            Ok(user) => Some(model.external_subject(
                &user.username,
                ldap::SOURCE,
                realm::LDAP,
                &user.groups,
            )),
            Err(ldap::LdapAuthError::Unavailable(error)) => {
                tracing::warn!(user = user_id, %error, "LDAP could not check a sign-in");
                None
            }
            Err(_) => None,
        }
    }

    pub async fn ldap_user(
        &self,
        user_id: &str,
    ) -> Result<Option<ldap::LdapUser>, ldap::LdapServerError> {
        let model = self.model();
        self.ldap
            .user(&model.ldap_servers, self.secrets(), user_id)
            .await
    }

    pub async fn ldap_subject(
        &self,
        user_id: &str,
    ) -> Result<Option<Subject>, ldap::LdapServerError> {
        let model = self.model();
        if !model.realm_active(realm::LDAP) {
            return Ok(None);
        }
        Ok(self.ldap_user(user_id).await?.map(|user| {
            model.external_subject(&user.username, ldap::SOURCE, realm::LDAP, &user.groups)
        }))
    }

    pub fn profile(&self, subject: &Subject) -> Profile {
        let model = self.model();
        match subject.source.as_str() {
            ldap::SOURCE => {
                let user = self.ldap.cached(&subject.user_id);
                let name = user
                    .as_ref()
                    .and_then(|user| user.real_name.clone())
                    .unwrap_or_default();
                let (first_name, last_name) = match name.split_once(' ') {
                    Some((first, last)) => (first.to_owned(), last.trim().to_owned()),
                    None => (name, String::new()),
                };
                Profile {
                    first_name,
                    last_name,
                    email: user.and_then(|user| user.email).unwrap_or_default(),
                }
            }
            oidc::SOURCE => model
                .oidc_users
                .get(&subject.user_id)
                .map(|user| Profile {
                    first_name: user.first_name.clone(),
                    last_name: user.last_name.clone(),
                    email: user.email.clone(),
                })
                .unwrap_or_default(),
            _ => model
                .users
                .get(&subject.user_id)
                .map(|user| Profile {
                    first_name: user.first_name.clone(),
                    last_name: user.last_name.clone(),
                    email: user.email.clone(),
                })
                .unwrap_or_default(),
        }
    }

    pub async fn oidc_sign_in(&self, user: &oidc::OidcUser) -> Result<Option<Subject>, Error> {
        let now = chrono::Utc::now().fixed_offset();
        astrum_db::entity::security_oidc_user::Entity::insert(
            astrum_db::entity::security_oidc_user::ActiveModel {
                user_id: ActiveValue::Set(user.user_id.clone()),
                subject: ActiveValue::Set(user.subject.clone()),
                first_name: ActiveValue::Set(user.first_name.clone()),
                last_name: ActiveValue::Set(user.last_name.clone()),
                email: ActiveValue::Set(user.email.clone()),
                groups: ActiveValue::Set(serde_json::json!(user.groups)),
                last_login_at: ActiveValue::Set(now),
                created_at: ActiveValue::Set(now),
            },
        )
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(
                astrum_db::entity::security_oidc_user::Column::UserId,
            )
            .update_columns([
                astrum_db::entity::security_oidc_user::Column::Subject,
                astrum_db::entity::security_oidc_user::Column::FirstName,
                astrum_db::entity::security_oidc_user::Column::LastName,
                astrum_db::entity::security_oidc_user::Column::Email,
                astrum_db::entity::security_oidc_user::Column::Groups,
                astrum_db::entity::security_oidc_user::Column::LastLoginAt,
            ])
            .to_owned(),
        )
        .exec(&self.db)
        .await?;
        self.reload().await?;
        Ok(self.model().oidc_subject(&user.user_id))
    }

    async fn rehash(&self, user_id: &str, password: &str) -> Result<(), Error> {
        let candidate = password.to_owned();
        let hash = tokio::task::spawn_blocking(move || password::hash(&candidate)).await?;
        security_user::Entity::update(security_user::ActiveModel {
            id: ActiveValue::Unchanged(user_id.to_owned()),
            password_hash: ActiveValue::Set(Some(hash)),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;
        tracing::info!(
            user = user_id,
            "stored the password in the current hash format"
        );
        self.reload().await
    }

    fn recently_verified(&self, key: &[u8; 32]) -> bool {
        let verified = self
            .verified
            .lock()
            .expect("the verification cache lock is never poisoned");
        verified
            .get(key)
            .is_some_and(|at| at.elapsed() < VERIFIED_FOR)
    }

    fn remember(&self, key: [u8; 32]) {
        let mut verified = self
            .verified
            .lock()
            .expect("the verification cache lock is never poisoned");
        if verified.len() >= VERIFIED_LIMIT {
            verified.retain(|_, at| at.elapsed() < VERIFIED_FOR);
            if verified.len() >= VERIFIED_LIMIT {
                verified.clear();
            }
        }
        verified.insert(key, Instant::now());
    }
}

fn cache_key(user_id: &str, password: &str, stored: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    for part in [user_id, password, stored] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    digest.finalize().into()
}
