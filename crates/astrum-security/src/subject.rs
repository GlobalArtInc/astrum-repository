use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::catalogue::DEFAULT_SOURCE;
use crate::permission::Permission;
use crate::privilege::{Action, SelectorGrant};

pub const EXTERNAL_CREDENTIALS: &str = "credentials of this account are managed outside Astrum";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Scope {
    Read,
    #[default]
    Write,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("a token limited to repositories cannot do this")]
pub struct NarrowedSubject;

pub const LIMITED_TOKEN_ID: &str = "limited-token";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicIdentity {
    pub id: String,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenRef {
    pub id: Uuid,
    pub customer: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Subject {
    pub user_id: String,
    pub source: String,
    pub realm: String,
    pub anonymous: bool,
    pub roles: BTreeSet<String>,
    pub scope: Scope,
    pub token: Option<TokenRef>,
    narrowed: bool,
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
            token: None,
            narrowed: false,
            permissions: permissions.into(),
            selectors: selectors.into(),
        }
    }

    pub fn as_anonymous(mut self) -> Self {
        self.anonymous = true;
        self
    }

    pub fn local_user_id(&self) -> Option<&str> {
        (!self.anonymous && self.source == DEFAULT_SOURCE).then_some(self.user_id.as_str())
    }

    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_token(mut self, token: TokenRef) -> Self {
        self.token = Some(token);
        self
    }

    pub fn narrowed_to(&self, repositories: &BTreeSet<String>) -> Subject {
        let mut permissions: Vec<Permission> = Vec::new();
        for permission in self
            .permissions
            .iter()
            .filter_map(|granted| granted.narrow_to(repositories))
        {
            if !permissions.contains(&permission) {
                permissions.push(permission);
            }
        }
        let mut selectors: Vec<SelectorGrant> = Vec::new();
        for grant in self
            .selectors
            .iter()
            .flat_map(|grant| grant.narrow_to(repositories))
        {
            if !selectors.contains(&grant) {
                selectors.push(grant);
            }
        }
        Subject {
            user_id: self.user_id.clone(),
            source: self.source.clone(),
            realm: self.realm.clone(),
            anonymous: self.anonymous,
            roles: BTreeSet::new(),
            scope: Scope::Read,
            token: self.token.clone(),
            narrowed: true,
            permissions: permissions.into(),
            selectors: selectors.into(),
        }
    }

    pub fn narrowed(&self) -> bool {
        self.narrowed
    }

    pub fn public_identity(&self) -> PublicIdentity {
        match self.narrowed {
            true => PublicIdentity {
                id: self
                    .token
                    .as_ref()
                    .and_then(|token| token.customer.clone())
                    .unwrap_or_else(|| LIMITED_TOKEN_ID.to_owned()),
                hidden: true,
            },
            false => PublicIdentity {
                id: self.user_id.clone(),
                hidden: false,
            },
        }
    }

    pub fn require_unnarrowed(&self) -> Result<&Subject, NarrowedSubject> {
        match self.narrowed {
            true => Err(NarrowedSubject),
            false => Ok(self),
        }
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
    fn only_a_signed_in_local_subject_names_a_local_user() {
        let jane = subject(&[]);
        assert_eq!(jane.local_user_id(), Some("jane"));
        assert_eq!(jane.clone().as_anonymous().local_user_id(), None);
        for source in ["LDAP", "OAuth2"] {
            let external = Subject::new(
                "jane",
                source,
                crate::realm::LDAP,
                BTreeSet::new(),
                vec![],
                vec![],
            );
            assert_eq!(external.local_user_id(), None, "{source}");
        }
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

    fn listed(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn a_narrowed_subject_reads_the_listed_repositories_and_nothing_else() {
        let selected = Subject::new(
            "svc",
            "default",
            crate::realm::ACCESS_TOKEN,
            BTreeSet::from(["nx-admin".to_owned()]),
            vec![
                Permission::parse("astrum:*").unwrap(),
                Permission::parse("astrum:repository-view:*:*:*").unwrap(),
            ],
            vec![SelectorGrant {
                selector: "docs".to_owned(),
                format: "*".to_owned(),
                repository: "*".to_owned(),
                actions: vec![Action::Read, Action::Add],
            }],
        );
        let token = selected.narrowed_to(&listed(&["raw-a"]));
        assert!(token.narrowed());
        assert!(token.roles.is_empty());
        assert_eq!(token.scope, Scope::Read);
        assert!(token.is_permitted("astrum:repository-view:raw:raw-a:read"));
        assert!(token.is_permitted("astrum:repository-view:raw:raw-a:browse"));
        assert!(!token.is_permitted("astrum:repository-view:raw:raw-a:add"));
        assert!(!token.is_permitted("astrum:repository-view:raw:raw-b:read"));
        assert!(!token.is_permitted("astrum:users:read"));
        assert!(!token.is_permitted("astrum:settings:read"));
        assert!(!token.may_add_content());
        assert!(token.permissions().iter().all(|permission| {
            permission
                .to_string()
                .starts_with("astrum:repository-view:")
        }));
        assert_eq!(token.selectors().len(), 1);
        assert_eq!(token.selectors()[0].repository, "raw-a");
        assert_eq!(token.selectors()[0].actions, vec![Action::Read]);
        assert!(!selected.narrowed());

        let again = token.narrowed_to(&listed(&["raw-a", "raw-b"]));
        assert!(again.is_permitted("astrum:repository-view:raw:raw-a:read"));
        assert!(!again.is_permitted("astrum:repository-view:raw:raw-b:read"));

        let nothing = selected.narrowed_to(&listed(&[]));
        assert!(nothing.permissions().is_empty() && nothing.selectors().is_empty());
        assert!(!nothing.is_permitted("astrum:repository-view:raw:raw-a:read"));
    }
}
