use std::collections::HashMap;

use astrum_db::audit::Event;
use astrum_db::entity::{user_passkey, user_second_factor};
use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, QuerySelect};
use serde::{Deserialize, Serialize};
use url::{Host, Url};

use crate::catalogue::DEFAULT_SOURCE;
use crate::model::{SecurityModel, tolerant};
use crate::{Error, realm};

pub mod qr;
pub mod store;

pub const POLICY_KEY: &str = "security.twoFactor";
pub const POLICY_LOCK: i64 = i64::from_be_bytes(*b"astr2fpl");
pub const AUDIT_DOMAIN: &str = "security.twofactor";
pub const FAILURE_VERIFY: &str = "SECOND_FACTOR_REQUIRED";
pub const FAILURE_ENROLL: &str = "SECOND_FACTOR_ENROLLMENT_REQUIRED";
pub const FAILURE_WRONG_PASSWORD: &str = "INCORRECT_CREDENTIALS";
const ADMINISTRATORS: &str = "astrum:*";

pub const fn passkey_flow_implemented() -> bool {
    false
}

pub async fn lock_policy(db: &impl ConnectionTrait) -> Result<(), DbErr> {
    db.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({POLICY_LOCK})"))
        .await?;
    Ok(())
}

pub async fn methods_in(db: &impl ConnectionTrait) -> Result<Methods, Error> {
    let policy = tolerant::<TwoFactorPolicy>(db, POLICY_KEY)
        .await?
        .unwrap_or_default();
    let base = tolerant::<BaseUrl>(db, BASE_URL_KEY)
        .await?
        .filter(|base| base.enabled && !base.url.trim().is_empty())
        .map(|base| base.url.trim().trim_end_matches('/').to_owned());
    Ok(policy.methods(base.as_deref()))
}

pub fn reset_event(user_id: &str) -> Event {
    Event::new(AUDIT_DOMAIN, "reset", user_id).with(serde_json::json!({"target": user_id}))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    Policy,
    Required,
    Exempt,
    Unreadable,
}

impl Requirement {
    pub fn as_str(self) -> &'static str {
        match self {
            Requirement::Policy => "policy",
            Requirement::Unreadable => "unreadable",
            Requirement::Required => "required",
            Requirement::Exempt => "exempt",
        }
    }

    pub fn from_stored(value: &str) -> Self {
        match value {
            "policy" => Requirement::Policy,
            "required" => Requirement::Required,
            "exempt" => Requirement::Exempt,
            other => {
                tracing::error!(
                    requirement = other,
                    "a stored two-factor requirement is unknown; the user is gated"
                );
                Requirement::Unreadable
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorSummary {
    pub enrolled: bool,
    pub requirement: Requirement,
    pub enroll_by: Option<DateTime<FixedOffset>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Standing {
    pub required: bool,
    pub enroll_by: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Open,
    Verify,
    Enroll,
}

impl Gate {
    pub fn failure_reason(self) -> &'static str {
        match self {
            Gate::Open => FAILURE_WRONG_PASSWORD,
            Gate::Verify => FAILURE_VERIFY,
            Gate::Enroll => FAILURE_ENROLL,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    pub user_id: String,
    pub realm: String,
    pub gate: Gate,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RequiredBy {
    pub everyone: bool,
    pub administrators: bool,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RelyingPartyPolicy {
    pub id: Option<String>,
    pub origins: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TwoFactorPolicy {
    pub totp: bool,
    pub passkeys: bool,
    pub required: RequiredBy,
    pub grace_days: u32,
    pub required_since: Option<DateTime<Utc>>,
    pub relying_party: RelyingPartyPolicy,
}

impl Default for TwoFactorPolicy {
    fn default() -> Self {
        Self {
            totp: true,
            passkeys: true,
            required: RequiredBy::default(),
            grace_days: 7,
            required_since: None,
            relying_party: RelyingPartyPolicy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Methods {
    Available,
    NoneEnabled,
    PasskeysWithoutId,
}

impl TwoFactorPolicy {
    pub fn methods(&self, base_url: Option<&str>) -> Methods {
        match (self.totp, self.passkeys) {
            (true, _) => Methods::Available,
            (false, false) => Methods::NoneEnabled,
            (false, true) if !passkey_flow_implemented() => Methods::NoneEnabled,
            (false, true) if self.relying_party_id_known(base_url) => Methods::Available,
            (false, true) => Methods::PasskeysWithoutId,
        }
    }

    pub fn enrollable(&self) -> bool {
        self.totp || (self.passkeys && passkey_flow_implemented())
    }

    fn relying_party_id_known(&self, base_url: Option<&str>) -> bool {
        self.relying_party
            .id
            .as_deref()
            .is_some_and(|id| !id.is_empty())
            || base_url
                .and_then(|base| Url::parse(base).ok())
                .is_some_and(|url| matches!(url.host(), Some(Host::Domain(_))))
    }
}

pub struct Loaded {
    pub policy: TwoFactorPolicy,
    pub factors: HashMap<String, FactorSummary>,
}

pub async fn load(db: &impl ConnectionTrait) -> Result<Loaded, Error> {
    let policy = tolerant::<TwoFactorPolicy>(db, POLICY_KEY)
        .await?
        .unwrap_or_default();
    let with_passkeys: Vec<String> = user_passkey::Entity::find()
        .select_only()
        .column(user_passkey::Column::UserId)
        .distinct()
        .into_tuple()
        .all(db)
        .await?;
    let mut factors = HashMap::new();
    for row in user_second_factor::Entity::find().all(db).await? {
        let enrolled = row.totp_secret.is_some() || row.totp_confirmed_at.is_some();
        factors.insert(
            row.user_id,
            FactorSummary {
                enrolled,
                requirement: Requirement::from_stored(&row.requirement),
                enroll_by: row.enroll_by,
            },
        );
    }
    for user_id in with_passkeys {
        factors
            .entry(user_id)
            .and_modify(|summary| summary.enrolled = true)
            .or_insert(FactorSummary {
                enrolled: true,
                requirement: Requirement::Policy,
                enroll_by: None,
            });
    }
    Ok(Loaded { policy, factors })
}

impl SecurityModel {
    pub fn second_factor_gate(&self, user_id: &str, now: DateTime<Utc>) -> Gate {
        let Some(user) = self.users.get(user_id) else {
            return Gate::Verify;
        };
        let summary = self.second_factor.get(user_id);
        if summary.is_some_and(|summary| summary.enrolled) {
            return Gate::Verify;
        }
        let requirement = summary.map_or(Requirement::Policy, |summary| summary.requirement);
        let deadline = || self.enroll_deadline(user.created_at, summary);
        let gate = match requirement {
            Requirement::Exempt => Gate::Open,
            Requirement::Unreadable => Gate::Enroll,
            Requirement::Required => Self::after(now, deadline()),
            Requirement::Policy if self.policy_requires(user_id) => Self::after(now, deadline()),
            Requirement::Policy => Gate::Open,
        };
        if gate == Gate::Enroll
            && requirement != Requirement::Unreadable
            && !self.two_factor.enrollable()
        {
            tracing::warn!(
                user_id,
                "a second factor is required but the policy leaves no method to enroll; the sign-in is not gated"
            );
            return Gate::Open;
        }
        gate
    }

    pub fn requires_second_factor(&self, requirement: Requirement, user_id: &str) -> bool {
        match requirement {
            Requirement::Required | Requirement::Unreadable => true,
            Requirement::Exempt => false,
            Requirement::Policy => self.policy_requires(user_id),
        }
    }

    pub fn second_factor_standing(&self, user_id: &str) -> Standing {
        let summary = self.second_factor.get(user_id);
        let requirement = summary.map_or(Requirement::Policy, |summary| summary.requirement);
        let required = self.requires_second_factor(requirement, user_id);
        let enroll_by = self
            .users
            .get(user_id)
            .filter(|_| required)
            .and_then(|user| self.enroll_deadline(user.created_at, summary));
        Standing {
            required,
            enroll_by,
        }
    }

    fn after(now: DateTime<Utc>, deadline: Option<DateTime<Utc>>) -> Gate {
        match deadline {
            Some(deadline) if now < deadline => Gate::Open,
            _ => Gate::Enroll,
        }
    }

    pub fn is_administrator(&self, user_id: &str) -> bool {
        self.subject(user_id, DEFAULT_SOURCE, realm::LOCAL)
            .is_permitted(ADMINISTRATORS)
    }

    fn policy_requires(&self, user_id: &str) -> bool {
        let required = &self.two_factor.required;
        if required.everyone || (required.administrators && self.is_administrator(user_id)) {
            return true;
        }
        if required.roles.is_empty() {
            return false;
        }
        let subject = self.subject(user_id, DEFAULT_SOURCE, realm::LOCAL);
        required
            .roles
            .iter()
            .any(|role| subject.roles.contains(role))
    }

    fn enroll_deadline(
        &self,
        created_at: DateTime<FixedOffset>,
        summary: Option<&FactorSummary>,
    ) -> Option<DateTime<Utc>> {
        if let Some(enroll_by) = summary.and_then(|summary| summary.enroll_by) {
            return Some(enroll_by.to_utc());
        }
        let created_at = created_at.to_utc();
        let start = self
            .two_factor
            .required_since
            .map_or(created_at, |since| since.max(created_at));
        let grace = Duration::try_days(i64::from(self.two_factor.grace_days))?;
        start.checked_add_signed(grace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(totp: bool, passkeys: bool, id: Option<&str>) -> TwoFactorPolicy {
        TwoFactorPolicy {
            totp,
            passkeys,
            relying_party: RelyingPartyPolicy {
                id: id.map(str::to_owned),
                origins: Vec::new(),
            },
            ..TwoFactorPolicy::default()
        }
    }

    #[test]
    fn passkeys_do_not_count_as_a_method_before_they_can_be_enrolled() {
        let base = Some("https://astrum.example.org");
        assert_eq!(policy(true, true, None).methods(base), Methods::Available);
        assert_eq!(policy(true, false, None).methods(None), Methods::Available);
        for known in [None, Some("example.org")] {
            assert_eq!(
                policy(false, true, known).methods(base),
                Methods::NoneEnabled
            );
        }
        assert_eq!(
            policy(false, false, None).methods(base),
            Methods::NoneEnabled
        );
        assert!(policy(true, false, None).enrollable());
        assert!(!policy(false, true, None).enrollable());
    }
}
