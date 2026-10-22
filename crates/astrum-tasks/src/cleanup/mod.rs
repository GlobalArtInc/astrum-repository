pub mod query;
pub mod task;

use std::collections::BTreeSet;

use astrum_db::entity::cleanup_policy;
use chrono::Utc;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{ActiveValue, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, SqlErr};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::Problems;

pub const ALL_FORMATS: &str = "*";
pub const MAX_DAYS: i64 = 36_500;
pub const MAX_NAME_LENGTH: usize = 255;
pub const MAX_NOTES_LENGTH: usize = 4_000;

pub const LAST_BLOB_UPDATED: &str = "lastBlobUpdated";
pub const LAST_DOWNLOADED: &str = "lastDownloaded";
pub const RELEASE_TYPE: &str = "isPrerelease";
pub const REGEX: &str = "regex";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReleaseType {
    Releases,
    Prereleases,
}

impl ReleaseType {
    pub fn as_str(self) -> &'static str {
        match self {
            ReleaseType::Releases => "RELEASES",
            ReleaseType::Prereleases => "PRERELEASES",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Criteria {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_blob_updated: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_downloaded: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_type: Option<ReleaseType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_regex: Option<String>,
}

impl Criteria {
    pub fn is_empty(&self) -> bool {
        self.last_blob_updated.is_none()
            && self.last_downloaded.is_none()
            && self.release_type.is_none()
            && self.asset_regex.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub name: String,
    pub format: String,
    pub notes: String,
    pub criteria: Criteria,
}

impl Policy {
    fn from_row(row: cleanup_policy::Model) -> Self {
        Self {
            name: row.name,
            format: row.format,
            notes: row.notes,
            criteria: serde_json::from_value(row.criteria).unwrap_or_default(),
        }
    }

    pub fn fits(&self, format: &str) -> bool {
        self.format == ALL_FORMATS || self.format == format
    }

    pub fn document(&self, repositories: &[String]) -> Value {
        json!({
            "name": self.name,
            "format": self.format,
            "notes": self.notes,
            "criteriaLastBlobUpdated": self.criteria.last_blob_updated,
            "criteriaLastDownloaded": self.criteria.last_downloaded,
            "retain": null,
            "sortBy": null,
            "criteriaReleaseType": self.criteria.release_type.map(ReleaseType::as_str),
            "criteriaAssetRegex": self.criteria.asset_regex,
            "inUseCount": repositories.len(),
            "repositories": repositories,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub criteria_last_blob_updated: Option<Value>,
    #[serde(default)]
    pub criteria_last_downloaded: Option<Value>,
    #[serde(default)]
    pub criteria_release_type: Option<Value>,
    #[serde(default)]
    pub criteria_asset_regex: Option<String>,
    #[serde(default)]
    pub retain: Option<Value>,
    #[serde(default)]
    pub sort_by: Option<Value>,
}

pub fn prerelease_rule(format: &str) -> Option<&'static str> {
    match format {
        "alpine" => Some("(c.version ~ '_(alpha|beta|pre|rc)[0-9]*(_|-r[0-9]+$|$)')"),
        "ansible" => Some("(split_part(c.version, '+', 1) LIKE '%-%')"),
        "apt" => Some("(c.version LIKE '%~%')"),
        "cocoapods" => Some("(c.version ~ '[A-Za-z-]')"),
        "maven2" => Some(
            "(c.version LIKE '%-SNAPSHOT' OR c.attributes #>> '{maven2,baseVersion}' LIKE '%-SNAPSHOT')",
        ),
        "bazel" | "hex" | "nuget" | "pub" | "puppet" => {
            Some("(split_part(c.version, '+', 1) LIKE '%-%')")
        }
        "jetbrains" => Some(
            "(COALESCE(c.attributes #>> '{jetbrains,channel}', '') <> '' OR c.version ~* '(^|[^a-z])(alpha|beta|rc|eap|snapshot|nightly|preview|pre|dev)([^a-z]|$)')",
        ),
        "openvsx" => Some(
            "(c.attributes #>> '{openvsx,preRelease}' = 'true' OR split_part(c.version, '+', 1) LIKE '%-%')",
        ),
        "rubygems" | "vagrant" => Some("(c.version ~ '[A-Za-z]')"),
        "yum" => Some(
            "(c.version ~ '~[^-]*-[^-]*$' OR (c.version !~ '-' AND c.version LIKE '%~%') OR c.version ~ '-0\\.[^-]*$')",
        ),
        _ => None,
    }
}

pub fn criteria_of(format: &str) -> Vec<&'static str> {
    if format == ALL_FORMATS {
        return vec![LAST_DOWNLOADED, LAST_BLOB_UPDATED];
    }
    let mut criteria = vec![REGEX];
    if prerelease_rule(format).is_some() {
        criteria.push(RELEASE_TYPE);
    }
    criteria.extend([LAST_DOWNLOADED, LAST_BLOB_UPDATED]);
    criteria
}

fn days(value: Option<&Value>, field: &str, problems: &mut Problems) -> Option<i64> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) if text.trim().is_empty() => None,
        Some(value) => {
            let number = match value {
                Value::Number(number) => number.as_i64(),
                Value::String(text) => text.trim().parse().ok(),
                _ => None,
            };
            match number {
                Some(days) if (1..=MAX_DAYS).contains(&days) => Some(days),
                _ => {
                    problems.add(field, format!("give a number of days from 1 to {MAX_DAYS}"));
                    None
                }
            }
        }
    }
}

pub fn check_regex(pattern: &str) -> Result<(), String> {
    regex::RegexBuilder::new(pattern)
        .size_limit(1 << 20)
        .build()
        .map_err(|error| match error {
            regex::Error::CompiledTooBig(_) => "the regular expression is too large".to_owned(),
            other => format!("the regular expression is not valid: {other}"),
        })?;
    astrum_security::selector::portable_pattern(pattern)
}

pub fn validate(
    request: &PolicyRequest,
    known_formats: &BTreeSet<String>,
    problems: &mut Problems,
) -> Option<Policy> {
    let name = request
        .name
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if name.is_empty() {
        problems.add("name", "give the policy a name");
    } else if !astrum_repo::config::valid_name(&name) {
        problems.add(
            "name",
            "may hold only letters, digits, '-', '_' and '.', and must not start with '_' or '.'",
        );
    } else if name.len() > MAX_NAME_LENGTH {
        problems.add(
            "name",
            format!("must be at most {MAX_NAME_LENGTH} characters long"),
        );
    }
    let format = request
        .format
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if format.is_empty() {
        problems.add("format", "choose a format or all formats");
    } else if format != ALL_FORMATS && !known_formats.contains(&format) {
        problems.add(
            "format",
            format!("{format} is not a format this server knows"),
        );
    }
    let notes = request.notes.clone().unwrap_or_default();
    if notes.chars().count() > MAX_NOTES_LENGTH {
        problems.add(
            "notes",
            format!("must be at most {MAX_NOTES_LENGTH} characters long"),
        );
    }
    let available = criteria_of(&format);
    let mut criteria = Criteria {
        last_blob_updated: days(
            request.criteria_last_blob_updated.as_ref(),
            "criteriaLastBlobUpdated",
            problems,
        ),
        last_downloaded: days(
            request.criteria_last_downloaded.as_ref(),
            "criteriaLastDownloaded",
            problems,
        ),
        ..Criteria::default()
    };
    match request.criteria_release_type.as_ref() {
        None | Some(Value::Null) => {}
        Some(Value::String(text)) if text.trim().is_empty() => {}
        Some(value) => match serde_json::from_value::<ReleaseType>(value.clone()) {
            Ok(_) if !available.contains(&RELEASE_TYPE) => problems.add(
                "criteriaReleaseType",
                format!("{format} does not tell releases from pre-releases"),
            ),
            Ok(release_type) => criteria.release_type = Some(release_type),
            Err(_) => problems.add("criteriaReleaseType", "use RELEASES or PRERELEASES"),
        },
    }
    match request.criteria_asset_regex.as_deref().map(str::trim) {
        None | Some("") => {}
        Some(_) if !available.contains(&REGEX) => problems.add(
            "criteriaAssetRegex",
            "a policy for all formats cannot match asset names; choose a format",
        ),
        Some(pattern) => match check_regex(pattern) {
            Ok(()) => criteria.asset_regex = Some(pattern.to_owned()),
            Err(message) => problems.add("criteriaAssetRegex", message),
        },
    }
    for (field, value) in [("retain", &request.retain), ("sortBy", &request.sort_by)] {
        let given = match value {
            None | Some(Value::Null) => false,
            Some(Value::String(text)) => !text.trim().is_empty(),
            Some(_) => true,
        };
        if given {
            problems.add(
                field,
                "keeping the newest versions is not supported by this server's formats",
            );
        }
    }
    if !problems.is_empty() {
        return None;
    }
    Some(Policy {
        name,
        format,
        notes,
        criteria,
    })
}

pub fn known_formats() -> BTreeSet<String> {
    astrum_repo::FORMATS
        .iter()
        .map(|format| format.id.to_owned())
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("a cleanup policy named \"{0}\" exists already (names ignore case)")]
    Duplicate(String),
    #[error("the cleanup policy \"{0}\" does not exist")]
    NotFound(String),
    #[error("{0}")]
    Refused(String),
    #[error(transparent)]
    Database(#[from] DbErr),
    #[error(transparent)]
    Repositories(#[from] astrum_repo::WriteError),
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Policy>, DbErr> {
    Ok(cleanup_policy::Entity::find()
        .order_by_asc(Expr::cust("\"name\" COLLATE \"C\""))
        .all(db)
        .await?
        .into_iter()
        .map(Policy::from_row)
        .collect())
}

pub async fn find(db: &impl ConnectionTrait, name: &str) -> Result<Option<Policy>, DbErr> {
    Ok(cleanup_policy::Entity::find_by_id(name.to_owned())
        .one(db)
        .await?
        .map(Policy::from_row))
}

pub async fn find_ignoring_case(
    db: &impl ConnectionTrait,
    name: &str,
) -> Result<Option<Policy>, DbErr> {
    Ok(cleanup_policy::Entity::find()
        .filter(
            Expr::expr(Func::lower(Expr::col(cleanup_policy::Column::Name)))
                .eq(name.to_lowercase()),
        )
        .one(db)
        .await?
        .map(Policy::from_row))
}

pub async fn create(db: &impl ConnectionTrait, policy: &Policy) -> Result<(), PolicyError> {
    let now = Utc::now().fixed_offset();
    let row = cleanup_policy::ActiveModel {
        name: ActiveValue::Set(policy.name.clone()),
        format: ActiveValue::Set(policy.format.clone()),
        notes: ActiveValue::Set(policy.notes.clone()),
        criteria: ActiveValue::Set(serde_json::to_value(&policy.criteria).unwrap_or_default()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    };
    cleanup_policy::Entity::insert(row)
        .exec(db)
        .await
        .map_err(|error| match error.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                PolicyError::Duplicate(policy.name.clone())
            }
            _ => PolicyError::Database(error),
        })?;
    Ok(())
}

pub async fn update(db: &impl ConnectionTrait, policy: &Policy) -> Result<(), PolicyError> {
    let row = cleanup_policy::ActiveModel {
        name: ActiveValue::Unchanged(policy.name.clone()),
        format: ActiveValue::Set(policy.format.clone()),
        notes: ActiveValue::Set(policy.notes.clone()),
        criteria: ActiveValue::Set(serde_json::to_value(&policy.criteria).unwrap_or_default()),
        updated_at: ActiveValue::Set(Utc::now().fixed_offset()),
        ..Default::default()
    };
    cleanup_policy::Entity::update(row).exec(db).await?;
    Ok(())
}

pub async fn delete(
    db: &sea_orm::DatabaseConnection,
    name: &str,
) -> Result<Vec<String>, PolicyError> {
    let writer = astrum_repo::Writer::begin(db)
        .await
        .map_err(astrum_repo::WriteError::from)?;
    match writer.forget_cleanup_policy(name).await? {
        Some(changed) => Ok(changed),
        None => Err(PolicyError::NotFound(name.to_owned())),
    }
}

pub async fn users(
    db: &impl ConnectionTrait,
) -> Result<Vec<(String, String, String, Vec<String>)>, astrum_repo::Error> {
    Ok(astrum_repo::store::list(db)
        .await?
        .into_iter()
        .map(|repository| {
            let names = repository
                .attributes
                .cleanup
                .as_ref()
                .and_then(|cleanup| cleanup.policy_names.clone())
                .unwrap_or_default();
            (
                repository.name.clone(),
                repository.format.id.to_owned(),
                repository.kind.as_str().to_owned(),
                names,
            )
        })
        .collect())
}

pub fn repositories_using(
    users: &[(String, String, String, Vec<String>)],
    policy: &str,
) -> Vec<String> {
    users
        .iter()
        .filter(|(_, _, _, names)| names.iter().any(|name| name == policy))
        .map(|(name, _, _, _)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(request: Value) -> Result<Policy, Vec<String>> {
        let request: PolicyRequest = serde_json::from_value(request).unwrap();
        let mut problems = Problems::default();
        let policy = validate(&request, &known_formats(), &mut problems);
        match policy {
            Some(policy) => Ok(policy),
            None => Err(problems
                .into_inner()
                .into_iter()
                .map(|problem| problem.id)
                .collect()),
        }
    }

    #[test]
    fn policies_carry_their_criteria() {
        let policy = check(json!({
            "name": "old-snapshots",
            "format": "maven2",
            "notes": "weekly",
            "criteriaLastBlobUpdated": 30,
            "criteriaLastDownloaded": "60",
            "criteriaReleaseType": "PRERELEASES",
            "criteriaAssetRegex": ".*\\.jar"
        }))
        .unwrap();
        assert_eq!(policy.criteria.last_blob_updated, Some(30));
        assert_eq!(policy.criteria.last_downloaded, Some(60));
        assert_eq!(policy.criteria.release_type, Some(ReleaseType::Prereleases));
        let document = policy.document(&["maven-snapshots".to_owned()]);
        assert_eq!(document["criteriaReleaseType"], "PRERELEASES");
        assert_eq!(document["inUseCount"], 1);
        let all =
            check(json!({"name": "any", "format": "*", "criteriaLastDownloaded": 90})).unwrap();
        assert!(all.fits("npm") && all.fits("raw"));
        let empty = check(json!({"name": "nothing", "format": "raw"})).unwrap();
        assert!(empty.criteria.is_empty());
    }

    #[test]
    fn bad_policies_name_their_fields() {
        for (request, field) in [
            (
                json!({"format": "raw", "criteriaLastBlobUpdated": 1}),
                "name",
            ),
            (json!({"name": "m4 x", "format": "raw"}), "name"),
            (json!({"name": "x", "format": "nope"}), "format"),
            (
                json!({"name": "x", "format": "raw", "criteriaLastBlobUpdated": 0}),
                "criteriaLastBlobUpdated",
            ),
            (
                json!({"name": "x", "format": "raw", "criteriaLastDownloaded": -1}),
                "criteriaLastDownloaded",
            ),
            (
                json!({"name": "x", "format": "raw", "criteriaReleaseType": "PRERELEASES"}),
                "criteriaReleaseType",
            ),
            (
                json!({"name": "x", "format": "maven2", "criteriaReleaseType": "BOGUS"}),
                "criteriaReleaseType",
            ),
            (
                json!({"name": "x", "format": "*", "criteriaAssetRegex": "a.*"}),
                "criteriaAssetRegex",
            ),
            (
                json!({"name": "x", "format": "raw", "criteriaAssetRegex": "(["}),
                "criteriaAssetRegex",
            ),
            (
                json!({"name": "x", "format": "raw", "criteriaAssetRegex": "\\bword"}),
                "criteriaAssetRegex",
            ),
            (json!({"name": "x", "format": "raw", "retain": 3}), "retain"),
        ] {
            assert_eq!(
                check(request.clone()),
                Err(vec![field.to_owned()]),
                "{request}"
            );
        }
    }

    #[test]
    fn formats_offer_what_they_can_evaluate() {
        assert_eq!(criteria_of("*"), [LAST_DOWNLOADED, LAST_BLOB_UPDATED]);
        assert_eq!(
            criteria_of("maven2"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("raw"),
            [REGEX, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("apt"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("alpine"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("cargo"),
            [REGEX, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("cocoapods"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("pub"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("ansible"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("vagrant"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("bazel"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("hex"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("openvsx"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
        assert_eq!(
            criteria_of("jetbrains"),
            [REGEX, RELEASE_TYPE, LAST_DOWNLOADED, LAST_BLOB_UPDATED]
        );
    }
}
