#![allow(dead_code)]

use std::sync::Arc;

use astrum_db::entity::{customer, repository, security_user, security_user_role};
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::two_factor::store::{Confirmation, RecoveryCodes, Verification, VerifyReport};
use astrum_security::{Gate, NewToken, Scope, SecretBox, Security, bootstrap, password, totp};
use astrum_testkit::TestDatabase;
use chrono::{DateTime, Duration, Utc};
use sea_orm::{ActiveValue, ConnectionTrait, DbBackend, EntityTrait, Statement};
use serde_json::json;
use totp::TotpSecret;

pub const PASSWORD: &str = "store-password-1";
pub const REALM: &str = "AstrumAuthenticatingRealm";

pub struct World {
    pub test: TestDatabase,
    pub db: astrum_db::Database,
    pub security: Arc<Security>,
    pub data: tempfile::TempDir,
}

pub async fn world() -> World {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let data = tempfile::tempdir().unwrap();
    bootstrap(db.conn(), data.path()).await.unwrap();
    let security = Security::load(db.conn().clone())
        .await
        .unwrap()
        .with_secrets(SecretBox::from_key([9u8; 32]));
    World {
        test,
        db,
        security: Arc::new(security),
        data,
    }
}

pub struct Enrolled {
    pub secret: TotpSecret,
    pub codes: Option<RecoveryCodes>,
    pub now: DateTime<Utc>,
}

impl Enrolled {
    pub fn code(&self, steps_ahead: i64) -> String {
        totp::code_at(&self.secret, self.step(steps_ahead))
    }

    pub fn step(&self, steps_ahead: i64) -> u64 {
        let step = totp::step_at(u64::try_from(self.now.timestamp()).unwrap());
        u64::try_from(i64::try_from(step).unwrap() + steps_ahead).unwrap()
    }

    pub fn at(&self, steps_ahead: i64) -> DateTime<Utc> {
        self.now + Duration::seconds(30 * steps_ahead)
    }

    pub fn wrong(&self, steps_ahead: i64) -> String {
        let near: Vec<String> = (-2..=2).map(|d| self.code(steps_ahead + d)).collect();
        (0..1_000_000)
            .map(|n| format!("{n:06}"))
            .find(|candidate| !near.contains(candidate))
            .unwrap()
    }
}

pub fn current_step(now: DateTime<Utc>) -> u64 {
    totp::step_at(u64::try_from(now.timestamp()).unwrap())
}

impl World {
    pub async fn user(&self, id: &str, age_days: i64) {
        let now = Utc::now().fixed_offset();
        security_user::Entity::insert(security_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            first_name: ActiveValue::Set(id.to_owned()),
            last_name: ActiveValue::Set("Tester".to_owned()),
            email: ActiveValue::Set(format!("{id}@example.org")),
            password_hash: ActiveValue::Set(Some(password::hash(PASSWORD))),
            status: ActiveValue::Set("active".to_owned()),
            created_at: ActiveValue::Set(now - Duration::days(age_days)),
            updated_at: ActiveValue::Set(now),
        })
        .exec(self.db.conn())
        .await
        .unwrap();
        security_user_role::Entity::insert(security_user_role::ActiveModel {
            user_id: ActiveValue::Set(id.to_owned()),
            source: ActiveValue::Set("default".to_owned()),
            roles: ActiveValue::Set(json!([])),
            updated_at: ActiveValue::Set(now),
        })
        .exec(self.db.conn())
        .await
        .unwrap();
        self.security.changed().await.unwrap();
    }

    pub async fn sql(&self, statement: &str) {
        self.db.conn().execute_unprepared(statement).await.unwrap();
    }

    pub async fn text(&self, query: &str) -> String {
        let row = self
            .db
            .conn()
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                format!("SELECT ({query})::text"),
            ))
            .await
            .unwrap()
            .unwrap();
        row.try_get_by_index::<String>(0).unwrap()
    }

    pub async fn number(&self, query: &str) -> i64 {
        self.text(query).await.parse().unwrap()
    }

    pub async fn passkey(&self, user: &str, credential: u8) {
        self.sql(&format!(
            "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
             VALUES (gen_random_uuid(), '{user}', 'key', '\\x{credential:02x}', 'localhost', '{{}}', false, false)"
        ))
        .await;
    }

    pub fn gate(&self, user: &str) -> Gate {
        self.security.model().second_factor_gate(user, Utc::now())
    }

    pub async fn begin(&self, user: &str) -> (String, TotpSecret) {
        let pending = self
            .security
            .begin_totp(user, REALM, "Astrum (test)", user, "127.0.0.1")
            .await
            .unwrap();
        (
            pending.ceremony.clone(),
            TotpSecret::from_base32(&pending.secret).unwrap(),
        )
    }

    pub async fn enroll(&self, user: &str) -> Enrolled {
        let (ceremony, secret) = self.begin(user).await;
        let now = Utc::now();
        let code = totp::code_at(&secret, current_step(now));
        let outcome = self
            .security
            .confirm_totp(user, &ceremony, &code, now)
            .await
            .unwrap();
        let Confirmation::Enrolled { recovery_codes, .. } = outcome else {
            panic!("{outcome:?}");
        };
        Enrolled {
            secret,
            codes: recovery_codes,
            now,
        }
    }

    pub async fn report(&self, user: &str, code: &str, now: DateTime<Utc>) -> VerifyReport {
        self.security.verify_totp(user, code, now).await.unwrap()
    }

    pub async fn verify(&self, user: &str, code: &str, now: DateTime<Utc>) -> Verification {
        self.report(user, code, now).await.result
    }
}

pub async fn customer_world() -> World {
    let world = world().await;
    let now = Utc::now().fixed_offset();
    customer::Entity::insert(customer::ActiveModel {
        id: ActiveValue::Set("acme".to_owned()),
        name: ActiveValue::Set("Acme".to_owned()),
        reference: ActiveValue::Set(String::new()),
        note: ActiveValue::Set(String::new()),
        disabled: ActiveValue::Set(false),
        created_by: ActiveValue::Set("admin".to_owned()),
        created_at: ActiveValue::Set(now),
    })
    .exec(world.db.conn())
    .await
    .unwrap();
    for name in ["raw-a", "raw-b"] {
        repository::Entity::insert(repository::ActiveModel {
            name: ActiveValue::Set(name.to_owned()),
            format: ActiveValue::Set("raw".to_owned()),
            kind: ActiveValue::Set("hosted".to_owned()),
            online: ActiveValue::Set(true),
            blob_store: ActiveValue::Set(None),
            attributes: ActiveValue::Set(serde_json::json!({})),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        })
        .exec(world.db.conn())
        .await
        .unwrap();
    }
    world
}

pub fn customer_token(repositories: &[&str]) -> NewToken {
    NewToken {
        name: "release".to_owned(),
        scope: Scope::Read,
        days: Some(30),
        repositories: Some(repositories.iter().map(|name| (*name).to_owned()).collect()),
        customer: Some("acme".to_owned()),
        created_by: Some("admin".to_owned()),
    }
}

pub async fn issue(world: &World, repositories: &[&str]) -> String {
    world
        .security
        .create_token(ADMIN_USER, customer_token(repositories))
        .await
        .unwrap()
        .1
}
