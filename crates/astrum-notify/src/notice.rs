use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Published,
    Updated,
    Deleted,
    #[serde(rename = "vulnerability.found")]
    VulnerabilityFound,
    #[serde(rename = "policy.blocked")]
    PolicyBlocked,
    #[serde(rename = "cooldown.released")]
    CooldownReleased,
}

impl Action {
    pub const ALL: [Action; 6] = [
        Action::Published,
        Action::Updated,
        Action::Deleted,
        Action::VulnerabilityFound,
        Action::PolicyBlocked,
        Action::CooldownReleased,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Action::Published => "published",
            Action::Updated => "updated",
            Action::Deleted => "deleted",
            Action::VulnerabilityFound => "vulnerability.found",
            Action::PolicyBlocked => "policy.blocked",
            Action::CooldownReleased => "cooldown.released",
        }
    }

    pub fn event(self) -> &'static str {
        match self {
            Action::Published => "component.published",
            Action::Updated => "component.updated",
            Action::Deleted => "component.deleted",
            Action::VulnerabilityFound => "vulnerability.found",
            Action::PolicyBlocked => "policy.blocked",
            Action::CooldownReleased => "cooldown.released",
        }
    }

    pub fn is_security(self) -> bool {
        matches!(self, Action::VulnerabilityFound | Action::PolicyBlocked)
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    #[serde(default)]
    pub policy: Option<String>,
    #[serde(default)]
    pub advisory: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub repository: String,
    pub repository_type: String,
    pub format: String,
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub action: Action,
    pub initiator: Option<String>,
    pub component_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
}

type Key = (String, String, String, String, String);

impl Notice {
    fn scope(&self, position: usize) -> String {
        match self.action {
            Action::VulnerabilityFound => format!(
                "{}:{}",
                self.action.id(),
                self.detail
                    .as_ref()
                    .and_then(|detail| detail.advisory.as_deref())
                    .unwrap_or_default()
            ),
            Action::PolicyBlocked => self.action.id().to_owned(),
            Action::CooldownReleased => format!("{}:{position}", self.action.id()),
            _ => String::new(),
        }
    }

    fn key(&self, position: usize) -> Key {
        (
            self.repository.clone(),
            self.namespace.clone(),
            self.name.clone(),
            self.version.clone(),
            self.scope(position),
        )
    }

    pub fn merge(&self, later: &Notice) -> Option<Notice> {
        if self.action == Action::CooldownReleased || later.action == Action::CooldownReleased {
            return None;
        }
        let action = match self.action {
            Action::Published if later.action == Action::Deleted => return None,
            Action::Published => Action::Published,
            Action::Updated => later.action,
            Action::Deleted if later.action == Action::Deleted => Action::Deleted,
            Action::Deleted => Action::Updated,
            Action::VulnerabilityFound => Action::VulnerabilityFound,
            Action::PolicyBlocked => Action::PolicyBlocked,
            Action::CooldownReleased => return None,
        };
        Some(Notice {
            action,
            format: if later.format.is_empty() {
                self.format.clone()
            } else {
                later.format.clone()
            },
            repository_type: if later.repository_type.is_empty() {
                self.repository_type.clone()
            } else {
                later.repository_type.clone()
            },
            initiator: later.initiator.clone().or_else(|| self.initiator.clone()),
            component_id: if later.action == Action::Deleted {
                self.component_id.or(later.component_id)
            } else {
                later.component_id.or(self.component_id)
            },
            ..self.clone()
        })
    }
}

pub struct Batch {
    pub notices: Vec<Notice>,
    pub overflow: usize,
}

pub fn combine(changes: impl IntoIterator<Item = Notice>, capacity: usize) -> Batch {
    let mut order: Vec<Key> = Vec::new();
    let mut pending: HashMap<Key, Notice> = HashMap::new();
    let mut overflow: std::collections::HashSet<Key> = Default::default();
    for (position, change) in changes.into_iter().enumerate() {
        let key = change.key(position);
        if let Some(previous) = pending.get(&key) {
            match previous.merge(&change) {
                Some(merged) => {
                    pending.insert(key, merged);
                }
                None => {
                    pending.remove(&key);
                    order.retain(|known| *known != key);
                }
            }
        } else if pending.len() < capacity {
            order.push(key.clone());
            pending.insert(key, change);
        } else {
            overflow.insert(key);
        }
    }
    Batch {
        notices: order
            .into_iter()
            .filter_map(|key| pending.remove(&key))
            .collect(),
        overflow: overflow.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn notice(name: &str, action: Action) -> Notice {
        Notice {
            repository: "maven-releases".into(),
            repository_type: "hosted".into(),
            format: "maven2".into(),
            namespace: "org.example".into(),
            name: name.into(),
            version: "1.0".into(),
            action,
            initiator: Some("admin".into()),
            component_id: Some(7),
            detail: None,
        }
    }

    #[test]
    fn changes_of_one_component_are_combined() {
        let published = notice("demo", Action::Published);
        let updated = notice("demo", Action::Updated);
        let deleted = notice("demo", Action::Deleted);
        assert_eq!(published.merge(&updated).unwrap().action, Action::Published);
        assert_eq!(published.merge(&deleted), None);
        assert_eq!(deleted.merge(&published).unwrap().action, Action::Updated);
        assert_eq!(deleted.merge(&deleted).unwrap().action, Action::Deleted);
        assert_eq!(updated.merge(&deleted).unwrap().action, Action::Deleted);
    }

    #[test]
    fn a_window_keeps_the_first_order_and_counts_the_overflow() {
        let batch = combine(
            [
                notice("a", Action::Published),
                notice("b", Action::Published),
                notice("a", Action::Updated),
                notice("c", Action::Published),
                notice("d", Action::Published),
                notice("b", Action::Deleted),
                notice("e", Action::Updated),
            ],
            3,
        );
        let names: Vec<_> = batch.notices.iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, ["a", "c", "e"]);
        assert_eq!(batch.overflow, 1);
    }

    #[test]
    fn security_notices_stay_apart_from_component_changes_and_from_each_other() {
        let security = |action: Action, advisory: Option<&str>| Notice {
            detail: Some(Detail {
                policy: Some("strict".into()),
                advisory: advisory.map(str::to_owned),
                severity: None,
            }),
            ..notice("demo", action)
        };
        let batch = combine(
            [
                notice("demo", Action::Published),
                security(Action::VulnerabilityFound, Some("GHSA-1")),
                security(Action::VulnerabilityFound, Some("GHSA-2")),
                security(Action::VulnerabilityFound, Some("GHSA-1")),
                security(Action::PolicyBlocked, None),
                notice("demo", Action::CooldownReleased),
                notice("demo", Action::CooldownReleased),
                notice("demo", Action::Updated),
            ],
            10,
        );
        let actions: Vec<_> = batch.notices.iter().map(|n| n.action).collect();
        assert_eq!(
            actions,
            [
                Action::Published,
                Action::VulnerabilityFound,
                Action::VulnerabilityFound,
                Action::PolicyBlocked,
                Action::CooldownReleased,
                Action::CooldownReleased
            ]
        );
        assert_eq!(
            Action::parse("vulnerability.found"),
            Some(Action::VulnerabilityFound)
        );
        assert_eq!(Action::parse("policy.blocked"), Some(Action::PolicyBlocked));
        assert_eq!(Action::PolicyBlocked.event(), "policy.blocked");
        assert_eq!(
            serde_json::to_string(&Action::VulnerabilityFound).unwrap(),
            "\"vulnerability.found\""
        );
    }
}
