use std::collections::{BTreeMap, BTreeSet};

use astrum_db::entity::{blob_store, cleanup_policy, repository};
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveValue, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, SqlErr, TransactionTrait,
};

use crate::RepositoryType;
use crate::config::{Problem, Problems, Repository, Spec};

const WRITE_LOCK: i64 = i64::from_be_bytes(*b"astrrepo");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the database refused a query: {0}")]
    Query(#[from] DbErr),
    #[error("the stored repository \"{name}\" cannot be read: {reason}")]
    Stored { name: String, reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("the configuration is not valid")]
    Invalid(Vec<Problem>),
    #[error("{0}")]
    Refused(String),
    #[error("the repository \"{0}\" does not exist")]
    NotFound(String),
    #[error(transparent)]
    Store(#[from] Error),
}

impl From<DbErr> for WriteError {
    fn from(error: DbErr) -> Self {
        WriteError::Store(Error::Query(error))
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Repository>, Error> {
    repository::Entity::find()
        .order_by_asc(repository::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(read)
        .collect()
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<Repository>, Error> {
    repository::Entity::find()
        .filter(
            Expr::expr(Func::lower(Expr::col(repository::Column::Name))).eq(name.to_lowercase()),
        )
        .one(db)
        .await?
        .map(read)
        .transpose()
}

pub async fn docker_on_port(
    db: &impl ConnectionTrait,
    port: u16,
) -> Result<Option<Repository>, Error> {
    repository::Entity::find()
        .filter(Expr::col(repository::Column::Format).eq("docker"))
        .filter(Expr::cust_with_values(
            "(attributes #>> $1::text[] = $3 OR attributes #>> $2::text[] = $3)",
            [
                "{docker,httpPort}".to_owned(),
                "{docker,httpsPort}".to_owned(),
                port.to_string(),
            ],
        ))
        .order_by_asc(repository::Column::Name)
        .one(db)
        .await?
        .map(read)
        .transpose()
}

pub async fn docker_on_subdomain(
    db: &impl ConnectionTrait,
    subdomain: &str,
) -> Result<Option<Repository>, Error> {
    repository::Entity::find()
        .filter(Expr::col(repository::Column::Format).eq("docker"))
        .filter(Expr::cust_with_values(
            "lower(attributes #>> $1::text[]) = $2",
            ["{docker,subdomain}".to_owned(), subdomain.to_lowercase()],
        ))
        .order_by_asc(repository::Column::Name)
        .one(db)
        .await?
        .map(read)
        .transpose()
}

type HostOf = fn(&Repository) -> Option<&str>;

fn subdomain_of(repository: &Repository) -> Option<&str> {
    repository
        .attributes
        .docker
        .as_ref()
        .and_then(|docker| docker.subdomain.as_deref())
}

fn registry_host_of(repository: &Repository) -> Option<&str> {
    repository
        .attributes
        .terraform
        .as_ref()
        .and_then(|terraform| terraform.registry_host.as_deref())
}

fn read(row: repository::Model) -> Result<Repository, Error> {
    let name = row.name.clone();
    Repository::from_stored(
        row.name,
        &row.format,
        &row.kind,
        row.online,
        row.blob_store,
        row.attributes,
    )
    .map_err(|reason| Error::Stored { name, reason })
}

pub struct Writer {
    txn: DatabaseTransaction,
    repositories: Vec<Repository>,
    blob_stores: BTreeSet<String>,
    grouped: BTreeMap<String, String>,
    cleanup_policies: BTreeMap<String, String>,
    routing_rules: BTreeSet<String>,
    reserved_ports: Vec<u16>,
}

impl Writer {
    pub async fn begin(db: &DatabaseConnection) -> Result<Self, Error> {
        let txn = db.begin().await?;
        txn.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({WRITE_LOCK})"))
            .await?;
        let repositories = list(&txn).await?;
        let stores: Vec<(String, String, serde_json::Value)> = blob_store::Entity::find()
            .select_only()
            .column(blob_store::Column::Name)
            .column(blob_store::Column::Kind)
            .column(blob_store::Column::Attributes)
            .into_tuple()
            .all(&txn)
            .await?;
        let mut grouped = BTreeMap::new();
        for (name, kind, attributes) in &stores {
            if kind != "Group" {
                continue;
            }
            for member in attributes["group"]["members"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
            {
                grouped.insert(member.to_owned(), name.clone());
            }
        }
        let blob_stores = stores.into_iter().map(|(name, _, _)| name).collect();
        let cleanup_policies: Vec<(String, String)> = cleanup_policy::Entity::find()
            .select_only()
            .column(cleanup_policy::Column::Name)
            .column(cleanup_policy::Column::Format)
            .into_tuple()
            .all(&txn)
            .await?;
        Ok(Self {
            txn,
            repositories,
            blob_stores,
            grouped,
            cleanup_policies: cleanup_policies.into_iter().collect(),
            routing_rules: BTreeSet::new(),
            reserved_ports: Vec::new(),
        })
    }

    pub fn reserving(mut self, ports: impl IntoIterator<Item = u16>) -> Self {
        self.reserved_ports.extend(ports);
        self
    }

    pub fn find(&self, name: &str) -> Option<&Repository> {
        self.repositories
            .iter()
            .find(|repository| repository.is_named(name))
    }

    pub async fn create(self, repository: Repository) -> Result<Repository, WriteError> {
        let mut problems = Problems::default();
        if self.find(&repository.name).is_some() {
            problems.add(
                "name",
                "is already used by another repository (names ignore case)",
            );
        }
        self.check(&repository, &mut problems);
        if let Some(blob_store) = repository.blob_store()
            && let Some(group) = self.grouped.get(blob_store)
        {
            problems.add(
                "blobStoreName",
                format!(
                    "the blob store \"{blob_store}\" is a member of the group \"{group}\"; use the group"
                ),
            );
        }
        problems.finish().map_err(WriteError::Invalid)?;
        let now = chrono::Utc::now().fixed_offset();
        let row = repository::ActiveModel {
            name: ActiveValue::Set(repository.name.clone()),
            format: ActiveValue::Set(repository.format.id.to_owned()),
            kind: ActiveValue::Set(repository.kind.as_str().to_owned()),
            online: ActiveValue::Set(repository.online),
            blob_store: ActiveValue::Set(repository.blob_store().map(str::to_owned)),
            attributes: ActiveValue::Set(repository.stored_attributes()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };
        repository::Entity::insert(row)
            .exec(&self.txn)
            .await
            .map_err(refusal)?;
        self.txn.commit().await?;
        Ok(repository)
    }

    pub async fn update(
        self,
        name: &str,
        repository: Repository,
    ) -> Result<Repository, WriteError> {
        let existing = self
            .find(name)
            .ok_or_else(|| WriteError::NotFound(name.to_owned()))?;
        unchanged(existing, repository.spec(), Some(&repository.name))?;
        if existing.blob_store() != repository.blob_store() {
            return Err(WriteError::Refused(
                "the blob store of a repository cannot be changed".to_owned(),
            ));
        }
        let mut problems = Problems::default();
        self.check(&repository, &mut problems);
        problems.finish().map_err(WriteError::Invalid)?;
        self.save(&repository).await?;
        self.txn.commit().await?;
        Ok(repository)
    }

    pub async fn delete(self, name: &str) -> Result<Repository, WriteError> {
        let existing = self
            .find(name)
            .cloned()
            .ok_or_else(|| WriteError::NotFound(name.to_owned()))?;
        repository::Entity::delete_by_id(existing.name.clone())
            .exec(&self.txn)
            .await?;
        for group in &self.repositories {
            let mut changed = group.clone();
            if changed.is_named(&existing.name) {
                continue;
            }
            if let Some(members) = changed.attributes.group.as_mut()
                && members.forget(&existing.name)
            {
                self.save(&changed).await?;
            }
        }
        self.txn.commit().await?;
        Ok(existing)
    }

    pub async fn forget_cleanup_policy(
        self,
        name: &str,
    ) -> Result<Option<Vec<String>>, WriteError> {
        let deleted = cleanup_policy::Entity::delete_by_id(name.to_owned())
            .exec(&self.txn)
            .await?;
        if deleted.rows_affected == 0 {
            return Ok(None);
        }
        let mut changed = Vec::new();
        for repository in &self.repositories {
            let Some(names) = repository
                .attributes
                .cleanup
                .as_ref()
                .and_then(|cleanup| cleanup.policy_names.as_ref())
            else {
                continue;
            };
            if !names.iter().any(|known| known == name) {
                continue;
            }
            let mut updated = repository.clone();
            let kept: Vec<String> = names
                .iter()
                .filter(|known| *known != name)
                .cloned()
                .collect();
            updated.attributes.cleanup = match kept.is_empty() {
                true => None,
                false => Some(crate::config::Cleanup {
                    policy_names: Some(kept),
                }),
            };
            self.save(&updated).await?;
            changed.push(repository.name.clone());
        }
        self.txn.commit().await?;
        Ok(Some(changed))
    }

    async fn save(&self, repository: &Repository) -> Result<(), WriteError> {
        repository::Entity::update(repository::ActiveModel {
            name: ActiveValue::Unchanged(repository.name.clone()),
            online: ActiveValue::Set(repository.online),
            attributes: ActiveValue::Set(repository.stored_attributes()),
            updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
            ..Default::default()
        })
        .exec(&self.txn)
        .await?;
        Ok(())
    }

    fn check(&self, candidate: &Repository, problems: &mut Problems) {
        if let Some(blob_store) = candidate.blob_store()
            && !self.blob_stores.contains(blob_store)
        {
            problems.add(
                "blobStoreName",
                format!("the blob store \"{blob_store}\" does not exist"),
            );
        }
        let cleanup = candidate.attributes.cleanup.as_ref();
        for policy in cleanup
            .and_then(|cleanup| cleanup.policy_names.as_ref())
            .into_iter()
            .flatten()
        {
            match self.cleanup_policies.get(policy) {
                None => problems.add(
                    "policyNames",
                    format!("the cleanup policy \"{policy}\" does not exist"),
                ),
                Some(format) if format != "*" && format != candidate.format.id => problems.add(
                    "policyNames",
                    format!(
                        "the cleanup policy \"{policy}\" is for {format} repositories, not {}",
                        candidate.format.id
                    ),
                ),
                Some(_) => {}
            }
        }
        if let Some(rule) = &candidate.attributes.routing_rule_name
            && !self.routing_rules.contains(rule)
        {
            problems.add(
                "routingRuleName",
                format!("the routing rule \"{rule}\" does not exist"),
            );
        }
        self.check_group(candidate, problems);
        self.check_ports(candidate, problems);
        self.check_hosts(candidate, problems);
    }

    fn check_hosts(&self, candidate: &Repository, problems: &mut Problems) {
        let checks: [(&str, &str, HostOf); 2] = [
            ("subdomain", "subdomain", subdomain_of),
            ("registryHost", "registry host", registry_host_of),
        ];
        for (field, label, value) in checks {
            let Some(wanted) = value(candidate) else {
                continue;
            };
            let taken = self.repositories.iter().find(|other| {
                !other.is_named(&candidate.name)
                    && value(other).is_some_and(|used| used.eq_ignore_ascii_case(wanted))
            });
            if let Some(other) = taken {
                problems.add(
                    field,
                    format!(
                        "the {label} {wanted} is used by the repository \"{}\"",
                        other.name
                    ),
                );
            }
        }
    }

    fn check_group(&self, candidate: &Repository, problems: &mut Problems) {
        let Some(group) = candidate.group() else {
            return;
        };
        for member in group.members() {
            if candidate.is_named(member) {
                problems.add(
                    "memberNames",
                    format!("the group \"{}\" cannot be its own member", candidate.name),
                );
                continue;
            }
            match self.find(member) {
                None => problems.add(
                    "memberNames",
                    format!("the repository \"{member}\" does not exist"),
                ),
                Some(found) if found.format != candidate.format => problems.add(
                    "memberNames",
                    format!(
                        "the repository \"{member}\" is a {} repository, not {}",
                        found.format.id, candidate.format.id
                    ),
                ),
                Some(found) if self.reaches(found, &candidate.name) => problems.add(
                    "memberNames",
                    format!(
                        "the group \"{member}\" contains \"{}\", so it cannot be its member",
                        candidate.name
                    ),
                ),
                Some(_) => {}
            }
        }
        if let Some(writable) = &group.writable_member {
            if !group.contains(writable) {
                problems.add("writableMember", "must be one of the members of the group");
            } else if self
                .find(writable)
                .is_some_and(|found| found.kind != RepositoryType::Hosted)
            {
                problems.add("writableMember", "must be a hosted repository");
            }
        }
    }

    fn reaches(&self, from: &Repository, target: &str) -> bool {
        let mut seen = BTreeSet::new();
        let mut pending = vec![from];
        while let Some(repository) = pending.pop() {
            if repository.is_named(target) {
                return true;
            }
            if !seen.insert(repository.name.to_ascii_lowercase()) {
                continue;
            }
            if let Some(group) = repository.group() {
                pending.extend(
                    group
                        .members()
                        .iter()
                        .filter_map(|member| self.find(member)),
                );
            }
        }
        false
    }

    fn check_ports(&self, candidate: &Repository, problems: &mut Problems) {
        let Some(docker) = &candidate.attributes.docker else {
            return;
        };
        for (field, port) in docker.ports() {
            if u16::try_from(port).is_ok_and(|port| self.reserved_ports.contains(&port)) {
                problems.add(
                    field,
                    format!("Port {port} is already used by the application"),
                );
                continue;
            }
            let taken = self.repositories.iter().find(|other| {
                !other.is_named(&candidate.name)
                    && other
                        .attributes
                        .docker
                        .as_ref()
                        .is_some_and(|docker| docker.ports().any(|(_, used)| used == port))
            });
            if let Some(other) = taken {
                problems.add(
                    field,
                    format!(
                        "the port {port} is used by the repository \"{}\"",
                        other.name
                    ),
                );
            }
        }
    }
}

pub fn unchanged(existing: &Repository, spec: Spec, name: Option<&str>) -> Result<(), WriteError> {
    if existing.spec() != spec {
        return Err(WriteError::Refused(format!(
            "the repository \"{}\" is a {} {} repository; its format and type cannot change",
            existing.name, existing.format.id, existing.kind
        )));
    }
    if name.is_some_and(|name| name != existing.name) {
        return Err(WriteError::Refused(
            "a repository cannot be renamed".to_owned(),
        ));
    }
    Ok(())
}

fn refusal(error: DbErr) -> WriteError {
    match error.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => WriteError::Invalid(vec![Problem::new(
            "name",
            "is already used by another repository (names ignore case)",
        )]),
        Some(SqlErr::ForeignKeyConstraintViolation(_)) => WriteError::Invalid(vec![Problem::new(
            "blobStoreName",
            "the blob store does not exist",
        )]),
        _ => error.into(),
    }
}
