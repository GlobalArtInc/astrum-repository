use serde::{Deserialize, Deserializer, Serialize};

use crate::config::problems::Problems;
use crate::config::{Check, Context};
use crate::tags::TagRules;

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
    pub immutable_tags: Option<TagRules>,
}

impl Check for Storage {
    fn check(&mut self, context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
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
            self.immutable_tags = None;
        }
        if let Some(rules) = &self.immutable_tags {
            rules.check(problems);
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
    use crate::tags::{MAX_ENTRIES, MAX_ENTRY_LENGTH};

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
            immutable_tags: None,
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
            immutable_tags: None,
        };
        assert_eq!(
            checked("docker", "hosted", given.clone()).0.latest_policy,
            Some(true)
        );
        assert_eq!(checked("raw", "hosted", given).0.latest_policy, None);
    }

    fn with_tags(patterns: &[&str], exceptions: &[&str]) -> Storage {
        Storage {
            blob_store_name: Some("default".to_owned()),
            strict_content_type_validation: Some(true),
            write_policy: Some(WritePolicy::AllowOnce),
            latest_policy: None,
            immutable_tags: Some(TagRules {
                patterns: patterns
                    .iter()
                    .map(|pattern| (*pattern).to_owned())
                    .collect(),
                exceptions: exceptions
                    .iter()
                    .map(|pattern| (*pattern).to_owned())
                    .collect(),
            }),
        }
    }

    #[test]
    fn only_docker_hosted_keeps_immutable_tags() {
        let given = with_tags(&["v*"], &[]);
        assert!(
            checked("docker", "hosted", given.clone())
                .0
                .immutable_tags
                .is_some()
        );
        for (format, kind) in [("raw", "hosted"), ("docker", "proxy"), ("docker", "group")] {
            assert_eq!(
                checked(format, kind, given.clone()).0.immutable_tags,
                None,
                "{format} {kind}"
            );
        }
    }

    #[test]
    fn immutable_tag_entries_are_checked_where_they_stand() {
        let ids = |storage: Storage| checked("docker", "hosted", storage).1;
        assert_eq!(
            ids(with_tags(&["v["], &[])),
            ["PARAMETER immutableTags.patterns[0]"]
        );
        assert_eq!(
            ids(with_tags(&[], &["[z-a]"])),
            ["PARAMETER immutableTags.exceptions[0]"]
        );
        let many = vec!["v1"; MAX_ENTRIES + 1];
        assert_eq!(
            ids(with_tags(&many, &[])),
            ["PARAMETER immutableTags.patterns"]
        );
        let long = "v".repeat(MAX_ENTRY_LENGTH + 1);
        assert_eq!(
            ids(with_tags(&[long.as_str()], &[])),
            ["PARAMETER immutableTags.patterns[0]"]
        );
        let longest = "v".repeat(MAX_ENTRY_LENGTH);
        let full = vec!["v1"; MAX_ENTRIES];
        assert!(ids(with_tags(&full, &[])).is_empty());
        assert!(ids(with_tags(&[longest.as_str()], &[])).is_empty());
        assert!(ids(with_tags(&["v*"], &["v*-rc*"])).is_empty());
    }

    #[test]
    fn immutable_tag_rules_refuse_unknown_fields() {
        let error = serde_json::from_str::<Storage>(
            r#"{"immutableTags":{"patterns":["v*"],"extra":true}}"#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("extra"), "{error}");
    }
}
