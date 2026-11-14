mod support;

use astrum_security::two_factor::Requirement;
use astrum_security::two_factor::store::{
    CeremonyKind, Confirmation, NewCeremony, RemoveOutcome, StoreError, Verification,
};
use astrum_security::{Gate, Security, recovery_codes, totp};
use chrono::{Duration, Utc};
use serde_json::json;
use support::{PASSWORD, REALM, current_step, world};

#[tokio::test]
async fn the_first_factor_returns_ten_codes_a_later_one_none_and_the_last_removal_drops_them() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let codes = enrolled.codes.as_ref().unwrap();
    assert_eq!(codes.len(), recovery_codes::SET_SIZE);
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_recovery_code")
            .await,
        10
    );
    assert!(format!("{codes:?}").contains("redacted"));
    assert_eq!(
        world.security.remove_totp("jane").await.unwrap(),
        RemoveOutcome::Removed
    );
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_recovery_code")
            .await,
        0
    );
    let again = world.enroll("jane").await;
    assert_eq!(again.codes.as_ref().unwrap().len(), 10);
    world.user("pat", 0).await;
    world
        .sql(
            "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
             VALUES (gen_random_uuid(), 'pat', 'key', '\\x02', 'localhost', '{}', false, false)",
        )
        .await;
    assert!(world.enroll("pat").await.codes.is_none());
}

#[tokio::test]
async fn an_unconfirmed_secret_never_reaches_the_user_row_and_the_gate_follows_the_confirmation() {
    let world = world().await;
    world.user("jane", 0).await;
    let (ceremony, secret) = world.begin("jane").await;
    assert_eq!(world.gate("jane"), Gate::Open);
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_second_factor WHERE totp_secret IS NOT NULL")
            .await,
        0
    );
    let sealed = world
        .text("SELECT state::text FROM second_factor_ceremony")
        .await;
    assert!(sealed.contains("astrum:v1:"));
    assert!(!sealed.contains(&secret.to_base32()));
    let now = Utc::now();
    let step = totp::step_at(u64::try_from(now.timestamp()).unwrap());
    let outcome = world
        .security
        .confirm_totp("jane", &ceremony, &totp::code_at(&secret, step), now)
        .await
        .unwrap();
    assert!(matches!(outcome, Confirmation::Enrolled { .. }));
    assert_eq!(world.gate("jane"), Gate::Verify);
}

#[tokio::test]
async fn a_clock_drift_gives_a_hint_but_never_enrols() {
    let world = world().await;
    world.user("jane", 0).await;
    let (ceremony, secret) = world.begin("jane").await;
    let now = Utc::now();
    let step = totp::step_at(u64::try_from(now.timestamp()).unwrap());
    let drifted = totp::code_at(&secret, step - 5);
    let outcome = world
        .security
        .confirm_totp("jane", &ceremony, &drifted, now)
        .await
        .unwrap();
    assert_eq!(outcome, Confirmation::ClockDrift { seconds_off: -150 });
    let outcome = world
        .security
        .confirm_totp("jane", &ceremony, &drifted, now)
        .await
        .unwrap();
    assert_eq!(outcome, Confirmation::ClockDrift { seconds_off: -150 });
    assert_eq!(world.gate("jane"), Gate::Open);
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_second_factor WHERE totp_secret IS NOT NULL")
            .await,
        0
    );
    let far = totp::code_at(&secret, step + 50);
    assert_eq!(
        world
            .security
            .confirm_totp("jane", &ceremony, &far, now)
            .await
            .unwrap(),
        Confirmation::Rejected
    );
    let right = totp::code_at(&secret, step);
    assert!(matches!(
        world
            .security
            .confirm_totp("jane", &ceremony, &right, now)
            .await
            .unwrap(),
        Confirmation::Enrolled { .. }
    ));
}

#[tokio::test]
async fn a_ceremony_is_taken_once_expires_and_belongs_to_its_user() {
    let world = world().await;
    world.user("jane", 0).await;
    world.user("pat", 0).await;
    let handle = world
        .security
        .create_ceremony(NewCeremony {
            kind: CeremonyKind::SignIn,
            user_id: "jane",
            realm: REALM,
            state: Some(json!({"k": 1})),
            client_ip: "127.0.0.1",
            lifetime: Duration::minutes(5),
        })
        .await
        .unwrap();
    assert!(
        world
            .security
            .take_ceremony(&handle, CeremonyKind::TotpEnroll)
            .await
            .unwrap()
            .is_none()
    );
    let taken = world
        .security
        .take_ceremony(&handle, CeremonyKind::SignIn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(taken.user_id, "jane");
    assert_eq!(taken.state, Some(json!({"k": 1})));
    assert!(
        world
            .security
            .take_ceremony(&handle, CeremonyKind::SignIn)
            .await
            .unwrap()
            .is_none()
    );

    let (ceremony, secret) = world.begin("jane").await;
    let now = Utc::now();
    let step = totp::step_at(u64::try_from(now.timestamp()).unwrap());
    let right = totp::code_at(&secret, step);
    assert_eq!(
        world
            .security
            .confirm_totp("pat", &ceremony, &right, now)
            .await
            .unwrap(),
        Confirmation::Expired
    );
    world
        .sql("UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second'")
        .await;
    assert_eq!(
        world
            .security
            .confirm_totp("jane", &ceremony, &right, now)
            .await
            .unwrap(),
        Confirmation::Expired
    );
    assert_eq!(world.security.expire_ceremonies().await.unwrap(), 1);

    let (ceremony, secret) = world.begin("jane").await;
    let wrong = totp::code_at(&secret, step + 90);
    for _ in 0..5 {
        assert_eq!(
            world
                .security
                .confirm_totp("jane", &ceremony, &wrong, now)
                .await
                .unwrap(),
            Confirmation::Rejected
        );
    }
    assert_eq!(
        world
            .security
            .confirm_totp("jane", &ceremony, &totp::code_at(&secret, step), now)
            .await
            .unwrap(),
        Confirmation::Expired
    );
}

#[tokio::test]
async fn enrolment_refuses_empty_labels_a_second_totp_and_a_missing_secrets_key() {
    let world = world().await;
    world.user("jane", 0).await;
    assert!(matches!(
        world
            .security
            .begin_totp("jane", REALM, " ", "jane", "")
            .await,
        Err(StoreError::InvalidLabel)
    ));
    assert!(matches!(
        world
            .security
            .begin_totp("jane", REALM, "Astrum", "", "")
            .await,
        Err(StoreError::InvalidLabel)
    ));
    let enrolled = world.enroll("jane").await;
    assert!(matches!(
        world
            .security
            .begin_totp("jane", REALM, "Astrum", "jane", "")
            .await,
        Err(StoreError::AlreadyEnrolled)
    ));
    let summary = world.security.summary("jane").await.unwrap();
    assert!(summary.totp && summary.enrolled());
    assert_eq!(summary.recovery_codes_left, 10);
    assert_eq!(summary.passkeys, 0);
    assert_eq!(
        world
            .verify("jane", &enrolled.code(2), enrolled.at(2))
            .await,
        Verification::Accepted
    );

    let bare = Security::load(world.db.conn().clone()).await.unwrap();
    assert!(matches!(
        bare.begin_totp("jane", REALM, "Astrum", "jane", "").await,
        Err(StoreError::NoSecretsKey)
    ));
}

#[tokio::test]
async fn no_row_holds_a_plain_secret_or_a_plain_code() {
    let world = world().await;
    world.user("jane", 0).await;
    let (ceremony, secret) = world.begin("jane").await;
    let now = Utc::now();
    let step = totp::step_at(u64::try_from(now.timestamp()).unwrap());
    let outcome = world
        .security
        .confirm_totp("jane", &ceremony, &totp::code_at(&secret, step), now)
        .await
        .unwrap();
    let Confirmation::Enrolled { recovery_codes, .. } = outcome else {
        panic!("{outcome:?}");
    };
    let dump = world
        .text(
            "SELECT (SELECT coalesce(string_agg(f::text, ' '), '') FROM user_second_factor f) || \
                    (SELECT coalesce(string_agg(r::text, ' '), '') FROM user_recovery_code r) || \
                    (SELECT coalesce(string_agg(c::text, ' '), '') FROM second_factor_ceremony c)",
        )
        .await
        .to_lowercase();
    assert!(!dump.contains(&secret.to_base32().to_lowercase()));
    for code in recovery_codes.unwrap().as_slice() {
        assert!(!dump.contains(code));
        assert!(!dump.contains(&code.replace('-', "")));
    }
    assert!(dump.contains("astrum:v1:"));
}

#[tokio::test]
async fn an_expired_ceremony_cannot_be_taken() {
    let world = world().await;
    world.user("jane", 0).await;
    let handle = world
        .security
        .create_ceremony(NewCeremony {
            kind: CeremonyKind::SignIn,
            user_id: "jane",
            realm: REALM,
            state: None,
            client_ip: "",
            lifetime: Duration::minutes(5),
        })
        .await
        .unwrap();
    world
        .sql("UPDATE second_factor_ceremony SET expires_at = now() - interval '1 second'")
        .await;
    assert!(
        world
            .security
            .take_ceremony(&handle, CeremonyKind::SignIn)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn an_unknown_user_is_reported_as_such() {
    let world = world().await;
    assert!(matches!(
        world
            .security
            .begin_totp("nobody", REALM, "Astrum", "nobody", "")
            .await,
        Err(StoreError::UnknownUser)
    ));
    assert!(matches!(
        world
            .security
            .set_requirement("nobody", Requirement::Required)
            .await,
        Err(StoreError::UnknownUser)
    ));
    assert!(matches!(
        world.security.reset("nobody").await,
        Err(StoreError::UnknownUser)
    ));
}

#[tokio::test]
async fn codes_can_be_regenerated_for_a_passkey_only_user_without_a_row() {
    let world = world().await;
    world.user("pat", 0).await;
    world.user("jane", 0).await;
    world.passkey("pat", 1).await;
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_second_factor WHERE user_id = 'pat'")
            .await,
        0
    );
    let codes = world
        .security
        .regenerate_recovery_codes("pat")
        .await
        .unwrap();
    assert_eq!(codes.len(), 10);
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_recovery_code WHERE user_id = 'pat'")
            .await,
        10
    );
    assert!(matches!(
        world.security.regenerate_recovery_codes("jane").await,
        Err(StoreError::NotEnrolled)
    ));
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_second_factor WHERE user_id = 'jane'")
            .await,
        0
    );
}

#[tokio::test]
async fn a_failed_model_reload_after_the_commit_keeps_the_codes_and_closes_the_sessions() {
    let world = world().await;
    world.user("jane", 0).await;
    let subject = world.security.authenticate("jane", PASSWORD).await.unwrap();
    world
        .security
        .open_session(&subject, "127.0.0.1", "test", false)
        .await
        .unwrap();
    let (ceremony, secret) = world.begin("jane").await;
    let revision = world
        .number("SELECT value FROM revision WHERE name = 'security'")
        .await;
    world
        .sql("ALTER TABLE security_user_role RENAME TO security_user_role_gone")
        .await;
    let now = Utc::now();
    let code = totp::code_at(&secret, current_step(now));
    let outcome = world
        .security
        .confirm_totp("jane", &ceremony, &code, now)
        .await
        .unwrap();
    let Confirmation::Enrolled { recovery_codes, .. } = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(recovery_codes.unwrap().len(), 10);
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_recovery_code WHERE user_id = 'jane'")
            .await,
        10
    );
    assert_eq!(
        world
            .number("SELECT value FROM revision WHERE name = 'security'")
            .await,
        revision + 1
    );
    let outcome = world.security.reset("jane").await.unwrap();
    assert_eq!(outcome.sessions_closed, 1);
    assert_eq!(world.number("SELECT count(*) FROM user_session").await, 0);
}

fn sign_in<'a>(user: &'a str) -> NewCeremony<'a> {
    NewCeremony {
        kind: CeremonyKind::SignIn,
        user_id: user,
        realm: REALM,
        state: None,
        client_ip: "127.0.0.1",
        lifetime: Duration::minutes(5),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_ceremonies_of_users_with_expired_rows_do_not_deadlock() {
    let world = world().await;
    world.user("jane", 0).await;
    world.user("pat", 0).await;
    for _ in 0..150 {
        world
            .sql(
                "INSERT INTO second_factor_ceremony (handle_hash, kind, user_id, realm, expires_at) \
                 SELECT md5(random()::text) || u, 'sign-in', u, 'r', now() - interval '1 minute' \
                 FROM (VALUES ('jane'), ('pat')) AS users (u)",
            )
            .await;
        let (jane, pat) = tokio::join!(
            world.security.replace_ceremony(sign_in("jane")),
            world.security.replace_ceremony(sign_in("pat")),
        );
        jane.unwrap();
        pat.unwrap();
    }
    assert_eq!(
        world
            .number("SELECT count(*) FROM second_factor_ceremony")
            .await,
        2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_replacements_for_one_user_leave_one_live_ceremony() {
    let world = world().await;
    world.user("jane", 0).await;
    for _ in 0..50 {
        let (first, second) = tokio::join!(
            world.security.replace_ceremony(sign_in("jane")),
            world.security.replace_ceremony(sign_in("jane")),
        );
        let (first, second) = (first.unwrap(), second.unwrap());
        assert_eq!(
            world
                .number("SELECT count(*) FROM second_factor_ceremony WHERE user_id = 'jane'")
                .await,
            1
        );
        let mut alive = 0;
        for handle in [&first, &second] {
            if world
                .security
                .take_ceremony(handle, CeremonyKind::SignIn)
                .await
                .unwrap()
                .is_some()
            {
                alive += 1;
            }
        }
        assert_eq!(alive, 1);
    }
}
