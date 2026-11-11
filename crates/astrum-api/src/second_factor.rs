use astrum_db::audit::Event;
use astrum_security::two_factor::store::{RecoveryOutcome, StoreError, Verification};
use astrum_security::{Pending, Subject};
use chrono::Utc;
use serde_json::json;

use crate::routes::internal::session::audit_rate_limited;
use crate::{ApiError, AppState, Caller};

pub const CODE_MAX_BYTES: usize = 64;
pub const DOMAIN: &str = "security.twofactor";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Totp,
    Recovery,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Totp => "totp",
            Method::Recovery => "recovery",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Accepted {
        left: Option<u32>,
        offset: Option<i8>,
    },
    Wrong,
    Locked(u64),
}

pub async fn check(
    state: &AppState,
    user: &str,
    method: Method,
    code: &str,
) -> Result<Step, ApiError> {
    if code.len() > CODE_MAX_BYTES {
        return Ok(Step::Wrong);
    }
    match method {
        Method::Totp => check_totp(state, user, code).await,
        Method::Recovery => {
            let outcome = state
                .security()
                .consume_recovery_code(user, code)
                .await
                .map_err(store_error)?;
            Ok(match outcome {
                RecoveryOutcome::Accepted { left } => Step::Accepted {
                    left: Some(left),
                    offset: None,
                },
                RecoveryOutcome::Rejected => Step::Wrong,
            })
        }
    }
}

async fn check_totp(state: &AppState, user: &str, code: &str) -> Result<Step, ApiError> {
    let report = state
        .security()
        .verify_totp(user, code, Utc::now())
        .await
        .map_err(store_error)?;
    if report.detail.unreadable_secret {
        state
            .audit(
                &Caller(None),
                Event::new(DOMAIN, "totp.unreadable", user).with(json!({"userId": user})),
            )
            .await;
    }
    Ok(match report.result {
        Verification::Accepted => Step::Accepted {
            left: None,
            offset: report.detail.offset_steps,
        },
        Verification::Rejected => Step::Wrong,
        Verification::Locked { until } => {
            if report.detail.fresh_lock {
                state
                    .audit(
                        &Caller(None),
                        Event::new(DOMAIN, "locked", user)
                            .with(json!({"userId": user, "until": until.to_rfc3339()})),
                    )
                    .await;
            }
            let seconds = (until.to_utc() - Utc::now()).num_seconds().max(0) + 1;
            Step::Locked(u64::try_from(seconds).unwrap_or(1))
        }
    })
}

pub fn subject_of(state: &AppState, pending: &Pending) -> Option<Subject> {
    state
        .security()
        .model()
        .local_subject(&pending.user_id, &pending.realm)
}

pub async fn audit_recovery_used(state: &AppState, subject: &Subject, left: u32) {
    state
        .audit(
            &Caller(Some(subject.clone())),
            Event::new(DOMAIN, "recovery.used", &subject.user_id)
                .with(json!({"userId": subject.user_id, "left": left})),
        )
        .await;
}

pub async fn record_failure(state: &AppState, user: &str, method: Method, via: &str) {
    let security = state.security();
    security.sign_in_failed(user);
    tracing::info!(
        user,
        method = method.as_str(),
        via,
        "a second step of a sign-in failed"
    );
    state
        .audit(
            &Caller(None),
            Event::new(DOMAIN, "verify.failed", user)
                .with(json!({"userId": user, "method": method.as_str()})),
        )
        .await;
    if security.sign_in_blocked(user) {
        audit_rate_limited(state, user, crate::client::current(), via).await;
    }
}

fn store_error(error: StoreError) -> ApiError {
    ApiError::internal(error)
}
