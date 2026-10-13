use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Published,
    Updated,
    Deleted,
}

impl Action {
    pub const ALL: [Action; 3] = [Action::Published, Action::Updated, Action::Deleted];

    pub fn id(self) -> &'static str {
        match self {
            Action::Published => "published",
            Action::Updated => "updated",
            Action::Deleted => "deleted",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == value)
    }
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
}

impl Notice {
    fn key(&self) -> (String, String, String, String) {
        (
            self.repository.clone(),
            self.namespace.clone(),
            self.name.clone(),
            self.version.clone(),
        )
    }

    pub fn merge(&self, later: &Notice) -> Option<Notice> {
        let action = match self.action {
            Action::Published if later.action == Action::Deleted => return None,
            Action::Published => Action::Published,
            Action::Updated => later.action,
            Action::Deleted if later.action == Action::Deleted => Action::Deleted,
            Action::Deleted => Action::Updated,
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
    let mut order: Vec<(String, String, String, String)> = Vec::new();
    let mut pending: HashMap<(String, String, String, String), Notice> = HashMap::new();
    let mut overflow: std::collections::HashSet<(String, String, String, String)> =
        Default::default();
    for change in changes {
        let key = change.key();
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
}
