use crate::privilege::{Action, Privilege, PrivilegeKind};

pub const PREFIX: &str = "as-";
pub const ALL: &str = "as-all";
pub const ADMIN_ROLE: &str = "as-admin";
pub const ANONYMOUS_ROLE: &str = "as-anonymous";
pub const ADMIN_USER: &str = "admin";
pub const ANONYMOUS_USER: &str = "anonymous";
pub const DEFAULT_SOURCE: &str = "default";

type Application = (&'static str, &'static str, &'static [Action], &'static str);

const APPLICATION: &[Application] = &[
    (
        "as-accesstokens-all",
        "accesstokens",
        &[Action::All],
        "Every action on access tokens",
    ),
    (
        "as-accesstokens-create",
        "accesstokens",
        &[Action::Add, Action::Read],
        "Create access tokens",
    ),
    (
        "as-accesstokens-delete",
        "accesstokens",
        &[Action::Delete, Action::Read],
        "Delete access tokens",
    ),
    (
        "as-accesstokens-read",
        "accesstokens",
        &[Action::Read],
        "Read access tokens",
    ),
    (
        "as-accesstokens-update",
        "accesstokens",
        &[Action::Edit, Action::Read],
        "Change access tokens",
    ),
    (
        "as-analytics-all",
        "analytics",
        &[Action::All],
        "Every action on analytics",
    ),
    (
        "as-apikey-all",
        "apikey",
        &[Action::All],
        "Every action on API keys",
    ),
    (
        "as-atlas-all",
        "atlas",
        &[Action::All],
        "Every action on support tools",
    ),
    (
        "as-audit-all",
        "audit",
        &[Action::All],
        "Every action on the audit log",
    ),
    (
        "as-audit-read",
        "audit",
        &[Action::Read],
        "Read the audit log",
    ),
    (
        "as-blobstores-all",
        "blobstores",
        &[Action::All],
        "Every action on blob stores",
    ),
    (
        "as-blobstores-create",
        "blobstores",
        &[Action::Add, Action::Read],
        "Create blob stores",
    ),
    (
        "as-blobstores-delete",
        "blobstores",
        &[Action::Delete, Action::Read],
        "Delete blob stores",
    ),
    (
        "as-blobstores-read",
        "blobstores",
        &[Action::Read],
        "Read blob stores",
    ),
    (
        "as-blobstores-update",
        "blobstores",
        &[Action::Edit, Action::Read],
        "Change blob stores",
    ),
    (
        "as-bundles-all",
        "bundles",
        &[Action::All],
        "Every action on bundles",
    ),
    (
        "as-bundles-read",
        "bundles",
        &[Action::Read],
        "Read bundles",
    ),
    (
        "as-capabilities-all",
        "capabilities",
        &[Action::All],
        "Every action on capabilities",
    ),
    (
        "as-capabilities-create",
        "capabilities",
        &[Action::Add, Action::Read],
        "Create capabilities",
    ),
    (
        "as-capabilities-delete",
        "capabilities",
        &[Action::Delete, Action::Read],
        "Delete capabilities",
    ),
    (
        "as-capabilities-read",
        "capabilities",
        &[Action::Read],
        "Read capabilities",
    ),
    (
        "as-capabilities-update",
        "capabilities",
        &[Action::Edit, Action::Read],
        "Change capabilities",
    ),
    (
        "as-component-upload",
        "component",
        &[Action::Add],
        "Upload components",
    ),
    (
        "as-datastores-all",
        "datastores",
        &[Action::All],
        "Every action on data stores",
    ),
    (
        "as-datastores-create",
        "datastores",
        &[Action::Add, Action::Read],
        "Create data stores",
    ),
    (
        "as-datastores-delete",
        "datastores",
        &[Action::Delete, Action::Read],
        "Delete data stores",
    ),
    (
        "as-datastores-read",
        "datastores",
        &[Action::Read],
        "Read data stores",
    ),
    (
        "as-datastores-update",
        "datastores",
        &[Action::Edit, Action::Read],
        "Change data stores",
    ),
    (
        "as-healthcheck-read",
        "healthcheck",
        &[Action::Read],
        "Read the health check",
    ),
    (
        "as-ldap-all",
        "ldap",
        &[Action::All],
        "Every action on LDAP servers",
    ),
    (
        "as-ldap-create",
        "ldap",
        &[Action::Add, Action::Read],
        "Create LDAP servers",
    ),
    (
        "as-ldap-delete",
        "ldap",
        &[Action::Delete, Action::Read],
        "Delete LDAP servers",
    ),
    ("as-ldap-read", "ldap", &[Action::Read], "Read LDAP servers"),
    (
        "as-ldap-update",
        "ldap",
        &[Action::Edit, Action::Read],
        "Change LDAP servers",
    ),
    (
        "as-logging-all",
        "logging",
        &[Action::All],
        "Every action on logging",
    ),
    (
        "as-logging-mark",
        "logging",
        &[Action::Add],
        "Add marks to the log",
    ),
    (
        "as-logging-read",
        "logging",
        &[Action::Read],
        "Read logging",
    ),
    (
        "as-logging-update",
        "logging",
        &[Action::Edit, Action::Read],
        "Change logging",
    ),
    (
        "as-metrics-all",
        "metrics",
        &[Action::All],
        "Every action on metrics",
    ),
    (
        "as-privileges-all",
        "privileges",
        &[Action::All],
        "Every action on privileges",
    ),
    (
        "as-privileges-create",
        "privileges",
        &[Action::Add, Action::Read],
        "Create privileges",
    ),
    (
        "as-privileges-delete",
        "privileges",
        &[Action::Delete, Action::Read],
        "Delete privileges",
    ),
    (
        "as-privileges-read",
        "privileges",
        &[Action::Read],
        "Read privileges",
    ),
    (
        "as-privileges-update",
        "privileges",
        &[Action::Edit, Action::Read],
        "Change privileges",
    ),
    (
        "as-roles-all",
        "roles",
        &[Action::All],
        "Every action on roles",
    ),
    (
        "as-roles-create",
        "roles",
        &[Action::Add, Action::Read],
        "Create roles",
    ),
    (
        "as-roles-delete",
        "roles",
        &[Action::Delete, Action::Read],
        "Delete roles",
    ),
    ("as-roles-read", "roles", &[Action::Read], "Read roles"),
    (
        "as-roles-update",
        "roles",
        &[Action::Edit, Action::Read],
        "Change roles",
    ),
    ("as-search-read", "search", &[Action::Read], "Use search"),
    (
        "as-selectors-all",
        "selectors",
        &[Action::All],
        "Every action on content selectors",
    ),
    (
        "as-selectors-create",
        "selectors",
        &[Action::Add, Action::Read],
        "Create content selectors",
    ),
    (
        "as-selectors-delete",
        "selectors",
        &[Action::Delete, Action::Read],
        "Delete content selectors",
    ),
    (
        "as-selectors-read",
        "selectors",
        &[Action::Read],
        "Read content selectors",
    ),
    (
        "as-selectors-update",
        "selectors",
        &[Action::Edit, Action::Read],
        "Change content selectors",
    ),
    (
        "as-settings-all",
        "settings",
        &[Action::All],
        "Every action on settings",
    ),
    (
        "as-settings-read",
        "settings",
        &[Action::Read],
        "Read settings",
    ),
    (
        "as-settings-update",
        "settings",
        &[Action::Edit, Action::Read],
        "Change settings",
    ),
    (
        "as-ssl-truststore-all",
        "ssl-truststore",
        &[Action::All],
        "Every action on the SSL truststore",
    ),
    (
        "as-ssl-truststore-create",
        "ssl-truststore",
        &[Action::Add, Action::Read],
        "Add certificates to the SSL truststore",
    ),
    (
        "as-ssl-truststore-delete",
        "ssl-truststore",
        &[Action::Delete, Action::Read],
        "Remove certificates from the SSL truststore",
    ),
    (
        "as-ssl-truststore-read",
        "ssl-truststore",
        &[Action::Read],
        "Read the SSL truststore",
    ),
    (
        "as-ssl-truststore-update",
        "ssl-truststore",
        &[Action::Edit, Action::Read],
        "Change the SSL truststore",
    ),
    (
        "as-tasks-all",
        "tasks",
        &[Action::All],
        "Every action on tasks",
    ),
    (
        "as-tasks-create",
        "tasks",
        &[Action::Add, Action::Read],
        "Create tasks",
    ),
    (
        "as-tasks-delete",
        "tasks",
        &[Action::Delete, Action::Read],
        "Delete tasks",
    ),
    ("as-tasks-read", "tasks", &[Action::Read], "Read tasks"),
    (
        "as-tasks-run",
        "tasks",
        &[Action::Start, Action::Stop],
        "Start and stop tasks",
    ),
    (
        "as-tasks-update",
        "tasks",
        &[Action::Edit, Action::Read],
        "Change tasks",
    ),
    (
        "as-uploader-metadata-read",
        "uploader-metadata",
        &[Action::Read],
        "Read uploader metadata",
    ),
    (
        "as-users-all",
        "users",
        &[Action::All],
        "Every action on users",
    ),
    (
        "as-users-create",
        "users",
        &[Action::Add, Action::Read],
        "Create users",
    ),
    (
        "as-users-delete",
        "users",
        &[Action::Delete, Action::Read],
        "Delete users",
    ),
    ("as-users-read", "users", &[Action::Read], "Read users"),
    (
        "as-users-update",
        "users",
        &[Action::Edit, Action::Read],
        "Change users",
    ),
    (
        "as-userschangepw",
        "userschangepw",
        &[Action::Add, Action::Read],
        "Change user passwords",
    ),
    (
        "as-wonderland-all",
        "wonderland",
        &[Action::All],
        "Every action on support downloads",
    ),
];

const VIEW_ACTIONS: [(Action, &str); 6] = [
    (Action::All, "*"),
    (Action::Add, "add"),
    (Action::Browse, "browse"),
    (Action::Delete, "delete"),
    (Action::Edit, "edit"),
    (Action::Read, "read"),
];

const REPOSITORY_ADMIN_ACTIONS: [(Action, &str); 5] = [
    (Action::All, "*"),
    (Action::Browse, "browse"),
    (Action::Delete, "delete"),
    (Action::Edit, "edit"),
    (Action::Read, "read"),
];

pub struct BuiltinRole {
    pub id: &'static str,
    pub description: &'static str,
    pub privileges: &'static [&'static str],
}

pub const ROLES: [BuiltinRole; 2] = [
    BuiltinRole {
        id: ADMIN_ROLE,
        description: "Administrator role",
        privileges: &[ALL],
    },
    BuiltinRole {
        id: ANONYMOUS_ROLE,
        description: "Anonymous role",
        privileges: &[
            "as-healthcheck-read",
            "as-repository-view-*-*-browse",
            "as-repository-view-*-*-read",
            "as-search-read",
        ],
    },
];

pub fn builtin_privileges<'a>(formats: impl IntoIterator<Item = &'a str>) -> Vec<Privilege> {
    let mut privileges = vec![Privilege {
        id: ALL.to_owned(),
        description: "Every permission".to_owned(),
        read_only: true,
        kind: PrivilegeKind::Wildcard {
            pattern: "astrum:*".to_owned(),
        },
    }];
    privileges.extend(
        APPLICATION
            .iter()
            .map(|(id, domain, actions, description)| Privilege {
                id: (*id).to_owned(),
                description: (*description).to_owned(),
                read_only: true,
                kind: PrivilegeKind::Application {
                    domain: (*domain).to_owned(),
                    actions: actions.to_vec(),
                },
            }),
    );
    for (action, name) in VIEW_ACTIONS {
        privileges.push(Privilege {
            id: format!("as-repository-admin-*-*-{name}"),
            description: format!("{} the configuration of every repository", verb(action)),
            read_only: true,
            kind: PrivilegeKind::RepositoryAdmin {
                format: "*".to_owned(),
                repository: "*".to_owned(),
                actions: vec![action],
            },
        });
        privileges.push(view("*", "*", action, name, "every repository".to_owned()));
    }
    for format in formats {
        for (action, name) in VIEW_ACTIONS {
            privileges.push(view(
                format,
                "*",
                action,
                name,
                format!("every {format} repository"),
            ));
        }
    }
    privileges
}

pub fn repository_privileges(format: &str, repository: &str) -> Vec<Privilege> {
    let mut privileges: Vec<Privilege> = VIEW_ACTIONS
        .into_iter()
        .map(|(action, name)| {
            view(
                format,
                repository,
                action,
                name,
                format!("the repository {repository}"),
            )
        })
        .collect();
    privileges.extend(
        REPOSITORY_ADMIN_ACTIONS
            .into_iter()
            .map(|(action, name)| Privilege {
                id: format!("as-repository-admin-{format}-{repository}-{name}"),
                description: format!(
                    "{} the configuration of the repository {repository}",
                    verb(action)
                ),
                read_only: true,
                kind: PrivilegeKind::RepositoryAdmin {
                    format: format.to_owned(),
                    repository: repository.to_owned(),
                    actions: vec![action],
                },
            }),
    );
    privileges
}

fn view(format: &str, repository: &str, action: Action, name: &str, target: String) -> Privilege {
    Privilege {
        id: format!("as-repository-view-{format}-{repository}-{name}"),
        description: format!("{} the content of {target}", verb(action)),
        read_only: true,
        kind: PrivilegeKind::RepositoryView {
            format: format.to_owned(),
            repository: repository.to_owned(),
            actions: vec![action],
        },
    }
}

fn verb(action: Action) -> &'static str {
    match action {
        Action::All => "Do everything with",
        Action::Add => "Add to",
        Action::Browse => "Browse",
        Action::Delete => "Delete from",
        Action::Edit => "Change",
        Action::Read => "Read",
        _ => "Use",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn the_catalogue_has_the_names_existing_installations_use() {
        let formats = ["maven2", "npm"];
        let privileges = builtin_privileges(formats);
        let ids: BTreeSet<&str> = privileges
            .iter()
            .map(|privilege| privilege.id.as_str())
            .collect();
        assert_eq!(ids.len(), privileges.len(), "ids are unique");
        for id in [
            "as-all",
            "as-users-read",
            "as-userschangepw",
            "as-tasks-run",
            "as-healthcheck-read",
            "as-repository-view-*-*-read",
            "as-repository-view-*-*-*",
            "as-repository-admin-*-*-add",
            "as-repository-view-maven2-*-browse",
            "as-repository-view-npm-*-*",
        ] {
            assert!(ids.contains(id), "{id}");
        }
        assert_eq!(privileges.len(), 1 + 77 + 12 + 12);
        assert!(
            privileges
                .iter()
                .all(|privilege| privilege.read_only && privilege.id.starts_with(PREFIX))
        );
        assert!(
            privileges
                .iter()
                .all(|privilege| privilege.permission().is_some()),
            "every built-in grants something"
        );
    }

    #[test]
    fn built_in_roles_name_existing_privileges() {
        let ids: BTreeSet<String> = builtin_privileges(["raw"])
            .into_iter()
            .map(|privilege| privilege.id)
            .collect();
        for role in ROLES {
            for privilege in role.privileges {
                assert!(ids.contains(*privilege), "{} -> {privilege}", role.id);
            }
        }
    }

    #[test]
    fn every_repository_gets_view_and_admin_privileges() {
        let privileges = repository_privileges("maven2", "maven-releases");
        let ids: Vec<&str> = privileges
            .iter()
            .map(|privilege| privilege.id.as_str())
            .collect();
        assert_eq!(ids.len(), 11);
        assert!(ids.contains(&"as-repository-view-maven2-maven-releases-read"));
        assert!(ids.contains(&"as-repository-view-maven2-maven-releases-add"));
        assert!(ids.contains(&"as-repository-admin-maven2-maven-releases-edit"));
        assert!(!ids.contains(&"as-repository-admin-maven2-maven-releases-add"));
        let read = &privileges[5];
        assert_eq!(
            read.permission().unwrap().to_string(),
            "astrum:repository-view:maven2:maven-releases:read"
        );
    }
}
