use astrum_db::entity::content_policy;
use chrono::Utc;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, SqlErr,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{Error, license};

pub const MAX_LICENSE_IDS: usize = 200;
pub const MAX_LICENSE_ID: usize = 100;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseList {
    Allow,
    #[default]
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
    #[serde(deserialize_with = "list_or_default")]
    pub list: LicenseList,
    pub ids: Vec<String>,
    pub unknown: UnknownLicense,
}

fn list_or_default<'de, D: Deserializer<'de>>(deserializer: D) -> Result<LicenseList, D::Error> {
    Ok(Option::<LicenseList>::deserialize(deserializer)?.unwrap_or_default())
}

impl LicenseRule {
    pub fn check(&self) -> Result<(), String> {
        if self.ids.len() > MAX_LICENSE_IDS {
            return Err(format!("must hold at most {MAX_LICENSE_IDS} ids"));
        }
        for id in &self.ids {
            if id.chars().count() > MAX_LICENSE_ID {
                return Err(format!(
                    "every id must be at most {MAX_LICENSE_ID} characters long"
                ));
            }
            if !license::is_identifier(id) {
                return Err(format!(
                    "{id:?} is not a license identifier; use an SPDX id, a deprecated SPDX id or a LicenseRef-* id"
                ));
            }
        }
        if self.mode == Mode::Block && self.list == LicenseList::Allow && self.ids.is_empty() {
            return Err(
                "must hold at least one id when the mode is block and the list is allow".to_owned(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SignatureRule {
    pub mode: Mode,
    pub keys: Vec<String>,
}

const COOLDOWN_DEFAULT_DAYS: u32 = 7;
const COOLDOWN_MAX_DAYS: u32 = 90;
const COOLDOWN_MAX_EXCLUDE: usize = 100;
const COOLDOWN_MAX_PATTERN: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "CooldownWire")]
pub struct CooldownRule {
    pub mode: Mode,
    pub min_age_days: u32,
    pub exclude: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
struct CooldownWire {
    mode: Mode,
    min_age_days: Option<u32>,
    exclude: Vec<String>,
}

impl Default for CooldownRule {
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            min_age_days: COOLDOWN_DEFAULT_DAYS,
            exclude: Vec::new(),
        }
    }
}

impl TryFrom<CooldownWire> for CooldownRule {
    type Error = String;

    fn try_from(wire: CooldownWire) -> Result<Self, Self::Error> {
        let min_age_days = wire.min_age_days.unwrap_or(COOLDOWN_DEFAULT_DAYS);
        if !(1..=COOLDOWN_MAX_DAYS).contains(&min_age_days) {
            return Err(format!(
                "minAgeDays must be from 1 to {COOLDOWN_MAX_DAYS}, not {min_age_days}"
            ));
        }
        if wire.exclude.len() > COOLDOWN_MAX_EXCLUDE {
            return Err(format!(
                "exclude takes at most {COOLDOWN_MAX_EXCLUDE} patterns"
            ));
        }
        if wire
            .exclude
            .iter()
            .any(|pattern| pattern.is_empty() || pattern.chars().count() > COOLDOWN_MAX_PATTERN)
        {
            return Err(format!(
                "an exclude pattern must be from 1 to {COOLDOWN_MAX_PATTERN} characters"
            ));
        }
        Ok(Self {
            mode: wire.mode,
            min_age_days,
            exclude: wire.exclude,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub name: String,
    pub notes: String,
    pub vulnerabilities: VulnerabilityRule,
    pub licenses: LicenseRule,
    pub signatures: SignatureRule,
    pub cooldown: CooldownRule,
    pub revision: i64,
}

impl Policy {
    pub fn judged_mode(&self) -> Mode {
        match (self.vulnerabilities.mode, self.licenses.mode) {
            (Mode::Block, _) | (_, Mode::Block) => Mode::Block,
            (Mode::Report, _) | (_, Mode::Report) => Mode::Report,
            _ => Mode::Off,
        }
    }

    fn from_row(row: content_policy::Model) -> Result<Self, Error> {
        Ok(Self {
            vulnerabilities: read(&row.name, row.vulnerabilities)?,
            licenses: read(&row.name, row.licenses)?,
            signatures: read(&row.name, row.signatures)?,
            cooldown: read(&row.name, row.cooldown)?,
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

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub revision: i64,
    pub policy: Result<Policy, String>,
}

pub async fn entries(db: &impl ConnectionTrait) -> Result<Vec<Entry>, Error> {
    let rows = content_policy::Entity::find()
        .order_by_asc(Expr::cust("\"name\" COLLATE \"C\""))
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let (name, revision) = (row.name.clone(), row.revision);
            Entry {
                name,
                revision,
                policy: Policy::from_row(row).map_err(|error| error.to_string()),
            }
        })
        .collect())
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
        cooldown: ActiveValue::Set(json(&policy.cooldown)),
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
            content_policy::Column::Cooldown,
            Expr::value(json(&policy.cooldown)),
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
        assert_eq!(licenses.list, LicenseList::Deny);
        assert!(licenses.ids.is_empty());
        assert_eq!(licenses.unknown, UnknownLicense::Report);
    }

    #[test]
    fn a_license_block_round_trips_and_an_older_null_list_reads_as_deny() {
        let rule = LicenseRule {
            mode: Mode::Block,
            list: LicenseList::Allow,
            ids: vec!["MIT".to_owned(), "LicenseRef-Own".to_owned()],
            unknown: UnknownLicense::Block,
        };
        let text = serde_json::to_string(&rule).unwrap();
        assert_eq!(
            text,
            r#"{"mode":"block","list":"allow","ids":["MIT","LicenseRef-Own"],"unknown":"block"}"#
        );
        assert_eq!(serde_json::from_str::<LicenseRule>(&text).unwrap(), rule);
        let older: LicenseRule =
            serde_json::from_str(r#"{"mode":"report","list":null,"ids":[],"unknown":"report"}"#)
                .unwrap();
        assert_eq!(older.list, LicenseList::Deny);
    }

    fn rule(mode: Mode, list: LicenseList, ids: &[&str]) -> LicenseRule {
        LicenseRule {
            mode,
            list,
            ids: ids.iter().map(|id| (*id).to_owned()).collect(),
            unknown: UnknownLicense::Report,
        }
    }

    #[test]
    fn the_ids_of_a_license_block_are_bounded_and_must_be_license_identifiers() {
        let many = vec!["MIT"; MAX_LICENSE_IDS];
        assert!(rule(Mode::Report, LicenseList::Deny, &many).check().is_ok());
        let too_many = vec!["MIT"; MAX_LICENSE_IDS + 1];
        let refused = rule(Mode::Report, LicenseList::Deny, &too_many).check();
        assert!(refused.unwrap_err().contains("at most 200 ids"));

        let long = format!("LicenseRef-{}", "a".repeat(MAX_LICENSE_ID));
        let refused = rule(Mode::Report, LicenseList::Deny, &[long.as_str()]).check();
        assert!(refused.unwrap_err().contains("at most 100 characters"));
        let longest = format!("LicenseRef-{}", "a".repeat(MAX_LICENSE_ID - 11));
        assert!(
            rule(Mode::Report, LicenseList::Deny, &[longest.as_str()])
                .check()
                .is_ok()
        );

        for accepted in ["MIT", "Apache-2.0", "GPL-2.0", "LicenseRef-Own"] {
            assert!(
                rule(Mode::Block, LicenseList::Allow, &[accepted])
                    .check()
                    .is_ok(),
                "{accepted}"
            );
        }
        for refused in ["Not-A-License", "GPLv3", "MIT OR ISC", "MIT License", ""] {
            let message = rule(Mode::Report, LicenseList::Deny, &["MIT", refused])
                .check()
                .unwrap_err();
            assert!(message.contains("is not a license identifier"), "{refused}");
        }
        assert!(rule(Mode::Block, LicenseList::Allow, &[]).check().is_err());
        assert!(rule(Mode::Block, LicenseList::Deny, &[]).check().is_ok());
        assert!(rule(Mode::Report, LicenseList::Allow, &[]).check().is_ok());
    }

    #[test]
    fn the_strictest_mode_of_the_judged_rules_decides_what_the_gate_does() {
        let mut policy = Policy {
            name: "p".to_owned(),
            notes: String::new(),
            vulnerabilities: VulnerabilityRule::default(),
            licenses: LicenseRule::default(),
            signatures: SignatureRule {
                mode: Mode::Block,
                keys: Vec::new(),
            },
            cooldown: CooldownRule::default(),
            revision: 1,
        };
        assert_eq!(policy.judged_mode(), Mode::Off);
        policy.licenses.mode = Mode::Report;
        assert_eq!(policy.judged_mode(), Mode::Report);
        policy.vulnerabilities.mode = Mode::Block;
        assert_eq!(policy.judged_mode(), Mode::Block);
        policy.vulnerabilities.mode = Mode::Off;
        policy.licenses.mode = Mode::Block;
        assert_eq!(policy.judged_mode(), Mode::Block);
    }

    #[test]
    fn unknown_fields_of_a_newer_row_are_ignored_on_read() {
        let vulnerabilities: VulnerabilityRule =
            serde_json::from_str(r#"{"mode":"block","cooldown":{"days":3}}"#).unwrap();
        assert_eq!(vulnerabilities.mode, Mode::Block);
        let licenses: LicenseRule = serde_json::from_str(r#"{"mode":"report","extra":1}"#).unwrap();
        assert_eq!(licenses.mode, Mode::Report);
        let signatures: SignatureRule =
            serde_json::from_str(r#"{"keys":["a"],"later":[]}"#).unwrap();
        assert_eq!(signatures.keys, ["a"]);
    }

    #[test]
    fn a_cooldown_rule_round_trips_and_defaults_to_seven_days() {
        let rule: CooldownRule = serde_json::from_str(r#"{"mode":"block"}"#).unwrap();
        assert_eq!(
            rule,
            CooldownRule {
                mode: Mode::Block,
                min_age_days: 7,
                exclude: Vec::new(),
            }
        );
        let json = serde_json::to_value(&rule).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"mode": "block", "minAgeDays": 7, "exclude": []})
        );
        let back: CooldownRule = serde_json::from_value(json).unwrap();
        assert_eq!(back, rule);
    }

    #[test]
    fn a_cooldown_rule_refuses_unknown_fields_and_days_out_of_range() {
        assert!(serde_json::from_str::<CooldownRule>(r#"{"mode":"block","days":3}"#).is_err());
        for days in [0, 91] {
            let body = format!(r#"{{"mode":"block","minAgeDays":{days}}}"#);
            assert!(
                serde_json::from_str::<CooldownRule>(&body).is_err(),
                "{body}"
            );
        }
        let body = r#"{"mode":"block","minAgeDays":90}"#;
        assert!(serde_json::from_str::<CooldownRule>(body).is_ok());
    }

    #[test]
    fn a_cooldown_rule_caps_its_exclude_list_and_the_length_of_its_patterns() {
        let many = serde_json::json!({"mode": "report", "exclude": vec!["a"; 101]});
        assert!(serde_json::from_value::<CooldownRule>(many).is_err());
        let long = serde_json::json!({"exclude": ["a".repeat(201)]});
        assert!(serde_json::from_value::<CooldownRule>(long).is_err());
        let edge = serde_json::json!({"exclude": ["a".repeat(200)]});
        assert!(serde_json::from_value::<CooldownRule>(edge).is_ok());
    }
}
