mod support;

use astrum_security::SecretBox;
use astrum_security::two_factor::store::{RecoveryOutcome, Verification};
use support::world;

#[tokio::test]
async fn a_code_is_accepted_once_and_an_older_step_is_refused() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let at = enrolled.at(2);
    let code = enrolled.code(2);
    assert_eq!(
        world.verify("jane", &code, at).await,
        Verification::Accepted
    );
    assert_eq!(
        world.verify("jane", &code, at).await,
        Verification::Rejected
    );
    assert_eq!(
        world.verify("jane", &enrolled.code(1), at).await,
        Verification::Rejected
    );
    assert_eq!(
        world
            .verify("jane", &enrolled.code(3), enrolled.at(3))
            .await,
        Verification::Accepted
    );
}

#[tokio::test]
async fn the_code_of_the_enrollment_itself_cannot_be_replayed() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    assert_eq!(
        world
            .verify("jane", &enrolled.code(0), enrolled.at(0))
            .await,
        Verification::Rejected
    );
}

#[tokio::test]
async fn two_concurrent_verifications_of_one_code_accept_exactly_one() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let code = enrolled.code(2);
    let at = enrolled.at(2);
    let tasks: Vec<_> = (0..6)
        .map(|_| {
            let security = world.security.clone();
            let code = code.clone();
            tokio::spawn(async move { security.verify_totp("jane", &code, at).await.unwrap() })
        })
        .collect();
    let mut accepted = 0;
    for task in tasks {
        if task.await.unwrap().result == Verification::Accepted {
            accepted += 1;
        }
    }
    assert_eq!(accepted, 1);
}

#[tokio::test]
async fn ten_failures_lock_the_factor_and_every_new_lock_doubles() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let at = enrolled.at(2);
    let wrong = enrolled.wrong(2);
    for _ in 0..9 {
        assert_eq!(
            world.verify("jane", &wrong, at).await,
            Verification::Rejected
        );
    }
    assert!(matches!(
        world.verify("jane", &wrong, at).await,
        Verification::Locked { .. }
    ));
    assert!(matches!(
        world.verify("jane", &enrolled.code(2), at).await,
        Verification::Locked { .. }
    ));
    let first = world
        .number("SELECT extract(epoch FROM locked_until - now())::bigint FROM user_second_factor")
        .await;
    assert!((880..=900).contains(&first), "{first}");
    world
        .sql("UPDATE user_second_factor SET locked_until = now() - interval '1 second'")
        .await;
    for _ in 0..10 {
        world.verify("jane", &wrong, at).await;
    }
    let second = world
        .number("SELECT extract(epoch FROM locked_until - now())::bigint FROM user_second_factor")
        .await;
    assert!((1780..=1800).contains(&second), "{second}");
    assert_eq!(
        world
            .number("SELECT lock_level FROM user_second_factor")
            .await,
        2
    );
    world
        .sql("UPDATE user_second_factor SET locked_until = now() - interval '1 second'")
        .await;
    assert_eq!(
        world.verify("jane", &enrolled.code(2), at).await,
        Verification::Accepted
    );
}

#[tokio::test]
async fn a_success_resets_the_failures_and_the_lock_level() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    world
        .sql("UPDATE user_second_factor SET lock_level = 3")
        .await;
    for _ in 0..5 {
        world
            .verify("jane", &enrolled.wrong(2), enrolled.at(2))
            .await;
    }
    assert_eq!(
        world
            .number("SELECT failed_codes FROM user_second_factor")
            .await,
        5
    );
    assert_eq!(
        world
            .verify("jane", &enrolled.code(2), enrolled.at(2))
            .await,
        Verification::Accepted
    );
    assert_eq!(
        world
            .number("SELECT failed_codes FROM user_second_factor")
            .await,
        0
    );
    assert_eq!(
        world
            .number("SELECT lock_level FROM user_second_factor")
            .await,
        0
    );
}

#[tokio::test]
async fn parallel_failures_are_all_counted() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let wrong = enrolled.wrong(2);
    let at = enrolled.at(2);
    let tasks: Vec<_> = (0..8)
        .map(|_| {
            let security = world.security.clone();
            let wrong = wrong.clone();
            tokio::spawn(async move { security.verify_totp("jane", &wrong, at).await.unwrap() })
        })
        .collect();
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(
        world
            .number("SELECT failed_codes FROM user_second_factor")
            .await,
        8
    );
}

#[tokio::test]
async fn a_recovery_code_works_once_and_two_parallel_uses_give_one_success() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let codes = enrolled.codes.as_ref().unwrap().as_slice().to_vec();
    assert_eq!(
        world
            .security
            .consume_recovery_code("jane", &codes[0])
            .await
            .unwrap(),
        RecoveryOutcome::Accepted { left: 9 }
    );
    assert_eq!(
        world
            .security
            .consume_recovery_code("jane", &codes[0])
            .await
            .unwrap(),
        RecoveryOutcome::Rejected
    );
    let upper = codes[1].to_uppercase().replace('-', " ");
    let tasks: Vec<_> = (0..6)
        .map(|_| {
            let security = world.security.clone();
            let upper = upper.clone();
            tokio::spawn(async move {
                security
                    .consume_recovery_code("jane", &upper)
                    .await
                    .unwrap()
            })
        })
        .collect();
    let mut accepted = 0;
    for task in tasks {
        if matches!(task.await.unwrap(), RecoveryOutcome::Accepted { .. }) {
            accepted += 1;
        }
    }
    assert_eq!(accepted, 1);
    assert_eq!(
        world
            .security
            .consume_recovery_code("jane", "not a code")
            .await
            .unwrap(),
        RecoveryOutcome::Rejected
    );
}

#[tokio::test]
async fn an_unreadable_secret_is_a_wrong_code_and_a_missing_factor_looks_the_same() {
    let world = world().await;
    world.user("jane", 0).await;
    world.user("ghost", 0).await;
    let enrolled = world.enroll("jane").await;
    let other = SecretBox::from_key([1u8; 32]).seal(&enrolled.secret.to_base32());
    world
        .sql(&format!(
            "UPDATE user_second_factor SET totp_secret = '{other}'"
        ))
        .await;
    assert_eq!(
        world
            .verify("jane", &enrolled.code(2), enrolled.at(2))
            .await,
        Verification::Rejected
    );
    world
        .sql("UPDATE user_second_factor SET totp_secret = 'garbage'")
        .await;
    assert_eq!(
        world
            .verify("jane", &enrolled.code(3), enrolled.at(3))
            .await,
        Verification::Rejected
    );
    assert_eq!(
        world
            .verify("ghost", &enrolled.code(3), enrolled.at(3))
            .await,
        Verification::Rejected
    );
    assert_eq!(
        world
            .verify("ghost", &enrolled.wrong(3), enrolled.at(3))
            .await,
        Verification::Rejected
    );
    assert_eq!(
        world
            .number("SELECT count(*) FROM user_second_factor WHERE user_id = 'ghost'")
            .await,
        0
    );
}

#[tokio::test]
async fn the_lock_never_exceeds_one_day() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    world
        .sql("UPDATE user_second_factor SET lock_level = 9")
        .await;
    for _ in 0..10 {
        world
            .verify("jane", &enrolled.wrong(2), enrolled.at(2))
            .await;
    }
    let seconds = world
        .number("SELECT extract(epoch FROM locked_until - now())::bigint FROM user_second_factor")
        .await;
    assert!((86_300..=86_400).contains(&seconds), "{seconds}");
}

#[tokio::test]
async fn a_right_recovery_code_lifts_the_lock_of_the_totp_factor() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let codes = enrolled.codes.as_ref().unwrap().as_slice().to_vec();
    for _ in 0..10 {
        world
            .verify("jane", &enrolled.wrong(2), enrolled.at(2))
            .await;
    }
    assert!(matches!(
        world
            .verify("jane", &enrolled.code(2), enrolled.at(2))
            .await,
        Verification::Locked { .. }
    ));
    assert_eq!(
        world
            .security
            .consume_recovery_code("jane", &codes[0])
            .await
            .unwrap(),
        RecoveryOutcome::Accepted { left: 9 }
    );
    assert_eq!(
        world
            .number("SELECT lock_level FROM user_second_factor")
            .await,
        0
    );
    assert_eq!(
        world
            .verify("jane", &enrolled.code(2), enrolled.at(2))
            .await,
        Verification::Accepted
    );
}

#[tokio::test]
async fn the_report_tells_the_audit_what_the_client_must_not_learn() {
    let world = world().await;
    world.user("jane", 0).await;
    let enrolled = world.enroll("jane").await;
    let early = world
        .report("jane", &enrolled.code(1), enrolled.at(2))
        .await;
    assert_eq!(early.result, Verification::Accepted);
    assert_eq!(early.detail.offset_steps, Some(-1));
    assert!(!early.detail.fresh_lock && !early.detail.unreadable_secret);
    let wrong = enrolled.wrong(3);
    for attempt in 1..=10 {
        let report = world.report("jane", &wrong, enrolled.at(3)).await;
        assert_eq!(report.detail.fresh_lock, attempt == 10, "{attempt}");
        assert_eq!(report.detail.offset_steps, None);
    }
    let locked = world
        .report("jane", &enrolled.code(3), enrolled.at(3))
        .await;
    assert!(matches!(locked.result, Verification::Locked { .. }));
    assert!(!locked.detail.fresh_lock);
    world
        .sql("UPDATE user_second_factor SET locked_until = NULL, totp_secret = 'garbage'")
        .await;
    let unreadable = world
        .report("jane", &enrolled.code(4), enrolled.at(4))
        .await;
    assert_eq!(unreadable.result, Verification::Rejected);
    assert!(unreadable.detail.unreadable_secret);
}
