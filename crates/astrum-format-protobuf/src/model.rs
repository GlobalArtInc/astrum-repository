use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::digest;
use crate::proto;

pub const KEY: &str = "protobuf";

pub fn now() -> DateTime<Utc> {
    let now = Utc::now();
    now.with_nanosecond(now.nanosecond() / 1_000 * 1_000)
        .unwrap_or(now)
}

pub fn timestamp(time: DateTime<Utc>) -> proto::Timestamp {
    proto::Timestamp {
        seconds: time.timestamp(),
        nanos: time.timestamp_subsec_nanos() as i32,
    }
}

pub fn attributes<T: Serialize>(meta: &T) -> Value {
    json!({ KEY: meta })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub id: String,
    pub name: String,
    pub created: DateTime<Utc>,
}

impl Owner {
    pub fn to_proto(&self) -> proto::Owner {
        proto::Owner {
            value: Some(proto::owner::Value::Organization(proto::Organization {
                id: self.id.clone(),
                create_time: Some(timestamp(self.created)),
                update_time: Some(timestamp(self.created)),
                name: self.name.clone(),
                description: String::new(),
                url: String::new(),
                verification_status: proto::OrganizationVerificationStatus::Unverified as i32,
                visibility: proto::OrganizationVisibility::Public as i32,
            })),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

impl Visibility {
    pub fn from_proto(value: i32) -> Option<Self> {
        match proto::ModuleVisibility::try_from(value).ok()? {
            proto::ModuleVisibility::Public => Some(Visibility::Public),
            proto::ModuleVisibility::Private => Some(Visibility::Private),
            proto::ModuleVisibility::Unspecified => None,
        }
    }

    pub fn to_proto(self) -> i32 {
        match self {
            Visibility::Public => proto::ModuleVisibility::Public as i32,
            Visibility::Private => proto::ModuleVisibility::Private as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Active,
    Deprecated,
}

impl State {
    pub fn from_proto(value: i32) -> Option<Self> {
        match proto::ModuleState::try_from(value).ok()? {
            proto::ModuleState::Active => Some(State::Active),
            proto::ModuleState::Deprecated => Some(State::Deprecated),
            proto::ModuleState::Unspecified => None,
        }
    }

    pub fn to_proto(self) -> i32 {
        match self {
            State::Active => proto::ModuleState::Active as i32,
            State::Deprecated => proto::ModuleState::Deprecated as i32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: String,
    pub owner_id: String,
    pub owner: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub visibility: Visibility,
    pub state: State,
    pub description: String,
    pub url: String,
    pub default_label: String,
}

impl Module {
    pub fn to_proto(&self) -> proto::Module {
        proto::Module {
            id: self.id.clone(),
            create_time: Some(timestamp(self.created)),
            update_time: Some(timestamp(self.updated)),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            visibility: self.visibility.to_proto(),
            state: self.state.to_proto(),
            description: self.description.clone(),
            url: self.url.clone(),
            default_label_name: self.default_label.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub id: String,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub id: String,
    pub owner_id: String,
    pub module_id: String,
    pub owner: String,
    pub module: String,
    pub created: DateTime<Utc>,
    pub digest: String,
    pub source_control_url: String,
    pub deps: Vec<Dependency>,
    pub direct: Vec<String>,
    pub files: Vec<String>,
}

impl Commit {
    pub fn to_proto(&self) -> proto::Commit {
        proto::Commit {
            id: self.id.clone(),
            create_time: Some(timestamp(self.created)),
            owner_id: self.owner_id.clone(),
            module_id: self.module_id.clone(),
            digest: Some(proto::Digest {
                kind: proto::DigestType::B5 as i32,
                value: digest::parse(&self.digest)
                    .map(|value| value.to_vec())
                    .unwrap_or_default(),
            }),
            created_by_user_id: String::new(),
            source_control_url: self.source_control_url.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Association {
    pub commit: String,
    pub time: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub owner_id: String,
    pub module_id: String,
    pub owner: String,
    pub module: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub archived: Option<DateTime<Utc>>,
    pub commit: String,
}

impl Label {
    pub fn check_state(&self) -> proto::CommitCheckState {
        proto::CommitCheckState {
            status: proto::CommitCheckStatus::Disabled as i32,
            update_time: Some(timestamp(self.updated)),
        }
    }

    pub fn to_proto(&self) -> proto::Label {
        proto::Label {
            id: self.id.clone(),
            create_time: Some(timestamp(self.created)),
            update_time: Some(timestamp(self.updated)),
            archive_time: self.archived.map(timestamp),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            module_id: self.module_id.clone(),
            commit_id: self.commit.clone(),
            updated_by_user_id: String::new(),
            commit_check_state: Some(self.check_state()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelFile {
    #[serde(flatten)]
    pub label: Label,
    pub history: Vec<Association>,
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn derived_commit_id(module_id: &str, digest_hex: &str) -> String {
    use sha2::{Digest as _, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"astrum.protobuf.commit\0");
    hasher.update(module_id.as_bytes());
    hasher.update([0]);
    hasher.update(digest_hex.as_bytes());
    hex::encode(&hasher.finalize()[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_thirty_two_lowercase_hex_characters() {
        let id = new_id();
        assert_eq!(id.len(), 32);
        assert!(
            id.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
        assert_ne!(id, new_id());
    }

    #[test]
    fn a_commit_id_depends_on_the_module_and_the_digest() {
        let one = derived_commit_id("aaaa", "00");
        assert_eq!(one.len(), 32);
        assert_eq!(one, derived_commit_id("aaaa", "00"));
        assert_ne!(one, derived_commit_id("aaab", "00"));
        assert_ne!(one, derived_commit_id("aaaa", "01"));
    }

    #[test]
    fn timestamps_keep_microseconds() {
        let time = DateTime::parse_from_rfc3339("2026-10-22T08:21:55.123456Z")
            .unwrap()
            .with_timezone(&Utc);
        let stamp = timestamp(time);
        assert_eq!(stamp.seconds, time.timestamp());
        assert_eq!(stamp.nanos, 123_456_000);
        assert_eq!(now().nanosecond() % 1_000, 0);
    }

    #[test]
    fn a_label_file_keeps_the_label_fields_beside_the_history() {
        let time = now();
        let file = LabelFile {
            label: Label {
                id: new_id(),
                owner_id: new_id(),
                module_id: new_id(),
                owner: "acme".to_owned(),
                module: "greeter".to_owned(),
                name: "main".to_owned(),
                created: time,
                updated: time,
                archived: None,
                commit: new_id(),
            },
            history: vec![Association {
                commit: new_id(),
                time,
            }],
        };
        let json = serde_json::to_value(&file).unwrap();
        assert_eq!(json["name"], "main");
        assert_eq!(json["history"].as_array().unwrap().len(), 1);
        assert_eq!(serde_json::from_value::<LabelFile>(json).unwrap(), file);
    }
}
