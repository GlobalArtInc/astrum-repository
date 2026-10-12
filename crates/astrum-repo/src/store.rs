use std::collections::BTreeSet;

use astrum_db::entity::{blob_store, repository};
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
    cleanup_policies: BTreeSet<String>,
    routing_rules: BTreeSet<String>,
}

impl Writer {
    pub async fn begin(db: &DatabaseConnection) -> Result<Self, Error> {
        let txn = db.begin().await?;
        txn.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({WRITE_LOCK})"))
            .await?;
        let repositories = list(&txn).await?;
        let blob_stores = blob_store::Entity::find()
            .select_only()
            .column(blob_store::Column::Name)
            .into_tuple::<String>()
            .all(&txn)
            .await?
            .into_iter()
            .collect();
        Ok(Self {
            txn,
            repositories,
            blob_stores,
            cleanup_policies: BTreeSet::new(),
            routing_rules: BTreeSet::new(),
        })
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
            if !self.cleanup_policies.contains(policy) {
                problems.add(
                    "policyNames",
                    format!("the cleanup policy \"{policy}\" does not exist"),
                );
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
