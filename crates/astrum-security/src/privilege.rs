use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::permission::Permission;

pub const DOMAIN: &str = "astrum";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    Read,
    Browse,
    Edit,
    Add,
    Delete,
    Run,
    Associate,
    Disassociate,
    Start,
    Stop,
    All,
}

impl Action {
    pub const BREAD: [Action; 5] = [
        Action::Browse,
        Action::Read,
        Action::Edit,
        Action::Add,
        Action::Delete,
    ];

    pub fn application_name(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Browse => "browse",
            Action::Edit => "update",
            Action::Add => "create",
            Action::Delete => "delete",
            Action::Run => "run",
            Action::Associate => "associate",
            Action::Disassociate => "disassociate",
            Action::Start => "start",
            Action::Stop => "stop",
            Action::All => "*",
        }
    }

    pub fn repository_name(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Browse => "browse",
            Action::Edit => "edit",
            Action::Add => "add",
            Action::Delete => "delete",
            Action::Run => "run",
            Action::Associate => "associate",
            Action::Disassociate => "disassociate",
            Action::Start => "start",
            Action::Stop => "stop",
            Action::All => "*",
        }
    }

    pub const ALL_ACTIONS: [Action; 11] = [
        Action::Read,
        Action::Browse,
        Action::Edit,
        Action::Add,
        Action::Delete,
        Action::Run,
        Action::Associate,
        Action::Disassociate,
        Action::Start,
        Action::Stop,
        Action::All,
    ];

    pub const APPLICATION: [Action; 9] = [
        Action::Read,
        Action::Edit,
        Action::Add,
        Action::Delete,
        Action::Associate,
        Action::Disassociate,
        Action::Start,
        Action::Stop,
        Action::All,
    ];

    pub fn is_bread(self) -> bool {
        Self::BREAD.contains(&self) || self == Action::All
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Action::Read => "READ",
            Action::Browse => "BROWSE",
            Action::Edit => "EDIT",
            Action::Add => "ADD",
            Action::Delete => "DELETE",
            Action::Run => "RUN",
            Action::Associate => "ASSOCIATE",
            Action::Disassociate => "DISASSOCIATE",
            Action::Start => "START",
            Action::Stop => "STOP",
            Action::All => "ALL",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL_ACTIONS
            .into_iter()
            .find(|action| action.as_str().eq_ignore_ascii_case(value.trim()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum PrivilegeKind {
    Application {
        domain: String,
        actions: Vec<Action>,
    },
    Wildcard {
        pattern: String,
    },
    RepositoryView {
        format: String,
        repository: String,
        actions: Vec<Action>,
    },
    RepositoryAdmin {
        format: String,
        repository: String,
        actions: Vec<Action>,
    },
    RepositoryContentSelector {
        content_selector: String,
        #[serde(default = "any")]
        format: String,
        repository: String,
        actions: Vec<Action>,
    },
}

fn any() -> String {
    "*".to_owned()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorGrant {
    pub selector: String,
    pub format: String,
    pub repository: String,
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Privilege {
    pub id: String,
    pub description: String,
    pub read_only: bool,
    pub kind: PrivilegeKind,
}

#[derive(Debug, thiserror::Error)]
#[error("the stored privilege \"{id}\" cannot be read: {source}")]
pub struct StoredPrivilegeError {
    pub id: String,
    #[source]
    pub source: serde_json::Error,
}

impl PrivilegeKind {
    pub fn type_id(&self) -> &'static str {
        match self {
            PrivilegeKind::Application { .. } => "application",
            PrivilegeKind::Wildcard { .. } => "wildcard",
            PrivilegeKind::RepositoryView { .. } => "repository-view",
            PrivilegeKind::RepositoryAdmin { .. } => "repository-admin",
            PrivilegeKind::RepositoryContentSelector { .. } => "repository-content-selector",
        }
    }

    pub fn actions(&self) -> &[Action] {
        match self {
            PrivilegeKind::Application { actions, .. }
            | PrivilegeKind::RepositoryView { actions, .. }
            | PrivilegeKind::RepositoryAdmin { actions, .. }
            | PrivilegeKind::RepositoryContentSelector { actions, .. } => actions,
            PrivilegeKind::Wildcard { .. } => &[],
        }
    }

    pub fn allowed_actions(type_id: &str) -> &'static [Action] {
        match type_id {
            "application" => &Action::APPLICATION,
            "repository-view" | "repository-admin" | "repository-content-selector" => &[
                Action::Browse,
                Action::Read,
                Action::Edit,
                Action::Add,
                Action::Delete,
                Action::All,
            ],
            _ => &[],
        }
    }

    pub fn effective_permission(&self) -> Option<Permission> {
        match self {
            PrivilegeKind::RepositoryContentSelector {
                content_selector,
                format,
                repository,
                actions,
            } => Permission::parse(&format!(
                "{DOMAIN}:repository-content-selector:{content_selector}:{format}:{repository}:{}",
                join(actions, Action::repository_name)?
            ))
            .ok(),
            other => other.permission(),
        }
    }

    pub fn permission(&self) -> Option<Permission> {
        let text = match self {
            PrivilegeKind::Application { domain, actions } => {
                format!(
                    "{DOMAIN}:{domain}:{}",
                    join(actions, Action::application_name)?
                )
            }
            PrivilegeKind::Wildcard { pattern } => pattern.clone(),
            PrivilegeKind::RepositoryView {
                format,
                repository,
                actions,
            } => format!(
                "{DOMAIN}:repository-view:{format}:{repository}:{}",
                join(actions, Action::repository_name)?
            ),
            PrivilegeKind::RepositoryAdmin {
                format,
                repository,
                actions,
            } => format!(
                "{DOMAIN}:repository-admin:{format}:{repository}:{}",
                join(actions, Action::repository_name)?
            ),
            PrivilegeKind::RepositoryContentSelector { .. } => return None,
        };
        Permission::parse(&text).ok()
    }

    pub fn selector_grant(&self) -> Option<SelectorGrant> {
        match self {
            PrivilegeKind::RepositoryContentSelector {
                content_selector,
                format,
                repository,
                actions,
            } => Some(SelectorGrant {
                selector: content_selector.clone(),
                format: format.clone(),
                repository: repository.clone(),
                actions: actions.clone(),
            }),
            _ => None,
        }
    }

    pub fn properties(&self) -> Value {
        let mut value = serde_json::to_value(self).unwrap_or(Value::Null);
        if let Value::Object(map) = &mut value {
            map.remove("type");
        }
        value
    }

    pub fn from_stored(
        id: &str,
        type_id: &str,
        properties: &Value,
    ) -> Result<Self, StoredPrivilegeError> {
        let mut value = match properties {
            Value::Object(map) => Value::Object(map.clone()),
            _ => Value::Object(Default::default()),
        };
        if let Value::Object(map) = &mut value {
            map.insert("type".to_owned(), Value::String(type_id.to_owned()));
        }
        serde_json::from_value(value).map_err(|source| StoredPrivilegeError {
            id: id.to_owned(),
            source,
        })
    }
}

impl Privilege {
    pub fn permission(&self) -> Option<Permission> {
        self.kind.permission()
    }
}

fn join(actions: &[Action], name: fn(Action) -> &'static str) -> Option<String> {
    if actions.is_empty() {
        return None;
    }
    if actions.contains(&Action::All) {
        return Some("*".to_owned());
    }
    let mut names: Vec<&str> = actions.iter().map(|action| name(*action)).collect();
    names.sort_unstable();
    names.dedup();
    Some(names.join(","))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn application_privileges_become_domain_permissions() {
        let kind = PrivilegeKind::Application {
            domain: "users".to_owned(),
            actions: vec![Action::Add, Action::Read],
        };
        assert_eq!(
            kind.permission().unwrap().to_string(),
            "astrum:users:create,read"
        );
        let all = PrivilegeKind::Application {
            domain: "users".to_owned(),
            actions: vec![Action::Read, Action::All],
        };
        assert_eq!(all.permission().unwrap().to_string(), "astrum:users:*");
    }

    #[test]
    fn repository_privileges_keep_the_bread_names() {
        let view = PrivilegeKind::RepositoryView {
            format: "maven2".to_owned(),
            repository: "*".to_owned(),
            actions: vec![Action::Edit, Action::Browse],
        };
        assert_eq!(
            view.permission().unwrap().to_string(),
            "astrum:repository-view:maven2:*:browse,edit"
        );
        let admin = PrivilegeKind::RepositoryAdmin {
            format: "npm".to_owned(),
            repository: "npm-proxy".to_owned(),
            actions: vec![Action::All],
        };
        assert_eq!(
            admin.permission().unwrap().to_string(),
            "astrum:repository-admin:npm:npm-proxy:*"
        );
    }

    #[test]
    fn privileges_without_actions_grant_nothing() {
        let kind = PrivilegeKind::Application {
            domain: "users".to_owned(),
            actions: vec![],
        };
        assert!(kind.permission().is_none());
        let broken = PrivilegeKind::Wildcard {
            pattern: "a::b".to_owned(),
        };
        assert!(broken.permission().is_none());
    }

    #[test]
    fn content_selector_privileges_are_grants_not_permissions() {
        let kind = PrivilegeKind::RepositoryContentSelector {
            content_selector: "team-a".to_owned(),
            format: "*".to_owned(),
            repository: "*".to_owned(),
            actions: vec![Action::Read],
        };
        assert!(kind.permission().is_none());
        assert_eq!(kind.selector_grant().unwrap().selector, "team-a");
        assert_eq!(
            kind.effective_permission().unwrap().to_string(),
            "astrum:repository-content-selector:team-a:*:*:read"
        );
    }

    #[test]
    fn actions_parse_without_regard_to_case() {
        assert_eq!(Action::parse("read"), Some(Action::Read));
        assert_eq!(Action::parse(" Disassociate "), Some(Action::Disassociate));
        assert_eq!(Action::parse("CREATE"), None);
        for action in Action::ALL_ACTIONS {
            assert_eq!(Action::parse(action.as_str()), Some(action));
            assert_eq!(
                serde_json::to_value(action).unwrap(),
                Value::String(action.as_str().to_owned())
            );
        }
    }

    #[test]
    fn each_type_has_its_actions() {
        let application = PrivilegeKind::allowed_actions("application");
        assert!(application.contains(&Action::Start));
        assert!(!application.contains(&Action::Browse));
        assert!(!application.contains(&Action::Run));
        let view = PrivilegeKind::allowed_actions("repository-view");
        assert!(view.contains(&Action::Browse));
        assert!(!view.contains(&Action::Start));
        assert!(PrivilegeKind::allowed_actions("wildcard").is_empty());
    }

    #[test]
    fn stored_form_round_trips() {
        let kind = PrivilegeKind::RepositoryContentSelector {
            content_selector: "team-a".to_owned(),
            format: "maven2".to_owned(),
            repository: "maven-releases".to_owned(),
            actions: vec![Action::Read, Action::Browse],
        };
        let properties = kind.properties();
        assert_eq!(
            properties,
            json!({"contentSelector": "team-a", "format": "maven2", "repository": "maven-releases", "actions": ["READ", "BROWSE"]})
        );
        assert_eq!(
            PrivilegeKind::from_stored("p", kind.type_id(), &properties).unwrap(),
            kind
        );
        let without_format = PrivilegeKind::from_stored(
            "p",
            "repository-content-selector",
            &json!({"contentSelector": "team-a", "repository": "*", "actions": ["READ"]}),
        )
        .unwrap();
        assert_eq!(without_format.selector_grant().unwrap().format, "*");
        assert!(PrivilegeKind::from_stored("p", "script", &json!({})).is_err());
    }
}
