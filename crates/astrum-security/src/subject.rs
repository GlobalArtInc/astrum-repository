use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::permission::Permission;
use crate::privilege::{Action, SelectorGrant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Scope {
    Read,
    #[default]
    Write,
}

#[derive(Clone, Debug)]
pub struct Subject {
    pub user_id: String,
    pub source: String,
    pub realm: String,
    pub anonymous: bool,
    pub roles: BTreeSet<String>,
    pub scope: Scope,
    permissions: Arc<[Permission]>,
    selectors: Arc<[SelectorGrant]>,
}

impl Subject {
    pub fn new(
        user_id: impl Into<String>,
        source: impl Into<String>,
        realm: impl Into<String>,
        roles: BTreeSet<String>,
        permissions: Vec<Permission>,
        selectors: Vec<SelectorGrant>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            source: source.into(),
            realm: realm.into(),
            anonymous: false,
            roles,
            scope: Scope::Write,
            permissions: permissions.into(),
            selectors: selectors.into(),
        }
    }

    pub fn as_anonymous(mut self) -> Self {
        self.anonymous = true;
        self
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn is_permitted(&self, permission: &str) -> bool {
        match Permission::parse(permission) {
            Ok(permission) => self.permits(&permission),
            Err(_) => false,
        }
    }

    pub fn permits(&self, permission: &Permission) -> bool {
        if self.scope == Scope::Read && !reads_only(permission) {
            return false;
        }
        self.permissions
            .iter()
            .any(|granted| granted.implies(permission))
    }

    pub fn may_add_content(&self) -> bool {
        const ADD: [Option<&str>; 5] = [
            Some("astrum"),
            Some("repository-view"),
            None,
            None,
            Some("add"),
        ];
        self.scope == Scope::Write
            && (self
                .permissions
                .iter()
                .any(|granted| granted.implies_shape(&ADD))
                || self.selectors.iter().any(|grant| {
                    grant
                        .actions
                        .iter()
                        .any(|action| matches!(action, Action::Add | Action::All))
                }))
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn selectors(&self) -> &[SelectorGrant] {
        &self.selectors
    }
}

fn reads_only(permission: &Permission) -> bool {
    permission.last_values().is_some_and(|values| {
        values
            .iter()
            .all(|value| matches!(*value, "read" | "browse"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject(grants: &[&str]) -> Subject {
        Subject::new(
            "jane",
            "default",
            crate::realm::LOCAL,
            BTreeSet::new(),
            grants
                .iter()
                .map(|grant| Permission::parse(grant).unwrap())
                .collect(),
            vec![],
        )
    }

    #[test]
    fn a_subject_holds_what_its_grants_imply() {
        let jane = subject(&["astrum:users:read", "astrum:repository-view:maven2:*:*"]);
        assert!(jane.is_permitted("astrum:users:read"));
        assert!(!jane.is_permitted("astrum:users:delete"));
        assert!(jane.is_permitted("astrum:repository-view:maven2:central:add"));
        assert!(!jane.is_permitted("not a permission::"));
    }

    #[test]
    fn adding_content_needs_an_add_grant_somewhere() {
        for granted in [
            "astrum:repository-view:maven2:central:add",
            "astrum:repository-view:*:*:*",
            "astrum:repository-view:npm:npm-hosted:read,add",
            "astrum:*",
        ] {
            assert!(subject(&[granted]).may_add_content(), "{granted}");
        }
        for granted in [
            "astrum:repository-view:maven2:central:browse,read,edit,delete",
            "astrum:blobstores:*",
            "astrum:users:read",
        ] {
            assert!(!subject(&[granted]).may_add_content(), "{granted}");
        }
        assert!(
            !subject(&["astrum:*"])
                .with_scope(Scope::Read)
                .may_add_content()
        );
        let selected = |actions: Vec<Action>| {
            Subject::new(
                "jane",
                "default",
                crate::realm::LOCAL,
                BTreeSet::new(),
                vec![],
                vec![SelectorGrant {
                    selector: "docs".to_owned(),
                    format: "raw".to_owned(),
                    repository: "*".to_owned(),
                    actions,
                }],
            )
        };
        assert!(selected(vec![Action::Read, Action::Add]).may_add_content());
        assert!(selected(vec![Action::All]).may_add_content());
        assert!(!selected(vec![Action::Read, Action::Browse]).may_add_content());
        assert!(
            !selected(vec![Action::Add])
                .with_scope(Scope::Read)
                .may_add_content()
        );
    }

    #[test]
    fn a_read_scope_only_reads() {
        let token = subject(&["astrum:*"]).with_scope(Scope::Read);
        assert!(token.is_permitted("astrum:repository-view:npm:npm-proxy:read"));
        assert!(token.is_permitted("astrum:repository-view:npm:npm-proxy:browse"));
        assert!(token.is_permitted("astrum:users:read"));
        assert!(!token.is_permitted("astrum:repository-view:npm:npm-hosted:add"));
        assert!(!token.is_permitted("astrum:users:read,update"));
        assert!(!token.is_permitted("astrum:users:*"));
    }
}
