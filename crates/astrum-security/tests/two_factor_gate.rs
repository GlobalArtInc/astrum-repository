use astrum_db::entity::{security_user, security_user_role};
use astrum_db::{revision, setting};
use astrum_security::catalogue::{ADMIN_ROLE, ADMIN_USER, ANONYMOUS_ROLE};
use astrum_security::two_factor::{POLICY_KEY, Requirement};
use astrum_security::{AuthError, Gate, Security, TwoFactorPolicy, bootstrap, password};
use astrum_testkit::TestDatabase;
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait};
use serde_json::{Value, json};

const PASSWORD: &str = "gate-password-1";

struct World {
    _test: TestDatabase,
    db: astrum_db::Database,
    security: Security,
    admin_password: String,
    _data: tempfile::TempDir,
}

async fn world() -> World {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let data = tempfile::tempdir().unwrap();
    let created = bootstrap(db.conn(), data.path()).await.unwrap();
    let admin_password = std::fs::read_to_string(created.admin_password_file.unwrap()).unwrap();
    let security = Security::load(db.conn().clone()).await.unwrap();
    World {
        _test: test,
        db,
        security,
        admin_password,
        _data: data,
    }
}

impl World {
    async fn user(&self, id: &str, age_days: i64, roles: &[&str]) {
        let now = chrono::Utc::now().fixed_offset();
        let created = now - chrono::Duration::days(age_days);
        security_user::Entity::insert(security_user::ActiveModel {
            id: ActiveValue::Set(id.to_owned()),
            first_name: ActiveValue::Set(id.to_owned()),
            last_name: ActiveValue::Set("Tester".to_owned()),
            email: ActiveValue::Set(format!("{id}@example.org")),
            password_hash: ActiveValue::Set(Some(password::hash(PASSWORD))),
            status: ActiveValue::Set("active".to_owned()),
            created_at: ActiveValue::Set(created),
            updated_at: ActiveValue::Set(now),
        })
        .exec(self.db.conn())
        .await
        .unwrap();
        security_user_role::Entity::insert(security_user_role::ActiveModel {
            user_id: ActiveValue::Set(id.to_owned()),
            source: ActiveValue::Set("default".to_owned()),
            roles: ActiveValue::Set(json!(roles)),
            updated_at: ActiveValue::Set(now),
        })
        .exec(self.db.conn())
        .await
        .unwrap();
    }

    async fn sql(&self, statement: &str) {
        self.db.conn().execute_unprepared(statement).await.unwrap();
    }

    async fn policy(&self, policy: Value) {
        setting::put(self.db.conn(), POLICY_KEY, &policy)
            .await
            .unwrap();
    }

    async fn reload(&self) {
        self.security.changed().await.unwrap();
    }

    async fn gate(&self, user: &str) -> Gate {
        self.security
            .model()
            .second_factor_gate(user, chrono::Utc::now())
    }

    async fn sign_in(&self, user: &str) -> Result<String, Gate> {
        match self.security.authenticate(user, PASSWORD).await {
            Ok(subject) => Ok(subject.user_id),
            Err(AuthError::SecondFactor(pending)) => {
                assert_eq!(pending.user_id, user);
                assert_eq!(pending.realm, "AstrumAuthenticatingRealm");
                Err(pending.gate)
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn everyone(grace_days: u32) -> Value {
    json!({"required": {"everyone": true}, "graceDays": grace_days})
}

fn totp(user: &str) -> String {
    format!(
        "INSERT INTO user_second_factor (user_id, totp_secret, totp_confirmed_at) VALUES ('{user}', 'sealed', now())"
    )
}

#[tokio::test]
async fn a_user_who_is_not_enrolled_signs_in_as_before() {
    let world = world().await;
    world.user("jane", 0, &[]).await;
    world.reload().await;
    assert_eq!(world.gate("jane").await, Gate::Open);
    assert_eq!(world.sign_in("jane").await, Ok("jane".to_owned()));
}

#[tokio::test]
async fn an_enrolled_user_needs_the_second_factor_and_a_wrong_password_still_fails() {
    let world = world().await;
    world.user("jane", 0, &[]).await;
    world.user("pat", 0, &[]).await;
    world.sql(&totp("jane")).await;
    world
        .sql(
            "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
             VALUES (gen_random_uuid(), 'pat', 'key', '\\x01', 'localhost', '{}', false, false)",
        )
        .await;
    world.reload().await;
    assert_eq!(world.sign_in("jane").await, Err(Gate::Verify));
    assert_eq!(world.sign_in("pat").await, Err(Gate::Verify));
    assert!(matches!(
        world.security.authenticate("jane", "wrong").await,
        Err(AuthError::BadCredentials)
    ));
}

#[tokio::test]
async fn the_grace_period_decides_between_open_and_enroll() {
    let world = world().await;
    world.user("fresh", 0, &[]).await;
    world.user("old", 30, &[]).await;
    world.user("fixed", 30, &[]).await;
    world
        .sql("INSERT INTO user_second_factor (user_id, enroll_by) VALUES ('fixed', now() + interval '1 day')")
        .await;
    world.policy(everyone(7)).await;
    world.reload().await;
    assert_eq!(world.sign_in("fresh").await, Ok("fresh".to_owned()));
    assert_eq!(world.sign_in("old").await, Err(Gate::Enroll));
    assert_eq!(world.sign_in("fixed").await, Ok("fixed".to_owned()));

    world
        .sql("UPDATE user_second_factor SET enroll_by = now() - interval '1 second'")
        .await;
    world
        .policy(json!({
            "required": {"everyone": true},
            "graceDays": 7,
            "requiredSince": chrono::Utc::now(),
        }))
        .await;
    world.reload().await;
    assert_eq!(world.gate("fixed").await, Gate::Enroll, "enroll_by wins");
    assert_eq!(
        world.gate("old").await,
        Gate::Open,
        "widening the requirement restarts the grace period"
    );
}

#[tokio::test]
async fn exempt_beats_the_policy_and_required_beats_the_policy_off() {
    let world = world().await;
    world.user("robot", 30, &[]).await;
    world.user("strict", 30, &[]).await;
    world.user("tester", 30, &[]).await;
    world
        .sql("INSERT INTO user_second_factor (user_id, requirement) VALUES ('robot', 'exempt'), ('strict', 'required')")
        .await;
    world.policy(everyone(0)).await;
    world.reload().await;
    assert_eq!(world.gate("robot").await, Gate::Open);
    assert_eq!(world.gate("strict").await, Gate::Enroll);
    assert_eq!(world.gate("tester").await, Gate::Enroll);

    world.policy(json!({})).await;
    world.reload().await;
    assert_eq!(world.gate("robot").await, Gate::Open);
    assert_eq!(world.gate("strict").await, Gate::Enroll);
    assert_eq!(world.gate("tester").await, Gate::Open);
}

#[tokio::test]
async fn administrators_are_found_by_the_all_permission() {
    let world = world().await;
    world.user("boss", 30, &[ADMIN_ROLE]).await;
    world.user("jane", 30, &[]).await;
    world
        .policy(json!({"required": {"administrators": true}, "graceDays": 0}))
        .await;
    world.reload().await;
    assert_eq!(world.gate("boss").await, Gate::Enroll);
    assert_eq!(world.gate(ADMIN_USER).await, Gate::Enroll);
    assert_eq!(world.gate("jane").await, Gate::Open);
    assert!(matches!(
        world.security.authenticate(ADMIN_USER, &world.admin_password).await,
        Err(AuthError::SecondFactor(pending)) if pending.gate == Gate::Enroll
    ));
}

#[tokio::test]
async fn a_role_in_the_policy_requires_the_factor() {
    let world = world().await;
    world.user("member", 30, &[ANONYMOUS_ROLE]).await;
    world.user("jane", 30, &[]).await;
    world
        .policy(json!({"required": {"roles": [ANONYMOUS_ROLE]}, "graceDays": 7}))
        .await;
    world.reload().await;
    assert_eq!(world.gate("member").await, Gate::Enroll);
    assert_eq!(world.gate("jane").await, Gate::Open);
}

#[tokio::test]
async fn an_enrolled_user_is_verified_even_when_exempt() {
    let world = world().await;
    world.user("robot", 30, &[]).await;
    world
        .sql("INSERT INTO user_second_factor (user_id, requirement, totp_secret, totp_confirmed_at) VALUES ('robot', 'exempt', 'sealed', now())")
        .await;
    world.reload().await;
    assert_eq!(world.gate("robot").await, Gate::Verify);
}

#[tokio::test]
async fn the_gate_follows_the_revision() {
    let world = world().await;
    let other = Security::load(world.db.conn().clone()).await.unwrap();
    world.user("jane", 0, &[]).await;
    world.reload().await;
    other.sync().await.unwrap();
    assert!(other.authenticate("jane", PASSWORD).await.is_ok());

    world.sql(&totp("jane")).await;
    assert!(
        other.authenticate("jane", PASSWORD).await.is_ok(),
        "the model is in memory until the revision moves"
    );
    revision::bump(world.db.conn(), revision::SECURITY)
        .await
        .unwrap();
    assert!(other.sync().await.unwrap());
    assert!(matches!(
        other.authenticate("jane", PASSWORD).await,
        Err(AuthError::SecondFactor(_))
    ));
}

#[tokio::test]
async fn an_unreadable_row_fails_closed() {
    let world = world().await;
    world.user("jane", 0, &[]).await;
    world.user("pat", 0, &[]).await;
    world
        .sql("ALTER TABLE user_second_factor DROP CONSTRAINT user_second_factor_requirement_check")
        .await;
    world
        .sql("INSERT INTO user_second_factor (user_id, requirement) VALUES ('jane', 'from-the-future')")
        .await;
    world.reload().await;
    assert_eq!(
        world.security.model().second_factor["jane"].requirement,
        Requirement::Unreadable
    );
    assert_eq!(world.gate("jane").await, Gate::Enroll);
    assert_eq!(world.gate("pat").await, Gate::Open);

    world.sql(&totp("pat")).await;
    world.reload().await;
    assert_eq!(world.gate("pat").await, Gate::Verify);
}

#[tokio::test]
async fn a_broken_policy_is_logged_and_replaced_by_the_default_without_reload_storms() {
    let world = world().await;
    world.user("jane", 0, &[]).await;
    world.user("pat", 0, &[]).await;
    world.sql(&totp("jane")).await;
    for broken in [Value::Null, json!({"graceDays": -1})] {
        setting::put(world.db.conn(), POLICY_KEY, &broken)
            .await
            .unwrap();
        revision::bump(world.db.conn(), revision::SECURITY)
            .await
            .unwrap();
        assert!(matches!(
            world.security.authenticate("pat", "wrong").await,
            Err(AuthError::BadCredentials)
        ));
        let loaded = world.security.model();
        for _ in 0..3 {
            assert!(matches!(
                world.security.authenticate("pat", "wrong").await,
                Err(AuthError::BadCredentials)
            ));
        }
        assert!(
            std::sync::Arc::ptr_eq(&loaded, &world.security.model()),
            "a wrong password must not reload the model again"
        );
        assert_eq!(loaded.two_factor, TwoFactorPolicy::default());
        assert_eq!(world.sign_in("pat").await, Ok("pat".to_owned()));
        assert_eq!(world.sign_in("jane").await, Err(Gate::Verify));
    }
}

#[tokio::test]
async fn any_one_of_secret_confirmation_or_passkey_counts_as_enrolled() {
    let world = world().await;
    for user in ["secret", "confirmed", "passkey", "none"] {
        world.user(user, 0, &[]).await;
    }
    world
        .sql("INSERT INTO user_second_factor (user_id, totp_secret) VALUES ('secret', 'sealed')")
        .await;
    world
        .sql("INSERT INTO user_second_factor (user_id, totp_confirmed_at) VALUES ('confirmed', now())")
        .await;
    world
        .sql("INSERT INTO user_second_factor (user_id) VALUES ('none')")
        .await;
    world
        .sql(
            "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
             VALUES (gen_random_uuid(), 'passkey', 'key', '\\x02', 'localhost', '{}', false, false)",
        )
        .await;
    world.reload().await;
    for user in ["secret", "confirmed", "passkey"] {
        assert_eq!(world.gate(user).await, Gate::Verify, "{user}");
    }
    assert_eq!(world.gate("none").await, Gate::Open);
}

#[tokio::test]
async fn a_stored_policy_with_no_method_to_enroll_gates_nobody_who_could_not_enroll() {
    let world = world().await;
    for user in ["jane", "pat", "ned"] {
        world.user(user, 30, &[]).await;
    }
    world.sql(&totp("pat")).await;
    world
        .sql("INSERT INTO user_second_factor (user_id, requirement) VALUES ('ned', 'required')")
        .await;
    world
        .policy(json!({"totp": false, "passkeys": true, "required": {"everyone": true}, "graceDays": 7}))
        .await;
    world.reload().await;
    assert_eq!(world.gate("jane").await, Gate::Open);
    assert_eq!(world.gate("ned").await, Gate::Open);
    assert_eq!(world.gate("pat").await, Gate::Verify);

    world
        .policy(
            json!({"totp": true, "passkeys": true, "required": {"everyone": true}, "graceDays": 7}),
        )
        .await;
    world.reload().await;
    assert_eq!(world.gate("jane").await, Gate::Enroll);
    assert_eq!(world.gate("ned").await, Gate::Enroll);
}
