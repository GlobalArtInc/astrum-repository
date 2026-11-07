use astrum_db::entity::content_policy;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, SqlErr,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Off,
    Report,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Malicious {
    #[default]
    Block,
    Threshold,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unscanned {
    #[default]
    Allow,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseList {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnknownLicense {
    Allow,
    #[default]
    Report,
    Block,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VulnerabilityRule {
    pub mode: Mode,
    pub severity: Option<Severity>,
    pub malicious: Malicious,
    pub unscanned: Unscanned,
    pub notify: Option<Severity>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LicenseRule {
    pub mode: Mode,
    pub list: Option<LicenseList>,
    pub ids: Vec<String>,
    pub unknown: UnknownLicense,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SignatureRule {
    pub mode: Mode,
    pub keys: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub name: String,
    pub notes: String,
    pub vulnerabilities: VulnerabilityRule,
    pub licenses: LicenseRule,
    pub signatures: SignatureRule,
    pub revision: i64,
}

impl Policy {
    fn from_row(row: content_policy::Model) -> Result<Self, Error> {
        Ok(Self {
            vulnerabilities: read(&row.name, row.vulnerabilities)?,
            licenses: read(&row.name, row.licenses)?,
            signatures: read(&row.name, row.signatures)?,
            name: row.name,
            notes: row.notes,
            revision: row.revision,
        })
    }
}

fn read<T: DeserializeOwned>(name: &str, value: Value) -> Result<T, Error> {
    serde_json::from_value(value).map_err(|error| Error::Stored {
        name: name.to_owned(),
        reason: error.to_string(),
    })
}

fn json(value: &impl Serialize) -> Value {
    serde_json::to_value(value).expect("the rule blocks serialize to JSON")
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Policy>, Error> {
    content_policy::Entity::find()
        .order_by_asc(Expr::cust("\"name\" COLLATE \"C\""))
        .all(db)
        .await?
        .into_iter()
        .map(Policy::from_row)
        .collect()
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<Policy>, Error> {
    content_policy::Entity::find_by_id(name.to_owned())
        .one(db)
        .await?
        .map(Policy::from_row)
        .transpose()
}

pub async fn create(db: &impl ConnectionTrait, policy: &Policy) -> Result<(), Error> {
    let now = Utc::now().fixed_offset();
    let row = content_policy::ActiveModel {
        name: ActiveValue::Set(policy.name.clone()),
        notes: ActiveValue::Set(policy.notes.clone()),
        vulnerabilities: ActiveValue::Set(json(&policy.vulnerabilities)),
        licenses: ActiveValue::Set(json(&policy.licenses)),
        signatures: ActiveValue::Set(json(&policy.signatures)),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
        ..Default::default()
    };
    content_policy::Entity::insert(row)
        .exec(db)
        .await
        .map_err(|error| match error.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                Error::Duplicate(format!("the content policy \"{}\"", policy.name))
            }
            _ => Error::Database(error),
        })?;
    Ok(())
}

pub async fn update(db: &impl ConnectionTrait, policy: &Policy) -> Result<(), Error> {
    let changed = content_policy::Entity::update_many()
        .col_expr(
            content_policy::Column::Notes,
            Expr::value(policy.notes.clone()),
        )
        .col_expr(
            content_policy::Column::Vulnerabilities,
            Expr::value(json(&policy.vulnerabilities)),
        )
        .col_expr(
            content_policy::Column::Licenses,
            Expr::value(json(&policy.licenses)),
        )
        .col_expr(
            content_policy::Column::Signatures,
            Expr::value(json(&policy.signatures)),
        )
        .col_expr(
            content_policy::Column::Revision,
            Expr::col(content_policy::Column::Revision).add(1),
        )
        .col_expr(
            content_policy::Column::UpdatedAt,
            Expr::value(Utc::now().fixed_offset()),
        )
        .filter(content_policy::Column::Name.eq(policy.name.as_str()))
        .exec(db)
        .await?;
    if changed.rows_affected == 0 {
        return Err(Error::NotFound(format!(
            "the content policy \"{}\"",
            policy.name
        )));
    }
    Ok(())
}

pub async fn delete(db: &impl ConnectionTrait, name: &str) -> Result<bool, Error> {
    let deleted = content_policy::Entity::delete_by_id(name.to_owned())
        .exec(db)
        .await?;
    Ok(deleted.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_the_defaults_of_the_design() {
        let vulnerabilities: VulnerabilityRule =
            serde_json::from_str(r#"{"mode":"report"}"#).unwrap();
        assert_eq!(vulnerabilities.mode, Mode::Report);
        assert_eq!(vulnerabilities.malicious, Malicious::Block);
        assert_eq!(vulnerabilities.unscanned, Unscanned::Allow);
        let licenses: LicenseRule = serde_json::from_str("{}").unwrap();
        assert_eq!(licenses.mode, Mode::Off);
        assert_eq!(licenses.unknown, UnknownLicense::Report);
    }
}
