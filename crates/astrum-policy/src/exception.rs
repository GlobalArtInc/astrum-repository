use astrum_db::entity::policy_exception;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, SqlErr,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rule {
    Vulnerability,
    License,
    Signature,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::Vulnerability => "vulnerability",
            Rule::License => "license",
            Rule::Signature => "signature",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "vulnerability" => Some(Rule::Vulnerability),
            "license" => Some(Rule::License),
            "signature" => Some(Rule::Signature),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    pub id: Uuid,
    pub policy: Option<String>,
    pub rule: Rule,
    pub advisory: Option<String>,
    pub license: Option<String>,
    pub format: Option<String>,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

impl Exception {
    fn from_row(row: policy_exception::Model) -> Result<Self, Error> {
        let rule = Rule::parse(&row.rule).ok_or_else(|| Error::Stored {
            name: row.id.to_string(),
            reason: format!("the rule \"{}\" is unknown", row.rule),
        })?;
        Ok(Self {
            id: row.id,
            policy: row.policy,
            rule,
            advisory: row.advisory,
            license: row.license,
            format: row.format,
            namespace: row.namespace,
            name: row.name,
            version: row.version,
            digest: row.digest,
            reason: row.reason,
            expires_at: row.expires_at.map(|at| at.with_timezone(&Utc)),
            created_by: row.created_by,
            created_at: row.created_at.with_timezone(&Utc),
        })
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Exception>, Error> {
    policy_exception::Entity::find()
        .order_by_asc(policy_exception::Column::CreatedAt)
        .order_by_asc(policy_exception::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .map(Exception::from_row)
        .collect()
}

pub async fn find(db: &impl ConnectionTrait, id: Uuid) -> Result<Option<Exception>, Error> {
    policy_exception::Entity::find_by_id(id)
        .one(db)
        .await?
        .map(Exception::from_row)
        .transpose()
}

fn validate(exception: &Exception) -> Result<(), Error> {
    if exception.reason.trim().is_empty() {
        return Err(Error::Invalid(
            "the reason of an exception must not be empty".to_owned(),
        ));
    }
    let targets = [
        ("advisory", &exception.advisory),
        ("license", &exception.license),
        ("format", &exception.format),
        ("namespace", &exception.namespace),
        ("name", &exception.name),
        ("version", &exception.version),
        ("digest", &exception.digest),
    ];
    for (field, value) in targets {
        if value.as_deref().is_some_and(|text| text.trim().is_empty()) {
            return Err(Error::Invalid(format!(
                "the target {field} of an exception must not be empty"
            )));
        }
    }
    let identified = [
        &exception.advisory,
        &exception.license,
        &exception.name,
        &exception.digest,
    ]
    .iter()
    .any(|value| value.is_some());
    if !identified {
        return Err(Error::Invalid(
            "an exception needs a target: an advisory, a license, a name or a digest".to_owned(),
        ));
    }
    Ok(())
}

pub async fn create(db: &impl ConnectionTrait, exception: &Exception) -> Result<(), Error> {
    validate(exception)?;
    let row = policy_exception::ActiveModel {
        id: ActiveValue::Set(exception.id),
        policy: ActiveValue::Set(exception.policy.clone()),
        rule: ActiveValue::Set(exception.rule.as_str().to_owned()),
        advisory: ActiveValue::Set(exception.advisory.clone()),
        license: ActiveValue::Set(exception.license.clone()),
        format: ActiveValue::Set(exception.format.clone()),
        namespace: ActiveValue::Set(exception.namespace.clone()),
        name: ActiveValue::Set(exception.name.clone()),
        version: ActiveValue::Set(exception.version.clone()),
        digest: ActiveValue::Set(exception.digest.clone()),
        reason: ActiveValue::Set(exception.reason.clone()),
        expires_at: ActiveValue::Set(exception.expires_at.map(|at| at.fixed_offset())),
        created_by: ActiveValue::Set(exception.created_by.clone()),
        created_at: ActiveValue::Set(exception.created_at.fixed_offset()),
    };
    policy_exception::Entity::insert(row)
        .exec(db)
        .await
        .map_err(|error| match error.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                Error::Duplicate(format!("the policy exception {}", exception.id))
            }
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => Error::NotFound(format!(
                "the content policy \"{}\"",
                exception.policy.as_deref().unwrap_or_default()
            )),
            _ => Error::Database(error),
        })?;
    Ok(())
}

pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<bool, Error> {
    let deleted = policy_exception::Entity::delete_many()
        .filter(policy_exception::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(deleted.rows_affected > 0)
}
