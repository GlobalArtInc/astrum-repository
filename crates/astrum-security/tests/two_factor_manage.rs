mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use astrum_security::two_factor::Requirement;
use astrum_security::two_factor::store::{
    RecoveryOutcome, RemoveOutcome, StoreError, ensure_factor_row,
};
use astrum_security::{Gate, Security, recovery_codes, totp};
use chrono::Utc;
use support::{PASSWORD, World, current_step, world};

#[tokio::test]
async fn regenerating_replaces_the_set_and_keeps_the_hash_context() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let old = enrolled.codes.as_ref().unwrap().as_slice().to_vec();
    let handle = world
        .text("SELECT webauthn_handle::text FROM user_second_factor")
        .await;
    let fresh = world
        .security
        .regenerate_recovery_codes("jane")
        .await
        .unwrap();
    assert_eq!(fresh.len(), 10);
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_recovery_code")
            .await,
        10
    );
    assert_eq!(
        world
            .security
            .consume_recovery_code("jane", &old[0])
            .await
            .unwrap(),
        RecoveryOutcome::Rejected
    );
    let first = &fresh.as_slice()[0];
    let stored = recovery_codes::hash(&handle, first).unwrap();
    assert_eq!(
        world
            .number(&format!(
                "SELECT count(*) FROM user_recovery_code WHERE code_hash = '{stored}'"
            ))
            .await,
        1
    );
    assert_eq!(
        world
            .security
            .consume_recovery_code("jane", first)
            .await
            .unwrap(),
        RecoveryOutcome::Accepted { left: 9 }
    );
    world.security.remove_totp("jane").await.unwrap();
    world.enroll("jane").await;
    assert_eq!(
        world
            .text("SELECT webauthn_handle::text FROM user_second_factor")
            .await,
        handle
    );
    assert!(matches!(
        world.security.regenerate_recovery_codes("nobody").await,
        Err(StoreError::NotEnrolled)
    ));
}

#[tokio::test]
async fn the_last_factor_stays_while_the_user_must_have_one() {
    let world = world().await;
    world.user("jane", 0).await;
    world.user("pat", 0).await;
    world.enroll("jane").await;
    world.enroll("pat").await;
    world
        .security
        .set_requirement("jane", Requirement::Required)
        .await
        .unwrap();
    assert_eq!(
        world.security.remove_totp("jane").await.unwrap(),
        RemoveOutcome::LastFactorRequired
    );
    assert_eq!(world.gate("jane"), Gate::Verify);
    world
        .sql(
            "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
             VALUES (gen_random_uuid(), 'jane', 'key', '\\x01', 'localhost', '{}', false, false)",
        )
        .await;
    assert_eq!(
        world.security.remove_totp("jane").await.unwrap(),
        RemoveOutcome::Removed
    );
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_recovery_code WHERE user_id = 'jane'")
            .await,
        10
    );
    assert_eq!(
        world.security.remove_totp("pat").await.unwrap(),
        RemoveOutcome::Removed
    );
    assert_eq!(
        world.security.remove_totp("pat").await.unwrap(),
        RemoveOutcome::NotEnrolled
    );
}

#[tokio::test]
async fn a_requirement_reaches_the_gate_and_the_other_node() {
    let world = world().await;
    world.user("jane", 30).await;
    let other = Security::load(world.db.conn().clone()).await.unwrap();
    world
        .security
        .set_requirement("jane", Requirement::Required)
        .await
        .unwrap();
    assert_eq!(world.gate("jane"), Gate::Enroll);
    assert!(other.sync().await.unwrap());
    assert_eq!(
        other.model().second_factor_gate("jane", Utc::now()),
        Gate::Enroll
    );
    assert!(matches!(
        world
            .security
            .set_requirement("jane", Requirement::Unreadable)
            .await,
        Err(StoreError::InvalidRequirement)
    ));
}

#[tokio::test]
async fn a_reset_clears_every_factor_and_sets_the_deadline() {
    let world = world().await;
    world.user("jane", 30).await;
    let subject = world.security.authenticate("jane", PASSWORD).await.unwrap();
    world
        .security
        .open_session(&subject, "127.0.0.1", "test")
        .await
        .unwrap();
    world.enroll("jane").await;
    world
        .security
        .set_requirement("jane", Requirement::Required)
        .await
        .unwrap();
    world
        .sql(
            "INSERT INTO user_passkey (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
             VALUES (gen_random_uuid(), 'jane', 'key', '\\x01', 'localhost', '{}', false, false)",
        )
        .await;
    world
        .sql("UPDATE user_second_factor SET lock_level = 2, failed_codes = 4")
        .await;
    let handle = world
        .text("SELECT webauthn_handle::text FROM user_second_factor")
        .await;
    let outcome = world.security.reset("jane").await.unwrap();
    assert_eq!(outcome.sessions_closed, 1);
    for table in ["user_passkey", "user_recovery_code", "user_session"] {
        assert_eq!(
            world.number(&format!("SELECT count(*) FROM {table}")).await,
            0,
            "{table}"
        );
    }
    let row = "SELECT totp_secret IS NULL AND totp_confirmed_at IS NULL AND lock_level = 0 \
               AND failed_codes = 0 AND requirement = 'required' FROM user_second_factor";
    assert_eq!(world.text(row).await, "true");
    let deadline = "SELECT extract(epoch FROM now() - enroll_by)::bigint FROM user_second_factor";
    assert!((0..30).contains(&world.number(deadline).await));
    assert_eq!(
        world
            .text("SELECT webauthn_handle::text FROM user_second_factor")
            .await,
        handle
    );
    assert_eq!(world.gate("jane"), Gate::Enroll);
}

const PAIRS: usize = 200;

fn pause(round: usize, centre: Duration, spread: Duration) -> Duration {
    let spread = spread.as_micros().max(1);
    let offset = (round as u128 * 7919) % (2 * spread);
    (centre + Duration::from_micros(offset as u64))
        .saturating_sub(Duration::from_micros(spread as u64))
}

fn hash_time() -> Duration {
    let started = std::time::Instant::now();
    recovery_codes::hash("timing", "abcde-23456").unwrap();
    started.elapsed()
}

async fn seed_code(world: &World, user: &str, enrolled: bool) -> String {
    let context = ensure_factor_row(world.db.conn(), user).await.unwrap();
    let code = recovery_codes::generate_set().remove(0);
    let hash = recovery_codes::hash(&context, &code).unwrap();
    world
        .sql(&format!(
            "DELETE FROM user_recovery_code WHERE user_id = '{user}'"
        ))
        .await;
    world
        .sql(&format!(
            "INSERT INTO user_recovery_code (user_id, code_hash) VALUES ('{user}', '{hash}')"
        ))
        .await;
    if enrolled {
        world
            .sql(&format!(
                "UPDATE user_second_factor SET totp_secret = 'sealed', totp_confirmed_at = now() \
                 WHERE user_id = '{user}'"
            ))
            .await;
    }
    code
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn confirming_and_resetting_at_once_never_deadlock() {
    let world = world().await;
    world.user("jane", 30).await;
    for round in 0..PAIRS {
        world.passkey("jane", 1).await;
        let (ceremony, secret) = world.begin("jane").await;
        let now = Utc::now();
        let code = totp::code_at(&secret, current_step(now));
        let (confirmer, resetter) = (world.security.clone(), world.security.clone());
        let delay = pause(round, Duration::from_millis(8), Duration::from_millis(8));
        let confirm =
            tokio::spawn(
                async move { confirmer.confirm_totp("jane", &ceremony, &code, now).await },
            );
        let reset = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            resetter.reset("jane").await
        });
        confirm.await.unwrap().unwrap();
        reset.await.unwrap().unwrap();
        world.security.reset("jane").await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn consuming_and_regenerating_at_once_never_deadlock() {
    const CONSUMERS: usize = 8;
    let world = world().await;
    world.user("jane", 30).await;
    seed_code(&world, "jane", true).await;
    let mut codes = world
        .security
        .regenerate_recovery_codes("jane")
        .await
        .unwrap()
        .as_slice()
        .to_vec();
    let single = hash_time();
    for round in 0..PAIRS {
        let consumers: Vec<_> = codes
            .iter()
            .take(CONSUMERS)
            .enumerate()
            .map(|(index, code)| {
                let security = world.security.clone();
                let code = code.clone();
                let delay = pause(round + index * 13, single * 9, Duration::from_millis(10));
                tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    security.consume_recovery_code("jane", &code).await
                })
            })
            .collect();
        let regenerator = world.security.clone();
        let regenerate =
            tokio::spawn(async move { regenerator.regenerate_recovery_codes("jane").await });
        for consume in consumers {
            consume.await.unwrap().unwrap();
        }
        codes = regenerate.await.unwrap().unwrap().as_slice().to_vec();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn consuming_and_resetting_at_once_never_deadlock() {
    let world = world().await;
    world.user("jane", 30).await;
    let single = hash_time();
    for round in 0..PAIRS {
        let code = seed_code(&world, "jane", true).await;
        let (consumer, resetter) = (world.security.clone(), world.security.clone());
        let delay = pause(round, single, Duration::from_millis(8));
        let consume =
            tokio::spawn(async move { consumer.consume_recovery_code("jane", &code).await });
        let reset = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            resetter.reset("jane").await
        });
        consume.await.unwrap().unwrap();
        reset.await.unwrap().unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn removing_and_consuming_at_once_never_deadlock() {
    let world = world().await;
    world.user("jane", 30).await;
    let single = hash_time();
    for round in 0..PAIRS {
        let code = seed_code(&world, "jane", true).await;
        let (consumer, remover) = (world.security.clone(), world.security.clone());
        let delay = pause(round, single, Duration::from_millis(8));
        let consume =
            tokio::spawn(async move { consumer.consume_recovery_code("jane", &code).await });
        let remove = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            remover.remove_totp("jane").await
        });
        consume.await.unwrap().unwrap();
        remove.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn hashing_recovery_codes_does_not_stall_the_runtime() {
    let world = world().await;
    world.user("jane", 0).await;
    world.enroll("jane").await;
    world
        .security
        .regenerate_recovery_codes("jane")
        .await
        .unwrap();
    let running = Arc::new(AtomicBool::new(true));
    let ticker = {
        let running = running.clone();
        tokio::spawn(async move {
            let mut widest = Duration::ZERO;
            while running.load(Ordering::SeqCst) {
                let started = std::time::Instant::now();
                tokio::time::sleep(Duration::from_millis(5)).await;
                widest = widest.max(started.elapsed());
            }
            widest
        })
    };
    tokio::time::sleep(Duration::from_millis(20)).await;
    world
        .security
        .regenerate_recovery_codes("jane")
        .await
        .unwrap();
    running.store(false, Ordering::SeqCst);
    let widest = ticker.await.unwrap();
    assert!(widest < Duration::from_millis(60), "{widest:?}");
}
