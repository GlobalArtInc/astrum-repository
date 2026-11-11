use serde::{Deserialize, Deserializer, Serialize};

use crate::config::problems::Problems;
use crate::config::{Check, Context};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WritePolicy {
    Allow,
    AllowOnce,
    Deny,
}

impl WritePolicy {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "ALLOW" => Some(Self::Allow),
            "ALLOW_ONCE" => Some(Self::AllowOnce),
            "DENY" => Some(Self::Deny),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for WritePolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "\"{value}\" is not a write policy: use ALLOW, ALLOW_ONCE or DENY"
            ))
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_store_name: Option<String>,
    pub strict_content_type_validation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_policy: Option<WritePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_policy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<Quota>,
}

pub const DEFAULT_WARN_PERCENT: i32 = 90;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quota {
    pub limit_bytes: Option<i64>,
    pub warn_percent: Option<i32>,
    #[serde(default)]
    pub enforce: bool,
}

impl Quota {
    pub fn hard_limit(&self) -> Option<i64> {
        match self.enforce {
            true => self.limit_bytes,
            false => None,
        }
    }

    fn check(&mut self, kind: crate::RepositoryType, problems: &mut Problems) {
        let hosted = matches!(kind, crate::RepositoryType::Hosted);
        if matches!(kind, crate::RepositoryType::Group) {
            problems.add("quota", "a group repository has no quota of its own");
            return;
        }
        match self.limit_bytes {
            None => problems.add("quota.limitBytes", "must not be null"),
            Some(limit) if limit <= 0 => {
                problems.add("quota.limitBytes", "must be greater than 0");
            }
            Some(_) => {}
        }
        problems.range("quota.warnPercent", self.warn_percent, 1, 100);
        self.warn_percent.get_or_insert(DEFAULT_WARN_PERCENT);
        if self.enforce && !hosted {
            problems.add(
                "quota.enforce",
                "a hard limit is only allowed for hosted repositories",
            );
        }
    }
}

impl Check for Storage {
    fn check(&mut self, context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        if let Some(quota) = &mut self.quota {
            quota.check(context.spec.kind, problems);
        }
        problems.require_text("blobStoreName", &self.blob_store_name);
        problems.require(
            "strictContentTypeValidation",
            &self.strict_content_type_validation,
        );
        match context.spec.kind {
            crate::RepositoryType::Hosted => problems.require("writePolicy", &self.write_policy),
            _ => self.write_policy = context.spec.fixed_write_policy(),
        }
        if !context.spec.has_latest_policy() {
            self.latest_policy = None;
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cleanup {
    pub policy_names: Option<Vec<String>>,
}

impl Check for Cleanup {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        let names = self.policy_names.get_or_insert_with(Vec::new);
        names.retain(|name| !name.trim().is_empty());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub proprietary_components: Option<bool>,
}

impl Check for Component {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.proprietary_components.get_or_insert(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Spec;

    fn checked(format: &str, kind: &str, mut storage: Storage) -> (Storage, Vec<String>) {
        let spec = Spec::from_api(format, kind).unwrap();
        let context = Context::new(spec, &|plain| plain.to_owned());
        let mut problems = Problems::default();
        storage.check(&context, None, &mut problems);
        let ids = problems
            .finish()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|problem| problem.id)
            .collect();
        (storage, ids)
    }

    #[test]
    fn write_policies_read_in_any_case() {
        let parsed: Storage =
            serde_json::from_str(r#"{"blobStoreName":"default","writePolicy":"allow_once"}"#)
                .unwrap();
        assert_eq!(parsed.write_policy, Some(WritePolicy::AllowOnce));
        assert_eq!(
            serde_json::to_value(WritePolicy::AllowOnce).unwrap(),
            "ALLOW_ONCE"
        );
        let error = serde_json::from_str::<Storage>(r#"{"writePolicy":"WHATEVER"}"#).unwrap_err();
        assert!(error.to_string().contains("ALLOW_ONCE"), "{error}");
    }

    #[test]
    fn hosted_storage_needs_a_write_policy() {
        let (_, ids) = checked("raw", "hosted", Storage::default());
        assert_eq!(
            ids,
            [
                "PARAMETER blobStoreName",
                "PARAMETER strictContentTypeValidation",
                "PARAMETER writePolicy"
            ]
        );
    }

    #[test]
    fn proxies_and_groups_get_their_fixed_write_policy() {
        let given = Storage {
            blob_store_name: Some("default".to_owned()),
            strict_content_type_validation: Some(true),
            write_policy: Some(WritePolicy::Deny),
            latest_policy: Some(true),
            quota: None,
        };
        let (maven, ids) = checked("maven", "proxy", given.clone());
        assert!(ids.is_empty());
        assert_eq!(maven.write_policy, Some(WritePolicy::Allow));
        assert_eq!(maven.latest_policy, None);
        let (pypi, _) = checked("pypi", "proxy", given.clone());
        assert_eq!(pypi.write_policy, None);
        let (raw, _) = checked("raw", "group", given.clone());
        assert_eq!(raw.write_policy, Some(WritePolicy::Allow));
        let (docker, _) = checked("docker", "hosted", given);
        assert_eq!(docker.write_policy, Some(WritePolicy::Deny));
        assert_eq!(docker.latest_policy, Some(true));
    }

    #[test]
    fn only_docker_hosted_keeps_a_latest_policy() {
        let given = Storage {
            blob_store_name: Some("default".to_owned()),
            strict_content_type_validation: Some(false),
            write_policy: Some(WritePolicy::Allow),
            latest_policy: Some(true),
            quota: None,
        };
        assert_eq!(
            checked("docker", "hosted", given.clone()).0.latest_policy,
            Some(true)
        );
        assert_eq!(checked("raw", "hosted", given).0.latest_policy, None);
    }

    #[test]
    fn quotas_need_a_positive_limit_and_a_hard_limit_only_on_hosted() {
        let with = |quota: Quota| Storage {
            blob_store_name: Some("default".to_owned()),
            strict_content_type_validation: Some(true),
            write_policy: Some(WritePolicy::Allow),
            latest_policy: None,
            quota: Some(quota),
        };
        let quota = |limit, warn, enforce| Quota {
            limit_bytes: limit,
            warn_percent: warn,
            enforce,
        };
        let (hosted, ids) = checked("raw", "hosted", with(quota(Some(1), None, true)));
        assert!(ids.is_empty());
        assert_eq!(
            hosted.quota.unwrap().warn_percent,
            Some(DEFAULT_WARN_PERCENT)
        );
        for (given, expected) in [
            (quota(Some(0), None, false), "PARAMETER quota.limitBytes"),
            (quota(None, None, false), "PARAMETER quota.limitBytes"),
            (
                quota(Some(1), Some(0), false),
                "PARAMETER quota.warnPercent",
            ),
            (
                quota(Some(1), Some(101), false),
                "PARAMETER quota.warnPercent",
            ),
        ] {
            assert_eq!(checked("raw", "hosted", with(given)).1, [expected]);
        }
        assert!(
            checked("raw", "proxy", with(quota(Some(1), Some(100), false)))
                .1
                .is_empty()
        );
        assert_eq!(
            checked("raw", "proxy", with(quota(Some(1), None, true))).1,
            ["PARAMETER quota.enforce"]
        );
    }

    #[test]
    fn a_group_repository_takes_no_quota_at_all() {
        let with = |enforce| Storage {
            blob_store_name: Some("default".to_owned()),
            strict_content_type_validation: Some(true),
            write_policy: Some(WritePolicy::Allow),
            latest_policy: None,
            quota: Some(Quota {
                limit_bytes: Some(1),
                warn_percent: None,
                enforce,
            }),
        };
        for enforce in [true, false] {
            assert_eq!(
                checked("raw", "group", with(enforce)).1,
                ["PARAMETER quota"]
            );
        }
    }
}
