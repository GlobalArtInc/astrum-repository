pub mod bootstrap;
pub mod catalogue;
pub mod model;
pub mod password;
pub mod permission;
pub mod privilege;
pub mod realm;
pub mod secrets;
pub mod sessions;
pub mod subject;
pub mod tokens;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use astrum_db::entity::security_user;
use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use sha2::{Digest, Sha256};

pub use bootstrap::{Bootstrapped, bootstrap};
pub use model::{AnonymousSettings, Role, SecurityModel, User, UserStatus};
pub use permission::Permission;
pub use privilege::{Action, Privilege, PrivilegeKind, SelectorGrant};
pub use secrets::SecretBox;
pub use sessions::{ActiveSession, NewSession};
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
        })
    }

    pub fn model(&self) -> Arc<SecurityModel> {
        self.model
            .read()
            .expect("the security model lock is never poisoned")
            .clone()
    }

    pub async fn reload(&self) -> Result<(), Error> {
        let model = SecurityModel::load(&self.db).await?;
        *self
            .model
            .write()
            .expect("the security model lock is never poisoned") = Arc::new(model);
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
        if !model.realm_active(realm::LOCAL) {
            return Err(AuthError::BadCredentials);
        }
        let user = model.users.get(user_id).ok_or(AuthError::BadCredentials)?;
        let stored = user
            .password_hash
            .clone()
            .ok_or(AuthError::BadCredentials)?;
        if !user.status.can_sign_in() {
            return Err(AuthError::BadCredentials);
        }
        let key = cache_key(user_id, password, &stored);
        if !self.recently_verified(&key) {
            let candidate = password.to_owned();
            let hash = stored.clone();
            let verified = tokio::task::spawn_blocking(move || password::verify(&hash, &candidate))
                .await
                .map_err(Error::from)?;
            if !verified.matched() {
                return Err(AuthError::BadCredentials);
            }
            if verified == password::Verified::MatchButOutdated {
                self.rehash(user_id, password).await?;
            }
            self.remember(key);
        }
        model
            .local_subject(user_id, realm::LOCAL)
            .ok_or(AuthError::BadCredentials)
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
