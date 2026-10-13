use std::fmt;

use astrum_db::entity::blob_store;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder,
    RuntimeErr, SqlErr, sqlx,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, S3Settings, SoftQuota};

const RESTRICT_VIOLATION: &str = "23001";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoreKind {
    File,
    S3,
    Group,
}

impl StoreKind {
    pub fn as_str(self) -> &'static str {
        match self {
            StoreKind::File => "File",
            StoreKind::S3 => "S3",
            StoreKind::Group => "Group",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "File" => Some(StoreKind::File),
            "S3" => Some(StoreKind::S3),
            "Group" => Some(StoreKind::Group),
            _ => None,
        }
    }
}

impl fmt::Display for StoreKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSettings {
    pub path: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FillPolicy {
    #[default]
    WriteToFirst,
    RoundRobin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSettings {
    pub members: Vec<String>,
    #[serde(default)]
    pub fill_policy: FillPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Settings {
    File(FileSettings),
    S3(Box<S3Settings>),
    Group(GroupSettings),
}

impl Settings {
    pub fn kind(&self) -> StoreKind {
        match self {
            Settings::File(_) => StoreKind::File,
            Settings::S3(_) => StoreKind::S3,
            Settings::Group(_) => StoreKind::Group,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreConfig {
    pub name: String,
    pub settings: Settings,
    pub soft_quota: Option<SoftQuota>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Attributes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file: Option<FileSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    s3: Option<S3Settings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    group: Option<GroupSettings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    soft_quota: Option<SoftQuota>,
}

impl StoreConfig {
    pub fn kind(&self) -> StoreKind {
        self.settings.kind()
    }

    pub fn file(&self) -> Option<&FileSettings> {
        match &self.settings {
            Settings::File(file) => Some(file),
            _ => None,
        }
    }

    pub fn s3(&self) -> Option<&S3Settings> {
        match &self.settings {
            Settings::S3(s3) => Some(s3),
            _ => None,
        }
    }

    pub fn group(&self) -> Option<&GroupSettings> {
        match &self.settings {
            Settings::Group(group) => Some(group),
            _ => None,
        }
    }

    pub fn attributes(&self) -> Value {
        let mut attributes = Attributes {
            soft_quota: self.soft_quota,
            ..Default::default()
        };
        match &self.settings {
            Settings::File(file) => attributes.file = Some(file.clone()),
            Settings::S3(s3) => attributes.s3 = Some((**s3).clone()),
            Settings::Group(group) => attributes.group = Some(group.clone()),
        }
        serde_json::to_value(attributes).expect("blob store attributes always serialize")
    }

    pub fn from_row(row: blob_store::Model) -> Result<Self, Error> {
        let broken = |message: String| Error::Config {
            store: row.name.clone(),
            message,
        };
        let kind = StoreKind::parse(&row.kind)
            .ok_or_else(|| broken(format!("the type \"{}\" is not known", row.kind)))?;
        let attributes: Attributes = serde_json::from_value(row.attributes.clone())
            .map_err(|error| broken(error.to_string()))?;
        let missing = || broken(format!("the {kind} settings are missing"));
        let settings = match kind {
            StoreKind::File => Settings::File(attributes.file.ok_or_else(missing)?),
            StoreKind::S3 => Settings::S3(Box::new(attributes.s3.ok_or_else(missing)?)),
            StoreKind::Group => Settings::Group(attributes.group.ok_or_else(missing)?),
        };
        Ok(Self {
            name: row.name,
            settings,
            soft_quota: attributes.soft_quota,
        })
    }
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<StoreConfig>, Error> {
    blob_store::Entity::find()
        .order_by_asc(blob_store::Column::CreatedAt)
        .order_by_asc(blob_store::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(StoreConfig::from_row)
        .collect()
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<StoreConfig>, Error> {
    blob_store::Entity::find_by_id(name.to_owned())
        .one(db)
        .await?
        .map(StoreConfig::from_row)
        .transpose()
}

pub async fn name_taken(db: &impl ConnectionTrait, name: &str) -> Result<Option<String>, Error> {
    Ok(blob_store::Entity::find()
        .filter(
            Expr::expr(Func::lower(Expr::col(blob_store::Column::Name))).eq(name.to_lowercase()),
        )
        .one(db)
        .await?
        .map(|row| row.name))
}

pub async fn insert(db: &impl ConnectionTrait, config: &StoreConfig) -> Result<(), Error> {
    let now = chrono::Utc::now().fixed_offset();
    let row = blob_store::ActiveModel {
        name: ActiveValue::Set(config.name.clone()),
        kind: ActiveValue::Set(config.kind().as_str().to_owned()),
        attributes: ActiveValue::Set(config.attributes()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    };
    match blob_store::Entity::insert(row).exec(db).await {
        Ok(_) => Ok(()),
        Err(error) if matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => {
            Err(Error::Duplicate(config.name.clone()))
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn update(db: &impl ConnectionTrait, config: &StoreConfig) -> Result<bool, Error> {
    let result = blob_store::Entity::update_many()
        .col_expr(
            blob_store::Column::Attributes,
            Expr::value(config.attributes()),
        )
        .col_expr(
            blob_store::Column::UpdatedAt,
            Expr::value(chrono::Utc::now().fixed_offset()),
        )
        .filter(blob_store::Column::Name.eq(config.name.as_str()))
        .filter(blob_store::Column::Kind.eq(config.kind().as_str()))
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}

pub async fn delete(db: &impl ConnectionTrait, name: &str) -> Result<bool, Error> {
    let in_group = list(db)
        .await?
        .iter()
        .filter_map(StoreConfig::group)
        .any(|group| group.members.iter().any(|member| member == name));
    if in_group {
        return Err(Error::InUse(name.to_owned()));
    }
    match blob_store::Entity::delete_by_id(name.to_owned())
        .exec(db)
        .await
    {
        Ok(result) => Ok(result.rows_affected > 0),
        Err(error) if still_referenced(&error) => Err(Error::InUse(name.to_owned())),
        Err(error) => Err(error.into()),
    }
}

fn still_referenced(error: &DbErr) -> bool {
    if matches!(
        error.sql_err(),
        Some(SqlErr::ForeignKeyConstraintViolation(_))
    ) {
        return true;
    }
    let (DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(database)))
    | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(database)))) = error
    else {
        return false;
    };
    database.code().as_deref() == Some(RESTRICT_VIOLATION)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuotaKind;
    use serde_json::json;

    fn row(kind: &str, attributes: Value) -> blob_store::Model {
        let now = chrono::Utc::now().fixed_offset();
        blob_store::Model {
            name: "store".to_owned(),
            kind: kind.to_owned(),
            attributes,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn a_file_store_keeps_its_path_and_quota() {
        let config = StoreConfig {
            name: "store".to_owned(),
            settings: Settings::File(FileSettings {
                path: "default".to_owned(),
            }),
            soft_quota: Some(SoftQuota {
                kind: QuotaKind::SpaceUsedQuota,
                limit: 10,
            }),
        };
        let attributes = config.attributes();
        assert_eq!(
            attributes,
            json!({"file": {"path": "default"}, "softQuota": {"type": "spaceUsedQuota", "limit": 10}})
        );
        assert_eq!(
            StoreConfig::from_row(row("File", attributes)).unwrap(),
            config
        );
    }

    #[test]
    fn a_group_reads_with_the_default_fill_policy() {
        let config =
            StoreConfig::from_row(row("Group", json!({"group": {"members": ["a", "b"]}}))).unwrap();
        assert_eq!(
            config.group().unwrap(),
            &GroupSettings {
                members: vec!["a".to_owned(), "b".to_owned()],
                fill_policy: FillPolicy::WriteToFirst,
            }
        );
        assert_eq!(
            serde_json::to_value(FillPolicy::RoundRobin).unwrap(),
            json!("roundRobin")
        );
    }

    #[test]
    fn broken_rows_name_the_store() {
        let error = StoreConfig::from_row(row("Tape", json!({}))).unwrap_err();
        assert!(error.to_string().contains("\"store\""), "{error}");
        let error = StoreConfig::from_row(row("S3", json!({"file": {"path": "x"}}))).unwrap_err();
        assert!(error.to_string().contains("S3 settings"), "{error}");
    }
}
