use std::collections::{BTreeMap, BTreeSet, VecDeque};

use astrum_db::entity::{
    repository, security_privilege, security_role, security_user, security_user_role,
};
use astrum_db::setting;
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::Error;
use crate::catalogue::{self, DEFAULT_SOURCE};
use crate::permission::Permission;
use crate::privilege::{Privilege, PrivilegeKind, SelectorGrant};
use crate::realm;
use crate::subject::Subject;

pub const REALMS_KEY: &str = "security.realms";
pub const ANONYMOUS_KEY: &str = "security.anonymous";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Locked,
    Disabled,
    Changepassword,
}

impl UserStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Locked => "locked",
            UserStatus::Disabled => "disabled",
            UserStatus::Changepassword => "changepassword",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        [
            UserStatus::Active,
            UserStatus::Locked,
            UserStatus::Disabled,
            UserStatus::Changepassword,
        ]
        .into_iter()
        .find(|status| status.as_str() == value)
    }

    pub fn can_sign_in(self) -> bool {
        matches!(self, UserStatus::Active | UserStatus::Changepassword)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub status: UserStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Role {
    pub id: String,
    pub source: String,
    pub name: String,
    pub description: String,
    pub privileges: Vec<String>,
    pub roles: Vec<String>,
    pub read_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymousSettings {
    pub enabled: bool,
    pub user_id: String,
    pub realm_name: String,
}

impl Default for AnonymousSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            user_id: catalogue::ANONYMOUS_USER.to_owned(),
            realm_name: realm::AUTHORIZING.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SecurityModel {
    pub users: BTreeMap<String, User>,
    pub roles: BTreeMap<String, Role>,
    pub privileges: BTreeMap<String, Privilege>,
    pub user_roles: BTreeMap<(String, String), Vec<String>>,
    pub anonymous: AnonymousSettings,
    pub realms: Vec<String>,
    pub session_timeout_minutes: u32,
}

impl SecurityModel {
    pub async fn load(db: &impl ConnectionTrait) -> Result<Self, Error> {
        let mut model = SecurityModel::default();

        for row in security_user::Entity::find().all(db).await? {
            let status = UserStatus::parse(&row.status).unwrap_or(UserStatus::Disabled);
            model.users.insert(
                row.id.clone(),
                User {
                    id: row.id,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    email: row.email,
                    password_hash: row.password_hash,
                    status,
                },
            );
        }

        for row in security_role::Entity::find().all(db).await? {
            model.roles.insert(
                row.id.clone(),
                Role {
                    id: row.id,
                    source: row.source,
                    name: row.name,
                    description: row.description,
                    privileges: strings(&row.privileges),
                    roles: strings(&row.roles),
                    read_only: row.read_only,
                },
            );
        }

        let formats = astrum_repo::FORMATS.iter().map(|format| format.id);
        for privilege in catalogue::builtin_privileges(formats) {
            model.privileges.insert(privilege.id.clone(), privilege);
        }
        let repositories: Vec<(String, String)> = repository::Entity::find()
            .select_only()
            .column(repository::Column::Name)
            .column(repository::Column::Format)
            .into_tuple()
            .all(db)
            .await?;
        for (name, format) in repositories {
            for privilege in catalogue::repository_privileges(&format, &name) {
                model.privileges.insert(privilege.id.clone(), privilege);
            }
        }
        for row in security_privilege::Entity::find().all(db).await? {
            match PrivilegeKind::from_stored(&row.id, &row.kind, &row.properties) {
                Ok(kind) => {
                    model.privileges.insert(
                        row.id.clone(),
                        Privilege {
                            id: row.id,
                            description: row.description,
                            read_only: row.read_only,
                            kind,
                        },
                    );
                }
                Err(error) => tracing::warn!(%error, "a stored privilege is ignored"),
            }
        }

        for row in security_user_role::Entity::find().all(db).await? {
            model
                .user_roles
                .insert((row.user_id, row.source), strings(&row.roles));
        }

        model.anonymous = setting::get(db, ANONYMOUS_KEY).await?.unwrap_or_default();
        model.realms = setting::get(db, REALMS_KEY).await?.unwrap_or_else(|| {
            realm::DEFAULT_ACTIVE
                .iter()
                .map(|id| (*id).to_owned())
                .collect()
        });
        model.session_timeout_minutes = setting::get(db, crate::sessions::TIMEOUT_KEY)
            .await?
            .unwrap_or(0);
        Ok(model)
    }

    pub fn realm_active(&self, id: &str) -> bool {
        self.realms.iter().any(|realm| realm == id)
    }

    pub fn roles_of(&self, user_id: &str, source: &str) -> &[String] {
        self.user_roles
            .get(&(user_id.to_owned(), source.to_owned()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn expand_roles<'a>(
        &self,
        direct: impl IntoIterator<Item = &'a String>,
    ) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue: VecDeque<String> = direct.into_iter().cloned().collect();
        while let Some(id) = queue.pop_front() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(role) = self.roles.get(&id) {
                queue.extend(role.roles.iter().cloned());
            }
        }
        seen
    }

    pub fn grants(&self, roles: &BTreeSet<String>) -> (Vec<Permission>, Vec<SelectorGrant>) {
        let mut ids = BTreeSet::new();
        for role in roles.iter().filter_map(|id| self.roles.get(id)) {
            ids.extend(role.privileges.iter());
        }
        let mut permissions = Vec::new();
        let mut selectors = Vec::new();
        for privilege in ids.into_iter().filter_map(|id| self.privileges.get(id)) {
            if let Some(permission) = privilege.permission() {
                permissions.push(permission);
            }
            if let Some(grant) = privilege.kind.selector_grant() {
                selectors.push(grant);
            }
        }
        (permissions, selectors)
    }

    pub fn subject_with_roles(
        &self,
        user_id: &str,
        source: &str,
        realm: &str,
        direct_roles: &[String],
    ) -> Subject {
        let roles = self.expand_roles(direct_roles);
        let (permissions, selectors) = self.grants(&roles);
        Subject::new(user_id, source, realm, roles, permissions, selectors)
    }

    pub fn subject(&self, user_id: &str, source: &str, realm: &str) -> Subject {
        let direct = self.roles_of(user_id, source).to_vec();
        self.subject_with_roles(user_id, source, realm, &direct)
    }

    pub fn local_subject(&self, user_id: &str, realm: &str) -> Option<Subject> {
        let user = self.users.get(user_id)?;
        if !user.status.can_sign_in() {
            return None;
        }
        Some(self.subject(&user.id, DEFAULT_SOURCE, realm))
    }

    pub fn anonymous_subject(&self) -> Option<Subject> {
        if !self.anonymous.enabled {
            return None;
        }
        let user = self.users.get(&self.anonymous.user_id)?;
        if !user.status.can_sign_in() {
            return None;
        }
        Some(
            self.subject(&user.id, DEFAULT_SOURCE, &self.anonymous.realm_name)
                .as_anonymous(),
        )
    }
}

pub fn strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(id: &str, privileges: &[&str], roles: &[&str]) -> Role {
        Role {
            id: id.to_owned(),
            source: DEFAULT_SOURCE.to_owned(),
            name: id.to_owned(),
            description: String::new(),
            privileges: privileges.iter().map(|p| (*p).to_owned()).collect(),
            roles: roles.iter().map(|r| (*r).to_owned()).collect(),
            read_only: false,
        }
    }

    fn model() -> SecurityModel {
        let mut model = SecurityModel::default();
        for privilege in catalogue::builtin_privileges(["maven2"]) {
            model.privileges.insert(privilege.id.clone(), privilege);
        }
        for role in [
            role("readers", &["as-repository-view-maven2-*-read"], &[]),
            role("devs", &["as-users-read"], &["readers", "missing"]),
            role("loop-a", &[], &["loop-b"]),
            role("loop-b", &["as-roles-read"], &["loop-a"]),
        ] {
            model.roles.insert(role.id.clone(), role);
        }
        model.user_roles.insert(
            ("jane".to_owned(), DEFAULT_SOURCE.to_owned()),
            vec!["devs".to_owned(), "loop-a".to_owned()],
        );
        model.users.insert(
            "jane".to_owned(),
            User {
                id: "jane".to_owned(),
                first_name: "Jane".to_owned(),
                last_name: "Doe".to_owned(),
                email: "jane@example.org".to_owned(),
                password_hash: None,
                status: UserStatus::Active,
            },
        );
        model
    }

    #[test]
    fn roles_nest_and_cycles_end() {
        let model = model();
        let subject = model.subject("jane", DEFAULT_SOURCE, realm::LOCAL);
        let roles: Vec<&str> = subject.roles.iter().map(String::as_str).collect();
        assert_eq!(roles, ["devs", "loop-a", "loop-b", "missing", "readers"]);
        assert!(subject.is_permitted("astrum:users:read"));
        assert!(subject.is_permitted("astrum:roles:read"));
        assert!(subject.is_permitted("astrum:repository-view:maven2:central:read"));
        assert!(!subject.is_permitted("astrum:repository-view:npm:npmjs:read"));
        assert!(!subject.is_permitted("astrum:users:update"));
    }

    #[test]
    fn disabled_users_have_no_subject() {
        let mut model = model();
        assert!(model.local_subject("jane", realm::LOCAL).is_some());
        model.users.get_mut("jane").unwrap().status = UserStatus::Disabled;
        assert!(model.local_subject("jane", realm::LOCAL).is_none());
        assert!(model.local_subject("nobody", realm::LOCAL).is_none());
    }

    #[test]
    fn anonymous_access_follows_its_settings() {
        let mut model = model();
        model.anonymous.user_id = "jane".to_owned();
        assert!(model.anonymous_subject().is_none());
        model.anonymous.enabled = true;
        let anonymous = model.anonymous_subject().unwrap();
        assert!(anonymous.anonymous);
        assert!(anonymous.is_permitted("astrum:users:read"));
    }

    #[test]
    fn statuses_round_trip() {
        for status in ["active", "locked", "disabled", "changepassword"] {
            assert_eq!(UserStatus::parse(status).unwrap().as_str(), status);
        }
        assert!(UserStatus::parse("Active").is_none());
    }
}
